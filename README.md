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
shellgame init
shellgame
```

### Playing the game

Show current level:

```sh
shellgame
```

Get hints:

```sh
shellgame hint
```

Submit answers:

```sh
shellgame submit <your-answer>
```

Example: `shellgame submit level-1`

Check progress:

```sh
shellgame status
```

Reset current level (rebuild workspace for the level and re-show assignment):

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

"Not initialized"
- Run: `shellgame init`

"Level not found"
- Run: `shellgame init`

Wrong answer
- Re-read the assignment
- Check `pwd` to confirm where you are
- Use `shellgame hint`

Need to start over:

```bash
shellgame remove
shellgame init
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

Create a standalone binary using `PyInstaller`:

```bash
make build
```

## Shell integration (built-in)

ShellGame provides built-in subshell integration automatically on startup (bash/fish). You don’t need to `source` anything.

Protocol reference:
- `docs/protocols/SHELL_PROTOCOL.md`

## License

Copyright (C) 2025 Jakub Dupak <dev@jakubdupak.com>

This project is licensed under the GNU General Public License v3.0 (GPL-3.0-only).
See `LICENSE`.
