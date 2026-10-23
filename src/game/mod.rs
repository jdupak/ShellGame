//! The game engine: course types, preparing a level, judging work and strict `cd` rules.
//!
//! A level is a plain `const` literal in `crate::levels`; this module is everything that gives its
//! fields meaning. The runtime API, all keyed by [`Pos`] (`"S.L"`, levels counted from 1):
//!
//! * [`prepare`]: clear markers, repair the section root, apply the level's setup;
//! * [`start_dir`]: where the player is placed (the section root unless the level says otherwise);
//! * [`Ctx::new`]: what judging needs to know (workspace, section root, canonical cwd, logical home);
//! * [`Level::judge`]: the level's checks in order, then its answer; `Err` is the text to show;
//! * [`cd::before`] and [`cd::after`]: the strict-`cd` hook around every `cd`, with a [`CdMove`];
//! * [`dedent`]: texts, intros and solutions are indented raw strings.
//!
//! The authoring vocabulary (`exact`, `at`, `file`, `cd::exact`, ...) lives in the submodules and is
//! re-exported here as one-line `const fn`s; the tables are `#[rustfmt::skip]` so they read as such.
//!
//! Evidence the shell witnesses lives in marker files in the workspace root: [`PWD_USED`] (written
//! by the shell wrapper), [`CD_DONE`] and custom `.cd_*` tracking files (`Ctx::marker`). Preparing
//! any level removes all of them, so evidence always belongs to the level being played.

pub mod answer;
pub mod cd;
pub mod check;
pub mod messages;
pub mod paths;
pub mod setup;

use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub use answer::*;
pub use cd::CdMove;
pub use check::*;
pub use setup::*;

use crate::levels::COURSE;

pub const PWD_USED: &str = ".pwd_used";
pub const CD_DONE: &str = ".cd_done";

#[derive(Clone, Copy)]
pub struct Section {
    /// Without a number: the UI adds "Sekce N".
    pub title: &'static str,
    /// Markdown pages separated by a line `---`, shown when entering the section and never saved.
    pub intro: &'static str,
    pub levels: &'static [Level],
}

#[derive(Clone, Copy)]
pub struct Level {
    pub title: &'static str,
    /// Markdown, an indented raw string (see [`dedent`]).
    pub text: &'static str,
    /// Plain text, shown one at a time.
    pub hints: &'static [&'static str],
    /// The player's start directory relative to the section root.
    pub start: &'static str,
    pub setup: &'static [Setup],
    pub answer: Answer,
    pub check: &'static [Check],
    pub cd: &'static [cd::Cd],
    /// A shell script that completes the level, an indented raw string (see [`dedent`]).
    pub solution: &'static str,
    /// One line, or an indented raw string (see [`dedent`]).
    pub success: &'static str,
}

/// The defaults: `Level { title: "..", ..LEVEL }`.
pub const LEVEL: Level = Level {
    title: "",
    text: "",
    hints: &[],
    start: ".",
    setup: &[],
    answer: NO_ANSWER,
    check: &[],
    cd: &[],
    solution: "",
    success: messages::CORRECT,
};

impl Level {
    /// The checks first, in order (the first failure wins), then the answer. Without an answer
    /// (`None`) the level passes when it takes none. A workspace or section directory that is a
    /// symlink is nobody's fault to judge: `PATH_ESCAPES_WORKSPACE`.
    pub fn judge(&self, ctx: &Ctx, answer: Option<&str>) -> Result<(), String> {
        ctx.path("", false)?;
        self.check.iter().try_for_each(|check| check.run(ctx))?;
        self.answer.judge(answer)
    }
}

/// A position in the course: `"S.L"`, section from 0 (the welcome), level from 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pos {
    pub section: usize,
    pub level: usize,
}

impl Pos {
    pub fn section(self) -> &'static Section {
        &COURSE[self.section]
    }

    pub fn level(self) -> &'static Level {
        &self.section().levels[self.level - 1]
    }

    /// Every position, in course order.
    pub fn all() -> impl Iterator<Item = Pos> {
        COURSE
            .iter()
            .enumerate()
            .flat_map(|(section, s)| (1..=s.levels.len()).map(move |level| Pos { section, level }))
    }

    pub fn next(self) -> Option<Pos> {
        Pos::all().skip_while(|pos| *pos != self).nth(1)
    }

    /// `level-S` below the workspace.
    pub fn section_dir(self, workspace: &Path) -> PathBuf {
        paths::join(workspace, &self.rel())
    }

    fn rel(self) -> String {
        format!("level-{}", self.section)
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.section, self.level)
    }
}

impl FromStr for Pos {
    type Err = String;

    /// `"S.L"`, or `"S"` for the first level of a section.
    fn from_str(text: &str) -> Result<Pos, String> {
        let (section, level) = text.split_once('.').unwrap_or((text, "1"));
        let number = |digits: &str| {
            let valid = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
            valid.then(|| digits.parse().ok()).flatten()
        };
        let pos = number(section).zip(number(level)).map(|(section, level)| Pos { section, level });
        pos.filter(|pos| Pos::all().any(|known| known == *pos))
            .ok_or_else(|| format!("neznámá pozice {text:?} (očekávám S.L, například 3.2)"))
    }
}

/// What the player of a level sees around the files: who, where and what the shell reported.
#[derive(Clone, Debug)]
pub struct Ctx<'a> {
    pub workspace: &'a Path,
    pub section_root: PathBuf,
    /// The player's directory, canonical; `None` when it no longer exists.
    pub cwd: Option<PathBuf>,
    /// The logical `$HOME`.
    pub home: &'a Path,
}

impl<'a> Ctx<'a> {
    pub fn new(workspace: &'a Path, pos: Pos, cwd: Option<PathBuf>, home: &'a Path) -> Self {
        Self { workspace, section_root: pos.section_dir(workspace), cwd, home }
    }

    /// A marker file in the workspace root; custom ones are named `.cd_*`.
    pub fn marker(&self, name: &str) -> PathBuf {
        self.workspace.join(name)
    }

    /// Write a marker, never through a symlink the player left in its place.
    pub fn mark(&self, name: &str, content: &str) -> io::Result<()> {
        let flags = rustix::fs::OFlags::NOFOLLOW.bits() as i32;
        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true).custom_flags(flags);
        options.open(self.marker(name))?.write_all(content.as_bytes())
    }

    /// A declared path below the section root; refused (with the player's text) when it passes
    /// through a symlink.
    fn path(&self, rel: &str, leaf_link_ok: bool) -> Result<PathBuf, String> {
        paths::resolve(&self.section_root, rel, leaf_link_ok)
            .map_err(|_| messages::PATH_ESCAPES_WORKSPACE.to_owned())
    }
}

/// Put the level into its starting state: remove all markers, repair the section root (a file or a
/// symlink in its place is removed, a directory the player locked is made usable) and apply the
/// setup. Refuses a workspace that is a symlink; the caller makes sure it is a usable directory.
pub fn prepare(pos: Pos, workspace: &Path) -> io::Result<()> {
    let root = paths::resolve(workspace, &pos.rel(), true)?;
    // A workspace that does not exist yet has no markers.
    for entry in fs::read_dir(workspace).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == PWD_USED || name.starts_with(".cd_") {
            remove_tree(&entry.path())?;
        }
    }
    setup::ensure_dir(&root)?;
    setup::apply(&root, pos.level().setup)
}

/// Where the player is placed when the level starts.
pub fn start_dir(pos: Pos, workspace: &Path) -> PathBuf {
    paths::join(&pos.section_dir(workspace), pos.level().start)
}

/// Undo the indentation of a raw string written in code: its first line (after the opening
/// delimiter) is empty, its last holds only the closing delimiter's indentation, and the rest
/// loses the spaces they all share. A string that does not start with a newline stays as it is.
pub fn dedent(raw: &str) -> String {
    let Some(body) = raw.strip_prefix('\n') else {
        return raw.to_owned();
    };
    let mut lines: Vec<&str> = body.split('\n').collect();
    if lines.last().is_some_and(|last| last.trim().is_empty()) {
        lines.pop();
    }
    let indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start_matches(' ').len())
        .min()
        .unwrap_or(0);
    let lines: Vec<&str> = lines.iter().map(|line| line.get(indent..).unwrap_or("")).collect();
    lines.join("\n")
}

/// The content a `file` writes and a `content_is` expects: an indented block (it starts with a
/// newline) is dedented and ends with a newline like a text file; any other text is as it is.
pub(crate) fn file_text(text: &str) -> String {
    if text.starts_with('\n') { dedent(text) + "\n" } else { text.to_owned() }
}

#[cfg(test)]
pub(crate) mod testing {
    use super::*;

    /// A failure with this text.
    pub fn err(message: &str) -> Result<(), String> {
        Err(message.to_owned())
    }

    /// A workspace (its path has a space, as in real play) with section 1's directory and a home.
    pub struct World {
        _tmp: tempfile::TempDir,
        pub ws: PathBuf,
        pub home: PathBuf,
    }

    impl World {
        pub fn new() -> World {
            let tmp = tempfile::tempdir().unwrap();
            let base = fs::canonicalize(tmp.path()).unwrap();
            let (ws, home) = (base.join("shellgame workspace"), base.join("home/ada"));
            fs::create_dir_all(ws.join("level-1")).unwrap();
            fs::create_dir_all(&home).unwrap();
            World { ws, home, _tmp: tmp }
        }

        pub fn root(&self) -> PathBuf {
            self.ws.join("level-1")
        }

        /// The context of section 1 with the player at `cwd`.
        pub fn ctx(&self, cwd: Option<PathBuf>) -> Ctx<'_> {
            Ctx::new(&self.ws, Pos { section: 1, level: 1 }, cwd, &self.home)
        }

        /// Write a file below section 1's directory.
        pub fn put(&self, rel: &str, content: &str) {
            let path = self.root().join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        }

        /// Run a check with the player at section 1's directory.
        pub fn run(&self, check: Check) -> Result<(), String> {
            check.run(&self.ctx(Some(self.root())))
        }

        /// Prepare a level of the course and put the player at its start.
        pub fn start(&self, pos: Pos) -> Ctx<'_> {
            prepare(pos, &self.ws).unwrap();
            let start = start_dir(pos, &self.ws);
            assert!(start.is_dir(), "{pos}: the start {start:?} must exist");
            Ctx::new(&self.ws, pos, Some(start), &self.home)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use super::testing::World;
    use super::*;

    #[test]
    fn positions_parse_print_and_walk_the_course() {
        let all: Vec<Pos> = Pos::all().collect();
        assert_eq!(all[0], Pos { section: 0, level: 1 });
        for (index, pos) in all.iter().enumerate() {
            assert_eq!(pos.to_string().parse(), Ok(*pos));
            assert_eq!(pos.next(), all.get(index + 1).copied());
        }
        assert_eq!("2".parse(), Ok(Pos { section: 2, level: 1 }));
        assert!(
            ["", "x", "1.", ".1", "1.0", "1.2.3", "+1.1", " 1.1", "99", "1.99"]
                .iter()
                .all(|bad| bad.parse::<Pos>().is_err())
        );
    }

    #[test]
    fn indented_raw_strings_are_dedented() {
        assert_eq!(dedent("\n    a\n\n      b\n  "), "a\n\n  b");
        assert_eq!(dedent("\n    a\n      \n    b\n    "), "a\n  \nb");
        assert_eq!([dedent("\n    x\n    "), dedent("one line")], ["x", "one line"]);
    }

    #[test]
    fn preparing_removes_every_marker_and_never_follows_a_symlink() {
        let (w, outside) = (World::new(), tempfile::tempdir().unwrap());
        // A level of section 1 that sets files up, so that writing through a link would show.
        let sets_up = |pos: &Pos| pos.section == 1 && !pos.level().setup.is_empty();
        let (pos, ws) = (Pos::all().find(sets_up).unwrap(), w.ws.clone());
        let markers = [PWD_USED, CD_DONE, ".cd_custom"];
        for name in markers.iter().chain(&[".not-a-marker"]) {
            fs::write(ws.join(name), "").unwrap();
        }
        fs::create_dir(ws.join(".cd_a_directory")).unwrap();
        prepare(pos, &ws).unwrap();
        assert!(markers.iter().all(|name| !ws.join(name).exists()));
        assert!(!ws.join(".cd_a_directory").exists() && ws.join(".not-a-marker").exists());

        // A marker the player replaced by a link is never written through, and preparing removes it.
        let precious = outside.path().join("precious");
        fs::write(&precious, "mine").unwrap();
        symlink(&precious, ws.join(CD_DONE)).unwrap();
        assert!(w.ctx(None).mark(CD_DONE, "x").is_err());
        prepare(pos, &ws).unwrap();
        assert_eq!(
            (fs::read_to_string(&precious).unwrap(), ws.join(CD_DONE).exists()),
            ("mine".into(), false)
        );

        // A symlinked workspace is refused; a symlinked section directory is replaced, and nothing is
        // written through either.
        symlink(&ws, ws.with_file_name("link")).unwrap();
        assert!(prepare(pos, &ws.with_file_name("link")).is_err());
        fs::remove_dir_all(w.root()).unwrap();
        symlink(outside.path(), w.root()).unwrap();
        prepare(pos, &ws).unwrap();
        assert!(!w.root().is_symlink() && w.root().is_dir());
        assert_eq!(
            fs::read_dir(outside.path()).unwrap().count(),
            1,
            "nothing was written through the link"
        );
        // Judging never follows one either.
        fs::remove_dir_all(w.root()).unwrap();
        symlink(outside.path(), w.root()).unwrap();
        const ANY: Level = Level { answer: exact("a"), ..LEVEL };
        let refused = ANY.judge(&w.ctx(None), Some("a"));
        assert_eq!(refused, Err(messages::PATH_ESCAPES_WORKSPACE.into()));
    }
}
