//! The level builder, and what a level does at run time: prepare, validate, start.
//!
//! A level is one builder chain: identity and prose, a [`Fixture`] describing its files,
//! the answer and requirements that complete it, an optional `cd` policy and file
//! descriptor rule, and a reference solution. Everything a session needs is a plain public
//! field; everything a level can *do* is a method here, so the contract lives in one place
//! instead of in each level.

use std::io;
use std::path::{Path, PathBuf};

use crate::engine::answer::Answer;
use crate::engine::cd::{CdDecision, CdMove, CdPolicy};
use crate::engine::fd::Fds;
use crate::engine::fixture::{Content, Fixture, FixtureError, repair_section_root};
use crate::engine::require::Requirement;
use crate::engine::section::Section;
use crate::engine::{Check, Ctx, LevelId};
use crate::markers::{Marker, Markers};
use crate::messages;
use crate::paths::{RelPath, same_path};

/// Whether a level is graded, or only read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelKind {
    /// A task the player completes.
    Regular,
    /// The intro page of a section (`N.0`): read, then Enter.
    Intro,
}

/// Where the player is placed when the level starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Start {
    /// Nowhere in particular: the player stays where they are.
    None,
    /// A directory relative to the section root (`""` is the root itself).
    Section(String),
    /// The workspace root, above the section: the game "home".
    Workspace,
}

/// How a level proves it can be completed: a shell script that does the task.
///
/// The replay harness runs the script in a real shell and expects the level to complete,
/// which proves levels are winnable, reset works and the evidence flows end to end.
#[derive(Debug, Clone)]
pub struct Solution {
    source: SolutionSource,
    bash_only: bool,
}

#[derive(Debug, Clone)]
enum SolutionSource {
    Script(String),
    Dynamic(fn(&Ctx) -> String),
}

impl Solution {
    /// The script to run, for the machine described by `ctx`.
    pub fn script(&self, ctx: &Ctx) -> String {
        match &self.source {
            SolutionSource::Script(script) => script.clone(),
            SolutionSource::Dynamic(generate) => generate(ctx),
        }
    }

    /// True when the script uses bash syntax and must not be replayed in fish.
    pub fn is_bash_only(&self) -> bool {
        self.bash_only
    }

    /// True when the script depends on the machine (it is generated from a [`Ctx`]).
    pub fn is_dynamic(&self) -> bool {
        matches!(self.source, SolutionSource::Dynamic(_))
    }
}

/// A level. Build one with [`level`] or [`intro`] and chain the methods below.
///
/// ```
/// use shellgame::engine::prelude::*;
///
/// let l = level(2, "Čtení souboru")
///     .text("Přečtěte si `message.txt`.")
///     .start("")
///     .hints(["Použijte `cat`.", "Zadejte první slovo."])
///     .file("message.txt", "Secret is the key.\n")
///     .answer(exact("secret").ignore_case())
///     .solution("shellgame submit \"$(cut -d' ' -f1 message.txt)\"");
/// assert_eq!(l.id.number, 2);
/// ```
#[derive(Debug, Clone)]
pub struct Level {
    /// The stable id. The section part is assigned when the level joins a [`Section`].
    pub id: LevelId,
    /// The title, without the id.
    pub title: String,
    /// Regular task or section intro.
    pub kind: LevelKind,
    /// The assignment, as Markdown. Pages of an intro are separated by `\n---\n`.
    pub text: String,
    /// Plain-text hints, revealed one at a time. Printed verbatim, never as Markdown.
    pub hints: Vec<String>,
    /// Where the player is placed when the level starts.
    pub start: Start,
    /// A bonus level never gates progress.
    pub optional: bool,
    /// Shown when the level is completed.
    pub success: String,
    /// What the level's files look like (applied after the section's fixture).
    pub fixture: Fixture,
    /// How a submitted answer is judged, if the level takes one.
    pub answer: Option<Answer>,
    /// Conditions that are always checked, before the answer.
    pub requirements: Vec<Requirement>,
    /// Whether `shellgame submit` without an answer may complete the level.
    pub allow_empty: bool,
    /// The level's `cd` policy, if it teaches one movement command.
    pub cd: Option<CdPolicy>,
    /// Decides whether the hooked command's output redirection counts as the drill.
    pub on_fd: Option<fn(&Fds) -> bool>,
    /// Extra markers cleared whenever the level is prepared.
    pub resets: Vec<Marker>,
    /// A script that completes the level.
    pub solution: Option<Solution>,
}

/// Start a regular level: `level(8, "Integrační výzva")`. The number is explicit, so
/// reordering never renumbers anything.
pub fn level(number: u16, title: impl Into<String>) -> Level {
    Level {
        id: LevelId::new(0, number),
        title: title.into(),
        kind: LevelKind::Regular,
        text: String::new(),
        hints: Vec::new(),
        start: Start::None,
        optional: false,
        success: messages::CORRECT.to_owned(),
        fixture: Fixture::new(),
        answer: None,
        requirements: Vec::new(),
        allow_empty: false,
        cd: None,
        on_fd: None,
        resets: Vec::new(),
        solution: None,
    }
}

/// Start a section intro (`N.0`): read, then Enter. It has the standard hint
/// ([`messages::INTRO_HINT`]) and no start directory unless one is set.
pub fn intro(title: impl Into<String>, text: impl Into<String>) -> Level {
    Level {
        kind: LevelKind::Intro,
        text: text.into(),
        hints: vec![messages::INTRO_HINT.to_owned()],
        ..level(0, title)
    }
}

impl Level {
    /// The assignment text (Markdown).
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// The hints, replacing any set before.
    #[must_use]
    pub fn hints<I, S>(mut self, hints: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.hints = hints.into_iter().map(Into::into).collect();
        self
    }

    /// Place the player in this section-relative directory when the level starts (`""` or
    /// `"."` is the section root).
    #[must_use]
    pub fn start(mut self, path: impl Into<String>) -> Self {
        self.start = Start::Section(path.into());
        self
    }

    /// Place the player at the workspace root, above the section.
    #[must_use]
    pub fn start_at_workspace(mut self) -> Self {
        self.start = Start::Workspace;
        self
    }

    /// Mark the level as a bonus: it never gates advancement.
    #[must_use]
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// The text shown on completion, instead of [`messages::CORRECT`].
    #[must_use]
    pub fn success(mut self, message: impl Into<String>) -> Self {
        self.success = message.into();
        self
    }

    /// Declare directories; see [`Fixture::dirs`].
    #[must_use]
    pub fn dirs<I, S>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.fixture = self.fixture.dirs(paths);
        self
    }

    /// Declare a directory; see [`Fixture::dir`].
    #[must_use]
    pub fn dir(mut self, path: impl Into<String>) -> Self {
        self.fixture = self.fixture.dir(path);
        self
    }

    /// Declare a file; see [`Fixture::file`].
    #[must_use]
    pub fn file(mut self, path: impl Into<String>, content: impl Into<Content>) -> Self {
        self.fixture = self.fixture.file(path, content);
        self
    }

    /// Declare a file with an explicit mode; see [`Fixture::file_mode`].
    #[must_use]
    pub fn file_mode(
        mut self,
        path: impl Into<String>,
        content: impl Into<Content>,
        mode: u32,
    ) -> Self {
        self.fixture = self.fixture.file_mode(path, content, mode);
        self
    }

    /// Declare a file that is only written when missing; see [`Fixture::keep_file`].
    #[must_use]
    pub fn keep_file(mut self, path: impl Into<String>, content: impl Into<Content>) -> Self {
        self.fixture = self.fixture.keep_file(path, content);
        self
    }

    /// Declare a directory mode; see [`Fixture::dir_mode`].
    #[must_use]
    pub fn dir_mode(mut self, path: impl Into<String>, mode: u32) -> Self {
        self.fixture = self.fixture.dir_mode(path, mode);
        self
    }

    /// Declare paths to remove before anything is created; see [`Fixture::clean`].
    #[must_use]
    pub fn clean<I, S>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.fixture = self.fixture.clean(paths);
        self
    }

    /// Merge a prebuilt or generated fixture into this level's.
    #[must_use]
    pub fn fixture(mut self, fixture: Fixture) -> Self {
        self.fixture = self.fixture.merge(fixture);
        self
    }

    /// How a submitted answer is judged.
    #[must_use]
    pub fn answer(mut self, answer: Answer) -> Self {
        self.answer = Some(answer);
        self
    }

    /// Add a condition that is always checked, in declaration order, before the answer.
    #[must_use]
    pub fn require(mut self, requirement: impl Into<Requirement>) -> Self {
        self.requirements.push(requirement.into());
        self
    }

    /// Let `shellgame submit` without an answer complete the level (once the requirements
    /// hold).
    #[must_use]
    pub fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    /// Give the level a `cd` policy; see [`CdPolicy`].
    #[must_use]
    pub fn cd(mut self, policy: CdPolicy) -> Self {
        self.cd = Some(policy);
        self
    }

    /// Record the level's `fd` evidence when this predicate holds for the hooked
    /// command's descriptors; pair it with `fd_evidence(..)`.
    #[must_use]
    pub fn on_fd(mut self, predicate: fn(&Fds) -> bool) -> Self {
        self.on_fd = Some(predicate);
        self
    }

    /// Markers to clear whenever the level is prepared, in addition to the ones the level
    /// already references (evidence, its `cd`/`fd` markers, `after` rules).
    #[must_use]
    pub fn resets<I>(mut self, markers: I) -> Self
    where
        I: IntoIterator<Item = Marker>,
    {
        self.resets.extend(markers);
        self
    }

    /// A reference solution: a shell script valid in both bash and fish.
    #[must_use]
    pub fn solution(mut self, script: impl Into<String>) -> Self {
        self.solution = Some(Solution {
            source: SolutionSource::Script(script.into()),
            bash_only: false,
        });
        self
    }

    /// A reference solution that needs bash syntax; it is not replayed in fish.
    #[must_use]
    pub fn solution_bash(mut self, script: impl Into<String>) -> Self {
        self.solution = Some(Solution {
            source: SolutionSource::Script(script.into()),
            bash_only: true,
        });
        self
    }

    /// A reference solution generated from the machine (such as a walk down `$HOME`); the
    /// script must be valid in both bash and fish.
    #[must_use]
    pub fn solution_with(mut self, generate: fn(&Ctx) -> String) -> Self {
        self.solution = Some(Solution {
            source: SolutionSource::Dynamic(generate),
            bash_only: false,
        });
        self
    }

    /// True for a section intro.
    pub fn is_intro(&self) -> bool {
        self.kind == LevelKind::Intro
    }

    /// True for a bonus level, which never gates advancement.
    pub fn is_bonus(&self) -> bool {
        self.optional
    }

    /// The title as shown to the player: `"1.4: Návrat na základnu"` for a regular level,
    /// the bare title for an intro.
    pub fn display_title(&self) -> String {
        if self.is_intro() {
            self.title.clone()
        } else {
            format!("{}: {}", self.id, self.title)
        }
    }

    /// Every marker this level reads or writes, without duplicates: evidence
    /// requirements, its own `cd` marker if it has a policy, its `fd` marker if it has an
    /// `on_fd` rule, the markers of `after` rules and the explicit [`Level::resets`].
    pub fn referenced_markers(&self) -> Vec<Marker> {
        let mut found: Vec<Marker> = Vec::new();
        let mut add = |marker: Marker| {
            if !found.contains(&marker) {
                found.push(marker);
            }
        };
        for requirement in &self.requirements {
            if let Some(marker) = requirement.marker() {
                add(marker);
            }
        }
        if let Some(policy) = &self.cd {
            add(Marker::Cd);
            for marker in policy.after_markers() {
                add(marker);
            }
        }
        if self.on_fd.is_some() {
            add(Marker::Fd);
        }
        for marker in &self.resets {
            add(*marker);
        }
        found
    }

    /// Put the level into its starting state: clear its markers and rebuild its files.
    /// Containment is verified first, so nothing is touched below a workspace or section
    /// directory that is a symlink.
    ///
    /// 1. Every marker the level references is removed, so evidence from an earlier
    ///    attempt (or an earlier visit) cannot complete it.
    /// 2. The section root is repaired: a file in its place is made removable and deleted,
    ///    a directory the player locked is made usable. (A symlink is refused. Section 0,
    ///    whose root is the workspace itself, is never repaired this way.)
    /// 3. The section's fixture is applied, then the level's own.
    ///
    /// Used for the first visit, for `shellgame reset` and for jumps; see [`Fixture::apply`]
    /// for the repair rules that make it work after anything the player may have done.
    pub fn prepare(&self, section: &Section, workspace: &Path) -> Result<(), FixtureError> {
        // Containment first: nothing is touched on behalf of a workspace or section root
        // that is a symlink.
        let root = section.section_root(workspace)?;

        let markers = Markers::new(workspace, self.id);
        for marker in self.referenced_markers() {
            markers.remove(&marker).map_err(|source| FixtureError::Io {
                action: "remove marker",
                path: markers.path(&marker),
                source,
            })?;
        }

        // Section 0 lives in the workspace itself, which is not the level's to repair or
        // delete (it may be a directory the user pointed the game at).
        if !section.root.is_empty() {
            repair_section_root(&root)?;
        }
        // Applying even an empty fixture makes sure the root exists.
        section.fixture.apply(&root)?;
        if !self.fixture.is_empty() {
            self.fixture.apply(&root)?;
        }
        Ok(())
    }

    /// Judge a submission: the requirements first, in order and unconditionally (the first
    /// failure wins), then the answer.
    ///
    /// With no answer (`None`): the level passes if it allows empty submissions or takes no
    /// answer, and otherwise fails with the answer's "required" message. With an answer, it
    /// is judged by the answer rule, or accepted when the level has none. On `Pass`, the
    /// session shows [`Level::success`].
    pub fn validate(&self, ctx: &Ctx, answer: Option<&str>) -> Check {
        for requirement in &self.requirements {
            if let failure @ Check::Fail(_) = requirement.check(ctx) {
                return failure;
            }
        }
        match (answer, &self.answer) {
            (None, Some(rule)) if !self.allow_empty => Check::fail(rule.required_message()),
            (Some(text), Some(rule)) => rule.check(ctx, text),
            _ => Check::Pass,
        }
    }

    /// The directory the player is placed in when the level starts, if it has one.
    pub fn start_dir(&self, workspace: &Path, section_root: &Path) -> Option<PathBuf> {
        match &self.start {
            Start::None => None,
            Start::Workspace => Some(workspace.to_path_buf()),
            Start::Section(path) => Some(RelPath::parse(path).map_or_else(
                |_| section_root.to_path_buf(),
                |rel| rel.under(section_root),
            )),
        }
    }

    /// True when this `cd` would bring the player back to the level's start directory.
    ///
    /// Strict `cd` rules always permit the recovery move, so a player can never be locked
    /// out of the level's own workspace. The destination is resolved the way the shell
    /// will ([`CdMove::destination`]); where it cannot be (an unknown `cd -`), the answer
    /// is no.
    pub fn returns_to_start(&self, mv: &CdMove) -> bool {
        let Some(start) = self.start_dir(mv.ctx.workspace, &mv.ctx.section_root) else {
            return false;
        };
        mv.destination()
            .is_some_and(|destination| same_path(&destination, &start))
    }

    /// True when a strict `cd` rule must not block this move: the level's `cd` evidence
    /// exists (the player has shown the command) or the move returns to the start
    /// directory.
    ///
    /// Without this, one extra step could leave a player in a directory from which every
    /// permitted move leads further away. Lifting waives *blocking* only; it never grants
    /// or denies evidence.
    pub fn cd_enforcement_lifted(&self, mv: &CdMove) -> bool {
        Markers::new(mv.ctx.workspace, self.id).exists(&Marker::Cd) || self.returns_to_start(mv)
    }

    /// Decide whether a `cd` about to happen is allowed. Levels without a `cd` policy allow
    /// everything. Printing the red panel and the recovery tip is the UI's job.
    pub fn cd_pre_move(&self, mv: &CdMove) -> CdDecision {
        match &self.cd {
            Some(policy) => policy.pre_move(self, mv),
            None => CdDecision::Allow,
        }
    }

    /// Record the outcome of a `cd` that succeeded (evidence, `on_move`).
    pub fn cd_post_move(&self, mv: &CdMove) {
        if let Some(policy) = &self.cd {
            policy.post_move(self, mv);
        }
    }

    /// [`Level::cd_pre_move`] for a shell started by the Python version of the game, which
    /// only reports the move *before* it happens; see [`CdPolicy::legacy_pre_move`].
    pub fn cd_legacy_pre_move(&self, mv: &CdMove) -> CdDecision {
        match &self.cd {
            Some(policy) => policy.legacy_pre_move(self, mv),
            None => CdDecision::Allow,
        }
    }

    /// The post-move report of a Python-era shell; see [`CdPolicy::legacy_post_move`].
    pub fn cd_legacy_post_move(&self, ctx: &Ctx, dest: &Path) {
        if let Some(policy) = &self.cd {
            policy.legacy_post_move(self, ctx, dest);
        }
    }

    /// Record the level's `fd` evidence if its `on_fd` rule accepts these descriptors.
    /// Returns whether the marker was recorded.
    pub fn record_fd(&self, fds: &Fds, markers: &Markers) -> io::Result<bool> {
        match self.on_fd {
            Some(accepts) if accepts(fds) => {
                markers.create(&Marker::Fd)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::answer::{exact, int};
    use crate::engine::cd::policy;
    use crate::engine::fd::FdTarget;
    use crate::engine::require::{at, cd_evidence, check, evidence, fd_evidence, file, perm_bits};
    use std::fs::{self, Permissions};
    use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};

    /// A workspace (its path contains a space) and a home, all canonical.
    struct World {
        _tmp: tempfile::TempDir,
        ws: PathBuf,
        home: PathBuf,
    }

    impl World {
        fn new() -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let base = fs::canonicalize(tmp.path()).unwrap();
            let home = base.join("home/ada");
            fs::create_dir_all(&home).unwrap();
            Self {
                ws: base.join("my workspace"),
                home,
                _tmp: tmp,
            }
        }

        fn section(&self, number: u16) -> Section {
            Section::new(number, format!("level-{number}"))
        }

        fn ctx<'a>(&'a self, section: &Section, level: &Level, cwd: Option<PathBuf>) -> Ctx<'a> {
            section.ctx(level, &self.ws, cwd, &self.home).unwrap()
        }

        fn mode(path: &Path) -> u32 {
            fs::symlink_metadata(path).unwrap().mode() & 0o7777
        }
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    // ------------------------------------------------------------------ builders ---

    #[test]
    fn a_new_level_has_sensible_defaults() {
        let l = level(4, "Návrat");
        assert_eq!(
            l.id,
            LevelId::new(0, 4),
            "the section is assigned by the Section"
        );
        assert_eq!(l.title, "Návrat");
        assert_eq!(l.kind, LevelKind::Regular);
        assert!(!l.is_intro());
        assert!(!l.is_bonus());
        assert_eq!(l.text, "");
        assert!(l.hints.is_empty());
        assert_eq!(l.start, Start::None);
        assert_eq!(l.success, messages::CORRECT);
        assert!(l.fixture.is_empty());
        assert!(l.answer.is_none());
        assert!(l.requirements.is_empty());
        assert!(!l.allow_empty);
        assert!(l.cd.is_none());
        assert!(l.on_fd.is_none());
        assert!(l.resets.is_empty());
        assert!(l.solution.is_none());
    }

    #[test]
    fn an_intro_has_the_standard_hint_and_no_start() {
        let l = intro("Navigace", "# Navigace\n---\nDruhá strana");
        assert_eq!(l.kind, LevelKind::Intro);
        assert!(l.is_intro());
        assert_eq!(l.id.number, 0);
        assert_eq!(l.hints, [messages::INTRO_HINT]);
        assert_eq!(l.hints, ["Přečtěte si úvod a pokračujte stisknutím Enter."]);
        assert_eq!(l.start, Start::None);
        assert_eq!(l.text, "# Navigace\n---\nDruhá strana");
        assert_eq!(l.title, "Navigace");
        // It can still be placed and given a greeting.
        let l = l.start_at_workspace().success("Jdeme na to!");
        assert_eq!(l.start, Start::Workspace);
        assert_eq!(l.success, "Jdeme na to!");
    }

    #[test]
    fn builder_methods_set_their_fields() {
        let l = level(2, "x")
            .text("Zadání")
            .hints(["a", "b"])
            .start("alpha")
            .optional()
            .success("Hotovo")
            .allow_empty();
        assert_eq!(l.text, "Zadání");
        assert_eq!(l.hints, ["a", "b"]);
        assert_eq!(l.start, Start::Section("alpha".to_owned()));
        assert!(l.optional && l.is_bonus());
        assert_eq!(l.success, "Hotovo");
        assert!(l.allow_empty);
        // `hints` replaces rather than appends.
        assert_eq!(l.hints(["c"]).hints, ["c"]);
        assert_eq!(level(1, "x").start("").start, Start::Section(String::new()));
        assert_eq!(
            level(1, "x").start(".").start,
            Start::Section(".".to_owned())
        );
        assert_eq!(
            level(1, "x").start("a").start_at_workspace().start,
            Start::Workspace
        );
    }

    #[test]
    fn fixture_methods_delegate_to_the_levels_fixture() {
        let l = level(1, "x")
            .dirs(["a", "b"])
            .dir("c")
            .file("f.txt", "text")
            .file_mode("run.sh", "x", 0o755)
            .keep_file("k.txt", "kept")
            .dir_mode("d", 0o700)
            .clean(["old"]);
        assert_eq!(l.fixture.dir_paths(), ["a", "b", "c"]);
        assert_eq!(l.fixture.file_entries().count(), 3);
        assert_eq!(l.fixture.clean_paths(), ["old"]);
        assert_eq!(
            l.fixture.dir_mode_entries().collect::<Vec<_>>(),
            [("d", 0o700)]
        );
    }

    #[test]
    fn fixture_merges_a_prebuilt_fixture_after_what_was_declared() {
        let generated = Fixture::new().file("maze/a", "x").dir("maze/b");
        let l = level(7, "x").dir("first").fixture(generated).dir("last");
        assert_eq!(l.fixture.dir_paths(), ["first", "maze/b", "last"]);
        assert_eq!(l.fixture.file_entries().count(), 1);
    }

    #[test]
    fn requirements_and_resets_accumulate_in_order() {
        let l = level(1, "x")
            .require(file("a"))
            .require(at("b"))
            .resets([Marker::Pwd])
            .resets([Marker::Own("x")]);
        assert_eq!(l.requirements.len(), 2);
        assert_eq!(l.requirements[0].paths(), ["a"]);
        assert_eq!(l.requirements[1].paths(), ["b"]);
        assert_eq!(l.resets, [Marker::Pwd, Marker::Own("x")]);
    }

    #[test]
    fn display_title_includes_the_id_only_for_regular_levels() {
        let section = Section::new(1, "level-1")
            .intro(intro("Navigace", "t"))
            .levels([level(4, "Návrat na základnu")]);
        assert_eq!(section.levels[0].display_title(), "Navigace");
        assert_eq!(section.levels[1].display_title(), "1.4: Návrat na základnu");
    }

    #[test]
    fn solutions_come_in_three_flavours() {
        let world = World::new();
        let section = world.section(1);
        let plain = level(1, "x").solution("cd a\nshellgame submit");
        let bash = level(1, "x").solution_bash("[[ -d a ]] && cd a");
        let dynamic = level(1, "x").solution_with(|ctx| format!("cd {}", ctx.home.display()));
        let ctx = world.ctx(&section, &plain, None);

        let solution = plain.solution.as_ref().unwrap();
        assert_eq!(solution.script(&ctx), "cd a\nshellgame submit");
        assert!(!solution.is_bash_only());
        assert!(!solution.is_dynamic());

        let solution = bash.solution.as_ref().unwrap();
        assert_eq!(solution.script(&ctx), "[[ -d a ]] && cd a");
        assert!(solution.is_bash_only());
        assert!(!solution.is_dynamic());

        let solution = dynamic.solution.as_ref().unwrap();
        assert_eq!(
            solution.script(&ctx),
            format!("cd {}", world.home.display())
        );
        assert!(!solution.is_bash_only(), "valid in both shells");
        assert!(solution.is_dynamic());
    }

    #[test]
    fn engine_types_can_live_in_a_static() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Level>();
        assert_send_sync::<Section>();
        assert_send_sync::<crate::engine::catalog::Catalog>();
        assert_send_sync::<Answer>();
        assert_send_sync::<Requirement>();
        assert_send_sync::<CdPolicy>();
        assert_send_sync::<Fixture>();
    }

    // ------------------------------------------------------------- start directory --

    #[test]
    fn start_dir_resolves_the_three_kinds_of_start() {
        let ws = Path::new("/ws");
        let root = Path::new("/ws/level-1");
        assert_eq!(level(1, "x").start_dir(ws, root), None);
        assert_eq!(
            level(1, "x").start_at_workspace().start_dir(ws, root),
            Some(PathBuf::from("/ws"))
        );
        assert_eq!(
            level(1, "x").start("").start_dir(ws, root),
            Some(PathBuf::from("/ws/level-1"))
        );
        assert_eq!(
            level(1, "x").start(".").start_dir(ws, root),
            Some(PathBuf::from("/ws/level-1"))
        );
        assert_eq!(
            level(1, "x").start("gamma/./deep/").start_dir(ws, root),
            Some(PathBuf::from("/ws/level-1/gamma/deep"))
        );
    }

    #[test]
    fn a_malformed_start_falls_back_to_the_section_root_and_the_catalog_lint_reports_it() {
        let root = Path::new("/ws/level-1");
        assert_eq!(
            level(1, "x")
                .start("../x")
                .start_dir(Path::new("/ws"), root),
            Some(PathBuf::from("/ws/level-1"))
        );
    }

    #[test]
    fn returns_to_start_resolves_the_destination_like_the_shell() {
        let world = World::new();
        let section = world.section(1);
        let l = level(4, "x").start("alpha");
        let section = section.levels([l]);
        let l = &section.levels[0];
        let start = world.ws.join("level-1/alpha");
        let ctx = world.ctx(&section, l, None);
        let args: Vec<String> = Vec::new();
        let far = world.ws.join("level-1/gamma/deep");

        let to = |target: &str, prev: Option<&Path>| {
            l.returns_to_start(&CdMove::new(&ctx, target, &args, &far).with_prev(prev))
        };
        assert!(to("../../alpha", None));
        assert!(to(start.to_str().unwrap(), None));
        assert!(to("-", Some(&start)));
        assert!(!to("-", Some(&far)));
        assert!(!to("-", None));
        assert!(!to("..", None));
        assert!(!to("", None), "home is not the start");
        // A level without a start never "returns" anywhere.
        assert!(!level(5, "x").returns_to_start(&CdMove::new(&ctx, "../../alpha", &args, &far)));
    }

    // ------------------------------------------------------------------- validate ---

    fn challenge(world: &World) -> (Section, Ctx<'_>) {
        let section = world.section(2).levels([level(8, "x")
            .answer(exact("navigator").ignore_case().required("Zadejte heslo."))
            .require(at("room2"))
            .require(file("room2/password.txt").message("Chybí password.txt."))
            .dirs(["room1", "room2"])
            .file("room2/password.txt", "navigator\n")]);
        let ctx = world.ctx(&section, &section.levels[0], None);
        section.levels[0].prepare(&section, &world.ws).unwrap();
        (section, ctx)
    }

    #[test]
    fn requirements_are_unconditional_so_a_correct_answer_cannot_shortcut_them() {
        let world = World::new();
        let (section, ctx) = challenge(&world);
        let l = &section.levels[0];
        let ctx = Ctx {
            cwd: Some(ctx.section_root.join("room1")),
            ..ctx
        };
        assert_eq!(
            l.validate(&ctx, Some("navigator")),
            Check::fail(messages::WRONG_DIRECTORY),
            "the right answer from the wrong place fails"
        );
        let ctx = Ctx {
            cwd: Some(ctx.section_root.join("room2")),
            ..ctx
        };
        assert_eq!(l.validate(&ctx, Some("Navigator")), Check::Pass);
    }

    #[test]
    fn the_first_failing_requirement_wins_in_declaration_order() {
        let world = World::new();
        let (section, ctx) = challenge(&world);
        let l = &section.levels[0];
        fs::remove_file(ctx.section_root.join("room2/password.txt")).unwrap();
        // Both requirements fail; the first declared (`at`) is reported.
        let wrong_place = Ctx {
            cwd: Some(ctx.section_root.join("room1")),
            ..ctx.clone()
        };
        assert_eq!(
            l.validate(&wrong_place, None),
            Check::fail(messages::WRONG_DIRECTORY)
        );
        // In the right place, the second one is.
        let right_place = Ctx {
            cwd: Some(ctx.section_root.join("room2")),
            ..ctx
        };
        assert_eq!(
            l.validate(&right_place, None),
            Check::fail("Chybí password.txt.")
        );
    }

    #[test]
    fn an_empty_submission_gets_the_answers_required_message() {
        let world = World::new();
        let (section, ctx) = challenge(&world);
        let l = &section.levels[0];
        let ctx = Ctx {
            cwd: Some(ctx.section_root.join("room2")),
            ..ctx
        };
        assert_eq!(l.validate(&ctx, None), Check::fail("Zadejte heslo."));
        // ...and the default text when the rule sets none.
        let default_text = level(1, "x").answer(exact("a"));
        assert_eq!(
            default_text.validate(&ctx, None),
            Check::fail(messages::ANSWER_REQUIRED)
        );
        let number = level(1, "x").answer(int(3));
        assert_eq!(
            number.validate(&ctx, None),
            Check::fail(messages::ANSWER_REQUIRED_NUMBER)
        );
        // An empty *string* is an answer, not a missing one.
        assert_eq!(
            default_text.validate(&ctx, Some("")),
            Check::fail(messages::INCORRECT)
        );
        assert_eq!(
            number.validate(&ctx, Some("")),
            Check::fail(messages::EXPECTED_INTEGER)
        );
    }

    #[test]
    fn allow_empty_accepts_an_empty_submission_but_never_skips_requirements() {
        let world = World::new();
        let (section, ctx) = challenge(&world);
        let strict = &section.levels[0];
        let lenient = strict.clone().allow_empty();

        let wrong_place = Ctx {
            cwd: Some(ctx.section_root.join("room1")),
            ..ctx.clone()
        };
        let right_place = Ctx {
            cwd: Some(ctx.section_root.join("room2")),
            ..ctx
        };
        assert_eq!(
            strict.validate(&right_place, None),
            Check::fail("Zadejte heslo.")
        );
        assert_eq!(lenient.validate(&right_place, None), Check::Pass);
        assert_eq!(
            lenient.validate(&wrong_place, None),
            Check::fail(messages::WRONG_DIRECTORY),
            "allow_empty waives the answer, not the requirements"
        );
        // With allow_empty the answer is still judged when one is given.
        assert_eq!(
            lenient.validate(&right_place, Some("wrong")),
            Check::fail(messages::INCORRECT)
        );
        assert_eq!(
            lenient.validate(&right_place, Some("navigator")),
            Check::Pass
        );
    }

    #[test]
    fn a_level_without_an_answer_passes_once_its_requirements_hold() {
        let world = World::new();
        let section = world.section(8).levels([level(3, "chmod")
            .file_mode("config.readonly", "x", 0o644)
            .require(
                perm_bits("config.readonly")
                    .lacks(0o222)
                    .message("Zápis zůstal."),
            )]);
        let l = &section.levels[0];
        l.prepare(&section, &world.ws).unwrap();
        let ctx = world.ctx(&section, l, None);

        assert_eq!(l.validate(&ctx, None), Check::fail("Zápis zůstal."));
        // An answer nobody asked for is not held against the player.
        assert_eq!(
            l.validate(&ctx, Some("whatever")),
            Check::fail("Zápis zůstal.")
        );
        fs::set_permissions(
            ctx.section_root.join("config.readonly"),
            Permissions::from_mode(0o444),
        )
        .unwrap();
        assert_eq!(l.validate(&ctx, None), Check::Pass);
        assert_eq!(l.validate(&ctx, Some("whatever")), Check::Pass);
    }

    #[test]
    fn a_level_with_nothing_to_check_passes() {
        let world = World::new();
        let section = world.section(1).intro(intro("x", "y"));
        let ctx = world.ctx(&section, &section.levels[0], None);
        assert_eq!(section.levels[0].validate(&ctx, None), Check::Pass);
        assert_eq!(
            section.levels[0].validate(&ctx, Some("anything")),
            Check::Pass
        );
    }

    #[test]
    fn a_requirement_that_leaves_the_workspace_fails_without_crashing() {
        let world = World::new();
        let outside = tempfile::tempdir().unwrap();
        let section = world
            .section(1)
            .levels([level(1, "x").require(file("link/f")).dir("a")]);
        let l = &section.levels[0];
        l.prepare(&section, &world.ws).unwrap();
        symlink(outside.path(), world.ws.join("level-1/link")).unwrap();
        let ctx = world.ctx(&section, l, None);
        assert_eq!(
            l.validate(&ctx, Some("x")),
            Check::fail(messages::PATH_ESCAPES_WORKSPACE)
        );
    }

    #[test]
    fn custom_requirements_run_in_order_with_the_rest() {
        let world = World::new();
        let section = world.section(1).levels([level(1, "x")
            .require(check(|_| Check::fail("první")))
            .require(check(|_| Check::fail("druhý")))]);
        let ctx = world.ctx(&section, &section.levels[0], None);
        assert_eq!(section.levels[0].validate(&ctx, None), Check::fail("první"));
    }

    // -------------------------------------------------------------------- markers --

    fn marker_level() -> Level {
        level(4, "x")
            .answer(exact("x"))
            .require(evidence(Marker::Pwd, "pwd"))
            .require(evidence(Marker::Own("walk"), "walk"))
            .require(evidence(Marker::Pwd, "again"))
            .require(fd_evidence("fd"))
            .require(cd_evidence("cd"))
            .cd(policy().after(Marker::Own("seen"), "seen").exact("..", "m"))
            .on_fd(|fds| fds.both_null())
            .resets([Marker::Own("extra"), Marker::Cd])
    }

    #[test]
    fn referenced_markers_lists_each_marker_once() {
        let mut found = marker_level().referenced_markers();
        found.sort_by_key(|marker| marker.file_name(LevelId::new(1, 4)));
        let mut expected = vec![
            Marker::Pwd,
            Marker::Own("walk"),
            Marker::Fd,
            Marker::Cd,
            Marker::Own("seen"),
            Marker::Own("extra"),
        ];
        expected.sort_by_key(|marker| marker.file_name(LevelId::new(1, 4)));
        assert_eq!(found, expected);
        assert!(level(1, "x").referenced_markers().is_empty());
        // A cd policy alone references its own marker; `on_fd` alone the fd marker.
        assert_eq!(
            level(1, "x").cd(policy()).referenced_markers(),
            [Marker::Cd]
        );
        assert_eq!(
            level(1, "x").on_fd(|_| true).referenced_markers(),
            [Marker::Fd]
        );
    }

    #[test]
    fn prepare_clears_every_marker_the_level_references_and_no_other() {
        let world = World::new();
        let section = world.section(1).levels([marker_level()]);
        let l = &section.levels[0];
        let mine = Markers::new(&world.ws, l.id);
        let other_level = Markers::new(&world.ws, LevelId::new(1, 5));
        fs::create_dir_all(&world.ws).unwrap();

        let referenced = [
            Marker::Pwd,
            Marker::Own("walk"),
            Marker::Fd,
            Marker::Cd,
            Marker::Own("seen"),
            Marker::Own("extra"),
        ];
        for marker in &referenced {
            mine.create(marker).unwrap();
        }
        // Not referenced: a private marker of this level, another level's, a plain file.
        mine.create(&Marker::Own("keep")).unwrap();
        other_level.create(&Marker::Cd).unwrap();
        other_level.create(&Marker::Fd).unwrap();
        fs::write(world.ws.join("notes.txt"), "mine").unwrap();

        l.prepare(&section, &world.ws).unwrap();

        for marker in &referenced {
            assert!(!mine.exists(marker), "{marker:?} should be cleared");
        }
        assert!(mine.exists(&Marker::Own("keep")));
        assert!(other_level.exists(&Marker::Cd));
        assert!(other_level.exists(&Marker::Fd));
        assert_eq!(read(&world.ws.join("notes.txt")), "mine");
    }

    #[test]
    fn prepare_clears_the_cd_marker_of_a_level_with_a_cd_policy_even_without_evidence_requirements()
    {
        let world = World::new();
        let section = world.section(1).levels([level(7, "maze")
            .cd(policy().within("maze", "m"))
            .require(at("maze"))]);
        let l = &section.levels[0];
        let markers = Markers::new(&world.ws, l.id);
        markers.create(&Marker::Cd).unwrap();
        l.prepare(&section, &world.ws).unwrap();
        assert!(
            !markers.exists(&Marker::Cd),
            "stale 1.7 evidence must not skip the maze"
        );
    }

    #[test]
    fn a_level_without_markers_leaves_the_workspace_markers_alone() {
        let world = World::new();
        let section = world.section(1).levels([level(1, "x").answer(exact("a"))]);
        let markers = Markers::new(&world.ws, section.levels[0].id);
        markers.create(&Marker::Pwd).unwrap();
        markers.create(&Marker::Cd).unwrap();
        section.levels[0].prepare(&section, &world.ws).unwrap();
        assert!(markers.exists(&Marker::Pwd));
        assert!(markers.exists(&Marker::Cd));
    }

    // -------------------------------------------------------------------- prepare --

    #[test]
    fn prepare_builds_the_root_and_applies_the_section_fixture_before_the_levels() {
        let world = World::new();
        let section = world
            .section(2)
            .fixture(
                Fixture::new()
                    .dir("shared")
                    .file("shared.txt", "from the section")
                    .file("both.txt", "section"),
            )
            .levels([level(1, "x")
                .file("both.txt", "level")
                .file("own/level.txt", "mine")]);
        section.levels[0].prepare(&section, &world.ws).unwrap();
        let root = world.ws.join("level-2");
        assert!(root.join("shared").is_dir());
        assert_eq!(read(&root.join("shared.txt")), "from the section");
        assert_eq!(
            read(&root.join("both.txt")),
            "level",
            "the level's fixture comes second"
        );
        assert_eq!(read(&root.join("own/level.txt")), "mine");
    }

    #[test]
    fn prepare_creates_a_missing_workspace_and_section_root() {
        let world = World::new();
        assert!(!world.ws.exists());
        let section = world.section(1).levels([level(1, "x").answer(exact("a"))]);
        section.levels[0].prepare(&section, &world.ws).unwrap();
        assert!(world.ws.join("level-1").is_dir());
        assert_eq!(World::mode(&world.ws.join("level-1")), 0o755);
    }

    #[test]
    fn prepare_is_idempotent_and_keeps_what_the_section_fixture_says_to_keep() {
        let world = World::new();
        let section = world
            .section(1)
            .fixture(Fixture::new().keep_file("alpha/inside.txt", "first step"))
            .levels([level(1, "x").file("fixed.txt", "level")]);
        let l = &section.levels[0];
        l.prepare(&section, &world.ws).unwrap();
        let root = world.ws.join("level-1");
        fs::write(root.join("alpha/inside.txt"), "edited by the player").unwrap();
        fs::write(root.join("fixed.txt"), "vandalized").unwrap();
        l.prepare(&section, &world.ws).unwrap();
        assert_eq!(read(&root.join("alpha/inside.txt")), "edited by the player");
        assert_eq!(read(&root.join("fixed.txt")), "level");
    }

    #[test]
    fn prepare_restores_a_level_after_whatever_the_player_did() {
        let world = World::new();
        let section = world.section(8).levels([level(7, "x")
            .clean(["directory-write"])
            .file("directory-write/readme.txt", "x")
            .dir_mode("directory-write/locked", 0o550)
            .file_mode("config.txt", "fixture", 0o644)
            .dir("data")]);
        let l = &section.levels[0];
        l.prepare(&section, &world.ws).unwrap();
        let root = world.ws.join("level-8");

        // The damage: a locked tree, a file swapped for a directory, a directory for a file
        // and a read-only file.
        fs::set_permissions(root.join("directory-write"), Permissions::from_mode(0o000)).unwrap();
        fs::remove_file(root.join("config.txt")).unwrap();
        fs::create_dir(root.join("config.txt")).unwrap();
        fs::write(root.join("config.txt/inner"), "x").unwrap();
        fs::remove_dir(root.join("data")).unwrap();
        fs::write(root.join("data"), "now a file").unwrap();
        fs::set_permissions(root.join("data"), Permissions::from_mode(0o444)).unwrap();

        l.prepare(&section, &world.ws).unwrap();

        assert_eq!(read(&root.join("directory-write/readme.txt")), "x");
        assert_eq!(World::mode(&root.join("directory-write")), 0o755);
        assert_eq!(World::mode(&root.join("directory-write/locked")), 0o550);
        assert_eq!(read(&root.join("config.txt")), "fixture");
        assert!(root.join("data").is_dir());
    }

    #[test]
    fn prepare_repairs_a_section_root_that_is_a_file_or_locked() {
        let world = World::new();
        fs::create_dir_all(&world.ws).unwrap();
        let section = world.section(1).levels([level(1, "x").file("a.txt", "a")]);
        let l = &section.levels[0];
        let root = world.ws.join("level-1");

        // A file in the way, with no permissions at all.
        fs::write(&root, "in the way").unwrap();
        fs::set_permissions(&root, Permissions::from_mode(0o000)).unwrap();
        l.prepare(&section, &world.ws).unwrap();
        assert!(root.is_dir());
        assert_eq!(read(&root.join("a.txt")), "a");

        // The directory itself locked.
        fs::set_permissions(&root, Permissions::from_mode(0o000)).unwrap();
        l.prepare(&section, &world.ws).unwrap();
        assert_eq!(World::mode(&root), 0o755);
        assert_eq!(read(&root.join("a.txt")), "a");
    }

    #[test]
    fn prepare_refuses_a_symlinked_section_root_and_touches_nothing_behind_it() {
        let world = World::new();
        let outside = tempfile::tempdir().unwrap();
        fs::create_dir_all(&world.ws).unwrap();
        symlink(outside.path(), world.ws.join("level-1")).unwrap();
        let section = world.section(1).levels([level(1, "x")
            .file("a.txt", "a")
            .require(evidence(Marker::Pwd, "m"))]);
        fs::write(world.ws.join(".pwd_used"), "").unwrap();

        let error = section.levels[0].prepare(&section, &world.ws).unwrap_err();
        assert!(matches!(error, FixtureError::Containment(_)), "{error}");
        assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
        assert!(
            world.ws.join(".pwd_used").exists(),
            "containment is checked before markers are cleared"
        );
    }

    #[test]
    fn prepare_refuses_a_symlinked_workspace() {
        let world = World::new();
        let real = world._tmp.path().join("real-workspace");
        fs::create_dir_all(&real).unwrap();
        fs::write(real.join(".cd-1.1"), "").unwrap();
        symlink(&real, &world.ws).unwrap();
        let section = world.section(1).levels([level(1, "x").cd(policy())]);
        let error = section.levels[0].prepare(&section, &world.ws).unwrap_err();
        assert!(
            matches!(
                error,
                FixtureError::Containment(crate::paths::ContainmentError::RootIsSymlink(_))
            ),
            "{error}"
        );
        assert!(
            real.join(".cd-1.1").exists(),
            "nothing behind the link is touched"
        );
        assert!(!real.join("level-1").exists());
    }

    #[test]
    fn prepare_never_repairs_or_deletes_the_workspace_for_section_zero() {
        let world = World::new();
        let section = Section::new(0, "").levels([level(1, "x").answer(exact("start"))]);
        let l = &section.levels[0];

        // A workspace that is a file belongs to the user: refuse, don't delete it.
        fs::write(&world.ws, "my precious file").unwrap();
        assert!(l.prepare(&section, &world.ws).is_err());
        assert_eq!(read(&world.ws), "my precious file");
        fs::remove_file(&world.ws).unwrap();

        // A private workspace stays private.
        fs::create_dir(&world.ws).unwrap();
        fs::set_permissions(&world.ws, Permissions::from_mode(0o700)).unwrap();
        l.prepare(&section, &world.ws).unwrap();
        assert_eq!(World::mode(&world.ws), 0o700);
    }

    #[test]
    fn prepare_reports_an_invalid_fixture_before_changing_anything() {
        let world = World::new();
        let section = world
            .section(1)
            .levels([level(1, "x").file("../escape.txt", "x")]);
        let error = section.levels[0].prepare(&section, &world.ws).unwrap_err();
        assert!(matches!(error, FixtureError::InvalidPath(_)), "{error}");
        assert!(!world.ws.join("escape.txt").exists());
        assert!(!world.ws.parent().unwrap().join("escape.txt").exists());
    }

    // ------------------------------------------------------------------------ fd ---

    #[test]
    fn record_fd_creates_the_marker_only_when_the_rule_accepts_the_descriptors() {
        let world = World::new();
        fn both(fds: &Fds) -> bool {
            fds.stdout_is_null() && fds.stderr_is_null()
        }
        let l = level(4, "x").on_fd(both);
        let mut section = world.section(10);
        section.levels.push(Level {
            id: LevelId::new(10, 4),
            ..l.clone()
        });
        let l = &section.levels[0];
        let markers = Markers::new(&world.ws, l.id);

        let null = FdTarget::from_path("/dev/null").unwrap();
        let file = tempfile::NamedTempFile::new().unwrap();
        let regular = FdTarget::from_fd(file.as_file()).unwrap();

        assert!(
            !l.record_fd(
                &Fds {
                    stdout: null,
                    stderr: regular
                },
                &markers
            )
            .unwrap()
        );
        assert!(
            !l.record_fd(
                &Fds {
                    stdout: regular,
                    stderr: null
                },
                &markers
            )
            .unwrap()
        );
        assert!(!markers.exists(&Marker::Fd));
        assert!(
            l.record_fd(
                &Fds {
                    stdout: null,
                    stderr: null
                },
                &markers
            )
            .unwrap()
        );
        assert!(markers.exists(&Marker::Fd));
        assert!(world.ws.join(".fd-10.4").is_file());

        // A level without a rule never records.
        let plain = level(5, "x");
        let plain_markers = Markers::new(&world.ws, LevelId::new(10, 5));
        assert!(
            !plain
                .record_fd(
                    &Fds {
                        stdout: null,
                        stderr: null
                    },
                    &plain_markers
                )
                .unwrap()
        );
        assert!(!plain_markers.exists(&Marker::Fd));
    }
}
