# Authoring ShellGame levels

This guide covers adding a level or a section. It assumes you have read
[ARCHITECTURE.md](../ARCHITECTURE.md) for the wider picture.

The rule that drives everything below: **a level declares what "solved" looks
like, and the engine decides how to check it.** Hand-written logic is allowed
(section 12), but every line of it is a line the lints cannot reason about.

Every `rust` block in this guide is compiled and run by `cargo test --doc`
(the guide is included as the module `shellgame::authoring`), so the examples
cannot drift from the API. Blocks marked `rust,ignore` show code that only
makes sense inside `src/levels/`.

Player-facing text is Czech: assignments, hints, messages and success texts.
Code, identifiers, comments and docs are English.

Sections: 1 Anatomy of a level, 2 Paths, 3 Fixtures, 4 Answers, 5 Requirements,
6 `cd` policies, 7 `fd` evidence, 8 Solutions, 9 Intros and paging, 10 Adding a
section, 11 What the lints protect, 12 Escape hatches, 13 Trying a level.

---

## 1. Anatomy of a level

A level is one function in its section module, and its assignment is a
Markdown file next to it:

```text
src/levels/s02_soubory/
├── mod.rs                    section() and one fn per level
├── 00-uvod.md                the intro, level 2.0
├── 01-sourozenci.md          the assignment of level 2.1
├── ...
└── 08-integracni-vyzva.md    the assignment of level 2.8
```

The Markdown file is pulled in with `include_str!`:

```rust,ignore
pub fn section() -> Section {
    Section::new(2, "level-2")
        .intro(intro("Práce se soubory", include_str!("00-uvod.md")))
        .levels([l1(), l2(), l3(), l4(), l5(), l6(), l7(), l8()])
}

fn l8() -> Level {
    level(8, "Integrační výzva")
        .text(include_str!("08-integracni-vyzva.md"))
        // ...
}
```

The real thing is `l8` in
[`src/levels/s02_soubory/mod.rs`](../src/levels/s02_soubory/mod.rs). Here it is
with the text inline, so the example compiles:

```rust
use shellgame::engine::prelude::*;

fn challenge() -> Level {
    level(8, "Integrační výzva")
        .text("### Úkol\n1. Přejděte do `challenge/room1`.\n2. Odevzdejte heslo z `room2`.")
        .start(".")
        .hints([
            "Ze startu přejděte do 'challenge/room1' a přečtěte 'hint.txt'.",
            "Do sousedního 'room2' se dostanete například příkazem 'cd ../room2'.",
            "V 'room2' přečtěte soubor 'password.txt' pomocí 'cat'.",
        ])
        .file("challenge/room1/hint.txt", "Heslo je v sousedním adresáři room2.\n")
        .file("challenge/room2/password.txt", "navigator\n")
        .file("challenge/room2/decoy.txt", "Toto není heslo.\n")
        .answer(
            exact("navigator")
                .ignore_case()
                .mistake("toto není heslo", "To je obsah decoy.txt, ne password.txt.")
                .error("Heslo není správné. Hledejte password.txt v room2.")
                .required("Musíte zadat heslo: shellgame submit <hodnota>")
                .secret(),
        )
        .require(at("challenge/room2"))
        .solution("cd challenge/room1\ncat hint.txt\ncd ../room2\nshellgame submit \"$(cat password.txt)\"")
        .success("Výborně! Dokončili jste Sekci 2.")
}

let section = Section::new(2, "level-2").levels([challenge()]);
assert!(Catalog::new(vec![section]).validate().is_empty());
```

| Part | Method | Section |
| --- | --- | --- |
| identity | `level(number, title)` | 1 |
| assignment (Markdown) | `.text(md)` | 1 |
| hints (plain text) | `.hints([..])` | 1 |
| where the player starts | `.start(path)`, `.start_at_workspace()` | 2 |
| files the level needs | `.file(..)`, `.dir(..)`, `.clean(..)`, ... | 3 |
| the typed answer | `.answer(rule)`, `.allow_empty()` | 4 |
| what must be true | `.require(..)` | 5 |
| strict movement | `.cd(policy()...)` | 6 |
| output redirection | `.on_fd(..)` | 7 |
| proof it can be finished | `.solution(..)` | 8 |
| bonus level | `.optional()` | 1 |
| text on completion | `.success(msg)` | 1 |

**Ids.** `level(8, ..)` gives the stable suffix, not a position. Ids are stored
in `state.json`, so never renumber a level, never reuse a retired number
(section 1 has no `1.11`, and that is fine), and do not worry about the order
of the list: the catalog sorts numerically (`1.12` after `1.10`).

**Text.** `.text()` is Markdown, rendered with paragraph reflow, lists, code
blocks, headings and block quotes (no tables). Existing levels use `### Cíl`,
`### Úkol` (numbered steps) and `### Odevzdání`. Code blocks are never
wrapped, so directory trees survive. Everything else the player reads (hints,
success text, feedback) is printed exactly as written, so `[ab]` and `*` in a
hint stay as they are.

**Hints** reveal one at a time. Scaffold them: the idea first, then the shape
of the command, then the command. Hint 1 should not already be the solution.
For discovery tasks, explain how to *find* the answer, never the answer.
Command examples are fine where the assignment itself prescribes the command.

**Success text** is the level's own (`Správně!` if you set none). Use it to
say what the command did, not to praise.

**Bonus levels** (`.optional()`) are the only ones `shellgame skip` accepts.

### Adding a level

1. Pick the next free number in the section. Never reuse a retired one.
2. Write the assignment in `NN-slug.md` (Czech words without diacritics).
3. Add `fn lN() -> Level` to the section's `mod.rs`, with a solution.
4. List it in `section().levels([..])`, in ascending order.
5. Run `cargo test`, then play it (section 13).

---

## 2. Paths: one vocabulary

**Every path a level declares is relative to its section root.** The section
declares the root once:

```rust
use shellgame::engine::prelude::*;

// Section 1 has the root "level-1". Levels never repeat it.
let deep = level(5, "Hluboký ponor")
    .start(".")                       // <workspace>/level-1
    .dir("gamma/deep/a/b/c")          // <workspace>/level-1/gamma/deep/a/b/c
    .answer(exact("c"))
    .require(at("gamma/deep/a/b/c"))  // the player stands there
    .solution("cd gamma/deep/a/b/c\nshellgame submit c");
let section = Section::new(1, "level-1").levels([deep]);
assert!(Catalog::new(vec![section]).validate().is_empty());
```

Fixtures, requirements, `.start(..)`, and `cd` policies all use it. Rules:

- `""` and `"."` are the section root. `a//b/./c/` means `a/b/c`.
- Absolute paths, `..` and NUL bytes are rejected when declared.
- A symlinked section root, or a symlink anywhere along a declared path, is
  refused when the path is used. Nothing is ever followed.
- A level that starts *above* its section (the walk home, the intro of
  section 1) says so with `.start_at_workspace()`. Do not abuse `""` for it.
- Section 0 is the workspace itself: root `""`.

Restating the section root (`.dir("level-1/gamma")` inside section 1) is a
mistake the catalog lint reports:

```rust
use shellgame::engine::prelude::*;

let wrong = level(5, "Hluboký ponor")
    .dir("level-1/gamma")             // already relative to level-1
    .answer(exact("c"))
    .solution("shellgame submit c");
let errors = Catalog::new(vec![Section::new(1, "level-1").levels([wrong])]).validate();
assert!(!errors.is_empty());
```

All containment goes through one function, `resolve_within` in
[`src/paths.rs`](../src/paths.rs). Do not write a second one.

In assignments and hints, write navigation relative to the start directory,
never `cd level-1/...` for a player who is already in `level-1`, and never a
hard-coded user name in an absolute path.

---

## 3. Fixtures: the files a level needs

A fixture says what the section directory looks like when the level starts.
Both the section and the level may declare one; the section's is applied
first.

| Method | Declares |
| --- | --- |
| `.dir(p)`, `.dirs([..])` | directories, parents included |
| `.file(p, content)` | a file, rewritten at every prepare |
| `.file_mode(p, content, mode)` | the same with an explicit mode |
| `.keep_file(p, content)` | a file written only when missing (for files the player edits) |
| `.dir_mode(p, mode)` | an exact directory mode, applied last |
| `.clean([..])` | paths removed before anything is created |
| `.fixture(Fixture)` | merge a prebuilt or generated fixture |

`content` is a `&str`, `String`, `&[u8]`, `&[u8; N]` or `Vec<u8>`. **Generated
content is a plain Rust function**: no special mechanism.

```rust
use shellgame::engine::prelude::*;

fn server_log() -> String {
    (1..=200)
        .map(|line| format!("[{line:03}] INFO: Normal operation\n"))
        .collect()
}

fn maze_fixture() -> Fixture {
    Fixture::new()
        .dirs(["maze/entry", "maze/entry/hall"])
        .file("maze/entry/GO_TO_DIR_hall", "")
}

let log = level(4, "Hledání ve velkém souboru")
    .start("logs")
    .file("logs/server.log", server_log())
    .file("logs/photo.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF")   // bytes
    .file("logs/blob.bin", vec![b'x'; 12345])
    .file_mode("logs/run.sh", "#!/bin/sh\necho hi\n", 0o755)
    .dir_mode("logs/locked", 0o550)
    .clean(["logs/report.txt"]);                               // the player may create it
let maze = level(7, "Bludiště").fixture(maze_fixture());

// Scaffolding shared by every level of a section.
let shared = Section::new(1, "level-1").fixture(
    Fixture::new()
        .dirs(["alpha", "delta"])
        .keep_file("alpha/inside.txt", "first step"),
);
```

Real examples: `server_log()` in
[section 5](../src/levels/s05_zkoumani/mod.rs) and the generator in
[`maze.rs`](../src/levels/s01_navigace/maze.rs).

### What `prepare` does

`Level::prepare` runs for the first visit, for `shellgame reset` and for
`dev jump`. It must work after anything a player can do, so:

1. Every marker the level references is removed (section 5), so evidence from
   an earlier attempt cannot complete the level.
2. The section directory is repaired (a file in its place is deleted, a
   locked directory is made usable). Section 0 is never repaired this way.
3. The section fixture, then the level fixture, are applied.

The repair rules of a fixture:

- **Chmod before delete.** Section 8 teaches `chmod`, so a directory may be
  `000` and a file read-only. Whatever has to be replaced is made accessible
  first, so `reset` works after a `chmod` level.
- **Never follow a symlink.** A link at a declared path is replaced, and its
  target is never read, written or deleted. Files are created with `O_EXCL`.
- **File and directory mix-ups are replaced** by what was declared.
- **A fixture cannot target its own root.**
- **Directory modes come last, deepest first**, so restricting a parent never
  blocks the chmod of a child.
- **Modes are explicit.** Created or repaired directories end `0o755` and files
  `0o644` unless you declare another mode, so the result never depends on the
  player's umask. `.keep_file` leaves an existing regular file alone.

Practical consequences:

- Anything the player is expected to create belongs in `.clean(..)`, so `reset`
  removes it. Counting tasks must clean away extra entries that would change
  the answer, and decoy counts should differ from the right one.
- Declare the mode of every file whose permissions a level grades or relies on
  (`.file_mode`, `.dir_mode`), as section 8 does.
- Verify discovery fixtures with the command the student is taught. Magic bytes
  alone may not make `file` print "image", and a script's description also
  contains `ASCII text`.

---

## 4. Answers

`.answer(rule)` judges what the player types after `shellgame submit`.

| Constructor | Accepts |
| --- | --- |
| `exact(text)` | the text |
| `one_of([..])` | any of the texts |
| `int(n)` | the integer; parsed like Python's `int()`, so `+5`, `1_000` and `007` are numbers |
| `suffix(text)` | `text` or anything ending in `/text` |
| `parts(sep, [rules..])` | several values joined by `sep`, each judged by its own rule |
| `exact_with(fn)` | a text computed from the situation (`ctx.home_name()`) |

| Modifier | Effect |
| --- | --- |
| `.ignore_case()` | compare case-insensitively |
| `.trim_trailing_slash()` | `delta/` counts as `delta` |
| `.mistake(value, msg)` | a tailored reply to one specific wrong answer |
| `.feedback(fn)` | a computed reply for classes of wrong answers |
| `.error(msg)` | the reply to any other wrong answer |
| `.invalid(msg)` | the reply to a non-number (`int` only) |
| `.required(msg)` | the reply to an empty submission |
| `.format_message(msg)` | the reply to the wrong number of parts (`parts` only) |
| `.secret()` | the player must find this answer (lint, below) |
| `.accepts([..])`, `.rejects([..])` | examples that the lint checks |

A text rule judges in this order: surrounding whitespace is stripped; a literal
`.mistake` that matches wins; the right answer passes; `.feedback` may reply;
otherwise `.error`, or the terse default `Nesprávně.`.

```rust
use shellgame::engine::prelude::*;

// Wrong answers of one shape get one reply. The function gets the normalized answer
// and returns Some(message) to speak, or None to fall through to `.error`.
fn shape(answer: &str) -> Option<String> {
    if answer.starts_with('d') && !answer.ends_with('a') {
        Some(format!("'{answer}' začíná na 'd', ale nekončí na 'a'."))
    } else {
        None
    }
}

let delta = exact("delta")
    .trim_trailing_slash()
    .mistake("data", "'data' není adresář: jeho řádek v 'ls -l' začíná '-'.")
    .feedback(shape)
    .required("Musíte zadat název adresáře: shellgame submit <hodnota>")
    .secret()
    .accepts(["delta", "delta/", " delta "])
    .rejects(["Delta", "data"]);

let text = exact("start").ignore_case();
let any_of = one_of(["human-readable", "--human-readable"]).ignore_case();
let code = int(42)
    .mistake(137, "137 je číslo řádku, ne kód na konci.")
    .invalid("Odpověď musí být číslo.");
let name = suffix("report.txt");                       // also "docs/report.txt"
let pair = parts(",", [int(2), int(3)]).format_message("Formát: chyby,výstup");
let home = exact_with(|ctx| ctx.home_name());          // the player's own $HOME name
```

**Never reveal the answer in a rejection.** That includes `.error`, `.mistake`
and `.feedback` texts and the failure message of a requirement. A player who
guesses once must not be handed the solution. The catalog lint catches the
literal cases (section 11).

**`.secret()`** marks an answer the player has to discover. The lint then
rejects a hint, or a ready-made `shellgame submit <answer>` line in the
assignment, that gives it away. Leave it off where the assignment states the
answer.

**`.accepts` and `.rejects`** are documentation that is also a test: the
lint runs them against the rule, so `one_of([..])` lists and tricky wrong
answers stay honest. They are never evaluated at run time, and rules that
depend on the machine (`exact_with`) are skipped.

**Empty submissions.** A level with an answer rejects a bare
`shellgame submit` with `.required(msg)` (a generic text if you set none).
Set `.required` whenever the level needs an argument, and show the command.
`.allow_empty()` lets a bare submit complete the level once the requirements
hold: the choice for levels where the file system is the real answer.
A level with no answer rule at all is graded on its requirements alone.

---

## 5. Requirements

`.require(..)` adds a condition that is **always checked, in order, before the
answer**. The first failure is what the player sees. A correct answer never
bypasses a requirement, so "submit `level-1` without using `pwd`" fails.

| Requirement | Passes when |
| --- | --- |
| `at(p)` | the player stands in `p`; canonical paths are compared, so a same-named directory elsewhere never matches |
| `at_home()` | the player stands in `$HOME` |
| `evidence(Marker::Pwd, msg)` | the marker exists (section 6 and 7 write the others) |
| `cd_evidence(msg)`, `fd_evidence(msg)` | this level's `cd` or `fd` drill was performed |
| `file(p)`, `dir(p)` | a regular file, or a directory, exists |
| `no_file(p)`, `no_dir(p)` | nothing remains there (a leftover symlink counts) |
| `content(p)` | `.exact(..)`, `.contains([..])`, `.excludes([..])`, `.strip()` hold |
| `line_count(p, n, msg)` | the file has exactly `n` lines |
| `same(a, b)` | `b` is an exact copy of `a` (names, bytes, dotfiles; symlinks compared, not followed) |
| `moved(from, to)` | `from` is gone and `to` exists |
| `perm_bits(p)` | `.has(bits)` and `.lacks(bits)` hold |
| `mode(p, m)`, `dir_mode(p, m)` | the permission bits are exactly `m` |
| `check(fn)` | an arbitrary check (section 12) |

Most take `.message(..)` to replace the default text; `content` also has
`.missing(..)` and `.unreadable(..)`. A custom `at(..).message(..)` must not
name the directory the player has to find (lint).

```rust
use shellgame::engine::prelude::*;

// No answer: the file system is the judge (level 4.4).
let delete = level(4, "Mazání souborů")
    .start("cleanup")
    .file("cleanup/stary_log.txt", "stará data\n")
    .require(no_file("cleanup/stary_log.txt").message("Soubor stary_log.txt stále existuje."))
    .solution("rm stary_log.txt\nshellgame submit");

// Several conditions, in order (level 10.1).
let errors = level(1, "Přesměrování chyb")
    .start(".")
    .file_mode("buggy.sh", "#!/bin/bash\necho out\necho err >&2\n", 0o755)
    .answer(exact("errors.log"))
    .require(
        content("errors.log")
            .excludes(["out"])
            .message("Soubor obsahuje i normální výstup.")
            .missing("Soubor neexistuje."),
    )
    .require(content("errors.log").contains(["err"]))
    .solution("./buggy.sh 2> errors.log\nshellgame submit errors.log");

// An answer and a permission requirement; a bare submit is fine (level 8.2).
let chmod = level(2, "Nastavení spustitelnosti")
    .start("permissions")
    .file_mode("permissions/run_me.sh", "#!/bin/bash\necho Run me", 0o644)
    .answer(exact("run_me.sh"))
    .require(perm_bits("permissions/run_me.sh").has(0o100).message("Vlastník nemá právo 'x'."))
    .allow_empty()
    .solution("chmod u+x run_me.sh\nshellgame submit 'run_me.sh'");

// The rest of the vocabulary at a glance. A real level needs a handful, never all.
let gallery = level(1, "Přehled")
    .require(at_home())
    .require(dir("backup"))
    .require(no_dir("tmp"))
    .require(line_count("errors.log", 2, "errors.log nemá přesně dva řádky."))
    .require(same("copying/a.txt", "copying/a.bak"))
    .require(moved("moving/old.txt", "moving/new.txt"))
    .require(perm_bits("config").lacks(0o222))
    .require(mode("deploy.sh", 0o755))
    .require(dir_mode("locked", 0o750))
    .require(content("notes.txt").exact("hotovo").strip());
```

### Evidence markers

Some facts can only be witnessed by the shell: that `pwd` ran, that a `cd`
used one exact command, that output went to `/dev/null`. Each leaves an empty
dotfile in the workspace root:

| `Marker` | File | Written by |
| --- | --- | --- |
| `Marker::Pwd` | `.pwd_used` | the shell's `pwd` function; shared by all levels |
| `Marker::Cd` | `.cd-<id>` | the `cd` hook (section 6) |
| `Marker::Fd` | `.fd-<id>` | the `fd` hook (section 7) |
| `Marker::Own("walk")` | `.<id>-walk` | your own code, for a level's private bookkeeping |

Names derive from the level id, so two levels cannot collide. Markers are
evidence, not saved state. `prepare` clears every marker the level references
(evidence requirements, its `cd` and `fd` markers, `after` rules). A private
marker that only your own code writes has to be listed:

```rust
use shellgame::engine::prelude::*;

let walk = level(9, "Cesta z kořene domů").resets([Marker::Own("walk")]);
```

---

## 6. Restricting movement: `cd` policies

Some navigation levels grade *which command* the player used, not only where
they ended up. Declare that; never write shell code for it.

```rust
use shellgame::engine::prelude::*;

// Level 1.12: from the deepest directory, after `pwd`, in one exact command.
let summary = level(12, "Souhrn")
    .start(".")
    .dir("gamma/deep/a/b/c")
    .require(at("gamma"))
    .require(cd_evidence("Nejdřív použijte `pwd`, přejděte do `c` a vraťte se jedním příkazem."))
    .cd(policy()
        .from_dir("gamma/deep/a/b/c")
        .after(Marker::Pwd, "Nejdřív použijte příkaz 'pwd'.")
        .exact("../../../..", "Z adresáře 'c' použijte jeden příkaz 'cd ../../../..'."))
    .solution("pwd\ncd gamma/deep/a/b/c\ncd ../../../..\nshellgame submit");

// Level 2.2: only `cd -` is of interest, and only from location-B.
let back = level(2, "Rychlý návrat")
    .start("location-A")
    .dirs(["location-A", "location-B"])
    .require(at("location-A"))
    .require(cd_evidence("Nejdřív přejděte do `location-B` a vraťte se příkazem `cd -`."))
    .cd(policy()
        .target("-")
        .source("location-B", "Příkaz 'cd -' použijte až z adresáře location-B."))
    .solution("cd ../location-B\ncd -\nshellgame submit");

// Level 1.8: an absolute path, and only arriving at the target counts (not `cd /`).
let absolute = level(8, "Skok absolutní cestou")
    .start(".")
    .file("absolute-target/PLACEHOLDER.answer", "")
    .require(at("absolute-target"))
    .require(cd_evidence("Použijte absolutní `cd` (začíná na /)."))
    .cd(policy()
        .absolute("Musíte použít absolutní cestu (začínající na /).")
        .bare_cd("Musíte zadat cestu.")
        .arrive("absolute-target"))
    .solution("cd \"$PWD/absolute-target\"\nshellgame submit");

// Level 1.7 (simplified): the player may not leave the maze until the goal is reached.
let maze = level(7, "Navigace v bludišti")
    .start("maze/entry")
    .dirs(["maze/entry/hall", "maze/entry/hall/goal"])
    .require(at("maze/entry/hall/goal"))
    .cd(policy()
        .within("maze", "Nemůžete opustit bludiště před jeho dokončením.")
        .bare_cd("Nemůžete opustit bludiště. Pokračujte v navigaci uvnitř maze/.")
        .arrive("maze/entry/hall/goal"))
    .solution("cd hall\ncd goal\nshellgame submit");
```

**Scope** decides whether the policy has an opinion about a move at all; a
move outside the scope is allowed silently. Scope clauses all have to hold.

| Scope | The move |
| --- | --- |
| `.from_dir(p)` | starts in `p` |
| `.target(t)` | has the target `t`, such as `"-"` (the first non-option argument of `cd`) |

**Rules** judge a move in scope. They run in the order added, and the first
failing one blocks with its message.

| Rule | Requires |
| --- | --- |
| `.exact(cmd, msg)` | the target equals `cmd` (whitespace and trailing slashes ignored) |
| `.absolute(msg)` | the target starts with `/` |
| `.source(p, msg)` | the move starts in `p` |
| `.after(marker, msg)` | the marker exists: an earlier drill, such as `Marker::Pwd` |
| `.within(p, msg)` | the destination stays inside `p` (canonical, component-wise) |
| `.rule(fn)` | your own check (section 12) |

`.bare_cd(msg)` replaces the message of a built-in rule when the player typed a
bare `cd`, which goes home and so is almost never what the level wants.

### The contract

The engine, not the author, performs it, so no level can forget it:

1. **Before the move.** Out of scope: allowed. *Lifted*: allowed. A move is
   lifted when the level's `cd` marker exists (the drill is done) or when it
   goes to the level's start directory (the recovery move). Otherwise the first
   failing rule blocks, with a red panel and the tip to run `shellgame reset`.
2. **After a successful move.** The shell reports the move that actually
   happened. The `cd` marker is written when the move was in scope, every rule
   passes (judged with the real destination) and, if the policy has
   `.arrive(p)`, the player landed in `p`. Lifting waives *blocking* only; it
   never grants or denies evidence, so `cd -` back to the start of 2.2 still
   counts.
3. **`.on_move(fn)`** runs after every successful move, lifted or not, in scope
   or not.

A failed `cd` is never reported, so it earns nothing. Without this contract a
strict level could leave a player in a directory where every permitted move
leads further away. A blocked player always has `shellgame reset`, and the
catalog test checks that each policy blocks something, always allows the move
to the start directory and blocks nothing once the drill is done.

`cd_evidence(msg)` in the requirements refers to the level's own marker without
naming it; `msg` is what the player sees when they submit without the drill.
Policy options that make no sense together are lints (`.bare_cd` with only
custom rules, `.manual_evidence` without `.on_move`, `.arrive` together with
`.manual_evidence`).

For stateful grading use `.on_move(fn)` with `.manual_evidence()`: you decide
when the drill is done by creating `Marker::Cd` yourself. Level 1.9, which
tracks a walk from `/` down to `$HOME`, is the one case:
[`walk.rs`](../src/levels/s01_navigace/walk.rs).

---

## 7. Output redirection: `fd` evidence

Level 10.4 grades whether a command's output was redirected to `/dev/null`
(`&> /dev/null`). The game cannot see that from the outside, so the level's own
script asks:

```rust
use shellgame::engine::prelude::*;

// Level 10.4: buggy.sh ends by running the hook, which looks at its own stdout and stderr.
let null = level(4, "Černá díra")
    .start(".")
    .file_mode(
        "buggy.sh",
        "#!/bin/bash\necho out\necho err >&2\n\"$SHELLGAME_FD_HOOK\"\n",
        0o755,
    )
    .answer(exact("/dev/null"))
    .require(fd_evidence("Spusťte `./buggy.sh` a přesměrujte stdout i stderr do `/dev/null`."))
    .on_fd(|fds| fds.stdout_is_null() && fds.stderr_is_null())
    .solution("./buggy.sh &> /dev/null\nshellgame submit '/dev/null'");
```

`$SHELLGAME_FD_HOOK` is set by the game shell. It is a small launcher that
`exec`s `shellgame fd-hook`, so the hook inherits the descriptors the script was
started with, which are the ones the player's redirection set up. `.on_fd(..)`
decides whether they count; `Fds` offers `stdout_is_null()`, `stderr_is_null()`,
`both_null()`, and the raw device and inode numbers. `fd_evidence(msg)` reads the
marker. The script must run the hook itself, directly, and not through the
`shellgame` shell function, which captures stderr by design.

---

## 8. Solutions: proof that a level can be finished

Every regular level has a solution, and the lint insists. It is a shell script:
what a student would type, one command per line, from the level's start
directory (the workspace root if it has none), ending with
`shellgame submit [answer]`.

```rust
use shellgame::engine::prelude::*;

// Both shells.
let a = level(1, "x").answer(exact("finish")).solution("cd ../finish\nshellgame submit");

// Bash-only syntax: replayed in bash, skipped in fish.
let b = level(2, "x").answer(exact("y")).solution_bash("[[ -d y ]] && shellgame submit y");

// Depends on the machine: generated from the situation (here: the player's $HOME).
let c = level(9, "Cesta z kořene domů")
    .start_at_workspace()
    .require(at_home())
    .solution_with(|ctx| {
        let mut lines = vec!["cd /".to_owned()];
        for part in ctx.home.iter().skip(1) {
            lines.push(format!("cd '{}'", part.to_string_lossy()));
        }
        lines.push("shellgame submit".to_owned());
        lines.join("\n")
    });
```

Inside `src/levels/`, `script([..])` joins lines, which reads better than `\n`:

```rust,ignore
use crate::levels::script;

.solution(script(["cd ../location-B", "cd -", "shellgame submit"]))
```

`tests/solutions.rs` replays every solution through the real binary and the
real shell integration: in bash (under `umask 022` and `077`) and in fish, each
in a sandbox whose workspace path contains a space. A trial (`solutions::2.8::bash-022`)
jumps to the level, checks that an empty `shellgame submit` does not complete
it, runs the solution and checks that it does. Then it does all of that again
over what the first run left behind, which is what `shellgame reset` has to
survive. One check therefore proves that a level is winnable, that `reset`
works, that `cd` policies accept their own drill, that `pwd` and `fd` evidence
arrive, and that the answer can be found with the taught tools.

Writing a solution:

- **Use the commands the assignment teaches.** Derive an answer with `grep`,
  `wc`, `file`, `cut`, not by typing it in. A solution that just knows the
  answer proves nothing about discoverability.
- **Valid in bash and fish.** Use `"$(...)"` and `"$PWD"`. Avoid `[[ ]]`,
  `$'..'`, `local`, `export X=1`. `&>` is fine. (`$(...)` needs fish 3.4 or
  later.) If a lesson teaches bash-only syntax, give fish players
  `bash -c '...'` in the assignment and use it in the solution, as section 7
  does; use `.solution_bash` only when that is not possible.
- **Non-interactive.** No `less`, no prompts, no `sleep`.
- **Independent of umask.** Use explicit `chmod` targets (`u+x`, `a-w`).
- **Portable tools.** The game also targets macOS, which has BSD tools and
  bash 3.2. Prefer options that GNU and BSD share; a GNU-only one (`ls --help`)
  needs a reworded level or a `cfg!(target_os = "macos")` variant.
- **One command per line**, so a failure points at a command.

---

## 9. Intros and paging

`intro(title, text)` creates the section intro (`N.0`): a page the player
reads, not a task. Pages are separated by a line containing only `---`:

```rust
use shellgame::engine::prelude::*;
use shellgame::ui::split_pages;

let text = "### Sekce 3\n\nPrvní strana.\n\n---\n\nDruhá strana.\n";
let section = Section::new(3, "level-3").intro(intro("Skryté soubory", text).start_at_workspace());
assert_eq!(split_pages(text), ["### Sekce 3\n\nPrvní strana.", "Druhá strana."]);
```

- Each page waits for Enter (`Stiskněte Enter pro pokračování...`); after the
  last one the game moves on to the first level of the section. Nothing is
  asked when stdin is not a terminal. Ctrl-C at the prompt is just a key: it
  stops the paging without advancing, and `shellgame` continues.
- The intro comes with the standard hint (`Přečtěte si úvod a pokračujte
  stisknutím Enter.`). **Intros continue with Enter; never tell the player to
  run `shellgame submit` there.**
- An intro cannot have an answer, requirements or a `cd` policy (lint).

---

## 10. Adding a section

1. Pick the next free number `N`. Sections are numbered 0, 1, 2, ... without gaps.
2. Create `src/levels/sNN_<topic>/` (the topic is a Czech word without
   diacritics) with a `mod.rs`:

   ```rust
   use shellgame::engine::prelude::*;

   fn section() -> Section {
       Section::new(12, "level-12")
           .fixture(Fixture::new().dir("shared"))
           .intro(intro("Sekce 12: Nové téma", "Úvod.\n\n---\n\nDruhá strana."))
           .levels([first()])
   }

   fn first() -> Level {
       level(1, "První").answer(exact("a")).solution("shellgame submit a")
   }

   assert!(Catalog::new(vec![section()]).validate().is_empty());
   ```

   In the real module `section()` is `pub fn`, the texts come from
   `include_str!`, and the file begins with `use crate::engine::prelude::*;`.
3. Put each assignment in `NN-slug.md` (Czech words without diacritics) and the
   intro in `00-uvod.md`.
4. Register the module in `src/levels/mod.rs`, then run `cargo test`:

   ```rust,ignore
   mod s12_nove_tema;

   pub fn catalog() -> Catalog {
       Catalog::new(vec![
           // ...
           s11_vyhledavani::section(),
           s12_nove_tema::section(),
       ])
   }
   ```

Ids are saved, so never renumber an existing section or level. Inserting a
section into the middle of the existing numbers needs a state migration; see
the 2.1 migration in [`src/state.rs`](../src/state.rs).

---

## 11. What the lints protect

`Catalog::validate()` returns a list of problems; it must be empty. It runs in
the unit tests (`src/levels/mod.rs`) and, in debug builds, at startup, so a
malformed level fails on the first `cargo run`.

| Lint | Protects |
| --- | --- |
| section numbers and level ids are unique, and a level's id carries its section's number | saves, which are keyed by id |
| an intro is exactly the `N.0` level, with no answer, requirements or policy | the Enter-only intro flow |
| a regular level has an answer or a requirement | a level that anything completes |
| a regular level has a solution | every level is proven winnable by the replay |
| declared paths are plain relative paths and do not restate the section root | one path vocabulary, containment |
| fixtures are valid: modes in range, nothing targets the root | a fixture that can always be applied |
| `cd_evidence` has a `cd` policy, `fd_evidence` has `.on_fd(..)` | a requirement nothing can satisfy |
| hints are not empty | a blank hint that still counts as revealed |
| the answer (4 characters or more) never appears in an error, mistake, invalid, required or format message | a wrong guess handing over the solution |
| `.secret()` answers are not in the hints or in a `shellgame submit <answer>` line of the text | discovery levels that stay discoverable |
| `.accepts` and `.rejects` examples hold | rules that mean what their author says |
| a custom `at(..).message(..)` does not name its target | location answers given away |
| modifiers that do nothing (`.ignore_case()` on `int`, `.invalid()` on text), empty `one_of`, bad `parts` | author mistakes |
| contradictory `perm_bits`, modes out of range, inconsistent `cd` policy options | author mistakes |

The leak lint in action:

```rust
use shellgame::engine::prelude::*;

let leaky = level(1, "Heslo")
    .answer(exact("navigator").error("Heslo není navigator, zkuste to znovu."))
    .solution("shellgame submit navigator");
let errors = Catalog::new(vec![Section::new(2, "level-2").levels([leaky])]).validate();
assert!(!errors.is_empty()); // the answer appears in the error message
```

The tests in `src/levels/mod.rs` add checks over the whole real catalog:

| Test | Protects |
| --- | --- |
| sections are numbered 0, 1, 2, ... and open with an intro; levels are listed in ascending order | the layout the session relies on |
| every level has text and every task has hints | an empty panel |
| every level can be prepared, in course order, in one workspace, and its start directory exists | fixtures of one level breaking another |
| at the start of a level, a bare submit and (if there are requirements) the right answer both fail | requirements that are not unconditional |
| every `cd` policy blocks something, allows the move to the start, and blocks nothing after the drill | no soft lock |
| every solution ends with a `submit` line | solutions that prove something |

If one of these fails, the check is usually right. Fix the level.

---

## 12. Escape hatches

Plain data covers most levels. When it does not, ordinary Rust functions plug
in. Each one is a line that no lint can reason about, so use the least
powerful tool that works:

| Hatch | Use it when | Mind |
| --- | --- | --- |
| `check(fn(&Ctx) -> Check)` | a condition no requirement expresses | return `Check::Pass` or `Check::fail(msg)`; resolve paths with `ctx.path(..)`, never by hand; do not leak the answer |
| `.feedback(fn(&str) -> Option<String>)` | wrong answers fall into classes you cannot list | runs after the literal mistakes; return `None` to fall through |
| `.rule(fn(&CdMove) -> Result<(), String>)` | a movement rule that needs state or the shape of the target | the engine still lifts, blocks and records evidence around it |
| `.on_move(fn(&CdMove, &Markers))` | bookkeeping after each move | pair with `.manual_evidence()` only if you write `Marker::Cd` yourself |
| `exact_with(fn(&Ctx) -> String)`, `.solution_with(..)` | the answer or the solution depends on the machine | `.accepts` examples are skipped for such rules |
| a function returning a `Fixture` | structure that is impractical to enumerate (the maze) | applied with the same repair rules |

```rust
use shellgame::engine::prelude::*;

// A requirement nothing in the vocabulary expresses.
fn sorted(ctx: &Ctx) -> Check {
    let Ok(path) = ctx.path("sorted.txt") else {
        return Check::fail(messages::PATH_ESCAPES_WORKSPACE);
    };
    match std::fs::read_to_string(path) {
        Ok(text) if text.lines().is_sorted() => Check::Pass,
        Ok(_) => Check::fail("Řádky v sorted.txt nejsou seřazené."),
        Err(_) => Check::fail(messages::with_path(messages::FILE_NOT_EXISTS, "sorted.txt")),
    }
}

let sort = level(8, "Řazení")
    .start(".")
    .file("data.txt", "b\na\n")
    .require(check(sorted))
    .solution("sort data.txt > sorted.txt\nshellgame submit");

// A stateful movement drill: you decide when it is done (a real one is walk.rs).
fn no_jumps(mv: &CdMove) -> Result<(), String> {
    if mv.is_bare() {
        Err("Skoky nejsou povoleny.".to_owned())
    } else {
        Ok(())
    }
}

fn arrived_home(mv: &CdMove, markers: &Markers) {
    if mv.dest == Some(mv.ctx.home) {
        let _ = markers.create(&Marker::Cd);
    }
}

let home = level(9, "Cesta domů")
    .start_at_workspace()
    .require(at_home())
    .require(cd_evidence("Dojděte domů příkazem `cd`."))
    .cd(policy().rule(no_jumps).on_move(arrived_home).manual_evidence())
    .solution("cd ~\nshellgame submit");
```

---

## 13. Trying a level

```text
cargo test                                    # lints, engine, catalog, every solution
cargo test --test solutions -- 2.8            # replay one level's solution in real bash and fish
cargo fmt && cargo clippy --all-targets -- -D warnings
cargo run -- --devmode                        # play from source
shellgame dev jump 2.8                        # inside the game: jump to a level (shortcut: j 2.8)
```

`--devmode` adds the one-letter shortcuts `j` (jump), `n` (next), `p` (previous),
`r` (reload), `s` (start), `e` (exit) and `g` (`shellgame`). `dev jump` also clears
the level's progress, so you can replay it from scratch. Level code is compiled
in: after an edit, leave the game shell and run `cargo run` again; the saved
game keeps your place.

Playtest with a throwaway `HOME`, `XDG_CONFIG_HOME` and `SHELLGAME_WORKSPACE`
(see the README) and never touch a real save. Check both shells
(`--shell bash`, `--shell fish`).
