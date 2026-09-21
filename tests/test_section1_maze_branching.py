"""Tests that the Section 1 maze feels like a maze (has branches/decoys)."""

from pathlib import Path

from shellgame.levels.sections.section1 import MazeLevel, resolve_maze_instruction


def test_level1_7_maze_has_branches_loops_and_many_decoys(tmp_path: Path) -> None:
    level = MazeLevel()
    level.prepare(tmp_path)

    maze_root = tmp_path / "level-1" / "maze"
    start = maze_root / "entry"
    assert start.is_dir()

    # Branching at the start: decoys along with the right path
    assert (start / "hall").is_dir()
    assert (start / "dungeon").is_dir()
    assert (start / "courtyard").is_dir()

    # Local loop: the instruction inside side_door sends the player back to hall.
    hall = start / "hall"
    side_door = hall / "side_door"
    assert side_door.is_dir()
    assert (side_door / "GO_UP_1").is_file()
    assert resolve_maze_instruction("GO_UP_1", side_door) == hall

    # The route contains enough wrong branches that learners must keep reading
    # instructions instead of guessing a single linear sequence.
    traps = list(maze_root.rglob("YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"))
    assert len(traps) >= 12

    nexus = hall / "nexus"
    assert (nexus / "passages").is_dir()
    assert (nexus / "sanctum").is_dir()
    assert (nexus / "rotunda").is_dir()
    assert (nexus / "archives").is_dir()
