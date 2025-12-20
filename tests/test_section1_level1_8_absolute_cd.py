"""Tests for Level 1.8 absolute-cd enforcement logic.

These tests validate the Python-side gate (marker required) that works together
with the shell wrapper's `cd` interception.
"""

from __future__ import annotations

from pathlib import Path

from shellgame.levels.sections.section1 import Level1_8


class _State:
    def __init__(self, workspace: Path, username: str) -> None:
        self.workspace = workspace
        self.username = username


def test_level1_8_submit_without_answer_requires_cwd_and_marker(
    tmp_path: Path, monkeypatch
) -> None:
    level = Level1_8()
    level.setup(tmp_path)

    state = _State(tmp_path, "tester")

    # In the right directory but no marker -> reject
    target = tmp_path / "level-1" / "absolute-target"
    monkeypatch.chdir(target)
    marker_dir = Path(f"/tmp/shellgame-{state.username}")
    marker_dir.mkdir(parents=True, exist_ok=True)
    marker = marker_dir / ".level1_8_absolute_cd_used"
    marker.unlink(missing_ok=True)

    ok, _ = level.validate(None, state)
    assert ok is False

    # With marker -> accept
    marker.write_text("")
    ok2, _ = level.validate(None, state)
    assert ok2 is True


def test_level1_8_answer_is_optional_and_ignored(tmp_path: Path, monkeypatch) -> None:
    level = Level1_8()
    level.setup(tmp_path)

    state = _State(tmp_path, "tester2")
    target = tmp_path / "level-1" / "absolute-target"
    monkeypatch.chdir(target)

    marker_dir = Path(f"/tmp/shellgame-{state.username}")
    marker_dir.mkdir(parents=True, exist_ok=True)
    (marker_dir / ".level1_8_absolute_cd_used").write_text("")

    ok, _ = level.validate("absolute-target", state)
    assert ok is True
    ok2, _ = level.validate("anything", state)
    assert ok2 is True
