# ShellGame

> [!WARNING]
> This project is **largely LLM-generated** and is in an **early stage of development**.
>
> Proceed with caution, and expect significant changes.

Interactive terminal learning experience for filesystem navigation and basic shell operations.

## What this is

ShellGame is a terminal-based learning tool designed for first-semester CS students to practice shell navigation and common filesystem operations through progressive levels. The game itself is in Czech.

It is a single Rust binary. It starts a bash or fish subshell with the game built in, so you don't need to `source` anything.

## Quick start (players)

### Install

The install script downloads the latest release for your machine into the current directory. It needs `curl` and `tar`, and no root or `sudo`:

```bash
curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | bash
./shellgame
```

To choose another installation directory:

```bash
curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | INSTALL_DIR="$HOME/bin" bash
"$HOME/bin/shellgame"
```

(`REPO=owner/name` installs from a fork.)

Supported systems: Linux on x86_64 and aarch64. macOS builds (aarch64 and x86_64) are built by CI but not published yet: a few levels still use GNU-only command options, and the macOS review of the levels is not finished. Windows is not supported; use WSL. The game shell is bash 3.2 or newer, or fish 3.3 or newer.

### Start the game

```bash
shellgame
```

ShellGame starts a wrapped subshell (the same kind you are using, bash or fish; zsh users get bash) with the integration enabled, creates your game on the first run, and shows the first level. Everything you type there is a normal shell command. The game's own commands start with `shellgame`. Leave with `shellgame exit` (or `exit`).

If auto-detection picks the wrong shell (nested shells, wrappers like `make` or `sudo`), choose one:

```bash
shellgame --shell fish
# or
shellgame --shell bash
```

This only affects the initial launch, when ShellGame is not already running inside its game shell. `SHELLGAME_FORCE_SHELL=fish` does the same.

### Playing the game

| Command | What it does |
| --- | --- |
| `shellgame` | show the current level |
| `shellgame hint` | reveal one more hint |
| `shellgame hint --repeat` (`-r`) | re-read the hints you already unlocked (does not consume a new one) |
| `shellgame submit <answer>` | submit your answer, e.g. `shellgame submit level-1`; some levels need no answer, just `shellgame submit` |
| `shellgame status` | progress and statistics |
| `shellgame skip` | skip the current level (only optional levels) |
| `shellgame reset` | rebuild the current level's files, show the assignment and move you back to its start directory |
| `shellgame repeat` | show the assignment again, without resetting |
| `shellgame repeat --section 2` | show the intro of a section (level `2.0`) |
| `shellgame repeat --level 1.7` | show a specific level |
| `shellgame show --level` / `--section` | show the current level or the current section's intro again |
| `shellgame remove` | delete all game data and the workspace (asks first; `--yes` skips the question) |
| `shellgame exit` | leave the game shell |
| `shellgame update` | update ShellGame to the latest release |
| `shellgame --version` | print the version and build |

### Tips

- Read the goal carefully; each level has specific requirements.
- Use hints wisely: they're tracked but there to help you learn.
- Experiment freely; the workspace is temporary and safe.
- Check your location with `pwd` before submitting.

### Troubleshooting

"Level not found"
- If you passed an ID to `repeat --level`, check it (for example `1.7`).
- Otherwise run `shellgame` again; if the problem persists, remove and recreate the game data.

Wrong answer
- Re-read the assignment.
- Check `pwd` to confirm where you are.
- Use `shellgame hint`.

Stuck: a navigation level rejects every `cd` you try
- Some levels practise one exact command, so other moves are refused on purpose.
- `shellgame reset` always takes you back to the level's start directory.
- Once you have used the required command, normal movement is restored.

The workspace disappeared (for example after a reboot cleared `/tmp`)
- Run `shellgame`. The workspace is rebuilt and your progress is kept.

"State cannot be loaded"
- The save file is damaged or from an unsupported version. ShellGame never overwrites it. Fix or delete the file named in the message, or run `shellgame remove` and start over.

Need to start over:

```bash
shellgame remove
shellgame
```

Colours off: set `NO_COLOR=1`.

### Where your data lives

| What | Where |
| --- | --- |
| progress (`state.json`) | `$SHELLGAME_STATE_DIR`, else `$XDG_CONFIG_HOME/shellgame`, else `~/.config/shellgame` (also on macOS) |
| level files (the workspace) | `$SHELLGAME_WORKSPACE`, else `/tmp/shellgame-$USER`; the path is saved when a game is created |

`shellgame remove` only deletes a workspace below `/tmp` or `$TMPDIR`. If you played the earlier Python version of ShellGame, your progress carries over: the save file is the same.

### Updating

```bash
shellgame update
```

This downloads the latest release for your platform (it needs `curl` and `tar`), checks that the new binary runs, and replaces the installed one. Your progress is kept. If ShellGame cannot write to its own directory, the message tells you to run the install script again.

### Learning goals

ShellGame teaches you to:
- Navigate the filesystem confidently
- Understand absolute vs relative paths
- Use `pwd`, `ls`, `cd` fluently
- Read and follow file-based instructions
- Create, inspect, copy, move, and remove filesystem entries safely
- Understand file and directory permissions, redirection, stdin, stdout, and stderr
- Quote paths containing spaces and react to command success with `&&` and `||`
- Select files with glob patterns and search with `grep` and `find`
- Work efficiently with built-in help, manuals, and short aliases
- Build mental models of directory trees and shell data flow

## Development

```bash
git clone https://github.com/jdupak/ShellGame.git
cd ShellGame

cargo run                  # build and play from source (starts the game shell)
cargo run -- --shell fish  # force a shell
cargo test                 # the whole suite
cargo clippy --all-targets -- -D warnings
cargo fmt
```

`rust-toolchain.toml` pins the Rust toolchain; rustup installs it on first use. CI runs `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`.

The tests drive real shells. Install `bash` (required), `fish` and `shellcheck`, plus `python3` (pseudo-terminal prompt tests) and `curl` with `tar` (the `update` tests). Tests that need a missing tool skip themselves with a note. One level's reference solution is replayed in real shells with `cargo test --test solutions -- 1.4`.

### Playtesting without touching your own save

Build first, then run the binary with a throwaway `HOME`, `XDG_CONFIG_HOME` and workspace. (Do not run `cargo run` with a changed `HOME`: cargo and rustup live there.)

```bash
cargo build
sandbox="$(mktemp -d)"
mkdir -p "$sandbox/home"
env HOME="$sandbox/home" XDG_CONFIG_HOME="$sandbox/config" \
    SHELLGAME_WORKSPACE="$sandbox/workspace" target/debug/shellgame --devmode
```

`--devmode` adds the shortcuts `j <id>` (jump to a level), `n`, `p`, `r`, `s`, `e` and `g`. Try both shells.

### Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md): how the pieces fit together and what they guarantee.
- [docs/AUTHORING.md](docs/AUTHORING.md): how to add a level or a section.
- [docs/protocols/SHELL_PROTOCOL.md](docs/protocols/SHELL_PROTOCOL.md): the contract between the binary and the shell scripts.
- [.github/copilot-instructions.md](.github/copilot-instructions.md): the rules for contributors and coding agents.

## License

Copyright (C) 2025-2026 Jakub Dupak <dev@jakubdupak.com>

This project is licensed under the GNU General Public License v3.0 (GPL-3.0-only).
See `LICENSE`.
