---
name: add-level
description: Write, change, move or remove a ShellGame level or section (a `Level { .. ..LEVEL }` struct literal in src/levels/sNN_<section>.rs) and verify it with the course validation and the bash and fish replay. Use when asked to add a level, fix a level's text, hints, answer, checks, cd policy or solution, reorder levels, or add a section.
---

# Add or change a level

The format and the rules (positions, Czech text, hints, no leaked answers, solutions, vocabulary) are in
`AGENTS.md`, section "Levels". Read it first; this skill is the procedure.

## 1. Find the place and the position

- Sections are `src/levels/sNN_<name>.rs`, numbered by position (`s00_welcome.rs` … `s11_search.rs`) and
  listed in that order in `COURSE` in `src/levels/mod.rs`; moving a section means renaming its file. Section S works in `<workspace>/level-S`.
- Read the section's goals block (the `//!` comment at the top of its file) before touching a level. The
  level must serve one of its learning objectives and must not ask the player for anything under *Leaves
  for later*. A change that widens or narrows what the section teaches updates the block in the same change
  (new commands go under *Introduces*, a borrowed later tool under *Relies on*).
- Read the levels around the spot and copy their tone, difficulty and use of the vocabulary. Address the
  player in the formal plural like the existing levels ("Přejděte", "Vytvořte").
- Appending to a section's `levels` list shifts nothing. Inserting, moving or removing a level (or a
  section) shifts the later positions "S.L" and therefore later saved games. That is accepted; say so in
  your report and in the commit message.

## 2. Decide what makes the level "solved"

- A discovery the player reports (a name, a content, a count, a size): `answer:` with `exact`, `one_of`,
  `int`, `suffix` or `parts`. A missing answer gets the default "Musíte zadat odpověď"; add
  `.format("...")` only to show the shape of a `parts` answer or how to submit. Judging drops trailing
  slashes (Tab completion adds them), so an answer never ends in one.
- Something the player must do to files or location: `check: &[..]` (`at`, `is_file`, `no_file`,
  `content_is`, `moved`, `mode_is`, ...). Checks run before the answer, so a correct answer never
  bypasses them. When the files are the real answer, the level has no answer: the text and the solution
  say a bare `shellgame submit`, never a label.
- Both, when the player must do something and report what they found.
- `cd: &[..]` only for a level that practises one exact movement command; then add `cd_done()` to `check`.
  The engine keeps a strict level from trapping the player (moving back to the start directory is always
  allowed) and a course test checks that every `cd` list blocks something.

## 3. Write the literal

Copy the shape from AGENTS.md or a neighbour. Fields: `title`, `text`, `hints`, `start`, `setup`,
`answer`, `check`, `cd`, `solution`, `success`, then `..LEVEL` (a literal that sets every field leaves it
out: clippy rejects a needless one).

- `text` and `solution` are indented raw strings: content starts on the line after `r#"`, spaces only.
- Paths, `start` included, are relative to the section directory (`level-S`); a level starts there unless
  `start` names a directory below it. Everything the level needs is in its own `setup`: no shared files.
- `setup`: everything the level needs; `clean` what the player may create; `.mode(..)` for anything whose
  permissions the level grades. It must survive `chmod 000` and similar, because `reset` re-applies it.
  Multi-line file contents are indented blocks like `text`; `file` adds the final newline.
- Text: goal, numbered steps, how to submit. Commands in backticks, navigation relative to the start
  directory, no user names. A level that teaches bash-only syntax says so, and offers fish players
  `bash -c '...'`.
- Hints (plain text, printed literally): the idea, then the shape of the command, then the command.
  Hint 1 is never the whole solution; for discovery tasks hints explain how to find the answer.
- Replies (`.error`, `.mistakes`, `.format`, check messages): short, teach without revealing. Never
  contain the answer, and a failed `at(..)` must not name the target directory.
- `success`: say what the command did, not praise.
- Custom logic (maze, generated files, stateful `cd` drills) is a plain `fn` at the bottom of the same
  section file, referenced from the literal.

## 4. Write the solution

What a student would type, one command per line, from the start directory, ending in
`shellgame submit [answer]`. Derive the answer with the commands the level teaches. Keep it valid in bash
and fish (`"$(...)"`, `"$PWD"`; no `[[ ]]`, `$'..'`, `local`, `export X=1`), non-interactive, and
independent of umask (`chmod u+x`, not numbers). Bash-only syntax that the level teaches goes through
`bash -c '...'`, so the solution still runs in fish.

## 5. Verify

```bash
cargo fmt
cargo test                           # course validation over every level, plus all other suites
cargo test --test e2e -- solutions::3.2::   # replay one level (here 3.2) in bash and fish, fresh and re-prepared
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps   # the goals blocks and their links between sections
```

If a validation test fails, it is usually right. Typical causes: an answer inside an error, mistake or
format text; an answer that ends in `/`; a solution that does not end in `shellgame submit`; a level
with neither answer nor check; tabs or text on the first line of an indented raw string; an absolute
or `..` path; a `level-N` or `Sekce N` mention that is not its own section's number (another section is
named by its title).

## 6. After moving, inserting or removing a level or section

Positions shift, so re-read the texts that mention them:

```bash
grep -nE 'level-[0-9]+|Sekc[ei] [0-9]+' src/levels/*.rs
```

Every hit, in texts, hints, answers (an answer can be `level-1`) and solutions, must match the position
of its own section; a test enforces this for texts. Another section is named by its title, never by
number, and a level in a comment likewise. Do not use blind search-and-replace: these literals are
shared. A new section is a new file `sNN_<name>.rs` with a `pub const SECTION: Section` (all four
fields), declared and listed at its position in `src/levels/mod.rs`. Its `intro` follows the neighbours:
the concept, a mental model, the commands introduced, pages split by `---`, and no heading at the top
(the panel title shows "Sekce N: <title>"). Every later section moves one position down: rename its
file, and its `level-N` directory changes too. A new section file starts with its goals block (copy a
neighbour's shape, link other sections as `[navigation](super::s01_navigation)`); after a rename or a
move fix those links and the `pub mod` lines, and let `cargo doc` (step 5) flag every stale one.

## 7. Look at it

Play the level the way a student does with the `playtest` skill; `shellgame resume S.L` takes you to it.
