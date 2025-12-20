# Shell Command Protocol

## Overview

ShellGame uses a **shell command protocol** to execute shell commands from Python, enabling features like automatic directory navigation between levels.

## Status (shell integration)

Shell integration is **built-in**: when you run `shellgame` outside the wrapped environment, ShellGame automatically starts a wrapped subshell (bash/fish) with integration loaded.

Integration scripts are generated at runtime from templates in:

- `src/shellgame/cli/templates/`

This document covers both the **protocol contract** (markers + execution) and how subshells are launched.

## Architecture

### Communication Flow

```
┌──────────┐        stderr         ┌──────────────┐
│          │  __SHELLGAME_EXEC__   │              │
│  Python  │  ─────────────────>   │ Shell Wrapper│
│   Game   │      cd /path         │  (Fish/Bash) │
│          │                       │              │
└──────────┘                       └──────────────┘
                                          │
                                          │ eval
                                          ▼
                                   User's Shell State
                                   (CWD, env vars, etc.)
```

### Protocol Specification

Python emits specially formatted commands to stderr:

```
__SHELLGAME_EXEC__<shell_command>
```

Shell wrappers intercept stderr, parse these markers, and execute the commands.

## Implementation

### Python side

The Python runtime emits protocol lines via `src/shellgame/shell/__init__.py`. The wrapper executes only the lines that start with the protocol marker.

**Security note**: arguments are shell-escaped on the Python side to reduce injection risk; still, treat any new “verbs” you add to the protocol as a security boundary.

### Shell Side

ShellGame generates per-shell integration scripts at runtime (from templates) and starts a wrapped subshell:

- **Fish**: launched with `--init-command` that suppresses `fish_greeting` and sources the integration script. Autostart is inside the template.
- **Bash**: launched with `--rcfile <integration_script> -i`. The integration script is used directly as the rcfile (no separate rc template).

**CRITICAL bash notes**:
- Do NOT use `--norc` — it disables `--rcfile` entirely!
- Do NOT use `--noprofile` — let users keep their PATH/env setup.

No manual sourcing is required. Just run:

```sh
shellgame
```

Both templates follow the same pattern:
1. Shell setup (disable interfering features)
2. Function definitions (`shellgame`, `pwd`, `cd` hooks, `__shellgame_eval`)
3. Autostart block **at the end** (calls `shellgame` function after it's defined)

The `shellgame` function (defined in the template) intercepts stderr and executes lines emitted by Python that start with:

```
__SHELLGAME_EXEC__<shell_command>
```

Implementation excerpts live in the template files:

- `src/shellgame/cli/templates/fish_integration.template`
- `src/shellgame/cli/templates/bash_integration.template`

## Usage in Game

The protocol is used primarily for:

1. **Navigation/teleportation** (e.g., `cd` into a level’s start directory)
2. Small integration helpers (env exports, etc.) tied to the wrapper session

Concrete orchestration lives in `src/shellgame/core/session.py` and the wrapper/integration glue in `src/shellgame/cli/*`.

## User Experience

### Without wrapper (manual navigation)

```sh
$ shellgame init
✓ Initialized ShellGame for user
Workspace: /tmp/shellgame-user

Important: Navigate to your workspace to begin:
  cd /tmp/shellgame-user/level-1

$ pwd
/home/user
$ cd /tmp/shellgame-user/level-1  # Manual navigation required
```

### With built-in wrapper (recommended)

```sh
$ shellgame
# ShellGame will launch a wrapped subshell if needed.
# Inside that wrapper, navigation can be automatic.
```

## Testing

Automated coverage lives in `tests/` (including wrapper generation and linting).

If you want a quick protocol sanity check, run ShellGame and observe that stderr contains protocol markers when it needs to change directory.

## Benefits

1. **Seamless UX**: No manual navigation between levels
2. **Universal**: Works with any shell command (cd, export, echo, etc.)
3. **Safe**: Proper escaping prevents command injection
4. **Flexible**: Easy to add new shell operations
5. **Transparent**: Regular stderr is preserved
6. **Testable**: Can be tested with/without wrappers

## Future Extensions

Potential additional shell commands:

Keep any new protocol commands conservative: every new verb expands what the wrapper is willing to execute.

## Troubleshooting

### Commands Not Executing

If navigation isn’t happening automatically, ensure you’re running `shellgame` normally (not via an alias that bypasses the wrapper), and check your default shell selection via `shellgame --shell fish|bash`.

### Debugging Protocol

View raw stderr output:
```sh
command shellgame init 2>&1 | grep SHELLGAME_EXEC
```

### Shell Compatibility

- **Fish**: Tested and working
- **Bash**: Tested and working
- **Zsh**: Should work (similar to Bash)
- **Other shells**: May need custom wrappers
