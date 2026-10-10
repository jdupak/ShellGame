//! End-to-end tests of the command line: the real binary, run the way the game shell runs it.
//!
//! Unless a test is about running outside the game shell, every command runs with
//! `SHELLGAME_WRAPPER=1`, so the protocol directives (`cd`, `export`, `exit`) appear on stderr and are
//! decoded here. Each test gets its own sandbox (`HOME`, `XDG_CONFIG_HOME`, `TMPDIR`, state directory
//! and workspace) and never touches the environment or working directory of the test process; see
//! `cli_support/`.
//!
//! The tests rely only on level ids, answers and start directories that the real course has (0.0, 0.1,
//! 1.0 to 1.4, 5.0 to 5.2, 10.4 for the fd evidence). Everything else about a level (its title, hints,
//! success text, where it starts, what goes next) is read from the library's catalog, which is the
//! course the binary under test runs, so reworded or added levels do not break them.

mod cli_support;

use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use chrono::NaiveDateTime;
use cli_support::{
    BIN, Game, Run, permissions_are_ignored, run_in_pty, skip, std_output_retrying, tool_works,
};
use shellgame::engine::prelude::{Catalog, Level, Section};
use shellgame::messages;
use shellgame::state::{LevelCompletion, State};

fn now() -> NaiveDateTime {
    shellgame::state::now()
}

fn path_text(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

/// The course as the binary under test has it.
fn course() -> &'static Catalog {
    static COURSE: OnceLock<Catalog> = OnceLock::new();
    COURSE.get_or_init(shellgame::levels::catalog)
}

/// A level of the course and its section.
fn entry(id: &str) -> (&'static Section, &'static Level) {
    course()
        .get(id.parse().expect("a level id"))
        .unwrap_or_else(|| panic!("the course has no level {id}"))
}

/// The title of a level's panel: `1.4: <title>` for a regular level, the bare title of an intro.
fn heading(id: &str) -> String {
    entry(id).1.display_title()
}

/// How many hints a level has.
fn hint_count(id: &str) -> usize {
    entry(id).1.hints.len()
}

/// What a level says when it is completed.
fn success_text(id: &str) -> &'static str {
    &entry(id).1.success
}

/// The directory a level's paths are relative to, in this game's workspace (`level-1` for 1.x).
fn section_dir(game: &Game, id: &str) -> PathBuf {
    entry(id)
        .0
        .section_root(&game.workspace)
        .expect("a section root")
}

/// The directory a level starts in, in this game's workspace.
fn start_dir(game: &Game, id: &str) -> PathBuf {
    entry(id)
        .1
        .start_dir(&game.workspace, &section_dir(game, id))
        .unwrap_or_else(|| panic!("level {id} has no start directory"))
}

/// A game that has advanced to level 1.4, where the player stands in its start directory.
fn game_at_1_4() -> Game {
    let game = Game::new();
    game.start();
    game.jump("1.4");
    game
}

/// The start directory of 1.4 as the shell hooks see it.
fn alpha(game: &Game) -> String {
    path_text(&start_dir(game, "1.4"))
}

/// The section directory of 1.4, one step up from [`alpha`].
fn level_1(game: &Game) -> String {
    path_text(&section_dir(game, "1.4"))
}

// ---------------------------------------------------------------------------------------------
// Starting and advancing
// ---------------------------------------------------------------------------------------------

#[test]
fn the_first_run_creates_the_game_and_moves_the_player_into_it() {
    let game = Game::new();
    let run = game.run(&[]);
    run.assert_code(0);

    // state.json: schema 2.1, written the way pydantic writes it.
    let raw = String::from_utf8(game.state_bytes()).unwrap();
    assert!(raw.contains("\"version\": \"2.1\""), "{raw}");
    assert!(!raw.ends_with('\n'), "pydantic writes no trailing newline");
    let state = game.state();
    assert_eq!(state.version, "2.1");
    assert_eq!(state.username, "ada");
    assert_eq!(state.workspace, game.workspace);
    assert!(game.workspace.is_dir(), "the workspace is created");

    // The protocol: the workspace and the level are exported, and the shell is moved into the
    // workspace (the player ran the game somewhere else).
    assert_eq!(
        run.exported("SHELLGAME_WORKSPACE"),
        Some(path_text(&game.workspace))
    );
    assert!(run.exported("SHELLGAME_LEVEL").is_some());
    assert_eq!(run.cds().first(), Some(&path_text(&game.workspace)));

    // What the player sees.
    run.assert_says("ShellGame inicializována pro ada");
    run.assert_says("Byli jste teleportováni do adresáře");
    run.assert_says(&heading("0.0"));
    assert!(run.plain_stderr().is_empty(), "{}", run.stderr);
}

#[test]
fn the_intro_advances_by_itself_when_nothing_can_press_enter() {
    let game = Game::new();
    let run = game.start();

    // Without a terminal no prompt is shown, so the intro is submitted right away.
    let state = game.state();
    assert!(state.levels_complete.contains_key("0.0"), "{state:#?}");
    assert_eq!(state.current_level, "0.1");
    assert!(!run.says("Stiskněte Enter"));
    run.assert_says(&heading("0.1"));
    assert_eq!(run.exported("SHELLGAME_LEVEL").as_deref(), Some("0.1"));
    assert!(state.level_started_at.contains_key("0.1"));
    assert!(state.completed_at.is_none());
}

#[test]
fn directives_come_before_the_first_screen_of_the_level() {
    // They are emitted before any prompt, so a Ctrl-C at a prompt cannot lose them. Without a
    // terminal the order of stderr against stdout cannot be observed, but stderr itself is ordered:
    // the exports of a level precede its teleport.
    let game = Game::new();
    let run = game.start();
    let verbs: Vec<String> = run.directives().into_iter().map(|d| d.verb).collect();
    let first_cd = verbs.iter().position(|v| v == "cd").unwrap();
    let last_export = verbs.iter().rposition(|v| v == "export").unwrap();
    assert!(first_cd < last_export, "{verbs:?}");
}

#[test]
fn a_wrong_answer_is_counted_and_explained() {
    let game = Game::new();
    game.start();

    let run = game.run(&["submit", "wrong"]);
    run.assert_code(0);
    run.assert_says("Zkuste to znovu nebo použijte: shellgame hint");
    assert!(!run.says("Čas:"), "no success panel for a wrong answer");

    let state = game.state();
    assert_eq!(state.level_attempts.get("0.1"), Some(&1));
    assert_eq!(state.current_level, "0.1");
    assert!(!state.levels_complete.contains_key("0.1"));

    game.run(&["submit", "again"]).assert_code(0);
    assert_eq!(game.state().level_attempts.get("0.1"), Some(&2));
}

#[test]
fn a_right_answer_records_the_completion_and_advances() {
    let game = Game::new();
    game.start();
    game.run(&["submit", "wrong"]);

    // Case does not matter for this answer.
    let run = game.run(&["submit", "START"]);
    run.assert_code(0);
    run.assert_says(success_text("0.1"));
    run.assert_says("Nápovědy: 0");
    run.assert_says("Chybné pokusy: 1");

    let state = game.state();
    let done = &state.levels_complete["0.1"];
    assert_eq!(done.attempts, 1);
    assert_eq!(done.hints, 0);
    // Section 1 starts with an intro, which the game passes by itself.
    assert!(state.levels_complete.contains_key("1.0"));
    assert_eq!(state.current_level, "1.1");
    run.assert_says(&heading("1.1"));
    assert_eq!(run.exported("SHELLGAME_LEVEL").as_deref(), Some("1.1"));
    assert_eq!(
        run.cds().last(),
        Some(&path_text(&start_dir(&game, "1.1"))),
        "the shell is sent to where 1.1 starts"
    );
    assert!(
        start_dir(&game, "1.4").is_dir(),
        "the fixture of section 1 is built when the section is entered"
    );
}

#[test]
fn several_words_are_one_answer_and_a_dash_is_not_an_option() {
    let game = Game::new();
    game.start();

    // "start now" is not "start".
    game.run(&["submit", "start", "now"])
        .assert_says("Zkuste to znovu nebo použijte: shellgame hint");
    assert_eq!(game.state().level_attempts.get("0.1"), Some(&1));

    // A negative number is an answer (not an unknown option) and gets a wrong-answer panel.
    game.jump("5.1");
    let run = game.run(&["submit", "-5"]);
    run.assert_code(0);
    run.assert_says("Zkuste to znovu nebo použijte: shellgame hint");
    assert_eq!(game.state().level_attempts.get("5.1"), Some(&1));
}

#[test]
fn an_empty_submission_gets_the_required_message_of_the_answer() {
    let game = Game::new();
    game.start();
    let run = game.run(&["submit"]);
    run.assert_code(0);
    run.assert_says(
        entry("0.1")
            .1
            .answer
            .as_ref()
            .expect("0.1 takes an answer")
            .required_message(),
    );
    assert_eq!(game.state().level_attempts.get("0.1"), Some(&1));
}

#[test]
fn requirements_are_unconditional() {
    let game = Game::new();
    game.start();
    game.jump("1.1");

    // The right answer without the `pwd` evidence fails...
    let run = game.run(&["submit", "level-1"]);
    run.assert_code(0);
    run.assert_says("Zkuste to znovu nebo použijte: shellgame hint");
    let state = game.state();
    assert_eq!(state.current_level, "1.1");
    assert_eq!(state.level_attempts.get("1.1"), Some(&1));
    assert!(!state.levels_complete.contains_key("1.1"));

    // ...and passes once the shell's `pwd` wrapper has left its marker.
    fs::write(game.marker(".pwd_used"), "").unwrap();
    let run = game.run(&["submit", "level-1"]);
    run.assert_says(success_text("1.1"));
    assert!(game.state().levels_complete.contains_key("1.1"));
}

#[test]
fn the_player_must_stand_where_the_level_asks() {
    let game = game_at_1_4();
    fs::write(game.marker(".cd-1.4"), "").unwrap();

    // In the start directory the level is not done yet: the player must be one step up.
    let run = game.run_in(&start_dir(&game, "1.4"), &["submit", "level-1"]);
    run.assert_says("Zkuste to znovu nebo použijte: shellgame hint");
    assert!(!game.state().levels_complete.contains_key("1.4"));

    let run = game.run_in(&section_dir(&game, "1.4"), &["submit", "level-1"]);
    run.assert_says(success_text("1.4"));
    assert!(game.state().levels_complete.contains_key("1.4"));
}

#[test]
fn a_deleted_working_directory_is_reported_not_crashed_on() {
    let game = game_at_1_4();
    fs::write(game.marker(".cd-1.4"), "").unwrap();
    let doomed = game.base.join("doomed");
    fs::create_dir(&doomed).unwrap();

    // The wrapper function moves the shell before it runs the binary; here the binary is started
    // in a directory that is gone, as a player in a deleted directory would.
    let output = game
        .sh(&doomed, "rmdir \"$PWD\"; exec \"$SG\" submit level-1")
        .output()
        .unwrap();
    let run = Run::of(&output);
    run.assert_code(0);
    run.assert_says(messages::CWD_MISSING);
    assert_eq!(
        game.state().current_level,
        "1.4",
        "the level is not completed from nowhere"
    );
}

// ---------------------------------------------------------------------------------------------
// Where things live
// ---------------------------------------------------------------------------------------------

#[test]
fn the_state_lives_where_the_lookup_says() {
    let mut game = Game::new();
    game.unset_env("SHELLGAME_STATE_DIR");
    game.start();
    // `$XDG_CONFIG_HOME/shellgame`, not the real home of the developer.
    assert!(game.xdg.join("shellgame/state.json").is_file());
    assert!(!game.home.join(".config").exists());

    // Without XDG it is `~/.config/shellgame`, also on macOS.
    let mut game = Game::new();
    game.unset_env("SHELLGAME_STATE_DIR")
        .unset_env("XDG_CONFIG_HOME");
    game.start();
    assert!(game.home.join(".config/shellgame/state.json").is_file());

    // The explicit directory wins over both.
    let mut game = Game::new();
    game.set_env("SHELLGAME_STATE_DIR", game.base.join("explicit"));
    game.start();
    assert!(game.base.join("explicit/state.json").is_file());
    assert!(!game.xdg.join("shellgame").exists());
}

#[test]
fn a_relative_workspace_is_made_absolute_when_the_game_is_created() {
    let mut game = Game::new();
    game.set_env("SHELLGAME_WORKSPACE", "relative-workspace");
    game.start();
    let expected = game.cwd.join("relative-workspace");
    assert_eq!(game.state().workspace, expected);
    assert!(expected.is_dir());
    // It still names the same directory from anywhere else.
    game.run_in(&expected, &["status"]).assert_code(0);
    assert!(!expected.join("relative-workspace").exists());
}

#[test]
fn a_workspace_that_cannot_be_created_is_reported_and_no_game_is_saved() {
    let game = Game::new();
    if permissions_are_ignored(&game.base) {
        skip("running as root: directory permissions are not enforced");
        return;
    }
    let locked = game.base.join("locked");
    fs::create_dir(&locked).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).unwrap();
    let mut game = game;
    game.set_env("SHELLGAME_WORKSPACE", locked.join("workspace"));

    let run = game.run(&[]);
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    run.assert_code(1);
    run.assert_says("Chyba při práci se soubory:");
    assert!(!game.state_dir.join("state.json").exists());
    assert!(run.directives().is_empty());
}

#[test]
fn a_workspace_path_with_spaces_works_everywhere() {
    let mut game = Game::new();
    game.workspace = game.tmp.join("my workspace");
    game.set_env("SHELLGAME_WORKSPACE", game.workspace.clone());
    let run = game.start();
    assert_eq!(run.cds().first(), Some(&path_text(&game.workspace)));
    game.jump("1.4");

    let alpha_dir = alpha(&game);
    assert!(alpha_dir.contains("my workspace"));
    game.run(&["cd-hook", "pre", &alpha_dir, "", "--", ".."])
        .assert_code(0);
    game.run(&["cd-hook", "pre", &alpha_dir, "", "--", "../.."])
        .assert_code(3);
    game.run(&["cd-hook", "post", &alpha_dir, &level_1(&game), "--", ".."])
        .assert_code(0);
    assert!(game.marker(".cd-1.4").is_file());
    game.run_in(&section_dir(&game, "1.4"), &["submit", "level-1"])
        .assert_says(success_text("1.4"));
    game.run(&["remove", "--yes"]).assert_code(0);
    assert!(!game.workspace.exists());
}

#[test]
fn a_workspace_below_a_symlink_is_the_same_place_as_its_real_path() {
    // On macOS `/tmp` is a symlink to `/private/tmp`, and the shell reports the logical path.
    let mut game = Game::new();
    let link = game.base.join("link");
    std::os::unix::fs::symlink(&game.tmp, &link).unwrap();
    game.workspace = link.join("workspace");
    game.set_env("SHELLGAME_WORKSPACE", game.workspace.clone());
    game.set_env("TMPDIR", link.clone());
    game.start();
    game.jump("1.4");
    // The start directory of 1.4 by its real path (the workspace is `link/workspace`).
    let relative_start = start_dir(&game, "1.4")
        .strip_prefix(&game.workspace)
        .unwrap()
        .to_owned();
    let real_alpha = game.tmp.join("workspace").join(&relative_start);

    // A player standing in the start directory (by its real path) is not teleported.
    let run = game.run_in(&real_alpha, &[]);
    assert!(run.cds().is_empty(), "{:?}", run.cds());
    assert!(!run.says("teleportováni"));

    // The hook knows the logical start directory is the real one: from one step up, going back to
    // it is the recovery move that is always allowed.
    let from = path_text(&section_dir(&game, "1.4"));
    let back = start_dir(&game, "1.4")
        .strip_prefix(section_dir(&game, "1.4"))
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let run = game.run(&["cd-hook", "pre", &from, "", "--", &back]);
    run.assert_code(0);

    // And removal is allowed through the link.
    game.run(&["remove", "--yes"]).assert_code(0);
    assert!(!game.tmp.join("workspace").exists());
}

// ---------------------------------------------------------------------------------------------
// Prompts (on a pseudo-terminal)
// ---------------------------------------------------------------------------------------------

/// Position of `needle` in `haystack`; the test fails with the whole output when it is missing.
#[track_caller]
fn position(haystack: &str, needle: &str) -> usize {
    haystack
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not found in\n{haystack}"))
}

const PROMPT: &str = "Stiskněte Enter pro pokračování...";
const ENTER: &str = "\r";
const CTRL_C: &str = "\u{3}";

#[test]
fn enter_at_the_last_page_of_an_intro_continues_to_the_first_level() {
    let game = Game::new();
    let Some(run) = run_in_pty(&game, &[], &[ENTER]) else {
        skip("python3 is not installed");
        return;
    };
    assert_eq!(run.code, Some(0), "{}", run.output);
    assert_eq!(run.output.matches(PROMPT).count(), 1, "{}", run.output);
    assert_eq!(game.state().current_level, "0.1");
    assert!(run.output.contains(&heading("0.1")));
}

#[test]
fn ctrl_c_at_a_prompt_ends_the_command_normally_with_the_directives_already_emitted() {
    let game = Game::new();
    let Some(run) = run_in_pty(&game, &[], &[CTRL_C]) else {
        skip("python3 is not installed");
        return;
    };
    // Ctrl-C is a key, not a signal: the process exits normally...
    assert_eq!(run.code, Some(0), "{}", run.output);
    // ...without submitting the intro...
    let state = game.state();
    assert_eq!(state.current_level, "0.0");
    assert!(!state.levels_complete.contains_key("0.0"));
    // ...and the shell was already told where to go and which level it is in, before the prompt.
    let prompt = position(&run.output, PROMPT);
    for directive in ["__SHELLGAME_EXEC__v1 cd ", "__SHELLGAME_EXEC__v1 export "] {
        assert!(
            position(&run.output, directive) < prompt,
            "{directive} comes after the prompt"
        );
    }
}

#[test]
fn ctrl_c_after_a_success_still_shows_the_next_level_and_has_moved_the_shell() {
    let game = Game::new();
    game.start();
    // The first prompt is the one after the success panel; the next level is the intro of
    // section 1, whose own prompt gets the second key (and ends the paging).
    let Some(run) = run_in_pty(&game, &["submit", "start"], &[CTRL_C, CTRL_C]) else {
        skip("python3 is not installed");
        return;
    };
    assert_eq!(run.code, Some(0), "{}", run.output);

    let state = game.state();
    assert!(
        state.levels_complete.contains_key("0.1"),
        "the answer was recorded"
    );
    assert_eq!(
        state.current_level, "1.0",
        "stopping at the intro does not submit it"
    );
    assert!(!state.levels_complete.contains_key("1.0"));
    assert!(
        run.output.contains(&heading("1.0")),
        "the next level was shown after the interrupt"
    );

    // The directives for the next level precede the first prompt, so Ctrl-C there cannot lose them.
    let first_prompt = position(&run.output, PROMPT);
    let level_export = position(
        &run.output,
        "__SHELLGAME_EXEC__v1 export U0hFTExHQU1FX0xFVkVM MS4w",
    );
    let teleport = position(&run.output, "__SHELLGAME_EXEC__v1 cd ");
    assert!(
        level_export < first_prompt && teleport < first_prompt,
        "{}",
        run.output
    );
    assert!(
        position(&run.output, "Nápovědy:") < first_prompt,
        "the success panel is on screen before the prompt"
    );
}

#[test]
fn a_ctrl_c_between_the_pages_of_an_intro_stops_paging_without_submitting() {
    let game = Game::new();
    game.start();
    let Some(run) = run_in_pty(&game, &["dev", "jump", "1.0"], &[CTRL_C]) else {
        skip("python3 is not installed");
        return;
    };
    assert_eq!(run.code, Some(0), "{}", run.output);
    assert_eq!(game.state().current_level, "1.0");
    assert!(!game.state().levels_complete.contains_key("1.0"));
    assert!(
        !run.output.contains("strana 2/2"),
        "the second page was not shown"
    );
    assert_eq!(run.output.matches(PROMPT).count(), 1);
}

// ---------------------------------------------------------------------------------------------
// Hints
// ---------------------------------------------------------------------------------------------

#[test]
fn hints_are_revealed_one_at_a_time_and_counted_once() {
    let game = Game::new();
    game.start();
    let total = hint_count("0.1");
    assert!(total >= 1, "0.1 has hints");

    for shown in 1..=total {
        let run = game.run(&["hint"]);
        run.assert_code(0);
        run.assert_says(&format!("💡 Nápověda {shown} z {total}"));
        run.assert_says("shellgame hint --repeat");
        if shown > 1 {
            assert!(
                !run.says(&format!("Nápověda {} z {total}", shown - 1)),
                "only the new hint is shown"
            );
        }
        assert_eq!(
            run.says("Potřebujete další pomoc? Napište: shellgame hint"),
            shown < total,
            "the last hint points nowhere, the others to the next one"
        );
        let used = u32::try_from(shown).unwrap();
        assert_eq!(game.state().level_hints_used.get("0.1"), Some(&used));
    }

    let none_left = game.run(&["hint"]);
    none_left.assert_code(0);
    none_left.assert_says("Pro tento level již nejsou k dispozici žádné další nápovědy.");
    assert_eq!(
        game.state().level_hints_used.get("0.1"),
        Some(&u32::try_from(total).unwrap()),
        "asking for more than exists counts nothing"
    );
}

#[test]
fn repeating_hints_never_consumes_one() {
    let game = Game::new();
    game.start();
    let total = hint_count("0.1");

    // Nothing revealed yet.
    let run = game.run(&["hint", "--repeat"]);
    run.assert_code(0);
    run.assert_says("Zatím jste si nezobrazili žádnou nápovědu.");
    run.assert_says("První nápovědu zobrazíte příkazem: shellgame hint");
    assert_eq!(game.state().level_hints_used.get("0.1"), None);

    let revealed = total.min(2);
    for _ in 0..revealed {
        game.run(&["hint"]);
    }
    let before = game.state_bytes();

    let run = game.run(&["hint", "-r"]);
    for shown in 1..=revealed {
        run.assert_says(&format!("Nápověda {shown} z {total}"));
    }
    assert!(!run.says(&format!("Nápověda {} z {total}", revealed + 1)));
    assert_eq!(
        run.says("Potřebujete další pomoc? Napište: shellgame hint"),
        revealed < total
    );
    assert_eq!(
        game.state_bytes(),
        before,
        "--repeat does not touch the state"
    );
}

#[test]
fn hints_are_printed_verbatim() {
    // Any hint of the course that contains characters a markup renderer would eat must come out
    // exactly as written. (The course may or may not have such a hint; the check is then skipped.)
    let catalog = shellgame::levels::catalog();
    let tricky = catalog.iter().find_map(|(_, level)| {
        level
            .hints
            .iter()
            .position(|hint| hint.contains('[') || hint.contains('*'))
            .map(|index| (level.id, index, level.hints.clone()))
    });
    let Some((id, index, hints)) = tricky else {
        skip("no hint of the course contains brackets or asterisks");
        return;
    };
    let game = Game::new();
    game.start();
    game.jump(&id.to_string());
    let mut shown = Run {
        code: 0,
        stdout: String::new(),
        stderr: String::new(),
    };
    for _ in 0..=index {
        shown = game.run(&["hint"]);
    }
    let expected = &hints[index];
    assert!(
        shown.says_compactly(expected),
        "hint {index} of {id} must be verbatim: {expected:?}\n{}",
        shown.stdout
    );
}

// ---------------------------------------------------------------------------------------------
// Skipping, resetting, jumping
// ---------------------------------------------------------------------------------------------

#[test]
fn a_required_level_cannot_be_skipped() {
    let game = Game::new();
    game.start();
    let before = game.state_bytes();

    let run = game.run(&["skip"]);
    run.assert_code(0);
    run.assert_says("Level 0.1 je povinný a nelze ho přeskočit.");
    run.assert_says("Zkuste nápovědu: shellgame hint");
    assert_eq!(game.state_bytes(), before);
}

#[test]
fn a_bonus_level_can_be_skipped_without_a_completion() {
    let game = Game::new();
    game.start();
    game.jump("5.2");

    let run = game.run(&["skip"]);
    run.assert_code(0);
    run.assert_says("Level 5.2 přeskočen (byl nepovinný).");
    assert!(!run.says("Čas:"), "no success panel for a skipped level");

    let state = game.state();
    assert!(
        !state.levels_complete.contains_key("5.2"),
        "a skip is not a completion"
    );
    assert_ne!(state.current_level, "5.2", "the game moved on");
}

#[test]
fn a_bonus_level_announces_that_it_can_be_skipped() {
    let game = Game::new();
    game.start();
    let run = game.jump("5.2");
    run.assert_says("Tento level je nepovinný. Můžete ho přeskočit příkazem: shellgame skip");
    // Its start directory is created and the shell is sent there.
    assert_eq!(run.cds().last(), Some(&path_text(&start_dir(&game, "5.2"))));
    let (file, _) = entry("5.2")
        .1
        .fixture
        .file_entries()
        .next()
        .expect("5.2 comes with files");
    assert!(section_dir(&game, "5.2").join(file).is_file());
}

#[test]
fn reset_rebuilds_the_level_and_moves_the_player_back() {
    let game = game_at_1_4();
    // A file of the section's fixture, deleted by the player.
    let (file, content) = entry("1.4")
        .0
        .fixture
        .file_entries()
        .next()
        .expect("section 1 comes with files");
    let path = section_dir(&game, "1.4").join(file);
    fs::remove_file(&path).unwrap();

    let run = game.run(&["reset"]);
    run.assert_code(0);
    assert_eq!(
        fs::read(&path).unwrap(),
        content.as_bytes(),
        "the fixture is rebuilt"
    );
    run.assert_says("✓ Level 1.4 byl resetován.");
    run.assert_says(&heading("1.4"));
    assert_eq!(run.exported("SHELLGAME_LEVEL").as_deref(), Some("1.4"));
    assert_eq!(run.cds(), [alpha(&game)]);
    run.assert_says("Byli jste teleportováni do adresáře");
}

#[test]
fn reset_clears_the_evidence_of_the_level() {
    let game = game_at_1_4();
    fs::write(game.marker(".cd-1.4"), "").unwrap();
    game.run(&["reset"]).assert_code(0);
    assert!(
        !game.marker(".cd-1.4").exists(),
        "stale evidence must not complete the level"
    );
}

#[test]
fn reset_always_rescues_a_player_who_stands_below_the_section() {
    // The directory the player stands in may be deleted and recreated by the rebuild, which leaves
    // their shell in a dead directory at an unchanged path. The binary cannot tell, so it sends the
    // shell to the start directory again (without announcing a teleport).
    let game = game_at_1_4();
    let run = game.run_in(&start_dir(&game, "1.4"), &["reset"]);
    run.assert_code(0);
    assert_eq!(run.cds(), [alpha(&game)]);
    assert!(
        !run.says("teleportováni"),
        "the player did not change place: {}",
        run.stdout
    );
}

#[test]
fn a_player_already_at_the_start_is_not_teleported_by_a_plain_command() {
    let game = game_at_1_4();
    let run = game.run_in(&start_dir(&game, "1.4"), &[]);
    run.assert_code(0);
    assert!(run.cds().is_empty(), "{:?}", run.cds());
    assert!(!run.says("teleportováni"));
    run.assert_says(&heading("1.4"));
}

#[test]
fn a_player_elsewhere_is_teleported_once_with_a_notice() {
    let game = game_at_1_4();
    let run = game.run(&[]);
    assert_eq!(run.cds(), [alpha(&game)]);
    assert_eq!(run.stdout.matches("Byli jste teleportováni").count(), 1);
}

#[test]
fn dev_jump_replays_a_level_from_scratch() {
    let game = Game::new();
    game.start();
    game.jump("1.1");
    fs::write(game.marker(".pwd_used"), "").unwrap();
    game.run(&["hint"]);
    game.run(&["submit", "level-1"]);
    let state = game.state();
    assert!(state.levels_complete.contains_key("1.1"));
    assert_eq!(state.level_hints_used.get("1.1"), Some(&1));

    game.jump("1.1");
    let state = game.state();
    assert_eq!(state.current_level, "1.1");
    assert!(
        !state.levels_complete.contains_key("1.1"),
        "its completion is forgotten"
    );
    assert_eq!(state.level_hints_used.get("1.1"), None);
    assert_eq!(state.level_attempts.get("1.1"), None);
    assert!(state.completed_at.is_none());
    assert!(
        !game.marker(".pwd_used").exists(),
        "its evidence is cleared too"
    );
}

#[test]
fn dev_jump_rejects_an_unknown_level_and_a_missing_game() {
    let game = Game::new();
    let run = game.run(&["dev", "jump", "1.4"]);
    run.assert_code(1);
    run.assert_says("Neinicializováno. Spusťte: shellgame init");

    game.start();
    let run = game.run(&["dev", "jump", "99.9"]);
    run.assert_code(1);
    run.assert_says("Neznámý level: 99.9");
    assert_eq!(game.state().current_level, "0.1", "nothing moved");
}

#[test]
fn dev_next_prev_start_and_reload() {
    let game = Game::new();
    game.start();
    let catalog = shellgame::levels::catalog();

    game.jump("1.4");
    game.run(&["dev", "prev"]).assert_code(0);
    let previous = catalog.prev("1.4".parse().unwrap()).unwrap().unwrap();
    assert_eq!(game.state().current_level, previous.to_string());

    game.jump("1.4");
    game.run(&["dev", "next"]).assert_code(0);
    let next = catalog.next("1.4".parse().unwrap()).unwrap().unwrap();
    let state = game.state();
    assert!(
        state.current_level == next.to_string() || next.is_intro(),
        "{} after {next}",
        state.current_level
    );

    let run = game.run(&["dev", "reload"]);
    run.assert_code(0);
    run.assert_says("✓ Level");
    run.assert_says("znovu načten.");

    game.run(&["dev", "start"]).assert_code(0);
    let state = game.state();
    assert!(state.levels_complete.contains_key("0.0"));
    assert_eq!(
        state.current_level, "0.1",
        "the first level is an intro, which passes by itself"
    );
}

#[test]
fn dev_next_on_the_last_level_and_dev_prev_on_the_first_say_so() {
    let game = Game::new();
    game.start();
    let catalog = shellgame::levels::catalog();

    game.jump(&catalog.last().unwrap().to_string());
    let run = game.run(&["dev", "next"]);
    run.assert_code(0);
    run.assert_says("Žádný další level.");

    let mut state = game.state();
    state.current_level = catalog.first().unwrap().to_string();
    game.save(&state);
    let run = game.run(&["dev", "prev"]);
    run.assert_code(0);
    run.assert_says("Žádný předchozí level.");
}

// ---------------------------------------------------------------------------------------------
// Repeat, show, status
// ---------------------------------------------------------------------------------------------

#[test]
fn repeat_shows_an_assignment_again_without_changing_anything() {
    let game = Game::new();
    game.start();
    game.jump("1.4");
    let before = game.state_bytes();

    let current = game.run(&["repeat"]);
    current.assert_code(0);
    current.assert_says(&heading("1.4"));

    let other = game.run(&["repeat", "--level", "1.1"]);
    other.assert_says(&heading("1.1"));
    assert!(!other.says(&heading("1.4")));

    let section = game.run(&["repeat", "--section", "1"]);
    section.assert_says(&heading("1.0"));

    // An explicit level wins over a section.
    let both = game.run(&["repeat", "--section", "5", "--level", "1.1"]);
    both.assert_says(&heading("1.1"));

    assert_eq!(game.state_bytes(), before);
    assert!(
        both.directives().is_empty() && section.directives().is_empty(),
        "repeat never moves the player"
    );
}

#[test]
fn repeat_reports_an_unknown_level_and_a_missing_game() {
    let game = Game::new();
    game.run(&["repeat"]).assert_code(1);

    game.start();
    // 1.11 was retired after ids were saved, so it is guaranteed to stay unused.
    let run = game.run(&["repeat", "--level", "1.11"]);
    run.assert_code(1);
    run.assert_says("Chyba: Level 1.11 nenalezen");
    let run = game.run(&["repeat", "--level", "garbage"]);
    run.assert_code(1);
    run.assert_says("Chyba: Level garbage nenalezen");
    let run = game.run(&["repeat", "--section", "98"]);
    run.assert_code(1);
    run.assert_says("Chyba: Level 98.0 nenalezen");
    // `--section` takes a number.
    game.run(&["repeat", "--section", "abc"]).assert_code(2);
}

#[test]
fn show_wants_exactly_one_of_its_flags() {
    let game = Game::new();
    game.start();
    game.jump("1.1");
    let usage = "Použití: shellgame show --level  nebo  shellgame show --section";

    for args in [&["show"][..], &["show", "--section", "--level"]] {
        let run = game.run(args);
        run.assert_code(2);
        run.assert_says(usage);
        assert!(!run.says(&heading("1.1")));
    }

    let level = game.run(&["show", "--level"]);
    level.assert_code(0);
    level.assert_says(&heading("1.1"));

    let section = game.run(&["show", "--section"]);
    section.assert_code(0);
    section.assert_says(&heading("1.0"));
    assert!(!section.says(&heading("1.1")));
}

#[test]
fn show_refuses_a_malformed_saved_level_and_a_missing_game() {
    let game = Game::new();
    game.run(&["show", "--level"]).assert_code(1);

    game.start();
    // A saved id that is not `S.N` is resynced by the next command; `show --section` reads the id
    // as it is saved, and a state that cannot even be loaded is reported as such.
    let mut state = game.state();
    state.current_level = "oops".to_owned();
    game.save(&state);
    let run = game.run(&["show", "--section"]);
    run.assert_code(1);
    run.assert_says("Chyba: Neplatný formát aktuálního levelu: oops");
}

#[test]
fn status_lists_the_progress_and_the_statistics() {
    let game = Game::new();
    let run = game.run(&["status"]);
    run.assert_code(1);
    run.assert_says("Neinicializováno. Spusťte: shellgame init");

    game.start();
    game.run(&["submit", "wrong"]);
    game.run(&["submit", "start"]);
    let run = game.run(&["status"]);
    run.assert_code(0);
    run.assert_says("═══ ShellGame Postup ═══");
    run.assert_says("Aktuální Level: 1.1");
    run.assert_says("Zahájeno:");
    run.assert_says("Dokončené Levely");
    run.assert_says("Level Čas Nápovědy Chybné pokusy");
    run.assert_says("Statistiky:");
    run.assert_says("Dokončené levely: 3");
    run.assert_says("Celkem chybných pokusů: 1");
    assert!(
        run.directives().is_empty(),
        "status says nothing to the shell"
    );
}

#[test]
fn status_sorts_levels_numerically() {
    let game = Game::new();
    game.start();
    let mut state = game.state();
    state.levels_complete.clear();
    for id in ["1.12", "1.2", "1.10", "0.1"] {
        state.levels_complete.insert(
            id.to_owned(),
            LevelCompletion {
                time_sec: 65,
                hints: 1,
                attempts: 0,
                completed_at: now(),
            },
        );
    }
    game.save(&state);

    let run = game.run(&["status"]);
    let order: Vec<&str> = run
        .stdout
        .lines()
        .filter(|line| line.starts_with('│'))
        .map(|line| {
            line.trim_start_matches('│')
                .split_whitespace()
                .next()
                .unwrap()
        })
        .collect();
    assert_eq!(order, ["0.1", "1.2", "1.10", "1.12"], "{}", run.stdout);
    run.assert_says("1m 5s");
}

// ---------------------------------------------------------------------------------------------
// Remove, exit, init
// ---------------------------------------------------------------------------------------------

#[test]
fn remove_deletes_the_workspace_and_the_state_and_closes_the_shell() {
    let game = Game::new();
    game.start();
    assert!(game.workspace.is_dir());

    let run = game.run(&["remove", "--yes"]);
    run.assert_code(0);
    run.assert_says("Stav ShellGame a pracovní prostor odstraněny.");
    assert!(run.asked_to_exit(), "{:?}", run.directives());
    assert!(!game.workspace.exists());
    assert!(!game.state_dir.join("state.json").exists());
}

#[test]
fn remove_repairs_locked_directories_before_deleting() {
    let game = Game::new();
    game.start();
    let locked = game.ws("locked");
    fs::create_dir_all(locked.join("inner")).unwrap();
    fs::write(locked.join("inner/file"), "x").unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

    game.run(&["remove", "--yes"]).assert_code(0);
    assert!(
        !game.workspace.exists(),
        "chmod 000 must not stop the removal"
    );
}

#[test]
fn remove_asks_first_and_declining_deletes_nothing() {
    let game = Game::new();
    game.start();
    let before = game.state_bytes();

    for answer in ["n\n", "\n", "no\n", ""] {
        let run = game.run_with_stdin(&["remove"], answer);
        run.assert_code(1);
        run.assert_says("Opravdu chcete odstranit všechna data ShellGame?");
        run.assert_says("Odstranění zrušeno.");
        assert!(
            !run.asked_to_exit(),
            "a declined remove must not close the shell"
        );
        assert!(game.workspace.is_dir());
        assert_eq!(game.state_bytes(), before);
    }

    let run = game.run_with_stdin(&["remove"], "y\n");
    run.assert_code(0);
    assert!(!game.workspace.exists());
    assert!(run.asked_to_exit());
}

#[test]
fn remove_works_on_a_corrupt_state() {
    let game = Game::new();
    game.start();
    fs::write(game.state_dir.join("state.json"), "{ not json").unwrap();

    // Everything else refuses to touch a state it cannot read...
    let run = game.run(&["status"]);
    run.assert_code(1);
    run.assert_says("Stav ShellGame nelze načíst z");
    assert_eq!(
        fs::read_to_string(game.state_dir.join("state.json")).unwrap(),
        "{ not json"
    );

    // ...but remove falls back to the default workspace, deletes it and the broken file.
    let run = game.run(&["remove", "--yes"]);
    run.assert_code(0);
    assert!(!game.workspace.exists());
    assert!(!game.state_dir.join("state.json").exists());
    assert!(run.asked_to_exit());
}

#[test]
fn remove_without_a_game_says_so() {
    let game = Game::new();
    let run = game.run(&["remove", "--yes"]);
    run.assert_code(1);
    run.assert_says("Neinicializováno. Spusťte: shellgame init");
    assert!(!run.asked_to_exit());
}

#[test]
fn remove_refuses_a_workspace_outside_the_temporary_directories() {
    // A canary directory outside /tmp and outside $TMPDIR. `/var/tmp` is the usual candidate.
    let Ok(canary_root) = tempfile::Builder::new()
        .prefix("sg-canary-")
        .tempdir_in("/var/tmp")
    else {
        skip("cannot create a directory in /var/tmp");
        return;
    };
    let canary = canary_root.path().canonicalize().unwrap();
    let tmp_root = Path::new("/tmp").canonicalize().unwrap();
    if canary.starts_with(&tmp_root) {
        skip("/var/tmp is inside /tmp here");
        return;
    }

    let game = Game::new();
    game.start();
    fs::write(canary.join("precious.txt"), "keep me").unwrap();
    let mut state = game.state();
    state.workspace = canary.clone();
    game.save(&state);
    let before = game.state_bytes();

    let run = game.run(&["remove", "--yes"]);
    run.assert_code(1);
    run.assert_says("Odmítám odstranit pracovní prostor mimo dočasný adresář");
    assert_eq!(
        fs::read_to_string(canary.join("precious.txt")).unwrap(),
        "keep me"
    );
    assert_eq!(game.state_bytes(), before, "the state stays, too");
    assert!(!run.asked_to_exit());
}

#[test]
fn exit_closes_the_shell() {
    let game = Game::new();
    let run = game.run(&["exit"]);
    run.assert_code(0);
    run.assert_says("Ukončuji ShellGame...");
    assert!(run.asked_to_exit());

    // Outside the game shell there is nobody to close.
    let mut outside = Game::new();
    outside.wrapped = false;
    let run = outside.run(&["exit"]);
    run.assert_code(0);
    assert!(run.directives().is_empty());
}

#[test]
fn the_legacy_init_creates_the_game_once() {
    let game = Game::new();
    let run = game.run(&["init"]);
    run.assert_code(0);
    run.assert_says("ShellGame inicializována pro ada");
    assert_eq!(
        game.state().current_level,
        "0.0",
        "init does not show or pass the intro"
    );

    let run = game.run(&["init"]);
    run.assert_code(0);
    run.assert_says("Již inicializováno! Spusťte 'shellgame' pro pokračování.");
}

#[test]
fn commands_without_a_game_say_so() {
    let game = Game::new();
    for args in [
        &["hint"][..],
        &["submit", "x"],
        &["skip"],
        &["reset"],
        &["status"],
    ] {
        let run = game.run(args);
        run.assert_code(1);
        run.assert_says("Neinicializováno. Spusťte: shellgame init");
        assert!(run.directives().is_empty(), "{args:?}");
    }
    assert!(
        !game.state_dir.join("state.json").exists(),
        "none of them creates a game"
    );
}

// ---------------------------------------------------------------------------------------------
// Robustness of saved games
// ---------------------------------------------------------------------------------------------

fn finished_state(game: &Game) -> State {
    let mut state = State::new("ada", &game.workspace, now());
    state.current_level = "1.1".to_owned();
    state.levels_complete.insert(
        "0.1".to_owned(),
        LevelCompletion {
            time_sec: 3700,
            hints: 1,
            attempts: 2,
            completed_at: now(),
        },
    );
    state.completed_at = Some(now());
    state
}

#[test]
fn a_finished_game_is_inert() {
    let game = Game::new();
    fs::create_dir_all(&game.workspace).unwrap();
    game.save(&finished_state(&game));
    let before = game.state_bytes();

    for args in [
        &[][..],
        &["submit", "level-1"],
        &["skip"],
        &["hint"],
        &["hint", "--repeat"],
        &["reset"],
        &["show", "--level"],
    ] {
        let run = game.run(args);
        run.assert_code(0);
        run.assert_says("Dokončili jste ShellGame!");
        run.assert_says("Dokončené levely: 1  |  Celkový čas: 1h 1m 40s");
        assert!(
            run.directives().is_empty(),
            "{args:?} must not move the shell"
        );
        assert_eq!(
            game.state_bytes(),
            before,
            "{args:?} must not change the state"
        );
    }
    // `status` still works, and `remove` still removes.
    game.run(&["status"]).assert_says("Dokončené levely: 1");
}

#[test]
fn a_stale_current_level_is_resynced_to_the_nearest_later_level() {
    let game = Game::new();
    game.start();
    let catalog = shellgame::levels::catalog();
    // 1.11 is a gap in the numbering of section 1.
    let resolved = catalog.resolve_str("1.11").unwrap().to_string();
    let mut state = game.state();
    state.current_level = "1.11".to_owned();
    game.save(&state);

    // A command aimed at a level that no longer exists is not applied to its replacement.
    let run = game.run(&["hint"]);
    run.assert_code(0);
    run.assert_says("Level 1.11 už v této verzi hry neexistuje.");
    run.assert_says(&format!(
        "Pokračujeme nejbližším dostupným levelem: {resolved}"
    ));
    run.assert_says("Váš dosavadní postup zůstává zachován: shellgame status");
    assert!(
        !run.says("💡 Nápověda"),
        "the hint was not applied to the replacement"
    );

    let state = game.state();
    assert_ne!(state.current_level, "1.11");
    assert!(state.level_hints_used.is_empty());
    assert!(
        state.levels_complete.contains_key("0.0"),
        "progress is kept"
    );
    assert!(
        state.completed_at.is_none(),
        "an unknown id never means the game is finished"
    );
}

#[test]
fn a_stale_level_is_resynced_by_every_gameplay_command() {
    for args in [
        &["submit", "anything"][..],
        &["skip"],
        &["reset"],
        &["hint"],
        &[],
    ] {
        let game = Game::new();
        game.start();
        let mut state = game.state();
        state.current_level = "77.7".to_owned();
        game.save(&state);

        let run = game.run(args);
        run.assert_code(0);
        run.assert_says("Level 77.7 už v této verzi hry neexistuje.");
        let state = game.state();
        assert_ne!(state.current_level, "77.7", "{args:?}");
        assert!(!state.levels_complete.contains_key("77.7"));
    }
}

#[test]
fn a_missing_workspace_is_rebuilt() {
    let game = game_at_1_4();
    fs::remove_dir_all(&game.workspace).unwrap();

    let run = game.run(&["hint"]);
    run.assert_code(0);
    run.assert_says("Pracovní prostor byl smazán (např. restart systému). Obnovuji...");
    run.assert_says(&format!(
        "✓ Pracovní prostor obnoven: {}",
        game.workspace.display()
    ));
    assert!(
        start_dir(&game, "1.4").is_dir(),
        "the current level is prepared again"
    );
    run.assert_says(&format!("💡 Nápověda 1 z {}", hint_count("1.4")));
    assert_eq!(game.state().level_hints_used.get("1.4"), Some(&1));
}

#[test]
fn a_failing_next_level_preparation_keeps_the_progress() {
    let game = Game::new();
    game.start();
    if permissions_are_ignored(&game.base) {
        skip("running as root: directory permissions are not enforced");
        return;
    }
    // Section 1 does not exist yet; with a read-only workspace its directory cannot be created.
    fs::set_permissions(&game.workspace, fs::Permissions::from_mode(0o555)).unwrap();

    let run = game.run(&["submit", "start"]);
    fs::set_permissions(&game.workspace, fs::Permissions::from_mode(0o755)).unwrap();
    run.assert_code(0);
    run.assert_says(success_text("0.1"));
    run.assert_says("Level 1.0 se nepodařilo připravit.");
    run.assert_says("Zkuste: shellgame reset");

    // The completion and the move to the next level were saved before the failure.
    let state = game.state();
    assert!(state.levels_complete.contains_key("0.1"));
    assert_eq!(state.current_level, "1.0");
    assert!(state.level_started_at.contains_key("1.0"));
    assert!(!game.ws("level-1").exists());
    // The shell was not sent into a level that does not exist.
    assert!(run.cds().is_empty(), "{:?}", run.cds());

    // `reset` repairs it.
    let run = game.run(&["reset"]);
    run.assert_code(0);
    assert!(game.ws("level-1").is_dir());
    run.assert_says("✓ Level 1.0 byl resetován.");
    assert_eq!(run.cds(), [path_text(&game.workspace)]);
}

#[test]
fn an_unreadable_state_stops_every_command_and_is_left_alone() {
    let game = Game::new();
    game.start();
    for bad in ["", "{ not json", "[]", "{\"version\": \"99\"}"] {
        fs::write(game.state_dir.join("state.json"), bad).unwrap();
        for args in [
            &[][..],
            &["hint"],
            &["submit", "x"],
            &["status"],
            &["reset"],
        ] {
            let run = game.run(args);
            run.assert_code(1);
            run.assert_says("Stav ShellGame nelze načíst z");
            assert!(run.directives().is_empty());
        }
        assert_eq!(
            fs::read_to_string(game.state_dir.join("state.json")).unwrap(),
            bad,
            "the file stays untouched"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The cd hook
// ---------------------------------------------------------------------------------------------

#[test]
fn a_blocked_cd_exits_3_with_the_panel_and_the_recovery_tip_on_stderr() {
    let game = game_at_1_4();
    let run = game.run(&["cd-hook", "pre", &alpha(&game), "", "--", "../.."]);
    run.assert_code(3);
    assert!(run.says_on_stderr("ShellGame (1.4)"), "{}", run.stderr);
    assert!(run.says_on_stderr(messages::CD_RECOVERY_TIP));
    assert!(
        run.stdout.is_empty(),
        "the explanation goes to stderr: {}",
        run.stdout
    );
    assert!(
        run.stderr.contains("\u{1b}["),
        "stderr is replayed later, so colour is forced"
    );
    assert!(
        !game.marker(".cd-1.4").exists(),
        "a blocked move earns nothing"
    );
}

#[test]
fn the_move_that_the_level_teaches_is_allowed_and_earns_the_evidence_after_it_happened() {
    let game = game_at_1_4();

    let pre = game.run(&["cd-hook", "pre", &alpha(&game), "", "--", ".."]);
    pre.assert_code(0);
    assert!(pre.stderr.is_empty(), "{}", pre.stderr);
    assert!(
        !game.marker(".cd-1.4").exists(),
        "the new hook records evidence only after the move succeeded"
    );

    let post = game.run(&[
        "cd-hook",
        "post",
        &alpha(&game),
        &level_1(&game),
        "--",
        "..",
    ]);
    post.assert_code(0);
    assert!(post.stdout.is_empty() && post.stderr.is_empty());
    assert!(game.marker(".cd-1.4").is_file());

    // With the evidence in place the level no longer blocks anything.
    let later = game.run(&["cd-hook", "pre", &level_1(&game), "", "--", "../.."]);
    later.assert_code(0);
}

#[test]
fn a_move_that_was_not_the_drill_earns_no_evidence() {
    let game = game_at_1_4();
    let post = game.run(&[
        "cd-hook",
        "post",
        &alpha(&game),
        &path_text(&game.workspace),
        "--",
        "../..",
    ]);
    post.assert_code(0);
    assert!(!game.marker(".cd-1.4").exists());
}

#[test]
fn options_and_the_double_dash_are_skipped_when_finding_the_cd_target() {
    let game = game_at_1_4();
    let dir = alpha(&game);
    for allowed in [
        &["-P", ".."][..],
        &["-L", "-P", ".."],
        &["--", ".."],
        &["-e", "--", ".."],
    ] {
        let mut args = vec!["cd-hook", "pre", dir.as_str(), "", "--"];
        args.extend_from_slice(allowed);
        game.run(&args).assert_code(0);
    }
    for blocked in [&["-P", "../.."][..], &["--", "-P"], &["-"], &[]] {
        let mut args = vec!["cd-hook", "pre", dir.as_str(), "", "--"];
        args.extend_from_slice(blocked);
        game.run(&args).assert_code(3);
    }
}

#[test]
fn returning_to_the_start_directory_is_always_allowed() {
    let game = game_at_1_4();
    // From the section directory, `cd <start>` goes back to the start of the level: the recovery
    // move.
    let start_name = start_dir(&game, "1.4")
        .strip_prefix(section_dir(&game, "1.4"))
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let run = game.run(&["cd-hook", "pre", &level_1(&game), "", "--", &start_name]);
    run.assert_code(0);
    // `cd -` with a known previous directory that is the start.
    let run = game.run(&["cd-hook", "pre", &level_1(&game), &alpha(&game), "--", "-"]);
    run.assert_code(0);
    // But `cd -` towards somewhere else is blocked.
    let run = game.run(&["cd-hook", "pre", &alpha(&game), &level_1(&game), "--", "-"]);
    run.assert_code(3);
}

#[test]
fn relative_or_empty_directories_are_treated_as_unknown() {
    let game = game_at_1_4();
    // The hook cannot tell where the player is, so it cannot call the move the drill: blocked
    // (the level has a rule) rather than a crash, and nothing is recorded.
    game.run(&["cd-hook", "pre", "", "", "--", ".."])
        .assert_code(0);
    game.run(&["cd-hook", "pre", "relative", "relative", "--", "../.."])
        .assert_code(3);
    game.run(&["cd-hook", "post", "", "", "--", ".."])
        .assert_code(0);
    assert!(!game.marker(".cd-1.4").exists());
}

#[test]
fn the_legacy_hook_of_a_python_shell_blocks_with_exit_1_and_records_evidence_before_the_move() {
    let game = game_at_1_4();

    let blocked = game.run(&["cd-hook", "../..", &alpha(&game)]);
    blocked.assert_code(1);
    assert!(blocked.says_on_stderr("ShellGame (1.4)"));
    assert!(blocked.says_on_stderr(messages::CD_RECOVERY_TIP));
    assert!(!game.marker(".cd-1.4").exists());

    // The old wrapper treats any non-zero status as "blocked", so an allowed move exits 0, and the
    // evidence is recorded right away (the Python hook had no after-move report for this).
    let allowed = game.run(&["cd-hook", "..", &alpha(&game)]);
    allowed.assert_code(0);
    assert!(allowed.stderr.is_empty());
    assert!(game.marker(".cd-1.4").is_file());
}

#[test]
fn the_legacy_post_move_report_is_accepted() {
    let game = game_at_1_4();
    let run = game.run(&["cd-hook", "--post-move", &level_1(&game)]);
    run.assert_code(0);
    assert!(run.stdout.is_empty() && run.stderr.is_empty());
    // A leading --devmode, as the old wrapper wrote it, changes nothing.
    game.run(&["--devmode", "cd-hook", "--post-move", &level_1(&game)])
        .assert_code(0);
    let blocked = game.run(&["--devmode", "cd-hook", "../..", &alpha(&game)]);
    blocked.assert_code(1);
}

#[test]
fn a_directory_named_pre_is_not_mistaken_for_the_new_hook_form() {
    let game = game_at_1_4();
    // `cd-hook pre <pwd>` is the legacy hook for `cd pre`: the target is "pre", which the level
    // blocks, and the legacy form blocks with exit status 1, not 3.
    let run = game.run(&["cd-hook", "pre", &alpha(&game)]);
    run.assert_code(1);
    assert!(run.says_on_stderr("ShellGame (1.4)"));
}

#[test]
fn the_hook_fails_open_without_a_usable_game() {
    let game = Game::new();
    // No state at all.
    for args in [
        &["cd-hook", "pre", "/", "", "--", ".."][..],
        &["cd-hook", "post", "/", "/tmp", "--", ".."],
        &["cd-hook", "..", "/"],
        &["cd-hook", "--post-move", "/"],
        &["cd-hook"],
        &["fd-hook"],
    ] {
        let run = game.run(args);
        run.assert_code(0);
        assert!(
            run.stdout.is_empty() && run.stderr.is_empty(),
            "{args:?}: {}",
            run.stderr
        );
    }
    assert!(
        !game.state_dir.join("state.json").exists(),
        "a hook never creates a game"
    );

    // A corrupt state.
    fs::create_dir_all(&game.state_dir).unwrap();
    fs::write(game.state_dir.join("state.json"), "{ nope").unwrap();
    let run = game.run(&["cd-hook", "pre", "/", "", "--", ".."]);
    run.assert_code(0);
    assert!(run.stdout.is_empty() && run.stderr.is_empty());

    // An unknown current level.
    let game = Game::new();
    game.start();
    let mut state = game.state();
    state.current_level = "77.7".to_owned();
    game.save(&state);
    game.run(&["cd-hook", "pre", "/", "", "--", ".."])
        .assert_code(0);
    game.run(&["fd-hook"]).assert_code(0);
}

#[test]
fn levels_without_a_cd_policy_allow_every_move_silently() {
    let game = Game::new();
    game.start();
    game.jump("1.1");
    let run = game.run(&["cd-hook", "pre", &level_1(&game), "", "--", "/etc"]);
    run.assert_code(0);
    assert!(run.stdout.is_empty() && run.stderr.is_empty());
    game.run(&["cd-hook", "/etc", &level_1(&game)])
        .assert_code(0);
}

// ---------------------------------------------------------------------------------------------
// The fd hook
// ---------------------------------------------------------------------------------------------

fn run_fd_hook(game: &Game, stdout: Stdio, stderr: Stdio) {
    let status = game
        .std_command()
        .arg("fd-hook")
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .status()
        .expect("run fd-hook");
    assert!(status.success(), "fd-hook is silent and always succeeds");
}

fn dev_null() -> Stdio {
    Stdio::from(File::options().write(true).open("/dev/null").unwrap())
}

#[test]
fn the_fd_hook_records_evidence_only_when_both_streams_go_to_dev_null() {
    let game = Game::new();
    game.start();
    game.jump("10.4");
    let marker = game.marker(".fd-10.4");
    let scratch = game.base.join("scratch.log");
    let file = || Stdio::from(File::create(&scratch).unwrap());

    run_fd_hook(&game, dev_null(), file());
    assert!(!marker.exists(), "stderr alone is not enough");
    run_fd_hook(&game, file(), dev_null());
    assert!(!marker.exists(), "stdout alone is not enough");
    run_fd_hook(&game, Stdio::piped(), Stdio::piped());
    assert!(!marker.exists(), "pipes are not /dev/null");

    run_fd_hook(&game, dev_null(), dev_null());
    assert!(marker.is_file(), "`&> /dev/null` leaves the evidence");

    // With the evidence in place the level can be completed by naming the special file.
    let run = game.run(&["submit", "/dev/null"]);
    run.assert_says(success_text("10.4"));
    assert!(game.state().levels_complete.contains_key("10.4"));
}

#[test]
fn the_fd_hook_is_silent_on_a_level_that_does_not_watch_descriptors() {
    let game = Game::new();
    game.start();
    game.jump("1.1");
    run_fd_hook(&game, dev_null(), dev_null());
    let hidden: Vec<String> = fs::read_dir(&game.workspace)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(".fd-"))
        .collect();
    assert!(hidden.is_empty(), "{hidden:?}");
}

// ---------------------------------------------------------------------------------------------
// Version, help, usage, outside the game shell
// ---------------------------------------------------------------------------------------------

#[test]
fn the_version_works_with_no_state_no_terminal_and_no_wrapper() {
    let mut game = Game::new();
    game.wrapped = false;
    let run = game.run(&["--version"]);
    run.assert_code(0);
    // `SHELLGAME_BUILD` is embedded at compile time; without it the build is called "dev".
    let build = option_env!("SHELLGAME_BUILD")
        .filter(|build| !build.is_empty())
        .unwrap_or("dev");
    assert_eq!(
        run.stdout.trim_end(),
        format!("shellgame {} ({build})", env!("CARGO_PKG_VERSION"))
    );
    assert!(run.stderr.is_empty());
    assert!(
        !game.state_dir.exists(),
        "the version never creates anything"
    );
    assert!(!game.workspace.exists());
    game.run(&["-V"]).assert_code(0);
}

#[test]
fn help_is_czech() {
    let game = Game::new();
    let run = game.run(&["--help"]);
    run.assert_code(0);
    for text in [
        "Použití:",
        "Možnosti:",
        "Příkazy:",
        "Odeslat odpověď pro aktuální level.",
    ] {
        run.assert_says(text);
    }
    assert!(
        !run.stdout.contains("cd-hook"),
        "the hooks are not for players"
    );
    let run = game.run(&["hint", "--help"]);
    run.assert_code(0);
    run.assert_says("Zobrazit znovu všechny již zobrazené nápovědy pro aktuální level.");
}

#[test]
fn usage_errors_exit_2() {
    let game = Game::new();
    game.run(&["bogus"]).assert_code(2);
    game.run(&["hint", "--nope"]).assert_code(2);
    game.run(&["--shell", "zsh"]).assert_code(2);
    game.run(&["dev"]).assert_code(2);
    assert!(!game.state_dir.exists(), "a usage error creates nothing");
}

#[test]
fn outside_the_game_shell_a_command_runs_directly_without_protocol_lines() {
    let mut game = Game::new();
    game.start();

    game.wrapped = false;
    let run = game.run(&["status"]);
    run.assert_code(0);
    run.assert_says("Aktuální Level: 0.1");
    assert!(run.directives().is_empty());
    assert!(!run.stderr.contains("__SHELLGAME_EXEC__"));

    // Even a command that would normally move the player, export variables or close the shell.
    let run = game.run(&["submit", "start"]);
    run.assert_code(0);
    run.assert_says(success_text("0.1"));
    assert!(!run.stderr.contains("__SHELLGAME_EXEC__"), "{}", run.stderr);
    assert_eq!(game.state().current_level, "1.1");

    let run = game.run(&["reset"]);
    run.assert_code(0);
    assert!(run.directives().is_empty());
    let run = game.run(&["exit"]);
    assert!(run.directives().is_empty());
    let run = game.run(&["remove", "--yes"]);
    run.assert_code(0);
    assert!(run.directives().is_empty());
    assert!(!game.workspace.exists());
}

// ---------------------------------------------------------------------------------------------
// Booting the game shell
// ---------------------------------------------------------------------------------------------

/// A PATH directory holding fake shells that record how they were started and exit with status 3.
fn fake_shells(game: &Game) -> std::path::PathBuf {
    let dir = game.base.join("fake-bin");
    fs::create_dir_all(&dir).unwrap();
    let script = "#!/bin/sh\n\
        {\n\
          printf 'argv:'; for a in \"$@\"; do printf ' [%s]' \"$a\"; done; printf '\\n'\n\
          printf 'wrapper:%s\\n' \"$SHELLGAME_WRAPPER\"\n\
          printf 'bin:%s\\n' \"$SHELLGAME_BIN\"\n\
          printf 'init:%s\\n' \"$SHELLGAME_INIT\"\n\
          printf 'fdhook:%s\\n' \"$SHELLGAME_FD_HOOK\"\n\
          printf 'autostarted:%s\\n' \"${SHELLGAME_AUTO_STARTED-unset}\"\n\
          printf 'cwd:%s\\n' \"$PWD\"\n\
        } > \"$RECORD\"\n\
        if [ -n \"$SHELLGAME_INIT\" ]; then\n\
          while IFS= read -r line || [ -n \"$line\" ]; do printf '%s\\n' \"$line\"; done < \"$SHELLGAME_INIT\" > \"$RECORD.init\"\n\
        fi\n\
        exit 3\n";
    for name in ["bash", "fish"] {
        let path = dir.join(name);
        fs::write(&path, script).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    dir
}

fn booting_game() -> (Game, std::path::PathBuf) {
    let mut game = Game::new();
    game.wrapped = false;
    let fake = fake_shells(&game);
    let record = game.base.join("record.txt");
    game.set_env("PATH", &fake)
        .set_env("RECORD", &record)
        // A game shell inside a game shell must still autostart.
        .set_env("SHELLGAME_AUTO_STARTED", "1");
    (game, record)
}

#[test]
fn outside_the_game_shell_the_bare_command_starts_it() {
    let (game, record) = booting_game();
    let run = game.run_booting(&["--shell", "bash"]);
    // The shell's own exit status (3 here) is not a failure of the game.
    run.assert_code(0);
    assert!(run.stderr.is_empty(), "{}", run.stderr);

    let recorded = fs::read_to_string(&record).unwrap();
    let line = |prefix: &str| {
        recorded
            .lines()
            .find_map(|l| l.strip_prefix(prefix))
            .unwrap_or_else(|| panic!("no {prefix:?} in\n{recorded}"))
            .to_owned()
    };
    let argv = line("argv: ");
    assert!(
        argv.starts_with("[--rcfile] [") && argv.ends_with("] [-i]"),
        "{argv}"
    );
    assert_eq!(line("wrapper:"), "1");
    assert_eq!(
        Path::new(&line("bin:")).canonicalize().unwrap(),
        Path::new(BIN).canonicalize().unwrap(),
        "the shell gets the real path of this binary"
    );
    assert_eq!(
        line("autostarted:"),
        "unset",
        "the inherited marker is removed"
    );
    let init = line("init:");
    assert!(
        !Path::new(&init).exists(),
        "the temporary script is removed afterwards"
    );
    let script = fs::read_to_string(format!("{}.init", record.display())).unwrap();
    assert!(
        script.contains("shellgame()"),
        "the rendered integration script was sourced"
    );
    assert!(
        !script.contains("alias r="),
        "no developer shortcuts without --devmode"
    );

    // The fd-hook launcher lives in the state directory.
    let launcher = game.state_dir.join("fd-hook.sh");
    assert_eq!(line("fdhook:"), path_text(&launcher));
    assert!(launcher.is_file());
    // Starting the shell does not create a game; the shell's autostart does.
    assert!(!game.state_dir.join("state.json").exists());
}

#[test]
fn the_shell_is_forced_by_the_flag_or_the_environment_and_devmode_adds_shortcuts() {
    let (mut game, record) = booting_game();

    let run = game.run_booting(&["--shell", "fish", "--devmode"]);
    run.assert_code(0);
    let recorded = fs::read_to_string(&record).unwrap();
    let argv = recorded.lines().next().unwrap();
    assert!(
        argv.starts_with(
            "argv: [--init-command] [function fish_greeting; end; source $SHELLGAME_INIT]"
        ),
        "{argv}"
    );
    let script = fs::read_to_string(format!("{}.init", record.display())).unwrap();
    assert!(
        script.contains("abbr -a r \"shellgame dev reload\""),
        "developer shortcuts for fish"
    );

    // The environment variable forces the shell, too, and the flag wins over it.
    game.set_env("SHELLGAME_FORCE_SHELL", "fish");
    game.run_booting(&[]).assert_code(0);
    assert!(
        fs::read_to_string(&record)
            .unwrap()
            .starts_with("argv: [--init-command]")
    );
    game.run_booting(&["--shell", "bash"]).assert_code(0);
    assert!(
        fs::read_to_string(&record)
            .unwrap()
            .starts_with("argv: [--rcfile]")
    );

    // The hidden flag alone is enough to be accepted before nothing at all.
    game.unset_env("SHELLGAME_FORCE_SHELL");
    game.run_booting(&["--shell", "bash", "--devmode"])
        .assert_code(0);
    let script = fs::read_to_string(format!("{}.init", record.display())).unwrap();
    assert!(
        script.contains("alias r=\"shellgame dev reload\""),
        "developer shortcuts for bash"
    );
}

#[test]
fn a_shell_that_cannot_be_started_is_reported() {
    let mut game = Game::new();
    game.wrapped = false;
    let empty = game.base.join("empty-bin");
    fs::create_dir_all(&empty).unwrap();
    game.set_env("PATH", &empty);

    let run = game.run_booting(&["--shell", "bash"]);
    run.assert_code(1);
    run.assert_says("CHYBA: Nepodařilo se spustit herní shell (");
}

#[test]
fn a_subcommand_never_starts_a_shell_outside_the_game_shell() {
    let (game, record) = booting_game();
    // `status` runs directly (and finds no game); the fake shell was never started.
    let run = game.run_booting(&["status"]);
    run.assert_code(1);
    run.assert_says("Neinicializováno");
    assert!(!record.exists());
}

/// Plays in a real game shell, started by the binary the way a player starts it
/// (`shellgame --shell <shell>`), with the commands on stdin instead of a keyboard.
fn play_in_a_real_shell(shell: &str) {
    let mut game = Game::new();
    game.wrapped = false;
    let script = [
        // The game autostarts: the first run, the intro and level 0.1.
        "shellgame submit start",   // 0.1, the intro of section 1, then 1.1
        "pwd",                      // the wrapped `pwd` leaves the evidence that 1.1 asks for
        "shellgame submit level-1", // 1.1 is done
        "shellgame dev jump 1.4",   // the shell is moved to where 1.4 starts
        "echo \"at:$PWD\"",
        "cd ../..", // blocked by the level's hook
        "echo \"at:$PWD\"",
        "cd ..", // the move the level teaches
        "echo \"at:$PWD\"",
        "shellgame submit level-1", // 1.4 is done
        "exit",
    ]
    .join("\n")
        + "\n";
    let run = game.run_with_stdin(&["--shell", shell], &script);

    // The game shell's own exit status is not the game's business.
    run.assert_code(0);
    let state = game.state();
    for done in ["0.1", "1.0", "1.1", "1.4"] {
        assert!(
            state.levels_complete.contains_key(done),
            "{done} is not complete in {shell}: {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            state.levels_complete.keys().collect::<Vec<_>>(),
            run.stdout,
            run.stderr
        );
    }
    assert_ne!(state.current_level, "1.4");

    // Where the shell was, step by step: moved by the game, held by the hook, moved by `cd`.
    let places: Vec<&str> = run
        .stdout
        .lines()
        .filter_map(|line| line.strip_prefix("at:"))
        .collect();
    let expected = [
        alpha(&game),
        alpha(&game), // `cd ../..` was refused
        level_1(&game),
    ];
    assert_eq!(places, expected, "{shell}\n{}", run.stdout);

    // The refusal was explained to the player, and no protocol line leaked into the output.
    assert!(run.says_on_stderr("ShellGame (1.4)"), "{}", run.stderr);
    assert!(run.says_on_stderr(messages::CD_RECOVERY_TIP));
    assert!(!run.stdout.contains("__SHELLGAME_EXEC__"));
    assert!(!run.stderr.contains("__SHELLGAME_EXEC__"));
}

#[test]
fn the_game_runs_inside_a_real_bash() {
    if !tool_works("bash") {
        skip("bash is not installed");
        return;
    }
    play_in_a_real_shell("bash");
}

#[test]
fn the_game_runs_inside_a_real_fish() {
    if !tool_works("fish") {
        skip("fish is not installed");
        return;
    }
    play_in_a_real_shell("fish");
}

// ---------------------------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------------------------

/// A sandbox for `shellgame update`: a COPY of the binary in its own directory (never the binary
/// under test, which `update` would replace) and a release archive to download.
struct UpdateBench {
    game: Game,
    /// The copy of the binary that the test runs.
    binary: std::path::PathBuf,
    archive: std::path::PathBuf,
}

impl UpdateBench {
    /// `contents` is what the archive's `shellgame` file holds.
    fn new(contents: &str) -> Self {
        let game = Game::new();
        let bin_dir = game.base.join("install");
        fs::create_dir_all(&bin_dir).unwrap();
        // A hard link is a private name for the same file: `update` replaces the name, never the
        // file, and nothing writes to it. Across file systems a copy has to do.
        let binary = bin_dir.join("shellgame");
        if fs::hard_link(BIN, &binary).is_err() {
            fs::copy(BIN, &binary).unwrap();
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        }

        let release = game.base.join("release");
        fs::create_dir_all(&release).unwrap();
        let fake = release.join("shellgame");
        fs::write(&fake, contents).unwrap();
        fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
        let archive = game.base.join("shellgame-test.tar.gz");
        let status = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(&release)
            .arg("shellgame")
            .status()
            .expect("run tar");
        assert!(status.success());
        Self {
            game,
            binary,
            archive,
        }
    }

    fn url(&self) -> String {
        format!("file://{}", self.archive.display())
    }

    /// Runs `shellgame update` with the copy of the binary.
    fn update(&mut self) -> Run {
        let url = self.url();
        self.update_with(&url)
    }

    fn update_with(&mut self, url: &str) -> Run {
        self.game.set_env("SHELLGAME_UPDATE_URL", url);
        let output = std_output_retrying(
            self.game
                .std_command_in(&self.binary, &self.game.cwd)
                .arg("update")
                .stdin(Stdio::null()),
        )
        .expect("run update");
        Run::of(&output)
    }

    fn version_of_copy(&self) -> String {
        let output = Command::new(&self.binary)
            .arg("--version")
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).trim().to_owned()
    }

    fn leftovers(&self) -> Vec<String> {
        fs::read_dir(self.binary.parent().unwrap())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name != "shellgame")
            .collect()
    }
}

#[test]
fn update_replaces_the_binary_with_the_downloaded_release() {
    if !tool_works("tar") || !tool_works("curl") {
        skip("tar or curl is not installed");
        return;
    }
    let mut bench = UpdateBench::new("#!/bin/sh\necho 'shellgame 9.9.9 (fake)'\n");
    let old = bench.version_of_copy();

    let run = bench.update();
    run.assert_code(0);
    run.assert_says("ShellGame byl aktualizován:");
    run.assert_says(&format!("{old} -> shellgame 9.9.9 (fake)"));

    assert_eq!(
        bench.version_of_copy(),
        "shellgame 9.9.9 (fake)",
        "the copy is the new binary now"
    );
    assert!(
        fs::read_to_string(&bench.binary)
            .unwrap()
            .starts_with("#!/bin/sh"),
        "the file was replaced, not patched"
    );
    assert!(
        fs::metadata(&bench.binary).unwrap().permissions().mode() & 0o111 != 0,
        "the new binary is executable"
    );
    assert!(
        bench.leftovers().is_empty(),
        "no temporary files stay behind: {:?}",
        bench.leftovers()
    );
}

#[test]
fn update_leaves_the_binary_alone_when_it_is_current() {
    if !tool_works("tar") || !tool_works("curl") {
        skip("tar or curl is not installed");
        return;
    }
    let same = shellgame::cli::version_line();
    let mut bench = UpdateBench::new(&format!("#!/bin/sh\necho '{same}'\n"));
    let before = fs::read(&bench.binary).unwrap();

    let run = bench.update();
    run.assert_code(0);
    run.assert_says(&format!("Již používáte nejnovější verzi: {same}"));
    assert_eq!(fs::read(&bench.binary).unwrap(), before);
    assert!(bench.leftovers().is_empty());
}

#[test]
fn update_refuses_a_download_that_does_not_run() {
    if !tool_works("tar") || !tool_works("curl") {
        skip("tar or curl is not installed");
        return;
    }
    let mut bench = UpdateBench::new("this is not a program\n");
    let before = fs::read(&bench.binary).unwrap();

    let run = bench.update();
    run.assert_code(1);
    run.assert_says("Stažený program nelze spustit");
    assert_eq!(
        fs::read(&bench.binary).unwrap(),
        before,
        "the running binary is untouched"
    );
    assert!(bench.leftovers().is_empty());

    // A program that fails `--version` is not an update either.
    let mut bench = UpdateBench::new("#!/bin/sh\nexit 7\n");
    let before = fs::read(&bench.binary).unwrap();
    bench.update().assert_code(1);
    assert_eq!(fs::read(&bench.binary).unwrap(), before);
}

#[test]
fn update_reports_a_failed_download() {
    if !tool_works("tar") || !tool_works("curl") {
        skip("tar or curl is not installed");
        return;
    }
    let mut bench = UpdateBench::new("#!/bin/sh\necho x\n");
    let before = fs::read(&bench.binary).unwrap();

    let missing = format!(
        "file://{}",
        bench.game.base.join("nonexistent.tar.gz").display()
    );
    let run = bench.update_with(&missing);
    run.assert_code(1);
    run.assert_says("Stažení");
    run.assert_says("se nepodařilo");
    assert_eq!(fs::read(&bench.binary).unwrap(), before);
    assert!(bench.leftovers().is_empty(), "{:?}", bench.leftovers());

    // An archive without the binary.
    let empty_release = bench.game.base.join("empty-release");
    fs::create_dir_all(&empty_release).unwrap();
    fs::write(empty_release.join("README"), "not the binary").unwrap();
    let archive = bench.game.base.join("wrong.tar.gz");
    let status = Command::new("tar")
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(&empty_release)
        .arg("README")
        .status()
        .unwrap();
    assert!(status.success());
    let run = bench.update_with(&format!("file://{}", archive.display()));
    run.assert_code(1);
    run.assert_says("Rozbalení staženého archivu se nepodařilo");
    assert_eq!(fs::read(&bench.binary).unwrap(), before);
}

#[test]
fn update_explains_how_to_reinstall_when_it_cannot_write_next_to_the_binary() {
    if !tool_works("tar") || !tool_works("curl") {
        skip("tar or curl is not installed");
        return;
    }
    let mut bench = UpdateBench::new("#!/bin/sh\necho 'shellgame 9.9.9 (fake)'\n");
    if permissions_are_ignored(&bench.game.base) {
        skip("running as root: directory permissions are not enforced");
        return;
    }
    let dir = bench.binary.parent().unwrap().to_owned();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).unwrap();
    let before = fs::read(&bench.binary).unwrap();

    let run = bench.update();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    run.assert_code(1);
    run.assert_says("nelze zapisovat");
    run.assert_says("Aktualizaci můžete provést znovu instalačním skriptem:");
    run.assert_says("curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | bash");
    assert_eq!(fs::read(&bench.binary).unwrap(), before);
}

#[test]
fn update_explains_how_to_reinstall_when_curl_is_missing() {
    let mut bench = if tool_works("tar") {
        UpdateBench::new("#!/bin/sh\necho x\n")
    } else {
        skip("tar is not installed");
        return;
    };
    let empty = bench.game.base.join("no-tools");
    fs::create_dir_all(&empty).unwrap();
    bench.game.set_env("PATH", &empty);

    let run = bench.update();
    run.assert_code(1);
    run.assert_says("Pro aktualizaci je potřeba příkaz 'curl', ale nebyl nalezen.");
    run.assert_says("install.sh");
}
