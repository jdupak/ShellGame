"""Tests for Level 1.7 validation behavior."""

from pathlib import Path

import pytest

from shellgame.levels.sections.section1 import MazeLevel


def test_level1_7_submit_without_answer_depends_on_exact_cwd(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    level = MazeLevel()
    level.prepare(tmp_path)

    # Minimal state stub; Level1_7.validate doesn't use it.
    class _State:
        def __init__(self, workspace: Path) -> None:
            self.workspace = workspace
            self.current_level = "1.7"
            self.username = "testuser"

    state = _State(tmp_path)

    # Not in final -> fail
    monkeypatch.chdir(tmp_path)
    ok, _ = level.validate(None, state)
    assert ok is False

    # A same-named directory outside the maze must not pass.
    decoy = tmp_path / "final"
    decoy.mkdir()
    monkeypatch.chdir(decoy)
    ok2, _ = level.validate(None, state)
    assert ok2 is False

    # Only the actual maze destination passes.
    final_dir = tmp_path / "level-1" / level._SANCTUARY
    monkeypatch.chdir(final_dir)
    ok3, _ = level.validate(None, state)
    assert ok3 is True


def test_level1_7_answer_is_ignored_when_in_final(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    level = MazeLevel()
    level.prepare(tmp_path)

    class _State:
        def __init__(self, workspace: Path) -> None:
            self.workspace = workspace
            self.current_level = "1.7"
            self.username = "testuser"

    state = _State(tmp_path)

    final_dir = tmp_path / "level-1" / level._SANCTUARY
    monkeypatch.chdir(final_dir)

    ok, _ = level.validate("anything", state)
    assert ok is True


def test_level1_7_is_mandatory() -> None:
    level = MazeLevel()
    assert not level.optional
    assert not level.extension
    assert not level.is_bonus
