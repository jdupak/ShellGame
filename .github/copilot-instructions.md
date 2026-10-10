# Copilot Instructions — ShellGame

You are working in **ShellGame**, an interactive terminal learning game written in **Rust**. The game runs inside a wrapped **subshell** (bash or fish) to control navigation, observe commands, and persist progress.

Player-facing text is **Czech**. Code, identifiers, comments, docs and commit messages are English.

Your job is to make changes that preserve existing UX and gameplay rules. Prefer small, targeted edits and keep player-facing text concise.

Read first: `ARCHITECTURE.md` (the whole picture), `docs/AUTHORING.md` (before adding or changing a level or section) and `docs/protocols/SHELL_PROTOCOL.md` (before any shell integration change). **Before touching the shell scripts, read section 4.2 below carefully.**

---

## 1) Tech stack and commands

- **Rust**, edition 2024, Unix only (Linux and macOS). One crate at the repo root: library plus a thin `main`. The toolchain is pinned in `rust-toolchain.toml`; `Cargo.lock` is committed and CI builds with `--locked`.
- CLI: **clap** (derive), with Czech help. `cd-hook` and `fd-hook` are parsed by hand before clap, because a `cd` target can be `""`, `-`, `-P` or a directory named `pre`.
- State: serde, serde_json, indexmap, chrono. UI: anstyle and anstream, pulldown-cmark and our own text layout. rustix for termios and `fstat`.
- Shell integration: a wrapped **bash** or **fish** (see 4.2).
- Run before you finish:

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

The suite drives real shells: bash is required, and fish, shellcheck, python3 (pseudo-terminal tests), curl and tar are used when present. There is no Python, uv, PyInstaller or Makefile any more.

---

## 2) Where code lives (edit the right place)

| You change | Edit |
| --- | --- |
| level content, hints, fixtures, solutions | `src/levels/sNN_<topic>/mod.rs` and the level's `NN-slug.md` |
| level logic that no builder method expresses | next to the level (`s01_navigace/maze.rs`, `walk.rs`) |
| the builder, answers, requirements, fixtures, `cd` policies | `src/engine/` (`level.rs`, `answer.rs`, `require.rs`, `fixture.rs`, `cd.rs`, `fd.rs`) |
| catalog lints | `src/engine/lint.rs` (`Catalog::validate`) |
| path containment | `src/paths.rs` (the only implementation) |
| evidence markers | `src/markers.rs` |
| shared Czech messages | `src/messages.rs` |
| commands: parsing, then behaviour | `src/cli.rs`, then `src/session.rs` |
| screens, panels, wording, paging, prompts | `src/ui/screens.rs`, `src/ui/panel.rs`, `src/ui/prompt.rs` |
| Markdown rendering of assignments | `src/ui/markdown.rs` |
| persistence | `src/state.rs` |
| shell scripts | `src/shell/bash.sh` **and** `src/shell/fish.fish` |
| the protocol, launching, shell detection | `src/shell/protocol.rs`, `launch.rs`, `render.rs`, `detect.rs` |
| the process environment | `src/env.rs` |

### Level IDs

- IDs are explicit: `level(8, "...")` in section 2 is `2.8`. Never derive an ID from declaration order, never renumber, never reuse a retired number (section 1 has no `1.11`). IDs are persisted in player state.
- Section intros are `N.0` and are created with `intro(..)`. Sections are numbered 0, 1, 2, ... without gaps. Inserting a section into existing numbers needs a state migration.
- A new section is a module `sNN_<topic>` with `pub fn section()`, registered in `catalog()` in `src/levels/mod.rs`.

---

## 3) Architectural principles

### 3.1 Declarative levels, plain Rust for the exceptions

- A level is one builder chain: `level(n, title).text(..).start(..).hints([..]).file(..).answer(..).require(..).cd(..).solution(..)`. Where the vocabulary falls short, use an ordinary Rust function (`check`, `.feedback`, `.rule`, `.on_move`, `exact_with`, `.solution_with`). Do not reintroduce a data format for levels.
- Do not hardcode level IDs in `session.rs` or `cli.rs`. Behaviour specific to a level lives with the level.
- **Unconditional safeguards.** Filesystem, location, permission and evidence checks are `.require(..)`. They run before the answer, so a correct or empty answer never bypasses them.
- **Exact completion locations.** Navigation levels use `at(path)`, which compares canonical paths. Never validate only the last directory name: a same-named directory elsewhere must not satisfy the level.
- **One path vocabulary.** Every path a level declares (`.start`, fixtures, requirements, `cd` rules) is relative to the level's section root. `""` and `"."` are the root; `.start_at_workspace()` is for a level above its section. Never restate the section directory in a level. The lint enforces it.
- **Fresh evidence.** A level becomes active through `Level::prepare()`: it clears every marker the level references, repairs the section root, then applies the section fixture and the level fixture. Do not apply fixtures by hand from session code.
- **Safe fixtures.** Absolute paths, `..`, symlinked components and a fixture that targets its own root are rejected. All containment goes through `paths::resolve_within`; do not add a second implementation. Exact directory modes use `.dir_mode(..)`, applied after the files.
- **Declarative movement rules.** Strict navigation is a `policy()` on the level. The engine performs the anti-soft-lock contract. There is no hand-written shell `cd` logic. Stateful grading (the 1.9 walk) uses `.rule(..)`, `.on_move(..)` and `.manual_evidence()` inside that contract.
- **Level-local evidence.** The `cd` and `fd` markers derive from the level ID (`.cd-1.4`, `.fd-10.4`). Do not add global marker names or ad-hoc dotfiles; `Marker::Pwd` is the only shared one.
- **Prove solvability.** Every regular level has a solution, a shell script that the replay runs through the real game shell in bash and fish. Derive answers with the commands the level teaches.

### 3.2 No ambient state

Everything the game needs from the process is in `Env`, built once in `main`. Engine and session code never read `std::env`, the process working directory or `HOME`; they take a `Ctx` or an `Env`. Tests therefore run in parallel with no `chdir` and no `set_var` (`set_var` is `unsafe` in Rust 2024).

### 3.3 Feedback is data

A wrong answer or an unmet requirement is `Check::Fail(text)`, not an error. Errors are for real failures (state, filesystem). Text for the player is printed as written; only a level's `.text(..)` goes through the Markdown renderer.

### 3.4 Thin shell integration

- Keep both scripts as small as possible. They capture events (`cd`, `pwd`) and forward them to the binary through hooks and protocol directives.
- No game logic, validation or state in shell scripts. The `cd` function is static: it never changes between levels.

---

## 4) Hard UX rules (do not break)

### 4.1 Teleport notices

The game may `cd` the player to a level's start directory.

- Show the notice **only if the directory actually changed**. The session tracks the player's directory in memory and updates it when it emits a `cd`.
- Format (`screens::teleport_notice`): the message in **yellow**, the destination in **violet**, **no** "from -> to", **no** reason text.

### 4.2 Shell startup (CRITICAL)

Both shells use **one integration script** that does, in order: setup, helper functions, `pwd`, `cd`, the `shellgame` function, completions, **autostart last**.

```text
bash --rcfile <script> -i
fish --init-command 'function fish_greeting; end; source $SHELLGAME_INIT'
```

- **Never** add `--norc` (it disables `--rcfile`) or `--noprofile` (the player keeps their PATH and environment).
- The script is the rcfile; there is no second template. Paths reach the shell through `SHELLGAME_BIN`, `SHELLGAME_INIT` and `SHELLGAME_FD_HOOK`, never pasted into script text.
- Autostart must be the last block, after the `shellgame` function exists. Calling the binary earlier would print protocol lines on the terminal. Never call `shellgame` in the fish init-command.
- **Protocol.** The binary writes `__SHELLGAME_EXEC__v1 <verb> [base64...]` lines to **stderr only**. The `shellgame` function captures stderr, executes the closed set `cd`, `export`, `echo`, `pwd`, `exit`, and prints other lines unchanged. No `eval`, no generic verb: arguments stay base64 until a verb handler decodes them.
- **`remove`** closes the shell through the `exit` directive the binary emits. No command name is special-cased in the shell. A declined `remove` must exit non-zero.
- **The `cd` hook** is one static function: `cd-hook pre <pwd> <prev> -- <args>` before the move (exit status 3 blocks, anything else allows: fail-open), `cd-hook post <old> <new> -- <args>` after a successful move. Teleports use the shell's own `cd` and never run the hook.
- **fd evidence.** Scripts run `"$SHELLGAME_FD_HOOK"`; never the `shellgame` function, which captures stderr.
- The workspace comes only from `$SHELLGAME_WORKSPACE`. Never reconstruct it from `$USER` in a script: a stale or foreign directory could collect evidence.
- **bash 3.2** (macOS) must keep working: no associative arrays, `mapfile`, `${x,,}`, `&>>`, `|&`. Newer `shopt` options stay guarded.
- **Keep bash and fish in parity.** A change to one script is made in the other, with a `shell_test!` in `tests/shell.rs` that runs in both. New commands also go into the completion word lists of both scripts.

### 4.3 One unified "Press Enter to continue"

- Exact text: `Stiskněte Enter pro pokračování...`, from `ui::prompt` (`enter_prompt`, `wait_for_enter`). Do not write custom prompts in session code.
- Prompts print to stdout and appear only when stdin and stdout are terminals, so piped input is never swallowed.
- Prompts read in non-canonical mode with `ISIG` off: Ctrl-C is a key, not a signal. The process always exits normally.
- Therefore the `export` and `cd` directives of a level are emitted **before** any prompt.

### 4.4 Section intro paging (`.0` levels)

- Pages are split by a line containing only `---` (`ui::split_pages`), shown in order with the Enter prompt between them. After the last page the session waits once more, then submits automatically.
- Intro text and hints say **Enter**, never `shellgame submit`. An intro has no answer, requirements or `cd` policy.

### 4.5 Hints: progressive plus repeat

- `shellgame hint` reveals the next hint and counts it only when newly revealed. `hint --repeat` reprints the revealed hints without consuming a new one.
- Helper text under hint panels goes through the screen's note helper (indent handled in code); do not bake leading spaces into strings.
- With `--repeat`, print no per-hint footers, the repeat tip **once at the end**, and "Potřebujete další pomoc? ..." only if another hint exists.
- Hint text is printed literally. `[ab]`, `[a-z]` and `*` must survive, in normal and repeated hints.
- No more hints: a **yellow** panel `Pro tento level již nejsou k dispozici žádné další nápovědy.` with a note under it showing `shellgame hint --repeat` in violet.
- Zero revealed and zero available are different states. `--repeat` with nothing revealed must show `screens::no_hints_revealed` and never `hints[0]`, which would be a free hint and desynchronise the counter in `shellgame status`. A level with no hints at all uses `screens::no_hints_available`.
- Hints scaffold progressively: concept, then structure, then the specific command. Hint 1 does not reveal the full command.

### 4.6 Success and failure panels

- Success is a **green** centred panel (`screens::success`) with the level's **own** success text and `time`, `hints` and `attempts`. Never substitute a generic "Správně!" for a level's text.
- `attempts` counts **failed** attempts only (`Chybné pokusy`).
- Failure is a **red** centred panel (`screens::failure`). A blocked `cd` is a red panel titled `ShellGame (<id>)` plus `messages::CD_RECOVERY_TIP`.
- Prefer panels for major state changes over raw lines.

### 4.7 Skipping bonus levels

- `Level::is_bonus()` is `optional`. `shellgame skip` advances only bonus levels; others get `screens::not_skippable`.
- Skipping is not a completion: no `levels_complete` entry and no success panel. The bonus panel advertises `shellgame skip` by itself.

### 4.8 Terminal state

Completing the last level sets `completed_at`. After that `submit`, `skip`, `hint` and showing the level are inert and only re-display `screens::game_complete`.

---

## 5) Level validation rules (gameplay correctness)

### 5.1 Empty submit policy

A level that takes an answer must not complete on a bare `shellgame submit`. Give its answer `.required("Musíte zadat ...: shellgame submit <hodnota>")` with the example command. Use `.allow_empty()` only where the file system is the real answer. Never let internals (`None`, `Debug` output, Rust error names) reach the player.

### 5.2 Never soft-lock the player

Strict `cd` levels accept one command shape. That must never become a dead end.

- Use `.cd(policy()...)`; the engine applies the contract: moves out of scope are allowed, a move to the start directory is always allowed, moves are allowed once the level's `cd` marker exists, and a block always carries the `shellgame reset` tip.
- `shellgame reset` rebuilds the level **and** moves the player back to its start. Never remove the teleport from `reset`.
- Lifting movement restrictions is safe because requirements still gate `submit`.
- The catalog test `every_cd_policy_blocks_something_and_never_traps_the_player` enforces this. Do not weaken it.
- Fixtures must be able to restore a level after anything a player did (`chmod 000`, a file where a directory belongs, a removed file), so `reset` always works. Do not remove the repair rules in `src/engine/fixture.rs`.

### 5.3 Never reveal the answer

- A wrong submission must not print the expected value. The default is the terse `Nesprávně.`. Write a specific `.error(..)`, `.mistake(..)`, `.feedback(..)` or requirement message only when it teaches without giving the answer away.
- Failed `at(..)` requirements must not name the target directory. The lint checks answers in messages, `.secret()` answers in hints, and custom `at` messages.
- Discovery hints teach how to *find* the answer. Command examples are fine for explicitly assigned filesystem tasks.

### 5.4 "Consistency is maintained by the game"

Do not rely on where the player ended last time. If a level needs a start directory, declare `.start(..)` and a fixture. Avoid instructions that tell the player to fix state the game can guarantee.

- Write navigation commands relative to the start directory. Never hard-code a user name; for `cd -` tasks, spell out the moves that make the previous directory predictable.
- When a command prints a path as the answer, state its reference directory or accept equivalent output. Suffix matching respects `/` boundaries (`suffix(..)`).

### 5.5 Shell-specific syntax

- Mark bash-only syntax in the affected level and in the section intro. Do not present bash's `?` or `[...]` globs as portable to fish.
- For filesystem-only exercises, offer `bash -c 'command'` to fish players and let them submit from the wrapped shell. Do not recommend an unwrapped bash: it would lose the game's hooks.
- Solutions must be valid in both shells (or use `.solution_bash(..)`). Use explicit symbolic `chmod` targets (`u+x`, `a-w`), so the outcome does not depend on `umask`.

### 5.6 Discovery fixtures

- Confirm discovery answers with the tool the lesson teaches. Magic bytes alone may not make `file` print "image", and a script's description also contains `ASCII text`.
- Teach directory types through the first character of `ls -l`. Counting fixtures remove extra entries on reset (`.clean(..)`), and decoy counts differ from the right one.

---

## 6) CLI behaviour constraints

- Bare `shellgame` shows the current level; outside the game shell it launches the game shell instead. Other commands run directly outside it, with protocol output off.
- `repeat` can show any level or section. `show` is intentionally restricted to the current level (`--level`) or the current section's intro (`--section`), and needs exactly one flag.
- `submit` joins several words and accepts a leading `-`.
- Exit statuses: 0 for all normal gameplay (a wrong answer included), 1 for errors and a declined `remove`, 2 for usage errors, 3 only from `cd-hook pre` when a move is blocked.
- A game shell started by the Python version may still be running after the binary is replaced. Keep the legacy `cd-hook [TARGET] [PWD]` and `cd-hook --post-move [PWD]` forms, the `SHELLGAME_LEVEL` export and the `--devmode` prefix working.

---

## 7) Tests: keep what protects players

- Prefer one generic, data-driven check over per-level hand-written cases. A solution replay or a catalog lint covers every level at once.
- Do not add tests that pin wording or current content, grep source text, or mock the thing under test. Do add a regression test for a bug that could hurt a player.
- Every behaviour of the shell scripts is tested once per shell (`shell_test!`). Engine rules are unit-tested next to the code.
- Adding a level needs no new test: its solution and the lints are the test. If a lint fails, the lint is usually right.

---

## 8) Known gotchas (avoid regressions)

- A level may make its own files read-only (`chmod`). Fixtures replace files and never write into an existing read-only one, so `reset` keeps working.
- `.keep_file(..)` preserves an existing regular file, not an incorrect type. `no_file(..)` and `no_dir(..)` require that nothing remains, a symlink included.
- Fixtures end with explicit modes (`0755` directories, `0644` files) unless declared, so a level that grades or depends on permissions declares the modes it needs.
- A deleted working directory is a normal failure: `Env::cwd` is `None` and rules report `messages::CWD_MISSING`. Show the real filesystem error otherwise; do not label every error as a deleted directory.
- Read failures include the `shellgame reset` tip (`messages::FILE_READ_ERROR`).
- State: `state.json` is schema 2.1 and byte-compatible with the earlier Python version (golden files in `tests/state/`). A schema change needs a new version and a migration. Missing state starts a game; **corrupt or unsupported state is an error and is never overwritten**. Every state-changing action performs **one** atomic save, and advancing saves before the next level is prepared.
- `shellgame remove` deletes a workspace only strictly inside `/tmp` or `$TMPDIR`.
- Marker files live in the workspace saved in the state; do not reconstruct workspace paths from `$USER`.
- Command-specific tasks that distinguish *how* an outcome was reached use a `cd` policy or `fd` evidence. Final-state validation is enough only when equivalent commands are intentionally accepted.
- Do not edit `src/levels/` text with blind search and replace: the modules mix code paths with Czech text containing the same literals.

### Isolated playtesting

- Use a throwaway `HOME`, `XDG_CONFIG_HOME` and `SHELLGAME_WORKSPACE` (and `TMPDIR` if you test `remove`). Build first and run `target/debug/shellgame`; `cargo run` needs the real `HOME` for cargo and rustup.
- Check that the state directory is inside the sandbox before the first save. Never read, overwrite, remove or restore a player's real save. Clean only paths you created in the sandbox.
- Try both shells (`--shell bash`, `--shell fish`). `--devmode` adds the shortcuts `j <id>`, `n`, `p`, `r`, `s`, `e`, `g`.

---

## 9) Working checklist

1. Find the right place in section 2. For a level or section, follow `docs/AUTHORING.md`.
2. Preserve UX: screens in `ui/screens.rs`, pauses through `ui::prompt`, short Czech text, commands shown in violet in tips.
3. Run the three commands of section 1. For shell changes, run `cargo test --test shell` with both shells installed.
4. Spot-check: `hint` and `hint --repeat`, a bare `submit`, the teleport notice on a real change only, intro paging, startup in bash and fish.

## 10) Maintenance

This file is the source of truth for contributors and agents; keep it accurate. If you change an architectural pattern, add a hard UX rule or introduce a new project principle, update it in the same change.
