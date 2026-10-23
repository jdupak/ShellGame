//! Starting the game shell: writes the script, optionally confines the shell ([`crate::sandbox`])
//! and waits for it. Signals are not touched: an interactive shell makes itself the foreground
//! process group, so Ctrl-C reaches it and never this process.

use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use super::{ENV_AUTO_STARTED, ENV_BIN, ENV_INIT, ENV_SANDBOX, ShellKind};
use crate::{sandbox, state};

#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub shell: ShellKind,
    /// The game binary, usually `current_exe()`: an absolute path, since the shell runs it from
    /// every directory.
    pub binary: PathBuf,
    /// The game's state directory (absolute): holds the script and stays writable in the sandbox.
    pub state_dir: PathBuf,
    /// Confine the shell ([`sandbox::confine`]); without Landlock the launch fails rather than start
    /// unconfined. The launcher itself is never confined.
    pub sandbox: bool,
}

/// Starts the game shell and waits for it. The shell's own exit status is data, not an error.
/// `configure` may still adjust the command (stdio, extra environment) before the shell starts;
/// tests use it to run the real launch path without a terminal.
pub fn launch_with(
    options: &LaunchOptions,
    configure: impl FnOnce(&mut Command),
) -> io::Result<ExitStatus> {
    // Sourced once at startup; one fixed file per shell, replaced by a rename so that a game shell
    // starting at the same time never sees a partial file. In the state directory it is never left
    // behind in /tmp when a terminal closes.
    let name = format!("init.{}", options.shell.name());
    let init = state::replace(&options.state_dir, &name, options.shell.script().as_bytes())?;

    let mut command = Command::new(options.shell.name());
    match options.shell {
        // `--norc` would disable the script and `--noprofile` would drop the player's PATH setup.
        ShellKind::Bash => command.arg("--rcfile").arg(&init).arg("-i"),
        // No config files, universal variables or history writes: the sandbox forbids them anyway.
        ShellKind::Fish => command.args([
            "--no-config",
            "--init-command",
            "function fish_greeting; end; source $SHELLGAME_INIT",
        ]),
    };
    command
        .env(ENV_BIN, &options.binary)
        .env(ENV_INIT, &init)
        .env(ENV_SANDBOX, if options.sandbox { "landlock" } else { "off" })
        .env("TMPDIR", "/tmp")
        .env("BASH_SILENCE_DEPRECATION_WARNING", "1")
        // Even inside another game shell the new one must autostart.
        .env_remove(ENV_AUTO_STARTED);
    if options.sandbox {
        sandbox::confine(&mut command, &options.state_dir)?;
    }
    configure(&mut command);
    command.status()
}
