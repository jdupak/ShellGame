//! `state.json`, version 3: where the player is and how each visited level went. Positions only
//! (`"S.L"`), no level content. A file of an older version is set aside as `state-old.json`; one
//! of a newer version is an error and is left alone.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::game::Pos;

pub const FILE: &str = "state.json";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Progress {
    /// Unix seconds when the level was first shown.
    pub started: u64,
    pub done: Option<u64>,
    pub attempts: u32,
    pub hints: u32,
}

impl Progress {
    pub fn seconds(&self) -> u64 {
        self.done.map_or(0, |done| done.saturating_sub(self.started))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    version: u32,
    pub workspace: PathBuf,
    /// A position as text: a saved one may no longer exist in this version of the course.
    pub level: String,
    pub finished: bool,
    pub levels: BTreeMap<String, Progress>,
}

pub fn now() -> u64 {
    UNIX_EPOCH.elapsed().map_or(0, |since| since.as_secs())
}

impl State {
    pub fn new(workspace: PathBuf, at: Pos) -> State {
        let mut state = State { version: 3, ..State::default() };
        state.workspace = workspace;
        state.go(at);
        state
    }

    pub fn pos(&self) -> Option<Pos> {
        self.level.parse().ok()
    }

    /// The progress of a level; its clock starts when it is first asked for.
    pub fn at(&mut self, pos: Pos) -> &mut Progress {
        let progress = self.levels.entry(pos.to_string()).or_default();
        if progress.started == 0 {
            progress.started = now();
        }
        progress
    }

    /// Makes `pos` the current level; true if anything changed.
    pub fn go(&mut self, pos: Pos) -> bool {
        let key = pos.to_string();
        let changed = self.level != key || !self.levels.contains_key(&key);
        self.level = key;
        self.at(pos);
        changed
    }

    pub fn save(&self, dir: &Path) -> io::Result<()> {
        let mut json = serde_json::to_vec_pretty(self)?;
        json.push(b'\n');
        replace(dir, FILE, &json).map(drop)
    }
}

/// Writes `<dir>/<name>` (mode 0600), creating `dir`, and returns its path. Atomic: a temporary file
/// in the same directory (Landlock ABI 1 forbids cross-directory renames), synced, then renamed over
/// the old one, so a reader never sees a partial file.
pub fn replace(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<PathBuf> {
    fs::create_dir_all(dir)?;
    let mut staged = tempfile::Builder::new().prefix(".staged-").tempfile_in(dir)?;
    staged.write_all(bytes)?;
    staged.as_file().sync_all()?;
    let target = dir.join(name);
    staged.persist(&target).map_err(|error| error.error)?;
    Ok(target)
}

pub enum Saved {
    None,
    /// A file of an older version; `retire` sets it aside. It holds where to `resume` in this
    /// version when the file says where the player was.
    Old(Option<String>),
    Game(State),
}

/// A damaged file, or one of a newer version, is an error with a Czech message and is left alone.
pub fn load(dir: &Path) -> io::Result<Saved> {
    let path = dir.join(FILE);
    let text = match fs::read_to_string(&path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Saved::None),
        text => text?,
    };
    let corrupt = |why: &dyn std::fmt::Display| {
        let hint = "Opravte ho, nebo hru odstraňte příkazem: shellgame remove";
        let message = format!("Soubor {} je poškozený ({why}). {hint}", path.display());
        io::Error::new(ErrorKind::InvalidData, message)
    };
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| corrupt(&e))?;
    match value.get("version").and_then(|version| version.as_u64()) {
        Some(3) => serde_json::from_value(value).map(Saved::Game).map_err(|e| corrupt(&e)),
        Some(newer) if newer > 3 => {
            let what = format!("Soubor {} pochází z novější verze ShellGame", path.display());
            let fix = "Aktualizujte hru příkazem: shellgame update";
            Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("{what} (stav verze {newer}). {fix}"),
            ))
        }
        _ if value.is_object() => Ok(Saved::Old(old_position(&value))),
        _ => Err(corrupt(&"není objekt JSON")),
    }
}

/// Where the player of a Python-era save (state version "2.1", whose level ids are this course's
/// positions, `S.0` being a section's intro) continues: `S.L`, or `S` for an intro.
fn old_position(value: &serde_json::Value) -> Option<String> {
    if value.get("version")?.as_str()? != "2.1" {
        return None;
    }
    let level = value.get("current_level")?.as_str()?;
    let target = level.strip_suffix(".0").unwrap_or(level);
    target.parse::<Pos>().is_ok().then(|| target.to_owned())
}

/// Renames the saved file to `state-old.json` (`state-old-2.json`, ... if taken), never overwriting;
/// returns the new path.
pub fn retire(dir: &Path) -> io::Result<PathBuf> {
    let (mut name, mut n) = ("state-old.json".to_owned(), 1);
    while dir.join(&name).exists() {
        n += 1;
        name = format!("state-old-{n}.json");
    }
    fs::rename(dir.join(FILE), dir.join(&name))?;
    Ok(dir.join(name))
}

pub fn remove(dir: &Path) -> io::Result<()> {
    match fs::remove_file(dir.join(FILE)) {
        Err(error) if error.kind() != ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

/// The workspace of a saved game of any version, even a damaged one, as far as it can be read.
pub fn workspace_of(dir: &Path) -> Option<PathBuf> {
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join(FILE)).ok()?).ok()?;
    value.get("workspace")?.as_str().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    const AT: Pos = Pos { section: 3, level: 2 };

    #[test]
    fn a_game_survives_a_save_and_leaves_no_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut state = State::new("/tmp/shellgame-ada".into(), AT);
        state.at(AT).attempts = 2;
        state.at(AT).done = Some(state.at(AT).started + 7);
        state.save(dir.path()).unwrap();
        state.finished = true;
        state.save(dir.path()).unwrap();
        let Saved::Game(loaded) = load(dir.path()).unwrap() else { panic!("a game") };
        assert_eq!(loaded, state);
        assert_eq!((loaded.pos(), loaded.levels["3.2"].seconds()), (Some(AT), 7));
        let names = fs::read_dir(dir.path()).unwrap().map(|entry| entry.unwrap().file_name());
        assert_eq!(names.collect::<Vec<_>>(), [FILE]);
    }
}
