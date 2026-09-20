# ShellGame

> [!WARNING]
> This project is **largely LLM-generated** and is in an **early stage of development**.
>
> Proceed with caution, and expect significant changes.

Interactive terminal learning experience for filesystem navigation and basic shell operations.

## What this is

ShellGame is a terminal-based learning tool designed for first-semester CS students to practice shell navigation and common filesystem operations through progressive levels.

## Quick start (players)

ShellGame starts a wrapped subshell (bash/fish) with integration enabled automatically. You don’t need to `source` anything.

Just run:

```bash
shellgame
```

### Manual shell selection (override)

If you want to explicitly choose which subshell ShellGame launches (useful in nested shells or under wrappers like `uv` / `make`), use:

```bash
shellgame --shell fish
# or
shellgame --shell bash
```

This affects only the initial wrapper launch (when ShellGame is not already running inside the wrapper).

### First time setup (recommended)

```bash
make dev
shellgame
```

### Playing the game

Show current level:

```sh
shellgame
```

Get hints (each call reveals one more hint):

```sh
shellgame hint
```

Re-read the hints you already unlocked (does not consume a new one):

```sh
shellgame hint --repeat
```

Submit answers:

```sh
shellgame submit <your-answer>
```

Example: `shellgame submit level-1`

Skip the current level (only possible on optional and extension levels):

```sh
shellgame skip
```

Check progress:

```sh
shellgame status
```

Reset current level (rebuild workspace for the level, re-show assignment, and
move you back to the level's start directory):

```sh
shellgame reset
```

Repeat assignment (without resetting):

```sh
shellgame repeat
```

You can also target a section or a specific level:
- `shellgame repeat --section 2` (shows level `2.0`)
- `shellgame repeat --level 1.7`

Clean up (removes all game data and workspace; requires confirmation):

```sh
shellgame remove
```

### Tips

- Read the goal carefully — each level has specific requirements
- Use hints wisely — they’re tracked but there to help you learn
- Experiment freely — the workspace is temporary and safe
- Check your location — use `pwd` before submitting

### Troubleshooting

"Level not found"
- Restart ShellGame; if the problem persists, remove and recreate the game data.

Wrong answer
- Re-read the assignment
- Check `pwd` to confirm where you are
- Use `shellgame hint`

Stuck: a navigation level rejects every `cd` you try
- Some levels practise one exact command, so other moves are refused on purpose.
- `shellgame reset` always takes you back to the level's start directory.
- Once you have used the required command, normal movement is restored.

Need to start over:

```bash
shellgame remove
shellgame
```

### Learning goals

ShellGame teaches you to:
- Navigate the filesystem confidently
- Understand absolute vs relative paths
- Use `pwd`, `ls`, `cd` fluently
- Read and follow file-based instructions
- Build mental models of directory trees

## Install (development)

```bash
git clone https://github.com/jdupak/ShellGame.git
cd ShellGame

make dev
```

Notes:
- This project uses `uv` for dependency management and virtual environments.
- `make dev` will create/sync `.venv` and install the `dev` extras.

## Install (distribution)

Install the latest Linux x86_64 binary into the current directory (no root or `sudo` required), then run it directly:

```bash
curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | bash
./shellgame
```

To choose another installation directory:

```bash
curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | INSTALL_DIR="$HOME/bin" bash
"$HOME/bin/shellgame"
```

Or build a standalone binary locally with `PyInstaller`:

```bash
make build
```

Every push to the default branch creates a GitHub Release asset containing a packed `shellgame` binary.

## Shell integration (built-in)

ShellGame provides built-in subshell integration automatically on startup (bash/fish). You don’t need to `source` anything.

Protocol reference:
- `docs/protocols/SHELL_PROTOCOL.md`

## License

Copyright (C) 2025 Jakub Dupak <dev@jakubdupak.com>

This project is licensed under the GNU General Public License v3.0 (GPL-3.0-only).
See `LICENSE`.
