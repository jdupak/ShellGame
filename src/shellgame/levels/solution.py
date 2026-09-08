"""Declarative reference solutions used to prove a level is completable.

Every other test in the suite proves a level rejects the *wrong* answer. Nothing
proved a level can be *finished*: a requirement pointing at a path the fixture
never creates, or an evidence marker no hook ever writes, makes a level
permanently unwinnable and ships silently.

A `Solution` is the author's own walkthrough, written in the same
section-relative vocabulary as fixtures and requirements. The harness prepares a
throwaway workspace, replays the steps, and asserts `validate()` then succeeds.

Solutions are opt-in - most levels are plain enough that their fixture and
requirements cannot disagree - but they are mandatory for the risky shapes
(custom `validate()`, custom `setup()`, `cd` policies, evidence markers), which
`tests/test_solutions.py` enforces.
"""

from __future__ import annotations

import os
import shutil
import subprocess
from collections.abc import Sequence
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Protocol, runtime_checkable

from shellgame.markers import MarkerManager
from shellgame.paths import WORKSPACE_ROOT, WorkspaceRoot, relative_path, resolve_within

if TYPE_CHECKING:
    from shellgame.levels.base import Level
    from shellgame.protocols import GameStateProtocol

_KIND = "Solution"


@dataclass(frozen=True, slots=True)
class SolutionContext:
    """Everything a step needs to act on the level under test."""

    level: Level
    state: GameStateProtocol
    root: Path

    def path(self, value: str) -> Path:
        """Resolve a step path with the same containment rules as requirements."""
        return resolve_within(self.root, value, kind=_KIND)

    def markers(self) -> MarkerManager:
        return MarkerManager(self.state.workspace)


@runtime_checkable
class SolutionStep(Protocol):
    def apply(self, context: SolutionContext) -> None: ...


@dataclass(frozen=True, slots=True)
class Chdir:
    """Move the player, exactly as a successful `cd` would."""

    path: str | WorkspaceRoot = ""

    def __post_init__(self) -> None:
        if isinstance(self.path, str):
            relative_path(self.path, kind=_KIND)

    def apply(self, context: SolutionContext) -> None:
        destination = context.state.workspace if isinstance(self.path, WorkspaceRoot) else context.path(self.path)
        destination.mkdir(parents=True, exist_ok=True)
        os.chdir(destination)


@dataclass(frozen=True, slots=True)
class GoHome:
    """Satisfy an `AtHome` requirement."""

    def apply(self, context: SolutionContext) -> None:
        del context
        os.chdir(Path.home())


@dataclass(frozen=True, slots=True)
class MakeDirectory:
    path: str

    def __post_init__(self) -> None:
        relative_path(self.path, kind=_KIND)

    def apply(self, context: SolutionContext) -> None:
        context.path(self.path).mkdir(parents=True, exist_ok=True)


@dataclass(frozen=True, slots=True)
class WriteFile:
    """Create a file, optionally with a given number of lines."""

    path: str
    content: str = ""
    lines: int | None = None

    def __post_init__(self) -> None:
        relative_path(self.path, kind=_KIND)
        if self.lines is not None and self.lines < 0:
            raise ValueError("WriteFile line count cannot be negative")

    def apply(self, context: SolutionContext) -> None:
        destination = context.path(self.path)
        destination.parent.mkdir(parents=True, exist_ok=True)
        if self.lines is None:
            destination.write_text(self.content, encoding="utf-8")
            return
        body = "".join(f"{self.content or 'radek'} {index + 1}\n" for index in range(self.lines))
        destination.write_text(body, encoding="utf-8")


@dataclass(frozen=True, slots=True)
class AppendFile:
    path: str
    content: str

    def __post_init__(self) -> None:
        relative_path(self.path, kind=_KIND)

    def apply(self, context: SolutionContext) -> None:
        destination = context.path(self.path)
        destination.parent.mkdir(parents=True, exist_ok=True)
        with destination.open("a", encoding="utf-8") as handle:
            handle.write(self.content)


@dataclass(frozen=True, slots=True)
class CopyPath:
    source: str
    destination: str

    def __post_init__(self) -> None:
        relative_path(self.source, kind=_KIND)
        relative_path(self.destination, kind=_KIND)

    def apply(self, context: SolutionContext) -> None:
        source = context.path(self.source)
        destination = context.path(self.destination)
        destination.parent.mkdir(parents=True, exist_ok=True)
        if source.is_dir():
            shutil.copytree(source, destination, dirs_exist_ok=True)
        else:
            shutil.copy2(source, destination)


@dataclass(frozen=True, slots=True)
class MovePath:
    source: str
    destination: str

    def __post_init__(self) -> None:
        relative_path(self.source, kind=_KIND)
        relative_path(self.destination, kind=_KIND)

    def apply(self, context: SolutionContext) -> None:
        source = context.path(self.source)
        destination = context.path(self.destination)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(str(source), str(destination))


@dataclass(frozen=True, slots=True)
class RemovePath:
    path: str

    def __post_init__(self) -> None:
        relative_path(self.path, kind=_KIND)

    def apply(self, context: SolutionContext) -> None:
        target = context.path(self.path)
        if target.is_dir() and not target.is_symlink():
            shutil.rmtree(target)
        else:
            target.unlink(missing_ok=True)


@dataclass(frozen=True, slots=True)
class Chmod:
    path: str
    mode: int

    def __post_init__(self) -> None:
        relative_path(self.path, kind=_KIND)
        if not 0 <= self.mode <= 0o7777:
            raise ValueError("Chmod mode must be between 0o0000 and 0o7777")

    def apply(self, context: SolutionContext) -> None:
        context.path(self.path).chmod(self.mode)


@dataclass(frozen=True, slots=True)
class RecordEvidence:
    """Write an evidence marker the shell templates would normally produce.

    Use this only for evidence Python cannot generate - `pwd_used` is written by
    the shell wrapper. Evidence produced by a `cd_policy` should be earned with
    `PerformCd`, which exercises the real hook.
    """

    marker: str

    def apply(self, context: SolutionContext) -> None:
        context.markers().create(self.marker)


@dataclass(frozen=True, slots=True)
class PerformCd:
    """Run the level's real `cd` hook, then move.

    This is the step that proves a strict navigation level is winnable: the hook
    must *accept* the intended command and record whatever evidence completion
    depends on. Faking the marker instead would hide a hook that rejects the very
    command its own instructions ask for - the exact failure that leaves a player
    unable to finish a level they solved correctly.

    `absolute` turns a section-relative path into the absolute one the player
    would type, for levels that grade *how* the path was written. `move_to`
    names the destination for shell-only targets such as `cd -`, which no
    filesystem path can express.
    """

    target: str = ""
    absolute: bool = False
    move_to: str | None = None

    def __post_init__(self) -> None:
        if self.absolute:
            relative_path(self.target, kind=_KIND)
        if self.move_to is not None:
            relative_path(self.move_to, kind=_KIND)

    def apply(self, context: SolutionContext) -> None:
        hook = context.level.hooks.get("cd")
        if hook is None:
            raise ValueError(f"{context.level.id}: PerformCd needs a cd hook or a cd_policy")
        origin = Path.cwd()
        target = str(context.path(self.target)) if self.absolute else self.target
        hook(target=target, pwd=str(origin), post_move=False, state=context.state)
        os.chdir(self._destination(context, origin, target))
        hook(target=target, pwd=str(Path.cwd()), post_move=True, state=context.state)

    def _destination(self, context: SolutionContext, origin: Path, target: str) -> Path:
        if self.move_to is not None:
            return context.path(self.move_to)
        destination = Path(target).expanduser()
        return destination if destination.is_absolute() else origin / destination


@dataclass(frozen=True, slots=True)
class WalkHome:
    """Walk from `/` to the home directory one segment at a time.

    The destination depends on the machine, so it cannot be spelled out in a
    declaration. The level grades the *sequence* of moves rather than the end
    state, so the walk has to go through the real hook.
    """

    def apply(self, context: SolutionContext) -> None:
        PerformCd("/").apply(context)
        for part in Path.home().resolve().parts[1:]:
            PerformCd(part).apply(context)


@dataclass(frozen=True, slots=True)
class RecordFdEvidence:
    """Report the redirection targets the shell wrapper would observe.

    Levels in section 9 grade where the streams were pointed, which is only
    visible to the wrapper. The step drives the level's own rule so a wrong
    marker name or condition still fails.
    """

    stdout_target: str = "/dev/null"
    stderr_target: str = "/dev/null"

    def apply(self, context: SolutionContext) -> None:
        context.level.record_fd_evidence(
            stdout_target=self.stdout_target,
            stderr_target=self.stderr_target,
            state=context.state,
        )


@dataclass(frozen=True, slots=True)
class RunShell:
    """Run a real command, for levels where *how* the result was reached matters.

    Sections 9-11 grade pipelines and redirection, so their solutions have to go
    through a shell rather than reproduce the end state directly.
    """

    command: str

    def apply(self, context: SolutionContext) -> None:
        del context
        subprocess.run(["bash", "-c", self.command], check=True, capture_output=True)  # noqa: S603, S607


@dataclass(frozen=True, slots=True)
class Solution:
    """A reference walkthrough that must make the level validate successfully."""

    steps: Sequence[SolutionStep] = field(default_factory=tuple)
    answer: str | None = None
    start_at: str | WorkspaceRoot | None = None

    def perform(self, level: Level, state: GameStateProtocol) -> None:
        """Replay the walkthrough. The caller owns preparing and restoring cwd."""
        context = SolutionContext(level=level, state=state, root=level.section_path(state.workspace))
        start = self._start_directory(level, state)
        if start is not None:
            start.mkdir(parents=True, exist_ok=True)
            os.chdir(start)
        for step in self.steps:
            step.apply(context)

    def _start_directory(self, level: Level, state: GameStateProtocol) -> Path | None:
        if self.start_at is None:
            return level.get_start_directory(state.workspace)
        if isinstance(self.start_at, WorkspaceRoot):
            return state.workspace
        return resolve_within(level.section_path(state.workspace), self.start_at, kind=_KIND)


__all__ = [
    "WORKSPACE_ROOT",
    "AppendFile",
    "Chdir",
    "Chmod",
    "CopyPath",
    "GoHome",
    "MakeDirectory",
    "MovePath",
    "PerformCd",
    "RecordEvidence",
    "RecordFdEvidence",
    "RemovePath",
    "RunShell",
    "Solution",
    "SolutionContext",
    "SolutionStep",
    "WalkHome",
    "WriteFile",
]
