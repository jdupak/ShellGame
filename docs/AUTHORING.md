# Authoring ShellGame levels

This guide covers adding a level or a section. It assumes you have read
`ARCHITECTURE.md` for the wider picture.

The rule that drives everything below: **a level declares what "solved" looks
like, and the engine decides how to check it.** Hand-written logic is allowed,
but every line of it is a line no invariant test can reason about.

---

## 1. Anatomy of a level

```python
@section.level(3)
class ExtensionLevel(Level):
    title = "Vstup a hlášení"
    instructions = """
        ### Cíl
        Zjistěte název souboru bez přípony.
    """
    hints = ("Zkuste `ls`.", "Podívejte se dovnitř `alpha`.")
    start_directory = ""
    fixture = WorkspaceFixture(files=(FileFixture("alpha/inside.txt"),))
    completion = Completion(
        answer=ExactAnswer("inside"),
        requirements=(AtDirectory("alpha"),),
    )
    solution = Solution(steps=(Chdir("alpha"),), answer="inside")
    success_message = "Přesně tak!"
```

`@section.level(3)` assigns the ID. The number is the **stable suffix**, not a
position: reordering the classes in the file does not renumber anything.
IDs live in `state.json`, so changing one invalidates a player's save.

---

## 2. Paths: one vocabulary

**Every path a level declares is relative to its section root.** The section
root is declared once, by the section:

```python
section = Section(1, root="level-1")
```

That single vocabulary covers `fixture`, `completion` requirements,
`start_directory` and `Solution` steps. A level never repeats its own
`level-N/` prefix:

```python
start_directory = "alpha"                      # -> <workspace>/level-1/alpha
AtDirectory("gamma/deep")                      # -> <workspace>/level-1/gamma/deep
FileFixture("absolute-target/PLACEHOLDER")     # -> <workspace>/level-1/absolute-target/…
```

`tests/test_path_vocabulary.py` fails the build if a level restates its section
root or builds a path by hand.

Absolute paths, `..` segments and symlinked parents are rejected at
declaration or resolution time by `paths.resolve_within()`, which is the single
implementation shared by fixtures, requirements, start directories and
solutions.

Need the workspace root itself, above your own section? Say so explicitly:

```python
from shellgame.paths import WORKSPACE_ROOT

start_directory = WORKSPACE_ROOT
```

Do **not** use `""` for that. `""` means the section root.

---

## 3. Building the workspace: `WorkspaceFixture`

```python
from shellgame.levels.fixture import DirectoryFixture

fixture = WorkspaceFixture(
    directories=("logs",),
    directory_fixtures=(DirectoryFixture("private", mode=0o750),),
    files=(
        FileFixture("logs/app.log", content="log1"),
        FileFixture("private/config", content="secret"),
    ),
    clean=("logs/archive",),
)
```

`clean` runs before the fixture is applied, so a level is restored to a known
state no matter what the previous attempt left behind. Fixtures are idempotent
and will overwrite a file even if the player removed write permission - section
7 teaches `chmod`, and `shellgame reset` has to work afterwards.

Fixtures also repair file/directory mixups at declared paths: a directory where
a file belongs is replaced, and a file blocking a declared directory (including
a parent directory) is removed before rebuilding it. `overwrite=False`
preserves existing **regular files**, not an incorrect type. Repair never
replaces the fixture root or follows a symlink; file fixtures cannot name `""`
or `"."`. Repair recursively restores traverse and write permissions so cleanup
succeeds even after `chmod 000` or read-only directory mistakes.

Use `directory_fixtures` with `DirectoryFixture(path, mode)` when a directory
must start with an exact mode. These declarations safely replace files or
symlinks at every declared path component without following the symlink; the
symlink target remains untouched. Like file fixtures, they cannot target the
fixture root itself. `WorkspaceFixture` creates all directories
and child files first, then applies exact directory modes, because creating a
child may temporarily restore its parent's traverse/write bits. Both fixture
and mode paths use the same section-root-relative vocabulary. Modes range from
`0o0000` through `0o7777`, including special permission bits.

For discovery tasks, verify fixtures using the command the student is taught.
Magic bytes alone may not produce an image recognized by `file`; distinguish
plain text from scripts whose descriptions also contain `ASCII text`.
Counting fixtures should reset away extra entries that would change the answer.

Put shared scaffolding on the section (`Section(..., fixture=...)`) and level
scaffolding on the level. Both are applied by `Level.prepare()`, in that order.

Write a custom `setup()` only when the structure cannot be enumerated - the
1.7 maze and the 200-line log in 5.4 are the two justified cases.

---

## 4. Deciding when a level is solved: `Completion`

```python
completion = Completion(
    answer=ExactAnswer("delta"),
    requirements=(AtDirectory("delta"), Evidence("pwd_used")),
    allow_empty=False,
    allow_empty_when=AtDirectory("delta"),
)
```

- `answer` grades what the player typed. One rule, optionally a `TupleAnswer`
  of several.
- `requirements` are **unconditional**. They are checked whether or not the
  answer was correct, and a correct answer never bypasses them. Filesystem
  state, working directory, permissions and evidence markers all belong here.
- `allow_empty` permits a bare `shellgame submit`.
- `allow_empty_when` permits it only in a given situation.

### Answer rules

`ExactAnswer`, `IntegerAnswer`, `IntegerRangeAnswer`, `ChoiceAnswer`,
`SuffixAnswer`, `OrderedListAnswer`, `TupleAnswer`.

They share one feedback contract:

- `error_message=None` → the terse `Messages.INCORRECT`.
- `error_message="…"` → your text.
- `mistakes={...}` → a targeted reply for a specific wrong answer, which is
  where teaching actually happens.

A rejection must never contain the answer, including targeted `mistakes`
feedback and failed navigation requirements. `test_level_invariants.py` and
`test_hint_quality.py` enforce this; a player who guesses once should not be
handed the solution. Discovery hints should explain how to derive the answer,
not state it. Command examples for explicitly assigned filesystem tasks are
still useful.

`required_message` is what an author sees when the player submits nothing. Set
it whenever a level needs an argument, and include the example command - never
let a `None` answer reach the player as text.

### Requirements

`AtDirectory`, `AtHome`, `Evidence`, `FileExists`, `DirectoryExists`,
`TextFileContent`, `FileLineCount`, `PathsMatch`, `PathMoved`,
`PermissionBits`, `PermissionMode`, `DirectoryPermissionMode`.

`FileExists` and `DirectoryExists` with `should_exist=False` require that no entry
(file, directory, or symlink) remains at the path. `PermissionMode` checks an
exact regular-file mode; `DirectoryPermissionMode` is its directory-only
counterpart. Both validate paths relative to the section root and compare all
permission and special bits (`0o7777`). A directory-mode requirement is a
filesystem requirement, so the level must also declare a `Solution`; use the
existing `Chmod` step when appropriate.

Navigation levels must use `AtDirectory` with a section-root-relative path.
Checking `Path.cwd().name` would accept a same-named directory elsewhere on the
machine. Its default failure points back to the task and `pwd`, without naming
the expected directory. Custom feedback must not reveal a discovery target.

Navigation commands in instructions and hints are relative to the player's
declared start directory, not the workspace root. Do not tell a player already
in `level-1` to run `cd level-1/...`, or hard-code a username in an absolute
path example. For a `cd -` exercise, spell out the intermediate moves so the
previous directory is predictable.

When a command prints a path as the answer, either state which directory that
path is relative to or accept the equivalent output produced after reasonable
navigation within the level.

### Shell-specific lessons

Mark Bash-specific syntax explicitly in both the section intro and the affected
level's instructions. Do not imply that Fish implements Bash's `?` or `[...]`
globs. For filesystem-only exercises, provide a quoted `bash -c 'command'`
example so a Fish player can run that command in Bash and submit from the
existing wrapped game shell. Do not tell players to enter an unwrapped
interactive Bash: it would lose the game's shell hooks.

Only require Bash where the syntax needs it; ordinary `*` globs and quoted
patterns interpreted by external commands such as `find -name` remain portable.
Replay displayed commands from the declared start directory, fresh and after
reset, and require a zero exit status as well as successful completion.
Use explicit `chmod` targets (`u+x`, `a-w`) when an exercise requires a specific
set of users; omitting the target makes symbolic modes depend on `umask`.

---

## 5. Restricting movement: `cd_policy`

Some navigation levels grade *which command* was used, not just where the
player ended up. Declare that; do not write a hook.

```python
cd_policy = CdPolicy(
    scope=(FromDirectory("gamma/deep/a/b/c"),),
    rules=(
        RequireEvidence(MarkerManager.PWD_USED, "Nejdřív použijte příkaz 'pwd'."),
        RequireExactCommand("../../../..", "Použijte jeden příkaz 'cd ../../../..'."),
    ),
)
completion = Completion(
    requirements=(
        AtDirectory("gamma"),
        CdEvidence("Nejdřív použijte `pwd`, přejděte do `c` a vraťte se jedním příkazem."),
    ),
)
```

- **scope** decides whether the policy applies at all. A move outside the scope
  passes silently.
- **rules** are checked in order; the first failure rejects the move with its
  message.
- The evidence marker is derived from the level ID (`cd-1.12`), so it is unique
  by construction. `CdEvidence(message)` in the completion refers to it without
  naming it, and `message` is what the player sees if they submit without having
  performed the drill.

Available scopes: `FromDirectory`, `WithTarget`.
Available rules: `RequireExactCommand`, `RequireAbsolutePath`,
`RequireSourceDirectory`, `RequireEvidence`.

### Why not a hook?

The engine performs the anti-soft-lock contract for you: it consults
`cd_enforcement_lifted()` before every rejection, so a player is always allowed
back to the level start directory, and all restrictions end once the marker is
written. Rejections always route through `block_cd()`, which appends the
`shellgame reset` recovery tip.

A strict level that forgets any of this can leave a player in a directory where
every permitted move leads further away, with no in-game way back. That is why
`tests/test_cd_hooks_no_softlock.py` fails the build when a level can reject a
move without soft-lock coverage.

Write `_handle_cd` only for genuinely stateful grading - level 1.9, which tracks
a step-by-step walk from `/` to `$HOME`, is the one remaining case.

---

## 6. Proving the level can be finished: `Solution`

Every other test proves a level rejects a wrong answer. A `Solution` proves it
accepts a right one.

```python
solution = Solution(steps=(RunShell("cp dulezite.txt dulezite.bak"),), answer="dulezite.bak")
```

`tests/test_solutions.py` prepares a throwaway workspace, replays the steps and
asserts `validate()` succeeds - then does it again after `reset()`, because a
level also has to survive a second attempt.

Steps: `Chdir`, `GoHome`, `MakeDirectory`, `WriteFile`, `AppendFile`,
`CopyPath`, `MovePath`, `RemovePath`, `Chmod`, `RecordEvidence`,
`RecordFdEvidence`, `PerformCd`, `WalkHome`, `RunShell`.

Guidance:

- Prefer `RunShell` with the command the instructions actually teach. It proves
  the taught command produces the graded result.
- Use `PerformCd` for `cd_policy` levels. It drives the real hook, so a policy
  that rejects its own intended command fails the build. Faking the marker with
  `RecordEvidence` would hide exactly that bug.
- `RecordEvidence` is only for markers Python cannot produce, such as
  `pwd_used`, which the shell wrapper writes.

A solution is optional in general but **required** for levels that can silently
become unwinnable: custom `validate()`, custom `setup()`, a `cd` policy or hook,
an evidence marker, or any filesystem requirement. The gate lives in
`test_risky_levels_declare_a_solution`.

---

## 7. Adding a section

1. Create `src/shellgame/levels/sections/sectionN.py`:

   ```python
   section = Section(N, root="level-N")
   ```

2. Add the intro as level `.0`:

   ```python
   @section.level(0)
   class SectionIntroLevel(Level):
       is_intro = True
       title = "Sekce N: …"
       instructions_file = "sectionN_intro.md"
       success_message = "Jdeme na to!"
   ```

   Intro markdown lives in `levels/content/`. A line containing only `---`
   starts a new page; the player advances with
   `Stiskněte Enter pro pokračování...`.
   Intros auto-continue on Enter — do not tell the player to run
   `shellgame submit` here. That command is only for real levels.

3. Add levels with `@section.level(1)`, `@section.level(2)`, …

There is no registration step and no `get_levels()` trailer: the loader reads
the module-level `section` object.

---

## 8. Custom `validate()`

Reach for it only when the outcome cannot be expressed declaratively - four
levels out of 91 qualify. When you do:

- Call `super().validate(...)` first and return early on failure, so the
  declarative rules still run.
- Return `(bool, str)`; the string is shown to the player.
- Never emit a message containing `None`.
- Declare a `Solution`; the gate requires one.

---

## 9. Quality gates

```
make test          # pytest with coverage
make lint          # ruff + mypy
make format-check  # ruff format
```

Structural invariants that will fail a careless level, and what each protects:

| Test | Protects |
|------|----------|
| `test_path_vocabulary.py` | no restated section roots, no hand-built paths |
| `test_marker_invariants.py` | every marker is owned and cleared by exactly one level |
| `test_cd_hooks_no_softlock.py` | no strict level can strand a player |
| `test_solutions.py` | risky levels stay winnable, before and after reset |
| `test_level_invariants.py` | start directories exist; defaults never reveal answers |
| `test_hint_quality.py` | assessment/discovery copy never hands over the answer |

If one of these fails, the invariant is usually right. Fix the level.
