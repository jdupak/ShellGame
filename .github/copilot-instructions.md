# Copilot Instructions — ShellGame

You are working in **ShellGame**, an interactive terminal learning game written in **Python** using **Click** (CLI) and **Rich** (UI). The game runs inside a wrapped **subshell** (bash or fish) to control navigation, observe commands, and persist progress.

Your job is to make changes that preserve existing UX and gameplay rules. Prefer small, targeted edits and keep user-facing text concise (Czech).

**Before making shell integration changes, read this entire section carefully.**

---

## 1) Tech stack (what to assume)

- Language: **Python**
- CLI: **Click**
- UI: **Rich** panels + consistent helper notes
- Tests: **pytest** (in `tests/`)
- Shell integration: wrapped subshell (**bash**/**fish**) — see section 3.2 for critical details
- Persistence: state saved/loaded between sessions

---

## 2) Where code lives (edit the right place)

### Common responsibilities

- CLI flow, teleport:
  - `src/shellgame/cli/commands.py`

- Subshell launch, shell detection:
  - `src/shellgame/cli/subshell.py`

- Shell integration templates (bash/fish):
  - `src/shellgame/cli/templates/bash_integration.template`
  - `src/shellgame/cli/templates/fish_integration.template`

- UI panels, wording consistency, indentation, paging:
  - `src/shellgame/ui/display.py`

- Level setup + validation rules (learning goals) + start directory:
  - `src/shellgame/levels/sections/section*.py`

- Section intro markdown (content only; supports paging via `---`):
  - `src/shellgame/levels/content/section*_intro.md`

- Persistent progress model/load/save:
  - `src/shellgame/state/manager.py`

### Level IDs
- Section intros end with `.0` (e.g., `1.0`, `2.0`) and support paging.
- Regular levels are `X.Y`.

---

## 3) Hard UX rules (do not break)

### 3.1 Teleport notices
ShellGame may auto-`cd` the user to a level start directory.

Rules:
- Show a teleport notice **only if cwd actually changed**.
- Notice formatting:
  - message in **yellow**
  - destination path in **violet**
  - **no** “from -> to”
  - **no** “reason” text
- Helper must remain: `_teleport_notice(destination: Path)` in `commands.py`
- Do not change the signature; call sites must not pass `reason=...`.

### 3.2 Subshell startup (CRITICAL — read carefully)

Both bash and fish use a **single integration template** that includes:
1. Shell setup (disable interfering features)
2. Function definitions (`shellgame`, `pwd`, `cd` hooks, `__shellgame_eval`)
3. Autostart block **at the end** (calls `shellgame` function after it's defined)

#### Bash launch flags
```bash
bash --rcfile <integration_script> -i
```

**CRITICAL**: Do NOT use `--norc` — it disables `--rcfile` entirely!
**CRITICAL**: Do NOT use `--noprofile` — let users keep their PATH/env setup.

The integration script is used directly as the rcfile. There is NO separate rc template.

#### Fish launch
```bash
fish --init-command "function fish_greeting; end; source <integration_script>"
```

Fish greeting is suppressed inline. Autostart happens inside the integration template.

#### Why autostart is at the end of the template
The `shellgame` **function** (defined in the template) captures stderr and processes protocol directives (`__SHELLGAME_EXEC__...`). If autostart called the external binary directly (before the function is defined), protocol directives would leak to the terminal.

#### Protocol directives
Python emits commands to stderr with prefix `__SHELLGAME_EXEC__`. The shell wrapper function:
1. Captures stderr to a temp file
2. Processes lines starting with `__SHELLGAME_EXEC__` as shell commands
3. Echoes other stderr lines normally

#### `remove` command special handling
The `shellgame remove` command deletes the workspace (which may be the user's cwd). To avoid `getcwd` errors and leaked protocol lines, the **shell wrapper detects `remove` success and exits the subshell directly** — we do NOT rely on a protocol directive for this.

### 3.3 One unified “Press Enter to continue”
- Exact text: `Stiskněte Enter pro pokračování...`
- Use UI helper: `Display.wait_for_continue()`
- Do not reintroduce custom prompts in CLI code.

### 3.4 Section intro paging (`.0` levels)
- Intro markdown pages are split by a line containing only `---`
- Show pages in the standard instruction flow
- Between pages call `Display.wait_for_continue()`

### 3.5 Hints: progressive + repeat semantics
- `shellgame hint` reveals the **next** hint and increments hint counter.
- `shellgame hint --repeat` reprints already revealed hints **without consuming new ones**.

Formatting + footers:
- Under hint panels, helper text must use `Display.note(...)` (indent handled programmatically).
- Do not bake leading spaces into helper strings.
- When using `--repeat`:
  - do **not** print per-hint helper footers repeatedly
  - print the repeat tip **once at the end**
  - only show “Potřebujete další pomoc? …” if another hint still exists

No-more-hints UX:
- Show a **yellow framed** panel containing:
  - `Pro tento level již nejsou k dispozici žádné další nápovědy.`
- Under it, show a *note* with the tip containing the command in violet:
  - `[violet]shellgame hint --repeat[/violet]`
- Do **not** show “To jsou všechny nápovědy…” in this flow.

### 3.6 Success/failure panels
- Success: **green framed panel**, centered text (`Display.show_success`)
- Failure: **red framed panel**, centered text (`Display.show_failure`)
- Prefer panels for major state changes rather than raw console lines.

---

## 4) Level validation rules (gameplay correctness)

### 4.1 “Empty submit” policy (do not workaround in CLI)
Some levels require an explicit answer argument; `shellgame submit` must not silently succeed.

Enforced expectations:
- Level **1.1** requires `shellgame submit level-1` (empty submit is invalid)
- Level **1.4** requires explicit answer
- Level **1.5** requires explicit answer
- Level **1.3** requires user to be in directory `alpha` when submitting

If you need to change this behavior, do it in the level’s `validate()` implementation, not in Click parsing.

### 4.2 Never leak `None` to the player
If a level expects an answer and `answer is None`, show a dedicated, friendly message such as:
- “Musíte zadat odpověď…” plus an example command.

Do not generate user-facing strings that include `None` (e.g., `"'None' ..."`)—special-case it.
(Level **1.2** already demonstrates this pattern.)

### 4.3 “Consistency is maintained by the game”
Do not rely on “where the user ended last time”.
If a level needs a specific start directory, enforce it via:
- `start_directory` attribute in the level class, and/or
- level setup logic

Avoid instructions that tell the user to manually correct state the game can guarantee.

---

## 5) CLI behavior constraints: `repeat` and `show`
- `repeat` can re-display arbitrary level/section content via options.
- `show` is intentionally restricted:
  - only current level (`--level`) OR current section intro (`--section`)
  - does not accept arbitrary IDs
- Validate exactly **one** flag is provided.
- Keep wording concise and user-friendly.

---

## 6) How to work effectively (agent checklist)

When implementing a change:

1. Identify the category:
   - **UI text/panels** → `ui/display.py`
   - **shell / teleport / subshell** → `cli/commands.py`
   - **level rules** → `levels/sections/section*.py`
   - **intro content** → `levels/content/*.md`
   - **progress/persistence** → `state/manager.py`

2. Preserve established UX:
   - use `Display.note(...)` for helper notes
   - use `Display.wait_for_continue()` for pauses
   - keep Czech text short; style commands in violet when shown as tips

3. After changes, run a quick regression pass focusing on:
   - `shellgame hint` and `shellgame hint --repeat` behaviors above
   - submit behavior when `answer is None`
   - teleport notice prints only on actual cwd change
   - `.0` intro paging via `---`
   - bash/fish startup suppression still holds

---

## 7) Known gotchas (avoid regressions)

- `_teleport_notice(destination: Path)` signature must not change.
- Ensure `shell.cd(...)` is always invoked with parentheses.

### Shell integration gotchas (IMPORTANT)
- **NEVER** add `--norc` to bash launch — it disables `--rcfile`.
- **NEVER** split bash into two templates (integration + rc) — use one template directly as rcfile.
- **NEVER** call `shellgame` in fish init-command directly — autostart must be inside the template after the function is defined.
- **NEVER** emit `shell.exit()` from `remove` — the wrapper handles exit directly to avoid protocol leakage.
- Protocol directives go to **stderr only** — do not emit to stdout.
- Fish and bash templates should follow the same pattern: setup → functions → autostart at end.

---

## 8) Maintenance (keep this file current)

- If you change architectural patterns (e.g., moving configuration from CLI to Level classes), **update this file**.
- If you add new hard UX rules, **add them here**.
- This file is the source of truth for future agents; keep it accurate.