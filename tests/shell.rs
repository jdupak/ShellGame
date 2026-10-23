//! The shell integration in real shells with a mock game binary (support/mock_shellgame.sh). Each
//! shell test exists as `<name>::bash` and `<name>::fish`; fish and shellcheck are skipped with a note
//! when they are not installed.

mod support;

use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use shellgame::game::dedent;
use shellgame::shell::protocol::encode;
use shellgame::shell::{LaunchOptions, ShellKind, launch_with};
use support::{Run, World, have};

macro_rules! shell_test {
    ($name:ident, |$sh:ident| $body:block) => {
        mod $name {
            use super::*;

            #[test]
            fn bash() {
                let $sh = ShellKind::Bash;
                $body
            }

            #[test]
            fn fish() {
                let $sh = ShellKind::Fish;
                if !have("fish") {
                    return eprintln!("SKIPPED: fish is not installed");
                }
                $body
            }
        }
    };
}

fn lines(lines: &[&str]) -> String {
    lines.join("\n") + "\n"
}

fn line(path: &Path) -> String {
    format!("{}\n", path.display())
}

/// A world whose game binary is the mock, which logs every call to `mock.log`.
fn mock() -> World {
    let mut w = World::new();
    w.bin = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/mock_shellgame.sh");
    w.set("MOCK_LOG", w.root.join("mock.log"));
    w
}

fn mkdir(w: &World, relative: &str) -> PathBuf {
    let path = w.root.join(relative);
    fs::create_dir_all(&path).unwrap();
    path
}

/// Sources the integration script, then runs `body` (a command, or an indented block of them) in
/// the shell, where `$STATUS` is `$?` or `$status`: what it printed, and the status the shell
/// exited with.
fn play(w: &World, shell: ShellKind, body: &str) -> Run {
    let status = if shell == ShellKind::Bash { "$?" } else { "$status" };
    let run = w.shell(shell, "", &[dedent(body).replace("$STATUS", status)]);
    Run { code: run.code, out: run.steps[0].out.clone(), err: run.steps[0].err.clone() }
}

/// The values the body printed as `name:<value>` lines (fish rings a bell for `cd -` at the start).
fn tagged<'a>(run: &'a Run, name: &str) -> Vec<&'a str> {
    let lines = run.out.lines().map(|line| line.trim_start_matches('\u{7}'));
    lines.filter_map(|line| line.strip_prefix(&format!("{name}:"))).collect()
}

/// The argv of a `cd-hook` call: `a` and `b` are the directories before and after (`pre`: the
/// shell's directory and where `cd -` would go, or empty).
fn hook(phase: &str, a: impl AsRef<Path>, b: impl AsRef<Path>, args: &[&str]) -> Vec<String> {
    let (a, b) = (a.as_ref().to_string_lossy(), b.as_ref().to_string_lossy());
    ["cd-hook", phase, &a, &b, "--"].iter().chain(args).map(|word| word.to_string()).collect()
}

/// The argv of the mock's calls whose kind (`game`, `cd-hook`, `guard`) starts with
/// `kind`; an empty `kind` means all of them (the kind is not part of the argv).
fn argvs(w: &World, kind: &str) -> Vec<Vec<String>> {
    let log = fs::read_to_string(w.root.join("mock.log")).unwrap_or_default();
    let calls = log.lines().filter(|line| line.starts_with(kind));
    calls.map(|call| call.split('\u{1f}').skip(1).map(str::to_owned).collect()).collect()
}

fn tmp_is_empty(w: &World) -> bool {
    fs::read_dir(&w.tmp).unwrap().next().is_none()
}

shell_test!(replay_runs_directives_and_passes_stderr, |sh| {
    let mut h = mock();
    let dir = mkdir(&h, "dir with spaces/and; $(touch PWNED) `touch PWNED` 'q'");
    let foo = "bar baz; touch PWNED $(touch PWNED) `touch PWNED`";
    let plain = "back\\slash $HOME `id` * Příliš žluťoučký";
    let set = |name: &str| encode("export", &[name, "1"]);
    let wire = lines(&[
        // Written by hand, since this format is a contract: HELLO=World.
        "__SHELLGAME_EXEC__v1 export SEVMTE8= V29ybGQ=",
        &encode("export", &["FOO", foo]),
        &encode("export", &["EMPTY", ""]),
        &encode("cd", &[dir.to_str().unwrap()]),
        // Dropped silently: unknown version and verb, wrong argument counts, invalid names.
        "__SHELLGAME_EXEC__v2 export V1JPTkc= MQ==",
        "__SHELLGAME_EXEC__v1 eval dG91Y2ggUFdORUQ=",
        "__SHELLGAME_EXEC__v1 cd",
        "__SHELLGAME_EXEC__v1 exit SGk=",
        &encode("export", &["X; touch PWNED", "x"]),
    ]);
    // A marker inside a line is text, one CR is stripped, the rest is replayed as it is.
    let rest = format!(
        "  indented  \n-n\n{plain}\n\nbefore {}\n{}\r\nplain\r\nend",
        set("INLINE"),
        set("RAN")
    );
    h.set("MOCK_STDOUT", "game output\n").set("MOCK_STDERR", wire + &rest);
    let body = r#"
        shellgame
        echo "vars:$HELLO $FOO [$EMPTY] [$WRONG$INLINE] [$RAN]"
        echo "here:$PWD"
    "#;

    let run = play(&h, sh, body);

    let out = format!("game output\nvars:World {foo} [] [] [1]\nhere:{}\n", dir.display());
    assert_eq!(run.out, out);
    let err = format!("  indented  \n-n\n{plain}\n\nbefore {}\nplain\nend\n", set("INLINE"));
    assert_eq!(run.err, err);
    let clean = [&h.cwd, &dir, &h.home].iter().all(|place| !place.join("PWNED").exists());
    assert!(clean, "nothing but the directives ran");
});

shell_test!(exit_directive_closes_the_shell, |sh| {
    let mut h = mock();
    h.set("MOCK_STDERR", lines(&[&encode("exit", &[] as &[&str])]));
    let body = r#"
        shellgame
        echo SHOULD_NOT_RUN
    "#;

    let run = play(&h, sh, body);

    assert_eq!((run.code, run.out.as_str()), (0, ""));
    assert!(tmp_is_empty(&h), "no temporary file is left");
});

shell_test!(arguments_arrive_verbatim_and_status_too, |sh| {
    let args: Vec<_> =
        "; touch PWNED|$(touch PWNED)|`touch PWNED`|a b||--|-x|*".split('|').collect();
    let quoted: Vec<_> = args.iter().map(|arg| format!("'{arg}'")).collect();
    let body = format!(
        r#"
            shellgame {}
            echo "exit:$STATUS"
        "#,
        quoted.join(" ")
    );
    let mut h = mock();
    h.set("MOCK_EXIT", "3");

    let run = play(&h, sh, &body);

    assert_eq!(tagged(&run, "exit"), ["3"]);
    assert_eq!(argvs(&h, "game"), std::slice::from_ref(&args));
    assert!(!h.cwd.join("PWNED").exists() && tmp_is_empty(&h));
});

shell_test!(pwd_leaves_evidence_in_a_real_workspace, |sh| {
    let body = r#"
        pwd
        echo "exit:$STATUS"
    "#;
    for workspace in ["ws", "missing", "", "unset"] {
        let mut h = mock();
        let ws = mkdir(&h, "ws");
        match workspace {
            "unset" => h.unset("SHELLGAME_WORKSPACE"),
            "" => h.set("SHELLGAME_WORKSPACE", ""),
            _ => h.set("SHELLGAME_WORKSPACE", h.root.join(workspace)),
        };

        let run = play(&h, sh, body);

        assert_eq!(run.out, format!("{}exit:0\n", line(&h.cwd)));
        let stray = [&h.cwd, &h.home, &h.root].iter().any(|dir| dir.join(".pwd_used").exists());
        assert_eq!(ws.join(".pwd_used").exists(), workspace == "ws", "{workspace:?}");
        assert!(!stray, "the workspace is never guessed: {workspace:?}");
    }
});

shell_test!(cd_hook_receives_every_argument, |sh| {
    let h = mock();
    let (pre, a) = (mkdir(&h, "cwd/pre"), mkdir(&h, "cwd/a"));
    let spaced = mkdir(&h, "cwd/dir with spaces");
    let body = r#"
        cd pre
        cd -- ../a
        cd '../dir with spaces'
        cd
        cd nonexistent
        cd -
        cd -
    "#;

    play(&h, sh, body);

    let (work, home) = (&h.cwd, &h.home);
    // `<prev>` is empty before the first move and the directory just left after it. A directory
    // named `pre` is only an argument and the arguments follow verbatim.
    assert_eq!(
        argvs(&h, "cd-hook"),
        [
            hook("pre", work, "", &["pre"]),
            hook("post", work, &pre, &["pre"]),
            hook("pre", &pre, work, &["--", "../a"]),
            hook("post", &pre, &a, &["--", "../a"]),
            hook("pre", &a, &pre, &["../dir with spaces"]),
            hook("post", &a, &spaced, &["../dir with spaces"]),
            hook("pre", &spaced, &a, &[]),
            hook("post", &spaced, home, &[]),
            // A move that fails is not reported. `cd -` goes where `<prev>` said, and back again.
            hook("pre", home, &spaced, &["nonexistent"]),
            hook("pre", home, &spaced, &["-"]),
            hook("post", home, &spaced, &["-"]),
            hook("pre", &spaced, home, &["-"]),
            hook("post", &spaced, home, &["-"]),
        ]
    );
});

shell_test!(cd_hook_blocks_on_3_and_fails_open, |sh| {
    let mut h = mock();
    let target = mkdir(&h, "target");
    mkdir(&h, "cwd/a");
    h.set("MOCK_PRE", "3").set("MOCK_NOTE", "BLOCKED: stay");
    h.set("MOCK_STDERR", encode("cd", &[target.to_str().unwrap()]) + "\n");
    let body = r#"
        cd a
        echo "exit:$STATUS"
        pwd
        shellgame
        cd ..
        pwd
    "#;

    let run = play(&h, sh, body);

    assert_eq!(run.out, format!("exit:1\n{}{}", line(&h.cwd), line(&target)));
    assert!(run.err.contains("BLOCKED: stay"), "the player sees the explanation");
    // A blocked move is not reported; a teleport never asks the hook: the game moves with its own cd.
    let asked = [hook("pre", &h.cwd, "", &["a"]), hook("pre", &target, &h.cwd, &[".."])];
    assert_eq!(argvs(&h, "cd-hook"), asked);

    // Any other status, a missing binary and an empty one: the move happens.
    let body = r#"
        cd a
        echo "exit:$STATUS"
        pwd
    "#;
    for (name, value) in
        [("MOCK_PRE", "1"), ("MOCK_PRE", "255"), ("SHELLGAME_BIN", "/x"), ("SHELLGAME_BIN", "")]
    {
        let mut h = mock();
        let a = mkdir(&h, "cwd/a");
        h.set(name, value);

        let run = play(&h, sh, body);

        assert_eq!(run.out, format!("exit:0\n{}", line(&a)), "{name}={value}");
    }
});

shell_test!(teleport_keeps_cd_dash_working, |sh| {
    let mut h = mock();
    let target = mkdir(&h, "target");
    h.set("MOCK_STDERR", encode("cd", &[target.to_str().unwrap()]) + "\n");
    let body = r#"
        shellgame
        cd -
        echo "here:$PWD"
    "#;

    let run = play(&h, sh, body);

    assert_eq!(tagged(&run, "here"), [h.cwd.to_str().unwrap()]);
});

shell_test!(chmod_runs_only_when_the_guard_allows, |sh| {
    let body = r#"
        chmod 600 'f g'
        echo "exit:$STATUS"
    "#;
    for guard in ["3", "0", "broken"] {
        let mut h = mock();
        let file = h.cwd.join("f g");
        fs::write(&file, "").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
        match guard {
            "broken" => h.set("SHELLGAME_BIN", "/x"),
            status => h.set("MOCK_GUARD", status),
        };

        let run = play(&h, sh, body);

        let mode = fs::metadata(&file).unwrap().permissions().mode() & 0o777;
        let refused = guard == "3";
        assert_eq!(
            (tagged(&run, "exit")[0], mode),
            if refused { ("1", 0o644) } else { ("0", 0o600) }
        );
        if guard != "broken" {
            assert_eq!(argvs(&h, "guard"), [["guard", "--", "600", "f g"]]);
        }
    }
});

#[test]
fn bash_functions_survive_set_e_and_set_u() {
    // Players paste `set -e` from tutorials: the functions must still move, replay and clean up.
    let mut h = mock();
    let a = mkdir(&h, "cwd/a");
    fs::write(h.cwd.join("f"), "").unwrap();
    h.set("MOCK_PRE", "1").set("MOCK_EXIT", "1");
    let replayed = encode("export", &["REPLAYED", "1"]);
    h.set("MOCK_STDERR", lines(&[&replayed, "__SHELLGAME_EXEC__v1 export", "plain"]));

    // The shell ends at `shellgame`, which fails: the trap shows what the function did before.
    let body = r#"
        trap 'echo "replayed:${REPLAYED-}"' EXIT
        set -eu
        cd a
        pwd
        cd -
        chmod 600 f
        shellgame
        echo SHOULD_NOT_RUN
    "#;
    let run = play(&h, ShellKind::Bash, body);

    assert_eq!(run.out, format!("{}{}replayed:1\n", line(&a), line(&h.cwd)));
    assert_eq!((run.err.as_str(), run.code), ("plain\n", 1));
    assert!(tmp_is_empty(&h));
}

shell_test!(completions_offer_the_commands, |sh| {
    let bash = r#"
        COMP_WORDS=(shellgame '')
        COMP_CWORD=1
        _shellgame_completions
        printf '%s\n' "${COMPREPLY[@]}"
    "#;
    let body = match sh {
        ShellKind::Bash => bash,
        ShellKind::Fish => "complete -C 'shellgame '",
    };

    let run = play(&mock(), sh, body);

    assert!(run.out.lines().any(|candidate| candidate.starts_with("resume")), "{run:?}");
});

#[test]
fn the_bash_script_is_shellcheck_clean() {
    if !have("shellcheck") {
        return eprintln!("SKIPPED: shellcheck is not installed");
    }
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/shell/bash.sh");
    let output = Command::new("shellcheck").arg(script).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stdout));
}

shell_test!(launch_starts_the_game_shell, |sh| {
    let h = mock();
    let (state, input) = (h.root.join("state dir"), h.root.join("input"));
    fs::write(&input, "shellgame smoke-arg\n").unwrap();
    let options =
        LaunchOptions { shell: sh, binary: h.bin.clone(), state_dir: state, sandbox: false };
    let (out, err, log) = (h.root.join("stdout"), h.root.join("stderr"), h.root.join("mock.log"));

    let status = launch_with(&options, |command| {
        command.stdin(File::open(&input).unwrap()).current_dir(&h.cwd);
        command.stdout(File::create(&out).unwrap()).stderr(File::create(&err).unwrap());
        command.env("HOME", &h.home).env("MOCK_LOG", &log);
        // An interactive bash writes its history and fish makes its directories: in the test's.
        command.env_remove("HISTFILE");
        for xdg in ["CONFIG", "DATA", "STATE", "CACHE"] {
            command.env(format!("XDG_{xdg}_HOME"), h.out.join(xdg));
        }
    })
    .unwrap();

    let stderr = fs::read_to_string(err).unwrap();
    assert!(status.success(), "{status:?}\n{stderr}");
    // Autostart ran the game; then the piped command ran it again.
    assert_eq!(argvs(&h, ""), [vec![], vec!["smoke-arg"]], "{stderr}");
});
