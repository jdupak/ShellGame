//! Support for `tests/shell.rs`: an isolated sandbox, a mock game binary and shell runners.
//!
//! The shell scripts are exercised the way the game uses them, but without the real game: the
//! tests source the rendered script in a non-interactive shell and point `SHELLGAME_BIN` at
//! `mock_shellgame.sh`, which records every call and replays whatever protocol lines and plain
//! stderr text the test prepared.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use shellgame::shell::protocol::encode;
use shellgame::shell::{ScriptOptions, ShellKind, render_bash, render_fish, write_fd_launcher};
use tempfile::TempDir;

/// Source of the mock binary. It is copied into the sandbox and made executable at test time, so
/// the tests do not depend on file modes surviving a checkout.
const MOCK_SCRIPT: &str = include_str!("mock_shellgame.sh");

/// Separates the fields of the mock's log records.
const FIELD_SEPARATOR: char = '\u{1f}';

/// Runs `tool --version`; true when the tool is installed and starts.
fn tool_works(tool: &str, arg: &str) -> bool {
    Command::new(tool)
        .arg(arg)
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Whether fish is installed. Tests that need it skip with a note when it is not.
pub fn fish_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| tool_works("fish", "--version"))
}

/// Whether shellcheck is installed. Lint tests skip with a note when it is not.
pub fn shellcheck_available() -> bool {
    static AVAILABLE: OnceLock<bool> = OnceLock::new();
    *AVAILABLE.get_or_init(|| tool_works("shellcheck", "--version"))
}

/// Prints why a test is not running; visible with `cargo test -- --nocapture`.
pub fn skip(reason: &str) {
    eprintln!("SKIPPED: {reason}");
}

/// The shell's exit-status variable: `$?` in bash, `$status` in fish.
pub fn status_var(shell: ShellKind) -> &'static str {
    match shell {
        ShellKind::Bash => "$?",
        ShellKind::Fish => "$status",
    }
}

/// One protocol line, built with the crate's encoder.
pub fn directive(verb: &str, args: &[&str]) -> String {
    encode(verb, args)
}

/// One protocol line whose arguments are raw bytes (a path that is not valid UTF-8, for example).
pub fn directive_bytes(verb: &str, args: &[&[u8]]) -> String {
    encode(verb, args)
}

/// Joins lines into stderr text, ending with a newline.
pub fn text(lines: &[&str]) -> String {
    let mut joined = lines.join("\n");
    joined.push('\n');
    joined
}

/// Output of one shell run.
#[derive(Debug)]
pub struct Run {
    pub stdout: String,
    pub stderr: String,
    /// `None` when the shell was killed by a signal.
    pub code: Option<i32>,
}

impl Run {
    /// The values printed as `name:<value>` lines by a test body (`echo "name:$value"`), in order.
    ///
    /// A leading bell character is ignored: fish rings it when `cd -` has no history to go back to.
    pub fn tagged(&self, name: &str) -> Vec<&str> {
        let prefix = format!("{name}:");
        self.stdout
            .lines()
            .filter_map(|line| line.trim_start_matches('\u{7}').strip_prefix(&prefix))
            .collect()
    }
}

/// One call of the mock binary, as recorded in its log.
#[derive(Debug, Default, Clone)]
pub struct Call {
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
    /// `fd-hook` calls: whether stdout and stderr were `/dev/null` (`"null"` or `"other"`).
    pub fds: Option<(String, String)>,
    /// What the call read from stdin, when the test asked the mock to read it.
    pub stdin: Option<String>,
}

impl Call {
    /// Whether this was a `cd-hook` call.
    pub fn is_hook(&self) -> bool {
        self.argv.first().is_some_and(|arg| arg == "cd-hook")
    }
}

/// The argv of a `cd-hook pre` call, as the shell must send it.
pub fn pre(pwd: &Path, prev: &str, cd_args: &[&str]) -> Vec<String> {
    hook("pre", &pwd.to_string_lossy(), prev, cd_args)
}

/// The argv of a `cd-hook post` call, as the shell must send it.
pub fn post(old: &Path, new: &Path, cd_args: &[&str]) -> Vec<String> {
    hook(
        "post",
        &old.to_string_lossy(),
        &new.to_string_lossy(),
        cd_args,
    )
}

fn hook(phase: &str, first: &str, second: &str, cd_args: &[&str]) -> Vec<String> {
    ["cd-hook", phase, first, second, "--"]
        .into_iter()
        .chain(cd_args.iter().copied())
        .map(str::to_owned)
        .collect()
}

/// An isolated sandbox with a mock game binary.
///
/// Layout (all below one temporary directory with a canonical path):
/// `home/`, `tmp/` (the shell's `TMPDIR`, which must stay empty), `work/` (the shell's working
/// directory), `state/` (holds the fd-hook launcher) and `bin/shellgame` (the mock).
pub struct Harness {
    _root: TempDir,
    pub base: PathBuf,
    pub home: PathBuf,
    pub tmp: PathBuf,
    pub work: PathBuf,
    pub bin: PathBuf,
    pub fd_hook: PathBuf,
    pub log: PathBuf,
    /// Render the scripts with the developer shortcuts.
    pub devmode: bool,
    /// Let the scripts autostart (by default `SHELLGAME_AUTO_STARTED=1` is inherited).
    pub autostart: bool,
    /// Environment overrides; `None` removes the variable.
    env: BTreeMap<String, Option<OsString>>,
}

impl Harness {
    pub fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("sg-shell-")
            .tempdir()
            .expect("create sandbox");
        // Canonical, so `$PWD` and `pwd -P` agree even where /tmp is a symlink (macOS).
        let base = root.path().canonicalize().expect("canonical sandbox path");
        let [home, tmp, work, state, bin_dir] =
            ["home", "tmp", "work", "state", "bin"].map(|name| base.join(name));
        for dir in [&home, &tmp, &work, &state, &bin_dir] {
            fs::create_dir_all(dir).expect("create sandbox directory");
        }

        let bin = bin_dir.join("shellgame");
        fs::write(&bin, MOCK_SCRIPT).expect("write mock binary");
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).expect("chmod mock binary");

        let fd_hook = write_fd_launcher(&state).expect("write fd-hook launcher");
        let log = base.join("mock.log");

        Self {
            _root: root,
            base,
            home,
            tmp,
            work,
            bin,
            fd_hook,
            log,
            devmode: false,
            autostart: false,
            env: BTreeMap::new(),
        }
    }

    /// Sets (or overrides) an environment variable of the shell, and so of the mock binary.
    pub fn set_env(&mut self, name: &str, value: impl Into<OsString>) -> &mut Self {
        self.env.insert(name.to_owned(), Some(value.into()));
        self
    }

    /// Removes a variable that the sandbox would otherwise set.
    pub fn unset_env(&mut self, name: &str) -> &mut Self {
        self.env.insert(name.to_owned(), None);
        self
    }

    /// Creates a directory below the sandbox and returns its path.
    pub fn mkdir(&self, relative: &str) -> PathBuf {
        let path = self.base.join(relative);
        fs::create_dir_all(&path).expect("create directory");
        path
    }

    /// Makes the mock write `stderr` verbatim to its stderr on every regular call.
    pub fn mock_stderr(&mut self, stderr: impl AsRef<[u8]>) -> &mut Self {
        let file = self.base.join("mock-stderr.txt");
        fs::write(&file, stderr).expect("write mock stderr");
        self.set_env("MOCK_STDERR_FILE", file)
    }

    /// Makes the mock write `stdout` verbatim to its stdout on every regular call.
    pub fn mock_stdout(&mut self, stdout: impl AsRef<[u8]>) -> &mut Self {
        let file = self.base.join("mock-stdout.txt");
        fs::write(&file, stdout).expect("write mock stdout");
        self.set_env("MOCK_STDOUT_FILE", file)
    }

    /// The path of the rendered integration script for `shell`, written on demand.
    fn init_script(&self, shell: ShellKind) -> PathBuf {
        let options = ScriptOptions {
            devmode: self.devmode,
        };
        let (name, script) = match shell {
            ShellKind::Bash => ("init.bash", render_bash(&options)),
            ShellKind::Fish => ("init.fish", render_fish(&options)),
        };
        let path = self.base.join(name);
        fs::write(&path, script).expect("write integration script");
        path
    }

    /// The environment every shell run gets: the launcher's contract plus a clean, private world.
    fn environment(&self, shell: ShellKind) -> BTreeMap<String, OsString> {
        let mut env: BTreeMap<String, OsString> = BTreeMap::new();
        env.insert("PATH".into(), std::env::var_os("PATH").unwrap_or_default());
        env.insert("HOME".into(), self.home.clone().into());
        env.insert("TMPDIR".into(), self.tmp.clone().into());
        env.insert("MOCK_LOG".into(), self.log.clone().into());
        // The launcher's contract (src/shell/mod.rs). The scripts export SHELLGAME_AUTO_STARTED
        // themselves when they autostart, so by default it is pre-set to keep them quiet.
        env.insert("SHELLGAME_WRAPPER".into(), "1".into());
        env.insert("SHELLGAME_BIN".into(), self.bin.clone().into());
        env.insert("SHELLGAME_INIT".into(), self.init_script(shell).into());
        env.insert("SHELLGAME_FD_HOOK".into(), self.fd_hook.clone().into());
        if !self.autostart {
            env.insert("SHELLGAME_AUTO_STARTED".into(), "1".into());
        }
        for (name, value) in &self.env {
            match value {
                Some(value) => env.insert(name.clone(), value.clone()),
                None => env.remove(name),
            };
        }
        env
    }

    /// Builds the shell command: the script is sourced non-interactively, then `body` runs.
    ///
    /// `bash --noprofile --norc -c 'source "$SHELLGAME_INIT"; ...'` and
    /// `fish --no-config -c 'source $SHELLGAME_INIT; ...'`, in an otherwise empty environment.
    /// In `body`, `$STATUS` stands for the shell's exit-status variable (`$?` or `$status`).
    pub fn command(&self, shell: ShellKind, body: &str) -> Command {
        let body = &body.replace("$STATUS", status_var(shell));
        let mut command = match shell {
            ShellKind::Bash => {
                let mut command = Command::new("bash");
                command.args([
                    "--noprofile",
                    "--norc",
                    "-c",
                    &format!("source \"$SHELLGAME_INIT\"\n{body}"),
                ]);
                command
            }
            ShellKind::Fish => {
                let mut command = Command::new("fish");
                command.args([
                    "--no-config",
                    "-c",
                    &format!("source $SHELLGAME_INIT\n{body}"),
                ]);
                command
            }
        };
        command
            .env_clear()
            .envs(self.environment(shell))
            .current_dir(&self.work);
        command
    }

    /// Runs `body` in the shell with the integration script sourced.
    pub fn run(&self, shell: ShellKind, body: &str) -> Run {
        let output = self.command(shell, body).output().expect("start the shell");
        Run {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            code: output.status.code(),
        }
    }

    /// Every call the mock binary received so far, in order.
    pub fn calls(&self) -> Vec<Call> {
        let Ok(log) = fs::read_to_string(&self.log) else {
            return Vec::new();
        };
        let mut calls: Vec<Call> = Vec::new();
        for line in log.lines() {
            let mut fields = line.split(FIELD_SEPARATOR);
            let kind = fields.next().unwrap_or_default();
            if kind == "argv" {
                calls.push(Call {
                    argv: fields.map(str::to_owned).collect(),
                    ..Call::default()
                });
                continue;
            }
            let call = calls.last_mut().expect("a record without a call");
            match kind {
                "cwd" => call.cwd = PathBuf::from(fields.next().unwrap_or_default()),
                "env" => {
                    call.env = fields
                        .filter_map(|pair| pair.split_once('='))
                        .map(|(name, value)| (name.to_owned(), value.to_owned()))
                        .collect();
                }
                "fds" => {
                    call.fds = Some((
                        fields.next().unwrap_or_default().to_owned(),
                        fields.next().unwrap_or_default().to_owned(),
                    ));
                }
                "stdin" => call.stdin = Some(fields.collect::<Vec<_>>().join("\u{1f}")),
                other => panic!("unknown mock record {other:?}"),
            }
        }
        calls
    }

    /// The `cd-hook` calls (argv only), in order.
    pub fn hook_argvs(&self) -> Vec<Vec<String>> {
        self.calls()
            .into_iter()
            .filter(Call::is_hook)
            .map(|call| call.argv)
            .collect()
    }

    /// The calls that are neither `cd-hook` nor `fd-hook`: what the `shellgame` function ran.
    pub fn game_calls(&self) -> Vec<Call> {
        self.calls()
            .into_iter()
            .filter(|call| !call.is_hook() && call.argv.first().is_none_or(|arg| arg != "fd-hook"))
            .collect()
    }

    /// Whether anything was left in the shell's `TMPDIR`.
    pub fn tmp_entries(&self) -> Vec<String> {
        fs::read_dir(&self.tmp)
            .expect("read TMPDIR")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }
}

/// All files called `name` below `root`.
pub fn find_named(root: &Path, name: &str) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_owned()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_name() == name {
                found.push(path.clone());
            }
            if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                pending.push(path);
            }
        }
    }
    found
}
