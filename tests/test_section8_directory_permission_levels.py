"""Behavior and pedagogy checks for interactive directory-permission levels."""

from __future__ import annotations

import os
import stat
import subprocess
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.sections.section8 import (
    DirectoryTraversePermissionLevel,
    DirectoryWritePermissionLevel,
    PermissionsChallengeLevel,
)
from shellgame.state.manager import GameState


def _state(workspace: Path, level_id: str) -> GameState:
    return GameState(username="tester", workspace=workspace, current_level=level_id, start_time=datetime.now())


def _mode(path: Path) -> int:
    return stat.S_IMODE(path.stat().st_mode)


def test_file_permission_challenge_is_no_longer_presented_as_section_finale() -> None:
    level = PermissionsChallengeLevel()

    assert "Souhrn Sekce 7" not in level.title
    assert "Dokončili jste Sekci 7" not in level.success_message


def test_directory_write_level_starts_locked_and_solution_explains_parent_directory(tmp_path: Path) -> None:
    level = DirectoryWritePermissionLevel()
    level.prepare(tmp_path)
    root = level.section_path(tmp_path)
    start = level.get_start_directory(tmp_path)

    assert start == root / "directory-write"
    assert _mode(start / "locked") == 0o550
    assert "Předpověď a pozorování" in level.instructions
    assert "cp source.txt locked/" in level.instructions
    assert "cp locked/report.txt recovered.txt" in level.instructions

    state = _state(tmp_path, level.id)
    assert not level.validate("adresář", state)[0]
    assert level.solution is not None
    level.solution.perform(level, state)
    assert level.validate("adresář", state)[0]
    assert _mode(start / "locked") == 0o750


@pytest.mark.skipif(os.geteuid() == 0, reason="root bypasses ordinary directory permission checks")
def test_directory_write_level_produces_the_failure_it_teaches(tmp_path: Path) -> None:
    level = DirectoryWritePermissionLevel()
    level.prepare(tmp_path)
    start = level.get_start_directory(tmp_path)
    assert start is not None

    result = subprocess.run(
        ["cp", "source.txt", "locked/"],
        cwd=start,
        capture_output=True,
        text=True,
        check=False,
        timeout=10,
    )

    assert result.returncode != 0
    assert not (start / "locked/source.txt").exists()


def test_directory_traverse_level_restores_only_owner_execute_and_exact_location(tmp_path: Path) -> None:
    level = DirectoryTraversePermissionLevel()
    level.prepare(tmp_path)
    root = level.section_path(tmp_path)
    parent = root / "directory-traverse/parent"

    assert level.get_start_directory(tmp_path) == parent / "child"
    assert _mode(parent) == 0o755
    assert "Předpověď a pozorování" in level.instructions
    assert "chmod a-x .." in level.instructions
    assert "chmod u+x .." in level.instructions

    state = _state(tmp_path, level.id)
    assert level.solution is not None
    level.solution.perform(level, state)
    assert level.validate("x", state)[0]
    assert Path.cwd() == parent
    assert _mode(parent) == 0o744


@pytest.mark.skipif(os.geteuid() == 0, reason="root bypasses ordinary directory permission checks")
def test_directory_traverse_level_produces_the_failure_it_teaches(tmp_path: Path) -> None:
    level = DirectoryTraversePermissionLevel()
    level.prepare(tmp_path)
    child = level.get_start_directory(tmp_path)
    assert child is not None
    parent = child.parent
    original = Path.cwd()
    try:
        os.chdir(child)
        parent.chmod(0o644)
        with pytest.raises(PermissionError):
            os.chdir("..")
    finally:
        parent.chmod(0o755)
        os.chdir(original)
