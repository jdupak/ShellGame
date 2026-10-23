//! Shared by the integration suites: an isolated world per test, a runner for the binary and a
//! single-process runner for the game shell. Nothing here touches the test process's own
//! environment or working directory, so worlds can live on any number of threads.
#![allow(dead_code)] // each suite uses a different subset

use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::ErrorKind::ExecutableFileBusy;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde_json::Value;
use shellgame::game::paths::inside_tmp;
use shellgame::game::remove_tree;
use shellgame::sandbox;
use shellgame::shell::{self, ShellKind};
use tempfile::TempDir;

pub const BIN: &str = env!("CARGO_BIN_EXE_shellgame");
const TICK: Duration = Duration::from_millis(2);

/// The words of a text: escape sequences and box drawing removed, whitespace collapsed.
pub fn squash(text: &str) -> String {
    let (mut escape, boxes) = (false, "│╭╮╰╯─");
    let kept = text.chars().filter(|&c| {
        let skip = escape || c == '\x1b';
        escape = c == '\x1b' || (escape && !c.is_ascii_alphabetic());
        !skip
    });
    let words: String = kept.map(|c| [c, ' '][boxes.contains(c) as usize]).collect();
    words.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn have(tool: &str) -> bool {
    Command::new(tool).arg("--version").output().is_ok_and(|output| output.status.success())
}

/// Whether Landlock confines here. `SHELLGAME_TEST_LANDLOCK=required` (CI) makes "no" a failure.
pub fn landlock() -> bool {
    static WORKS: OnceLock<bool> = OnceLock::new();
    let works = *WORKS.get_or_init(|| sandbox::probe(Path::new(BIN), Path::new("/tmp")));
    let required = std::env::var("SHELLGAME_TEST_LANDLOCK").as_deref() == Ok("required");
    assert!(works || !required, "Landlock is required but unavailable");
    works
}

/// `C.UTF-8` where the system has it (level texts are Czech), else `C`.
fn locale() -> &'static str {
    static LOCALE: OnceLock<&str> = OnceLock::new();
    LOCALE.get_or_init(|| {
        let charmap = Command::new("locale").arg("charmap").env("LC_ALL", "C.UTF-8").output();
        let utf8 = charmap.is_ok_and(|o| String::from_utf8_lossy(&o.stdout).trim() == "UTF-8");
        if utf8 { "C.UTF-8" } else { "C" }
    })
}

#[derive(Debug)]
pub struct Run {
    pub code: i32,
    pub out: String,
    pub err: String,
}

/// One isolated world per test. `home` and `state` live outside /tmp (below `CARGO_TARGET_TMPDIR`)
/// as they do for a player; the workspace (its name has a space), the working directory and
/// `TMPDIR` live in /tmp. `tools` is for what a test writes and runs (/tmp may be `noexec`); `bin`
/// is the game binary the shells use.
pub struct World {
    pub out: PathBuf,
    pub home: PathBuf,
    pub state: PathBuf,
    pub tools: PathBuf,
    pub root: PathBuf,
    pub ws: PathBuf,
    pub cwd: PathBuf,
    pub tmp: PathBuf,
    pub bin: PathBuf,
    env: Vec<(&'static str, Option<OsString>)>,
    _dirs: [TempDir; 2],
}

impl World {
    pub fn new() -> World {
        let temp = |dir: &str| tempfile::Builder::new().prefix("sg-").tempdir_in(dir).unwrap();
        let (outside, inside) = (temp(env!("CARGO_TARGET_TMPDIR")), temp("/tmp"));
        // Canonical, so `$PWD` and `pwd -P` agree where /tmp is a link.
        let [out, root] = [&outside, &inside].map(|dir| dir.path().canonicalize().unwrap());
        // The sandbox tests need a home it protects, and the shells write there.
        assert!(
            !inside_tmp(&out),
            "the target directory {} is inside /tmp, where the game shell may write everything: \
             set CARGO_TARGET_DIR to a directory outside /tmp",
            out.display()
        );
        let [home, state, tools] = ["home", "state", "tools"].map(|name| out.join(name));
        let [ws, cwd, tmp] = ["work space", "cwd", "tmp"].map(|name| root.join(name));
        [&home, &tools, &cwd, &tmp].iter().for_each(|dir| fs::create_dir(dir).unwrap());
        // A fish player's home has fish's own directories (fish makes them on its first start).
        // fish 3.3 warns on every start when it cannot create them, and the sandbox forbids that.
        for dir in [".config/fish", ".local/share/fish"] {
            fs::create_dir_all(home.join(dir)).unwrap();
        }
        let (bin, env, _dirs) = (BIN.into(), vec![], [outside, inside]);
        World { out, home, state, tools, root, ws, cwd, tmp, bin, env, _dirs }
    }

    /// Sets a variable of everything that runs here, or removes it.
    pub fn set(&mut self, name: &'static str, value: impl Into<OsString>) -> &mut World {
        self.env.push((name, Some(value.into())));
        self
    }

    pub fn unset(&mut self, name: &'static str) -> &mut World {
        self.env.push((name, None));
        self
    }

    /// The world's variables on `cmd`, without clearing anything. `SHELLGAME_SANDBOX` makes the
    /// binary act as inside the game shell, with its directives on stderr: `unset` it to run outside.
    pub fn apply_env(&self, cmd: &mut Command) {
        cmd.env("USER", "ada").env("LC_ALL", locale()).env("HOME", &self.home);
        cmd.env("TMPDIR", &self.tmp).env(shell::ENV_SANDBOX, "off");
        cmd.env("SHELLGAME_STATE_DIR", &self.state).env("SHELLGAME_WORKSPACE", &self.ws);
        for (name, value) in &self.env {
            match value {
                Some(value) => cmd.env(name, value),
                None => cmd.env_remove(name),
            };
        }
    }

    /// A command in the cwd with an empty environment but `PATH` and the world's variables.
    pub fn command(&self, program: impl AsRef<OsStr>) -> Command {
        let mut cmd = Command::new(program);
        cmd.env_clear().env("PATH", std::env::var_os("PATH").unwrap_or_default());
        self.apply_env(cmd.current_dir(&self.cwd));
        cmd
    }

    /// Runs `cmd` to its end with `stdin` as its input. Output goes to files, so a runaway child
    /// cannot block the harness; a hung one is killed after a minute and fails the test.
    pub fn exec(&self, cmd: &mut Command, stdin: &str) -> Run {
        let [input, out, err] = ["in", "out", "err"].map(|name| self.root.join(format!(".{name}")));
        fs::write(&input, stdin).unwrap();
        cmd.stdin(File::open(input).unwrap());
        cmd.stdout(File::create(&out).unwrap()).stderr(File::create(&err).unwrap());
        let mut child = loop {
            match cmd.spawn() {
                // A file written a moment ago is "busy" while a thread of another test forks.
                Err(e) if e.kind() == ExecutableFileBusy => std::thread::sleep(TICK * 10),
                spawned => break spawned.unwrap(),
            }
        };
        let start = Instant::now();
        let status = loop {
            match child.try_wait().unwrap() {
                Some(status) => break status,
                None if start.elapsed() > Duration::from_secs(60) => {
                    child.kill().unwrap();
                    panic!("{cmd:?} was killed after a minute");
                }
                None => std::thread::sleep(TICK),
            }
        };
        let text = |path: PathBuf| String::from_utf8_lossy(&fs::read(path).unwrap()).into_owned();
        Run { code: status.code().unwrap_or(-1), out: text(out), err: text(err) }
    }

    pub fn run(&self, args: &[&str]) -> Run {
        self.exec(self.command(BIN).args(args), "")
    }

    /// Runs `steps` in ONE game shell process: the real script, hooks and `self.bin`, confined to /tmp,
    /// the state directory and /dev when Landlock works (so no level may need more). Each step runs
    /// in the shell itself with stdout and stderr in files of its own; afterwards the shell records its
    /// status, `$PWD`, whether that directory still exists and the saved game. `prelude` runs first.
    pub fn shell(&self, kind: ShellKind, prelude: &str, steps: &[impl AsRef<str>]) -> ShellRun {
        let (dir, init) = (self.root.join("steps"), self.out.join(format!("init.{kind}")));
        drop(fs::remove_dir_all(&dir)); // a world may run more than one shell
        fs::create_dir_all(&dir).unwrap();
        fs::write(&init, kind.script()).unwrap();
        let (flags, template): (&[&str], _) = match kind {
            ShellKind::Bash => (&["--noprofile", "--norc", "-c"], BASH_STEP),
            ShellKind::Fish => (&["--no-config", "-c"], FISH_STEP),
        };
        let step = |(n, step): (usize, &_)| {
            template.replace("{n}", &n.to_string()).replace("{step}", AsRef::<str>::as_ref(step))
        };
        let body: String = steps.iter().enumerate().map(step).collect();
        let script = SCRIPT.replace("{prelude}", prelude).replace("{body}", &body);
        // Landlock silently skips a directory that does not exist, and the game saves there.
        fs::create_dir_all(&self.state).unwrap();
        let mut cmd = self.command(kind.name());
        cmd.args(flags).arg(script).env("SG", &dir).env(shell::ENV_BIN, &self.bin);
        cmd.env(shell::ENV_INIT, init).env(shell::ENV_AUTO_STARTED, "1");
        self.apply_env(&mut cmd); // after the rest, so that a test may override any variable
        let confined = landlock();
        cmd.env(shell::ENV_SANDBOX, if confined { "landlock" } else { "off" });
        if confined {
            sandbox::confine(&mut cmd, &self.state).unwrap();
        }
        let run = self.exec(&mut cmd, "");
        let read = |n: usize, ext: &str| {
            fs::read_to_string(dir.join(format!("{n}.{ext}"))).unwrap_or_default()
        };
        let steps = steps.iter().enumerate().map(|(n, cmd)| {
            let info = read(n, "info");
            let mut words = info.splitn(3, ' ');
            let (status, alive) = (words.next().and_then(|w| w.parse().ok()), words.next());
            Step {
                cmd: cmd.as_ref().to_owned(),
                out: read(n, "out"),
                err: read(n, "err"),
                status: status.unwrap_or(-1),
                alive: alive == Some("1"),
                pwd: words.next().unwrap_or_default().into(),
                state: serde_json::from_str(&read(n, "state")).unwrap_or_default(),
            }
        });
        ShellRun {
            steps: steps.collect(),
            code: run.code,
            finished: dir.join("done").exists(),
            noise: run.out + &run.err,
        }
    }
}

/// A shell run: the prelude, the integration script, the steps and a mark that the last one ran.
const SCRIPT: &str = r#"{prelude}
source "$SHELLGAME_INIT"
{body}printf done >"$SG/done"
"#;

/// A step, then its status, whether the directory the shell is in still exists, `$PWD` and the saved
/// game. bash redirects with `exec`, not a group (a group parses an `alias` and its use together);
/// fish has no aliases to expand and cannot redirect itself, so it uses a group. The directory
/// exists when `$PWD` names it or, where `$PWD` cannot be resolved (`chmod a-x ..`), when it still
/// has a link: a deleted directory has none.
const BASH_STEP: &str = r#"exec 3>&1 4>&2 >"$SG/{n}.out" 2>"$SG/{n}.err"
{step}
__rc=$?
exec >&3 2>&4 3>&- 4>&-
__gone="$(command find . -maxdepth 0 -links 0 2>/dev/null)"
command test . -ef "$PWD" || [ -z "$__gone" ] && __ok=1 || __ok=0
printf '%s %s %s' "$__rc" "$__ok" "$PWD" >"$SG/{n}.info"
command cp -- "$SHELLGAME_STATE_DIR/state.json" "$SG/{n}.state" 2>/dev/null || :
"#;
const FISH_STEP: &str = r#"begin
{step}
end >"$SG/{n}.out" 2>"$SG/{n}.err"
set __rc $status
set __gone (command find . -maxdepth 0 -links 0 2>/dev/null)
command test . -ef "$PWD"; or test -z "$__gone"; and set __ok 1; or set __ok 0
printf '%s %s %s' $__rc $__ok "$PWD" >"$SG/{n}.info"
command cp -- "$SHELLGAME_STATE_DIR/state.json" "$SG/{n}.state" 2>/dev/null
"#;

impl Drop for World {
    /// Levels take permissions away: the game's own `remove_tree` makes everything removable.
    fn drop(&mut self) {
        [&self.root, &self.out].iter().for_each(|dir| drop(remove_tree(dir)));
    }
}

/// What one step of a shell run did.
#[derive(Debug)]
pub struct Step {
    pub cmd: String,
    pub out: String,
    pub err: String,
    /// `$?` after the step; negative when the shell never got through it.
    pub status: i32,
    /// The shell's logical `$PWD`, whether that directory still exists, and the saved game after
    /// the step (`Null` before there is one).
    pub pwd: PathBuf,
    pub alive: bool,
    pub state: Value,
}

/// The shell's exit status, whether the last line of the script ran, and what the shell printed
/// outside the steps.
#[derive(Debug)]
pub struct ShellRun {
    pub steps: Vec<Step>,
    pub code: i32,
    pub finished: bool,
    pub noise: String,
}
