"""Prove that every level declaring a reference solution can actually be finished.

The rest of the suite checks that levels reject wrong answers. These tests check
the other direction, which is the one that silently breaks: a requirement path
the fixture never creates, an evidence marker no hook writes, or a `cd_policy`
that rejects the command its own instructions ask for all make a level
unwinnable without failing anything.

Each case prepares a throwaway workspace, replays the author's walkthrough, and
asserts `validate()` accepts it.
"""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import pytest

from shellgame.levels.base import Level
from shellgame.levels.completion import (
    AtDirectory,
    AtHome,
    DirectoryExists,
    FileExists,
    FileLineCount,
    PathMoved,
    PathsMatch,
    PermissionBits,
    PermissionMode,
    TextFileContent,
)
from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import get_registry


@dataclass
class _State:
    workspace: Path
    username: str = "solver"
    current_level: str = ""
    level_hints_used: dict[str, int] = field(default_factory=dict)
    levels_complete: dict[str, Any] = field(default_factory=dict)


def _levels() -> list[Level]:
    initialize_levels()
    return get_registry().list_levels()


def _solvable_levels() -> list[Level]:
    return [level for level in _levels() if level.solution is not None]


def _ids(levels: list[Level]) -> list[str]:
    return [level.id for level in levels]


@pytest.fixture
def restore_cwd() -> Any:
    origin = Path.cwd()
    yield
    os.chdir(origin)


SOLVABLE = _solvable_levels()


@pytest.mark.parametrize("level", SOLVABLE, ids=_ids(SOLVABLE))
def test_declared_solution_completes_the_level(level: Level, tmp_path: Path, restore_cwd: None) -> None:
    """The author's own walkthrough must satisfy the level's own rules."""
    del restore_cwd
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    state = _State(workspace=workspace, current_level=level.id)

    level.prepare(workspace)
    solution = level.solution
    assert solution is not None
    solution.perform(level, state)

    passed, message = level.validate(solution.answer, state)
    assert passed, f"{level.id} is not completable by its own solution: {message}"


@pytest.mark.parametrize("level", SOLVABLE, ids=_ids(SOLVABLE))
def test_solution_is_repeatable_after_reset(level: Level, tmp_path: Path, restore_cwd: None) -> None:
    """`shellgame reset` must hand the player a level that is still winnable.

    Reset re-applies the fixture over a workspace the player has already
    modified. A fixture that only creates missing files, or a marker that reset
    forgets to clear, would leave the second attempt unwinnable.
    """
    del restore_cwd
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    state = _State(workspace=workspace, current_level=level.id)
    solution = level.solution
    assert solution is not None

    level.prepare(workspace)
    solution.perform(level, state)

    os.chdir(tmp_path)
    level.reset(workspace)
    solution.perform(level, state)

    passed, message = level.validate(solution.answer, state)
    assert passed, f"{level.id} is not completable after reset: {message}"


def test_risky_levels_declare_a_solution() -> None:
    """The shapes that can silently become unwinnable must stay covered.

    A plain level whose fixture and requirements share one vocabulary is hard to
    get wrong. Custom code and evidence markers are not, so those levels are
    required to ship a walkthrough.
    """
    missing: list[str] = []
    for level in _levels():
        if level.is_intro or level.solution is not None:
            continue
        reasons = _risk_reasons(level)
        if reasons:
            missing.append(f"{level.id} ({', '.join(reasons)})")

    assert not missing, "These levels can silently become unwinnable and must declare a Solution:\n  " + "\n  ".join(
        missing
    )


_FILESYSTEM_RULES = (
    AtDirectory,
    AtHome,
    DirectoryExists,
    FileExists,
    FileLineCount,
    PathMoved,
    PathsMatch,
    PermissionBits,
    PermissionMode,
    TextFileContent,
)


def _risk_reasons(level: Level) -> list[str]:
    reasons = []
    if type(level).validate is not Level.validate:
        reasons.append("custom validate()")
    if type(level).setup is not Level.setup:
        reasons.append("custom setup()")
    if level.cd_policy is not None or "cd" in level.hooks:
        reasons.append("cd hook")
    if level.completion is not None and level.completion.evidence_markers:
        reasons.append("evidence marker")
    if level.completion is not None and any(
        isinstance(rule, _FILESYSTEM_RULES) for rule in level.completion.requirements
    ):
        reasons.append("filesystem requirement")
    return reasons
