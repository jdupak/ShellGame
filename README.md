# ShellGame

ShellGame is a terminal course for first-semester students; the game itself is in Czech. It teaches the
shell by doing: navigation (`pwd`, `ls`, `cd`, absolute and relative paths), creating, copying and
removing files, hidden files, permissions, wildcards, redirection and pipes, `grep` and `find`.

It is one Rust binary. `shellgame` starts a wrapped bash or fish shell with the game built in, so there
is nothing to `source`. Everything you type there is a normal shell command; the game's own commands
start with `shellgame`.

The course outline with each section's learning goals is published with the code documentation:
<https://jdupak.github.io/ShellGame/shellgame/levels/>.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | bash
./shellgame
```

The script downloads the latest release for your machine into the current directory. It needs `curl`
and `tar`, no root, and runs in `sh` as well as `bash`. Another directory:
`... | SHELLGAME_INSTALL_DIR="$HOME/bin" bash` (`SHELLGAME_REPO=owner/name` installs from a fork).

Supported: Linux on x86_64 and aarch64. macOS builds are produced by CI but not published yet: the
macOS review of the levels is not finished. Windows is not supported; use WSL. The game shell is bash
3.2 or newer, or fish 3.3 or newer.

## Commands

| Command | What it does |
| --- | --- |
| `shellgame` | start the game; inside the game shell, show the current level |
| `shellgame hint` | reveal one more hint; the ones you unlocked before are shown again with it (`hint -r` shows only those) |
| `shellgame submit <answer>` | submit an answer, e.g. `shellgame submit level-1`; levels graded on your files or location take a bare `shellgame submit` |
| `shellgame status` | progress and statistics |
| `shellgame reset` | rebuild the current level's files, show the assignment, move you back to its start directory |
| `shellgame show` | show the current level's assignment again; `show 1.7` shows level 1.7's, `show 2` the intro of section 2 |
| `shellgame levels` | list every level by section, marking the one you are at and the completed ones |
| `shellgame resume 3.2` | continue at level 3.2 (completed levels keep their records); `resume 3` goes to the first level of section 3 (after its intro). Typed outside the game shell it starts the game shell and runs there |
| `shellgame remove` | delete all game data and the workspace (asks first; `--yes` skips the question) |
| `shellgame exit` | leave the game shell (`exit` works too) |
| `shellgame update` | update to the latest release; run it outside the game shell |
| `shellgame --shell bash` | start the game in bash (or `fish`) instead of the detected shell |
| `shellgame --unsafe` | start without isolation, only for systems without Landlock (see below) |
| `shellgame --version` | print the version and build |

Progress lives in `state.json` in `~/.config/shellgame` (or `$XDG_CONFIG_HOME/shellgame`); the level
files live in `/tmp/shellgame-$USER`. `SHELLGAME_WORKSPACE` can move them, but only to a directory of
yours inside `/tmp`. `NO_COLOR=1` turns colours off.

### Continuing on another machine

Install ShellGame there and continue where you were. `shellgame status` and `shellgame exit` print the
command, for example `shellgame resume 3.2`; `shellgame levels` lists every level. Level numbers are
positions (`3.2` is the second level of section 3). The levels before the one you resume at are not
marked as completed.

## Isolation

You type real commands, so a mistyped `rm` could hurt real files. On Linux the game shell and every
program started from it are confined with Landlock: they can change files only in `/tmp`, in the
game's state directory and in `/dev`. Writing, deleting, renaming or creating anything else fails with
"Permission denied", and a line under each level says the isolation is on.

Landlock also forbids gaining privileges, so `sudo` and `su` do not work inside the game shell; open
another terminal for those. The same goes for `shellgame update`, which replaces the installed binary:
run it in an ordinary terminal.

Where Landlock is missing (some older or minimal kernels, WSL 1, some containers, macOS) the game
refuses to start and says why. `shellgame --unsafe` starts it anyway, with a red warning; use it only
on a machine you can afford to lose files on.

The isolation protects against mistakes, not against someone trying to get around it. Known limits:

- Landlock cannot restrict permission, ownership or timestamp changes. A typed `chmod` outside `/tmp` is
  refused by a guard, but `chmod`, `chown` and timestamp changes (for example `touch` on an existing
  file) still work through scripts, `find -exec` or `command chmod`.
- `truncate` can still empty files outside `/tmp` on kernels older than 6.2.
- Your own files in `/tmp` can be deleted, including other programs' files there.
- Programs that were already running outside the game keep their access, and other terminal windows
  are not confined.

## Troubleshooting

- **Stuck: a navigation level rejects every `cd`.** Some levels practise one exact command and refuse
  other moves on purpose. `shellgame reset` always takes you back to the level's start directory, and
  `cd` back to that directory is always allowed.
- **"Uložená hra pochází ze starší verze".** The old save is kept as `state-old.json` and a new game
  starts. Continue where you were with `shellgame resume S.L`, e.g. `shellgame resume 3.2`; the notice
  names your level when the old save records it.
- **"… pochází z novější verze ShellGame".** The save was written by a newer release: run
  `shellgame update` (outside the game shell).
- **The save file is damaged.** ShellGame never overwrites it. Repair the file named in the message, or
  run `shellgame remove` and start over.
- **The workspace is gone** (a reboot cleared `/tmp`). Run `shellgame`: the level files are rebuilt and
  your progress is kept.

## Development

```bash
cargo run                                  # build and play from source; add -- --shell fish for fish
cargo test                                 # needs bash; fish, shellcheck, python3, curl, tar optional
cargo clippy --all-targets -- -D warnings
cargo fmt
```

`rust-toolchain.toml` pins the toolchain; rustup installs it on first use. Contributors and coding
agents: read [AGENTS.md](https://github.com/jdupak/ShellGame/blob/main/AGENTS.md) first. It covers the repository layout, how to add a level, the
invariants, the testing philosophy and the release flow.

## License

Copyright (C) 2025-2026 Jakub Dupak <dev@jakubdupak.com>

This project is licensed under the GNU General Public License v3.0 (GPL-3.0-only).
See `LICENSE`.
