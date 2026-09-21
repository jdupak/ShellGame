# Copilot Instructions — ShellGame

You are working in **ShellGame**, an interactive terminal learning game written in **Python** using **Click** (CLI) and **Rich** (UI). The game runs inside a wrapped **subshell** (bash or fish) to control navigation, observe commands, and persist progress.

Your job is to make changes that preserve existing UX and gameplay rules. Prefer small, targeted edits and keep user-facing text concise (Czech).

**Before making shell integration changes, read this entire section carefully.**

---

## 1) Tech stack (what to assume)

- Language: **Python 3.10+**
- Dependency Management: **uv** (use `uv sync`, `uv add`, etc.)
- Task Runner: **make** (use `make test`, `make lint`, etc.)
- CLI: **Click**
- UI: **Rich** panels + consistent helper notes
- Tests: **pytest** (run via `make test`)
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

- Level setup + validation rules + shell hooks (OOP):
  - `src/shellgame/levels/sections/section*.py`
  - Logic specific to a level (hooks, validation) must live in the `Level` class, not global handlers.

- Declarative level infrastructure:
  - `src/shellgame/levels/collector.py` — section registration and stable local IDs
  - `src/shellgame/levels/fixture.py` — repeatable workspace setup/cleanup
  - `src/shellgame/levels/completion.py` — answers and unconditional requirements
  - `src/shellgame/levels/cdpolicy.py` — declarative `cd` restrictions
  - `src/shellgame/levels/solution.py` — reference solutions that prove a level is solvable
  - `src/shellgame/paths.py` — the single path-containment implementation

- Authoring guide (read before adding a level or section):
  - `docs/AUTHORING.md`

- Section intro markdown (content only; supports paging via `---`):
  - `src/shellgame/levels/content/section*_intro.md`

- Persistent progress model/load/save:
  - `src/shellgame/state/manager.py`
  - Missing state may initialize a game; corrupt state must raise an error and must not be overwritten.
  - State-changing actions should perform one atomic save.

### Level IDs
- Declare the persistent section number with `Section(number, ...)`.
- Declare each stable level suffix locally with `@section.level(number)`.
- Never derive IDs from declaration order; IDs are persisted in player state.
- Section intros end with `.0` (e.g., `1.0`, `2.0`) and support paging.
- Regular levels are `X.Y`.
- Sections have no `get_levels()` trailer; the loader reads the module-level
  `section` object. Do not reintroduce one.

---

## 3) Architectural Principles

### 3.1 OOP over Special Casing
- **Encapsulate logic in Level classes**: Do not hardcode level IDs in `session.py` or `commands.py`.
- **Use overrides**: If a level needs special behavior (e.g., custom validation, hooks), override the corresponding method in its class.
- **Declarative configuration**: Regular levels should combine
  `WorkspaceFixture`, `Completion`, typed answer rules, and typed requirements.
  Override `setup()` or `validate()` only for genuinely custom behavior.
- **Unconditional safeguards**: Put filesystem, cwd, permissions, and evidence
  checks in `Completion.requirements`. Correct or empty answers must never
  bypass these checks.
- **Exact completion locations**: Navigation levels must use `AtDirectory`
  with a section-root-relative path. Never validate only `Path.cwd().name`,
  because a same-named directory outside the workspace must not satisfy the
  level.
- **One path vocabulary**: every path a level declares — `start_directory`,
  fixtures, requirements, solution steps — is relative to that level's section
  root. `""` is the section root; use the `WORKSPACE_ROOT` sentinel for the
  rare level that starts above its own section. Never restate the section
  directory name inside a level, and never build a path with `state.workspace /
  ...` in a section module.
- **Fresh evidence**: Runtime level activation goes through `Level.prepare()`,
  which clears declarative evidence, applies the section fixture, applies the
  level fixture, and then runs custom setup. Do not call `setup()` directly
  from session orchestration.
- **Safe fixtures**: Fixture paths are relative to the section root. Absolute
  paths, `..`, cleanup of the fixture root, and symlink escapes are rejected.
  All containment goes through `paths.resolve_within()`; do not add a second
  implementation. Declare exact directory modes with `DirectoryFixture` in
  `WorkspaceFixture.directory_fixtures`; modes are applied after child files so
  parent permission recovery cannot overwrite the requested final mode.
- **Declarative movement rules**: strict navigation levels declare a
  `cd_policy` (`levels/cdpolicy.py`) instead of writing `_handle_cd`. The
  engine then applies the anti-soft-lock contract structurally. Write a custom
  `_handle_cd` only for stateful grading that a policy cannot express (level
  1.9 is the sole example).
- **Level-local evidence**: a `cd` policy derives its marker from the level ID
  (`cd_marker("1.4")` → `cd-1.4`). Do not add global marker constants and do
  not invent ad-hoc dotfile names.
- **Prove solvability**: a level with custom `validate()`/`setup()`, a
  `cd_policy`, evidence, or a filesystem requirement must declare a `Solution`
  (`levels/solution.py`). It is replayed against a throwaway workspace, both
  fresh and after `reset()`.

### 3.2 Thin Shell Integration
- **Minimal templates**: Keep bash/fish templates as small as possible.
- **Delegate to Python**: Shell scripts should only capture events (like `cd`) and forward them to Python via hooks or protocol directives.
- **No game logic in shell**: Do not implement validation or state management in shell scripts.

---

## 4) Hard UX rules (do not break)

### 4.1 Teleport notices
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

### 4.2 Subshell startup (CRITICAL — read carefully)

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
Python emits versioned operations to stderr using:
`__SHELLGAME_EXEC__v1 <verb> [base64-argument...]`.
The shell wrapper function:
1. Captures stderr to a temp file
2. Decodes and dispatches only supported verbs (`cd`, `export`, `echo`, `pwd`, `exit`)
3. Echoes other stderr lines normally

Do not add a generic `eval` fallback. Protocol arguments must remain encoded
until the shell-specific verb handler decodes them.

#### `remove` command special handling
The `shellgame remove` command deletes the workspace (which may be the user's cwd). To avoid `getcwd` errors and leaked protocol lines, the **shell wrapper detects `remove` success and exits the subshell directly** — we do NOT rely on a protocol directive for this.

### 4.3 One unified “Press Enter to continue”
- Exact text: `Stiskněte Enter pro pokračování...`
- Use UI helper: `Display.wait_for_continue()`
- Do not reintroduce custom prompts in CLI code.

### 4.4 Section intro paging (`.0` levels)
- Intro markdown pages are split by a line containing only `---`
- Show pages in the standard instruction flow
- Between pages call `Display.wait_for_continue()`
- After the last page the engine waits once more, then auto-submits. Intros
  continue with **Enter**, never `shellgame submit`.
- Intro copy and intro hints must say Enter. `shellgame submit` is only for
  real levels.

### 4.5 Hints: progressive + repeat semantics
- `shellgame hint` reveals the **next** hint and increments hint counter.
- `shellgame hint --repeat` reprints already revealed hints **without consuming new ones**.

Formatting + footers:
- Under hint panels, helper text must use `Display.note(...)` (indent handled programmatically).
- Do not bake leading spaces into helper strings.
- When using `--repeat`:
  - do **not** print per-hint helper footers repeatedly
  - print the repeat tip **once at the end**
  - only show “Potřebujete další pomoc? …” if another hint still exists
- Render level-authored hint text literally. Rich markup must not consume shell
  syntax such as `[ab]` or `[a-z]`, in either normal or repeated hints.

No-more-hints UX:
- Show a **yellow framed** panel containing:
  - `Pro tento level již nejsou k dispozici žádné další nápovědy.`
- Under it, show a *note* with the tip containing the command in violet:
  - `[violet]shellgame hint --repeat[/violet]`
- Do **not** show “To jsou všechny nápovědy…” in this flow.

Zero-revealed and zero-available are **different** states:
- `--repeat` with nothing revealed yet must call `Display.show_no_hints_revealed()`.
  It must **never** print `hints[0]`, which would be a free hint and would
  desynchronise the hint counter shown by `shellgame status`.
- A level with no hints at all uses `Display.show_no_hints_available()`.

### 4.6 Success/failure panels
- Success: **green framed panel**, centered text (`Display.show_success`)
- Failure: **red framed panel**, centered text (`Display.show_failure`)
- Prefer panels for major state changes rather than raw console lines.
- `Display.show_success` must be given the level's own message plus
  `time_sec` / `hints_used` / `attempts`. Never substitute a generic
  “Správně!” for a level's `success_message`.
- `attempts` counts **failed** attempts, so it is labelled `Chybné pokusy`.

### 4.7 Skipping bonus levels
- `Level.is_bonus` is `optional or extension`.
- `shellgame skip` advances only on bonus levels; core levels must show
  `Display.show_not_skippable(...)`.
- Skipping is not a completion: it must not write to `levels_complete` and
  must not show a success panel.
- Instructions for bonus levels advertise `shellgame skip` via `Display.note`.

### 4.8 Terminal state
- Completing the last level sets `GameState.completed_at`.
- `submit`, `skip`, `hint`, and `show_current_level` must then be inert and
  only re-show `Display.show_game_complete(state)`.

---

## 5) Level validation rules (gameplay correctness)

### 5.1 “Empty submit” policy (do not workaround in CLI)
Some levels require an explicit answer argument; `shellgame submit` must not silently succeed.

Enforced expectations:
- Level **1.1** requires `shellgame submit level-1` (empty submit is invalid)
- Level **1.4** requires explicit answer
- Level **1.5** requires explicit answer
- Level **1.3** requires user to be in directory `alpha` when submitting

If you need to change this behavior, configure the level's `Completion`; use a
custom `validate()` only when the result cannot be expressed declaratively.

### 5.2 Never leak `None` to the player
If a level expects an answer and `answer is None`, configure the answer rule's
`required_message` with a dedicated, friendly message such as:
- “Musíte zadat odpověď…” plus an example command.

Do not generate user-facing strings that include `None` (e.g., `"'None' ..."`)—special-case it.
(Level **1.2** already demonstrates this pattern.)

### 5.3 Never soft-lock the player
Strict `cd` levels only accept one command shape. That must never become a
dead end.

- **Prefer `cd_policy`.** A declared policy applies everything below
  structurally, so a new level cannot forget it. The rules below are the
  contract a custom `_handle_cd` must honour by hand.
- Reject a move only through `block_cd(self.id, message)`, which always adds
  the `shellgame reset` recovery tip.
- Before rejecting, call
  `self.cd_enforcement_lifted(<marker>, target=target, pwd=pwd, state=state)`.
  It lifts enforcement once the evidence marker exists (the drill is done) or
  when the move returns the player to the level start directory.
- `shellgame reset` re-exports shell context and re-teleports the player.
  Do not remove `ensure_user_in_reasonable_place()` from `GameSession.reset()`.
- Lifting *movement* restrictions is safe because `Completion.requirements`
  still gate submission.
- `tests/test_cd_hooks_no_softlock.py` enforces all of the above and fails if a
  new blocking hook is added without coverage.

### 5.4 Never reveal the answer in an error
A wrong submission must not print the expected value. Answer rules take
`error_message: str | None = None`; leaving it `None` yields the terse
`Messages.INCORRECT`. Only write an explicit `error_message` when it teaches
without giving the answer away.
`tests/test_level_invariants.py` enforces the default path.
- This also applies to `mistakes` feedback and failed navigation requirements.
  `AtDirectory` must not name the expected directory in its default error.
- Discovery hints teach how to find the answer instead of printing it; retain
  command examples for explicitly assigned filesystem tasks.

### 5.5 “Consistency is maintained by the game”
Do not rely on “where the user ended last time”.
If a level needs a specific start directory, enforce it via:
- `start_directory` attribute in the level class, and/or
- a declarative fixture

Avoid instructions that tell the user to manually correct state the game can guarantee.
- Write navigation commands relative to the declared start directory, and never
  hard-code a workspace username. For `cd -` tasks, ensure the instructed moves
  establish the intended previous directory.
- When a command prints a path as the answer, state its reference directory or
  accept equivalent output after reasonable navigation. Path suffix matching
  must respect `/` component boundaries.

### 5.6 Shell-specific syntax
- Explicitly mark Bash-only syntax in the affected level and section intro.
  Do not present Bash's `?` or `[...]` globs as portable to Fish.
- For filesystem-only exercises, offer `bash -c 'command'` from Fish, then
  submit in the existing wrapped shell. Do not recommend an unwrapped
  interactive Bash, which would lose the game's hooks.
- Portable exercises remain usable in Fish. Displayed command replays must
  exit successfully, not merely leave a final state that passes validation.
- Use explicit symbolic `chmod` targets (`u+x`, `a-w`) so the taught outcome
  does not depend on the player's `umask`.

### 5.7 Discovery fixtures
- Confirm discovery answers using the actual tool taught by the lesson.
  Binary signatures alone do not guarantee that `file` recognizes an image.
  Define whether text counts exclude scripts containing `ASCII text`.
- Teach directory types through the first character of a long listing (`ls -l`
  or `ls -la` for hidden entries); `d` denotes a directory. Counting fixtures
  must remove extra entries on reset so their answers stay
  consistent, and decoy counts should differ from the correct count.

---

## 6) CLI behavior constraints: `repeat` and `show`
- `repeat` can re-display arbitrary level/section content via options.
- `show` is intentionally restricted:
  - only current level (`--level`) OR current section intro (`--section`)
  - does not accept arbitrary IDs
- Validate exactly **one** flag is provided.
- Keep wording concise and user-friendly.
- When adding a command, also add it to the completion word list in **both**
  `bash_integration.template` and `fish_integration.template`.

---

## 7) How to work effectively (agent checklist)

When implementing a change:

1. Identify the category:
   - **UI text/panels** → `ui/display.py`
   - **shell / teleport / subshell** → `cli/commands.py`
   - **level rules** → `levels/sections/section*.py`
   - **shared answer/requirement rule** → `levels/completion.py`
   - **workspace setup/reset declaration** → `levels/fixture.py`
   - **`cd` restriction** → `levels/cdpolicy.py`
   - **reference solution** → `levels/solution.py`
   - **intro content** → `levels/content/*.md`
   - **progress/persistence** → `state/manager.py`

   When adding a level or a section, follow `docs/AUTHORING.md`.

2. Preserve established UX:
   - use `Display.note(...)` for helper notes
   - use `Display.wait_for_continue()` for pauses
   - keep Czech text short; style commands in violet when shown as tips

3. After changes, run a quick regression pass:
   - Run tests: `make test`
   - Check linting: `make lint`
   - Verify `shellgame hint` and `shellgame hint --repeat` behaviors above
   - submit behavior when `answer is None`
   - teleport notice prints only on actual cwd change
   - `.0` intro paging via `---`
   - bash/fish startup suppression still holds

---

## 8) Known gotchas (avoid regressions)

- `_teleport_notice(destination: Path)` signature must not change.
- Ensure `shell.cd(...)` is always invoked with parentheses.
- A level may make its own files read-only (`chmod`). Fixtures must still be
  able to restore them, so `shellgame reset` keeps working — replace files,
  never write into an existing read-only one.
- Fixtures repair file/directory mixups at declared paths, including parent
  directories, without replacing their root or following symlinks.
  `DirectoryFixture` may unlink and rebuild a symlink at one of its declared
  path components, but must never target the fixture root or follow or modify
  the symlink target.
  `overwrite=False` preserves existing regular files, not incorrect types.
  Custom generated files should also use `FileFixture` for this recovery.
  Fixture cleanup and replacement recursively recover traverse/write permissions
  so `reset()` succeeds even after `chmod 000` or read-only directories.
- Use `DirectoryPermissionMode` for an exact directory mode. Like every
  filesystem requirement, its path is section-root-relative and the level must
  declare a `Solution`; `Chmod` works for directory solution steps.
- `Level.prepare()` cleans up accidental non-directory files and restores
  `stat.S_IRWXU` on the section root directory if damaged, ensuring `shellgame reset`
  works even if the player damages the section root itself.
- `Messages.FILE_READ_ERROR` must always include the `shellgame reset` recovery tip.
- Hints should scaffold progressively (Concept -> Scaffolding/structure -> Specific command)
  rather than revealing the full command in Hint 1.
- `FileExists` and `DirectoryExists` with `should_exist=False` check that no
  entry (file, directory, or symlink) remains at the path.
- Structural invariant tests must discover levels through
  `get_registry().list_levels()`, never by grepping method source. Source-based
  discovery silently stops covering a level the moment its behaviour moves from
  a hand-written method to a declaration.
- Never blind-regex the section modules: they mix code paths with user-facing
  Czech text containing the same literals. Edit them AST-precisely or by hand.

### Shell integration gotchas (IMPORTANT)
- **NEVER** add `--norc` to bash launch — it disables `--rcfile`.
- **NEVER** split bash into two templates (integration + rc) — use one template directly as rcfile.
- **NEVER** call `shellgame` in fish init-command directly — autostart must be inside the template after the function is defined.
- **NEVER** emit `shell.exit()` from `remove` — the wrapper handles exit directly to avoid protocol leakage.
- Protocol directives go to **stderr only** — do not emit to stdout.
- Scripts that must preserve inherited stdout/stderr use the executable path
  in `$SHELLGAME_FD_HOOK`; do not call the exported `shellgame` wrapper, which
  captures stderr by design.
- Marker files live in `state.workspace`; do not reconstruct workspace paths from `$USER`.
- The shell templates read the workspace **only** from `$SHELLGAME_WORKSPACE`.
  Never reintroduce a `/tmp/shellgame-$USER` fallback: a stale or foreign
  directory could then collect evidence markers.
- The bash protocol dispatch relies on word splitting; keep it wrapped in
  `set -f` / `set +f` so a payload can never glob against the filesystem.
- Command-specific tasks that distinguish *how* an outcome is reached must use
  a level hook plus marker evidence; final-state validation is sufficient only
  when equivalent commands are intentionally accepted.
- Fish and bash templates should follow the same pattern: setup → functions → autostart at end.
- Do not label every filesystem exception as a deleted cwd. Show the actual
  filesystem failure; use `Messages.CWD_MISSING` only when cwd is unavailable.

### Isolated playtesting
- Use a throwaway HOME, XDG_CONFIG_HOME, XDG_DATA_HOME, XDG_CACHE_HOME and
  workspace. Set them before importing modules that create global services.
- Create a new `StateManager` after isolation is configured, and assert its
  `state_file` is inside the owned temporary root before any save or CLI run.
  Changing environment variables does not relocate an existing manager.
- Never read, overwrite, remove or restore a player's real save during
  playtesting. Clean only paths created within the isolated test root.

---

## 9) Maintenance (keep this file current)

- If you change architectural patterns (e.g., moving configuration from CLI to Level classes), **update this file**.
- If you add new hard UX rules, **add them here**.
- If you introduce new project values or principles, **update this file**.
- This file is the source of truth for future agents; keep it accurate.
