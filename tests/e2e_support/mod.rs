//! Support for the end-to-end suites `tests/solutions.rs` and `tests/strict_cd.rs`: an isolated
//! sandbox, a way to run the real binary and the real integration script in a real shell, and a
//! parser for what the shell did.
//!
//! # How a shell run works
//!
//! A [`Script`] is a list of named *steps*: a few commands each, such as `shellgame dev jump 1.4`
//! or one `cd`. [`Sandbox::run_shell`] turns it into one `bash --noprofile --norc -c` or
//! `fish --no-config -c` command that
//!
//! 1. sets the `umask`, then sources the rendered integration script (`SHELLGAME_INIT`), so the
//!    shell has the real `shellgame` function, `cd` hook and `pwd` wrapper, and
//! 2. runs every step between two marker lines, on stdout and on stderr, and after the step prints
//!    its exit status, the shell's `$PWD` and, when the step asks for it, a dump of `state.json`.
//!
//! The whole script runs in **one shell process**, so the shell keeps what a player's shell keeps
//! between commands: the directory, `OLDPWD` and fish's directory history. The harness reads the
//! markers back ([`ShellRun::step`]), which gives every step its own stdout, stderr, status,
//! final directory and a snapshot of the saved game. The checks between steps are made on those
//! records after the shell exits.
//!
//! # Isolation
//!
//! Every [`Sandbox`] owns one temporary directory with its own `HOME`, `XDG_CONFIG_HOME`, state
//! directory, `TMPDIR` and a workspace whose path contains a space. The shell and the binary run
//! with a cleared environment plus exactly the variables the integration contract names (see
//! `src/shell/mod.rs`), the test process's `PATH` and a fixed locale, so nothing leaks in from the
//! machine and nothing leaks out. The test process never changes its own environment or working
//! directory, so sandboxes can run on any number of threads.
//!
//! The shell reads its standard input from `/dev/null`: the game only prompts on a terminal, so
//! no prompt can swallow a script. Output goes to files, never to pipes, so a runaway child
//! cannot block the harness, and a shell that runs longer than [`TIMEOUT`] is killed (a child it
//! started may outlive it for a moment, but cannot hold the harness up).
//!
//! `$SHELLGAME_TEST_LOCALE` picks the locale of the shells (`C.UTF-8` or `C` otherwise), to try
//! the course under another one, such as the player's `cs_CZ.UTF-8`.

#![allow(dead_code)] // each suite uses a different subset

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use libtest_mimic::{Completion, Failed, Trial};
use serde_json::Value;
use shellgame::engine::prelude::Catalog;
use shellgame::shell::{ScriptOptions, ShellKind, render_bash, render_fish, write_fd_launcher};
use tempfile::TempDir;

/// The binary under test.
pub const BIN: &str = env!("CARGO_BIN_EXE_shellgame");

/// How long one shell (or one run of the binary) may take before it is killed.
pub const TIMEOUT: Duration = Duration::from_secs(60);

/// What the marker lines of a script start with. Never appears in the output of a level.
const MARK: &str = "@@SG@@ ";

/// The text of a protocol line; it must never reach the player.
const PROTOCOL: &str = "__SHELLGAME_EXEC__";

// ---------------------------------------------------------------------------------------------
// The course and the tools
// ---------------------------------------------------------------------------------------------

/// The course as the binary under test has it.
pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(shellgame::levels::catalog)
}

/// Runs `tool arg`; true when it starts and succeeds.
fn tool_works(tool: &str, arg: &str) -> bool {
    Command::new(tool)
        .arg(arg)
        .stdin(Stdio::null())
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Whether the shell is installed. fish is optional: its trials are ignored when it is missing.
pub fn shell_available(shell: ShellKind) -> bool {
    static BASH: OnceLock<bool> = OnceLock::new();
    static FISH: OnceLock<bool> = OnceLock::new();
    let cell = match shell {
        ShellKind::Bash => &BASH,
        ShellKind::Fish => &FISH,
    };
    *cell.get_or_init(|| tool_works(shell.name(), "--version"))
}

/// The first line of `tool --version`, for the note at the start of a suite.
fn version_of(tool: &str) -> String {
    Command::new(tool)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "not installed".to_owned())
}

/// Prints which shells and locale the suite runs with (a failure on one machine and not on another
/// is usually a version), and a note when fish is missing: its trials are then ignored.
pub fn announce(suite: &str) {
    eprintln!(
        "{suite}: {}; {}; locale {}",
        version_of("bash"),
        version_of("fish"),
        locale()
    );
    if !shell_available(ShellKind::Fish) {
        eprintln!("{suite}: fish is not installed, so the fish trials are ignored");
    }
}

/// A trial that needs `shell`. It is ignored, with a note, when the shell is not installed, so
/// the whole suite passes with plain `cargo test` on a machine without fish.
pub fn shell_trial(
    name: impl Into<String>,
    shell: ShellKind,
    run: impl FnOnce() -> Result<(), Failed> + Send + 'static,
) -> Trial {
    Trial::ignorable_test(name, move || {
        if !shell_available(shell) {
            return Ok(Completion::ignored_with(format!(
                "{shell} is not installed"
            )));
        }
        run().map(|()| Completion::Completed)
    })
}

/// The locale of every shell: `$SHELLGAME_TEST_LOCALE` when it is set (to try the course in
/// `cs_CZ.UTF-8`, say), else `C.UTF-8` where the system has it (the level texts are Czech), else
/// `C`.
fn locale() -> &'static str {
    static LOCALE: OnceLock<String> = OnceLock::new();
    LOCALE.get_or_init(|| {
        if let Some(wanted) = std::env::var("SHELLGAME_TEST_LOCALE")
            .ok()
            .filter(|locale| !locale.is_empty())
        {
            return wanted;
        }
        let utf8 = Command::new("locale")
            .arg("charmap")
            .env("LC_ALL", "C.UTF-8")
            .stdin(Stdio::null())
            .output()
            .is_ok_and(|output| {
                output.status.success()
                    && String::from_utf8_lossy(&output.stdout)
                        .trim()
                        .eq_ignore_ascii_case("UTF-8")
            });
        (if utf8 { "C.UTF-8" } else { "C" }).to_owned()
    })
}

// ---------------------------------------------------------------------------------------------
// Text helpers
// ---------------------------------------------------------------------------------------------

/// `text` without ANSI escape sequences (the game forces colour on the stderr it prints, because
/// the wrapper replays it later).
pub fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // `ESC [` parameters, then one final letter.
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// `text` without escapes and box-drawing characters, with all whitespace collapsed: panels fold
/// long lines, and the checks are about the words.
pub fn squash(text: &str) -> String {
    const BOX: &str = "│╭╮╰╯─┃┏┓┡┩┳╇┴━└┘┬┼├┤";
    let spaced: String = plain(text)
        .chars()
        .map(|c| if BOX.contains(c) { ' ' } else { c })
        .collect();
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// At most `limit` characters of `text`: the head and the tail, when it is longer.
pub fn clip(text: &str, limit: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= limit {
        return text.to_owned();
    }
    let half = limit / 2;
    let head: String = chars[..half].iter().collect();
    let tail: String = chars[chars.len() - half..].iter().collect();
    format!(
        "{head}\n... [{} characters left out] ...\n{tail}",
        chars.len() - 2 * half
    )
}

/// `text` indented, for reports.
pub fn indent(text: &str) -> String {
    if text.is_empty() {
        return "    (nothing)".to_owned();
    }
    text.lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A path as one shell word that bash and fish read the same way. The sandbox paths contain
/// spaces but never quotes or backslashes.
pub fn quote(path: &Path) -> String {
    let text = path.to_str().expect("sandbox paths are UTF-8");
    assert!(
        !text.contains(['\'', '\\']),
        "cannot quote {text:?} for both shells"
    );
    format!("'{text}'")
}

/// The command that moves the player without asking the game: what the `shellgame` function does
/// for a teleport (`builtin cd` in bash, the stock `cd` that the script keeps in fish).
pub fn teleport(shell: ShellKind, dir: &Path) -> String {
    match shell {
        ShellKind::Bash => format!("builtin cd -- {}", quote(dir)),
        ShellKind::Fish => format!("__shellgame_cd -- {}", quote(dir)),
    }
}

// ---------------------------------------------------------------------------------------------
// The sandbox
// ---------------------------------------------------------------------------------------------

/// What one captured process did.
#[derive(Debug)]
pub struct Captured {
    /// The exit status; `None` when the process was killed by a signal (also by the timeout).
    pub status: Option<i32>,
    pub timed_out: bool,
    pub stdout: String,
    pub stderr: String,
}

/// An isolated world for one game: see the module documentation.
pub struct Sandbox {
    /// The sandbox directory, canonical (so `$PWD` and `pwd -P` agree where `/tmp` is a link).
    pub root: PathBuf,
    pub home: PathBuf,
    pub tmp: PathBuf,
    pub config: PathBuf,
    /// `SHELLGAME_STATE_DIR`: holds `state.json` and the fd-hook launcher.
    pub state_dir: PathBuf,
    /// `SHELLGAME_WORKSPACE`; its name contains a space.
    pub workspace: PathBuf,
    pub fd_hook: PathBuf,
    /// How long one process may run; [`TIMEOUT`] unless a test shortens it.
    timeout: Duration,
    runs: AtomicUsize,
    // Dropped after `Drop for Sandbox` has made everything removable.
    _dir: TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("sg-e2e-")
            .tempdir()
            .expect("create the sandbox");
        let root = dir.path().canonicalize().expect("canonical sandbox path");
        let [home, tmp, config, state_dir] =
            ["home", "tmp", "config", "state"].map(|name| root.join(name));
        for path in [&home, &tmp, &config] {
            fs::create_dir_all(path).expect("create a sandbox directory");
        }
        let fd_hook = write_fd_launcher(&state_dir).expect("write the fd-hook launcher");
        Self {
            workspace: root.join("work space"),
            root,
            home,
            tmp,
            config,
            state_dir,
            fd_hook,
            timeout: TIMEOUT,
            runs: AtomicUsize::new(0),
            _dir: dir,
        }
    }

    /// The same sandbox with a shorter time limit (for the tests of the harness itself).
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// The environment of everything that runs in the sandbox: a private world and the test
    /// process's `PATH`. Nothing else is inherited.
    fn base_env(&self) -> BTreeMap<&'static str, OsString> {
        let path = |value: &Path| OsString::from(value.as_os_str());
        BTreeMap::from([
            ("PATH", std::env::var_os("PATH").unwrap_or_default()),
            ("HOME", path(&self.home)),
            ("USER", "student".into()),
            ("LC_ALL", locale().into()),
            ("TMPDIR", path(&self.tmp)),
            ("XDG_CONFIG_HOME", path(&self.config)),
            ("SHELLGAME_STATE_DIR", path(&self.state_dir)),
            ("SHELLGAME_WORKSPACE", path(&self.workspace)),
        ])
    }

    /// The environment of a game shell: the base plus the launcher's contract.
    fn shell_env(&self, init: &Path) -> BTreeMap<&'static str, OsString> {
        let mut env = self.base_env();
        env.insert("SHELLGAME_WRAPPER", "1".into());
        env.insert("SHELLGAME_BIN", BIN.into());
        env.insert("SHELLGAME_INIT", init.as_os_str().to_owned());
        env.insert("SHELLGAME_FD_HOOK", self.fd_hook.as_os_str().to_owned());
        env.insert("SHELLGAME_AUTO_STARTED", "1".into());
        env
    }

    /// Runs a command with its output in files, and kills it when it runs too long.
    fn capture(&self, mut command: Command) -> Captured {
        let tag = self.runs.fetch_add(1, Ordering::Relaxed);
        let out_path = self.root.join(format!("run-{tag}.out"));
        let err_path = self.root.join(format!("run-{tag}.err"));
        command
            .stdin(Stdio::null())
            .stdout(File::create(&out_path).expect("create the stdout file"))
            .stderr(File::create(&err_path).expect("create the stderr file"));
        let mut child = command
            .spawn()
            .unwrap_or_else(|error| panic!("cannot start {:?}: {error}", command.get_program()));
        let started = Instant::now();
        let mut pause = Duration::from_millis(1);
        let (status, timed_out) = loop {
            match child.try_wait().expect("wait for the process") {
                Some(status) => break (status, false),
                None if started.elapsed() >= self.timeout => {
                    let _ = child.kill();
                    break (child.wait().expect("reap the killed process"), true);
                }
                None => {
                    thread::sleep(pause);
                    pause = (pause * 2).min(Duration::from_millis(20));
                }
            }
        };
        let read =
            |path: &Path| String::from_utf8_lossy(&fs::read(path).unwrap_or_default()).into_owned();
        Captured {
            status: status.code(),
            timed_out,
            stdout: read(&out_path),
            stderr: read(&err_path),
        }
    }

    /// Runs the binary directly (inside the game shell's environment, but without a shell), in
    /// the player's home directory.
    pub fn game(&self, args: &[&str]) -> Captured {
        self.game_in(&self.home, args)
    }

    /// Runs the binary directly in `dir`.
    pub fn game_in(&self, dir: &Path, args: &[&str]) -> Captured {
        let mut env = self.base_env();
        env.insert("SHELLGAME_WRAPPER", "1".into());
        let mut command = Command::new(BIN);
        command.env_clear().envs(env).current_dir(dir).args(args);
        self.capture(command)
    }

    /// Runs the script in a game shell and parses what it printed.
    pub fn run_shell(&self, script: &Script) -> ShellRun {
        let options = ScriptOptions { devmode: false };
        let init = self.root.join(format!("init.{}", script.shell.name()));
        let rendered = match script.shell {
            ShellKind::Bash => render_bash(&options),
            ShellKind::Fish => render_fish(&options),
        };
        fs::write(&init, rendered).expect("write the integration script");

        let mut command = Command::new(script.shell.name());
        match script.shell {
            ShellKind::Bash => command.args(["--noprofile", "--norc", "-c"]),
            ShellKind::Fish => command.args(["--no-config", "-c"]),
        };
        command
            .arg(script.text())
            .env_clear()
            .envs(self.shell_env(&init))
            .current_dir(&self.home);
        ShellRun::parse(self.capture(command), self.timeout, script)
    }

    /// Where a level lives in this sandbox.
    pub fn place(&self, id: &str) -> Place {
        let (section, level) = catalog()
            .get(id.parse().expect("a level id"))
            .unwrap_or_else(|| panic!("the course has no level {id}"));
        let section_root = section
            .section_root(&self.workspace)
            .expect("a section root");
        let start = level.start_dir(&self.workspace, &section_root);
        Place {
            home: self.home.clone(),
            workspace: self.workspace.clone(),
            section_root,
            start,
        }
    }

    /// The reference solution of a level, for the player standing at its start directory.
    pub fn solution(&self, id: &str) -> Result<String, String> {
        let (section, level) = catalog()
            .get(id.parse().expect("a level id"))
            .unwrap_or_else(|| panic!("the course has no level {id}"));
        let solution = level
            .solution
            .as_ref()
            .ok_or_else(|| format!("level {id} has no solution"))?;
        let ctx = section
            .ctx(level, &self.workspace, self.place(id).start, &self.home)
            .map_err(|error| error.to_string())?;
        Ok(solution.script(&ctx))
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        // Levels take permissions away (section 8): the directories must be usable again before
        // `TempDir` can remove them.
        make_removable(&self.root);
    }
}

fn make_removable(path: &Path) {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return;
    };
    if !meta.is_dir() {
        return;
    }
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o700));
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            make_removable(&entry.path());
        }
    }
}

/// Where a level lives in a sandbox.
#[derive(Debug, Clone)]
pub struct Place {
    /// The player's `$HOME`.
    pub home: PathBuf,
    pub workspace: PathBuf,
    /// The directory the level's paths are relative to (`level-1`; the workspace for section 0).
    pub section_root: PathBuf,
    /// Where the player stands when the level starts, if the level places them at all.
    pub start: Option<PathBuf>,
}

impl Place {
    /// A path below the section root.
    pub fn in_section(&self, relative: &str) -> PathBuf {
        self.section_root.join(relative)
    }
}

// ---------------------------------------------------------------------------------------------
// Scripts
// ---------------------------------------------------------------------------------------------

struct Step {
    name: String,
    commands: String,
    state: bool,
}

/// A shell script made of named steps; see the module documentation.
pub struct Script {
    pub shell: ShellKind,
    umask: String,
    steps: Vec<Step>,
}

impl Script {
    /// A script for `shell` that runs everything under `umask`.
    pub fn new(shell: ShellKind, umask: &str) -> Self {
        Self {
            shell,
            umask: umask.to_owned(),
            steps: Vec::new(),
        }
    }

    /// Adds a step: `commands` run in the one shell, between two markers.
    pub fn step(&mut self, name: &str, commands: &str) -> &mut Self {
        self.push(name, commands, false)
    }

    /// Adds a step after which the saved game is dumped (see [`StepRecord::state`]).
    pub fn step_with_state(&mut self, name: &str, commands: &str) -> &mut Self {
        self.push(name, commands, true)
    }

    fn push(&mut self, name: &str, commands: &str, state: bool) -> &mut Self {
        assert!(
            self.steps.iter().all(|step| step.name != name),
            "step names are unique: {name}"
        );
        self.steps.push(Step {
            name: name.to_owned(),
            commands: commands.to_owned(),
            state,
        });
        self
    }

    /// The commands of every step, for reports.
    pub fn describe(&self) -> String {
        let mut out = String::new();
        for step in &self.steps {
            let mut lines = step.commands.lines();
            let first = lines.next().unwrap_or_default();
            let _ = writeln!(out, "  [{}] {first}", step.name);
            for line in lines {
                let _ = writeln!(out, "  {:width$} {line}", "", width = step.name.len() + 2);
            }
        }
        out
    }

    /// The text handed to the shell.
    fn text(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(text, "umask {}", self.umask);
        text.push_str(match self.shell {
            ShellKind::Bash => "source \"$SHELLGAME_INIT\"\n",
            ShellKind::Fish => "source $SHELLGAME_INIT\n",
        });
        for (index, step) in self.steps.iter().enumerate() {
            let n = index + 1;
            let _ = writeln!(text, "printf '{MARK}begin {n}\\n'");
            let _ = writeln!(text, "printf '{MARK}begin {n}\\n' >&2");
            text.push_str(&step.commands);
            if !step.commands.ends_with('\n') {
                text.push('\n');
            }
            // The status of the step, then whether the shell's directory is still the one its
            // path names (`.` and `$PWD` are the same directory; not so after the directory was
            // deleted and built again).
            text.push_str(match self.shell {
                ShellKind::Bash => "__sg_rc=$?\ncommand test . -ef \"$PWD\"\n__sg_alive=$?\n",
                ShellKind::Fish => {
                    "set __sg_rc $status\ncommand test . -ef \"$PWD\"\nset __sg_alive $status\n"
                }
            });
            if step.state {
                let _ = writeln!(text, "printf '{MARK}state\\n'");
                text.push_str("command cat -- \"$SHELLGAME_STATE_DIR/state.json\"\n");
                let _ = writeln!(text, "printf '\\n{MARK}/state\\n'");
            }
            let (status, alive) = match self.shell {
                ShellKind::Bash => ("\"$__sg_rc\"", "\"$__sg_alive\""),
                ShellKind::Fish => ("$__sg_rc", "$__sg_alive"),
            };
            let _ = writeln!(
                text,
                "printf '{MARK}end {n} status=%s alive=%s pwd=%s\\n' {status} {alive} \"$PWD\""
            );
            let _ = writeln!(text, "printf '{MARK}end {n}\\n' >&2");
        }
        let _ = writeln!(text, "printf '{MARK}done\\n'");
        text
    }
}

// ---------------------------------------------------------------------------------------------
// Failed checks
// ---------------------------------------------------------------------------------------------

/// A check that failed: in which step (when it is about one) and what was wrong.
#[derive(Debug, Clone)]
pub struct Problem {
    pub step: Option<String>,
    pub reason: String,
}

impl Problem {
    /// A problem of the run as a whole.
    pub fn run(reason: impl Into<String>) -> Self {
        Self {
            step: None,
            reason: reason.into(),
        }
    }

    /// A problem of one step.
    pub fn at(step: &str, reason: impl Into<String>) -> Self {
        Self {
            step: Some(step.to_owned()),
            reason: reason.into(),
        }
    }

    /// The headline of a report: `step `x`: reason`.
    pub fn headline(&self) -> String {
        match &self.step {
            Some(step) => format!("step `{step}`: {}", self.reason),
            None => self.reason.clone(),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// What a shell did
// ---------------------------------------------------------------------------------------------

/// What one step did.
#[derive(Debug, Clone, Default)]
pub struct StepRecord {
    pub name: String,
    pub commands: String,
    pub stdout: String,
    pub stderr: String,
    /// The exit status of the step's last command (`$?` in bash, `$status` in fish).
    pub status: Option<i32>,
    /// The shell's logical `$PWD` after the step.
    pub pwd: Option<PathBuf>,
    /// The directory the shell is in still exists and is the one `$PWD` names. It does not after
    /// something deleted it underneath the shell (and perhaps built it again at the same path).
    pub alive: Option<bool>,
    /// The saved game after the step, for steps added with [`Script::step_with_state`].
    pub state: Option<Value>,
    /// The step's end marker was seen: the shell got through it.
    pub ended: bool,
}

impl StepRecord {
    /// Whether `levels_complete` of the snapshot has this level; `None` without a snapshot.
    pub fn completed(&self, id: &str) -> Option<bool> {
        let state = self.state.as_ref()?;
        Some(state["levels_complete"].get(id).is_some())
    }

    /// `current_level` of the snapshot.
    pub fn current_level(&self) -> Option<&str> {
        self.state.as_ref()?["current_level"].as_str()
    }

    /// Whether the snapshot says the whole game is finished (`completed_at` is set).
    pub fn game_finished(&self) -> Option<bool> {
        Some(!self.state.as_ref()?["completed_at"].is_null())
    }

    /// The step's stderr without colour, panels' borders and line folds.
    pub fn stderr_words(&self) -> String {
        squash(&self.stderr)
    }

    /// One line for reports.
    fn summary(&self) -> String {
        let mut line = format!("[{}]", self.name);
        if !self.ended {
            line.push_str(" did not finish");
            return line;
        }
        let _ = write!(
            line,
            " status {}",
            self.status.map_or("?".to_owned(), |s| s.to_string())
        );
        if let Some(pwd) = &self.pwd {
            let _ = write!(line, ", pwd {}", pwd.display());
        }
        if let Some(state) = &self.state {
            let done: Vec<&str> = state["levels_complete"]
                .as_object()
                .map(|map| map.keys().map(String::as_str).collect())
                .unwrap_or_default();
            let _ = write!(
                line,
                ", game on {}, {} done",
                state["current_level"].as_str().unwrap_or("?"),
                match done.as_slice() {
                    [] => "none".to_owned(),
                    [.., last] if done.len() > 4 => format!("{} (last {last})", done.len()),
                    all => all.join(" "),
                }
            );
        }
        line
    }
}

/// The parsed output of a shell run.
#[derive(Debug)]
pub struct ShellRun {
    /// The shell's exit status; `None` when it was killed.
    pub exit: Option<i32>,
    pub timed_out: bool,
    /// The limit that was applied, for the message of a timeout.
    pub timeout: Duration,
    pub stdout: String,
    pub stderr: String,
    pub steps: Vec<StepRecord>,
    /// The last line of the script ran.
    pub finished: bool,
}

impl ShellRun {
    fn parse(captured: Captured, timeout: Duration, script: &Script) -> Self {
        let mut steps: Vec<StepRecord> = script
            .steps
            .iter()
            .map(|step| StepRecord {
                name: step.name.clone(),
                commands: step.commands.clone(),
                ..StepRecord::default()
            })
            .collect();
        let finished = parse_stdout(&captured.stdout, &mut steps);
        parse_stderr(&captured.stderr, &mut steps);
        Self {
            exit: captured.status,
            timed_out: captured.timed_out,
            timeout,
            stdout: captured.stdout,
            stderr: captured.stderr,
            steps,
            finished,
        }
    }

    /// The record of a step that ran to its end.
    pub fn step(&self, name: &str) -> Result<&StepRecord, String> {
        let step = self
            .steps
            .iter()
            .find(|step| step.name == name)
            .unwrap_or_else(|| panic!("the script has no step {name}"));
        if step.ended {
            Ok(step)
        } else {
            Err(format!(
                "the shell never got through step `{name}` (it ended, hung or died there)"
            ))
        }
    }

    /// Whether the run as a whole is sound: the shell exited normally after the last step, and
    /// neither a protocol line nor a Rust panic reached the player.
    pub fn sanity(&self) -> Result<(), String> {
        if self.timed_out {
            return Err(format!(
                "the shell did not finish within {} s and was killed",
                self.timeout.as_secs()
            ));
        }
        match self.exit {
            Some(0) => {}
            Some(code) => return Err(format!("the shell exited with status {code}")),
            None => return Err("the shell was killed by a signal".to_owned()),
        }
        if !self.finished {
            let at = self
                .steps
                .iter()
                .find(|step| !step.ended)
                .map_or("before the first step", |step| step.name.as_str());
            return Err(format!("the script stopped early, at step `{at}`"));
        }
        for (stream, text) in [("stdout", &self.stdout), ("stderr", &self.stderr)] {
            if text.contains(PROTOCOL) {
                return Err(format!("a protocol line ({PROTOCOL}) leaked into {stream}"));
            }
        }
        if self.stderr.contains("panicked at") {
            return Err("the game panicked".to_owned());
        }
        Ok(())
    }

    /// A report of the run for a failed check.
    pub fn report(&self, solution: Option<&str>, step: Option<&str>) -> String {
        let mut out = String::new();
        if let Some(solution) = solution {
            let _ = writeln!(out, "solution:\n{}", indent(solution));
        }
        out.push_str("steps:\n");
        let focus_index = step.and_then(|name| self.steps.iter().position(|r| r.name == name));
        let mut left_out = 0;
        for (index, record) in self.steps.iter().enumerate() {
            // A long script (the whole course) shows its first steps and the ones around the
            // failure; the rest is counted.
            let near = |center: usize| index + 8 >= center && index <= center + 1;
            let shown = self.steps.len() <= 14
                || index < 2
                || focus_index.map_or(index + 6 >= self.steps.len(), near);
            if shown {
                if left_out > 0 {
                    let _ = writeln!(out, "  ... {left_out} steps left out");
                    left_out = 0;
                }
                let _ = writeln!(out, "  {}", record.summary());
            } else {
                left_out += 1;
            }
        }
        if left_out > 0 {
            let _ = writeln!(out, "  ... {left_out} steps left out");
        }
        let focus = focus_index.map(|index| &self.steps[index]);
        if let Some(record) = focus {
            let _ = writeln!(
                out,
                "commands of step `{}`:\n{}",
                record.name,
                indent(&record.commands)
            );
            let _ = writeln!(
                out,
                "stdout of step `{}`:\n{}",
                record.name,
                indent(&clip(&plain(&record.stdout), 3000))
            );
            let _ = writeln!(
                out,
                "stderr of step `{}`:\n{}",
                record.name,
                indent(&clip(&plain(&record.stderr), 3000))
            );
        }
        // When the run itself is broken, the whole output tells more than any step.
        if self.sanity().is_err() {
            let _ = writeln!(
                out,
                "whole stdout:\n{}",
                indent(&clip(&plain(&self.stdout), 3000))
            );
            let _ = writeln!(
                out,
                "whole stderr:\n{}",
                indent(&clip(&plain(&self.stderr), 3000))
            );
        }
        out
    }
}

/// Reads the markers on stdout into the step records. Returns whether the last line ran.
fn parse_stdout(text: &str, steps: &mut [StepRecord]) -> bool {
    let mut current: Option<usize> = None;
    let mut state: Option<String> = None;
    let mut finished = false;
    for line in text.split_inclusive('\n') {
        // A command that ended without a newline leaves its text in front of the marker.
        let (before, marker) = match line.find(MARK) {
            Some(at) => (
                &line[..at],
                Some(line[at + MARK.len()..].trim_end_matches('\n')),
            ),
            None => (line, None),
        };
        if let Some(buffer) = state.as_mut() {
            buffer.push_str(before);
        } else if let Some(index) = current {
            steps[index].stdout.push_str(before);
        }
        let Some(marker) = marker else { continue };
        let (kind, rest) = marker.split_once(' ').unwrap_or((marker, ""));
        match kind {
            "begin" => current = step_index(rest, steps.len()),
            "end" => {
                if let Some(index) =
                    step_index(rest.split(' ').next().unwrap_or_default(), steps.len())
                {
                    let step = &mut steps[index];
                    step.ended = true;
                    if let Some((_, tail)) = rest.split_once(" status=") {
                        let (status, tail) = tail.split_once(" alive=").unwrap_or((tail, ""));
                        let (alive, pwd) = tail.split_once(" pwd=").unwrap_or((tail, ""));
                        step.status = status.trim().parse().ok();
                        step.alive = alive.trim().parse::<i32>().ok().map(|code| code == 0);
                        step.pwd = Some(PathBuf::from(pwd));
                    }
                }
                current = None;
            }
            "state" => state = Some(String::new()),
            "/state" => {
                if let (Some(buffer), Some(index)) = (state.take(), current) {
                    steps[index].state = serde_json::from_str(&buffer).ok();
                }
            }
            "done" => finished = true,
            _ => {}
        }
    }
    finished
}

/// Reads the markers on stderr into the step records.
fn parse_stderr(text: &str, steps: &mut [StepRecord]) {
    let mut current: Option<usize> = None;
    for line in text.split_inclusive('\n') {
        let (before, marker) = match line.find(MARK) {
            Some(at) => (
                &line[..at],
                Some(line[at + MARK.len()..].trim_end_matches('\n')),
            ),
            None => (line, None),
        };
        if let Some(index) = current {
            steps[index].stderr.push_str(before);
        }
        let Some(marker) = marker else { continue };
        let (kind, rest) = marker.split_once(' ').unwrap_or((marker, ""));
        match kind {
            "begin" => current = step_index(rest, steps.len()),
            "end" => current = None,
            _ => {}
        }
    }
}

/// The index of the step that a 1-based marker number names.
fn step_index(number: &str, count: usize) -> Option<usize> {
    number
        .trim()
        .parse::<usize>()
        .ok()
        .and_then(|n| n.checked_sub(1))
        .filter(|&index| index < count)
}
