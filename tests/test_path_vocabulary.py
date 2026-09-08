"""Invariants for the section-relative path vocabulary.

Every path a level declares - its fixtures, its completion requirements and its
start directory - is resolved against that level's section root. A level that
also spells the section root inside the path resolves to
``level-1/level-1/alpha`` and can never be completed, which is invisible until a
student reaches it.

These checks make the vocabulary structural instead of conventional.
"""

from __future__ import annotations

import ast
import re
from pathlib import Path

import pytest

from shellgame.levels.base import Level
from shellgame.levels.completion import (
    AtDirectory,
    DirectoryExists,
    FileExists,
    FileLineCount,
    PathMoved,
    PathsMatch,
    PermissionBits,
    PermissionMode,
    TextFileContent,
)
from shellgame.levels.fixture import WorkspaceFixture
from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import get_registry
from shellgame.paths import WorkspaceRoot

SECTIONS_DIR = Path(__file__).resolve().parent.parent / "src" / "shellgame" / "levels" / "sections"

SINGLE_PATH_REQUIREMENTS = (
    AtDirectory,
    DirectoryExists,
    FileExists,
    FileLineCount,
    PermissionBits,
    PermissionMode,
    TextFileContent,
)
PAIRED_PATH_REQUIREMENTS = (PathMoved, PathsMatch)


def _all_levels() -> list[Level]:
    initialize_levels()
    return get_registry().list_levels()


def _declared_paths(level: Level) -> list[tuple[str, str]]:
    """Every path this level declares, as ``(source, value)`` pairs."""
    declared: list[tuple[str, str]] = []

    if isinstance(level.start_directory, str):
        declared.append(("start_directory", level.start_directory))

    for name, fixture in (("section_fixture", level.section_fixture), ("fixture", level.fixture)):
        if isinstance(fixture, WorkspaceFixture):
            declared.extend((name, value) for value in fixture.directories)
            declared.extend((name, file.path) for file in fixture.files)
            declared.extend((name, value) for value in fixture.clean)

    if level.completion is not None:
        requirements = [*level.completion.requirements]
        if level.completion.allow_empty_when is not None:
            requirements.append(level.completion.allow_empty_when)
        for requirement in requirements:
            name = type(requirement).__name__
            if isinstance(requirement, SINGLE_PATH_REQUIREMENTS):
                declared.append((name, requirement.path))
            elif isinstance(requirement, PAIRED_PATH_REQUIREMENTS):
                declared.append((name, requirement.source))
                declared.append((name, requirement.destination))

    return declared


@pytest.mark.parametrize("level", _all_levels(), ids=lambda level: level.id)
def test_level_paths_do_not_restate_their_section_root(level: Level) -> None:
    """``AtDirectory("level-1/alpha")`` inside section 1 resolves twice over."""
    root = level.section_root
    if not root:
        return

    offenders = [
        (source, value) for source, value in _declared_paths(level) if value == root or value.startswith(f"{root}/")
    ]

    assert not offenders, (
        f"Level {level.id} restates its own section root {root!r}: {offenders}. "
        "Level paths are relative to the section root, so drop the prefix."
    )


@pytest.mark.parametrize("level", _all_levels(), ids=lambda level: level.id)
def test_level_paths_stay_inside_the_section(level: Level, tmp_path: Path) -> None:
    """Declared paths must not climb out of the section with `..` or an anchor."""
    for source, value in _declared_paths(level):
        candidate = Path(value)
        assert not candidate.is_absolute(), f"Level {level.id}: {source} path {value!r} is absolute"
        assert ".." not in candidate.parts, f"Level {level.id}: {source} path {value!r} escapes the section"


@pytest.mark.parametrize("level", _all_levels(), ids=lambda level: level.id)
def test_start_directory_resolves_in_the_declared_vocabulary(level: Level, tmp_path: Path) -> None:
    """`""` is the section root; only the sentinel reaches the workspace root."""
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    resolved = level.get_start_directory(workspace)

    if level.start_directory is None:
        assert resolved is None
        return

    if isinstance(level.start_directory, WorkspaceRoot):
        assert resolved == workspace
    elif level.start_directory == "":
        assert resolved == level.section_path(workspace)
    else:
        assert resolved == level.section_path(workspace) / level.start_directory


def test_workspace_root_starts_are_explicitly_acknowledged() -> None:
    """Starting above your own section is deliberate, so it is listed here.

    Both directions matter. A level that quietly gains the sentinel moves the
    player out of its own fixtures; a level that quietly loses it starts one
    directory too deep. Neither shows up as a test failure anywhere else,
    because both destinations exist and both are inside the workspace.
    """
    expected = {
        "1.0",  # section intro: the player has not entered section 1 yet
        "1.9",  # walking home from the workspace root
        "1.10",  # absolute paths, taught from the workspace root
        "3.6",  # section checkpoint, reviewed from the workspace root
    }
    actual = {level.id for level in _all_levels() if isinstance(level.start_directory, WorkspaceRoot)}

    assert actual == expected, (
        "Levels starting at the workspace root instead of their section root changed. "
        f"Added: {sorted(actual - expected)}; removed: {sorted(expected - actual)}. "
        "Confirm the new placement is intended, then update this list."
    )


def test_section_modules_do_not_hardcode_their_own_root() -> None:
    """Code that builds paths by hand must go through `section_path()`."""
    offenders: list[str] = []
    for module_path in sorted(SECTIONS_DIR.glob("section*.py")):
        number = int(re.search(r"section(\d+)\.py$", module_path.name).group(1))
        root = "" if number == 0 else f"level-{number}"
        if not root:
            continue

        tree = ast.parse(module_path.read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            if not isinstance(node, ast.BinOp) or not isinstance(node.op, ast.Div):
                continue
            if isinstance(node.right, ast.Constant) and node.right.value == root:
                offenders.append(f"{module_path.name}:{node.lineno}")

    assert not offenders, (
        "These lines join the section root onto a workspace path by hand; "
        f"use `self.section_path(workspace)` instead: {offenders}"
    )
