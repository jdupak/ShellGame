//! The process environment, read once in `main` and passed down, so nothing below reads
//! `std::env` or the working directory. An empty variable counts as unset.

use std::ffi::OsString;
use std::path::{self, PathBuf};

use crate::game::paths::canonical;
use crate::shell::{ENV_PENDING, ENV_SANDBOX, ENV_WORKSPACE};

#[derive(Clone, Debug)]
pub struct Env {
    /// The logical `$HOME`.
    pub home: PathBuf,
    pub user: String,
    /// The player's directory, canonical (`getcwd` resolves every symlink, and unlike
    /// `canonicalize` it works when a parent lacks `x`); `None` when it no longer exists.
    pub cwd: Option<PathBuf>,
    /// Where `state.json` lives: `$SHELLGAME_STATE_DIR`, `$XDG_CONFIG_HOME/shellgame` or `~/.config/shellgame`.
    pub state_dir: PathBuf,
    /// The workspace of a new game: `$SHELLGAME_WORKSPACE` or `/tmp/shellgame-<user>`.
    pub workspace: PathBuf,
    /// Running inside the game shell, which replays the directives on stderr: the launcher sets
    /// `SHELLGAME_SANDBOX`, nothing else does.
    pub wrapped: bool,
    /// `landlock` or `off`, set by the launcher; absent outside the game shell.
    pub sandbox: Option<String>,
    /// A command typed outside the game shell, left by the launcher for the autostart: the subcommand
    /// and its arguments joined by U+001F.
    pub pending: Option<String>,
    /// `$SHELL`, for shell detection.
    pub shell: Option<OsString>,
    pub update_url: Option<String>,
    /// This binary, canonical.
    pub exe: Option<PathBuf>,
}

impl Env {
    pub fn from_process() -> Env {
        let var = |name: &str| std::env::var_os(name).filter(|value| !value.is_empty());
        let text = |name: &str| var(name).map(|value| value.to_string_lossy().into_owned());
        let home = var("HOME")
            .map(PathBuf::from)
            .or_else(std::env::home_dir)
            .unwrap_or_else(|| "/".into());
        let user = text("USER").unwrap_or_else(|| "player".into());
        let absolute = |dir: PathBuf| path::absolute(&dir).unwrap_or(dir);
        let state_dir = var("SHELLGAME_STATE_DIR")
            .map(PathBuf::from)
            .or_else(|| var("XDG_CONFIG_HOME").map(|xdg| PathBuf::from(xdg).join("shellgame")))
            .unwrap_or_else(|| home.join(".config/shellgame"));
        let workspace = var(ENV_WORKSPACE)
            .map(PathBuf::from)
            .unwrap_or_else(|| format!("/tmp/shellgame-{user}").into());
        let sandbox = text(ENV_SANDBOX);
        Env {
            cwd: std::env::current_dir().ok(),
            state_dir: absolute(state_dir),
            workspace: absolute(workspace),
            wrapped: sandbox.is_some(),
            sandbox,
            pending: text(ENV_PENDING),
            shell: var("SHELL"),
            update_url: text("SHELLGAME_UPDATE_URL"),
            exe: std::env::current_exe().ok().and_then(|exe| exe.canonicalize().ok()),
            home,
            user,
        }
    }

    /// The state directory stays writable inside the sandbox, so it must not be `$HOME` or above it.
    pub fn check(&self) -> Result<(), String> {
        if canonical(&self.home).starts_with(canonical(&self.state_dir)) {
            let what = "Adresář pro uložení hry nesmí být domovský adresář ani adresář nad ním";
            return Err(format!("{what}: {}", self.state_dir.display()));
        }
        Ok(())
    }
}
