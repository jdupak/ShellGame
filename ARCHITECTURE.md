# ShellGame Architecture

ShellGame is a Click application that runs inside a controlled bash or fish
subshell. Python owns all game rules and persistence; shell templates only
forward events and apply a small, versioned command protocol.

## Runtime flow

1. `shellgame` creates application services and discovers level modules.
2. Outside a wrapper, `boot_if_needed()` launches bash or fish with the
   generated integration script.
3. Inside the wrapper, `GameSession` loads or creates state, prepares the
   current level, renders instructions, exports shell context, and teleports
   when the level declares a start directory.
4. User commands invoke a new Python process through the shell wrapper.
5. State-changing actions are persisted with one atomic write.

## Responsibilities

| Area | Module | Responsibility |
| --- | --- | --- |
| Click interface | `cli/commands.py` | Parse commands and delegate to `GameSession` |
| Orchestration | `core/session.py` | Initialization, display, hints, submit, reset, developer actions |
| Navigation | `core/navigation.py` | Resolve level start directories and emit teleport requests |
| Shell launch | `cli/subshell.py` | Render templates, detect shell, launch wrapper |
| Shell hooks | `cli/hooks.py` | Generate bash/fish `cd` dispatch from level hook metadata |
| Protocol client | `shell/client.py` | Emit versioned, base64-encoded operations to stderr |
| Persistence | `state/manager.py` | Validation, migrations, atomic state replacement |
| Progress | `core/progress.py` | Pure in-memory progress mutations |
| Level lifecycle | `levels/base.py` | Preparation, rendering metadata, custom extension points |
| Level registration | `levels/collector.py` | Section-local stable IDs, roots, and shared fixtures |
| Completion rules | `levels/completion.py` | Typed answers and unconditional gameplay requirements |
| Workspace fixtures | `levels/fixture.py` | Safe, repeatable setup and cleanup declarations |
| `cd` policies | `levels/cdpolicy.py` | Declarative movement rules with structural soft-lock protection |
| Reference solutions | `levels/solution.py` | Author walkthroughs proving a level is completable |
| Path containment | `paths.py` | The one implementation of "stay inside this root" |
| UI | `ui/display.py` | Rich panels, paging, notes, prompts |

## Shell integration

### Bash

```text
bash --rcfile <integration_script> -i
```

Do not add `--norc`; it disables `--rcfile`. Do not add `--noprofile`;
the inherited environment must remain available.

### Fish

```text
fish --init-command "function fish_greeting; end; source <integration_script>"
```

Both templates follow the same order: setup, function definitions, generated
hooks, wrapper function, autostart.

## Shell protocol

Python emits protocol lines to stderr:

```text
__SHELLGAME_EXEC__v1 <verb> [base64-argument...]
```

Only `cd`, `export`, `echo`, `pwd`, and `exit` are supported. Templates decode
arguments and dispatch these verbs explicitly; arbitrary shell evaluation is
not part of the protocol. `remove` remains a wrapper-level success case and
exits the subshell without an `exit` directive.

Commands that must inspect their inherited file descriptors invoke the
temporary executable referenced by `SHELLGAME_FD_HOOK`. They must not call the
shell wrapper function, because that function intentionally captures stderr.

## Level contracts

- A section declares its persistent number with `Section(number, ...)`.
- Every level declares its stable suffix locally with `@section.level(number)`;
  IDs never depend on declaration order.
- Section intro behavior is declared with `is_intro`.
- Start-directory enforcement is declared by the level.
- Regular levels declare a `Completion`; only genuinely custom interactions
  override `validate()`.
- `Completion.requirements` are unconditional. A correct or empty answer never
  bypasses filesystem, location, permission, or command-evidence checks.
- Navigation uses `AtDirectory`, which compares the exact resolved
  workspace-relative path rather than only the final directory name.
- Reusable answer shapes use typed rules such as `ExactAnswer`,
  `IntegerAnswer`, `TupleAnswer`, and `OrderedListAnswer`.
- **Every level-declared path is relative to that level's section root** -
  fixtures, requirements, `start_directory` and solution steps all speak one
  vocabulary, so a level can never disagree with itself about where it lives. A
  level that starts above its own section says so with the `WORKSPACE_ROOT`
  sentinel rather than by overloading `""`.
- All four resolve through `paths.resolve_within()`, which rejects absolute
  paths, `..` and symlinked parents. It is the single implementation; fixtures,
  requirements and section roots share it.
- Default answer feedback never contains the answer. A rule with
  `error_message=None` rejects with the terse `Messages.INCORRECT`; anything
  more specific is written deliberately by the author.
- Movement restrictions are declared as a `CdPolicy`, not hand-written. The
  engine consults `cd_enforcement_lifted()` before every rejection and routes
  rejections through `block_cd()`, so the anti-soft-lock guarantee is
  structural rather than a convention each author must remember.
- A policy's evidence marker is derived from the level ID, so marker names are
  unique by construction and need no global registry.
- Custom `_handle_cd` remains available for genuinely stateful grading (level
  1.9 walks from `/` to `$HOME` one segment at a time) and carries the same
  obligations by hand.
- Levels that can silently become unwinnable - custom `validate()` or `setup()`,
  a `cd` policy, an evidence marker, or a filesystem requirement - must declare
  a `Solution`. It is replayed against a throwaway workspace, before and after
  `reset()`.
- `Level.is_bonus` is `optional or extension`. Only bonus levels may be
  skipped with `shellgame skip`; core levels never can.
- `Level.prepare()` clears declarative evidence, applies the shared section
  fixture, applies the level fixture, and only then calls custom `setup()`.
- Use custom `setup()` only for content that is impractical to declare as a
  fixture.
- Gameplay validation belongs to the level class, not CLI or shell code.

## Session guarantees

- `GameSession.submit()` shows the level's own `success_message`; it never
  substitutes a generic one.
- Advancing performs exactly one atomic save, which also starts the next
  level's timer. The save happens before the next level's fixture is applied,
  so a failing fixture reports `show_level_setup_error` without losing
  progress.
- Finishing the last level sets `completed_at`. After that, `submit`, `skip`,
  `hint`, and showing the current level are inert and only re-display the
  completion summary.
- `reset()` re-exports the shell context and calls
  `ensure_user_in_reasonable_place()`, so it always rescues a player who
  wandered out of the level. Fixtures overwrite their files even when the player
  revoked write permission, so reset works after a `chmod` level.
- A deleted working directory is a normal, recoverable failure. `Path.cwd()` is
  never called unguarded; `paths.current_directory()` returns `None` and rules
  report it instead of raising.
- An unknown `current_level` resyncs to the nearest valid level and keeps
  `levels_complete`, so renumbering between versions cannot brick a save.

## Persistence

State files contain a version and are migrated during loading. Missing state
causes initialization; malformed or unsupported state raises an explicit error
and is never treated as missing. Each action mutates state in memory and then
uses `os.replace()` for one atomic commit.

Command evidence is not state: markers live in the workspace and are named after
the level that owns them. Renaming a marker therefore never invalidates a save;
at worst a player standing in the middle of that one level repeats a single
move.

## Quality gates

```text
make lint
make format-check
make test
```

CI runs these gates on the oldest and newest supported Python versions with
bash, fish, and shellcheck available.

Beyond per-feature tests, several suites assert structural invariants that a
new level cannot opt out of:

| Suite | Invariant |
| --- | --- |
| `test_path_vocabulary.py` | No level restates its section root or builds a path by hand |
| `test_marker_invariants.py` | Every evidence marker is owned and cleared by exactly one level |
| `test_cd_hooks_no_softlock.py` | No level that can reject a move lacks soft-lock coverage |
| `test_solutions.py` | Risky levels stay completable, before and after `reset()` |
| `test_level_invariants.py` | Start directories exist; default feedback never reveals an answer |
| `test_authoring_docs.py` | Every example in `docs/AUTHORING.md` runs against the real API |

See [docs/AUTHORING.md](docs/AUTHORING.md) for how to add a level or section.
