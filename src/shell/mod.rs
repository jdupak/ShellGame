//! The shell integration: how the game runs inside a wrapped bash or fish and changes it.
//!
//! The game is a short-lived process, but the player lives in a long-running shell. A child
//! process cannot change the directory or the environment of its parent, so the game starts a
//! *game shell* of its own, and a small script sourced into that shell (`bash.sh`, `fish.fish`)
//! connects the two. This module owns both halves of the connection: the shell scripts and the
//! Rust code that starts the shell and speaks the protocol.
//!
//! | module                 | responsibility                                                  |
//! |------------------------|-----------------------------------------------------------------|
//! | [`detect`](mod@detect) | which shell to start ([`ShellKind`], `--shell`, parent processes, `$SHELL`) |
//! | [`render`](mod@render) | the integration script text ([`render_bash`], [`render_fish`])  |
//! | [`launch`](mod@launch) | starting the shell with the right environment ([`launch()`](fn@launch)) |
//! | [`protocol`]           | the directives the binary writes for the shell ([`Directives`]) |
//!
//! # Contract between the binary and the shell scripts
//!
//! ## Environment
//!
//! The launcher sets these with `Command::env`. They are never pasted into script text, so no
//! path is ever quoted.
//!
//! | variable                           | value                                              |
//! |------------------------------------|----------------------------------------------------|
//! | `SHELLGAME_WRAPPER`                | `1`: the binary runs inside the game shell         |
//! | `SHELLGAME_BIN`                    | absolute path of the binary                        |
//! | `SHELLGAME_INIT`                   | path of the rendered integration script            |
//! | `SHELLGAME_FD_HOOK`                | path of the fd-hook launcher                       |
//! | `BASH_SILENCE_DEPRECATION_WARNING` | `1`: no zsh banner from the macOS system bash 3.2  |
//!
//! An inherited `SHELLGAME_AUTO_STARTED` is removed, so the new shell autostarts. The scripts
//! export `SHELLGAME_AUTO_STARTED=1` when they autostart. The binary itself exports
//! `SHELLGAME_WORKSPACE` (where evidence markers live; used by `pwd` and the deleted-cwd
//! relocation) and `SHELLGAME_LEVEL` through the protocol.
//!
//! ## The `shellgame` function
//!
//! `shellgame ARGS...` runs `"$SHELLGAME_BIN" ARGS...` with stdin and stdout on the terminal and
//! stderr captured in a temporary file, then replays that file line by line. A line that starts
//! with `__SHELLGAME_EXEC__` is a [directive](protocol) and is executed, in order; every other
//! line is written to stderr unchanged; a marker anywhere else in a line is plain text. The
//! function returns the binary's exit status. If the shell's current directory no longer exists,
//! the function first moves to `$SHELLGAME_WORKSPACE` (or `/tmp`).
//!
//! There is no special casing of command names. In particular `shellgame remove` closes the shell
//! because the binary emits the `exit` directive ([`Directives::exit`]).
//!
//! A directive `cd` moves with the shell's own `cd` implementation and never runs the `cd` hook
//! below, so teleports are not subject to level rules. In bash that is `builtin cd --`, in fish
//! the stock `cd` that the integration copies to `__shellgame_cd` before it defines its hook, so
//! `cd -`, `prevd` and `nextd` keep working after a teleport.
//!
//! ## The `cd` hook
//!
//! Both scripts define one static `cd` function; there is no per-level list. Before moving it
//! runs, directly and not through the `shellgame` function (so stderr is live and nothing is
//! relocated):
//!
//! ```text
//! "$SHELLGAME_BIN" cd-hook pre <pwd> <prev> -- <cd args...>
//! ```
//!
//! * `<pwd>` is `$PWD` before the move.
//! * `<prev>` is the directory `cd -` would move to: `$OLDPWD` in bash, and in fish (which never
//!   sets `OLDPWD`) the last entry of `$dirprev`, or of `$dirnext` right after a `cd -`. It is an
//!   empty argument when there is none.
//! * `<cd args...>` are all arguments of the `cd` call, verbatim: possibly none, `-`, `-P`, `--`,
//!   an empty string, or a directory that is literally named `pre`. The binary must parse this
//!   command line by hand, before any option parser.
//!
//! Exit status **3** means *blocked*: the function returns 1 without moving. The binary prints
//! the explanation itself, on stderr. Any other status (0, a crash, a missing binary) allows the
//! move, so a broken hook can never lock the player in. Then the function performs the real move
//! (bash: `builtin cd "$@"`, fish: the stock `cd` copy) and, only if that succeeded, runs
//!
//! ```text
//! "$SHELLGAME_BIN" cd-hook post <pwd before the move> <new pwd> -- <cd args...> >/dev/null 2>&1
//! ```
//!
//! where `<new pwd>` is `$PWD` after the move (never re-resolved). Its output and status are
//! ignored, and a failed move is never reported. The function returns the status of the move.
//!
//! The bash functions are written to survive `set -e` in the player's shell: a failing hook or a
//! failing binary must not end the shell before the move, the replay or the cleanup.
//!
//! ## The fd hook
//!
//! Level scripts execute `"$SHELLGAME_FD_HOOK" ARGS...`. The launcher file is written by
//! [`write_fd_launcher`] into the state directory (mode 0700) and consists of
//!
//! ```text
//! #!/bin/sh
//! exec "$SHELLGAME_BIN" fd-hook "$@"
//! ```
//!
//! `exec` keeps the descriptors the script was started with, so `fd-hook` sees whether stdout and
//! stderr were redirected to `/dev/null`.
//!
//! ## `pwd`
//!
//! Both scripts define `pwd` as `builtin pwd "$@"` (the logical path, in both shells) followed
//! by `touch "$SHELLGAME_WORKSPACE/.pwd_used"`, only when `pwd` succeeded and that variable is set
//! and names an existing directory. The workspace is never guessed.
//!
//! ## Dependencies and supported shells
//!
//! The scripts need `mktemp` without arguments, `base64 --decode` (GNU and macOS), `touch` and
//! `rm`. `bash.sh` runs on bash 3.2, the system bash of macOS, and exports `shellgame` with its
//! two helper functions so that child bash processes can call it too. `fish.fish` runs on fish
//! 3.3 and later and relies on nothing from fish 3.4 or 4 (for example `$(...)`). `tests/shell.rs`
//! runs the scripts in real shells; it was checked with bash 3.2.57 and 5.3, fish 3.3.1, 3.7.0 and
//! 4.8.1, and shellcheck 0.9.0 and 0.11. To try another version, put it first in `PATH`.

pub mod detect;
pub mod launch;
pub mod protocol;
pub mod render;

pub use detect::{ShellKind, detect};
pub use launch::{LaunchOptions, launch, launch_with, write_fd_launcher};
pub use protocol::{Directives, InvalidName};
pub use render::{ScriptOptions, render_bash, render_fish};

/// `1` inside the game shell; the binary only runs the game (and emits directives) when it is set.
pub const ENV_WRAPPER: &str = "SHELLGAME_WRAPPER";
/// Absolute path of the binary, set by the launcher.
pub const ENV_BIN: &str = "SHELLGAME_BIN";
/// Path of the rendered integration script, set by the launcher.
pub const ENV_INIT: &str = "SHELLGAME_INIT";
/// Path of the fd-hook launcher, set by the launcher.
pub const ENV_FD_HOOK: &str = "SHELLGAME_FD_HOOK";
/// Set by the scripts once the game autostarted; removed by the launcher.
pub const ENV_AUTO_STARTED: &str = "SHELLGAME_AUTO_STARTED";
/// The workspace directory, exported by the binary through the protocol.
pub const ENV_WORKSPACE: &str = "SHELLGAME_WORKSPACE";
/// The id of the current level, exported by the binary through the protocol.
pub const ENV_LEVEL: &str = "SHELLGAME_LEVEL";
