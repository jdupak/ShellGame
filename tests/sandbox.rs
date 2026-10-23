//! The game shell cannot change anything outside /tmp, the game's state directory and /dev, and is
//! not confined further. The real binary and scripts run through `launch_with` in bash and fish, in
//! a fake home full of victims and a state directory outside /tmp. Without Landlock the tests skip
//! with a note, unless `SHELLGAME_TEST_LANDLOCK=required` (CI sets it) turns that into a failure.

mod support;

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;

use shellgame::shell::{LaunchOptions, ShellKind, launch_with};
use support::{BIN, World, have, landlock};

/// What the player types in the game shell, where `$H` is the fake home, `$S` a directory in /tmp
/// and `$P` the test process, which runs outside the sandbox; each after its kind:
///
/// * `ok` always works;
/// * an `attack` on the home works only without the sandbox;
/// * an `escape` never works in the sandbox, and without it may fail anyway (a hard link cannot
///   cross filesystems);
/// * a typed `chmod` that would reach the home is `guarded` by the scripts even without it;
/// * `ignored` does not matter.
const STEPS: [&str; 27] = [
    "ok ln -s $H/trapped $S/trap",
    "attack touch $H/new-file",
    "attack rm $H/rm-me",
    "attack mv $H/mv-me $H/moved",
    "attack echo x > $H/write-me",
    "attack echo x >> $H/append-me",
    "attack truncate -s 0 $H/truncate-me",
    "attack mkdir $H/new-dir",
    "attack rmdir $H/rmdir-me",
    "attack ln -s write-me $H/link",
    "attack echo x > $S/trap",
    "escape ln $H/hard-me $S/hard",
    "ignored echo x >> $S/hard",
    "guarded chmod 000 $H/chmod-me",
    "guarded chmod 000 $S/trap",
    "ok touch $S/a",
    "ok mkdir $S/d",
    "ok echo hi > $S/b",
    "ok mv $S/b $S/d/b",
    "ok chmod 600 $S/d/b",
    "ok rm $S/a",
    "ok mktemp > $S/mktemp.out",
    "ok echo x > /dev/null",
    // Signals to processes outside stay allowed: the sandbox is about files only.
    "ok kill -0 $P",
    "ignored cp $SHELLGAME_STATE_DIR/state.json $S/before.json",
    "ok shellgame submit start",
    "ok cp $SHELLGAME_STATE_DIR/state.json $S/after.json",
];

/// Steps that need python3: `truncate(2)` itself (the `truncate` tool opens the file for writing
/// first, which Landlock refuses even where it knows nothing of truncation), and a connection to a
/// unix socket outside /tmp, where the test listens, which the sandbox must not forbid.
const PYTHON_STEPS: [&str; 2] = [
    "attack python3 -I -c 'import os, sys; os.truncate(sys.argv[1], 0)' $H/shrink-me",
    "ok python3 -I -c 'import socket, sys; socket.socket(socket.AF_UNIX).connect(sys.argv[1])' $H/sock",
];

fn shells() -> Vec<ShellKind> {
    let found: Vec<_> =
        [ShellKind::Bash, ShellKind::Fish].into_iter().filter(|shell| have(shell.name())).collect();
    if found.len() < 2 {
        eprintln!("SKIPPED: only {found:?} of bash and fish is installed");
    }
    assert!(!found.is_empty(), "bash is needed");
    found
}

/// A world whose home holds the victims and a listening unix socket (`None` with a note where
/// python3 is missing or the socket cannot be made, and then [`PYTHON_STEPS`] are left out);
/// `w.cwd` is the scratch directory in /tmp.
fn victims() -> (World, Option<UnixListener>) {
    let w = World::new();
    fs::create_dir(w.home.join("rmdir-me")).unwrap();
    let names = "rm-me mv-me write-me append-me truncate-me shrink-me hard-me chmod-me trapped";
    for victim in names.split(' ') {
        fs::write(w.home.join(victim), "precious\n").unwrap();
    }
    let socket = match have("python3") {
        true => UnixListener::bind(w.home.join("sock"))
            .inspect_err(|error| eprintln!("SKIPPED: no unix socket in the home: {error}"))
            .ok(),
        false => {
            eprintln!("SKIPPED: python3 is needed for the truncate(2) and unix socket steps");
            None
        }
    };
    (w, socket)
}

/// Every entry of the home with its mode and content: what an attack would change.
fn home(w: &World) -> BTreeMap<String, String> {
    let mut seen = BTreeMap::new();
    for entry in fs::read_dir(&w.home).unwrap() {
        let path = entry.unwrap().path();
        let mode = fs::symlink_metadata(&path).unwrap().permissions().mode();
        let content = fs::read_to_string(&path).unwrap_or_default();
        seen.insert(path.display().to_string(), format!("{mode:o} {content}"));
    }
    seen
}

fn scratch(w: &World, name: &str) -> String {
    fs::read_to_string(w.cwd.join(name)).unwrap_or_default()
}

/// Plays [`STEPS`] (and [`PYTHON_STEPS`] with `python`) in a game shell that is fed from a file and
/// started the way the player's is, and checks that each worked as its kind says. Returns what the
/// shell wrote to stderr.
fn play(w: &World, shell: ShellKind, sandbox: bool, python: bool) -> String {
    let status = ["$?", "$status"][usize::from(shell == ShellKind::Fish)];
    let extra = if python { &PYTHON_STEPS[..] } else { &[] };
    let steps: Vec<_> =
        STEPS.iter().chain(extra).map(|step| step.split_once(' ').unwrap()).collect();
    let script: String =
        steps.iter().map(|(_, command)| format!("{command}\necho \"@{status}\"\n")).collect();
    let [input, out, err] = ["input", "stdout", "stderr"].map(|name| w.root.join(name));
    fs::write(&input, script).unwrap();
    let options = LaunchOptions { shell, binary: BIN.into(), state_dir: w.state.clone(), sandbox };

    let status = launch_with(&options, |command| {
        command.stdin(File::open(&input).unwrap()).current_dir(&w.cwd);
        command.stdout(File::create(&out).unwrap()).stderr(File::create(&err).unwrap());
        command.env("S", &w.cwd).env("H", &w.home).env("HOME", &w.home);
        command.env("P", std::process::id().to_string());
        command.env("SHELLGAME_STATE_DIR", &w.state).env("SHELLGAME_WORKSPACE", &w.ws);
        command.env_remove("HISTFILE");
        // Nothing of the developer's own fish or XDG directories is touched without the sandbox.
        for xdg in ["CONFIG", "DATA", "STATE", "CACHE"] {
            command.env(format!("XDG_{xdg}_HOME"), w.out.join(xdg));
        }
    });

    // `mktemp` made its file in /tmp (the launcher sets TMPDIR): remove it before anything can fail.
    let temp = scratch(w, "mktemp.out");
    let removed = temp.starts_with("/tmp/") && fs::remove_file(temp.trim()).is_ok();
    assert!(status.unwrap().success(), "{shell}: the shell failed");
    let stdout = fs::read_to_string(out).unwrap();
    let results: Vec<_> = stdout.lines().filter_map(|line| line.strip_prefix('@')).collect();
    assert_eq!(results.len(), steps.len(), "{shell} stopped early");
    for ((kind, command), ok) in steps.into_iter().zip(results.iter().map(|code| *code == "0")) {
        let wanted = match kind {
            "attack" => Some(!sandbox),
            "escape" => sandbox.then_some(false),
            "guarded" => Some(false),
            "ignored" => None,
            _ => Some(true),
        };
        assert!(wanted.is_none_or(|wanted| wanted == ok), "{shell} sandbox={sandbox}: {command}");
    }
    assert!(removed, "{shell}: mktemp printed {temp:?}");
    fs::read_to_string(err).unwrap()
}

#[test]
fn the_game_shell_cannot_change_files_outside_tmp() {
    if !landlock() {
        return eprintln!("SKIPPED: Landlock is not available here");
    }
    for shell in shells() {
        let (w, socket) = victims();
        let before = home(&w);

        let stderr = play(&w, shell, true, socket.is_some());

        // The home is untouched, the files in /tmp are fine, and `shellgame submit` could write its
        // state in the state directory.
        assert_eq!(home(&w), before, "{shell}: the home changed");
        let (state_before, state_after) = (scratch(&w, "before.json"), scratch(&w, "after.json"));
        assert!(!state_after.is_empty() && state_after != state_before, "{shell}: state.json");
        let noise = stderr.to_lowercase();
        assert!(!noise.contains("history") && !noise.contains("fish_variables"));
    }
}

#[test]
fn without_the_sandbox_the_same_steps_do_change_the_home() {
    if !landlock() {
        return eprintln!("SKIPPED: Landlock is not available here");
    }
    for shell in shells() {
        let (w, socket) = victims();
        let before = home(&w);

        play(&w, shell, false, socket.is_some());

        let path = |name: &str| w.home.join(name);
        let read = |name: &str| fs::read_to_string(path(name)).unwrap();
        assert_ne!(home(&w), before);
        assert!(path("new-file").exists() && path("moved").exists() && !path("rm-me").exists());
        assert_eq!([read("write-me"), read("truncate-me")], ["x\n", ""]);
        if socket.is_some() {
            assert_eq!(read("shrink-me"), "");
        }
        // The link in /tmp led to a victim.
        assert_eq!(read("trapped"), "x\n");
    }
}

#[test]
fn the_guard_command_refuses_with_status_3() {
    let w = World::new();
    let guard = |path: &str| w.run(&["guard", "--", "755", path]);

    let refused = guard("/etc");

    // It refuses with an explanation that names the path.
    assert_eq!(refused.code, 3);
    assert!(refused.err.contains("/etc"), "{refused:?}");
    assert_eq!([guard("/tmp").code, guard("/tmp/no-such-file-for-the-guard").code], [3, 0]);
}
