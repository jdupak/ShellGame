# Shell protocol

How ShellGame changes the player's shell, and how the shell asks the game for
decisions.

A process cannot change the directory or the environment of its parent. So the
game starts a *game shell* of its own and sources a small script into it
([`src/shell/bash.sh`](../../src/shell/bash.sh),
[`src/shell/fish.fish`](../../src/shell/fish.fish)). The script connects the
shell to the binary in both directions:

| Direction | Mechanism |
| --- | --- |
| binary to shell | *directives* written to stderr, replayed by the `shellgame` function |
| shell to binary | the `cd` hook, the `fd` hook launcher and the `pwd` marker |

The contract is also written down in the module documentation of
[`src/shell/mod.rs`](../../src/shell/mod.rs). If the two ever disagree, the
code wins.

## Environment

The launcher sets these with `Command::env`. They are never pasted into script
text, so no path is ever quoted.

| Variable | Value | Set by |
| --- | --- | --- |
| `SHELLGAME_WRAPPER` | `1`: the binary runs inside the game shell | launcher |
| `SHELLGAME_BIN` | absolute path of the binary | launcher |
| `SHELLGAME_INIT` | path of the rendered integration script | launcher |
| `SHELLGAME_FD_HOOK` | path of the fd-hook launcher | launcher |
| `BASH_SILENCE_DEPRECATION_WARNING` | `1`: no zsh banner from the macOS bash 3.2 | launcher |
| `SHELLGAME_AUTO_STARTED` | `1` once the game autostarted; removed by the launcher, so a shell started from inside a game shell starts the game again | script |
| `SHELLGAME_WORKSPACE` | the workspace, where evidence markers live | binary, `export` directive |
| `SHELLGAME_LEVEL` | the current level id, such as `1.4` | binary, `export` directive |

`SHELLGAME_LEVEL` is exported only because the `cd` function of a game shell
started by the Python version dispatches on it. The scripts read the workspace
from `$SHELLGAME_WORKSPACE` and nowhere else; they never guess `/tmp/shellgame-$USER`.

The binary also reads `SHELLGAME_FORCE_SHELL` (`bash` or `fish`, which shell to
start; `--shell` wins) and `SHELLGAME_UPDATE_URL`. They are not part of the
protocol.

## Launching the game shell

Outside the game shell, a bare `shellgame` renders the script into a private
temporary file and runs

```text
bash --rcfile <script> -i
fish --init-command 'function fish_greeting; end; source $SHELLGAME_INIT'
```

- Never add `--norc` (it disables `--rcfile`) or `--noprofile` (the player keeps
  their `PATH` and environment setup).
- The script is deleted when the shell exits. The one thing rendering changes is
  the `@DEV_SHORTCUTS@` placeholder: with `--devmode`, bash gets aliases and
  fish abbreviations `r n p j s e g`.
- Both scripts end with an autostart block that calls the `shellgame` function
  once. It has to come last: calling the binary before the function exists
  would print directives on the terminal.
- The shell is chosen by `--shell` or `SHELLGAME_FORCE_SHELL`, else by the
  nearest bash, fish or zsh among the parent processes (up to 25 levels; `/proc`
  on Linux, `ps -o ppid=,comm=` elsewhere, so wrappers such as `make` or `sudo`
  are seen through), else by `$SHELL`. zsh and anything unknown mean bash.
- The exit status of the game shell is not a launch failure.

## Directives, protocol v1

```text
__SHELLGAME_EXEC__v1 <verb>[ <argument>...]
```

- A line of stderr that **starts** with `__SHELLGAME_EXEC__` is a directive.
  The marker anywhere else in a line, or after leading whitespace, is plain text.
- `v1` follows the prefix directly. Then come one space and the verb, and one
  space before each argument.
- An argument is the raw bytes of the value in standard, padded base64
  (`A-Za-z0-9+/=`). It contains no space or newline, so splitting the line on
  single spaces is safe, and any bytes (a path that is not UTF-8) survive. An
  empty argument is an empty word, so the line ends in a space.
- Lines end in LF; the shell removes one trailing CR.

| Verb | Arguments | Effect in the shell |
| --- | --- | --- |
| `cd` | destination | change directory with the shell's own `cd`, **not** through the hook (bash `builtin cd --`, fish `__shellgame_cd --`) |
| `export` | name, value | set and export one variable; the name must match `^[A-Za-z_][A-Za-z0-9_]*$` |
| `echo` | message | print the message and a newline to stdout |
| `pwd` | none | print the working directory (`builtin pwd`); not recorded as `pwd` evidence |
| `exit` | none | delete the capture file and exit the game shell with status 0; later directives are not run |

Arities are fixed. An unknown version, an unknown verb, a wrong number of
arguments, an invalid name or undecodable base64 makes the directive a no-op.
Such a line is never printed and never stops the replay of the lines after it.
There is no way to run an arbitrary command: no `eval`, no command strings.

```text
__SHELLGAME_EXEC__v1 echo aGVsbG8=
__SHELLGAME_EXEC__v1 export U0hFTExHQU1FX0xFVkVM MS40
__SHELLGAME_EXEC__v1 cd L3RtcC9zaGVsbGdhbWU=
__SHELLGAME_EXEC__v1 pwd
__SHELLGAME_EXEC__v1 exit
```

The emitter is `Directives` in
[`src/shell/protocol.rs`](../../src/shell/protocol.rs). It writes each line with
one `write_all` and flushes, and writes nothing at all when
`SHELLGAME_WRAPPER` is not set. The session emits the `export` and `cd`
directives of a level *before* any Enter prompt: Ctrl-C at a prompt is only a
key, the process exits normally, and the shell still applies what it was sent.

Limits: values should not contain newlines. Bash drops a trailing one, fish
turns a line break inside a value into a space. Paths and level ids never
have any.

## The `shellgame` function

`shellgame ARGS...` runs the binary and replays what it wrote to stderr:

1. Create a capture file with `mktemp`. If that fails, return 1 without running
   the binary.
2. If `$PWD` no longer exists, move to `$SHELLGAME_WORKSPACE` (when it is an
   existing directory) or else to `/tmp`. This move never goes through the
   hook.
3. Run `"$SHELLGAME_BIN" ARGS...` as argv, never through `eval`, with stderr in
   the capture file. stdin and stdout stay on the terminal, so prompts work
   and piped input reaches the binary.
4. Replay the capture file line by line, in order. A directive is executed;
   any other line is written to stderr unchanged (apart from one trailing CR),
   and a last line without a newline gets one.
5. Delete the capture file and return the binary's exit status. A failing
   directive never changes it.

Nothing is special-cased by command name. `shellgame remove` closes the shell
because the binary sends `exit`; a declined `remove` exits 1, because a Python-era
wrapper closes the shell whenever `remove` returns 0.

Because stderr is a file while the binary runs, the binary forces colour on for
the panels it prints on stderr (unless the terminal says no: `NO_COLOR`,
`TERM=dumb`).

bash also exports `shellgame` and its two helpers (`__shellgame_decode`,
`__shellgame_eval`), so child bash processes can call it.

Exit statuses of the binary: 0 for normal gameplay (a wrong answer included), 1
for errors and a declined `remove`, 2 for a usage error, 3 only from `cd-hook pre`.

## The `cd` hook

Both scripts define one static `cd` function. There is no per-level list: the
binary decides from the saved state, which cannot drift from the shell. The
function:

```text
1. "$SHELLGAME_BIN" cd-hook pre <pwd> <prev> -- <cd args...>
      exit status 3            -> return 1, no move
      any other exit status    -> go on
2. perform the move       (bash: builtin cd "$@", fish: __shellgame_cd $argv)
3. if the move succeeded:
   "$SHELLGAME_BIN" cd-hook post <pwd before> <pwd after> -- <cd args...> >/dev/null 2>&1
4. return the status of the move
```

| Argument | Meaning |
| --- | --- |
| `<pwd>` | `$PWD` before the move |
| `<prev>` | where `cd -` would go: bash `$OLDPWD`; fish never sets `OLDPWD`, so the newest entry of `$dirprev` (or of `$dirnext` right after a `cd -`, since fish toggles direction). An empty argument when unknown |
| `<pwd after>` | `$PWD` after the move, as the shell reports it, never re-resolved |
| `<cd args...>` | every argument of the `cd` call, verbatim, after the literal `--`: possibly none, `-`, `-P`, `--`, an empty string, or a directory named `pre` |

Properties:

- **Direct call.** The hook does not go through the `shellgame` function, so
  its stderr is live (the red panel appears at once) and nothing is replayed or
  relocated. The binary parses these command lines by hand, before its option
  parser, because the arguments are arbitrary text.
- **Exit status 3 means blocked.** The binary has printed the explanation
  itself, on stderr. **Every other status allows the move**: 0, a crash, a
  missing or empty `SHELLGAME_BIN`. The hook fails open, so a broken hook can
  never lock a player in.
- **`post` is silent.** Its output and status are ignored, and a move that
  failed in the shell is never reported, so it earns no evidence.
- **Teleports bypass the hook.** The `cd` directive and the deleted-directory
  relocation call the shell's own `cd`, so level rules never trip over the
  game's own moves.
- **fish.** The script runs `functions --copy cd __shellgame_cd` before it
  defines the hook (guarded, so sourcing twice is harmless). The hook and the
  teleports call the copy, so `cd -`, `prevd` and `nextd` work, also right
  after a teleport.
- **`set -e`.** The bash functions are written to survive it (and `set -u`) in
  the player's shell: a failing hook must not end the shell before the move.

### Legacy forms

A game shell started by the Python version can still be running when the binary
is replaced. Its `cd` function calls, through the `shellgame` function:

```text
shellgame cd-hook <target> <pwd>        before the move; non-zero status = blocked
shellgame cd-hook --post-move <pwd>     after the move; output discarded
```

The binary still answers them. After `cd-hook`, anything that is not
`pre|post <a> <b> --` is read as a legacy call, so a legacy `cd pre` is not
mistaken for the new form. A blocked legacy call exits with **1**, because the
old function treats every non-zero status as a block. The legacy pre-move call
records the level's evidence before the move, as Python did (for policies
without `arrive` and without manual evidence). The post-move call records
evidence for policies with `arrive` and runs `on_move`. The binary keeps
exporting `SHELLGAME_LEVEL` and `SHELLGAME_WORKSPACE`, and accepts a leading
`--devmode` on any command, because the Python wrapper prefixed it.

## `pwd`

Both scripts define `pwd` as `builtin pwd "$@"`, the logical path in both
shells, followed by `touch "$SHELLGAME_WORKSPACE/.pwd_used"`. That happens only
if `pwd` succeeded and `$SHELLGAME_WORKSPACE` is set and names an existing
directory. The workspace is never guessed.

## The `fd` hook

Level scripts that need to show where their output went run
`"$SHELLGAME_FD_HOOK"`. The launcher is written by the binary whenever it
starts a game shell, to `<state dir>/fd-hook.sh` (mode 0700, replaced
atomically, so shells starting together never see a partial file):

```sh
#!/bin/sh
exec "$SHELLGAME_BIN" fd-hook "$@"
```

`exec` keeps the descriptors the script was started with, which are the
player's redirections (`./buggy.sh &> /dev/null`). The `fd-hook` command
compares its descriptors 1 and 2 with `/dev/null` using `fstat` (device and
inode, no `/proc`, so it works on macOS) and records the level's `fd` marker
if the level's rule accepts them. It is silent and exits 0. The launcher lives in
the state directory because `$TMPDIR` may be mounted `noexec`.

A script must call the launcher directly. The `shellgame` function would capture
stderr, which is exactly what the hook has to see.

## Compatibility

| Component | Requirement |
| --- | --- |
| bash | 3.2 and later. `bash.sh` is written for the macOS system bash: no associative arrays, `mapfile`, `${x,,}`, `&>>` or `\|&`, and newer `shopt` options are guarded (`globasciiranges`) |
| fish | the scripts need 3.3 or later and nothing from 3.4 or 4. Only the level *solutions* use `$(...)`, which needs 3.4 |
| tools | `mktemp` without arguments, `base64 --decode` (GNU and macOS), `touch`, `rm` |
| shellcheck | `bash.sh` passes 0.9.0 and 0.11 |

`tests/shell.rs` runs the scripts in real shells. It has been checked with
bash 3.2.57 and 5.3, fish 3.3.1, 3.7.0 and 4.8.1, and shellcheck 0.9.0 and
0.11. To try another version, put it first in `PATH`.

## Extending the protocol

Adding a verb widens the boundary between the game and the player's shell. A
new verb must:

- have a fixed arity and validate its arguments in Rust *and* in both scripts,
- avoid any generic evaluation,
- be implemented in `bash.sh` **and** `fish.fish`, with a test in
  `tests/shell.rs` (`shell_test!` runs each test once per shell),
- keep the scripts free of game logic.

Changing the wire format needs a new version, because a shell that started with
an older script keeps running it. Directives go to stderr only. Exported
variable names must not collide with the locals of bash's `__shellgame_eval`
(`name`, `value`, `verb`, `version`, `destination`, `stderr_file`): keep the
`SHELLGAME_` prefix.
