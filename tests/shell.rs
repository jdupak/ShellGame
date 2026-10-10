//! Behavior tests for the shell integration, in bash and in fish.
//!
//! The rendered scripts are sourced into a real shell (`bash --noprofile --norc -c`,
//! `fish --no-config -c`) in an isolated environment, with a mock binary as `SHELLGAME_BIN` that
//! records its calls and replays prepared stderr text (see `shell_support/`). Every test that is
//! about shell behavior exists once per shell, as `<name>::bash` and `<name>::fish`.
//!
//! bash is required. Tests that need fish or shellcheck print a note and pass when the tool is
//! missing. Nothing here changes the environment or the working directory of the test process:
//! shells run with `Command::env` and `Command::current_dir`.
//!
//! To check another shell version, put it first in `PATH`, for example
//! `PATH=/opt/fish-3.7/bin:$PATH cargo test --test shell`.

mod shell_support;

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::ErrorKind;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use shell_support::{
    Harness, directive, directive_bytes, find_named, fish_available, post, pre,
    shellcheck_available, skip, text,
};
use shellgame::shell::launch::{LaunchOptions, launch_with};
use shellgame::shell::{ScriptOptions, ShellKind, render_bash, render_fish, write_fd_launcher};

/// Defines `<name>::bash` and `<name>::fish` from one body that receives the shell as `$sh`.
/// The fish test passes (with a note) when fish is not installed.
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
                if !fish_available() {
                    skip("fish is not installed");
                    return;
                }
                $body
            }
        }
    };
}

/// A path as the shells print it: followed by a newline.
fn line(path: &Path) -> String {
    format!("{}\n", path.display())
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("test paths are UTF-8")
}

// ---------------------------------------------------------------------------------------------
// The `shellgame` function: running the binary and replaying its stderr
// ---------------------------------------------------------------------------------------------

shell_test!(directives_run_and_other_stderr_passes_through, |sh| {
    let mut h = Harness::new();
    h.mock_stdout("game output\n");
    h.mock_stderr(text(&[
        &directive("echo", &["PROTOCOL_WORKED"]),
        "normal stderr",
    ]));

    let run = h.run(sh, "shellgame");

    assert_eq!(run.code, Some(0));
    // The binary's stdout is not captured; the echo directive runs after the binary exited.
    assert_eq!(run.stdout, "game output\nPROTOCOL_WORKED\n");
    assert_eq!(run.stderr, "normal stderr\n");
    assert!(!run.stderr.contains("__SHELLGAME_EXEC__"));
});

shell_test!(wire_format_v1_is_executed_as_documented, |sh| {
    // Hand-written lines, independent of the Rust encoder: this format is a contract.
    //   "Hello World" = SGVsbG8gV29ybGQ=, "SHELLGAME_LEVEL" = U0hFTExHQU1FX0xFVkVM,
    //   "1.4" = MS40, "/" = Lw==
    let mut h = Harness::new();
    h.mock_stderr(text(&[
        "__SHELLGAME_EXEC__v1 echo SGVsbG8gV29ybGQ=",
        "__SHELLGAME_EXEC__v1 export U0hFTExHQU1FX0xFVkVM MS40",
        "__SHELLGAME_EXEC__v1 cd Lw==",
        "__SHELLGAME_EXEC__v1 pwd",
    ]));

    let run = h.run(sh, "shellgame\necho \"level:$SHELLGAME_LEVEL\"");

    assert_eq!(run.stdout, "Hello World\n/\nlevel:1.4\n");
    assert_eq!(run.stderr, "");
});

shell_test!(
    cd_directive_moves_to_paths_with_spaces_and_metacharacters,
    |sh| {
        let mut h = Harness::new();
        let target = h.mkdir("dir with spaces/and; $(touch PWNED) `touch PWNED` 'q' \"dq\"");
        h.mock_stderr(text(&[&directive("cd", &[path_str(&target)])]));

        let run = h.run(sh, "shellgame\npwd");

        assert_eq!(run.stdout, line(&target));
        assert_eq!(run.stderr, "");
        assert!(
            find_named(&h.base, "PWNED").is_empty(),
            "nothing was executed"
        );
    }
);

shell_test!(cd_directive_handles_paths_that_are_not_utf8, |sh| {
    let mut h = Harness::new();
    let weird = h.base.join(OsStr::from_bytes(b"bad-\xff\xfe-name"));
    if fs::create_dir(&weird).is_err() {
        skip("this file system does not allow names that are not UTF-8");
        return;
    }
    fs::write(weird.join("marker"), "").unwrap();
    h.mock_stderr(text(&[&directive_bytes(
        "cd",
        &[weird.as_os_str().as_bytes()],
    )]));

    // The marker file proves where the shell is without printing the unprintable name.
    let run = h.run(sh, "shellgame\ntest -e marker && echo here");

    assert_eq!(run.stdout, "here\n");
    assert_eq!(run.stderr, "");
});

shell_test!(export_directive_sets_and_exports_variables, |sh| {
    let mut h = Harness::new();
    h.mock_stderr(text(&[
        &directive(
            "export",
            &[
                "FOO",
                "bar baz; touch PWNED $(touch PWNED) `touch PWNED` \"q\" 'q'",
            ],
        ),
        &directive("export", &["EMPTY", ""]),
        &directive("export", &["SHELLGAME_LEVEL", "1.4"]),
    ]));

    // `env` shows what a child process would inherit, so this also proves the variables are exported.
    let run = h.run(
        sh,
        "shellgame\necho \"foo:$FOO\"\necho \"empty:[$EMPTY]\"\nenv | grep -E '^(FOO|EMPTY|SHELLGAME_LEVEL)=' | sort",
    );

    assert_eq!(
        run.stdout,
        "foo:bar baz; touch PWNED $(touch PWNED) `touch PWNED` \"q\" 'q'\n\
         empty:[]\n\
         EMPTY=\n\
         FOO=bar baz; touch PWNED $(touch PWNED) `touch PWNED` \"q\" 'q'\n\
         SHELLGAME_LEVEL=1.4\n"
    );
    assert_eq!(run.stderr, "");
    assert!(find_named(&h.base, "PWNED").is_empty());
});

shell_test!(export_directive_rejects_invalid_names, |sh| {
    let mut h = Harness::new();
    h.mock_stderr(text(&[
        &directive("export", &["BAD-NAME", "x"]),
        &directive("export", &["1ABC", "x"]),
        &directive("export", &["A B", "x"]),
        &directive("export", &["X; touch PWNED", "x"]),
        &directive("export", &["X$(touch PWNED)", "x"]),
        &directive("export", &["", "x"]),
        &directive("echo", &["still alive"]),
    ]));

    let run = h.run(sh, "shellgame");

    assert_eq!(run.stdout, "still alive\n");
    assert_eq!(run.stderr, "", "a rejected directive is dropped silently");
    assert!(find_named(&h.base, "PWNED").is_empty());
});

shell_test!(
    pwd_directive_prints_the_directory_without_recording_usage,
    |sh| {
        let mut h = Harness::new();
        let workspace = h.mkdir("workspace");
        h.set_env("SHELLGAME_WORKSPACE", &workspace);
        h.mock_stderr(text(&[&directive("pwd", &[])]));

        let run = h.run(sh, "shellgame");

        assert_eq!(run.stdout, line(&h.work));
        assert!(
            !workspace.join(".pwd_used").exists(),
            "only the player's own `pwd` counts as evidence"
        );
    }
);

shell_test!(
    exit_directive_closes_the_shell_and_leaves_no_temp_files,
    |sh| {
        let mut h = Harness::new();
        h.mock_stderr(text(&[
            &directive("echo", &["before"]),
            &directive("exit", &[]),
            &directive("echo", &["after"]),
        ]));

        let run = h.run(sh, "shellgame\necho SHOULD_NOT_RUN");

        assert_eq!(run.code, Some(0));
        assert_eq!(
            run.stdout, "before\n",
            "directives after `exit` are not run"
        );
        assert_eq!(h.tmp_entries(), Vec::<String>::new());
    }
);

shell_test!(marker_inside_a_line_is_printed_not_executed, |sh| {
    let echo = directive("echo", &["SHOULD_NOT_RUN"]);
    let middle = format!("text before {echo}");
    let indented = format!(" {echo}");
    let expected = text(&[&middle, &indented, "tail"]);
    let mut h = Harness::new();
    h.mock_stderr(&expected);

    let run = h.run(sh, "shellgame");

    assert_eq!(run.stdout, "");
    assert_eq!(run.stderr, expected);
});

shell_test!(carriage_returns_are_stripped_once, |sh| {
    let mut h = Harness::new();
    h.mock_stderr(format!(
        "{}\r\nplain\r\ndouble\r\r\n",
        directive("echo", &["X"])
    ));

    let run = h.run(sh, "shellgame");

    assert_eq!(run.stdout, "X\n", "a CRLF-terminated directive still runs");
    assert_eq!(run.stderr, "plain\ndouble\r\n");
});

shell_test!(plain_stderr_is_replayed_byte_for_byte, |sh| {
    let lines = [
        "  indented  ",
        "\ttab\t",
        "back\\slash \\n \\",
        "-n",
        "-e",
        "--",
        "%s %d %%",
        "$HOME `id` $(id) * ? [a-z] ~",
        "",
        "\u{1b}[31mred\u{1b}[0m",
        "Příliš žluťoučký kůň úpěl ďábelské ódy",
    ];
    let mut h = Harness::new();
    // The last line has no newline; the replay terminates it.
    h.mock_stderr(format!("{}no newline at the end", text(&lines)));

    let run = h.run(sh, "shellgame");

    assert_eq!(run.stdout, "");
    assert_eq!(
        run.stderr,
        format!("{}no newline at the end\n", text(&lines))
    );
});

shell_test!(consecutive_directives_all_run_in_order, |sh| {
    let mut h = Harness::new();
    let target = h.mkdir("target");
    h.mock_stderr(text(&[
        &directive("echo", &["A"]),
        &directive("export", &["X", "1"]),
        &directive("echo", &["B"]),
        &directive("cd", &[path_str(&target)]),
        &directive("echo", &["C"]),
        &directive("pwd", &[]),
    ]));

    let run = h.run(sh, "shellgame\necho \"x:$X\"");

    assert_eq!(run.stdout, format!("A\nB\nC\n{}x:1\n", line(&target)));
});

shell_test!(
    malformed_directives_are_ignored_and_the_replay_goes_on,
    |sh| {
        let mut h = Harness::new();
        h.mock_stderr(text(&[
            "__SHELLGAME_EXEC__v2 echo SGk=",
            "__SHELLGAME_EXEC__v1 eval dG91Y2ggUFdORUQ=",
            "__SHELLGAME_EXEC__v1 echo",
            "__SHELLGAME_EXEC__v1 echo SGk= SGk=",
            "__SHELLGAME_EXEC__v1 cd",
            "__SHELLGAME_EXEC__v1 cd SGk= SGk=",
            "__SHELLGAME_EXEC__v1 pwd SGk=",
            "__SHELLGAME_EXEC__v1 exit SGk=",
            "__SHELLGAME_EXEC__v1 export Rk9P",
            "__SHELLGAME_EXEC__v1  echo SGk=",
            "__SHELLGAME_EXEC__v1",
            "__SHELLGAME_EXEC__",
            &directive("echo", &["OK"]),
        ]));

        let run = h.run(sh, "shellgame\npwd");

        assert_eq!(run.stdout, format!("OK\n{}", line(&h.work)));
        assert_eq!(run.stderr, "", "directive lines never leak, valid or not");
        assert!(find_named(&h.base, "PWNED").is_empty());
    }
);

shell_test!(
    binary_failure_keeps_its_status_and_directives_still_run,
    |sh| {
        let mut h = Harness::new();
        h.set_env("MOCK_EXIT", "42");
        h.mock_stderr(text(&[
            &directive("echo", &["still replayed"]),
            &directive("cd", &["/definitely/not/a/directory"]),
            "plain",
        ]));

        let run = h.run(sh, "shellgame\necho \"exit:$STATUS\"");

        assert_eq!(
            run.tagged("exit"),
            ["42"],
            "a failing directive must not mask the status"
        );
        assert!(run.stdout.contains("still replayed\n"));
        assert!(run.stderr.contains("plain\n"));
    }
);

shell_test!(exit_codes_propagate, |sh| {
    for code in ["0", "1", "3", "42", "255"] {
        let mut h = Harness::new();
        h.set_env("MOCK_EXIT", code);

        let run = h.run(sh, "shellgame\necho \"exit:$STATUS\"");

        assert_eq!(run.tagged("exit"), [code], "exit code {code}");
    }
});

shell_test!(injection_attempts_arrive_as_single_argv_items, |sh| {
    let h = Harness::new();

    h.run(
        sh,
        "shellgame '; touch PWNED' '$(touch PWNED2)' '`touch PWNED3`' 'a b' '' -- -x '*' '~' '\"q\"'",
    );

    let calls = h.game_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].argv,
        [
            "; touch PWNED",
            "$(touch PWNED2)",
            "`touch PWNED3`",
            "a b",
            "",
            "--",
            "-x",
            "*",
            "~",
            "\"q\"",
        ]
    );
    for marker in ["PWNED", "PWNED2", "PWNED3"] {
        assert!(
            find_named(&h.base, marker).is_empty(),
            "{marker} must not exist"
        );
    }
});

shell_test!(binary_script_and_state_paths_may_contain_spaces, |sh| {
    let mut h = Harness::new();
    let dir = h.mkdir("my programs/shell game");
    let bin = dir.join("shell game");
    fs::copy(&h.bin, &bin).unwrap();
    let script_dir = h.mkdir("my scripts");
    let script = script_dir.join("init script");
    let options = ScriptOptions::default();
    fs::write(
        &script,
        match sh {
            ShellKind::Bash => render_bash(&options),
            ShellKind::Fish => render_fish(&options),
        },
    )
    .unwrap();
    let launcher = write_fd_launcher(&h.base.join("state dir")).unwrap();
    h.set_env("SHELLGAME_BIN", &bin)
        .set_env("SHELLGAME_INIT", &script)
        .set_env("SHELLGAME_FD_HOOK", &launcher);

    let run = h.run(
        sh,
        "shellgame 'first arg' second\nsh -c '\"$SHELLGAME_FD_HOOK\" from-script >/dev/null 2>&1'\ncd .\n",
    );

    assert_eq!(run.stderr, "");
    let calls = h.calls();
    let argvs: Vec<_> = calls.iter().map(|call| call.argv.clone()).collect();
    assert_eq!(argvs[0], ["first arg", "second"]);
    assert_eq!(argvs[1], ["fd-hook", "from-script"]);
    assert_eq!(argvs[2], pre(&h.work, "", &["."]));
    assert_eq!(calls[0].env["SHELLGAME_BIN"], path_str(&bin));
});

shell_test!(stdin_stays_on_the_terminal_and_reaches_the_binary, |sh| {
    let mut h = Harness::new();
    h.set_env("MOCK_READ_STDIN", "1");

    h.run(sh, "printf 'from-pipe' | shellgame submit");

    let calls = h.game_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].stdin.as_deref(), Some("from-pipe"));
});

shell_test!(no_temp_files_are_left_behind, |sh| {
    let mut h = Harness::new();
    h.mock_stderr(text(&[&directive("echo", &["x"]), "plain"]));
    h.set_env("MOCK_EXIT", "3");

    h.run(sh, "shellgame\nshellgame again");

    assert_eq!(h.tmp_entries(), Vec::<String>::new());
});

shell_test!(
    unusable_temp_directory_fails_without_running_the_binary,
    |sh| {
        let mut h = Harness::new();
        let missing = h.base.join("no-such-tmp");
        h.set_env("TMPDIR", &missing);

        let run = h.run(sh, "shellgame\necho \"exit:$STATUS\"");

        assert_eq!(run.tagged("exit"), ["1"]);
        assert!(
            h.game_calls().is_empty(),
            "the binary must not run without its stderr capture"
        );
    }
);

shell_test!(missing_binary_is_reported_as_command_not_found, |sh| {
    let mut h = Harness::new();
    h.set_env("SHELLGAME_BIN", h.base.join("missing/shellgame"));

    let run = h.run(sh, "shellgame\necho \"exit:$STATUS\"");

    assert_eq!(run.tagged("exit"), ["127"]);
    assert!(
        !run.stderr.is_empty(),
        "the player should see why nothing happened"
    );
    assert_eq!(h.tmp_entries(), Vec::<String>::new());
});

shell_test!(long_output_is_replayed_complete_and_in_order, |sh| {
    let mut lines = Vec::new();
    for number in 0..400 {
        lines.push(format!("line {number}"));
        if number % 100 == 0 {
            let message = format!("echo {number}");
            lines.push(directive("echo", &[message.as_str()]));
        }
    }
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    let mut h = Harness::new();
    h.mock_stderr(text(&refs));

    let run = h.run(sh, "shellgame");

    let plain: Vec<String> = (0..400).map(|number| format!("line {number}")).collect();
    assert_eq!(run.stderr.lines().collect::<Vec<_>>(), plain);
    assert_eq!(run.stdout, "echo 0\necho 100\necho 200\necho 300\n");
});

// ---------------------------------------------------------------------------------------------
// Deleted working directory and `pwd`
// ---------------------------------------------------------------------------------------------

shell_test!(
    deleted_cwd_relocates_to_the_workspace_before_the_binary_runs,
    |sh| {
        let mut h = Harness::new();
        let workspace = h.mkdir("workspace");
        let doomed = h.mkdir("workspace/doomed");
        h.set_env("SHELLGAME_WORKSPACE", &workspace)
            .set_env("DOOMED", &doomed);

        let run = h.run(
            sh,
            "cd \"$DOOMED\"\nrm -rf \"$DOOMED\"\nshellgame check\necho \"exit:$STATUS\"\npwd",
        );

        assert_eq!(run.tagged("exit"), ["0"]);
        let game_calls = h.game_calls();
        assert_eq!(game_calls.len(), 1);
        assert_eq!(game_calls[0].argv, ["check"]);
        assert_eq!(
            game_calls[0].cwd, workspace,
            "the binary must start in the workspace"
        );
        assert!(run.stdout.ends_with(&line(&workspace)), "{:?}", run.stdout);
        // Relocating is not a player move: it never goes through the cd hook.
        assert_eq!(
            h.hook_argvs().len(),
            2,
            "only the player's own `cd` was reported"
        );
    }
);

shell_test!(
    deleted_cwd_falls_back_to_tmp_without_a_usable_workspace,
    |sh| {
        let tmp = Path::new("/tmp").canonicalize().unwrap();
        // No workspace, an empty one, and one that points nowhere: all are no better than none.
        for workspace in [None, Some(""), Some("missing")] {
            let mut h = Harness::new();
            let doomed = h.mkdir("doomed");
            h.set_env("DOOMED", &doomed);
            if let Some(workspace) = workspace {
                let value = if workspace.is_empty() {
                    h.base.clone()
                } else {
                    h.base.join(workspace)
                };
                h.set_env(
                    "SHELLGAME_WORKSPACE",
                    if workspace.is_empty() {
                        PathBuf::new()
                    } else {
                        value
                    },
                );
            }

            h.run(sh, "cd \"$DOOMED\"\nrm -rf \"$DOOMED\"\nshellgame check");

            let calls = h.game_calls();
            assert_eq!(calls.len(), 1, "workspace {workspace:?}");
            assert_eq!(calls[0].cwd, tmp, "workspace {workspace:?}");
        }
    }
);

shell_test!(pwd_records_usage_only_in_an_existing_workspace, |sh| {
    // The workspace exists: the marker appears and `pwd` still prints the directory.
    let mut h = Harness::new();
    let workspace = h.mkdir("workspace");
    h.set_env("SHELLGAME_WORKSPACE", &workspace);
    let run = h.run(sh, "pwd\necho \"exit:$STATUS\"");
    assert_eq!(run.stdout, format!("{}exit:0\n", line(&h.work)));
    assert!(workspace.join(".pwd_used").is_file());

    // The variable points at a missing directory, is empty or is not set at all: nothing is
    // created anywhere, and `pwd` is unaffected. The workspace is never guessed.
    for workspace in [Some("missing/workspace"), Some(""), None] {
        let mut h = Harness::new();
        if let Some(workspace) = workspace {
            let value = if workspace.is_empty() {
                PathBuf::new()
            } else {
                h.base.join(workspace)
            };
            h.set_env("SHELLGAME_WORKSPACE", value);
        }

        let run = h.run(sh, "pwd\necho \"exit:$STATUS\"");

        assert_eq!(
            run.stdout,
            format!("{}exit:0\n", line(&h.work)),
            "{workspace:?}"
        );
        assert_eq!(run.stderr, "");
        assert_eq!(
            find_named(&h.base, ".pwd_used"),
            Vec::<PathBuf>::new(),
            "{workspace:?}"
        );
        assert!(!h.base.join("missing").exists());
    }
});

shell_test!(pwd_prints_the_logical_path_and_accepts_options, |sh| {
    let h = Harness::new();
    h.mkdir("work/real");
    symlink("real", h.work.join("link")).unwrap();

    let run = h.run(sh, "cd link\npwd\npwd -P\npwd -L");

    assert_eq!(
        run.stdout,
        format!(
            "{}{}{}",
            line(&h.work.join("link")),
            line(&h.work.join("real")),
            line(&h.work.join("link"))
        ),
        "plain `pwd` is logical in both shells, `-P` is physical"
    );
});

// ---------------------------------------------------------------------------------------------
// The cd hook
// ---------------------------------------------------------------------------------------------

shell_test!(cd_hook_reports_pre_and_post_with_every_argument, |sh| {
    let h = Harness::new();
    let pre_dir = h.mkdir("work/pre");
    let a = h.mkdir("work/a");
    let spaced = h.mkdir("work/dir with spaces");
    let b = h.mkdir("work/b");

    let run = h.run(
        sh,
        "cd pre\ncd -- ../a\ncd \"../dir with spaces\"\ncd ../b\ncd\npwd",
    );

    assert_eq!(run.stderr, "");
    assert_eq!(run.stdout, line(&h.home), "a bare `cd` goes home");
    assert_eq!(
        h.hook_argvs(),
        vec![
            // `<prev>` is empty before the first move and the directory left behind after it.
            // A directory that is literally named `pre` is just an argument after the `--`.
            pre(&h.work, "", &["pre"]),
            post(&h.work, &pre_dir, &["pre"]),
            // All arguments follow, verbatim, including a `--` of the player's own.
            pre(&pre_dir, path_str(&h.work), &["--", "../a"]),
            post(&pre_dir, &a, &["--", "../a"]),
            pre(&a, path_str(&pre_dir), &["../dir with spaces"]),
            post(&a, &spaced, &["../dir with spaces"]),
            pre(&spaced, path_str(&a), &["../b"]),
            post(&spaced, &b, &["../b"]),
            // No arguments at all: the hook line ends at its `--`.
            pre(&b, path_str(&spaced), &[]),
            post(&b, &h.home, &[]),
        ]
    );
});

shell_test!(cd_hook_blocks_on_exit_status_3, |sh| {
    let mut h = Harness::new();
    h.mkdir("work/a");
    h.set_env("MOCK_PRE_EXIT", "3")
        .set_env("MOCK_HOOK_STDERR", "BLOCKED: stay where you are");

    let run = h.run(sh, "cd a\necho \"exit:$STATUS\"\npwd");

    assert_eq!(run.tagged("exit"), ["1"]);
    assert!(
        run.stdout.ends_with(&line(&h.work)),
        "the directory must be unchanged"
    );
    assert!(
        run.stderr.contains("BLOCKED: stay where you are"),
        "the hook's explanation reaches the player: {:?}",
        run.stderr
    );
    assert_eq!(
        h.hook_argvs(),
        vec![pre(&h.work, "", &["a"])],
        "a blocked move is not reported as a move"
    );
});

shell_test!(cd_hook_fails_open_on_any_other_status, |sh| {
    for status in ["1", "2", "4", "127", "255"] {
        let mut h = Harness::new();
        let a = h.mkdir("work/a");
        h.set_env("MOCK_PRE_EXIT", status);

        let run = h.run(sh, "cd a\necho \"exit:$STATUS\"\npwd");

        assert_eq!(run.tagged("exit"), ["0"], "hook status {status}");
        assert!(run.stdout.ends_with(&line(&a)), "hook status {status}");
        assert_eq!(
            h.hook_argvs(),
            vec![pre(&h.work, "", &["a"]), post(&h.work, &a, &["a"])],
            "hook status {status}"
        );
    }
});

shell_test!(cd_hook_fails_open_when_the_binary_is_unusable, |sh| {
    // A path that does not exist, an empty value and no variable at all.
    let cases: [Option<&str>; 3] = [Some("/nonexistent-shellgame-dir/shellgame"), Some(""), None];
    for bin in cases {
        let mut h = Harness::new();
        let a = h.mkdir("work/a");
        match bin {
            Some(path) => h.set_env("SHELLGAME_BIN", path),
            None => h.unset_env("SHELLGAME_BIN"),
        };

        let run = h.run(sh, "cd a\necho \"exit:$STATUS\"\npwd");

        assert_eq!(run.tagged("exit"), ["0"], "SHELLGAME_BIN = {bin:?}");
        assert!(run.stdout.ends_with(&line(&a)), "SHELLGAME_BIN = {bin:?}");
    }
});

shell_test!(cd_hook_post_output_is_discarded, |sh| {
    let mut h = Harness::new();
    h.mkdir("work/a");
    h.set_env("MOCK_POST_NOISE", "POST-NOISE")
        .set_env("MOCK_POST_EXIT", "7");

    let run = h.run(sh, "cd a\necho \"exit:$STATUS\"");

    assert_eq!(run.tagged("exit"), ["0"], "the status of `post` is ignored");
    assert!(!run.stdout.contains("POST-NOISE"), "{:?}", run.stdout);
    assert!(!run.stderr.contains("POST-NOISE"), "{:?}", run.stderr);
});

shell_test!(failed_cd_is_not_reported_as_a_move, |sh| {
    let h = Harness::new();

    let run = h.run(sh, "cd nonexistent\necho \"exit:$STATUS\"\npwd");

    assert_eq!(run.tagged("exit"), ["1"]);
    assert!(run.stdout.ends_with(&line(&h.work)));
    assert_eq!(h.hook_argvs(), vec![pre(&h.work, "", &["nonexistent"])]);
});

shell_test!(
    cd_dash_returns_to_the_previous_directory_and_toggles,
    |sh| {
        let h = Harness::new();
        let a = h.mkdir("work/a");

        let run = h.run(
            sh,
            "cd a\ncd -\necho \"one:$PWD\"\ncd -\necho \"two:$PWD\"\ncd -\necho \"three:$PWD\"",
        );

        assert_eq!(run.tagged("one"), [path_str(&h.work)]);
        assert_eq!(run.tagged("two"), [path_str(&a)]);
        assert_eq!(run.tagged("three"), [path_str(&h.work)]);
        // `<prev>` is where `cd -` goes, also when the shell toggles back and forth.
        assert_eq!(
            h.hook_argvs(),
            vec![
                pre(&h.work, "", &["a"]),
                post(&h.work, &a, &["a"]),
                pre(&a, path_str(&h.work), &["-"]),
                post(&a, &h.work, &["-"]),
                pre(&h.work, path_str(&a), &["-"]),
                post(&h.work, &a, &["-"]),
                pre(&a, path_str(&h.work), &["-"]),
                post(&a, &h.work, &["-"]),
            ]
        );
    }
);

shell_test!(cd_dash_without_history_fails_cleanly, |sh| {
    let h = Harness::new();

    let run = h.run(sh, "cd -\necho \"exit:$STATUS\"\npwd");

    assert_eq!(run.tagged("exit"), ["1"]);
    assert!(run.stdout.ends_with(&line(&h.work)));
    assert_eq!(
        h.hook_argvs(),
        vec![pre(&h.work, "", &["-"])],
        "an empty `<prev>`, no move"
    );
    for noise in ["dirprev", "dirnext", "index", "syntax", "Unknown command"] {
        assert!(!run.stderr.contains(noise), "{noise}: {:?}", run.stderr);
    }
});

shell_test!(teleport_never_consults_the_cd_hook, |sh| {
    let mut h = Harness::new();
    let target = h.mkdir("target");
    // A hook that blocks every move of the player must not be able to stop a teleport.
    h.set_env("MOCK_PRE_EXIT", "3");
    h.mock_stderr(text(&[&directive("cd", &[path_str(&target)])]));

    let run = h.run(sh, "shellgame\npwd\ncd ..\necho \"exit:$STATUS\"\npwd");

    assert_eq!(
        run.stdout,
        format!("{}exit:1\n{}", line(&target), line(&target))
    );
    assert_eq!(
        h.hook_argvs(),
        vec![pre(&target, path_str(&h.work), &[".."])],
        "only the player's own `cd` asked the hook"
    );
});

shell_test!(teleport_keeps_cd_dash_working, |sh| {
    let mut h = Harness::new();
    let target = h.mkdir("target");
    h.mock_stderr(text(&[&directive("cd", &[path_str(&target)])]));

    let run = h.run(sh, "shellgame\ncd -\necho \"here:$PWD\"");

    assert_eq!(run.tagged("here"), [path_str(&h.work)]);
    assert_eq!(
        h.hook_argvs(),
        vec![
            pre(&target, path_str(&h.work), &["-"]),
            post(&target, &h.work, &["-"])
        ]
    );
});

// ---------------------------------------------------------------------------------------------
// Completions, autostart, developer shortcuts, child processes, the fd hook
// ---------------------------------------------------------------------------------------------

shell_test!(completions_offer_commands_and_options, |sh| {
    let h = Harness::new();
    fs::write(h.work.join("answer.txt"), "x").unwrap();

    // (what the player has typed, what must be offered)
    let cases: [(&str, &[&str]); 7] = [
        (
            "shellgame ",
            &[
                "init", "hint", "submit", "skip", "status", "reset", "repeat", "show", "remove",
                "exit", "update",
            ],
        ),
        ("shellgame hint -", &["--repeat", "-r"]),
        ("shellgame repeat --", &["--section", "--level"]),
        ("shellgame show --", &["--section", "--level"]),
        (
            "shellgame dev ",
            &["jump", "next", "prev", "reload", "start"],
        ),
        ("shellgame submit ans", &["answer.txt"]),
        ("sg hint -", &["--repeat"]),
    ];

    for (typed, expected) in cases {
        let offered: Vec<String> = match sh {
            ShellKind::Bash => {
                // Complete the way bash does: split the line into words and name the current one.
                let words: Vec<&str> = typed.split(' ').collect();
                let body = format!(
                    "COMP_WORDS=({})\nCOMP_CWORD={}\n_shellgame_completions\nprintf '%s\\n' \"${{COMPREPLY[@]}}\"",
                    words
                        .iter()
                        .map(|word| format!("'{word}'"))
                        .collect::<Vec<_>>()
                        .join(" "),
                    words.len() - 1
                );
                h.run(sh, &body).stdout.lines().map(str::to_owned).collect()
            }
            ShellKind::Fish => {
                let run = h.run(sh, &format!("complete -C '{typed}'"));
                run.stdout
                    .lines()
                    .map(|candidate| candidate.split('\t').next().unwrap_or_default().to_owned())
                    .collect()
            }
        };
        for candidate in expected {
            assert!(
                offered.iter().any(|offer| offer == candidate),
                "`{typed}` should offer {candidate}; got {offered:?}"
            );
        }
    }
});

shell_test!(
    autostart_runs_the_game_once_and_survives_sourcing_again,
    |sh| {
        let mut h = Harness::new();
        h.autostart = true;
        h.mock_stderr(text(&[&directive("echo", &["game started"])]));
        let sub = h.mkdir("work/sub");

        // The harness sources the script once (which autostarts); the body sources it twice more.
        let run = h.run(
        sh,
        "echo \"marker:$SHELLGAME_AUTO_STARTED\"\nsource $SHELLGAME_INIT\nsource $SHELLGAME_INIT\ncd sub\npwd",
    );

        assert_eq!(run.tagged("marker"), ["1"]);
        assert_eq!(
            run.stdout.matches("game started").count(),
            1,
            "{:?}",
            run.stdout
        );
        assert!(
            run.stdout.ends_with(&line(&sub)),
            "`cd` must still work after re-sourcing"
        );
        let calls = h.game_calls();
        assert_eq!(calls.len(), 1, "the game starts exactly once");
        assert!(
            calls[0].argv.is_empty(),
            "autostart runs a bare `shellgame`"
        );
        assert_eq!(calls[0].env["SHELLGAME_AUTO_STARTED"], "1");
    }
);

shell_test!(no_autostart_when_the_game_already_started, |sh| {
    let h = Harness::new();

    h.run(sh, "echo sourced");

    assert!(
        h.calls().is_empty(),
        "SHELLGAME_AUTO_STARTED=1 keeps the script quiet"
    );
});

shell_test!(dev_shortcuts_exist_only_in_devmode, |sh| {
    let shortcuts = [
        ("r", "shellgame dev reload"),
        ("n", "shellgame dev next"),
        ("p", "shellgame dev prev"),
        ("j", "shellgame dev jump"),
        ("s", "shellgame dev start"),
        ("e", "shellgame exit"),
        ("g", "shellgame"),
    ];
    let list = match sh {
        ShellKind::Bash => "alias",
        ShellKind::Fish => "abbr --show",
    };

    let mut h = Harness::new();
    assert_eq!(h.run(sh, list).stdout, "", "no shortcuts without --devmode");

    h.devmode = true;
    let run = h.run(sh, list);
    assert_eq!(run.stderr, "");
    for (name, command) in shortcuts {
        let defined = match sh {
            ShellKind::Bash => run
                .stdout
                .lines()
                .any(|line| line == format!("alias {name}='{command}'")),
            // The printed form varies between fish versions (`abbr -a -g -- r ...` in 3.3, without
            // `-g` later), and the expansion is quoted only when it needs it.
            ShellKind::Fish => run.stdout.lines().any(|line| {
                line.starts_with("abbr -a ")
                    && (line.ends_with(&format!(" -- {name} '{command}'"))
                        || line.ends_with(&format!(" -- {name} {command}")))
            }),
        };
        assert!(
            defined,
            "missing shortcut {name} = {command}: {:?}",
            run.stdout
        );
    }
});

shell_test!(
    fd_hook_is_available_to_scripts_with_descriptors_intact,
    |sh| {
        let h = Harness::new();

        // The launcher is a child process of a child process, as in a level's script.
        h.run(
            sh,
            "sh -c '\"$SHELLGAME_FD_HOOK\" both a >/dev/null 2>&1'\n\
         sh -c '\"$SHELLGAME_FD_HOOK\" err-only \"b c\" 2>/dev/null'\n\
         sh -c '\"$SHELLGAME_FD_HOOK\" none 2>&1'",
        );

        let calls = h.calls();
        let seen: Vec<_> = calls
            .iter()
            .map(|call| (call.argv.clone(), call.fds.clone().unwrap()))
            .collect();
        let null = |out: &str, err: &str| (out.to_owned(), err.to_owned());
        assert_eq!(
            seen,
            vec![
                (
                    vec!["fd-hook".into(), "both".into(), "a".into()],
                    null("null", "null")
                ),
                (
                    vec!["fd-hook".into(), "err-only".into(), "b c".into()],
                    null("other", "null")
                ),
                (
                    vec!["fd-hook".into(), "none".into()],
                    null("other", "other")
                ),
            ]
        );
    }
);

#[test]
fn child_bash_processes_can_call_shellgame() {
    // `export -f` carries the function (and its helpers) into child bash processes; the binary
    // path travels as the exported SHELLGAME_BIN.
    let mut h = Harness::new();
    h.mock_stderr(text(&[&directive("echo", &["child works"]), "plain"]));

    let run = h.run(
        ShellKind::Bash,
        "bash -c 'shellgame child-arg; echo \"child-exit:$?\"'",
    );

    assert_eq!(run.stdout, "child works\nchild-exit:0\n");
    assert_eq!(run.stderr, "plain\n");
    assert_eq!(h.game_calls()[0].argv, ["child-arg"]);
}

#[test]
fn bash_functions_survive_set_e_and_set_u() {
    // Players paste `set -e` from tutorials. The functions must still finish their job: replay the
    // output and clean up when the binary fails, and let a move through when the hook fails.
    let mut h = Harness::new();
    h.set_env("MOCK_EXIT", "1");
    h.mock_stderr(text(&[&directive("echo", &["replayed"]), "plain"]));
    let run = h.run(ShellKind::Bash, "set -e\nshellgame\necho SHOULD_NOT_RUN");
    assert_eq!(run.stdout, "replayed\n");
    assert_eq!(run.stderr, "plain\n");
    assert_eq!(
        run.code,
        Some(1),
        "the failing status ends the shell, as for any command"
    );
    assert_eq!(h.tmp_entries(), Vec::<String>::new());

    let mut h = Harness::new();
    let a = h.mkdir("work/a");
    h.set_env("MOCK_PRE_EXIT", "1");
    let run = h.run(ShellKind::Bash, "set -e\ncd a\npwd");
    assert_eq!(run.stdout, line(&a), "a failing hook allows the move");
    assert_eq!(run.code, Some(0));

    // Malformed directives and an unset OLDPWD are no reason to stop either.
    let mut h = Harness::new();
    let a = h.mkdir("work/a");
    h.mock_stderr(text(&[
        "__SHELLGAME_EXEC__v1 echo",
        "__SHELLGAME_EXEC__v9",
        "plain",
    ]));
    let run = h.run(ShellKind::Bash, "set -eu\nshellgame\ncd a\ncd\ncd -\npwd");
    assert_eq!(run.stderr, "plain\n");
    assert_eq!(run.stdout, format!("{}{}", line(&a), line(&a)));
    assert_eq!(run.code, Some(0));
}

#[test]
fn fd_hook_launcher_is_a_static_script_that_execs_the_binary() {
    let h = Harness::new();
    let launcher = &h.fd_hook;
    assert_eq!(
        fs::read_to_string(launcher).unwrap(),
        "#!/bin/sh\nexec \"$SHELLGAME_BIN\" fd-hook \"$@\"\n"
    );
    assert_eq!(
        fs::metadata(launcher).unwrap().permissions().mode() & 0o777,
        0o700
    );

    // Run it directly, with the descriptors a level script would give it.
    let mut command = Command::new(launcher);
    command
        .args(["x", "y z"])
        .env_clear()
        .env("SHELLGAME_BIN", &h.bin)
        .env("MOCK_LOG", &h.log)
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // A file that was written a moment ago can still be open for writing in a child that another
    // test thread has forked but not yet exec'd; Linux then refuses to execute it. Try again.
    let mut attempts = 0;
    let status = loop {
        match command.status() {
            Err(error) if error.kind() == ErrorKind::ExecutableFileBusy && attempts < 50 => {
                attempts += 1;
                thread::sleep(Duration::from_millis(20));
            }
            result => break result.unwrap(),
        }
    };

    assert!(status.success());
    let calls = h.calls();
    assert_eq!(calls[0].argv, ["fd-hook", "x", "y z"]);
    assert_eq!(calls[0].fds, Some(("null".into(), "null".into())));
}

// ---------------------------------------------------------------------------------------------
// Lint
// ---------------------------------------------------------------------------------------------

#[test]
fn bash_script_passes_shellcheck() {
    if !shellcheck_available() {
        skip("shellcheck is not installed");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    for devmode in [false, true] {
        let script = dir.path().join(format!("integration-{devmode}.bash"));
        fs::write(&script, render_bash(&ScriptOptions { devmode })).unwrap();

        let output = Command::new("shellcheck")
            .args(["-s", "bash", "-S", "error"])
            .arg(&script)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "devmode={devmode}\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn fish_script_has_valid_syntax() {
    if !fish_available() {
        skip("fish is not installed");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    for devmode in [false, true] {
        let script = dir.path().join(format!("integration-{devmode}.fish"));
        fs::write(&script, render_fish(&ScriptOptions { devmode })).unwrap();

        // `fish -n` parses without executing anything.
        let output = Command::new("fish")
            .arg("-n")
            .arg(&script)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "devmode={devmode}\n{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn bash_script_has_valid_syntax() {
    // bash is required, so unlike shellcheck this check always runs.
    let dir = tempfile::tempdir().unwrap();
    for devmode in [false, true] {
        let script = dir.path().join(format!("integration-{devmode}.bash"));
        fs::write(&script, render_bash(&ScriptOptions { devmode })).unwrap();

        let output = Command::new("bash")
            .arg("-n")
            .arg(&script)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

// ---------------------------------------------------------------------------------------------
// The real launch path
// ---------------------------------------------------------------------------------------------

/// Starts a game shell through `launch_with`, as the binary does, but with piped input instead of
/// a terminal: the shell autostarts the game, runs `shellgame smoke-arg` from stdin and exits on
/// end of input.
fn launch_smoke(shell: ShellKind, devmode: bool) {
    let h = Harness::new();
    let state_dir = h.base.join("launch state");
    let stdin = h.base.join("smoke-stdin.txt");
    fs::write(&stdin, "shellgame smoke-arg\n").unwrap();
    let options = LaunchOptions {
        shell,
        binary: h.bin.clone(),
        devmode,
        state_dir: state_dir.clone(),
    };

    let mut init_script = None;
    let status = launch_with(&options, |command| {
        init_script = command
            .get_envs()
            .find(|(name, _)| *name == "SHELLGAME_INIT")
            .and_then(|(_, value)| value.map(PathBuf::from));
        command
            .stdin(File::open(&stdin).unwrap())
            .stdout(File::create(h.base.join("smoke-stdout.txt")).unwrap())
            .stderr(File::create(h.base.join("smoke-stderr.txt")).unwrap())
            .current_dir(&h.work)
            // Keep the player's real configuration and any outer game out of the picture.
            .env("HOME", &h.home)
            .env("XDG_CONFIG_HOME", h.home.join(".config"))
            .env("XDG_DATA_HOME", h.home.join(".local/share"))
            .env("XDG_STATE_HOME", h.home.join(".local/state"))
            .env("TMPDIR", &h.tmp)
            .env("MOCK_LOG", &h.log)
            .env_remove("SHELLGAME_WORKSPACE")
            .env_remove("SHELLGAME_LEVEL")
            .env_remove("SHELLGAME_STATE_DIR")
            .env_remove("BASH_ENV");
    })
    .expect("the game shell starts");

    let stderr = fs::read_to_string(h.base.join("smoke-stderr.txt")).unwrap_or_default();
    assert!(status.success(), "{shell}: {status:?}\n{stderr}");

    // Autostart ran the game once, then the piped command ran it again with its argument.
    let calls = h.game_calls();
    let argvs: Vec<_> = calls.iter().map(|call| call.argv.clone()).collect();
    assert_eq!(
        argvs,
        vec![Vec::<String>::new(), vec!["smoke-arg".to_owned()]],
        "{shell}\n{stderr}"
    );

    // The environment contract, as the binary sees it.
    let init_script = init_script.expect("the launcher passes SHELLGAME_INIT");
    let env = &calls[0].env;
    assert_eq!(env["SHELLGAME_WRAPPER"], "1");
    assert_eq!(env["SHELLGAME_BIN"], path_str(&h.bin));
    assert_eq!(env["SHELLGAME_INIT"], path_str(&init_script));
    assert_eq!(
        env["SHELLGAME_FD_HOOK"],
        path_str(&state_dir.join("fd-hook.sh"))
    );
    assert_eq!(env["SHELLGAME_AUTO_STARTED"], "1");
    assert_eq!(env["BASH_SILENCE_DEPRECATION_WARNING"], "1");

    // The temporary script is gone, the launcher stays, and nothing leaked into TMPDIR.
    assert!(
        !init_script.exists(),
        "{shell}: {init_script:?} was not removed"
    );
    let launcher = state_dir.join("fd-hook.sh");
    assert_eq!(
        fs::metadata(&launcher).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(h.tmp_entries(), Vec::<String>::new());
}

#[test]
fn launch_starts_bash_with_the_integration() {
    launch_smoke(ShellKind::Bash, false);
}

#[test]
fn launch_starts_bash_in_devmode() {
    launch_smoke(ShellKind::Bash, true);
}

#[test]
fn launch_starts_fish_with_the_integration() {
    if !fish_available() {
        skip("fish is not installed");
        return;
    }
    launch_smoke(ShellKind::Fish, false);
}

#[test]
fn launch_starts_fish_in_devmode() {
    if !fish_available() {
        skip("fish is not installed");
        return;
    }
    launch_smoke(ShellKind::Fish, true);
}
