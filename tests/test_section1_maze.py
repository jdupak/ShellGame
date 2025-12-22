"""Regression tests for Section 1 maze content."""

from pathlib import Path

from shellgame.levels.sections.section1 import MazeLevel


def test_level1_7_maze_has_first_step(tmp_path: Path) -> None:
    level = MazeLevel()
    level.setup(tmp_path)

    start_dir = tmp_path / "level-1" / "maze" / "00"
    assert start_dir.is_dir()
    assert (start_dir / "GO_TO_DIR_01").is_file()

    # The first instruction must point to an actual directory.
    assert (tmp_path / "level-1" / "maze" / "01").is_dir()


def test_level1_7_deep_step_go_up_4_then_go_to_02_is_valid(tmp_path: Path) -> None:
    level = MazeLevel()
    level.setup(tmp_path)

    maze_root = tmp_path / "level-1" / "maze"
    deep_step = maze_root / "01" / "deep" / "a" / "b"
    assert deep_step.is_dir()

    instr = deep_step / "GO_UP_4_THEN_GO_TO_02"
    assert instr.is_file()

    # Simulate the meaning: go up 4 levels from b -> a -> deep -> 01 -> maze
    landing = deep_step
    for _ in range(4):
        landing = landing.parent
    assert landing == maze_root
    assert (landing / "02").is_dir()
