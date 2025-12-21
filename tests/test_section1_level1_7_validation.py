"""Tests for Level 1.7 validation behavior."""

from pathlib import Path

from typing import Any

from shellgame.levels.sections.section1 import Level1_7


def test_level1_7_submit_without_answer_depends_on_cwd(tmp_path: Path, monkeypatch: Any) -> None:
    level = Level1_7()

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

    # In final -> pass
    final_dir = tmp_path / "final"
    final_dir.mkdir()
    monkeypatch.chdir(final_dir)
    ok2, _ = level.validate(None, state)
    assert ok2 is True


def test_level1_7_answer_is_ignored_when_in_final(tmp_path: Path, monkeypatch: Any) -> None:
    level = Level1_7()

    class _State:
        def __init__(self, workspace: Path) -> None:
            self.workspace = workspace
            self.current_level = "1.7"
            self.username = "testuser"

    state = _State(tmp_path)

    final_dir = tmp_path / "final"
    final_dir.mkdir()
    monkeypatch.chdir(final_dir)

    ok, _ = level.validate("anything", state)
    assert ok is True
