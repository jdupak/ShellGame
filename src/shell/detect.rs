//! Which shell the game starts: the `--shell` override, else the nearest bash, fish or zsh among the
//! parent processes (this sees through `make`, `sudo`, ...), else the basename of `$SHELL`. Only bash
//! and fish are supported, so zsh and anything unknown mean bash.

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::process::Command;

/// The shells the game can run in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellKind {
    Bash,
    Fish,
}

impl ShellKind {
    /// `bash` or `fish`, ignoring case.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "fish" => Some(Self::Fish),
            _ => None,
        }
    }

    /// The name, which is also the executable that is started.
    pub fn name(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Fish => "fish",
        }
    }

    /// The integration script that is sourced into the game shell. It is static: everything that
    /// varies reaches the shell through the environment, so nothing is ever quoted into script text.
    pub fn script(self) -> &'static str {
        match self {
            Self::Bash => include_str!("bash.sh"),
            Self::Fish => include_str!("fish.fish"),
        }
    }
}

impl fmt::Display for ShellKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Picks the shell. `forced` is `--shell` (clap has checked it), `env_shell` is `$SHELL`.
pub fn detect(forced: Option<&str>, env_shell: Option<&OsStr>) -> ShellKind {
    forced.and_then(ShellKind::from_name).unwrap_or_else(|| {
        // zsh is recognised only so that the walk stops there: a player in zsh must not get the
        // fish that started that zsh.
        let seen = ancestor_shell()
            .or_else(|| env_shell.and_then(|shell| shell_name(&shell.to_string_lossy())));
        ShellKind::from_name(seen.unwrap_or_default()).unwrap_or(ShellKind::Bash)
    })
}

/// `bash`, `fish` or `zsh` for a process name, a path or a login-shell name such as `-zsh`.
fn shell_name(comm: &str) -> Option<&'static str> {
    let base = comm.trim().rsplit('/').next().unwrap_or_default();
    let base = base.strip_prefix('-').unwrap_or(base);
    ["bash", "fish", "zsh"].into_iter().find(|name| name.eq_ignore_ascii_case(base))
}

/// The nearest shell among the parent processes, looking at most 25 levels up.
fn ancestor_shell() -> Option<&'static str> {
    let mut pid = std::os::unix::process::parent_id();
    for _ in 0..25 {
        let (parent, comm) = parent_and_name(pid)?;
        if let Some(shell) = shell_name(&comm) {
            return Some(shell);
        }
        // pid 0 and 1 are the kernel and init; a process that is its own parent ends a cycle.
        if parent <= 1 || parent == pid {
            return None;
        }
        pid = parent;
    }
    None
}

/// The parent pid and executable name of `pid`: from `/proc` on Linux, from `ps` elsewhere (and in
/// sandboxes without a readable `/proc`).
fn parent_and_name(pid: u32) -> Option<(u32, String)> {
    let from_proc = || {
        let comm = fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        // `pid (name) state ppid ...`: the name may contain spaces and parentheses.
        let parent = stat.rsplit_once(')')?.1.split_whitespace().nth(1)?;
        Some((parent.parse().ok()?, comm.trim().to_owned()))
    };
    from_proc().or_else(|| {
        let output =
            Command::new("ps").args(["-o", "ppid=,comm=", "-p", &pid.to_string()]).output().ok()?;
        // `comm` may contain spaces, so only the first word is the pid.
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        let (parent, comm) = text.lines().next()?.trim_start().split_once(char::is_whitespace)?;
        Some((parent.parse().ok()?, comm.trim().to_owned()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shells_are_recognised_by_name_path_or_login_name() {
        assert_eq!(shell_name("-zsh"), Some("zsh"));
        assert_eq!(shell_name("/opt/homebrew/bin/Fish"), Some("fish"));
        assert_eq!(shell_name("bash"), Some("bash"));
        assert_eq!(shell_name("bash-wrapper"), None);
        assert_eq!(detect(Some("FISH"), None), ShellKind::Fish);
    }
}
