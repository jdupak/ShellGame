"""Tests that the Section 1 maze feels like a maze (has branches/decoys)."""

from pathlib import Path

from shellgame.levels.sections.section1 import MazeLevel, resolve_maze_instruction


def test_level1_7_maze_has_branches_and_a_loop(tmp_path: Path) -> None:
    level = MazeLevel()
    level.prepare(tmp_path)

    maze_root = tmp_path / "level-1" / "maze"
    start = maze_root / "entry"
    assert start.is_dir()

    # Branching at the start: decoys along with the right path
    assert (start / "hall").is_dir()
    assert (start / "dungeon").is_dir()
    assert (start / "courtyard").is_dir()

    # A local backtrack loop: entry/hall/side_door points back to entry/hall via GO_UP_1
    side_door = start / "hall" / "side_door"
    assert side_door.is_dir()
    assert (side_door / "GO_UP_1").is_file()
    assert resolve_maze_instruction("GO_UP_1", side_door) == start / "hall"
