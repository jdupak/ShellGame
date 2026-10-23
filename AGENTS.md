# AGENTS.md

Guide for coding agents and human contributors, and the single source of truth for working in this
repository; GitHub Copilot and Claude Code (2.1.277 or newer) both read it natively. Players: see [README.md](README.md).

## What this is

ShellGame is a Czech-language terminal course for first-semester students: filesystem navigation and
basic shell work in short levels, played inside a wrapped bash or fish shell. One Rust binary (edition
2024, toolchain pinned in `rust-toolchain.toml`, `Cargo.lock` committed, CI builds with `--locked`).
Player-facing text is Czech; identifiers, comments, docs and commit messages are English.

Outside the game shell, `shellgame` starts bash or fish, confined by Landlock, with an integration script
sourced. Inside it, every `shellgame ...` command is a short-lived process started by a shell function.
The binary owns all rules (levels, grading, state) and tells the shell what to do with directives on
stderr; the shell reports `cd` and `pwd` back through hooks.

## Layout

| Path | Holds |
| --- | --- |
| `src/levels/` | the course. `mod.rs` is the only registry (`COURSE`); one file per section, named by position: `s00_welcome.rs` … `s11_search.rs` (moving a section means renaming its file and its line in `COURSE`), each opening with a `//!` goals block |
| `src/game/` | the engine: `Level` and `Section`, setup, answers, checks, `cd` policies, path containment (`paths.rs`), shared messages |
| `src/session.rs`, `src/session/update.rs`, `src/cli.rs`, `src/env.rs` | every player command (including `resume`, `levels` and `update`) and the hook commands; clap definitions with Czech help; the process environment, read once in `main` |
| `src/state.rs` | `state.json` v3 |
| `src/ui/` | terminal output: markdown, panels, prompts, screens |
| `src/shell/` | `bash.sh`, `fish.fish`, shell detection, launching, the directive protocol (wire format: module docs of `src/shell/mod.rs`) |
| `src/sandbox.rs` | Landlock confinement and the `chmod` guard |
| `tests/` | `support/` (sandboxed env, binary runner, single-process shell runner), `e2e.rs`, `cli.rs`, `shell.rs`, `sandbox.rs` |
| `scripts/install.sh`, `.github/workflows/` | installer; CI and release (`ci.yml`), the docs site (`docs.yml`) |
| `.claude/skills/` | skills `add-level`, `playtest`, `release` (read by Claude Code and GitHub Copilot) |

## Commands

```bash
cargo run                                   # play from source; add -- --shell fish for fish
cargo test                                  # all suites; needs bash (fish, shellcheck, python3, curl, tar optional)
cargo test --test e2e -- solutions::3.2::   # replay level 3.2's solution in bash and fish
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps   # the docs site: a broken link between sections fails it
cargo fmt                                   # CI runs fmt --check, clippy and test with --locked
```

Run fmt, clippy and test before you finish. Run cargo in an ordinary terminal, not inside a game shell
(it cannot write `target/`). Tests that need a missing tool skip with a note; the sandbox tests skip when
Landlock is unavailable (containers, some agent sandboxes) unless `SHELLGAME_TEST_LANDLOCK=required`,
which CI sets. Without Landlock (macOS, some containers) play with `cargo run -- --unsafe`. Play by hand
with the `playtest` skill (throwaway HOME, state directory and workspace); never read, change or delete
a player's real save in `~/.config/shellgame`.

## Levels

Use the `add-level` skill. A section is one file; a level is one struct literal in its `levels` list:

```rust
// src/levels/s02_files.rs, listed once, in order, in `COURSE` in src/levels/mod.rs
//! Working with files: moving between directories with purpose, and reading and creating files.
//! ...goals block: purpose, learning objectives, Introduces / Relies on / Leaves for later...
#[rustfmt::skip]
pub const SECTION: Section = Section {
    title: "Práce se soubory",                 // no number: the UI adds "Sekce 2"
    intro: r#"
        ...markdown; pages are separated by a line `---`...
    "#,
    levels: &[
        Level {
            title: "Integrační výzva",
            text: r#"
                ### Výzva
                1. Začínáte v `level-2`. Přejděte do `challenge/room1`.
            "#,
            hints: &["Ze startu přejděte do 'challenge/room1' a přečtěte 'hint.txt'.", "..."],
            setup: &[file("challenge/room1/hint.txt", "Heslo je v room2.\n"),
                     file("challenge/room2/password.txt", "navigator\n")],
            answer: exact("navigator").nocase().error("Heslo není správné. Hledejte password.txt v room2."),
            check: &[at("challenge/room2")],
            solution: r#"
                cd challenge/room1
                cd ../room2
                shellgame submit "$(cat password.txt)"
            "#,
            success: "Výborně! Umíte propojit navigaci se čtením souborů.",
            ..LEVEL
        },
    ],
};
```

- **Section goals.** Every section file starts with a `//!` goals block (copy the shape of a neighbour;
  it is the course outline on the docs site): purpose, learning objectives, then *Introduces*, *Relies
  on* and *Leaves for later*. A new or changed level must serve its section's objectives and must not
  ask the player for what the section leaves for later (a later tool may be handed over ready-made, as
  `file *` is in `s05_inspect`; list it under *Relies on*). Update the block in the same
  change whenever a section's scope changes. Name other sections by intra-doc link,
  `[navigation](super::s01_navigation)`, never by number, so `cargo doc` (above) catches stale
  references after a move.
- **Positions, not ids.** "S.L" is the L-th level of the S-th section in `COURSE`; section S works in
  `<workspace>/level-S`. Saves store positions, so inserting, moving or removing a level or section
  shifts later saved games (accepted; say so in the commit message). Texts, hints, answers and
  solutions may mention only their own section's number (`level-N`, `Sekce N`; a test enforces it for
  texts) and it must match the section's position, so never blind search-and-replace after a move.
  Name another section in a text, and a level in a comment, by its title, never by number. No ids,
  macros, builders or data formats; ask the maintainer before changing the level format.
- **Fields:** `title, text, hints, start, setup, answer, check, cd, solution, success`; `..LEVEL`
  supplies the rest (`start: "."`, no answer, `success: "Správně!"`). A level needs an answer or a
  check; without an answer it is submitted bare, `shellgame submit`, once its checks hold, and its
  text and solution say so. Paths, `start` included, are relative to the section directory; a course
  test rejects absolute paths and `..`.
- **Text.** `text`, `intro` and `solution` are indented raw strings dedented at runtime: content starts on
  the line after `r#"` and is indented with spaces only (tests enforce both); `file` contents, `content_is`
  expectations and `success` may be such blocks too, and file contents and expectations get a final
  newline. Markdown: headings, lists, code blocks (never wrapped), no tables. Hints are plain text,
  printed literally. Intro pages are split by `---` and continue with Enter; never tell the player to run
  `shellgame submit` there, and don't start an intro with a heading: the panel title already shows
  "Sekce N: <title>" (a test enforces it). Write navigation relative to the start directory and never a
  user name.
- **Hints escalate:** the idea, the shape of the command, then the command; hint 1 is never the whole
  solution. For discovery tasks they explain how to find the answer, never the answer. Answers are natural
  discoveries (names, contents, counts, sizes), never artificial tokens.
- **Never put an answer into an `error`, `mistakes` or `format` text**, or into a failed
  check's message, and do not name the target directory in a failed `at(..)` message (a test catches
  literal leaks). The default reply to a wrong answer is the terse `Nesprávně.`.
- **Vocabulary** (constructors, then modifiers; details in `src/game/{setup,answer,check,cd}.rs`):
  - setup: `dir, file, binary, clean, build(fn(&Path))`; `.mode(0o..)`
  - answer: `exact, one_of, int, suffix, parts(sep, &[..])`; `.nocase() .mistakes(&[..]) .error()
    .format()`. A submission loses surrounding spaces and trailing slashes (Tab completion adds one);
    a text that is no number is a plain wrong answer; `.format()` is also the reply to a missing answer.
  - check (each is a `Result<(), String>`): `at, at_home, is_file, is_dir, no_file, content_is, contains,
    same, moved, mode_is, mode_has, mode_lacks, pwd_used, cd_done`; `.msg() .missing()`: the failure
    text, and the text when the file (`same`: the copy, `moved`: the destination) is missing. Checks run
    before the answer, so a correct answer never bypasses them; `cd_done` needs `cd` clauses.
  - cd (a flat list, for levels that practise one exact movement): `cd::when_in` chooses the moves it
    judges; `cd::exact, cd::absolute, cd::within` block the others with a message; `cd::arrive` counts
    the drill only when the player lands in the given directory; `cd::custom(rule, track)` is for
    stateful drills.
- **Solution (required).** The player's moves, one command per line, from the level's start
  directory, ending in `shellgame submit [answer]`. Tests replay it through the real game shell in bash
  (umask 077) and in fish, on a fresh level and again after a re-prepare (what `reset` meets); it must
  run unchanged in both (bash-only syntax goes through `bash -c '...'`, as in the wildcards). Derive
  answers with commands, not by typing them: the ones the level teaches (`grep`, `wc`, `file`), plus
  plumbing (`cut`, `sed`) where the player would read the answer off the screen. Use `"$(...)"` and
  `"$PWD"`; avoid `[[ ]]`, `$'..'`, `local`, `export X=1`; fish needs 3.4 for `$(...)`, older fish skips
  the replay of such a solution. No `less`, prompts or `sleep`. Use explicit symbolic `chmod` (`u+x`,
  `a-w`) so results do not depend on umask.
- **Custom logic** is a plain `fn` next to the section, referenced from the literal: `build` and
  `cd::custom` (the maze, the home walk, the server log). Use it only where the vocabulary falls short;
  no validation reasons about it. Data stays in the literal (file contents, expectations, messages,
  tables): a fn holds only logic, and data it needs is passed in from the literal, as the maze's plan is.
- **Setup must survive the player.** `reset` re-applies it after anything (`chmod 000`, a file where a
  directory belongs, a symlink). Declare what the player may create with `clean`, declare `.mode(..)`
  for anything whose permissions are graded, and verify discovery files with the command the lesson
  teaches (magic bytes alone may not make `file` say "image").

## Invariants

1. **Sandbox.** The game shell and everything it starts may change only `/tmp`, the state directory and
   `/dev` (Landlock, applied in the launcher between fork and exec; a typed `chmod` goes through
   `shellgame guard`). Never widen this or add a bypass: `shellgame --unsafe` is the only opt-out, and
   without Landlock the game refuses to start. Rights are `AccessFs::from_write(ABI::V5)`, best effort
   on older kernels; do not ask for newer ABIs (V9 breaks X11, Wayland, D-Bus and ssh-agent sockets) or
   signal scoping (RHEL 9 misreports it). The workspace (also `$SHELLGAME_WORKSPACE`) must resolve inside
   `/tmp` and belong to the user; the state directory must not be `$HOME` or an ancestor of it.
2. **Thin shell.** `src/shell/bash.sh` and `fish.fish` only forward events and replay directives; all
   logic lives in the binary. Directives (`__SHELLGAME_EXEC__v1 <verb> [base64...]`, a closed set of
   `cd`, `export` and `exit`, never `eval`) go to stderr only. Paths reach the shell through environment
   variables, never pasted into script text. The `cd` hook is one static function: `cd-hook pre` exiting
   3 blocks, anything else allows, so a broken hook never locks a player in. Autostart is the last block
   of each script. Keep both shells in parity: change both scripts and test the change in both. bash 3.2
   compatible (no associative arrays, `mapfile`, `${x,,}`, `&>>`, `|&`); fish 3.3 compatible (nothing
   from 3.4 or 4). bash starts with `--rcfile`, fish with `--no-config`; never `--norc` or
   `--noprofile`.
3. **Workspace containment.** All game file operations (setup, checks, `reset`, `remove`) go through the
   containment helpers in `src/game/paths.rs`: declared paths stay inside the section directory,
   symlinks are never followed, `remove` deletes only a workspace strictly inside `/tmp` (`paths::inside_tmp`,
   which the `chmod` guard shares). Do not write a second implementation.
4. **`reset` always rescues the player.** It repairs the level's setup whatever the player did (the
   workspace and the section directory included: lost permissions, a file or a symlink in their place)
   and moves them to the start directory. Never weaken the repair rules or drop the teleport.
5. **Strict `cd` levels never soft-lock.** The engine guarantees that moving back to the start directory
   is always allowed, that every move is allowed once the drill is done, and that a block shows the
   `shellgame reset` tip. A course test checks that every policy blocks something and never traps.
6. **State.** `state.json` v3 holds positions only. An older file is renamed to `state-old.json` (never
   overwritten) and a fresh game starts, with a notice that names the position to resume when the old
   file says where the player was; a newer, corrupt or unsupported file is an error and is never
   overwritten (only `remove` deletes it). A saved position that no longer exists resumes at the nearest
   following level. Saves are atomic (temporary file, rename).
7. **No level specifics in general code.** Code outside `src/levels/` never refers to a particular
   level or section (no positions, titles, file names, or mechanisms that exist for one level); a
   level's special needs are met with its own data or a fn in its section file. Validation tests are
   generic: no per-level exceptions or allowlists.

## Code

Compactness over performance. `rustfmt.toml` sets `use_small_heuristics = "Max"`; course sections are
`#[rustfmt::skip]`. Doc comments only for non-obvious contracts. One error style: `io::Error` with
context for real failures, `Result<(), String>` for feedback shown to the player. No Python-parity
hacks. New dependencies need a reason (chrono and indexmap were dropped on purpose; timestamps are unix
seconds).

Multi-line text is a raw string with real line breaks, never `\n` escapes, and a raw string also spares
`\"`. In indented code it is an indented block that starts on the line after `r#"` and is dedented at
runtime, as level `text` is; at module level, a raw string starting at column 0. Keep `\n` for a single
trailing newline, separators in code (`join("\n")`), control bytes and whitespace-exact test inputs.

Every shell script passes shellcheck at its default severity: `tests/shell.rs` checks the bash
script, CI every tracked `*.sh`. A `# shellcheck disable=` needs a reason on the line above. Level
solutions are typed commands (`cd dir`, `ls | grep`), not scripts, and are not shellchecked.

## Tests

Only tests that protect players: solution replay in real bash and fish (`tests/e2e.rs`: every level, a
full playthrough per shell, strict `cd` scenarios), the sandbox (`tests/sandbox.rs`), CLI flows
(`tests/cli.rs`), the shell protocol and hooks (`tests/shell.rs`: both scripts in real shells, shellcheck),
course validation (generic checks over every level), and a few unit tests for tricky pure
code (setup repair, containment, answer order, `same`/`moved`, `cd` lifting, markdown widths). Prefer
one generic data-driven check to per-level cases. Do not add tests that pin wording or allowlists of
current content, grep source text, or mock the thing under test. A new level needs no new test: its
solution and the course validation are the test, and when validation fails it is usually right.

## Release

A push to `main` builds Linux x86_64 and aarch64 (static musl) and macOS archives and publishes release
`build-N` (N is the CI run number) with `shellgame-linux-{x86_64,aarch64}.tar.gz`. macOS archives are held
back by `PUBLISH_MACOS` in `.github/workflows/ci.yml` until a macOS content review. A dry run is the CI
workflow started with `workflow_dispatch` on any branch: it builds everything and publishes nothing.
`scripts/install.sh` and `shellgame update` download `releases/latest/download/shellgame-<os>-<arch>.tar.gz`,
so asset names are an interface. The same push publishes the docs site (`cargo doc`, the README as its
front page) to GitHub Pages through `.github/workflows/docs.yml`. See the `release` skill.

## Commits

Keep commits focused: one concern each, an imperative English subject, a body that says what and why.
Commit only when asked. Never push, merge or tag without the maintainer asking: a push to `main`
publishes to students.
