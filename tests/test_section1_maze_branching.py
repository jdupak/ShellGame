"""Tests that the Section 1 maze feels like a maze (has branches/decoys)."""

from pathlib import Path

from shellgame.levels.sections.section1 import Level1_7


def test_level1_7_maze_has_branches_and_a_loop(tmp_path: Path) -> None:
    level = Level1_7()
    level.setup(tmp_path)

    maze_root = tmp_path / "level-1" / "maze"
    start = maze_root / "00"
    assert start.is_dir()

    # Branching at the start: at least one decoy directory besides the "right" path.
    assert (start / "01").exists() is False  # 01 is sibling of 00, not inside it
    assert (maze_root / "01").is_dir()
    assert (start / "side").is_dir()
    assert (start / "trap").is_dir()

    # A small loop: 00/side/loop contains an instruction that points back to 00.
    loop_dir = start / "side" / "loop"
    assert loop_dir.is_dir()
    assert (loop_dir / "GO_UP_2_THEN_GO_TO_00").is_file()
