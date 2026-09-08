"""Tests for Level 1.8 absolute-cd enforcement logic.

These tests validate the Python-side gate (marker required) that works together
with the shell wrapper's `cd` interception.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from shellgame.levels.cdpolicy import cd_marker
from shellgame.levels.sections.section1 import AbsoluteCdLevel
from shellgame.markers import MarkerManager


class _State:
    def __init__(self, workspace: Path, username: str) -> None:
        self.workspace = workspace
        self.username = username
        self.current_level = "1.8"


def test_level1_8_submit_without_answer_requires_cwd_and_marker(tmp_path: Path, monkeypatch: Any) -> None:
    level = AbsoluteCdLevel()
    level.prepare(tmp_path)

    state = _State(tmp_path, "tester")

    # In the right directory but no marker -> reject
    target = tmp_path / "level-1" / "absolute-target"
    monkeypatch.chdir(target)
    state.workspace.mkdir(parents=True, exist_ok=True)
    markers = MarkerManager(state.workspace)
    markers.remove(cd_marker("1.8"))

    ok, _ = level.validate(None, state)
    assert ok is False

    # With marker -> accept
    markers.create(cd_marker("1.8"))
    ok2, _ = level.validate(None, state)
    assert ok2 is True


def test_level1_8_answer_is_optional_and_ignored(tmp_path: Path, monkeypatch: Any) -> None:
    level = AbsoluteCdLevel()
    level.prepare(tmp_path)

    state = _State(tmp_path, "tester2")
    target = tmp_path / "level-1" / "absolute-target"
    monkeypatch.chdir(target)

    state.workspace.mkdir(parents=True, exist_ok=True)
    MarkerManager(state.workspace).create(cd_marker("1.8"))

    ok, _ = level.validate("absolute-target", state)
    assert ok is True
    ok2, _ = level.validate("anything", state)
    assert ok2 is True
