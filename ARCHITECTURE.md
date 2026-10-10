# ShellGame Architecture

ShellGame is one Rust binary (a library crate and a thin `main`) that runs
inside a controlled bash or fish subshell. Rust owns all game rules and
persistence; the shell scripts only forward events and apply a small, versioned
command protocol. Levels are Rust code: builder chains in section modules, with
the assignment text in Markdown files compiled in. Nothing is loaded at run
time; new content ships as a new binary (`shellgame update`).

## Runtime flow

1. `main` calls `cli::main`, which builds `Env` once (home, user, working
   directory, state directory, workspace override, wrapped flag) and parses the
   command line. Nothing reads the process environment or working directory
   afterwards: engine functions take an explicit `Ctx`, so tests run in
   parallel without `chdir` or `set_var`.
2. Outside the game shell, a bare `shellgame` renders the integration script and
   launches bash or fish with it. Subcommands run directly, with protocol
   output off.
3. Inside the game shell, `Session` loads or creates the state, prepares the
   current level, renders the assignment, exports the shell context and
   teleports the player to the level's start directory when it has one.
4. Every player command is a new short-lived process started by the
   `shellgame` shell function, which replays the directives the process wrote
   to stderr.
5. State-changing actions are persisted with one atomic write.
6. Debug builds run `Catalog::validate()` at startup, so a malformed level fails
   on the first `cargo run`.

## Responsibilities

| Area | Module | Responsibility |
| --- | --- | --- |
| Entry point | `src/main.rs`, `src/cli.rs` | clap definitions with Czech help, hand-parsed `cd-hook` and `fd-hook`, booting the game shell, dispatch, exit statuses |
| Environment | `src/env.rs` | `Env`, built once: home, user, cwd (`None` if deleted), state dir, workspace override, wrapped flag |
| Orchestration | `src/session.rs` | every command (show, hint, submit, skip, reset, repeat, status, remove, exit, dev, update), teleports, the hook handlers |
| Persistence | `src/state.rs` | `state.json` v2.1 model, migrations, atomic `StateStore`, locations |
| Evidence | `src/markers.rs` | marker files in the workspace root |
| Path containment | `src/paths.rs` | the one implementation of "stay inside this root" |
| Player texts | `src/messages.rs` | shared Czech messages |
| Level builder | `src/engine/level.rs` | `level()`, `intro()`, `Level::prepare`, `validate`, `Solution` |
| Sections and catalog | `src/engine/section.rs`, `catalog.rs`, `lint.rs` | ordering, navigation, stale-id recovery, `Catalog::validate` |
| Workspace fixtures | `src/engine/fixture.rs` | declarations and repair on apply |
| Answers | `src/engine/answer.rs` | typed rules and feedback |
| Requirements | `src/engine/require.rs` | unconditional conditions on the player's work |
| `cd` policies | `src/engine/cd.rs` | declarative rules with structural soft-lock protection |
| `fd` evidence | `src/engine/fd.rs` | where a command's stdout and stderr point |
| Context | `src/engine/ctx.rs`, `check.rs`, `id.rs` | `Ctx`, `Check`, `LevelId` |
| Course content | `src/levels/` | one module per section, `catalog()` |
| Shell scripts | `src/shell/bash.sh`, `fish.fish` | the thin integration, one per shell |
| Shell launch | `src/shell/detect.rs`, `render.rs`, `launch.rs` | choose the shell, render the script, start it |
| Protocol | `src/shell/protocol.rs` | typed emitter of the versioned directives |
| UI | `src/ui/` | `Term`, `Panel`, Markdown on pulldown-cmark, text layout, prompts, and `screens.rs` with every screen |

`src/engine/pytext.rs` (private) reproduces Python's `strip()`, `int()` and
`splitlines()`, so answers and checks behave as they did before the rewrite.

## Shell integration

The binary starts the game shell; a script sourced into it connects the two.
Nothing is pasted into script text: `SHELLGAME_BIN`, `SHELLGAME_INIT`,
`SHELLGAME_FD_HOOK` and `SHELLGAME_WRAPPER` reach the shell through the
environment.

```text
bash --rcfile <script> -i
fish --init-command 'function fish_greeting; end; source $SHELLGAME_INIT'
```

Do not add `--norc` (it disables `--rcfile`) or `--noprofile` (the player keeps
their environment). Both scripts follow the same order: setup, helper
functions, `pwd`, `cd`, the `shellgame` function, completions, autostart. The
autostart comes last, because calling the binary before the function exists
would print protocol lines on the terminal.

**Directives.** The binary writes `__SHELLGAME_EXEC__v1 <verb> [base64...]`
lines to stderr. The `shellgame` function captures stderr, replays it, and
executes only the closed set `cd`, `export`, `echo`, `pwd` and `exit`; other
lines pass through unchanged. There is no `eval`. `shellgame remove` closes the
shell with the `exit` directive, not by a check on the command name.

**The `cd` hook.** One static `cd` function in each script asks the binary
before and tells it after every move (`cd-hook pre ...` may block with exit
status 3, `cd-hook post ...` reports a move that happened). The binary decides
from the saved state, so there is no per-level list in the shell. A failing or
missing hook allows the move (fail-open). Teleports bypass the hook.

**The `fd` hook.** Level scripts run `"$SHELLGAME_FD_HOOK"`, a static launcher in
the state directory that `exec`s `shellgame fd-hook`, which inspects its own
descriptors.

Both scripts are written for bash 3.2 (the macOS system bash) and fish 3.3 or
later. The exact wire format, argument contract and compatibility rules are in
[docs/protocols/SHELL_PROTOCOL.md](docs/protocols/SHELL_PROTOCOL.md).

## Level contracts

- A section declares its number and root with `Section::new(number, root)`, a
  level its stable suffix with `level(number, title)`. IDs never depend on
  declaration order (the catalog sorts them numerically), and `N.0` is an intro
  and only an intro.
- A regular level declares an answer, requirements, or both. Only genuinely
  custom interactions use the escape hatches (`check`, `.feedback`, `.rule`,
  `.on_move`).
- Requirements are unconditional and run before the answer. A correct or empty
  answer never bypasses a filesystem, location, permission or evidence check.
  `at` compares canonical paths, never just the final directory name.
- **Every level-declared path is relative to that level's section root**:
  fixtures, requirements, start directories and `cd` rules speak one vocabulary,
  so a level can never disagree with itself about where it lives. A level that
  starts above its section says so with `.start_at_workspace()`. All of them
  resolve through `paths::resolve_within`, the single implementation, which
  rejects absolute paths, `..` and symlinked components.
- Feedback is data (`Check::Pass` or `Check::Fail(text)`), never an error type.
  The default wrong-answer text is the terse `Nesprávně.`; the catalog lint
  checks that no answer appears in an error, mistake or format message.
- Movement restrictions are a `CdPolicy`, not shell code. The engine performs
  the anti-soft-lock contract: a move out of scope, or *lifted* (the drill is
  done, or the move returns to the start directory), is allowed; otherwise the
  first failing rule blocks with the recovery tip. Evidence is recorded after
  the move from scope, rules and `arrive` only, and `on_move` runs after every
  successful move. Lifting waives blocking, never evidence.
- Evidence markers are named after the level (`.cd-1.4`, `.fd-10.4`,
  `.1.9-walk`), so they are unique by construction; only `.pwd_used` is shared.
  `Level::prepare` clears every marker the level references, repairs the
  section root, then applies the section fixture and the level fixture.
  Fixtures repair whatever the player did first (`chmod 000`, file and directory
  swaps, symlinks).
- Every regular level declares a solution: a shell script that the replay runs
  through the real game shell in bash and fish, and again over the workspace the
  first run left behind, as a `reset` would find it.
- Only `optional` levels may be skipped with `shellgame skip`.
- Gameplay validation belongs to the engine and the level, never to CLI or
  shell code.

## Session guarantees

- `submit` shows the level's own success text, never a generic one. Intros show
  none. Only failed attempts are counted, and a hint counts only when it is
  newly revealed. Skipping records no completion.
- Advancing performs exactly one atomic save, which also starts the next
  level's timer. It happens before the next level is prepared, so a failing
  fixture reports the problem without losing progress.
- Finishing the last level sets `completed_at`. After that `submit`, `skip`,
  `hint` and showing the level are inert and only re-display the completion
  summary.
- `reset` rebuilds the level and moves the player to its start directory, so it
  always rescues a player who wandered off. Fixtures overwrite files even when
  the player revoked write permission, so `reset` works after a `chmod` level.
- A deleted working directory is a normal, recoverable failure. `Env::cwd` is
  `None`, rules report it instead of failing, and the shell function moves to the
  workspace before running the binary.
- An unknown `current_level` resyncs to the nearest later level and keeps
  `levels_complete`, so renumbering between versions cannot brick a save. A
  missing workspace (a cleared `/tmp`) is rebuilt by the next gameplay command.
- The session keeps the player's directory in memory and updates it whenever it
  emits a `cd`, so a teleport notice appears only when the directory really
  changes. The `export` and `cd` directives of a level are emitted before any
  Enter prompt. Prompts read in non-canonical mode, where Ctrl-C is a key and
  not a signal, so the process always exits normally and the shell applies
  what it was told. Prompts appear only when stdin and stdout are terminals.
- Hooks fail open: no state, an unreadable state or an unknown level means
  nothing is enforced.
- `remove` deletes a workspace only when it lies strictly inside `/tmp` or
  `$TMPDIR` after symlinks are resolved.

## Persistence

`state.json` lives in `$SHELLGAME_STATE_DIR`, else `$XDG_CONFIG_HOME/shellgame`,
else `~/.config/shellgame` (also on macOS). It is schema `2.1`, written byte for
byte like the Python version did (field order, two-space indent, no trailing
newline, pydantic timestamps), and the two implementations read each other's
files. Versions `1.0` and `2.0` are migrated when loaded, and the next save
writes `2.1`.

Missing state means a new game. Malformed or unsupported state is an explicit
error and the file is never overwritten; `remove` is the one command that then
deletes it. A save writes a private temporary file in the same directory,
flushes it and renames it over `state.json`, so a failed save leaves the old
file intact and no temporary file behind.

A new game's workspace is `$SHELLGAME_WORKSPACE` or `/tmp/shellgame-$USER`, and
the saved path is used from then on. Command evidence is not state: markers
live in the workspace and are named after the level that owns them. Renaming a
marker never invalidates a save; at worst a player in the middle of that one
level repeats a single move.

## Quality gates

```text
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

The tests drive real shells, so the full suite needs bash, fish and shellcheck.
`rust-toolchain.toml` pins the toolchain, and CI builds with `--locked`.

| Suite | Protects |
| --- | --- |
| unit tests in `src/engine/` and `src/paths.rs` | fixture repair (`chmod 000`, file and directory swaps, symlinks never followed, modes), containment, every requirement, answer parsing and ordering, each `cd` rule against its counterexample, catalog navigation and lints |
| `src/levels/mod.rs` | the real catalog is sound; every level prepares in one workspace; nothing completes at the start; every `cd` policy blocks something yet never traps the player; every solution ends in a submit |
| `src/levels/s01_navigace/` tests | the maze (every instruction leads somewhere) and the walk home |
| `tests/state_compat.rs` | `state.json` compatible with Python byte for byte, migrations, corrupt state never overwritten, atomic save |
| `src/shell/` unit tests | script rendering, shell detection, the launcher's environment, and that `bash.sh` uses no bash 4 syntax |
| `tests/shell.rs` | both scripts in real bash and fish (the macOS job puts bash 3.2 first on `PATH`): directives, replay, injection, exit codes, deleted cwd, `pwd`, `cd` hook (block, allow, fail-open), teleports, `cd -`, autostart, the real launch; `shellcheck` and syntax checks |
| `tests/ui_render.rs` | Markdown, panels, text layout and paging |
| `tests/cli.rs` | CLI flows against the real binary in a sandbox, with a real pseudo-terminal for the prompts |
| `tests/solutions.rs` | every level's solution replayed through the real game shell, in bash (two umasks) and fish, and again over what the first run left behind; an empty submit never completes a level |
| `tests/strict_cd.rs` | the strict `cd` levels (1.4, 1.6 to 1.9, 1.12, 2.2, 2.3) in both shells: a wrong move is blocked with the recovery tip, the move to the start is always allowed, and the refused move is free once the drill is done |

The golden files in `tests/state/` were written by the last Python release (see
`tests/state/generate_golden.py`); the check that Python reads what Rust wrote
skips itself when Python is not available.

### Distribution

CI (`.github/workflows/ci.yml`) has four jobs. `rust-test` runs the gates above
on Linux with fish and shellcheck and also lints `scripts/install.sh`.
`rust-test-macos` runs them with the system bash 3.2 first on `PATH`; it does
not block until the macOS review of the levels is done. `build` produces
`shellgame-{linux,macos}-{x86_64,aarch64}.tar.gz` (static musl on Linux) on
pushes to the default branch and on manual runs; the build number is embedded
and shown by `--version`. `publish` attaches the archives to a GitHub release
tagged `build-<run_number>` on pushes to the default branch; the macOS archives
are held back while `PUBLISH_MACOS` is `"false"`. `scripts/install.sh`
downloads the latest archive for the machine. `shellgame update` does the same
from inside the game: it downloads the archive for its own platform with
`curl` and `tar`, checks that the new binary runs, and renames it over the
running one.

## Differences from the Python version

The state file, level ids, workspace layout and marker names are unchanged.
These behaviours changed on purpose:

- Evidence of a `cd` is recorded after the move succeeded (`cd-hook post`), so a
  move that fails in the shell earns nothing.
- `cd -` works in fish: fish never sets `OLDPWD`, so the hook gets the previous
  directory from `$dirprev`, and the stock `cd` is kept as `__shellgame_cd`.
- Fixtures end with explicit modes (`0755` directories, `0644` files) unless the
  level declares others, so results no longer depend on the player's umask.
- 1.7: the text no longer promises that reaching the goal completes the level
  (it never did; `shellgame submit` does). 1.8: only an absolute `cd` that lands
  in `absolute-target` counts, not `cd /` to somewhere else.
- The shell has one static `cd` hook and no per-level dispatch list, so it cannot
  drift from the binary's state.
- `dev jump <id>` also clears that level's progress, so it can be replayed.
  Prompts read Ctrl-C as a key, so interrupting one ends the command normally
  and the shell still applies the directives it was sent.

See [docs/AUTHORING.md](docs/AUTHORING.md) for how to add a level or section.
