"""Regression tests for Section 1 maze marker invariants."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from shellgame.levels.sections.section1 import MazeLevel


@dataclass
class _State:
    username: str
    workspace: Path


def test_level1_7_each_dir_has_go_xor_warning(tmp_path: Path, monkeypatch: Any) -> None:
    """Each maze directory must contain either a GO_ file or a warning marker, but never both."""

    workspace = tmp_path / "ws"
    workspace.mkdir()

    level = MazeLevel()
    level.prepare(workspace)

    maze_root = workspace / "level-1" / "maze"
    assert maze_root.is_dir()

    for d in maze_root.rglob("*"):
        if not d.is_dir():
            continue

        entries = [p.name for p in d.iterdir()]
        has_go = any(name.startswith("GO_") for name in entries)
        has_warn = "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE" in entries
        has_victory = "VICTORY.marker" in entries

        if has_victory:
            assert not has_go and not has_warn, f"Victory dir {d} must not have GO or warning markers"
        else:
            # XOR: exactly one of them is present
            assert has_go != has_warn, (
                f"Dir {d} violates marker rule; has_go={has_go}, has_warn={has_warn}, entries={entries}"
            )


def test_level1_7_trap_file_includes_reset_instruction(tmp_path: Path) -> None:
    workspace = tmp_path / "ws"
    workspace.mkdir()

    level = MazeLevel()
    level.prepare(workspace)

    maze_root = workspace / "level-1" / "maze"
    trap_file = maze_root / "entry" / "dungeon" / "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"
    assert trap_file.is_file()
    content = trap_file.read_text(encoding="utf-8")
    assert "shellgame reset" in content
    assert "maze/entry" in content

