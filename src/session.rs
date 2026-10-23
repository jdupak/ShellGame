//! What every `shellgame` command does, on top of the course ([`crate::game`]), the saved game
//! ([`crate::state`]) and the screens ([`crate::ui`]).
//!
//! The wrapping shell is asked to export variables and to `cd` with directives, which are emitted
//! *before* any Enter prompt, so a Ctrl-C at the prompt (which ends it like Enter) cannot lose them.
//! The player's directory is tracked in memory and updated whenever the game moves them, so a
//! second check in the same run neither repeats the directive nor the notice.

mod update;

use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use anstyle::Style;

use crate::cli::version_line;
use crate::env::Env;
use crate::game::paths::{canonical, inside_tmp};
use crate::game::{self, CdMove, Ctx, Pos, cd, dedent, messages};
use crate::levels::COURSE;
use crate::shell::{Directives, ENV_WORKSPACE};
use crate::state::{self, Progress, Saved, State};
use crate::ui::{
    DIM, DIM_VIOLET, GREEN, Line, RED, Term, YELLOW, markdown, prompt, screens, span, words,
};

pub const OK: i32 = 0;
/// A command that could not do what was asked: no game, an unknown level, an error, a declined `remove`.
pub const FAILURE: i32 = 1;
/// A `cd` or `chmod` that is refused.
pub const BLOCKED: i32 = 3;

const FIRST: Pos = Pos { section: 0, level: 1 };

/// Why a command stops early.
pub enum Halt {
    /// The player has been told already.
    Exit(i32),
    /// Something broke: a red panel, status [`FAILURE`].
    Fail(String),
}

impl From<io::Error> for Halt {
    fn from(error: io::Error) -> Halt {
        let damaged = error.kind() == io::ErrorKind::InvalidData;
        Halt::Fail(if damaged {
            error.to_string()
        } else {
            messages::fill(messages::FILESYSTEM_ERROR, error)
        })
    }
}

pub type Flow<T = ()> = Result<T, Halt>;

/// The workspace holds what the game prepares and deletes: it must be inside `/tmp` and the player's own.
fn check_workspace(workspace: &Path) -> Result<(), String> {
    if !inside_tmp(workspace) {
        return Err(format!("Pracovní prostor musí být uvnitř /tmp: {}", workspace.display()));
    }
    match fs::symlink_metadata(workspace) {
        Ok(meta)
            if meta.file_type().is_symlink()
                || meta.uid() != rustix::process::getuid().as_raw() =>
        {
            Err(format!(
                "Pracovní prostor {} musí být adresář, který patří vám (ne odkaz).",
                workspace.display()
            ))
        }
        _ => Ok(()),
    }
}

/// Checks the workspace, makes it a usable directory (created when missing, repaired after whatever
/// the player did to it) and checks the result.
fn ensure_workspace(workspace: &Path) -> Flow {
    check_workspace(workspace).map_err(Halt::Fail)?;
    game::ensure_dir(workspace)?;
    check_workspace(workspace).map_err(Halt::Fail)
}

/// The nearest level at or after a saved position that no longer exists; the last level if none follows.
fn resolve(saved: &str) -> Pos {
    let number = |digits: &str| digits.parse().ok();
    let (section, level) =
        saved.split_once('.').and_then(|(s, l)| Some((number(s)?, number(l)?))).unwrap_or((0, 0));
    Pos::all()
        .find(|pos| *pos >= Pos { section, level })
        .or_else(|| Pos::all().last())
        .unwrap_or(FIRST)
}

/// The directory a `cd` call names: its first non-option argument (`-` is a target, `--` ends the
/// options, a bare `cd` has the target `""`).
fn cd_target(args: &[String]) -> &str {
    let mut options = true;
    for arg in args {
        match arg.as_str() {
            "--" if options => options = false,
            option if options && option.starts_with('-') && option.len() > 1 => {}
            target => return target,
        }
    }
    ""
}

pub struct Session<'a> {
    env: &'a Env,
    term: Term,
    shell: Directives,
    /// Where the player's shell is.
    cwd: Option<PathBuf>,
    /// A level was prepared below the player, whose directory may have been recreated underneath
    /// the shell: the next teleport asks it to `cd` even to the same path.
    stale: bool,
    /// An older saved game that was set aside (its new name) and where to resume it: told at the end
    /// of the command, after any screen that clears the terminal.
    retired: Option<(PathBuf, Option<String>)>,
}

impl<'a> Session<'a> {
    pub fn new(env: &'a Env) -> Session<'a> {
        Session {
            env,
            term: Term::stdout(),
            shell: Directives::new(env.wrapped),
            cwd: env.cwd.clone(),
            stale: false,
            retired: None,
        }
    }

    /// The exit status of a finished command; a failure is shown as a red panel. A saved game that
    /// was set aside is told last, so no screen clears it away.
    pub fn finish(&self, result: Flow) -> i32 {
        let code = match result {
            Ok(()) => OK,
            Err(Halt::Exit(code)) => code,
            Err(Halt::Fail(message)) => {
                let lost = self.env.cwd.is_none().then(|| format!("\n\n{}", messages::CWD_MISSING));
                self.print(screens::error(&self.term, &(message + &lost.unwrap_or_default())));
                FAILURE
            }
        };
        if let Some((file, resume)) = &self.retired {
            self.print(screens::retired(&self.term, &file.to_string_lossy(), resume.as_deref()));
        }
        code
    }

    fn print(&self, lines: Vec<Line>) {
        self.term.print(&lines);
    }

    fn say(&self, style: Style, text: &str) {
        self.print(screens::say(&self.term, style, text));
    }

    /// Tells the player and stops the command with `code`.
    fn stop<T>(&self, style: Style, text: &str, code: i32) -> Flow<T> {
        self.say(style, text);
        Err(Halt::Exit(code))
    }

    fn save(&self, state: &State) -> Flow {
        Ok(state.save(&self.env.state_dir)?)
    }

    /// The saved game. An old file is set aside (the notice comes at the end of the command), which
    /// leaves no game.
    fn load(&mut self) -> Flow<Option<State>> {
        match state::load(&self.env.state_dir)? {
            Saved::Game(state) => Ok(Some(state)),
            Saved::None => Ok(None),
            Saved::Old(resume) => {
                self.retired = Some((state::retire(&self.env.state_dir)?, resume));
                Ok(None)
            }
        }
    }

    /// The saved game of a command that needs one.
    fn game(&mut self) -> Flow<State> {
        match self.load()? {
            Some(state) => Ok(state),
            None => self.stop(YELLOW, "Hra ještě nezačala. Spusťte: shellgame", FAILURE),
        }
    }

    /// The game of a command that needs one, at a level that exists: a finished game is inert (the
    /// completion screen), a stale position shows the level it was moved to instead of the command.
    fn active(&mut self) -> Flow<(State, Pos)> {
        let state = self.game()?;
        if state.finished {
            self.print(screens::complete(&self.term, &state));
            return Err(Halt::Exit(OK));
        }
        match self.restore(state)? {
            (state, pos, false) => Ok((state, pos)),
            (state, pos, true) => self.enter(state, pos, false).and(Err(Halt::Exit(OK))),
        }
    }

    /// Makes a loaded game playable: a position that no longer exists becomes the nearest following
    /// level (with a notice), a vanished workspace (a cleared `/tmp`) is rebuilt. True when the
    /// position changed.
    fn restore(&mut self, mut state: State) -> Flow<(State, Pos, bool)> {
        let missing = !state.workspace.is_dir();
        ensure_workspace(&state.workspace)?;
        let (pos, moved) =
            state.pos().map_or_else(|| (resolve(&state.level), true), |pos| (pos, false));
        if moved {
            self.print(screens::stale(&self.term, &state.level, pos));
            state.go(pos);
            self.save(&state)?;
        }
        if missing {
            self.say(YELLOW, "⚠ Pracovní prostor byl smazán (např. restart systému). Obnovuji...");
            self.say(GREEN, &format!("✓ Pracovní prostor obnoven: {}", state.workspace.display()));
        }
        if missing || moved {
            self.prepare_or_report(pos, &state.workspace);
        }
        Ok((state, pos, moved))
    }

    fn prepare(&mut self, pos: Pos, workspace: &Path) -> io::Result<()> {
        let root = canonical(&pos.section_dir(workspace));
        if self.cwd.as_ref().is_some_and(|cwd| cwd.starts_with(&root) && *cwd != root) {
            self.stale = true;
        }
        game::prepare(pos, workspace)
    }

    /// Prepares a level, telling the player when that fails; true if it worked.
    fn prepare_or_report(&mut self, pos: Pos, workspace: &Path) -> bool {
        let prepared = self.prepare(pos, workspace);
        if let Err(error) = &prepared {
            self.print(screens::setup_error(&self.term, pos, &error.to_string()));
        }
        prepared.is_ok()
    }

    /// Asks the shell to `cd` to the start of the level unless the player is there; returns the
    /// directory to announce when they were moved.
    fn teleport(&mut self, pos: Pos, workspace: &Path) -> Option<PathBuf> {
        let start = game::start_dir(pos, workspace);
        let target = canonical(&start);
        let there = self.cwd.as_deref() == Some(target.as_path());
        if there && !self.stale {
            return None;
        }
        self.shell.cd(&start);
        self.stale = false;
        if !start.is_dir() {
            return None;
        }
        self.cwd = Some(target);
        (!there).then_some(start)
    }

    /// The export of the workspace and the teleport to the start of the level.
    fn arrive(&mut self, state: &State, pos: Pos) -> Option<PathBuf> {
        self.shell.export(ENV_WORKSPACE, &state.workspace);
        self.teleport(pos, &state.workspace)
    }

    /// Pages of a section's intro, Enter between them. Ctrl-C stops paging.
    fn intro(&self, section: usize, then_level: bool) {
        let text = dedent(COURSE[section].intro);
        let pages = markdown::pages(&text);
        for (index, page) in pages.iter().enumerate() {
            self.term.clear();
            self.print(screens::intro_page(&self.term, section, page, index + 1, pages.len()));
            if (index + 1 < pages.len() || then_level) && !prompt::pause(&self.term) {
                break;
            }
        }
    }

    /// The assignment, and the line about the isolation of the game shell.
    fn show_level(&self, pos: Pos) {
        self.term.clear();
        self.print(screens::level(&self.term, pos));
        self.print(screens::banner(&self.term, self.env.sandbox.as_deref()));
    }

    /// The screens of a level the player has arrived at, the section's intro first when entering one.
    fn present(&self, pos: Pos, notice: Option<PathBuf>, intro: bool) {
        if intro {
            self.intro(pos.section, true);
        }
        self.show_level(pos);
        if let Some(dir) = notice {
            self.print(screens::teleport(&self.term, &dir.to_string_lossy()));
        }
    }

    /// Puts the player at a level: its clock starts, the shell is moved, the screens are shown.
    fn enter(&mut self, mut state: State, pos: Pos, intro: bool) -> Flow {
        if state.go(pos) {
            self.save(&state)?;
        }
        let notice = self.arrive(&state, pos);
        self.present(pos, notice, intro);
        Ok(())
    }

    /// `shellgame` inside the game shell: the current level, a new game on the first run.
    pub fn current(&mut self) -> Flow {
        let Some(state) = self.load()? else {
            return self.first_run();
        };
        if state.finished {
            self.print(screens::complete(&self.term, &state));
            return Ok(());
        }
        let (state, pos, _) = self.restore(state)?;
        self.enter(state, pos, false)
    }

    fn first_run(&mut self) -> Flow {
        let workspace = self.env.workspace.clone();
        ensure_workspace(&workspace)?;
        let state = State::new(workspace, FIRST);
        self.prepare_or_report(FIRST, &state.workspace);
        self.save(&state)?;
        self.print(screens::init(&self.term, &self.env.user, &state.workspace.to_string_lossy()));
        self.enter(state, FIRST, true)
    }

    /// The hints unlocked so far and the next one, if there is one left; with `repeat` only the
    /// unlocked ones.
    pub fn hint(&mut self, repeat: bool) -> Flow {
        let (mut state, pos) = self.active()?;
        let texts = pos.level().hints;
        let mut shown = (state.at(pos).hints as usize).min(texts.len());
        if repeat && shown == 0 {
            self.say(YELLOW, "Zatím jste v tomto levelu žádnou nápovědu neodemkli.");
            self.print(screens::cmd_note(
                &self.term,
                "První odemknete příkazem: ",
                "shellgame hint",
            ));
            self.print(vec![Line::new()]);
            return Ok(());
        }
        if !repeat && shown < texts.len() {
            shown += 1;
            state.at(pos).hints += 1;
            self.save(&state)?;
        }
        self.print(screens::hints(&self.term, texts, shown));
        Ok(())
    }

    /// A wrong answer is counted and explained; a right one is recorded and the game moves on.
    pub fn submit(&mut self, answer: Option<String>) -> Flow {
        let (mut state, pos) = self.active()?;
        let ctx = Ctx::new(&state.workspace, pos, self.cwd.clone(), &self.env.home);
        if let Err(message) = pos.level().judge(&ctx, answer.as_deref()) {
            state.at(pos).attempts += 1;
            self.save(&state)?;
            self.print(screens::failure(&self.term, &message));
            return Ok(());
        }
        state.at(pos).done = Some(state::now());
        let done = state.at(pos).clone();
        self.advance(state, pos, done)
    }

    fn show_success(&self, pos: Pos, done: &Progress) {
        self.print(screens::success(&self.term, &dedent(pos.level().success), done));
    }

    /// Leaves `pos` with ONE atomic save (the completion, the next position and its clock), so a
    /// next level that cannot be prepared never costs the player their progress.
    fn advance(&mut self, mut state: State, pos: Pos, done: Progress) -> Flow {
        let Some(next) = pos.next() else {
            state.finished = true;
            self.save(&state)?;
            self.show_success(pos, &done);
            self.print(screens::complete(&self.term, &state));
            return Ok(());
        };
        state.go(next);
        self.save(&state)?;
        let ready = self.prepare_or_report(next, &state.workspace);
        let notice = if ready { self.arrive(&state, next) } else { None };
        self.show_success(pos, &done);
        prompt::pause(&self.term);
        if ready {
            self.present(next, notice, next.section != pos.section);
        }
        Ok(())
    }

    /// Rebuilds the level and puts the player back at its start, wherever they wandered off to.
    pub fn reset(&mut self) -> Flow {
        let (state, pos) = self.active()?;
        if let Err(error) = self.prepare(pos, &state.workspace) {
            return self.stop(RED, &format!("Chyba při obnově levelu: {error}"), FAILURE);
        }
        let notice = self.arrive(&state, pos);
        self.show_level(pos);
        self.say(GREEN, &format!("✓ Level {pos} byl resetován."));
        if let Some(dir) = notice {
            self.print(screens::teleport(&self.term, &dir.to_string_lossy()));
        }
        Ok(())
    }

    /// The position a command line names: `S.L`, or `S` for the first level of a section.
    fn position(&self, target: &str) -> Flow<Pos> {
        target.parse().map_err(|error| {
            self.say(RED, &format!("Chyba: {error}"));
            self.print(screens::cmd_note(&self.term, "Seznam levelů: ", "shellgame levels"));
            Halt::Exit(FAILURE)
        })
    }

    /// Shows the assignment of a level (`S.L`) or the intro of a section (`S`) again, the current
    /// level's assignment without a target (the completion screen once the course is finished); the
    /// game is not touched.
    pub fn show(&mut self, target: Option<&str>) -> Flow {
        let Some(target) = target else {
            let (_, current) = self.active()?;
            self.show_level(current);
            return Ok(());
        };
        self.game()?;
        let pos = self.position(target)?;
        if target.contains('.') {
            self.show_level(pos);
        } else {
            self.intro(pos.section, false);
        }
        Ok(())
    }

    /// Continues at a level (`S.L`) or at the first level of a section after its intro (`S`),
    /// starting a game if there is none. Every record is kept (earlier levels are not marked
    /// completed), the level's clock starts if it has not, a finished game is open again.
    /// Without a valid target: the error and the levels to choose from.
    pub fn resume(&mut self, target: Option<&str>) -> Flow {
        let Some(target) = target else {
            self.say(RED, "Chyba: Chybí ID levelu.");
            self.print(screens::cmd_note(&self.term, "Použití: ", "shellgame resume <S.L | S>"));
            let state = self.load()?;
            self.print(screens::levels(&self.term, state.as_ref()));
            return Err(Halt::Exit(FAILURE));
        };
        let pos = self.position(target)?;
        let mut state = match self.load()? {
            Some(state) => state,
            None => State::new(self.env.workspace.clone(), pos),
        };
        ensure_workspace(&state.workspace)?;
        state.finished = false;
        state.go(pos);
        if !self.prepare_or_report(pos, &state.workspace) {
            return Err(Halt::Exit(FAILURE));
        }
        self.save(&state)?;
        let notice = self.arrive(&state, pos);
        if !target.contains('.') {
            self.intro(pos.section, true);
        }
        self.show_level(pos);
        self.say(GREEN, &format!("✓ Pokračujete od levelu {pos}."));
        if let Some(dir) = notice {
            self.print(screens::teleport(&self.term, &dir.to_string_lossy()));
        }
        Ok(())
    }

    /// Every level, with the saved position and the completed ones marked; works without a game.
    pub fn levels(&mut self) -> Flow {
        let state = self.load()?;
        self.print(screens::levels(&self.term, state.as_ref()));
        Ok(())
    }

    pub fn status(&mut self) -> Flow {
        let state = self.game()?;
        self.print(screens::status(&self.term, &state));
        Ok(())
    }

    /// Deletes the workspace (only inside `/tmp`, and only the player's own) and the saved game,
    /// whatever its state; closes the game shell. A declined confirmation is a failure.
    pub fn remove(&mut self, yes: bool) -> Flow {
        if !self.env.state_dir.join(state::FILE).exists() {
            return self.stop(YELLOW, "Není co odstranit: hra ještě nezačala.", FAILURE);
        }
        if !yes && !prompt::confirm("Opravdu chcete odstranit všechna data ShellGame?") {
            return self.stop(YELLOW, "Odstranění zrušeno. Žádná data nebyla smazána.", FAILURE);
        }
        let workspace =
            state::workspace_of(&self.env.state_dir).unwrap_or_else(|| self.env.workspace.clone());
        check_workspace(&workspace)
            .map_err(|why| Halt::Fail(format!("Odmítám odstranit pracovní prostor. {why}")))?;
        game::remove_tree(&workspace)?;
        state::remove(&self.env.state_dir)?;
        self.say(GREEN, "Stav ShellGame a pracovní prostor odstraněny.");
        self.shell.exit();
        Ok(())
    }

    pub fn exit(&mut self) -> Flow {
        if !self.env.wrapped {
            return self.stop(YELLOW, "Nejste v herním shellu, není z čeho odejít.", FAILURE);
        }
        self.say(YELLOW, "Ukončuji ShellGame...");
        if let Ok(Saved::Game(state)) = state::load(&self.env.state_dir)
            && !state.finished
        {
            self.print(screens::resume_hint(&self.term, &state.level));
        }
        self.shell.exit();
        Ok(())
    }

    pub fn update(&mut self) -> Flow {
        if self.env.sandbox.as_deref() == Some("landlock") {
            return self.stop(
                RED,
                "Aktualizaci spusťte mimo herní shell: shellgame exit, pak shellgame update",
                FAILURE,
            );
        }
        self.say(DIM, "Hledám nejnovější verzi ShellGame...");
        let old = version_line();
        match update::run(self.env, &old) {
            Ok(Some(new)) => {
                self.say(GREEN, &format!("✓ ShellGame byl aktualizován: {old} -> {new}"))
            }
            Ok(None) => self.say(GREEN, &format!("Již používáte nejnovější verzi: {old}")),
            Err(update::Failed { why, installer }) => {
                self.say(RED, &why);
                if installer {
                    let command =
                        [words(DIM, "Aktualizaci můžete provést znovu instalačním skriptem:")];
                    self.print(screens::note(&self.term, &command));
                    self.print(vec![vec![span(format!("   {}", update::INSTALLER), DIM_VIOLET)]]);
                }
                return Err(Halt::Exit(FAILURE));
            }
        }
        Ok(())
    }

    /// The game a hook looks at. Hooks fail open: no game, an unreadable one or an unknown level
    /// mean nothing to enforce, never an error in the player's `cd`.
    fn hook_game(&self) -> Option<(State, Pos)> {
        let Ok(Saved::Game(state)) = state::load(&self.env.state_dir) else {
            return None;
        };
        let pos = state.pos()?;
        Some((state, pos))
    }

    /// `cd-hook pre|post <before> <after-or-prev> -- <cd args...>`: `pre` may block the move
    /// ([`BLOCKED`], with the reason on stderr), `post` records the evidence of a move that happened.
    pub fn cd_hook(&self, post: bool, first: &Path, second: &Path, args: &[OsString]) -> i32 {
        let Some((state, pos)) = self.hook_game() else {
            return OK;
        };
        let args: Vec<String> = args.iter().map(|arg| arg.to_string_lossy().into_owned()).collect();
        let ctx = Ctx::new(&state.workspace, pos, self.cwd.clone(), &self.env.home);
        let target = cd_target(&args);
        if post {
            let dest = Some(second).filter(|dir| dir.is_absolute());
            cd::after(pos, &CdMove { target, pwd: first, prev: None, dest, ctx: &ctx });
            return OK;
        }
        let prev = Some(second).filter(|dir| dir.is_absolute());
        match cd::before(pos, &CdMove { target, pwd: first, prev, dest: None, ctx: &ctx }) {
            Ok(()) => OK,
            Err(message) => {
                let term = Term::stderr();
                term.print(&screens::cd_block(&term, pos, &message));
                BLOCKED
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cd_target_is_the_first_non_option_argument() {
        let target = |args: &[&str]| {
            cd_target(&args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>()).to_owned()
        };
        assert_eq!(
            [target(&[]), target(&["-P"]), target(&[""]), target(&["-"])],
            ["", "", "", "-"]
        );
        assert_eq!(
            [target(&["-L", "-P", "dir", "x"]), target(&["--", "-P"]), target(&["-P", "--", "--"])],
            ["dir", "-P", "--"]
        );
    }
}
