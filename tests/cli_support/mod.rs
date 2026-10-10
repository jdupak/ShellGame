//! Support for `tests/cli.rs`: an isolated game, the real binary and helpers that read what it did.
//!
//! Every [`Game`] owns one temporary directory (canonical, so paths compare equal even where
//! `/tmp` is a symlink) with its own `HOME`, `XDG_CONFIG_HOME`, `TMPDIR`, state directory and
//! workspace. The binary runs with a cleared environment plus exactly those variables and an
//! explicit `PATH`, so nothing leaks in from the developer's machine, and the test process never
//! changes its own environment or working directory.

#![allow(dead_code)] // each test file uses a different subset

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use std::time::Duration;

use assert_cmd::Command;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use shellgame::state::{State, StateStore};
use tempfile::TempDir;

/// The binary under test.
pub const BIN: &str = env!("CARGO_BIN_EXE_shellgame");

/// Marks a protocol line on stderr.
const DIRECTIVE_PREFIX: &str = "__SHELLGAME_EXEC__v1 ";

/// How long a command may run before the test fails instead of hanging.
const TIMEOUT: Duration = Duration::from_secs(60);

/// How often a spawn is retried while the kernel reports `ETXTBSY`.
const BUSY_RETRIES: u32 = 100;

/// Runs a command, retrying while the kernel says the executable is "busy".
///
/// A test that has just written an executable (a copy of the binary, a fake shell) and then runs
/// it can lose a race against another test thread: a child that thread forks briefly holds a copy
/// of the still-open write descriptor, and `exec` of a file that is open for writing fails with
/// `ETXTBSY` until that child has exec'd. The condition clears within milliseconds.
pub fn output_retrying(command: &mut Command) -> io::Result<std::process::Output> {
    let mut retries = 0;
    loop {
        match command.output() {
            Err(error)
                if error.kind() == io::ErrorKind::ExecutableFileBusy && retries < BUSY_RETRIES =>
            {
                retries += 1;
                std::thread::sleep(Duration::from_millis(20));
            }
            other => return other,
        }
    }
}

/// [`output_retrying`] for a plain `std` command.
pub fn std_output_retrying(command: &mut StdCommand) -> io::Result<std::process::Output> {
    let mut retries = 0;
    loop {
        match command.output() {
            Err(error)
                if error.kind() == io::ErrorKind::ExecutableFileBusy && retries < BUSY_RETRIES =>
            {
                retries += 1;
                std::thread::sleep(Duration::from_millis(20));
            }
            other => return other,
        }
    }
}

/// One protocol directive the binary printed on stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    pub verb: String,
    /// The decoded arguments (lossy UTF-8).
    pub args: Vec<String>,
}

/// What one run of the binary did.
#[derive(Debug, Clone)]
pub struct Run {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Run {
    pub fn of(output: &std::process::Output) -> Self {
        Self {
            code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Every directive on stderr, in order.
    pub fn directives(&self) -> Vec<Directive> {
        self.stderr
            .lines()
            .filter_map(|line| line.strip_prefix(DIRECTIVE_PREFIX))
            .map(|rest| {
                let mut words = rest.split(' ');
                let verb = words.next().unwrap_or_default().to_owned();
                let args = words
                    .map(|word| {
                        let bytes = STANDARD.decode(word).expect("base64 argument");
                        String::from_utf8_lossy(&bytes).into_owned()
                    })
                    .collect();
                Directive { verb, args }
            })
            .collect()
    }

    /// Stderr without the protocol lines: what the wrapper would print to the player.
    pub fn plain_stderr(&self) -> String {
        self.stderr
            .lines()
            .filter(|line| !line.starts_with("__SHELLGAME_EXEC__"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The destinations of the `cd` directives, in order.
    pub fn cds(&self) -> Vec<String> {
        self.directives()
            .into_iter()
            .filter(|d| d.verb == "cd")
            .map(|d| d.args[0].clone())
            .collect()
    }

    /// The `export` directives as `(name, value)`, in order.
    pub fn exports(&self) -> Vec<(String, String)> {
        self.directives()
            .into_iter()
            .filter(|d| d.verb == "export")
            .map(|d| (d.args[0].clone(), d.args[1].clone()))
            .collect()
    }

    /// The value of the last export of `name`.
    pub fn exported(&self, name: &str) -> Option<String> {
        self.exports()
            .into_iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }

    /// Whether the run asked the shell to exit.
    pub fn asked_to_exit(&self) -> bool {
        self.directives().iter().any(|d| d.verb == "exit")
    }

    /// Whether stdout contains `needle`, ignoring box-drawing characters and how the text was
    /// wrapped: panels fold long lines, and the test is about the words.
    pub fn says(&self, needle: &str) -> bool {
        squash(&self.stdout).contains(&squash(needle))
    }

    /// Like [`Run::says`], ignoring spaces altogether: a word longer than the panel is folded in
    /// the middle, which `says` would see as a space.
    pub fn says_compactly(&self, needle: &str) -> bool {
        let compact = |text: &str| squash(text).replace(' ', "");
        compact(&self.stdout).contains(&compact(needle))
    }

    /// [`Run::says`] for stderr, with the protocol lines removed.
    pub fn says_on_stderr(&self, needle: &str) -> bool {
        squash(&self.plain_stderr()).contains(&squash(needle))
    }

    /// Fails with the whole output when stdout does not contain `needle`.
    #[track_caller]
    pub fn assert_says(&self, needle: &str) {
        assert!(
            self.says(needle),
            "stdout does not say {needle:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            self.stdout,
            self.stderr
        );
    }

    /// Fails with the whole output when the exit status is not `code`.
    #[track_caller]
    pub fn assert_code(&self, code: i32) {
        assert_eq!(
            self.code, code,
            "unexpected exit status\n--- stdout ---\n{}\n--- stderr ---\n{}",
            self.stdout, self.stderr
        );
    }
}

/// Removes ANSI escape sequences, replaces box-drawing characters by spaces and collapses all
/// whitespace.
fn squash(text: &str) -> String {
    const BOX: &str = "│╭╮╰╯─┃┏┓┡┩┳╇┴━└┘┬┼├┤";
    let mut plain = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // `ESC [` parameters, then one final letter.
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else if BOX.contains(c) {
            plain.push(' ');
        } else {
            plain.push(c);
        }
    }
    plain.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// An isolated game.
pub struct Game {
    _root: TempDir,
    /// The sandbox root (canonical).
    pub base: PathBuf,
    pub home: PathBuf,
    pub xdg: PathBuf,
    /// `TMPDIR`; the workspace lives inside it, so `remove` accepts it.
    pub tmp: PathBuf,
    pub state_dir: PathBuf,
    pub workspace: PathBuf,
    /// The directory the binary runs in: the player's "current directory".
    pub cwd: PathBuf,
    /// Run inside the game shell (`SHELLGAME_WRAPPER=1`). The default.
    pub wrapped: bool,
    /// Environment overrides; `None` removes the variable.
    overrides: BTreeMap<String, Option<OsString>>,
}

impl Game {
    pub fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("sg-cli-")
            .tempdir()
            .expect("create the sandbox");
        let base = root.path().canonicalize().expect("canonical sandbox");
        let [home, xdg, tmp, state_dir, cwd] =
            ["home", "config", "tmp", "state", "cwd"].map(|name| base.join(name));
        for dir in [&home, &xdg, &tmp, &cwd] {
            fs::create_dir_all(dir).expect("create a sandbox directory");
        }
        let workspace = tmp.join("workspace");
        Self {
            _root: root,
            base,
            home,
            xdg,
            tmp,
            state_dir,
            workspace,
            cwd,
            wrapped: true,
            overrides: BTreeMap::new(),
        }
    }

    /// Sets an environment variable for the binary.
    pub fn set_env(&mut self, name: &str, value: impl Into<OsString>) -> &mut Self {
        self.overrides.insert(name.to_owned(), Some(value.into()));
        self
    }

    /// Removes a variable the sandbox would otherwise set.
    pub fn unset_env(&mut self, name: &str) -> &mut Self {
        self.overrides.insert(name.to_owned(), None);
        self
    }

    /// The environment of the binary: a private world and an explicit `PATH`.
    fn environment(&self) -> BTreeMap<String, OsString> {
        let mut env: BTreeMap<String, OsString> = BTreeMap::new();
        env.insert("PATH".into(), std::env::var_os("PATH").unwrap_or_default());
        env.insert("HOME".into(), self.home.clone().into());
        env.insert("USER".into(), "ada".into());
        env.insert("XDG_CONFIG_HOME".into(), self.xdg.clone().into());
        env.insert("TMPDIR".into(), self.tmp.clone().into());
        env.insert("SHELLGAME_STATE_DIR".into(), self.state_dir.clone().into());
        env.insert("SHELLGAME_WORKSPACE".into(), self.workspace.clone().into());
        if self.wrapped {
            env.insert("SHELLGAME_WRAPPER".into(), "1".into());
        }
        for (name, value) in &self.overrides {
            match value {
                Some(value) => env.insert(name.clone(), value.clone()),
                None => env.remove(name),
            };
        }
        env
    }

    /// The binary under test as a plain command, in the sandbox, with the given working directory.
    pub fn std_command_in(&self, binary: &Path, dir: &Path) -> StdCommand {
        let mut command = StdCommand::new(binary);
        command
            .env_clear()
            .envs(self.environment())
            .current_dir(dir);
        command
    }

    /// `sh -c <script>` in the sandbox environment, started in `dir`; the script finds the binary
    /// under test in `$SG`.
    pub fn sh(&self, dir: &Path, script: &str) -> StdCommand {
        let mut command = StdCommand::new("/bin/sh");
        command
            .env_clear()
            .envs(self.environment())
            .env("SG", BIN)
            .current_dir(dir)
            .arg("-c")
            .arg(script);
        command
    }

    /// The binary under test as a plain command, started in [`Game::cwd`].
    pub fn std_command(&self) -> StdCommand {
        self.std_command_in(Path::new(BIN), &self.cwd)
    }

    /// The binary under test as an `assert_cmd` command, started in [`Game::cwd`].
    pub fn command(&self) -> Command {
        let mut command = Command::from_std(self.std_command());
        command.timeout(TIMEOUT);
        command
    }

    /// Runs the binary with these arguments (stdin is an empty pipe, so never a terminal).
    pub fn run(&self, args: &[&str]) -> Run {
        let output = output_retrying(self.command().args(args)).expect("run the binary");
        Run::of(&output)
    }

    /// Like [`Game::run`], for commands that start another program that this test wrote just
    /// before (a fake shell): the program may be reported busy (see [`output_retrying`]), which
    /// the binary prints as a launch failure, so the whole run is repeated.
    pub fn run_booting(&self, args: &[&str]) -> Run {
        for _ in 0..BUSY_RETRIES {
            let run = self.run(args);
            if !run.stdout.contains("Text file busy") {
                return run;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("the fake shell stayed busy");
    }

    /// Like [`Game::run`], with this text on stdin.
    pub fn run_with_stdin(&self, args: &[&str], stdin: &str) -> Run {
        let output =
            output_retrying(self.command().args(args).write_stdin(stdin)).expect("run the binary");
        Run::of(&output)
    }

    /// Runs the binary in another directory (the player stands elsewhere).
    pub fn run_in(&self, dir: &Path, args: &[&str]) -> Run {
        let mut command = Command::from_std(self.std_command_in(Path::new(BIN), dir));
        let output = output_retrying(command.timeout(TIMEOUT).args(args)).expect("run the binary");
        Run::of(&output)
    }

    /// Starts a new game: the bare command on the first run (it also runs through the intro).
    pub fn start(&self) -> Run {
        let run = self.run(&[]);
        run.assert_code(0);
        run
    }

    /// `dev jump <id>`.
    pub fn jump(&self, id: &str) -> Run {
        let run = self.run(&["dev", "jump", id]);
        run.assert_code(0);
        run
    }

    /// The state store of this game.
    pub fn store(&self) -> StateStore {
        StateStore::new(&self.state_dir)
    }

    /// The saved state. Fails the test when there is none.
    #[track_caller]
    pub fn state(&self) -> State {
        self.store()
            .load()
            .expect("the state loads")
            .expect("a state exists")
    }

    /// Overwrites the saved state.
    pub fn save(&self, state: &State) {
        self.store().save(state).expect("save the state");
    }

    /// The raw bytes of `state.json`.
    pub fn state_bytes(&self) -> Vec<u8> {
        fs::read(self.state_dir.join("state.json")).expect("read state.json")
    }

    /// Where the evidence marker `name` (a dotfile) of the workspace lives.
    pub fn marker(&self, name: &str) -> PathBuf {
        self.workspace.join(name)
    }

    /// A path inside the workspace.
    pub fn ws(&self, relative: &str) -> PathBuf {
        self.workspace.join(relative)
    }
}

/// Whether the current user can write into a directory made read-only (that is, is root); such
/// tests are skipped.
pub fn permissions_are_ignored(probe_dir: &Path) -> bool {
    let probe = probe_dir.join("probe-dir");
    fs::create_dir_all(&probe).expect("create a probe directory");
    fs::set_permissions(&probe, fs::Permissions::from_mode(0o555)).expect("chmod the probe");
    let writable = fs::write(probe.join("file"), "x").is_ok();
    fs::set_permissions(&probe, fs::Permissions::from_mode(0o755)).expect("restore the probe");
    writable
}

/// Runs the binary on a pseudo-terminal, so prompts appear. A small Python helper drives the
/// terminal (the dependencies allowed here have no pty support), which is why these tests are
/// skipped where `python3` is missing.
///
/// `keys` are what the "press Enter" prompts receive, one per prompt, in order. Standard output and
/// standard error share the terminal, so their order in `output` is the order in which the binary
/// wrote them.
pub struct PtyRun {
    /// The exit status; `None` when the program was killed (by a signal or the helper's timeout).
    pub code: Option<i32>,
    pub output: String,
}

const PTY_HELPER: &str = r#"
import json, os, pty, select, signal, sys, termios, time

binary, cwd, keys_json, *args = sys.argv[1:]
keys = [key.encode() for key in json.loads(keys_json)]
pid, fd = pty.fork()
if pid == 0:
    os.chdir(cwd)
    os.execv(binary, [binary] + args)

out = b""
deadline = time.time() + 30
exited = None
while time.time() < deadline:
    ready, _, _ = select.select([fd], [], [], 0.2)
    if ready:
        try:
            data = os.read(fd, 65536)
        except OSError:
            break
        if not data:
            break
        out += data
        if keys and "Enter pro pokra".encode() in data:
            # Wait until the prompt has put the terminal into non-canonical mode (ISIG off), so a
            # Ctrl-C is delivered as a key and not as a signal.
            for _ in range(300):
                if not termios.tcgetattr(fd)[3] & termios.ICANON:
                    break
                time.sleep(0.01)
            os.write(fd, keys.pop(0))
    else:
        finished, status = os.waitpid(pid, os.WNOHANG)
        if finished:
            exited = status
            break
if exited is None:
    # The terminal closes a moment before the exit status can be collected.
    for _ in range(500):
        finished, status = os.waitpid(pid, os.WNOHANG)
        if finished:
            exited = status
            break
        time.sleep(0.01)
    else:
        os.kill(pid, signal.SIGKILL)
        os.waitpid(pid, 0)
code = os.WEXITSTATUS(exited) if exited is not None and os.WIFEXITED(exited) else -1
sys.stdout.buffer.write(out + b"\nSTATUS:" + str(code).encode() + b"\n")
"#;

/// Runs the binary of `game` on a pseudo-terminal; see [`PtyRun`]. `None` when `python3` is
/// missing.
pub fn run_in_pty(game: &Game, args: &[&str], keys: &[&str]) -> Option<PtyRun> {
    if !tool_works("python3") {
        return None;
    }
    let keys = serde_json::to_string(keys).expect("encode the keys");
    let output = game
        .std_command_in(Path::new("python3"), &game.cwd)
        .args(["-I", "-c", PTY_HELPER, BIN])
        .arg(&game.cwd)
        .arg(keys)
        .args(args)
        .output()
        .expect("run the pty helper");
    assert!(
        output.status.success(),
        "the pty helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout).into_owned();
    let (output, status) = text
        .rsplit_once("\nSTATUS:")
        .expect("the helper reports a status");
    let code = status.trim().parse::<i32>().ok().filter(|code| *code >= 0);
    Some(PtyRun {
        code,
        output: output.to_owned(),
    })
}

/// Prints why a test does not run; visible with `cargo test -- --nocapture`.
pub fn skip(reason: &str) {
    eprintln!("SKIPPED: {reason}");
}

/// Whether an external tool starts (`tool --version`).
pub fn tool_works(tool: &str) -> bool {
    StdCommand::new(tool)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
