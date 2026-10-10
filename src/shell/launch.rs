//! Starting the wrapped game shell.
//!
//! [`launch`] renders the integration script into a temporary file, writes the fd-hook launcher
//! into the state directory and runs the player's shell with the environment documented in the
//! [parent module](super), with inherited stdin, stdout and stderr. It returns when the shell
//! exits; the shell's own exit status is data, not an error.
//!
//! # Terminal and signals
//!
//! The launcher does not touch signals. An interactive bash or fish makes itself the foreground
//! process group of the terminal, so Ctrl-C reaches the game shell and never this process, and
//! hands the terminal back when it exits, exactly as for any nested shell.

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{self, Path, PathBuf};
use std::process::{Command, ExitStatus};

use tempfile::NamedTempFile;

use super::detect::ShellKind;
use super::render::{ScriptOptions, render};
use super::{ENV_AUTO_STARTED, ENV_BIN, ENV_FD_HOOK, ENV_INIT, ENV_WRAPPER};

/// File name of the fd-hook launcher inside the state directory.
pub const FD_LAUNCHER_NAME: &str = "fd-hook.sh";

/// Content of the fd-hook launcher. It is static: the binary is found through the environment, so
/// no path needs quoting and the file never changes between runs, versions or binaries.
const FD_LAUNCHER: &str = "#!/bin/sh\nexec \"$SHELLGAME_BIN\" fd-hook \"$@\"\n";

/// The fish start-up command. It silences the greeting and sources the script by the path in the
/// environment (fish does not split variables, so the path needs no quoting either).
const FISH_INIT_COMMAND: &str = "function fish_greeting; end; source $SHELLGAME_INIT";

/// What [`launch`] needs to know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchOptions {
    /// The shell to start.
    pub shell: ShellKind,
    /// The game binary, usually `std::env::current_exe()`. A relative path is made absolute.
    pub binary: PathBuf,
    /// Define the developer shortcuts in the shell.
    pub devmode: bool,
    /// The game's state directory, where the fd-hook launcher is kept.
    pub state_dir: PathBuf,
}

/// Writes the fd-hook launcher, `<dir>/fd-hook.sh`, and returns its path.
///
/// The launcher is `#!/bin/sh` plus `exec "$SHELLGAME_BIN" fd-hook "$@"`, mode 0700. `exec` keeps
/// the redirected file descriptors of the caller (`script.sh &> /dev/null`), which is exactly what
/// the fd evidence of the level inspects. It lives in the state directory rather than in the
/// temporary directory, which may be mounted `noexec`.
///
/// `dir` is created when missing and made absolute. The file is replaced atomically, so game
/// shells starting at the same time never see a partial file.
///
/// # Errors
///
/// Fails when the directory or the file cannot be created.
pub fn write_fd_launcher(dir: &Path) -> io::Result<PathBuf> {
    let dir = path::absolute(dir)?;
    fs::create_dir_all(&dir)?;

    let mut staged = tempfile::Builder::new()
        .prefix(".fd-hook-")
        .tempfile_in(&dir)?;
    staged.write_all(FD_LAUNCHER.as_bytes())?;
    staged
        .as_file()
        .set_permissions(fs::Permissions::from_mode(0o700))?;

    let target = dir.join(FD_LAUNCHER_NAME);
    staged.persist(&target).map_err(|error| error.error)?;
    Ok(target)
}

/// Starts the game shell and waits for it to exit.
///
/// # Errors
///
/// Fails when the script or the launcher cannot be written or the shell cannot be started. A
/// shell that ran and exited with a non-zero status is *not* an error: the status is returned.
pub fn launch(options: &LaunchOptions) -> io::Result<ExitStatus> {
    launch_with(options, |_| {})
}

/// Like [`launch`], but lets the caller adjust the [`Command`] (stdio, extra environment) after
/// the game's own environment is in place and before the shell starts. Tests use it to run the
/// real launch path without a terminal.
///
/// # Errors
///
/// See [`launch`].
pub fn launch_with(
    options: &LaunchOptions,
    configure: impl FnOnce(&mut Command),
) -> io::Result<ExitStatus> {
    let binary = path::absolute(&options.binary)?;
    let script = render(
        options.shell,
        &ScriptOptions {
            devmode: options.devmode,
        },
    );
    let init = write_init_script(options.shell, &script)?;
    let fd_hook = write_fd_launcher(&options.state_dir)?;

    let mut command = build_command(options.shell, &binary, init.path(), &fd_hook);
    configure(&mut command);
    let status = command.status();

    // The script is only sourced, never executed, so a `noexec` temporary directory is fine. It
    // is removed as soon as the shell is gone, also when the shell could not be started.
    drop(init);
    status
}

/// Writes the rendered script to a private temporary file in the system temporary directory.
fn write_init_script(shell: ShellKind, script: &str) -> io::Result<NamedTempFile> {
    let mut file = tempfile::Builder::new()
        .prefix("shellgame-init-")
        .suffix(match shell {
            ShellKind::Bash => ".bash",
            ShellKind::Fish => ".fish",
        })
        .tempfile()?;
    file.write_all(script.as_bytes())?;
    file.flush()?;
    Ok(file)
}

/// Builds the shell command line and its environment. Nothing is spawned here.
fn build_command(shell: ShellKind, binary: &Path, init: &Path, fd_hook: &Path) -> Command {
    let mut command = Command::new(shell.name());
    match shell {
        // The script is the rcfile. Neither `--norc` (it would disable the rcfile) nor
        // `--noprofile` (it would drop the player's PATH and environment setup) may be used.
        ShellKind::Bash => command.arg("--rcfile").arg(init).arg("-i"),
        ShellKind::Fish => command.arg("--init-command").arg(FISH_INIT_COMMAND),
    };
    command
        .env(ENV_WRAPPER, "1")
        .env(ENV_BIN, binary)
        .env(ENV_INIT, init)
        .env(ENV_FD_HOOK, fd_hook)
        // macOS bash 3.2 otherwise prints a banner recommending zsh.
        .env("BASH_SILENCE_DEPRECATION_WARNING", "1")
        // The new shell must autostart the game, even when launched from inside another game shell.
        .env_remove(ENV_AUTO_STARTED);
    command
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::ffi::OsString;

    use super::*;

    fn env_of(command: &Command) -> HashMap<String, Option<String>> {
        command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.map(|value| value.to_string_lossy().into_owned()),
                )
            })
            .collect()
    }

    fn args_of(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    fn some(value: &str) -> Option<String> {
        Some(value.to_owned())
    }

    #[test]
    fn fd_launcher_is_static_and_executable() {
        let dir = tempfile::tempdir().unwrap();

        let path = write_fd_launcher(dir.path()).unwrap();

        assert_eq!(path, dir.path().join("fd-hook.sh"));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "#!/bin/sh\nexec \"$SHELLGAME_BIN\" fd-hook \"$@\"\n"
        );
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }

    #[test]
    fn fd_launcher_creates_missing_directories_and_leaves_no_staging_files() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a/b/state");

        let path = write_fd_launcher(&nested).unwrap();

        assert_eq!(path, nested.join("fd-hook.sh"));
        let entries: Vec<_> = fs::read_dir(&nested)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(entries, [OsString::from("fd-hook.sh")]);
    }

    #[test]
    fn fd_launcher_repairs_a_stale_or_damaged_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("fd-hook.sh");
        // What an older version, a full disk or a curious player may have left behind.
        fs::write(&path, "garbage").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();

        let written = write_fd_launcher(dir.path()).unwrap();

        assert_eq!(written, path);
        assert_eq!(fs::read_to_string(&path).unwrap(), FD_LAUNCHER);
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }

    #[test]
    fn bash_is_started_with_the_script_as_rcfile_and_no_suppression_flags() {
        let command = build_command(
            ShellKind::Bash,
            Path::new("/opt/game/shellgame"),
            Path::new("/tmp/init.bash"),
            Path::new("/state/fd-hook.sh"),
        );

        assert_eq!(command.get_program(), "bash");
        let args = args_of(&command);
        assert_eq!(args, ["--rcfile", "/tmp/init.bash", "-i"]);
        for forbidden in ["--norc", "--noprofile"] {
            assert!(!args.iter().any(|arg| arg == forbidden), "{forbidden}");
        }
    }

    #[test]
    fn fish_greeting_is_silenced_and_the_script_is_sourced_from_the_environment() {
        let command = build_command(
            ShellKind::Fish,
            Path::new("/opt/game/shellgame"),
            Path::new("/tmp/init.fish"),
            Path::new("/state/fd-hook.sh"),
        );

        assert_eq!(command.get_program(), "fish");
        assert_eq!(
            args_of(&command),
            [
                "--init-command",
                "function fish_greeting; end; source $SHELLGAME_INIT"
            ]
        );
    }

    #[test]
    fn environment_carries_every_path_so_nothing_is_quoted_into_the_script() {
        for shell in [ShellKind::Bash, ShellKind::Fish] {
            let command = build_command(
                shell,
                Path::new("/opt/My Apps/shellgame"),
                Path::new("/tmp/init script"),
                Path::new("/state dir/fd-hook.sh"),
            );
            let env = env_of(&command);

            assert_eq!(env["SHELLGAME_WRAPPER"], some("1"));
            assert_eq!(env["SHELLGAME_BIN"], some("/opt/My Apps/shellgame"));
            assert_eq!(env["SHELLGAME_INIT"], some("/tmp/init script"));
            assert_eq!(env["SHELLGAME_FD_HOOK"], some("/state dir/fd-hook.sh"));
            assert_eq!(env["BASH_SILENCE_DEPRECATION_WARNING"], some("1"));
            assert_eq!(
                env["SHELLGAME_AUTO_STARTED"], None,
                "an inherited marker must be removed so the new shell autostarts"
            );
        }
    }

    #[test]
    fn nothing_else_is_added_to_the_inherited_environment() {
        let command = build_command(
            ShellKind::Bash,
            Path::new("/b"),
            Path::new("/i"),
            Path::new("/f"),
        );
        let mut names: Vec<_> = env_of(&command).into_keys().collect();
        names.sort();

        assert_eq!(
            names,
            [
                "BASH_SILENCE_DEPRECATION_WARNING",
                "SHELLGAME_AUTO_STARTED",
                "SHELLGAME_BIN",
                "SHELLGAME_FD_HOOK",
                "SHELLGAME_INIT",
                "SHELLGAME_WRAPPER",
            ]
        );
    }

    #[test]
    fn a_shell_that_cannot_start_is_an_error_and_leaves_no_script_behind() {
        let state = tempfile::tempdir().unwrap();
        let empty_path = tempfile::tempdir().unwrap();
        let options = LaunchOptions {
            shell: ShellKind::Bash,
            binary: PathBuf::from("relative/shellgame"),
            devmode: false,
            state_dir: state.path().to_owned(),
        };

        let mut seen = None;
        let result = launch_with(&options, |command| {
            seen = Some(env_of(command));
            // With an empty PATH there is no `bash` to execute.
            command.env("PATH", empty_path.path());
        });

        let error = result.expect_err("bash cannot be found");
        assert_eq!(error.kind(), io::ErrorKind::NotFound);

        let env = seen.expect("the closure ran before the spawn");
        let init = PathBuf::from(env["SHELLGAME_INIT"].clone().unwrap());
        assert!(
            !init.exists(),
            "the temporary script must be removed: {init:?}"
        );
        assert!(init.is_absolute());

        let binary = PathBuf::from(env["SHELLGAME_BIN"].clone().unwrap());
        assert!(
            binary.is_absolute(),
            "a relative binary path must be made absolute"
        );
        assert!(binary.ends_with("relative/shellgame"));

        let fd_hook = PathBuf::from(env["SHELLGAME_FD_HOOK"].clone().unwrap());
        assert_eq!(fd_hook, state.path().join("fd-hook.sh"));
        assert!(
            fd_hook.exists(),
            "the launcher stays in the state directory"
        );
    }
}
