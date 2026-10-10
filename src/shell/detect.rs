//! Choosing which shell the game runs in.
//!
//! The player usually types `shellgame` in bash or fish, and the game should start the same
//! kind of shell. [`detect`] decides in this order:
//!
//! 1. the `--shell` override (`bash` or `fish`, case-insensitive),
//! 2. the nearest `bash`, `fish` or `zsh` among the parent processes, up to 25 levels. This also
//!    sees through wrappers such as `make`, `uv run` or `sudo`, and ignores transient `sh -c`
//!    shells. Linux reads `/proc`, other systems ask `ps`,
//! 3. the basename of `$SHELL`.
//!
//! Only bash and fish are supported, so anything else (zsh, a missing `$SHELL`, ...) means bash.

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::process::Command;

/// The shells the game can run in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellKind {
    /// GNU bash, also the fallback for every unsupported shell. Must work with bash 3.2.
    Bash,
    /// The friendly interactive shell.
    Fish,
}

impl ShellKind {
    /// Parses an explicit shell choice: `bash` or `fish`, ignoring case and surrounding space.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "fish" => Some(Self::Fish),
            _ => None,
        }
    }

    /// The shell's name, which is also the executable that is started.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Fish => "fish",
        }
    }
}

impl fmt::Display for ShellKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// How many parent processes are examined before giving up.
const MAX_ANCESTORS: usize = 25;

/// Picks the shell to launch. See the [module documentation](self) for the order of the checks.
///
/// `forced` is the value of `--shell`; a value that is not `bash` or `fish` is ignored.
/// `env_shell` is the value of `$SHELL`. Both are passed in, never read from the environment, so
/// callers and tests stay in control.
#[must_use]
pub fn detect(forced: Option<&str>, env_shell: Option<&OsStr>) -> ShellKind {
    if let Some(kind) = forced.and_then(ShellKind::from_name) {
        return kind;
    }
    let parent = std::os::unix::process::parent_id();
    resolve(nearest_shell(&SystemProcesses, parent), env_shell)
}

/// A shell family found while walking up the process tree. zsh is recognised only so that the
/// walk stops at it: a player in zsh must not be moved to the fish that started that zsh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seen {
    Bash,
    Fish,
    Zsh,
}

/// Maps a process name to a shell family. Accepts a bare name (Linux `comm`), a path and a
/// login-shell name such as `-zsh` (macOS `ps`).
fn classify(comm: &str) -> Option<Seen> {
    let base = comm.trim().rsplit('/').next().unwrap_or_default();
    let base = base.strip_prefix('-').unwrap_or(base);
    match base.to_ascii_lowercase().as_str() {
        "bash" => Some(Seen::Bash),
        "fish" => Some(Seen::Fish),
        "zsh" => Some(Seen::Zsh),
        _ => None,
    }
}

/// Combines the parent-process result with `$SHELL`. Unsupported shells fall back to bash.
fn resolve(ancestor: Option<Seen>, env_shell: Option<&OsStr>) -> ShellKind {
    let seen = ancestor.or_else(|| env_shell.and_then(|shell| classify(&shell.to_string_lossy())));
    match seen {
        Some(Seen::Fish) => ShellKind::Fish,
        Some(Seen::Bash | Seen::Zsh) | None => ShellKind::Bash,
    }
}

/// The two facts about a process that the walk needs.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessInfo {
    /// Parent process id.
    ppid: u32,
    /// Executable name as reported by the system (not the full command line).
    comm: String,
}

/// Source of process information; a trait so the walk can be tested with a fake process tree.
trait ProcessTable {
    fn lookup(&self, pid: u32) -> Option<ProcessInfo>;
}

/// Walks up from `first` and returns the nearest shell, if any.
fn nearest_shell(table: &dyn ProcessTable, first: u32) -> Option<Seen> {
    let mut pid = first;
    for _ in 0..MAX_ANCESTORS {
        let info = table.lookup(pid)?;
        if let Some(seen) = classify(&info.comm) {
            return Some(seen);
        }
        // pid 0 and 1 are the kernel and init; a process that is its own parent ends a cycle.
        if info.ppid <= 1 || info.ppid == pid {
            return None;
        }
        pid = info.ppid;
    }
    None
}

/// The real process table of this machine.
struct SystemProcesses;

impl ProcessTable for SystemProcesses {
    fn lookup(&self, pid: u32) -> Option<ProcessInfo> {
        if cfg!(target_os = "linux") {
            // `ps` is only a fallback for sandboxes without a readable /proc.
            proc_lookup(pid).or_else(|| ps_lookup(pid))
        } else {
            ps_lookup(pid)
        }
    }
}

/// Reads `/proc/<pid>/comm` and the parent id from `/proc/<pid>/stat` (Linux).
fn proc_lookup(pid: u32) -> Option<ProcessInfo> {
    let comm = fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    Some(ProcessInfo {
        ppid: parse_stat_ppid(&stat)?,
        comm: comm.trim().to_owned(),
    })
}

/// Extracts the parent pid from the contents of `/proc/<pid>/stat`.
///
/// The format is `pid (comm) state ppid ...`. The name may itself contain spaces and
/// parentheses, so the fields are counted from the last `)`.
fn parse_stat_ppid(stat: &str) -> Option<u32> {
    let (_, after_name) = stat.rsplit_once(')')?;
    after_name.split_whitespace().nth(1)?.parse().ok()
}

/// Asks `ps` for the parent pid and executable name of one process (macOS and other systems
/// without a readable `/proc`).
fn ps_lookup(pid: u32) -> Option<ProcessInfo> {
    let output = Command::new("ps")
        .args(["-o", "ppid=,comm=", "-p"])
        .arg(pid.to_string())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_ps_line(&String::from_utf8_lossy(&output.stdout))
}

/// Parses `ps -o ppid=,comm=` output. The pid comes first because `comm` may contain spaces.
fn parse_ps_line(output: &str) -> Option<ProcessInfo> {
    let line = output.lines().next()?.trim_start();
    let (ppid, comm) = line.split_once(char::is_whitespace)?;
    let comm = comm.trim();
    if comm.is_empty() {
        return None;
    }
    Some(ProcessInfo {
        ppid: ppid.parse().ok()?,
        comm: comm.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ffi::OsString;

    use super::*;

    /// A fake process tree: pid -> (ppid, comm).
    struct Tree(HashMap<u32, ProcessInfo>);

    impl Tree {
        fn new(processes: &[(u32, u32, &str)]) -> Self {
            Self(
                processes
                    .iter()
                    .map(|&(pid, ppid, comm)| {
                        (
                            pid,
                            ProcessInfo {
                                ppid,
                                comm: comm.to_owned(),
                            },
                        )
                    })
                    .collect(),
            )
        }
    }

    impl ProcessTable for Tree {
        fn lookup(&self, pid: u32) -> Option<ProcessInfo> {
            self.0.get(&pid).cloned()
        }
    }

    fn os(value: &str) -> OsString {
        OsString::from(value)
    }

    #[test]
    fn forced_shell_wins_case_insensitively() {
        let zsh = os("/bin/zsh");
        assert_eq!(detect(Some("fish"), Some(&zsh)), ShellKind::Fish);
        assert_eq!(detect(Some("FISH"), None), ShellKind::Fish);
        assert_eq!(
            detect(Some(" Bash "), Some(&os("/usr/bin/fish"))),
            ShellKind::Bash
        );
    }

    #[test]
    fn shell_names_parse_only_for_supported_shells() {
        assert_eq!(ShellKind::from_name("bash"), Some(ShellKind::Bash));
        assert_eq!(ShellKind::from_name("Fish"), Some(ShellKind::Fish));
        assert_eq!(ShellKind::from_name("zsh"), None);
        assert_eq!(ShellKind::from_name(""), None);
        assert_eq!(ShellKind::Bash.to_string(), "bash");
        assert_eq!(ShellKind::Fish.name(), "fish");
    }

    #[test]
    fn process_names_are_classified_by_basename() {
        assert_eq!(classify("bash"), Some(Seen::Bash));
        assert_eq!(classify("-bash"), Some(Seen::Bash));
        assert_eq!(classify("/bin/bash"), Some(Seen::Bash));
        assert_eq!(classify("/opt/homebrew/bin/fish"), Some(Seen::Fish));
        assert_eq!(classify("-zsh"), Some(Seen::Zsh));
        assert_eq!(classify("FISH"), Some(Seen::Fish));
        assert_eq!(classify("/Applications/My Apps/fish"), Some(Seen::Fish));
        for other in ["sh", "dash", "make", "uv", "bash-wrapper", "fishd", "", "-"] {
            assert_eq!(classify(other), None, "{other:?}");
        }
    }

    #[test]
    fn env_shell_is_used_when_no_ancestor_is_a_shell() {
        assert_eq!(resolve(None, Some(&os("/usr/bin/fish"))), ShellKind::Fish);
        assert_eq!(resolve(None, Some(&os("/bin/bash"))), ShellKind::Bash);
        assert_eq!(resolve(None, Some(&os("fish"))), ShellKind::Fish);
    }

    #[test]
    fn unsupported_or_missing_shells_mean_bash() {
        assert_eq!(resolve(None, Some(&os("/bin/zsh"))), ShellKind::Bash);
        assert_eq!(resolve(None, Some(&os("/usr/bin/nu"))), ShellKind::Bash);
        assert_eq!(resolve(None, Some(&os(""))), ShellKind::Bash);
        assert_eq!(resolve(None, None), ShellKind::Bash);
        assert_eq!(
            resolve(Some(Seen::Zsh), Some(&os("/usr/bin/fish"))),
            ShellKind::Bash
        );
    }

    #[test]
    fn a_shell_ancestor_beats_env_shell() {
        assert_eq!(
            resolve(Some(Seen::Fish), Some(&os("/bin/bash"))),
            ShellKind::Fish
        );
        assert_eq!(
            resolve(Some(Seen::Bash), Some(&os("/usr/bin/fish"))),
            ShellKind::Bash
        );
    }

    #[test]
    fn walk_skips_wrappers_and_finds_the_nearest_shell() {
        // shellgame <- make <- uv <- fish <- bash (the outer one must not win)
        let tree = Tree::new(&[
            (400, 300, "make"),
            (300, 200, "uv"),
            (200, 100, "fish"),
            (100, 50, "bash"),
            (50, 1, "sshd"),
        ]);
        assert_eq!(nearest_shell(&tree, 400), Some(Seen::Fish));
    }

    #[test]
    fn walk_stops_at_zsh_instead_of_reaching_an_outer_fish() {
        let tree = Tree::new(&[(300, 200, "zsh"), (200, 100, "fish"), (100, 1, "init")]);
        assert_eq!(nearest_shell(&tree, 300), Some(Seen::Zsh));
    }

    #[test]
    fn walk_ends_at_init_unknown_pids_and_cycles() {
        let tree = Tree::new(&[(300, 200, "make"), (200, 1, "sshd")]);
        assert_eq!(
            nearest_shell(&tree, 300),
            None,
            "stops once the parent is init"
        );
        assert_eq!(nearest_shell(&tree, 999), None, "unknown pid");

        let cycle = Tree::new(&[(30, 20, "a"), (20, 30, "b")]);
        assert_eq!(
            nearest_shell(&cycle, 30),
            None,
            "bounded even for a cyclic table"
        );

        let own_parent = Tree::new(&[(30, 30, "a")]);
        assert_eq!(nearest_shell(&own_parent, 30), None);
    }

    #[test]
    fn walk_gives_up_after_25_levels() {
        // A chain of 40 wrappers with a fish at the very top: too deep to be found.
        let mut processes: Vec<(u32, u32, &str)> =
            (1000..1040).map(|pid| (pid, pid + 1, "wrapper")).collect();
        processes.push((1040, 500, "fish"));
        let tree = Tree::new(&processes);
        assert_eq!(nearest_shell(&tree, 1000), None);

        // 25 levels are enough: pids 1000..=1023 are wrappers and 1024 is the 25th process.
        let mut processes: Vec<(u32, u32, &str)> =
            (1000..1024).map(|pid| (pid, pid + 1, "wrapper")).collect();
        processes.push((1024, 500, "bash"));
        let tree = Tree::new(&processes);
        assert_eq!(nearest_shell(&tree, 1000), Some(Seen::Bash));
    }

    #[test]
    fn stat_parsing_counts_fields_after_the_last_parenthesis() {
        assert_eq!(
            parse_stat_ppid("1234 (bash) S 1200 1234 1234 34816 1234 4194304"),
            Some(1200)
        );
        // The process name may contain spaces and parentheses.
        assert_eq!(
            parse_stat_ppid("77 (my (weird) name) R 4242 77 77 0"),
            Some(4242)
        );
        assert_eq!(parse_stat_ppid("77 (x) S"), None, "no ppid field");
        assert_eq!(parse_stat_ppid("no parentheses at all"), None);
        assert_eq!(parse_stat_ppid("77 (x) S notanumber 1"), None);
    }

    #[test]
    fn ps_output_is_parsed_pid_first() {
        let info = parse_ps_line("  4242 /bin/zsh\n").unwrap();
        assert_eq!(
            info,
            ProcessInfo {
                ppid: 4242,
                comm: "/bin/zsh".into()
            }
        );

        // comm may contain spaces, so only the first word is the pid.
        let info =
            parse_ps_line("1 /Applications/Visual Studio Code.app/Contents/MacOS/Electron\n")
                .unwrap();
        assert_eq!(info.ppid, 1);
        assert_eq!(
            info.comm,
            "/Applications/Visual Studio Code.app/Contents/MacOS/Electron"
        );

        assert_eq!(parse_ps_line("  300 -bash").unwrap().comm, "-bash");
        assert_eq!(parse_ps_line(""), None);
        assert_eq!(parse_ps_line("abc bash"), None);
        assert_eq!(parse_ps_line("123"), None);
    }

    #[test]
    fn real_process_lookups_agree_with_the_standard_library() {
        let me = std::process::id();
        let parent = std::os::unix::process::parent_id();

        // Both sources are optional (a sandbox may hide /proc or lack `ps`), but whichever one
        // works must report the true parent.
        if let Some(info) = proc_lookup(me) {
            assert_eq!(info.ppid, parent);
            assert!(!info.comm.is_empty());
        }
        if let Some(info) = ps_lookup(me) {
            assert_eq!(info.ppid, parent);
            assert!(!info.comm.is_empty());
        }

        // Whatever the process tree looks like, the walk terminates with a usable answer.
        assert!(matches!(
            detect(None, None),
            ShellKind::Bash | ShellKind::Fish
        ));
    }
}
