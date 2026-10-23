//! The command line against the real binary, run the way the game shell runs it: with
//! `SHELLGAME_SANDBOX` set, so the directives on stderr show and are decoded here. Every test gets
//! its own world (see support/). Positions come from the course (first strict `cd` level, ...);
//! written down are only where a few sections begin. What the game says is taken from the course
//! (titles, hints, answers), from `messages` or from the screens the binary draws, never retyped.
//! The solutions and strict `cd` levels in the real shells are tests/e2e.rs.

mod support;

use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use shellgame::game::{Pos, dedent, messages, remove_tree, start_dir};
use shellgame::levels::COURSE;
use shellgame::state::{Progress, State};
use shellgame::ui::{Line, Term, plain, screens};
use support::{BIN, Run, World, have, landlock, squash};

const FIRST: Pos = Pos { section: 0, level: 1 };

/// The binary draws for 80 columns without colour when it has no terminal, as here.
const TERM: Term = Term { width: 80, color: false, stderr: false };

fn at(section: usize, level: usize) -> Pos {
    Pos { section, level }
}

/// The first level that sets files up.
fn with_files() -> Pos {
    Pos::all().find(|pos| !pos.level().setup.is_empty()).unwrap()
}

fn heading(pos: Pos) -> String {
    format!("{pos}: {}", pos.level().title)
}

fn section_title(n: usize) -> String {
    format!("Sekce {n}: {}", COURSE[n].title)
}

/// The answer that completes the first level: what its solution submits, a plain word.
fn first_answer() -> String {
    let solution = dedent(FIRST.level().solution);
    let answer = solution.lines().last().and_then(|line| line.strip_prefix("shellgame submit "));
    let answer = answer.expect("the first solution submits an answer").trim_matches(['"', '\'']);
    assert!(!answer.contains(['$', ' ']), "the first answer is a plain word: {answer:?}");
    answer.to_owned()
}

/// The text of a screen as the binary prints it.
fn screen(lines: Vec<Line>) -> String {
    lines.iter().map(|line| plain(line)).collect::<Vec<_>>().join("\n")
}

/// The pages of a section's intro: separated by a line `---`.
fn pages(intro: &str) -> usize {
    dedent(intro).lines().filter(|line| line.trim() == "---").count() + 1
}

fn mode(path: &Path) -> u32 {
    fs::symlink_metadata(path).unwrap().permissions().mode() & 0o7777
}

fn chmod(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

impl Run {
    #[track_caller]
    fn code(&self, code: i32) -> &Run {
        assert_eq!(self.code, code, "{self:#?}");
        self
    }

    /// `text` appears in stdout, ignoring escapes, box drawing and line folds.
    #[track_caller]
    fn says(&self, text: &str) -> &Run {
        assert!(squash(&self.out).contains(&squash(text)), "no {text:?} in {self:#?}");
        self
    }

    #[track_caller]
    fn says_not(&self, text: &str) -> &Run {
        assert!(!squash(&self.out).contains(&squash(text)), "{text:?} in {self:#?}");
        self
    }

    /// The directives on stderr, decoded: `verb argument ...`.
    fn directives(&self) -> Vec<String> {
        let decode = |word: &str| String::from_utf8(STANDARD.decode(word).unwrap()).unwrap();
        let lines = self.err.lines().filter_map(|l| l.strip_prefix("__SHELLGAME_EXEC__v1 "));
        let join = |line: &str| -> String {
            let words = line.split(' ').enumerate();
            words
                .map(|(i, word)| if i == 0 { word.into() } else { decode(word) })
                .collect::<Vec<_>>()
                .join(" ")
        };
        lines.map(join).collect()
    }

    fn moved_to(&self, dir: &Path) -> bool {
        self.directives().contains(&format!("cd {}", dir.display()))
    }
}

impl World {
    /// A game whose intro and level 0.1 were shown.
    fn started() -> World {
        let game = World::new();
        game.run(&[]).code(0);
        game
    }

    fn run_with(&self, args: &[&str], stdin: &str) -> Run {
        self.exec(self.command(BIN).args(args), stdin)
    }

    /// Runs a command that must succeed and say `text`.
    #[track_caller]
    fn ok(&self, args: &[&str], text: &str) -> Run {
        let run = self.run(args);
        run.code(0).says(text);
        run
    }

    /// Runs commands that must end as expected: (arguments, status, a text on stdout).
    #[track_caller]
    fn expect(&self, cases: &[(&[&str], i32, &str)]) {
        for (args, code, text) in cases {
            self.run(args).code(*code).says(text);
        }
    }

    fn resume(&self, pos: Pos) -> Run {
        self.ok(&["resume", &pos.to_string()], &heading(pos))
    }

    fn state(&self) -> Value {
        serde_json::from_slice(&self.raw_state()).unwrap()
    }

    fn raw_state(&self) -> Vec<u8> {
        fs::read(self.state_file()).unwrap()
    }

    fn state_file(&self) -> PathBuf {
        self.state.join("state.json")
    }

    fn put_state(&self, state: &Value) {
        fs::create_dir_all(&self.state).unwrap();
        fs::write(self.state_file(), state.to_string()).unwrap();
    }

    /// The saved game with this current level and nothing completed.
    fn put_level(&self, level: &str, finished: bool) {
        self.put_state(&json!({"version": 3, "workspace": self.ws, "level": level, "finished": finished, "levels": {}}));
    }

    /// The saved progress of a level.
    fn progress(&self, pos: Pos) -> Progress {
        serde_json::from_value(self.state()["levels"][pos.to_string()].clone()).unwrap()
    }
}

/// Every path below a directory.
fn tree(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).unwrap().flatten() {
        found.push(entry.path());
        if entry.file_type().unwrap().is_dir() {
            found.extend(tree(&entry.path()));
        }
    }
    found.sort();
    found
}

/// An executable script from an indented block of lines (see `dedent`).
fn write_script(path: &Path, text: &str) {
    fs::write(path, dedent(text) + "\n").unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn the_first_run_starts_a_game_with_the_intro_and_the_first_level() {
    let g = World::new();
    let r = g.ok(&[], &section_title(0));
    r.says(&heading(FIRST));
    let (state, ws) = (g.state(), g.ws.display());
    assert_eq!([&state["version"], &state["level"]], [&json!(3), &json!("0.1")]);
    assert!(state["levels"].as_object().unwrap().keys().eq(["0.1"]));
    let expected = [format!("export SHELLGAME_WORKSPACE {ws}"), format!("cd {ws}/level-0")];
    assert_eq!(r.directives(), expected);
    assert_eq!(r.err.lines().count(), 2, "nothing but the directives");
    // Back at the level, nothing is moved and no intro is shown again.
    let again = g.exec(g.command(BIN).current_dir(start_dir(FIRST, &g.ws)), "");
    again.code(0).says(&heading(FIRST)).says_not(&section_title(0));
    assert!(again.directives().iter().all(|directive| !directive.starts_with("cd ")));
}

#[test]
fn answers_are_counted_and_a_right_one_moves_on_into_the_next_section() {
    let g = World::started();
    let reply = FIRST.level().answer.judge(Some("wrong")).unwrap_err();
    // Several words are one answer, a dash is not an option, a missing answer is asked for.
    #[rustfmt::skip]
    g.expect(&[
        (&["submit", "wrong"], 0, &reply), (&["submit", "-5", "a", "b"], 0, &reply),
        (&["submit"], 0, messages::ANSWER_REQUIRED), (&["hint"], 0, FIRST.level().hints[0]),
    ]);
    assert!(g.state()["levels"]["0.1"]["done"].is_null());
    let r = g.ok(&["submit", &first_answer()], FIRST.level().success);
    let done = g.progress(FIRST);
    assert_eq!((done.attempts, done.hints), (3, 1), "every wrong or missing answer counts");
    r.says(&screens::stats(&done));
    r.says(&section_title(1)).says(&heading(at(1, 1)));
    let state = g.state();
    assert_eq!(state["level"], "1.1");
    assert!(state["levels"]["0.1"]["done"].is_u64() && state["levels"]["1.1"]["done"].is_null());
    g.ok(&["status"], "shellgame resume 1.1");
}

#[test]
fn every_hint_shows_the_unlocked_ones_and_one_more_until_none_is_left() {
    let g = World::started();
    let hints = FIRST.level().hints;
    let shown =
        |r: &Run| hints.iter().filter(|hint| squash(&r.out).contains(&squash(hint))).count();
    let saved = || g.state()["levels"]["0.1"]["hints"].clone();
    // Repeating before the first hint unlocks nothing and shows none.
    let r = g.run(&["hint", "-r"]);
    assert_eq!((r.code, shown(&r), saved()), (0, 0, json!(0)), "{r:#?}");
    for n in 1..=hints.len() {
        let r = g.ok(&["hint"], hints[n - 1]);
        assert_eq!(shown(&r), n, "{r:#?}");
        // `--repeat` shows the same hints again without unlocking another.
        let again = g.run(&["hint", "--repeat"]);
        assert_eq!((again.code, shown(&again), saved()), (0, n, json!(n)), "{again:#?}");
    }
    let r = g.run(&["hint"]);
    assert_eq!((r.code, shown(&r), saved()), (0, hints.len(), json!(hints.len())), "{r:#?}");
}

#[test]
fn reset_rebuilds_the_level_and_brings_the_player_back() {
    let (g, pos) = (World::new(), with_files());
    g.resume(pos);
    let root = pos.section_dir(&g.ws);
    let before = tree(&root);
    assert!(before.len() > 1);
    for entry in fs::read_dir(&root).unwrap().flatten() {
        let is_dir = entry.file_type().unwrap().is_dir();
        let gone =
            if is_dir { fs::remove_dir_all(entry.path()) } else { fs::remove_file(entry.path()) };
        gone.unwrap();
    }
    let r = g.ok(&["reset"], &heading(pos));
    assert!(r.moved_to(&start_dir(pos, &g.ws)));
    assert_eq!(tree(&root), before);
}

/// What a player can do to the directories of the game: (what, damage to the workspace and the
/// section directory, given a directory outside the game).
type Damage = (&'static str, fn(&Path, &Path, &Path));

/// Whatever the player did to the workspace or the section directory, `reset` rebuilds the level,
/// leaves both usable and brings the player to the start. A link is removed, never followed.
#[test]
fn reset_repairs_the_workspace_and_the_section_directory() {
    let pos = with_files();
    let damages: [Damage; 5] = [
        ("chmod 000 on the workspace", |ws, _, _| chmod(ws, 0o000)),
        ("chmod a-x on the workspace", |ws, _, _| chmod(ws, mode(ws) & !0o111)),
        ("a file in place of the workspace", |ws, _, _| {
            remove_tree(ws).unwrap();
            fs::write(ws, "a file").unwrap();
        }),
        ("a link to elsewhere in place of the section directory", |_, section, outside| {
            remove_tree(section).unwrap();
            symlink(outside, section).unwrap();
        }),
        ("chmod 000 on the section directory", |_, section, _| chmod(section, 0o000)),
    ];
    for (what, damage) in damages {
        let g = World::new();
        g.resume(pos);
        let (section, outside) = (pos.section_dir(&g.ws), g.out.join("outside"));
        let before = tree(&section);
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("keep"), "keep").unwrap();

        damage(&g.ws, &section, &outside);
        let r = g.run(&["reset"]);

        r.code(0).says(&heading(pos));
        assert!(r.moved_to(&start_dir(pos, &g.ws)), "{what}: {r:#?}");
        for dir in [&g.ws, &section] {
            assert!(!dir.is_symlink() && mode(dir) & 0o700 == 0o700, "{what}: {dir:?}");
        }
        assert_eq!(tree(&section), before, "{what}");
        assert_eq!(tree(&outside), [outside.join("keep")], "{what}: the link was followed");
        assert_eq!(fs::read_to_string(outside.join("keep")).unwrap(), "keep", "{what}");
    }
}

#[test]
fn show_shows_an_assignment_or_an_intro_without_touching_the_game() {
    let g = World::started();
    let before = g.raw_state();
    let (first, second, third) = (heading(FIRST), heading(at(1, 2)), section_title(2));
    // `repeat` is another name for `show`.
    #[rustfmt::skip]
    g.expect(&[
        (&["show"], 0, &first), (&["show", "0.1"], 0, &first), (&["show", "1.2"], 0, &second),
        (&["show", "2"], 0, &third), (&["show", "0"], 0, &section_title(0)),
        (&["repeat"], 0, &first), (&["repeat", "1.2"], 0, &second),
        (&["show", "1.99"], 1, "shellgame levels"), (&["show", "99"], 1, "shellgame levels"),
        (&["show", "x"], 1, "shellgame levels"),
    ]);
    assert_eq!(g.raw_state(), before);
}

#[test]
fn resume_continues_on_another_computer() {
    let g = World::new();
    let r = g.resume(at(3, 2));
    r.says_not(&section_title(3));
    assert!(r.moved_to(&start_dir(at(3, 2), &g.ws)));
    assert!(g.state()["levels"].as_object().unwrap().keys().eq(["3.2"]), "nothing else completed");
    g.ok(&["status"], &screen(screens::resume_hint(&TERM, "3.2")));
    // A section starts with its intro and its first level.
    g.ok(&["resume", "3"], &section_title(3)).says(&heading(at(3, 1)));
    // Without a target the levels are listed; a wrong one leads to them.
    let row = format!("3.1 {}", at(3, 1).level().title);
    g.run(&["resume"]).code(1).says(&section_title(0)).says(&row);
    for target in ["99", "1.99", "x", ""] {
        g.run(&["resume", target]).code(1).says("shellgame levels");
    }
    assert_eq!(g.state()["level"], "3.1");
    g.ok(&["exit"], "shellgame resume 3.1");
}

#[test]
fn resume_keeps_every_record_and_plays_a_finished_game_on() {
    let g = World::started();
    g.ok(&["submit", &first_answer()], FIRST.level().success);
    let done = g.state()["levels"]["0.1"].clone();
    assert!(done["done"].is_u64());
    g.resume(FIRST);
    assert_eq!(g.state()["levels"]["0.1"], done);
    assert_eq!(g.state()["level"], "0.1");
    g.put_state(&json!({"version": 3, "workspace": g.ws, "level": "0.1", "finished": true, "levels": {"0.1": done}}));
    g.resume(at(1, 1));
    let state = g.state();
    assert_eq!([&state["finished"], &state["level"]], [&json!(false), &json!("1.1")]);
    assert_eq!(state["levels"]["0.1"], done);
    assert!(state["levels"]["1.1"]["started"].as_u64() > Some(0));
}

#[test]
fn levels_lists_every_level_marking_the_saved_and_the_completed_ones() {
    let g = World::new();
    let r = g.ok(&["levels"], "shellgame resume");
    assert!(!g.state_file().exists(), "listing starts no game");
    let rows = |r: &Run| -> Vec<String> {
        r.out.lines().map(squash).filter(|line| !line.is_empty()).collect()
    };
    // Every section and level, in course order; no marks without a game.
    let mut expected = Vec::new();
    for (n, section) in COURSE.iter().enumerate() {
        expected.push(section_title(n));
        let row = |level: usize| format!("{} {}", at(n, level), section.levels[level - 1].title);
        expected.extend((1..=section.levels.len()).map(row));
    }
    let shown = rows(&r);
    let found: Vec<_> =
        expected.iter().map(|text| shown.iter().position(|line| line == text)).collect();
    assert!(
        found.windows(2).all(|pair| pair[0] < pair[1]) && found[0].is_some(),
        "{expected:?}\n{r:#?}"
    );
    r.says_not("▸").says_not("✓");
    // Marks follow the saved game; the section heading is not a row.
    let done = json!({"started": 100, "done": 160, "attempts": 0, "hints": 0});
    let levels = json!({"0.1": done, "1.1": done, "1.2": {"started": 100, "done": null, "attempts": 0, "hints": 0}});
    g.put_state(&json!({"version": 3, "workspace": g.ws, "level": "1.2", "finished": false, "levels": levels}));
    let marked = rows(&g.ok(&["levels"], "shellgame resume"));
    let mark = |pos: Pos| marked.iter().find(|line| line.starts_with(&format!("{pos} "))).unwrap();
    assert!(mark(FIRST).ends_with('✓') && mark(at(1, 1)).ends_with('✓'));
    assert!(mark(at(1, 2)).contains('▸') && !mark(at(1, 2)).contains('✓'));
    assert!(!mark(at(1, 3)).contains(['✓', '▸']));
}

#[test]
fn remove_deletes_the_workspace_and_the_state_after_asking() {
    let g = World::started();
    for answer in ["", "n\n"] {
        g.run_with(&["remove"], answer).code(1);
        assert!(g.ws.is_dir() && g.state_file().is_file(), "declining deletes nothing");
    }
    let r = g.run_with(&["remove"], "y\n");
    r.code(0);
    assert!(!g.ws.exists() && !g.state_file().exists() && r.directives() == ["exit"]);
    g.run(&["remove", "--yes"]).code(1);
    // It works on a damaged state too, which nothing else repairs, and never leaves /tmp.
    g.run(&[]).code(0);
    fs::write(g.state_file(), "{ not json").unwrap();
    g.run(&[]).code(1);
    assert_eq!(g.raw_state(), b"{ not json");
    g.run(&["remove", "--yes"]).code(0);
    assert!(!g.ws.exists() && !g.state_file().exists());
    g.put_state(&json!({"version": 3, "workspace": "/etc/sg", "level": "0.1", "finished": false, "levels": {}}));
    g.run(&["remove", "--yes"]).code(1);
    assert!(g.state_file().exists());
}

#[test]
fn a_saved_game_of_an_older_version_is_set_aside_and_of_a_newer_one_refused() {
    let g = World::new();
    // A Python save names its level, so the notice, at the end of the output, says how to get back
    // there; other old saves point to the list of levels.
    let saves = [
        (json!({"version": "2.1", "workspace": g.ws, "current_level": "1.4"}), Some("1.4")),
        (json!({"version": "2.1", "workspace": g.ws, "current_level": "5.0"}), Some("5")),
        (json!({"version": "2.0", "workspace": g.ws}), None),
    ];
    for (n, (old, resume)) in saves.into_iter().enumerate() {
        g.put_state(&old);
        let r = g.ok(&[], &heading(FIRST));
        let name =
            if n == 0 { "state-old.json".into() } else { format!("state-old-{}.json", n + 1) };
        let file = g.state.join(name);
        assert_eq!(fs::read_to_string(&file).unwrap(), old.to_string());
        assert_eq!(g.state()["version"], 3);
        let notice = squash(&screen(screens::retired(&TERM, &file.display().to_string(), resume)));
        assert!(squash(&r.out).ends_with(&notice), "not last: {notice:?} in {r:#?}");
    }
    // A save of a newer version is refused and left as it is.
    let newer =
        json!({"version": 4, "workspace": g.ws, "level": "0.1", "finished": false, "levels": {}});
    g.put_state(&newer);
    for args in [&[][..], &["status"], &["hint"]] {
        g.run(args).code(1).says("novější verze").says("shellgame update");
        assert_eq!(g.raw_state(), newer.to_string().into_bytes());
    }
    assert!(!g.state.join("state-old-4.json").exists(), "a newer save is not set aside");
}

#[test]
fn a_finished_game_is_inert() {
    let g = World::started();
    let done = json!({"started": 100, "done": 160, "attempts": 0, "hints": 0});
    let last = Pos::all().last().unwrap().to_string();
    let finished = json!({"version": 3, "workspace": g.ws, "level": last, "finished": true, "levels": {"0.1": done}});
    g.put_state(&finished);
    let complete =
        screen(screens::complete(&TERM, &serde_json::from_value::<State>(finished).unwrap()));
    let before = g.raw_state();
    for args in [&["hint"][..], &["submit", "x"], &["reset"], &[], &["show"]] {
        g.ok(args, &complete).says(&screens::duration(60));
    }
    assert_eq!(g.raw_state(), before);
    g.ok(&["show", "1.2"], &heading(at(1, 2))).says_not(&complete);
    assert_eq!(g.raw_state(), before);
    g.resume(at(1, 1));
    assert_eq!(g.state()["finished"], false, "resuming plays on");
}

#[test]
fn a_position_that_no_longer_exists_resumes_at_the_nearest_following_level() {
    let g = World::started();
    for (saved, resumed) in [("1.99", "2.1"), ("garbage", "0.1")] {
        let resumed: Pos = resumed.parse().unwrap();
        g.put_level(saved, false);
        let r = g.ok(&["hint"], &screen(screens::stale(&TERM, saved, resumed)));
        r.says(&heading(resumed));
        assert_eq!(g.state()["level"], resumed.to_string());
        assert_eq!(g.progress(resumed).hints, 0, "the command was not applied to it");
    }
}

#[test]
fn a_cleared_workspace_is_rebuilt() {
    let (g, pos) = (World::new(), with_files());
    g.resume(pos);
    let before = tree(&g.ws);
    assert!(before.len() > 1);
    fs::remove_dir_all(&g.ws).unwrap();
    g.ok(&["hint"], pos.level().hints[0]);
    assert_eq!(tree(&g.ws), before);
}

#[test]
fn a_link_in_place_of_the_next_section_is_replaced_and_never_followed() {
    let g = World::started();
    let (next, outside) = (FIRST.next().unwrap(), g.out.join("outside"));
    assert_ne!(next.section, FIRST.section, "the first section has one level");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, next.section_dir(&g.ws)).unwrap();
    let r = g.ok(&["submit", &first_answer()], FIRST.level().success);
    r.says(&heading(next));
    assert!(r.moved_to(&start_dir(next, &g.ws)));
    let state = g.state();
    assert!(state["levels"]["0.1"]["done"].is_u64() && state["level"] == next.to_string());
    assert!(!next.section_dir(&g.ws).is_symlink() && start_dir(next, &g.ws).is_dir());
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0, "nothing was written through it");
}

#[test]
fn the_hooks_fail_open() {
    // Malformed calls and a missing game block nothing: a broken hook never locks the player in.
    let g = World::new();
    #[rustfmt::skip]
    g.expect(&[
        (&["cd-hook"], 0, ""), (&["cd-hook", "..", "/w"], 0, ""), (&["cd-hook", "pre", "/w"], 0, ""),
        (&["cd-hook", "pre", "/w", "", "--", "/etc"], 0, ""),
    ]);
}

#[test]
fn a_mistyped_command_names_what_was_typed_and_what_was_meant() {
    let g = World::new();
    let r = g.run(&["sumbit", "x"]);
    assert!(r.code == 2 && r.err.contains("'sumbit'") && r.err.contains("'submit'"), "{r:#?}");
    let r = g.run(&["--shell", "zsh"]);
    assert!(r.code == 2 && r.err.contains("'zsh'") && r.err.contains("bash, fish"), "{r:#?}");
}

#[test]
fn update_replaces_the_binary_with_the_downloaded_release() {
    if !have("curl") || !have("tar") {
        return eprintln!("SKIPPED: curl and tar are needed");
    }
    let mut g = World::new();
    let (release, copy, archive) =
        (g.tools.join("release"), g.tools.join("shellgame"), g.root.join("release.tar.gz"));
    fs::create_dir(&release).unwrap();
    let script = r#"
        #!/bin/sh
        echo 'shellgame 9.9.9 (test)'
    "#;
    write_script(&release.join("shellgame"), script);
    let tar = Command::new("tar")
        .arg("-czf")
        .arg(&archive)
        .arg("-C")
        .arg(&release)
        .arg("shellgame")
        .status();
    assert!(tar.unwrap().success());
    fs::copy(BIN, &copy).unwrap();
    g.set("SHELLGAME_UPDATE_URL", format!("file://{}", archive.display()));
    let run = |g: &World, args: &[&str]| g.exec(g.command(&copy).args(args), "");
    let original = run(&g, &["--version"]).out;
    // Inside the sandbox the binary cannot be replaced, so it does not try.
    g.set("SHELLGAME_SANDBOX", "landlock");
    run(&g, &["update"]).code(1).says("shellgame update");
    assert_eq!(run(&g, &["--version"]).out, original);
    g.unset("SHELLGAME_SANDBOX");
    run(&g, &["update"]).code(0).says("9.9.9");
    assert_eq!(run(&g, &["--version"]).out, "shellgame 9.9.9 (test)\n");
    // A failed download changes nothing; no scratch directory is left next to the binary.
    g.set("SHELLGAME_UPDATE_URL", "file:///nonexistent.tar.gz");
    let again = g.tools.join("again");
    fs::copy(BIN, &again).unwrap();
    g.exec(g.command(&again).arg("update"), "").code(1);
    assert_eq!(fs::read(&again).unwrap(), fs::read(BIN).unwrap());
    let names: Vec<_> = fs::read_dir(&g.tools).unwrap().map(|e| e.unwrap().file_name()).collect();
    assert!(names.iter().all(|name| !name.to_string_lossy().starts_with('.')), "{names:?}");
}

#[test]
fn the_version_works_anywhere_and_outside_the_game_shell_nothing_is_replayed() {
    let mut g = World::new();
    g.unset("SHELLGAME_SANDBOX");
    let version = format!("shellgame {} (", env!("CARGO_PKG_VERSION"));
    assert!(g.run(&["--version"]).code(0).out.starts_with(&version));
    assert!(g.run(&["-V"]).code(0).out.starts_with(&version));
    assert!(g.run(&["levels"]).code(0).err.is_empty());
    g.run(&["exit"]).code(1);
    g.run(&["status"]).code(1);
    assert!(!g.state_file().exists(), "`status` starts no game");
    g.put_level("0.1", false);
    assert!(g.run_with(&["remove"], "y\n").code(0).err.is_empty());
}

#[test]
fn a_workspace_outside_tmp_and_a_state_directory_above_home_are_refused() {
    let mut g = World::new();
    symlink("/usr", g.root.join("link")).unwrap();
    for workspace in [g.root.join("link/ws"), "/var/sg-test".into(), "/tmp".into()] {
        g.set("SHELLGAME_WORKSPACE", workspace);
        g.run(&[]).code(1);
        assert!(!g.state.exists());
    }
    for state_dir in [g.home.clone(), g.out.clone()] {
        g.set("SHELLGAME_STATE_DIR", &state_dir).unset("SHELLGAME_WORKSPACE");
        g.run(&[]).code(1);
        assert!(!state_dir.join("state.json").exists());
    }
}

#[test]
fn outside_the_game_shell_the_bare_command_starts_it_confined_unconfined_or_not_at_all() {
    let mut g = World::new();
    let (fakes, log) = (g.tools.join("fakes"), g.root.join("shell.log"));
    fs::create_dir(&fakes).unwrap();
    let script = format!(
        r#"
            #!/bin/sh
            env > {}
            exit 7
        "#,
        log.display()
    );
    write_script(&fakes.join("bash"), &script);
    g.unset("SHELLGAME_SANDBOX");
    g.set("PATH", format!("{}:{}", fakes.display(), std::env::var("PATH").unwrap()));
    // `--unsafe` starts it as it is; the shell's own exit status is not an error.
    g.run(&["--shell", "bash", "--unsafe"]).code(0);
    assert!(fs::read_to_string(&log).unwrap().contains("SHELLGAME_SANDBOX=off"));
    // Without it the shell runs confined where Landlock works, and does not run elsewhere.
    fs::remove_file(&log).unwrap();
    if landlock() {
        g.run(&["--shell", "bash"]).code(0);
        assert!(fs::read_to_string(&log).unwrap().contains("SHELLGAME_SANDBOX=landlock"));
    } else {
        g.expect(&[(&["--shell", "bash"], 1, "Landlock")]);
        assert!(!log.exists());
    }
}

#[test]
fn outside_the_game_shell_resume_runs_in_a_new_game_shell_and_listings_start_none() {
    let mut g = World::new();
    g.unset("SHELLGAME_SANDBOX");
    let (fakes, log) = (g.tools.join("fakes"), g.root.join("shell.log"));
    fs::create_dir(&fakes).unwrap();
    write_script(&fakes.join("bash"), &format!("#!/bin/sh\ntouch {}\n", log.display()));
    let path = format!("{}:{}", fakes.display(), std::env::var("PATH").unwrap());
    // Print-only commands never start a shell, `resume` with a level does.
    g.set("PATH", path);
    g.expect(&[
        (&["levels"], 0, "shellgame resume"),
        (&["resume"], 1, &section_title(0)),
        (&["resume", "99"], 1, "shellgame levels"),
    ]);
    assert!(!log.exists());
    g.run(&["--shell", "bash", "--unsafe", "resume", "3.2"]);
    assert!(log.exists());
    // In the real shells the player ends up at the level, saved, and the command runs only once.
    let (pos, here) = (at(3, 2), g.root.join("here"));
    for shell in ["bash", "fish"].into_iter().filter(|shell| have(shell)) {
        g.set("PATH", std::env::var("PATH").unwrap());
        let g = &g;
        let _ = fs::remove_file(g.state_file());
        let typed = format!(
            "pwd > '{}'; echo \"[$SHELLGAME_PENDING]\" >> '{}'\nexit\n",
            here.display(),
            here.display()
        );
        let args = ["--shell", shell, "--unsafe", "resume", &pos.to_string()];
        let run = g.exec(g.command(BIN).args(args), &typed);
        run.code(0).says(&heading(pos));
        let written = fs::read_to_string(&here).unwrap();
        assert_eq!(
            written,
            format!("{}\n[]\n", start_dir(pos, &g.ws).display()),
            "{shell}: {run:?}"
        );
        assert_eq!(g.state()["level"], "3.2");
    }
}

/// Runs a binary on a terminal and answers each prompt with the next key. A prompt is the program
/// waiting in non-canonical mode (as `pause` does) after printing something new since the last key.
/// Prints what the terminal showed, the offsets in it where keys were sent, and the status.
const PTY_HELPER: &str = r#"
import fcntl, json, os, pty, select, struct, sys, termios, time
binary, cwd, keys, *args = sys.argv[1:]
keys = [key.encode() for key in json.loads(keys)]
pid, fd = pty.fork()
if pid == 0:
    fcntl.ioctl(0, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 100, 0, 0))
    os.chdir(cwd)
    os.execv(binary, [binary] + args)

def waits():
    try:
        return not termios.tcgetattr(fd)[3] & termios.ICANON
    except termios.error:
        return False

out, sent, fresh, deadline = b"", [], True, time.time() + 30
while time.time() < deadline:
    if select.select([fd], [], [], 0.05)[0]:
        try:
            data = os.read(fd, 65536)
        except OSError:
            break
        if not data:
            break
        out, fresh = out + data, True
    elif keys and fresh and waits():
        sent.append(len(out))
        os.write(fd, keys.pop(0))
        fresh = False
for _ in range(500):
    done, status = os.waitpid(pid, os.WNOHANG)
    if done:
        break
    time.sleep(0.01)
else:
    os.kill(pid, 9)
    _, status = os.waitpid(pid, 0)
code = os.waitstatus_to_exitcode(status)
sys.stdout.buffer.write(out + ("\nKEYS:%s\nSTATUS:%d" % (json.dumps(sent), code)).encode())
"#;

/// What the program showed on a terminal and at which offsets of it each key was sent.
fn in_pty(g: &World, args: &[&str], keys: &[&str]) -> Option<(Run, Vec<usize>)> {
    if !have("python3") {
        eprintln!("SKIPPED: python3 is needed for a terminal");
        return None;
    }
    let mut python = g.command("python3");
    python.args(["-I", "-c", PTY_HELPER, BIN]).arg(&g.cwd).arg(json!(keys).to_string()).args(args);
    let text = g.exec(&mut python, "").out;
    let (rest, status) = text.rsplit_once("\nSTATUS:").unwrap_or_else(|| panic!("{text}"));
    let (shown, sent) = rest.rsplit_once("\nKEYS:").unwrap_or_else(|| panic!("{text}"));
    let run = Run { code: status.trim().parse().unwrap(), out: shown.into(), err: shown.into() };
    Some((run, serde_json::from_str(sent).unwrap()))
}

#[test]
fn enter_pages_through_an_intro_and_ctrl_c_stops_it_without_losing_anything() {
    let Some((run, sent)) = in_pty(&World::new(), &[], &["\n"; 8]) else { return };
    run.code(0).says(&section_title(0)).says(&heading(FIRST));
    assert!(!sent.is_empty(), "the intro waited for Enter: {run:#?}");
    // Ctrl-C at the first prompt of an intro with more pages ends the paging, not the command.
    let n = COURSE.iter().position(|s| pages(s.intro) > 1).expect("an intro with several pages");
    let g = World::new();
    let Some((run, sent)) = in_pty(&g, &["resume", &n.to_string()], &["\x03"]) else { return };
    run.code(0).says(&heading(at(n, 1)));
    assert_eq!(sent.len(), 1, "one prompt, then no more pages: {run:#?}");
    // The directives were emitted before the prompt, so Ctrl-C could not lose them.
    let at: Vec<_> = run.err.match_indices("__SHELLGAME_EXEC__").map(|(at, _)| at).collect();
    assert!(!at.is_empty() && at.iter().all(|at| *at < sent[0]), "{run:?}");
    assert_eq!(g.state()["level"], format!("{n}.1"));
}
