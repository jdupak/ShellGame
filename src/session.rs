//! The game session: what every `shellgame` command does.
//!
//! [`Session`] ports `GameSession`, `ProgressTracker` and `NavigationManager` of the Python
//! version. It is the one place that combines the pieces: the [`Catalog`] says what the levels
//! are, [`StateStore`] what the player has done, the engine judges work and prepares workspaces,
//! [`screens`] shows the result, and [`Directives`] asks the wrapping shell to move or to
//! export variables. The CLI layer ([`crate::cli`]) only parses arguments and calls one method.
//!
//! | command                  | method                         |
//! |--------------------------|--------------------------------|
//! | (none)                   | [`Session::show_current_level`] |
//! | `hint [-r]`              | [`Session::hint`]              |
//! | `submit [ANSWER...]`     | [`Session::submit`]            |
//! | `status`, `skip`, `reset`| [`Session::status`], [`Session::skip`], [`Session::reset`] |
//! | `repeat`, `show`         | [`Session::repeat`], [`Session::show`] |
//! | `remove`, `exit`, `init` | [`Session::remove`], [`Session::exit`], [`Session::init`] |
//! | `dev ...`                | [`Session::dev_jump`], [`Session::dev_next`], [`Session::dev_prev`], [`Session::dev_reload`], [`Session::dev_start`] |
//! | `update`                 | [`Session::update`]            |
//! | `cd-hook ...`, `fd-hook` | [`Session::cd_pre`], [`Session::cd_post`], [`Session::legacy_cd_pre`], [`Session::legacy_cd_post`], [`Session::fd_hook`] |
//!
//! Every method returns the process exit status: [`OK`] when the command did what the player
//! asked (a wrong answer is a normal outcome), [`FAILURE`] when it could not, [`USAGE`] for a
//! usage error and, for `cd-hook pre`, [`BLOCKED`].
//!
//! # Where the player is
//!
//! The session keeps the player's directory in memory and updates it whenever it asks the shell to
//! `cd`, as Python's `os.chdir` did, so a second check in the same run neither repeats the
//! directive nor shows the teleport notice twice. When a level was prepared in this run while the
//! player stands below its section directory, that directory may have been deleted and recreated
//! underneath the shell, so the next teleport check asks the shell to `cd` even when the path is
//! unchanged (without a notice).
//!
//! # Directives before prompts
//!
//! The environment exports and the teleport of a level are emitted *before* the Enter prompts
//! that may follow, so the wrapper applies them even when the player presses Ctrl-C at a prompt
//! (the prompt ends like Enter, the process exits normally, the wrapper replays what was
//! emitted).

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::engine::fd::Fds;
use crate::engine::fixture::{FixtureError, remove_tree};
use crate::engine::prelude::{
    Catalog, CdDecision, CdMove, Check, Ctx, Level, LevelId, Markers, Section,
};
use crate::env::Env;
use crate::messages;
use crate::paths::soft_canonicalize;
use crate::shell::{Directives, ENV_LEVEL, ENV_WORKSPACE};
use crate::state::{self, LevelCompletion, State, StateError, StateStore};
use crate::ui::prompt::PromptOutcome;
use crate::ui::{
    Term, clear_screen, confirm, enter_prompt, print_lines, screens, split_pages, style,
    wait_for_enter,
};

/// Exit status of a command that did what the player asked, wrong answers included.
pub const OK: i32 = 0;
/// Exit status of a command that could not do what was asked: no game, an unknown level, a
/// filesystem or state error, a declined `remove`.
pub const FAILURE: i32 = 1;
/// Exit status of a usage error.
pub const USAGE: i32 = 2;
/// Exit status of `cd-hook pre` when the level forbids the move.
pub const BLOCKED: i32 = 3;

/// The question `remove` asks before it deletes anything.
const REMOVE_PROMPT: &str = "Opravdu chcete odstranit všechna data ShellGame?";

// ---------------------------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------------------------

/// Why a command could not do its job, as opposed to the player getting something wrong. Shown as
/// a red panel; the exit status is [`FAILURE`].
#[derive(Debug)]
pub enum Failure {
    /// The state file could not be read, written or removed.
    State(StateError),
    /// A filesystem operation failed; the text says which.
    Filesystem(String),
}

impl From<StateError> for Failure {
    fn from(error: StateError) -> Self {
        Self::State(error)
    }
}

impl From<io::Error> for Failure {
    fn from(error: io::Error) -> Self {
        Self::Filesystem(error.to_string())
    }
}

impl From<FixtureError> for Failure {
    fn from(error: FixtureError) -> Self {
        Self::Filesystem(error.to_string())
    }
}

/// Why a command stops early.
#[derive(Debug)]
enum Halt {
    /// Stop with this exit status; the player has been told why already.
    Exit(i32),
    /// Something broke: report it as an error panel and exit with [`FAILURE`].
    Failed(Failure),
}

impl<E: Into<Failure>> From<E> for Halt {
    fn from(error: E) -> Self {
        Self::Failed(error.into())
    }
}

/// The result of a step that may stop the command.
type Flow<T = ()> = Result<T, Halt>;

/// Whether the saved level exists in the catalog (see [`Session::resync`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LevelSync {
    /// The saved level exists.
    Valid,
    /// The saved level was replaced by the nearest later one; the command's target is gone.
    Resynced,
    /// The catalog has no levels at all.
    Unavailable,
}

/// How a level was left behind by `advance`.
enum Outcome {
    /// The player completed it.
    Completed(LevelCompletion),
    /// The player skipped a bonus level.
    Skipped,
}

/// Whether the pages of an intro were all shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Paging {
    /// Every page was shown.
    Finished,
    /// The player pressed Ctrl-C at a prompt; the remaining pages were not shown.
    Interrupted,
}

/// The saved state of a running game and the level it points at.
struct Active<'a> {
    state: State,
    section: &'a Section,
    level: &'a Level,
}

/// What happened when the player was moved to the start of a level.
struct Arrival {
    /// The directory to announce, when the player's location changed.
    notice: Option<PathBuf>,
}

// ---------------------------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------------------------

/// One run of the game: the environment, the catalog, the state store and the output.
///
/// Build it with [`Session::new`] and call the method of the command. Methods print their
/// screens to standard output and their directives to standard error (inside the game shell
/// only) and return the exit status.
#[derive(Debug)]
pub struct Session<'a> {
    env: &'a Env,
    catalog: &'a Catalog,
    store: StateStore,
    term: Term,
    shell: Directives,
    /// Where the player's shell is (canonical), or `None` when unknown (the directory is gone).
    cwd: Option<PathBuf>,
    /// A level below which the player stands was prepared in this run, so `cwd` may have been
    /// deleted and recreated underneath the shell.
    cwd_unverified: bool,
}

impl<'a> Session<'a> {
    /// A session for this environment. Directives are emitted only inside the game shell
    /// ([`Env::wrapped`]); the state lives in [`Env::state_dir`].
    pub fn new(env: &'a Env, catalog: &'a Catalog) -> Self {
        Self {
            env,
            catalog,
            store: StateStore::new(&env.state_dir),
            term: Term::stdout(),
            shell: Directives::new(env.wrapped),
            cwd: env.cwd.clone(),
            cwd_unverified: false,
        }
    }

    /// The same session writing through a different [`Term`] (a fixed width, for example).
    #[must_use]
    pub fn with_term(mut self, term: Term) -> Self {
        self.term = term;
        self
    }

    // -----------------------------------------------------------------------------------------
    // Output and plumbing
    // -----------------------------------------------------------------------------------------

    /// Prints a screen. A closed or broken output is not worth failing a command for.
    fn print(&self, lines: Vec<String>) {
        let _ = print_lines(&self.term, &lines);
    }

    /// Runs one command and turns how it ended into an exit status.
    fn guarded(&mut self, command: impl FnOnce(&mut Self) -> Flow) -> i32 {
        match command(self) {
            Ok(()) => OK,
            Err(Halt::Exit(code)) => code,
            Err(Halt::Failed(failure)) => self.report(&failure),
        }
    }

    /// Shows a failure of the game itself as a red panel (the Python `show_state_error`).
    fn report(&self, failure: &Failure) -> i32 {
        let message = match failure {
            Failure::State(error) => error.to_string(),
            Failure::Filesystem(detail) if self.env.cwd.is_none() => {
                format!("{}\n\n{detail}", messages::CWD_MISSING)
            }
            Failure::Filesystem(detail) => messages::with_error(messages::FILESYSTEM_ERROR, detail),
        };
        self.print(screens::state_error(&message, &self.term));
        FAILURE
    }

    fn load(&self) -> Result<Option<State>, StateError> {
        self.store.load()
    }

    /// Loads the saved game or tells the player there is none.
    fn load_or_explain(&self) -> Flow<State> {
        let Some(state) = self.load()? else {
            self.print(screens::not_initialized(&self.term));
            return Err(Halt::Exit(FAILURE));
        };
        Ok(state)
    }

    /// The level the state points at, when the saved id is spelled canonically and exists.
    fn lookup(&self, state: &State) -> Option<(&'a Section, &'a Level)> {
        let id: LevelId = state.current_level.parse().ok()?;
        if id.to_string() != state.current_level {
            return None;
        }
        self.catalog.get(id)
    }

    /// The level the state points at, or a message that it does not exist.
    fn lookup_or_explain(&self, state: &State) -> Flow<(&'a Section, &'a Level)> {
        self.lookup(state).ok_or_else(|| {
            self.print(screens::level_not_found(&state.current_level, &self.term));
            Halt::Exit(FAILURE)
        })
    }

    /// A finished game is inert: show the completion screen instead of the command.
    fn show_if_finished(&self, state: &State) -> bool {
        if !state.is_finished() {
            return false;
        }
        self.print(screens::game_complete(state, &self.term));
        true
    }

    // -----------------------------------------------------------------------------------------
    // Preparing levels and placing the player
    // -----------------------------------------------------------------------------------------

    /// Puts a level into its starting state.
    ///
    /// Preparing can delete and recreate anything below the section directory, including the
    /// directory the player stands in, which leaves their shell in a directory that no longer
    /// exists even though its path is unchanged. So when the player is below the section directory
    /// their position is marked unverified, whether or not preparing succeeded; the next
    /// [`Session::teleport`] then asks the shell to `cd` even to the same path. (The section
    /// directory itself and the workspace are never deleted.)
    fn prepare(
        &mut self,
        section: &Section,
        level: &Level,
        workspace: &Path,
    ) -> Result<(), FixtureError> {
        let outcome = level.prepare(section, workspace);
        if let (Some(cwd), Ok(root)) = (&self.cwd, section.section_root(workspace)) {
            let root = soft_canonicalize(&root);
            if cwd.starts_with(&root) && *cwd != root {
                self.cwd_unverified = true;
            }
        }
        outcome
    }

    /// [`Session::prepare`] that tells the player when it fails; returns whether it worked.
    fn prepare_or_report(&mut self, section: &Section, level: &Level, workspace: &Path) -> bool {
        match self.prepare(section, level, workspace) {
            Ok(()) => true,
            Err(error) => {
                self.print(screens::level_setup_error(
                    &level.id.to_string(),
                    &error.to_string(),
                    &self.term,
                ));
                false
            }
        }
    }

    /// Exports the workspace and the level, which the old `cd` function of a Python-era game
    /// shell dispatches on.
    fn export_context(&self, workspace: &Path, level: LevelId) {
        // The names are constants that match the protocol's pattern, so this cannot fail.
        let _ = self.shell.export(ENV_WORKSPACE, workspace);
        let _ = self.shell.export(ENV_LEVEL, level.to_string());
    }

    /// Asks the shell to `cd` to `start` unless the player is there already. Returns the
    /// directory to announce when the player's location changed.
    fn teleport(&mut self, start: &Path) -> Option<PathBuf> {
        let target = soft_canonicalize(start);
        let there = self.cwd.as_deref() == Some(target.as_path());
        if there && !self.cwd_unverified {
            return None;
        }
        self.shell.cd(start);
        self.cwd_unverified = false;
        if !can_enter(start) {
            // `chdir` would have failed: the shell will report the problem, nothing changes here.
            return None;
        }
        self.cwd = Some(target);
        (!there).then(|| start.to_path_buf())
    }

    /// Moves the player to the start directory of the level, if it has one.
    fn move_to_start(
        &mut self,
        section: &Section,
        level: &Level,
        workspace: &Path,
    ) -> Option<PathBuf> {
        let root = section.section_root(workspace).ok()?;
        let start = level.start_dir(workspace, &root)?;
        self.teleport(&start)
    }

    /// Exports the level and moves the player to its start (`ensure_user_in_reasonable_place`).
    fn arrive(&mut self, state: &State, section: &Section, level: &Level) -> Arrival {
        self.export_context(&state.workspace, level.id);
        Arrival {
            notice: self.move_to_start(section, level, &state.workspace),
        }
    }

    // -----------------------------------------------------------------------------------------
    // Showing levels
    // -----------------------------------------------------------------------------------------

    /// Shows the assignment of a level: one panel for a regular level, and for an intro one panel
    /// per page with an Enter prompt between them.
    fn show_instructions(&self, level: &Level) -> Paging {
        if !level.is_intro() {
            clear_screen(&self.term);
            self.print(screens::level_panel(level, &self.term));
            return Paging::Finished;
        }
        let pages = split_pages(&level.text);
        let total = pages.len();
        for (index, page) in pages.iter().enumerate() {
            clear_screen(&self.term);
            self.print(screens::intro_page(
                level,
                page,
                index + 1,
                total,
                &self.term,
            ));
            if index + 1 < total && self.pause() == PromptOutcome::Interrupted {
                return Paging::Interrupted;
            }
        }
        Paging::Finished
    }

    /// The "press Enter" prompt. Nothing is asked when stdin or stdout is not a terminal.
    fn pause(&self) -> PromptOutcome {
        wait_for_enter(&enter_prompt(&self.term))
    }

    /// Shows a level the player has arrived at: its assignment, the teleport notice and, for an
    /// intro, the last Enter prompt followed by the automatic submission.
    fn present(&mut self, level: &Level, arrival: &Arrival) -> Flow {
        let paging = self.show_instructions(level);
        if let Some(destination) = &arrival.notice {
            self.print(screens::teleport_notice(
                &destination.to_string_lossy(),
                &self.term,
            ));
        }
        if level.is_intro()
            && paging == Paging::Finished
            && self.pause() != PromptOutcome::Interrupted
        {
            return self.submit_flow(None);
        }
        Ok(())
    }

    /// Starts the timer of the level and saves, when it was not running yet.
    fn start_level(&self, state: &mut State, level: &Level) -> Flow {
        if state.ensure_level_started(&level.id.to_string(), state::now()) {
            self.store.save(state)?;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------------------------
    // Starting and resuming
    // -----------------------------------------------------------------------------------------

    /// Creates a new game: state at `0.0`, workspace, first level.
    fn initialize_game(&mut self) -> Flow<State> {
        let workspace =
            state::default_workspace(self.env.workspace_override.as_deref(), &self.env.user);
        // A relative `$SHELLGAME_WORKSPACE` would name a different directory from every other
        // directory the player moves to, so the game keeps it absolute.
        let workspace = match &self.env.cwd {
            Some(cwd) if workspace.is_relative() => cwd.join(workspace),
            _ => workspace,
        };
        let state = State::new(&self.env.user, workspace, state::now());
        fs::create_dir_all(&state.workspace)?;

        let first = self.lookup(&state);
        if let Some((section, level)) = first {
            self.prepare_or_report(section, level, &state.workspace);
        }
        self.store.save(&state)?;
        self.print(screens::init_success(
            &state.username,
            &state.workspace.to_string_lossy(),
            &self.term,
        ));

        if let Some((section, level)) = first
            && let Ok(root) = section.section_root(&state.workspace)
            && let Some(start) = level.start_dir(&state.workspace, &root)
        {
            if let Some(destination) = self.teleport(&start) {
                self.print(screens::teleport_notice(
                    &destination.to_string_lossy(),
                    &self.term,
                ));
            }
            self.print(screens::current_directory(
                &start.to_string_lossy(),
                &self.term,
            ));
        }
        Ok(state)
    }

    /// Rebuilds a workspace that is gone (a cleared `/tmp`) and prepares the current level.
    fn restore_workspace_if_missing(&mut self, state: &State) -> Flow {
        if state.workspace.exists() {
            return Ok(());
        }
        fs::create_dir_all(&state.workspace)?;
        if let Some((section, level)) = self.lookup(state) {
            self.prepare_or_report(section, level, &state.workspace);
        }
        self.print(screens::workspace_restored(
            &state.workspace.to_string_lossy(),
            &self.term,
        ));
        Ok(())
    }

    /// Rescues a save whose `current_level` no longer exists. Levels get renumbered between
    /// versions; without this every command would report "level not found" and `remove` would be
    /// the only way out. The nearest later level takes over.
    fn resync(&mut self, state: &mut State) -> Flow<LevelSync> {
        if self.lookup(state).is_some() {
            return Ok(LevelSync::Valid);
        }
        let stale = state.current_level.clone();
        let Some(resolved) = self.catalog.resolve_str(&stale) else {
            self.print(screens::no_levels_available(&self.term));
            return Ok(LevelSync::Unavailable);
        };
        self.print(screens::level_missing(
            &stale,
            &resolved.to_string(),
            &self.term,
        ));
        state.current_level = resolved.to_string();
        state.ensure_level_started(&state.current_level.clone(), state::now());
        self.store.save(state)?;
        if let Some((section, level)) = self.catalog.get(resolved) {
            self.prepare_or_report(section, level, &state.workspace);
        }
        Ok(LevelSync::Resynced)
    }

    /// The preamble of every gameplay command: load the state, rescue a stale level, rebuild a
    /// missing workspace. A command aimed at a level that no longer exists is not applied to its
    /// replacement; the new assignment is shown instead.
    fn active(&mut self) -> Flow<Active<'a>> {
        let mut state = self.load_or_explain()?;
        match self.resync(&mut state)? {
            LevelSync::Valid => {}
            LevelSync::Unavailable => return Err(Halt::Exit(FAILURE)),
            LevelSync::Resynced => {
                self.current_level_flow()?;
                return Err(Halt::Exit(OK));
            }
        }
        self.restore_workspace_if_missing(&state)?;
        let (section, level) = self.lookup_or_explain(&state)?;
        Ok(Active {
            state,
            section,
            level,
        })
    }

    // -----------------------------------------------------------------------------------------
    // Commands
    // -----------------------------------------------------------------------------------------

    /// `shellgame`: shows the current level, creating the game on the first run. An intro is
    /// shown page by page and then submitted automatically.
    pub fn show_current_level(&mut self) -> i32 {
        self.guarded(Self::current_level_flow)
    }

    fn current_level_flow(&mut self) -> Flow {
        let mut state = match self.load()? {
            Some(state) => state,
            None => self.initialize_game()?,
        };
        if self.show_if_finished(&state) {
            return Ok(());
        }
        if self.resync(&mut state)? == LevelSync::Unavailable {
            return Err(Halt::Exit(FAILURE));
        }
        self.restore_workspace_if_missing(&state)?;
        let (section, level) = self.lookup_or_explain(&state)?;

        self.start_level(&mut state, level)?;
        let arrival = self.arrive(&state, section, level);
        self.present(level, &arrival)
    }

    /// `shellgame init`: the legacy explicit initialization.
    pub fn init(&mut self) -> i32 {
        self.guarded(|session| {
            if session.load()?.is_some() {
                session.print(screens::already_initialized(&session.term));
                return Ok(());
            }
            session.initialize_game().map(drop)
        })
    }

    /// `shellgame status`: progress and statistics.
    pub fn status(&mut self) -> i32 {
        self.guarded(|session| {
            let state = session.load_or_explain()?;
            session.print(screens::status(&state, &session.term));
            Ok(())
        })
    }

    /// `shellgame hint [--repeat]`: reveals the next hint, or lists the revealed ones. Only newly
    /// revealed hints are counted.
    pub fn hint(&mut self, repeat: bool) -> i32 {
        self.guarded(|session| session.hint_flow(repeat))
    }

    fn hint_flow(&mut self, repeat: bool) -> Flow {
        let Active {
            mut state, level, ..
        } = self.active()?;
        if self.show_if_finished(&state) {
            return Ok(());
        }
        let total = level.hints.len();
        if total == 0 {
            self.print(screens::no_hints_available(&self.term));
            return Ok(());
        }
        let id = level.id.to_string();
        if repeat {
            let revealed = state.hint_status(&id, total);
            self.print(screens::repeated_hints(&level.hints, revealed, &self.term));
            return Ok(());
        }
        match state.reveal_next_hint(&id, total) {
            None => self.print(screens::no_more_hints(&self.term)),
            Some(index) => {
                self.store.save(&state)?;
                self.print(screens::hint(&level.hints[index], index, total, &self.term));
            }
        }
        Ok(())
    }

    /// `shellgame submit [ANSWER]`: judges the answer. A wrong one is counted and explained; a
    /// right one is recorded and the game moves on.
    pub fn submit(&mut self, answer: Option<&str>) -> i32 {
        self.guarded(|session| session.submit_flow(answer))
    }

    fn submit_flow(&mut self, answer: Option<&str>) -> Flow {
        let Active {
            mut state,
            section,
            level,
        } = self.active()?;
        if self.show_if_finished(&state) {
            return Ok(());
        }
        let verdict = match section.ctx(level, &state.workspace, self.cwd.clone(), &self.env.home) {
            Ok(ctx) => level.validate(&ctx, answer),
            Err(error) => {
                // The workspace itself is broken (a symlinked section): not the player's fault, so
                // no attempt is counted.
                let message = format!("{} ({error})", messages::PATH_ESCAPES_WORKSPACE);
                self.print(screens::failure(&message, &self.term));
                return Err(Halt::Exit(FAILURE));
            }
        };
        let id = level.id.to_string();
        match verdict {
            Check::Fail(message) => {
                state.record_attempt(&id);
                self.store.save(&state)?;
                self.print(screens::failure(&message, &self.term));
                Ok(())
            }
            Check::Pass => {
                let completion = state.record_completion(&id, state::now());
                self.advance(state, level, Outcome::Completed(completion))
            }
        }
    }

    /// `shellgame skip`: skips a bonus level. No completion is recorded.
    pub fn skip(&mut self) -> i32 {
        self.guarded(Self::skip_flow)
    }

    fn skip_flow(&mut self) -> Flow {
        let Active { state, level, .. } = self.active()?;
        if self.show_if_finished(&state) {
            return Ok(());
        }
        if !level.is_bonus() {
            self.print(screens::not_skippable(&level.id.to_string(), &self.term));
            return Ok(());
        }
        self.print(screens::skipped(&level.id.to_string(), &self.term));
        self.advance(state, level, Outcome::Skipped)
    }

    /// Moves on from `level` with exactly one atomic save: the completion (if any),
    /// `current_level` and the timer of the next level are written together, before the next
    /// level is prepared, so a failing preparation never costs the player their progress.
    fn advance(&mut self, mut state: State, level: &Level, outcome: Outcome) -> Flow {
        let next = match self.catalog.next(level.id) {
            Ok(next) => next,
            Err(_) => {
                // Never mistake an unknown id for "the player finished".
                if self.resync(&mut state)? == LevelSync::Resynced {
                    self.current_level_flow()?;
                }
                return Ok(());
            }
        };

        let Some(next_id) = next else {
            state.completed_at = Some(state::now());
            self.store.save(&state)?;
            if let Outcome::Completed(completion) = &outcome {
                self.print(screens::success(
                    &level.success,
                    Some(completion),
                    &self.term,
                ));
            }
            self.print(screens::game_complete(&state, &self.term));
            return Ok(());
        };

        state.current_level = next_id.to_string();
        state.ensure_level_started(&state.current_level.clone(), state::now());
        self.store.save(&state)?;

        // `next` only knows ids of the catalog, so the lookup cannot miss.
        let (section, next_level) = self.catalog.get(next_id).ok_or(Halt::Exit(FAILURE))?;
        let arrival = self
            .prepare_or_report(section, next_level, &state.workspace)
            .then(|| self.arrive(&state, section, next_level));

        if let Outcome::Completed(completion) = &outcome
            && !level.is_intro()
        {
            self.print(screens::success(
                &level.success,
                Some(completion),
                &self.term,
            ));
            // Ctrl-C here is just another Enter: the next level is shown either way.
            let _ = self.pause();
        }
        match arrival {
            Some(arrival) => self.present(next_level, &arrival),
            None => Ok(()),
        }
    }

    /// `shellgame reset`: rebuilds the current level and puts the player back at its start,
    /// wherever they wandered off to.
    pub fn reset(&mut self) -> i32 {
        self.guarded(Self::reset_flow)
    }

    fn reset_flow(&mut self) -> Flow {
        let Active {
            state,
            section,
            level,
        } = self.active()?;
        if self.show_if_finished(&state) {
            return Ok(());
        }
        match self.prepare(section, level, &state.workspace) {
            Ok(()) => {}
            Err(error @ FixtureError::Io { .. }) => return Err(error.into()),
            Err(error) => {
                self.print(screens::reset_failed(&error.to_string(), &self.term));
                return Err(Halt::Exit(FAILURE));
            }
        }
        let arrival = self.arrive(&state, section, level);
        self.show_instructions(level);
        self.print(screens::reset_notice(&state.current_level, &self.term));
        if let Some(destination) = &arrival.notice {
            self.print(screens::teleport_notice(
                &destination.to_string_lossy(),
                &self.term,
            ));
        }
        Ok(())
    }

    /// `shellgame repeat [--section N] [--level ID]`: shows an assignment again (an explicit
    /// level wins over a section, and the current level is the default).
    pub fn repeat(&mut self, section: Option<u32>, level: Option<&str>) -> i32 {
        self.guarded(|session| {
            let state = session.load_or_explain()?;
            let wanted = match (level.filter(|id| !id.is_empty()), section) {
                (Some(id), _) => id.to_owned(),
                (None, Some(number)) => format!("{number}.0"),
                (None, None) => state.current_level.clone(),
            };
            session.show_level_by_id(&wanted)
        })
    }

    /// `shellgame show --section | --level`: shows the intro of the current section or the
    /// current level again. Exactly one flag is required.
    pub fn show(&mut self, section: bool, level: bool) -> i32 {
        self.guarded(|session| {
            let state = session.load_or_explain()?;
            if session.show_if_finished(&state) {
                return Ok(());
            }
            if section == level {
                session.print(screens::show_usage(&session.term));
                return Err(Halt::Exit(USAGE));
            }
            let target = if section {
                match state.current_level.split_once('.') {
                    Some((prefix, _))
                        if !prefix.is_empty() && prefix.bytes().all(|b| b.is_ascii_digit()) =>
                    {
                        format!("{prefix}.0")
                    }
                    _ => {
                        session.print(screens::invalid_level_format(
                            &state.current_level,
                            &session.term,
                        ));
                        return Err(Halt::Exit(FAILURE));
                    }
                }
            } else {
                state.current_level.clone()
            };
            session.show_level_by_id(&target)
        })
    }

    /// Shows the assignment of the level with this id, without touching the game.
    fn show_level_by_id(&self, id: &str) -> Flow {
        let found = id
            .parse::<LevelId>()
            .ok()
            .and_then(|parsed| self.catalog.get(parsed));
        let Some((_, level)) = found else {
            self.print(screens::level_not_found(id, &self.term));
            return Err(Halt::Exit(FAILURE));
        };
        self.show_instructions(level);
        Ok(())
    }

    /// `shellgame remove`: deletes the workspace and the state, after asking unless `confirmed`.
    /// Inside the game shell the shell is closed afterwards. A declined confirmation exits with
    /// [`FAILURE`], because the old wrapper closes the shell whenever `remove` returns 0.
    pub fn remove(&mut self, confirmed: bool) -> i32 {
        self.guarded(|session| session.remove_flow(confirmed))
    }

    fn remove_flow(&mut self, confirmed: bool) -> Flow {
        if !confirmed && !confirm(REMOVE_PROMPT) {
            self.print(screens::removal_declined(&self.term));
            return Err(Halt::Exit(FAILURE));
        }
        let workspace = match self.load() {
            Ok(Some(state)) => state.workspace,
            Ok(None) => {
                self.print(screens::not_initialized(&self.term));
                return Err(Halt::Exit(FAILURE));
            }
            // The state is unreadable, so the workspace cannot be read from it: fall back to the
            // default location for this user and remove the broken file too.
            Err(StateError::Load { .. }) => {
                state::default_workspace(self.env.workspace_override.as_deref(), &self.env.user)
            }
            Err(error) => return Err(error.into()),
        };
        self.remove_workspace(&workspace)?;
        self.store.remove()?;
        self.print(screens::removed(&self.term));
        self.shell.exit();
        Ok(())
    }

    /// Deletes a workspace, but only when it is strictly inside `/tmp` or `$TMPDIR` (after
    /// resolving symlinks): a corrupt or edited state must never point `remove` at real data.
    fn remove_workspace(&self, workspace: &Path) -> Flow {
        // Nothing to delete (or nothing reachable) when the path does not resolve.
        let Ok(real) = fs::canonicalize(workspace) else {
            return Ok(());
        };
        if !is_removable(&real, self.env.tmpdir.as_deref()) {
            self.print(screens::removal_refused(
                &workspace.to_string_lossy(),
                &self.term,
            ));
            return Err(Halt::Exit(FAILURE));
        }
        remove_tree(workspace)?;
        Ok(())
    }

    /// `shellgame exit`: closes the game shell.
    pub fn exit(&self) -> i32 {
        self.print(screens::exiting(&self.term));
        self.shell.exit();
        OK
    }

    // -----------------------------------------------------------------------------------------
    // Developer commands
    // -----------------------------------------------------------------------------------------

    /// `shellgame dev jump ID`: prepares and shows any level. Its progress is cleared, so a level
    /// can be replayed from scratch.
    pub fn dev_jump(&mut self, id: &str) -> i32 {
        self.guarded(|session| session.dev_jump_flow(id))
    }

    fn dev_jump_flow(&mut self, id: &str) -> Flow {
        let mut state = self.load_or_explain()?;
        let found = id
            .parse::<LevelId>()
            .ok()
            .and_then(|parsed| self.catalog.get(parsed));
        let Some((section, level)) = found else {
            self.print(screens::unknown_level(id, &self.term));
            return Err(Halt::Exit(FAILURE));
        };
        if !self.prepare_or_report(section, level, &state.workspace) {
            return Err(Halt::Exit(FAILURE));
        }
        let canonical = level.id.to_string();
        state.reset_level_progress(&canonical, state::now());
        state.current_level = canonical;
        self.store.save(&state)?;
        self.current_level_flow()
    }

    /// `shellgame dev next`: jumps to the level after the current one.
    pub fn dev_next(&mut self) -> i32 {
        self.guarded(|session| session.dev_step(Step::Next))
    }

    /// `shellgame dev prev`: jumps to the level before the current one.
    pub fn dev_prev(&mut self) -> i32 {
        self.guarded(|session| session.dev_step(Step::Previous))
    }

    fn dev_step(&mut self, step: Step) -> Flow {
        let mut state = self.load_or_explain()?;
        let neighbour = state
            .current_level
            .parse::<LevelId>()
            .ok()
            .map(|id| match step {
                Step::Next => self.catalog.next(id),
                Step::Previous => self.catalog.prev(id),
            });
        match neighbour {
            Some(Ok(Some(target))) => self.dev_jump_flow(&target.to_string()),
            Some(Ok(None)) => {
                self.print(match step {
                    Step::Next => screens::no_next_level(&self.term),
                    Step::Previous => screens::no_previous_level(&self.term),
                });
                Ok(())
            }
            // An unknown current level: rescue it rather than guess a neighbour.
            _ => {
                if self.resync(&mut state)? == LevelSync::Resynced {
                    self.current_level_flow()?;
                }
                Ok(())
            }
        }
    }

    /// `shellgame dev reload`: rebuilds the current level and shows it again.
    pub fn dev_reload(&mut self) -> i32 {
        self.guarded(Self::dev_reload_flow)
    }

    fn dev_reload_flow(&mut self) -> Flow {
        let state = self.load_or_explain()?;
        let (section, level) = self.lookup_or_explain(&state)?;
        self.prepare(section, level, &state.workspace)?;
        let _ = self.pause();
        self.show_instructions(level);
        self.print(screens::level_reloaded(&state.current_level, &self.term));
        Ok(())
    }

    /// `shellgame dev start`: jumps to the first level.
    pub fn dev_start(&mut self) -> i32 {
        self.guarded(|session| {
            let Some(first) = session.catalog.first() else {
                session.print(screens::no_levels_registered(&session.term));
                return Err(Halt::Exit(FAILURE));
            };
            session.dev_jump_flow(&first.to_string())
        })
    }

    // -----------------------------------------------------------------------------------------
    // Hooks
    // -----------------------------------------------------------------------------------------

    /// The saved game and its current level, for a hook. Hooks fail open: no state, an unreadable
    /// state or an unknown level means "nothing to enforce", never an error in the player's `cd`.
    fn hook_context(&self) -> Option<Active<'a>> {
        let state = self.load().ok().flatten()?;
        let (section, level) = self.lookup(&state)?;
        Some(Active {
            state,
            section,
            level,
        })
    }

    /// Asks the current level whether a move is allowed. A block is explained on stderr, in red
    /// even though the wrapper captures stderr, and the exit status is `blocked`.
    fn cd_decision(
        &self,
        blocked: i32,
        decide: impl FnOnce(&Level, &Ctx<'_>) -> CdDecision,
    ) -> i32 {
        let Some(Active {
            state,
            section,
            level,
        }) = self.hook_context()
        else {
            return OK;
        };
        let Ok(ctx) = section.ctx(level, &state.workspace, self.cwd.clone(), &self.env.home) else {
            return OK;
        };
        match decide(level, &ctx) {
            CdDecision::Allow => OK,
            CdDecision::Block(message) => {
                let term = Term::stderr_forced();
                let _ = print_lines(&term, &screens::cd_block(level.id, &message, &term));
                blocked
            }
        }
    }

    /// `cd-hook pre <pwd> <prev> -- <cd args...>`: may the player `cd`? [`BLOCKED`] if not.
    ///
    /// `pwd` is the shell's directory before the move and `prev` the directory `cd -` would go to
    /// (empty when unknown); a relative or empty value counts as unknown.
    pub fn cd_pre(&self, pwd: &Path, prev: &Path, cd_args: &[OsString]) -> i32 {
        let args = lossy(cd_args);
        let target = cd_target(&args);
        let prev = Some(prev).filter(|dir| dir.is_absolute());
        self.cd_decision(BLOCKED, |level, ctx| {
            let mv = CdMove::new(ctx, target, &args, pwd).with_prev(prev);
            level.cd_pre_move(&mv)
        })
    }

    /// `cd-hook post <old pwd> <new pwd> -- <cd args...>`: the move happened; records evidence.
    /// Silent, and ignored when the new directory is not an absolute path.
    pub fn cd_post(&self, old: &Path, new: &Path, cd_args: &[OsString]) -> i32 {
        if !new.is_absolute() {
            return OK;
        }
        let args = lossy(cd_args);
        let target = cd_target(&args);
        self.cd_decision(BLOCKED, |level, ctx| {
            let mv = CdMove::new(ctx, target, &args, old).with_dest(new);
            level.cd_post_move(&mv);
            CdDecision::Allow
        })
    }

    /// Legacy `cd-hook [TARGET] [PWD]` of a game shell started by the Python version: the move is
    /// reported only before it happens. A block exits with [`FAILURE`], because the old wrapper
    /// treats every non-zero status as "blocked".
    pub fn legacy_cd_pre(&self, target: Option<&OsStr>, pwd: Option<&OsStr>) -> i32 {
        let target = target
            .map(|text| text.to_string_lossy().into_owned())
            .unwrap_or_default();
        let args: Vec<String> = if target.is_empty() {
            Vec::new()
        } else {
            vec![target.clone()]
        };
        let pwd = Path::new(pwd.unwrap_or_default());
        self.cd_decision(FAILURE, |level, ctx| {
            let mv = CdMove::new(ctx, &target, &args, pwd);
            level.cd_legacy_pre_move(&mv)
        })
    }

    /// Legacy `cd-hook --post-move [PWD]`: the move happened and `pwd` is where the shell is now.
    pub fn legacy_cd_post(&self, pwd: Option<&OsStr>) -> i32 {
        let Some(dest) = pwd.map(Path::new).filter(|dir| dir.is_absolute()) else {
            return OK;
        };
        self.cd_decision(FAILURE, |level, ctx| {
            level.cd_legacy_post_move(ctx, dest);
            CdDecision::Allow
        })
    }

    /// `fd-hook`: runs inside a redirected command and looks at where its own stdout and stderr
    /// point. Silent.
    pub fn fd_hook(&self) -> i32 {
        if let Some(Active { state, level, .. }) = self.hook_context()
            && let Ok(fds) = Fds::from_process()
        {
            let markers = Markers::new(&state.workspace, level.id);
            // Evidence is best effort: a marker that cannot be written leaves the level incomplete.
            let _ = level.record_fd(&fds, &markers);
        }
        OK
    }

    // -----------------------------------------------------------------------------------------
    // Update
    // -----------------------------------------------------------------------------------------

    /// `shellgame update`: replaces this binary with the latest release (see [`update`]).
    pub fn update(&self) -> i32 {
        let old = crate::cli::version_line();
        self.print(screens::plain_line(
            "Hledám nejnovější verzi ShellGame...",
            style::DIM,
            &self.term,
        ));
        match update::run(self.env, &old) {
            Ok(update::Outcome::Updated { new }) => {
                self.print(screens::update_done(&old, &new, &self.term));
                OK
            }
            Ok(update::Outcome::Current) => {
                self.print(screens::update_current(&old, &self.term));
                OK
            }
            Err(problem) => {
                self.print(screens::update_failed(
                    &problem.to_string(),
                    problem.suggests_installer(),
                    &self.term,
                ));
                FAILURE
            }
        }
    }
}

/// Which neighbour `dev next` and `dev prev` jump to.
#[derive(Clone, Copy)]
enum Step {
    Next,
    Previous,
}

// ---------------------------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------------------------

/// The arguments of a `cd` call as text (invalid UTF-8 is replaced, as the levels compare text).
fn lossy(args: &[OsString]) -> Vec<String> {
    args.iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

/// The directory a `cd` call names: its first non-option argument.
///
/// Options (`-L`, `-P`, `-e`, `-@`, ...) are skipped; `--` ends them, so `cd -- -P` goes to a
/// directory named `-P`; a lone `-` is the target (`cd -`); a bare `cd` has the target `""`.
///
/// ```
/// use shellgame::session::cd_target;
///
/// let args = |list: &[&str]| list.iter().map(|a| a.to_string()).collect::<Vec<_>>();
/// assert_eq!(cd_target(&args(&["-P", "..", "ignored"])), "..");
/// assert_eq!(cd_target(&args(&["-"])), "-");
/// assert_eq!(cd_target(&args(&["--", "-P"])), "-P");
/// assert_eq!(cd_target(&args(&[])), "");
/// ```
pub fn cd_target(args: &[String]) -> &str {
    let mut options_ended = false;
    for arg in args {
        if options_ended {
            return arg;
        }
        match arg.as_str() {
            "--" => options_ended = true,
            option if option.starts_with('-') && option.len() > 1 => {}
            target => return target,
        }
    }
    ""
}

/// The directories `shellgame remove` may delete from, resolved: `/tmp` and `$TMPDIR` (on macOS
/// `$TMPDIR` is not `/tmp`, and `/tmp` itself is a symlink).
fn removal_roots(tmpdir: Option<&Path>) -> Vec<PathBuf> {
    std::iter::once(Path::new("/tmp"))
        .chain(tmpdir)
        .filter_map(|root| fs::canonicalize(root).ok())
        .collect()
}

/// Whether `real` (a canonical path) lies strictly inside one of the `roots`: the root itself
/// never qualifies.
fn is_strictly_inside(real: &Path, roots: &[PathBuf]) -> bool {
    roots
        .iter()
        .any(|root| real != root && real.starts_with(root))
}

/// Whether `shellgame remove` may delete this (canonical) workspace.
fn is_removable(real: &Path, tmpdir: Option<&Path>) -> bool {
    is_strictly_inside(real, &removal_roots(tmpdir))
}

/// Whether the shell could `cd` into this directory: it exists and may be searched.
fn can_enter(path: &Path) -> bool {
    path.is_dir() && rustix::fs::access(path, rustix::fs::Access::EXEC_OK).is_ok()
}

// ---------------------------------------------------------------------------------------------
// Self-update
// ---------------------------------------------------------------------------------------------

/// `shellgame update`: downloads the release archive for this platform, checks that the new
/// binary runs and replaces the running one.
///
/// The archive is `shellgame-<linux|macos>-<x86_64|aarch64>.tar.gz` from the latest GitHub
/// release (or `$SHELLGAME_UPDATE_URL`, which tests point at a `file://` archive). `curl` and
/// `tar` do the work, which keeps TLS out of the binary. The download goes into a temporary
/// directory *next to* the running binary, so the final `rename` stays on one filesystem and
/// replaces the binary atomically; a running binary can be replaced on Linux and macOS.
pub mod update {
    use std::fmt;
    use std::fs;
    use std::io;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use crate::env::Env;

    /// Where releases are published; the archive name is appended.
    const RELEASE_BASE: &str = "https://github.com/jdupak/ShellGame/releases/latest/download";

    /// The name of the binary inside the archive.
    const BINARY_NAME: &str = "shellgame";

    /// The archive of this platform, chosen at compile time.
    pub fn asset_name() -> String {
        let os = if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };
        let arch = if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86_64"
        };
        format!("shellgame-{os}-{arch}.tar.gz")
    }

    /// The download address: `$SHELLGAME_UPDATE_URL`, or the latest release.
    pub fn download_url(env: &Env) -> String {
        env.update_url
            .clone()
            .unwrap_or_else(|| format!("{RELEASE_BASE}/{}", asset_name()))
    }

    /// How an update ended when nothing went wrong.
    #[derive(Debug, PartialEq, Eq)]
    pub enum Outcome {
        /// The binary was replaced; the new one reports this version line.
        Updated {
            /// The version line printed by the new binary.
            new: String,
        },
        /// The downloaded binary is the one already running.
        Current,
    }

    /// Why an update failed. The messages are Czech and say what to do.
    #[derive(Debug)]
    pub enum Problem {
        /// The location of this binary is unknown.
        UnknownBinary,
        /// The directory of the binary cannot be written to.
        NotWritable {
            /// The directory.
            dir: PathBuf,
            /// Why.
            error: io::Error,
        },
        /// `curl` or `tar` is not installed.
        ToolMissing(&'static str),
        /// The download failed.
        Download {
            /// The address.
            url: String,
            /// What `curl` said.
            detail: String,
        },
        /// The archive could not be unpacked.
        Extract(String),
        /// The downloaded binary does not run.
        Unusable(String),
        /// The binary could not be replaced.
        Replace {
            /// The binary.
            exe: PathBuf,
            /// Why.
            error: io::Error,
        },
    }

    impl Problem {
        /// Whether rerunning the installer would help.
        pub fn suggests_installer(&self) -> bool {
            matches!(
                self,
                Self::NotWritable { .. } | Self::ToolMissing(_) | Self::Replace { .. }
            )
        }
    }

    impl fmt::Display for Problem {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::UnknownBinary => {
                    f.write_str("Nepodařilo se zjistit, kde je ShellGame nainstalován.")
                }
                Self::NotWritable { dir, error } => write!(
                    f,
                    "Do adresáře {} nelze zapisovat, ShellGame se proto nemůže sám aktualizovat ({error}).",
                    dir.display()
                ),
                Self::ToolMissing(tool) => write!(
                    f,
                    "Pro aktualizaci je potřeba příkaz '{tool}', ale nebyl nalezen."
                ),
                Self::Download { url, detail } => {
                    write!(f, "Stažení {url} se nepodařilo: {detail}")
                }
                Self::Extract(detail) => {
                    write!(f, "Rozbalení staženého archivu se nepodařilo: {detail}")
                }
                Self::Unusable(detail) => {
                    write!(f, "Stažený program nelze spustit: {detail}")
                }
                Self::Replace { exe, error } => {
                    write!(
                        f,
                        "Nahrazení programu {} se nepodařilo: {error}",
                        exe.display()
                    )
                }
            }
        }
    }

    impl std::error::Error for Problem {}

    /// Runs one external tool and returns its stdout; a tool that is not installed is
    /// `ToolMissing`, one that fails carries its stderr.
    fn run_tool(
        tool: &'static str,
        command: &mut Command,
    ) -> Result<String, Result<Problem, String>> {
        let output = command
            .stdin(Stdio::null())
            .output()
            .map_err(|error| match error.kind() {
                io::ErrorKind::NotFound => Ok(Problem::ToolMissing(tool)),
                _ => Err(error.to_string()),
            })?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        Err(Err(if detail.is_empty() {
            format!("{tool} skončil se stavem {}", output.status)
        } else {
            detail.to_owned()
        }))
    }

    /// Downloads, unpacks, checks and installs the new binary. `old` is the version line of the
    /// running binary.
    ///
    /// # Errors
    ///
    /// A [`Problem`] that says in Czech what went wrong and, where the installer helps, how to
    /// rerun it.
    pub fn run(env: &Env, old: &str) -> Result<Outcome, Problem> {
        let exe = env.exe.as_deref().ok_or(Problem::UnknownBinary)?;
        let dir = exe.parent().ok_or(Problem::UnknownBinary)?;
        let url = download_url(env);

        let scratch = tempfile::Builder::new()
            .prefix(".shellgame-update-")
            .tempdir_in(dir)
            .map_err(|error| Problem::NotWritable {
                dir: dir.to_path_buf(),
                error,
            })?;
        let archive = scratch.path().join(asset_name());

        run_tool(
            "curl",
            Command::new("curl")
                .args(["-fsSL", "-o"])
                .arg(&archive)
                .arg(&url),
        )
        .map_err(|failure| {
            failure.unwrap_or_else(|detail| Problem::Download {
                url: url.clone(),
                detail,
            })
        })?;
        run_tool(
            "tar",
            Command::new("tar")
                .arg("-xzf")
                .arg(&archive)
                .arg("-C")
                .arg(scratch.path())
                .arg(BINARY_NAME),
        )
        .map_err(|failure| failure.unwrap_or_else(Problem::Extract))?;

        let fresh = scratch.path().join(BINARY_NAME);
        fs::set_permissions(&fresh, fs::Permissions::from_mode(0o755))
            .map_err(|error| Problem::Extract(error.to_string()))?;
        let new = version_of(&fresh)?;
        if new == old {
            return Ok(Outcome::Current);
        }

        fs::rename(&fresh, exe).map_err(|error| Problem::Replace {
            exe: exe.to_path_buf(),
            error,
        })?;
        Ok(Outcome::Updated { new })
    }

    /// The version line a binary prints for `--version`; the binary must run and succeed.
    fn version_of(binary: &Path) -> Result<String, Problem> {
        let output = Command::new(binary)
            .arg("--version")
            .stdin(Stdio::null())
            .output()
            .map_err(|error| Problem::Unusable(error.to_string()))?;
        if !output.status.success() {
            return Err(Problem::Unusable(format!(
                "`--version` skončilo se stavem {}",
                output.status
            )));
        }
        let text = String::from_utf8_lossy(&output.stdout);
        match text.lines().next().map(str::trim) {
            Some(line) if !line.is_empty() => Ok(line.to_owned()),
            _ => Err(Problem::Unusable("`--version` nic nevypsalo".to_owned())),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn the_asset_name_encodes_this_platform() {
            let name = asset_name();
            assert!(name.starts_with("shellgame-"));
            assert!(name.ends_with(".tar.gz"));
            assert!(name.contains(if cfg!(target_os = "macos") {
                "macos"
            } else {
                "linux"
            }));
            assert!(name.contains(if cfg!(target_arch = "aarch64") {
                "aarch64"
            } else {
                "x86_64"
            }));
        }

        #[test]
        fn the_url_defaults_to_the_latest_release_and_can_be_overridden() {
            let mut env = Env::new("/home/ada", "ada", "/state");
            assert_eq!(
                download_url(&env),
                format!(
                    "https://github.com/jdupak/ShellGame/releases/latest/download/{}",
                    asset_name()
                )
            );
            env.update_url = Some("file:///tmp/x.tar.gz".to_owned());
            assert_eq!(download_url(&env), "file:///tmp/x.tar.gz");
        }

        #[test]
        fn problems_explain_themselves_in_czech() {
            let not_writable = Problem::NotWritable {
                dir: PathBuf::from("/usr/local/bin"),
                error: io::Error::from(io::ErrorKind::PermissionDenied),
            };
            assert!(not_writable.to_string().contains("/usr/local/bin"));
            assert!(not_writable.suggests_installer());
            assert!(Problem::ToolMissing("curl").suggests_installer());
            assert!(Problem::ToolMissing("curl").to_string().contains("'curl'"));
            assert!(
                !Problem::Download {
                    url: "u".into(),
                    detail: "d".into()
                }
                .suggests_installer()
            );
            assert!(!Problem::UnknownBinary.suggests_installer());
        }

        #[test]
        fn an_environment_without_a_binary_cannot_update() {
            let env = Env::new("/home/ada", "ada", "/state");
            assert!(matches!(run(&env, "x"), Err(Problem::UnknownBinary)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn the_cd_target_is_the_first_non_option_argument() {
        assert_eq!(cd_target(&args(&[])), "");
        assert_eq!(cd_target(&args(&[".."])), "..");
        assert_eq!(cd_target(&args(&["-L", "-P", "dir"])), "dir");
        assert_eq!(cd_target(&args(&["-e", "-@", "-P", "a", "b"])), "a");
        assert_eq!(cd_target(&args(&["-P"])), "");
        assert_eq!(cd_target(&args(&["a", "b"])), "a");
    }

    #[test]
    fn a_lone_dash_is_the_target() {
        assert_eq!(cd_target(&args(&["-"])), "-");
        assert_eq!(cd_target(&args(&["-L", "-"])), "-");
    }

    #[test]
    fn double_dash_ends_the_options() {
        assert_eq!(cd_target(&args(&["--"])), "");
        assert_eq!(cd_target(&args(&["--", "-P"])), "-P");
        assert_eq!(cd_target(&args(&["--", "--"])), "--");
        assert_eq!(cd_target(&args(&["-P", "--", "-weird", "other"])), "-weird");
    }

    #[test]
    fn an_empty_argument_is_a_target_of_its_own() {
        // `cd ""` is not a bare `cd`: the shell treats it as the current directory.
        assert_eq!(cd_target(&args(&[""])), "");
        assert_eq!(cd_target(&args(&["", "x"])), "");
    }

    #[test]
    fn a_directory_named_pre_is_an_ordinary_target() {
        assert_eq!(cd_target(&args(&["pre"])), "pre");
    }

    #[test]
    fn lossy_keeps_valid_text_and_replaces_the_rest() {
        use std::os::unix::ffi::OsStringExt;
        let args = [OsString::from("ok"), OsString::from_vec(b"a\xffb".to_vec())];
        assert_eq!(lossy(&args), ["ok", "a\u{fffd}b"]);
    }

    #[test]
    fn a_workspace_must_be_strictly_inside_a_removal_root() {
        let roots = [PathBuf::from("/tmp"), PathBuf::from("/var/folders/ab/T")];
        assert!(is_strictly_inside(Path::new("/tmp/shellgame-ada"), &roots));
        assert!(is_strictly_inside(
            Path::new("/var/folders/ab/T/shellgame-ada"),
            &roots
        ));
        assert!(
            !is_strictly_inside(Path::new("/tmp"), &roots),
            "never the root itself"
        );
        assert!(!is_strictly_inside(Path::new("/var/folders/ab/T"), &roots));
        assert!(!is_strictly_inside(Path::new("/"), &roots));
        assert!(!is_strictly_inside(Path::new("/etc"), &roots));
        assert!(!is_strictly_inside(Path::new("/home/ada"), &roots));
        // Siblings that merely share a prefix are outside.
        assert!(!is_strictly_inside(Path::new("/tmpfoo/x"), &roots));
        assert!(!is_strictly_inside(Path::new("/tmp"), &[]));
    }

    #[test]
    fn the_removal_roots_are_resolved_through_symlinks() {
        let tmp = tempfile::tempdir().unwrap();
        let real = fs::canonicalize(tmp.path()).unwrap();
        let link = real.join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let roots = removal_roots(Some(&link));
        assert!(roots.contains(&real), "{roots:?}");
        assert!(roots.iter().all(|root| root.is_absolute()));
        // A missing $TMPDIR is simply not a root.
        assert_eq!(
            removal_roots(Some(&real.join("missing"))),
            removal_roots(None)
        );
    }

    #[test]
    fn removal_accepts_paths_below_slash_tmp() {
        let tmp = tempfile::Builder::new()
            .prefix("sg-session-test-")
            .tempdir_in("/tmp")
            .unwrap();
        let real = fs::canonicalize(tmp.path()).unwrap();
        assert!(is_removable(&real, None));
        assert!(is_removable(&real.join("deeper/still"), None));
        assert!(!is_removable(Path::new("/etc"), None));
        assert!(!is_removable(Path::new("/tmp"), None), "never /tmp itself");
    }

    #[test]
    fn can_enter_needs_a_searchable_directory() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        assert!(can_enter(tmp.path()));
        assert!(!can_enter(&tmp.path().join("missing")));
        let file = tmp.path().join("file");
        fs::write(&file, "x").unwrap();
        assert!(!can_enter(&file), "a file is not a directory");

        let locked = tmp.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        // root can enter anything, so the unprivileged case is only checked for everyone else.
        let privileged = fs::read_dir(&locked).is_ok();
        let enterable = can_enter(&locked);
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(enterable, privileged);
    }

    // --- where the player is ------------------------------------------------------------------

    use crate::engine::prelude::{exact, level};

    /// One section with one level that starts in `a` and has the directory `a/b`.
    fn tiny_catalog() -> Catalog {
        Catalog::new(vec![
            Section::new(1, "level-1").levels([level(1, "Start")
                .dirs(["a/b"])
                .start("a")
                .answer(exact("x"))
                .solution("shellgame submit x")]),
        ])
    }

    /// A canonical sandbox: its path, an environment inside it and the workspace path.
    fn sandbox() -> (tempfile::TempDir, PathBuf, Env, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let base = fs::canonicalize(tmp.path()).unwrap();
        let env = Env::new(&base, "ada", base.join("state"));
        let workspace = base.join("ws");
        (tmp, base, env, workspace)
    }

    #[test]
    fn a_player_elsewhere_is_moved_once_and_remembered() {
        let (_tmp, base, mut env, _) = sandbox();
        let elsewhere = base.join("elsewhere");
        let start = base.join("start");
        fs::create_dir_all(&elsewhere).unwrap();
        fs::create_dir_all(&start).unwrap();
        env.cwd = Some(elsewhere);
        let catalog = tiny_catalog();
        let mut session = Session::new(&env, &catalog);

        assert_eq!(session.teleport(&start), Some(start.clone()), "announced");
        assert_eq!(session.cwd.as_deref(), Some(start.as_path()));
        assert_eq!(
            session.teleport(&start),
            None,
            "a second check neither moves nor announces"
        );
    }

    #[test]
    fn a_player_without_a_directory_is_always_moved() {
        let (_tmp, base, env, _) = sandbox();
        let start = base.join("start");
        fs::create_dir_all(&start).unwrap();
        let catalog = tiny_catalog();
        let mut session = Session::new(&env, &catalog);
        assert_eq!(env.cwd, None);

        assert_eq!(session.teleport(&start), Some(start));
    }

    #[test]
    fn an_unverified_position_is_refreshed_without_a_notice() {
        let (_tmp, base, mut env, _) = sandbox();
        let start = base.join("start");
        fs::create_dir_all(&start).unwrap();
        env.cwd = Some(start.clone());
        let catalog = tiny_catalog();
        let mut session = Session::new(&env, &catalog);

        assert_eq!(session.teleport(&start), None, "verified and there");
        session.cwd_unverified = true;
        assert_eq!(session.teleport(&start), None, "the same place: no notice");
        assert!(
            !session.cwd_unverified,
            "but the shell was asked to cd, which settles it"
        );
    }

    #[test]
    fn moving_to_a_directory_that_cannot_be_entered_changes_nothing_here() {
        let (_tmp, base, mut env, _) = sandbox();
        let elsewhere = base.join("elsewhere");
        fs::create_dir_all(&elsewhere).unwrap();
        env.cwd = Some(elsewhere.clone());
        let catalog = tiny_catalog();
        let mut session = Session::new(&env, &catalog);

        assert_eq!(session.teleport(&base.join("missing")), None);
        assert_eq!(session.cwd.as_deref(), Some(elsewhere.as_path()));
    }

    #[test]
    fn preparing_a_level_distrusts_a_position_below_its_section_only() {
        let (_tmp, _base, mut env, workspace) = sandbox();
        let catalog = tiny_catalog();
        let (section, level) = catalog.get(LevelId::new(1, 1)).unwrap();
        let root = workspace.join("level-1");
        let below = root.join("a/b");

        for (cwd, unverified) in [
            (Some(below), true),
            (Some(root.clone()), false),
            (Some(workspace.clone()), false),
            (Some(workspace.join("level-2/x")), false),
            (None, false),
        ] {
            env.cwd = cwd.clone();
            let mut session = Session::new(&env, &catalog);
            session.prepare(section, level, &workspace).unwrap();
            assert_eq!(session.cwd_unverified, unverified, "{cwd:?}");
        }
    }

    #[test]
    fn a_saved_level_id_must_be_spelled_canonically() {
        let (_tmp, _base, env, workspace) = sandbox();
        let catalog = tiny_catalog();
        let session = Session::new(&env, &catalog);
        let mut state = State::new("ada", &workspace, state::now());

        for (saved, found) in [
            ("1.1", true),
            ("01.1", false),
            ("1.01", false),
            ("1.2", false),
            ("2.1", false),
            ("x", false),
            ("", false),
        ] {
            state.current_level = saved.to_owned();
            assert_eq!(session.lookup(&state).is_some(), found, "{saved:?}");
        }
    }

    #[test]
    fn exit_statuses_are_the_documented_ones() {
        assert_eq!((OK, FAILURE, USAGE, BLOCKED), (0, 1, 2, 3));
    }
}
