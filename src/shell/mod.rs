//! The game shell: how the game runs inside a wrapped bash or fish and changes that shell.
//!
//! A process cannot change the directory or environment of its parent, so the game starts a *game
//! shell* of its own ([`launch_with`](fn@launch_with), confined by [`crate::sandbox`]) and sources
//! an integration script into it (`bash.sh`, `fish.fish`; [`ShellKind::script`]). The scripts hold no
//! game logic and know no paths: they forward events to the binary and replay what it asks for.
//!
//! # Environment
//!
//! Set by the launcher, never pasted into script text. An inherited `SHELLGAME_AUTO_STARTED` is
//! removed, so the new shell autostarts the game; the scripts export it as `1` when they do.
//!
//! | variable                           | value                                            |
//! |------------------------------------|--------------------------------------------------|
//! | `SHELLGAME_BIN`                    | absolute path of the binary                      |
//! | `SHELLGAME_INIT`                   | the script, `<state dir>/init.<shell>`           |
//! | `SHELLGAME_SANDBOX`                | `landlock` or `off`: is the shell confined       |
//! | `SHELLGAME_PENDING`                | a command typed outside the game shell, see below |
//! | `TMPDIR`                           | `/tmp`, the writable temporary directory         |
//! | `BASH_SILENCE_DEPRECATION_WARNING` | `1`: no zsh banner from the macOS bash 3.2       |
//!
//! `SHELLGAME_PENDING` is the subcommand and its arguments joined by U+001F (`resume`, `3.2`), set
//! when the player typed `shellgame resume 3.2`, `reset` or `submit ...` outside the game shell. The
//! autostart's bare `shellgame` finds it, empties it with an `export` directive (so it runs once)
//! and runs that command as if it had been typed there.
//!
//! The binary takes `SHELLGAME_SANDBOX` as the sign that it runs inside the game shell, where its
//! directives are replayed. It exports `SHELLGAME_WORKSPACE` (where evidence markers live) itself.
//!
//! # The `shellgame` function and directives
//!
//! `shellgame ARGS...` runs `"$SHELLGAME_BIN" ARGS...` with stdin and stdout on the terminal and
//! stderr in a temporary file, replays that file line by line and returns the binary's status. A
//! line that starts with the marker is a directive and is run; every other line goes to stderr
//! unchanged, a marker anywhere else in a line is plain text. No command name is special:
//! `shellgame remove` closes the shell with the `exit` directive.
//!
//! ```text
//! __SHELLGAME_EXEC__v1 <verb>[ <base64 argument>...]
//! ```
//!
//! | verb     | arguments   | effect                                                           |
//! |----------|-------------|------------------------------------------------------------------|
//! | `cd`     | destination | the shell's own `cd`: never the hook below, so teleports are free |
//! | `export` | name, value | set and export one variable (`^[A-Za-z_][A-Za-z0-9_]*$`)         |
//! | `exit`   | none        | close the game shell                                             |
//!
//! Arguments are standard padded base64 of the raw bytes, so no metacharacter reaches a shell
//! parser; lines end in LF (one CR is tolerated). Unknown versions, verbs and argument counts are
//! ignored, and nothing is ever evaluated.
//!
//! # The `cd` hook
//!
//! One static `cd` function, no per-level list. Before moving it runs the binary directly (not
//! through `shellgame`: stderr stays live):
//!
//! ```text
//! "$SHELLGAME_BIN" cd-hook pre <pwd> <prev> -- <cd args...>
//! ```
//!
//! `<prev>` is where `cd -` would go (`$OLDPWD`; fish never sets it, so the newest entry of
//! `$dirprev`, or of `$dirnext` right after a `cd -`), empty when unknown. `<cd args...>` are the
//! arguments of the `cd` call verbatim (none, `-`, `--`, an empty string, a directory named `pre`),
//! so the binary parses this line by hand. Exit status **3** blocks: the function returns 1 without
//! moving, the binary having explained on stderr. Any other status, a crash or a missing binary
//! allows the move, so a broken hook never locks the player in. After a successful move it runs
//! `cd-hook post <old pwd> <new pwd> -- <cd args...>` with its output discarded and status ignored.
//!
//! # The other hooks
//!
//! * **`pwd`** is `builtin pwd` plus `touch "$SHELLGAME_WORKSPACE/.pwd_used"` after a success, only
//!   when that variable names an existing directory. The workspace is never guessed.
//! * **`chmod`** first runs `"$SHELLGAME_BIN" guard -- <args...>`. Status 3 refuses (nothing runs,
//!   return 1, the binary explained), anything else runs the real `chmod`. Only a typed `chmod` is
//!   guarded, not scripts or `command chmod`.
//!
//! # Shells
//!
//! bash 3.2 (the macOS system bash) and later, started as `bash --rcfile <script> -i`; the function
//! `shellgame` and its helpers are exported to child bash processes. The functions survive a
//! player's `set -e`. fish 3.3 and later with nothing from 3.4 or 4 (no `$(...)`), started as `fish
//! --no-config`. `tests/shell.rs` runs both scripts in real shells.

pub mod detect;
pub mod launch;
pub mod protocol;

pub use detect::{ShellKind, detect};
pub use launch::{LaunchOptions, launch_with};
pub use protocol::Directives;

pub const ENV_BIN: &str = "SHELLGAME_BIN";
pub const ENV_INIT: &str = "SHELLGAME_INIT";
pub const ENV_SANDBOX: &str = "SHELLGAME_SANDBOX";
pub const ENV_AUTO_STARTED: &str = "SHELLGAME_AUTO_STARTED";
pub const ENV_PENDING: &str = "SHELLGAME_PENDING";
pub const ENV_WORKSPACE: &str = "SHELLGAME_WORKSPACE";
