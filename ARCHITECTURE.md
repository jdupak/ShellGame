# ARCHITECTURE — ShellGame

This document provides a concise overview of ShellGame's architecture after the shell integration and CLI orchestration refactor.

Detailed descriptions:
- `docs/protocols/SHELL_PROTOCOL.md` (wrapper contract / env variables / hooks)

---

## 1) Architecture Goals

- Single source of truth for shell integration (bash/fish) without duplication.
- Separate Click parsing from game orchestration (testability, less "god-file").
- Centralize tracking (time/attempts/hints) into state layer.
- Preserve UX rules (teleport notice, paging intro, unified Enter prompt, hint semantics).

---

## 2) High-level Flow (Runtime)

1. User runs `shellgame`.
2. If the process is not "wrapped" (`SHELLGAME_WRAPPER` is not set), ShellGame launches a wrapped subshell (bash/fish) that:
   - loads the integration script,
   - suppresses banner/greeting,
   - re-runs `shellgame` inside the wrapper.
3. Inside the wrapper, `shellgame`:
   - loads or initializes state and workspace,
   - shows the current level assignment,
   - sets shell context (env variables),
   - teleports to start directory if needed (only when cwd actually changed).

---

## 3) Modules and Responsibilities (Source Map)

### CLI (Click Layer)
- `src/shellgame/cli/commands.py`
  - Click group + commands (`hint`, `submit`, `status`, `reset`, `repeat`, `show`, `remove`, …)
  - thin delegation to `GameSession`
  - holds singletons (for testing)
  - **contains UX helper** `_teleport_notice(destination: Path)` — signature must not change

### Session / Orchestration
- `src/shellgame/core/session.py`
  - `GameSession`: "what the game does"
  - boot decision (whether to launch subshell)
  - init/show/hint/submit/reset/repeat/status/remove
  - enforcement of start directory + teleport (notice callback injected from CLI)

### Subshell Integration (bash/fish)
- `src/shellgame/cli/subshell.py`
  - generates integration script from templates using `string.Template`
  - injects hook code at runtime
  - launches subshell with suppression:
    - bash: `--noprofile --norc -i --rcfile <tempfile>`
    - fish: override `fish_greeting` for session

### Shell Hooks (Single Source of Truth)
- `src/shellgame/cli/hooks.py`
  - generates bash/fish hook code (e.g., wrapper for `cd`)
  - injected into templates via `subshell.py`

### UI/UX
- `src/shellgame/ui/display.py`
  - Rich panels, notes, unified prompt `Stiskněte Enter pro pokračování...`
  - paging of intro levels (`X.0`) via delimiter `---`
  - progressive hint system + `--repeat` semantics + "no more hints" panel

### State (Persistence + Tracking)
- `src/shellgame/state/manager.py`
  - `GameState` (Pydantic) + atomic saving
  - per-level tracking:
    - `level_attempts`
    - `level_hints_used`
    - `level_started_at`
  - level completion into `levels_complete[level_id]` (time, hints, attempts, timestamp)

### Level System
- `src/shellgame/levels/*`
  - registry/loader/base + concrete levels in `levels/sections/`
  - answer validation is the level's responsibility (`validate(...)`)
  - start directory declared by level; enforcement in session layer

---

## 4) UX Contracts (Must Hold)

1. `_teleport_notice(destination: Path)` remains in `commands.py` (signature unchanged).
2. Teleport notice shown **only if cwd actually changed**.
3. Bash/fish startup must not display user banners/greetings per wrapper rules.
4. Single "Press Enter to continue": `Display.wait_for_continue()` with text:
   - `Stiskněte Enter pro pokračování...`
5. Intro levels (`.0`) are paged via `---` and wait for Enter between pages.
6. `hint` is progressive; `hint --repeat` only reprints (without consuming) and must not spam footers.

---

## 5) Where to Edit What (Quick Navigation)

- Shell wrappers/templates: `src/shellgame/cli/subshell.py`, `src/shellgame/cli/templates/`
- Hook logic (cd wrapper): `src/shellgame/cli/hooks.py`
- Game action orchestration: `src/shellgame/core/session.py`
- Click flags/args/text: `src/shellgame/cli/commands.py`
- UI text/panels/hints/paging: `src/shellgame/ui/display.py`
- Persistence + tracking: `src/shellgame/state/manager.py`
- Gameplay rules/validation: `src/shellgame/levels/sections/section*.py`
