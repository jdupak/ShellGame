//! The process environment, read once.
//!
//! Everything the game needs to know about the process it runs in is gathered here, in `main`,
//! and passed down: the home directory, the user name, the working directory, where the state
//! lives, whether the binary runs inside the game shell, and a few knobs that tests and
//! developers use. No game logic reads `std::env` or the process working directory after this, so
//! every rule below this layer is a plain function of its inputs and tests never `chdir` or
//! `set_var`.
//!
//! | variable                  | field                | meaning                                         |
//! |---------------------------|----------------------|-------------------------------------------------|
//! | `HOME`                    | [`Env::home`]        | the logical home directory                      |
//! | `USER`                    | [`Env::user`]        | the player's name (`player` when unset)         |
//! | `SHELLGAME_STATE_DIR`, `XDG_CONFIG_HOME` | [`Env::state_dir`] | where `state.json` lives          |
//! | `SHELLGAME_WORKSPACE`     | [`Env::workspace_override`] | workspace of a *new* game                |
//! | `SHELLGAME_WRAPPER`       | [`Env::wrapped`]     | the binary runs inside the game shell           |
//! | `TMPDIR`                  | [`Env::tmpdir`]      | a second place `remove` may delete from         |
//! | `SHELL`                   | [`Env::shell`]       | the player's login shell, for shell detection   |
//! | `SHELLGAME_FORCE_SHELL`   | [`Env::force_shell`] | `bash` or `fish`: which game shell to start     |
//! | `SHELLGAME_UPDATE_URL`    | [`Env::update_url`]  | where `shellgame update` downloads from        |
//!
//! An empty value counts as unset for every variable, as it does in the Python version, where `""`
//! is falsy. `NO_COLOR` and friends are not read here: [`anstream`] handles them when output is
//! written.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::shell::{ENV_WORKSPACE, ENV_WRAPPER};
use crate::state;

/// The environment variable that names the state directory, ahead of `XDG_CONFIG_HOME`.
const ENV_STATE_DIR: &str = "SHELLGAME_STATE_DIR";
/// The environment variable that forces the game shell (`bash` or `fish`).
const ENV_FORCE_SHELL: &str = "SHELLGAME_FORCE_SHELL";
/// The environment variable that overrides the download location of `shellgame update`.
const ENV_UPDATE_URL: &str = "SHELLGAME_UPDATE_URL";
/// The user name when `USER` is not set.
const DEFAULT_USER: &str = "player";

/// What the game knows about the process it runs in. Built once by [`Env::from_process`] and passed
/// down by reference; tests build one with [`Env::new`] or a struct literal.
///
/// ```
/// use shellgame::env::Env;
///
/// let env = Env {
///     wrapped: true,
///     ..Env::new("/home/ada", "ada", "/home/ada/.config/shellgame")
/// };
/// assert_eq!(env.user, "ada");
/// assert!(env.wrapped);
/// assert_eq!(env.cwd, None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env {
    /// The logical `$HOME`, as the player's shell sees it.
    pub home: PathBuf,
    /// The player's name: `$USER`, or `player`.
    pub user: String,
    /// The player's working directory, canonical, or `None` when it no longer exists (the player
    /// deleted the directory they were standing in).
    pub cwd: Option<PathBuf>,
    /// The directory that holds `state.json`.
    pub state_dir: PathBuf,
    /// `$SHELLGAME_WORKSPACE`: the workspace of a *new* game. A running game keeps the workspace
    /// saved in its state.
    pub workspace_override: Option<OsString>,
    /// True when `$SHELLGAME_WRAPPER` is set: the binary runs inside the game shell, which replays
    /// its protocol directives.
    pub wrapped: bool,
    /// `$TMPDIR`. `shellgame remove` deletes workspaces below `/tmp` and below this directory.
    pub tmpdir: Option<PathBuf>,
    /// `$SHELL`, the last resort of shell detection.
    pub shell: Option<OsString>,
    /// `$SHELLGAME_FORCE_SHELL`: `bash` or `fish` (anything else is ignored by shell detection).
    pub force_shell: Option<String>,
    /// `$SHELLGAME_UPDATE_URL`: where `shellgame update` downloads the release archive from.
    pub update_url: Option<String>,
    /// This binary, canonical (`current_exe` may report a symlink path on macOS). `None` when the
    /// system cannot tell.
    pub exe: Option<PathBuf>,
}

impl Env {
    /// An environment with the given home, user and state directory, outside the game shell, with
    /// no working directory and every optional knob unset. Meant for tests; struct update syntax
    /// (`Env { wrapped: true, ..Env::new(..) }`) adjusts the rest.
    pub fn new(
        home: impl Into<PathBuf>,
        user: impl Into<String>,
        state_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            home: home.into(),
            user: user.into(),
            cwd: None,
            state_dir: state_dir.into(),
            workspace_override: None,
            wrapped: false,
            tmpdir: None,
            shell: None,
            force_shell: None,
            update_url: None,
            exe: None,
        }
    }

    /// Gather the environment of this process: the variables, the working directory and the path
    /// of this binary. Called once, from `main`.
    pub fn from_process() -> Self {
        // `$HOME` falls back to the passwd database (what Python's `Path.home()` does).
        let lookup = |name: &str| -> Option<OsString> {
            std::env::var_os(name)
                .filter(|value| !value.is_empty())
                .or_else(|| {
                    (name == "HOME")
                        .then(std::env::home_dir)
                        .flatten()
                        .map(PathBuf::into_os_string)
                })
        };
        let cwd = std::env::current_dir()
            .ok()
            .and_then(|dir| dir.canonicalize().ok());
        let exe = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.canonicalize().ok());
        Self::from_vars(lookup, cwd, exe)
    }

    /// Build an environment from a variable lookup and the two facts that are not variables.
    ///
    /// `lookup` returns the value of a variable; empty values are treated as unset here, so a
    /// lookup can simply wrap `std::env::var_os`. Without `HOME` the home directory is `/`.
    ///
    /// ```
    /// use std::ffi::OsString;
    /// use shellgame::env::Env;
    ///
    /// let vars = |name: &str| match name {
    ///     "HOME" => Some(OsString::from("/home/ada")),
    ///     "XDG_CONFIG_HOME" => Some(OsString::from("/home/ada/cfg")),
    ///     "SHELLGAME_WRAPPER" => Some(OsString::from("1")),
    ///     "USER" => Some(OsString::new()), // empty counts as unset
    ///     _ => None,
    /// };
    /// let env = Env::from_vars(vars, None, None);
    /// assert_eq!(env.user, "player");
    /// assert_eq!(env.state_dir, std::path::Path::new("/home/ada/cfg/shellgame"));
    /// assert!(env.wrapped);
    /// ```
    pub fn from_vars(
        lookup: impl Fn(&str) -> Option<OsString>,
        cwd: Option<PathBuf>,
        exe: Option<PathBuf>,
    ) -> Self {
        let get = |name: &str| lookup(name).filter(|value| !value.is_empty());
        let text = |name: &str| get(name).map(|value| value.to_string_lossy().into_owned());

        let home = get("HOME").map_or_else(|| PathBuf::from("/"), PathBuf::from);
        let state_dir = state::state_dir(
            get(ENV_STATE_DIR).as_deref(),
            get("XDG_CONFIG_HOME").as_deref(),
            &home,
        );
        Self {
            user: text("USER").unwrap_or_else(|| DEFAULT_USER.to_owned()),
            cwd,
            state_dir,
            workspace_override: get(ENV_WORKSPACE),
            wrapped: get(ENV_WRAPPER).is_some(),
            tmpdir: get("TMPDIR").map(PathBuf::from),
            shell: get("SHELL"),
            force_shell: text(ENV_FORCE_SHELL),
            update_url: text(ENV_UPDATE_URL),
            exe,
            home,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::Path;

    use super::*;

    fn env_of(vars: &[(&str, &str)]) -> Env {
        let vars: HashMap<String, OsString> = vars
            .iter()
            .map(|(name, value)| ((*name).to_owned(), OsString::from(value)))
            .collect();
        Env::from_vars(|name| vars.get(name).cloned(), None, None)
    }

    #[test]
    fn an_empty_environment_has_sane_defaults() {
        let env = env_of(&[]);
        assert_eq!(env.home, Path::new("/"));
        assert_eq!(env.user, "player");
        assert_eq!(env.state_dir, Path::new("/.config/shellgame"));
        assert_eq!(env.cwd, None);
        assert_eq!(env.workspace_override, None);
        assert!(!env.wrapped);
        assert_eq!(env.tmpdir.as_deref(), None);
        assert_eq!(env.shell, None);
        assert_eq!(env.force_shell, None);
        assert_eq!(env.update_url, None);
        assert_eq!(env.exe, None);
    }

    #[test]
    fn the_state_directory_follows_the_documented_lookup() {
        let home = ("HOME", "/home/ada");
        assert_eq!(
            env_of(&[home]).state_dir,
            Path::new("/home/ada/.config/shellgame")
        );
        assert_eq!(
            env_of(&[home, ("XDG_CONFIG_HOME", "/xdg")]).state_dir,
            Path::new("/xdg/shellgame")
        );
        assert_eq!(
            env_of(&[
                home,
                ("XDG_CONFIG_HOME", "/xdg"),
                ("SHELLGAME_STATE_DIR", "/explicit")
            ])
            .state_dir,
            Path::new("/explicit")
        );
    }

    #[test]
    fn empty_values_count_as_unset() {
        let env = env_of(&[
            ("HOME", ""),
            ("USER", ""),
            ("SHELLGAME_STATE_DIR", ""),
            ("XDG_CONFIG_HOME", ""),
            ("SHELLGAME_WORKSPACE", ""),
            ("SHELLGAME_WRAPPER", ""),
            ("TMPDIR", ""),
            ("SHELL", ""),
            ("SHELLGAME_FORCE_SHELL", ""),
            ("SHELLGAME_UPDATE_URL", ""),
        ]);
        assert_eq!(env, env_of(&[]));
    }

    #[test]
    fn every_knob_is_read() {
        let env = env_of(&[
            ("HOME", "/home/ada"),
            ("USER", "ada"),
            ("SHELLGAME_WORKSPACE", "/tmp/my game"),
            ("SHELLGAME_WRAPPER", "1"),
            ("TMPDIR", "/var/tmp"),
            ("SHELL", "/usr/bin/fish"),
            ("SHELLGAME_FORCE_SHELL", "fish"),
            ("SHELLGAME_UPDATE_URL", "file:///x.tar.gz"),
        ]);
        assert_eq!(env.home, Path::new("/home/ada"));
        assert_eq!(env.user, "ada");
        assert_eq!(env.workspace_override, Some(OsString::from("/tmp/my game")));
        assert!(env.wrapped);
        assert_eq!(env.tmpdir.as_deref(), Some(Path::new("/var/tmp")));
        assert_eq!(env.shell, Some(OsString::from("/usr/bin/fish")));
        assert_eq!(env.force_shell.as_deref(), Some("fish"));
        assert_eq!(env.update_url.as_deref(), Some("file:///x.tar.gz"));
    }

    #[test]
    fn the_working_directory_and_executable_are_passed_through() {
        let env = Env::from_vars(
            |_| None,
            Some(PathBuf::from("/work")),
            Some(PathBuf::from("/bin/shellgame")),
        );
        assert_eq!(env.cwd.as_deref(), Some(Path::new("/work")));
        assert_eq!(env.exe.as_deref(), Some(Path::new("/bin/shellgame")));
    }

    #[test]
    fn a_test_environment_is_outside_the_game_shell_and_has_no_cwd() {
        let env = Env::new("/home/ada", "ada", "/state");
        assert!(!env.wrapped);
        assert_eq!(env.cwd, None);
        assert_eq!(env.state_dir, Path::new("/state"));
    }

    #[test]
    fn from_process_reads_the_real_process() {
        // Whatever the harness environment is, the pieces must be consistent.
        let env = Env::from_process();
        assert!(env.home.is_absolute());
        assert!(!env.user.is_empty());
        if let Some(cwd) = &env.cwd {
            assert!(cwd.is_absolute());
        }
        assert!(env.exe.as_deref().is_some_and(Path::is_absolute));
    }
}
