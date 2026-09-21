"""Regression tests for Section 1 maze content."""

from pathlib import Path

from shellgame.levels.sections.section1 import MazeLevel, resolve_maze_instruction


def test_level1_7_maze_has_first_step(tmp_path: Path) -> None:
    level = MazeLevel()
    level.prepare(tmp_path)

    start_dir = tmp_path / "level-1" / "maze" / "entry"
    assert start_dir.is_dir()
    instruction = start_dir / "GO_TO_DIR_hall"
    assert instruction.is_file()

    destination = resolve_maze_instruction(instruction.name, start_dir)
    assert destination is not None
    assert destination.is_dir()
    assert destination == tmp_path / "level-1" / "maze" / "entry" / "hall"


def test_level1_7_deep_step_go_up_2_then_go_to_catacombs_is_valid(tmp_path: Path) -> None:
    level = MazeLevel()
    level.prepare(tmp_path)

    maze_root = tmp_path / "level-1" / "maze"
    deep_step = maze_root / "entry" / "hall" / "nexus" / "passages" / "tunnel" / "cavern" / "depths"
    assert deep_step.is_dir()

    instruction = deep_step / "GO_UP_2_THEN_GO_TO_catacombs"
    assert instruction.is_file()

    destination = resolve_maze_instruction(instruction.name, deep_step)
    assert destination == maze_root / "entry" / "hall" / "nexus" / "passages" / "tunnel" / "catacombs"
    assert destination.is_dir()


def test_level1_7_main_route_provides_sustained_navigation_practice(tmp_path: Path) -> None:
    level = MazeLevel()
    level.prepare(tmp_path)

    current = tmp_path / "level-1" / "maze" / "entry"
    final = tmp_path / "level-1" / level._SANCTUARY
    visited = {current}
    instructions: list[str] = []

    while current != final:
        go_files = sorted(path for path in current.glob("GO_*") if path.is_file())
        assert len(go_files) == 1, f"Expected one instruction in {current}, found {go_files}"
        instruction = go_files[0].name
        instructions.append(instruction)
        destination = resolve_maze_instruction(instruction, current)
        assert destination is not None and destination.is_dir()
        assert destination not in visited, f"Main route loops at {destination}"
        visited.add(destination)
        current = destination

    assert len(instructions) >= 16
    assert instructions.count("GO_UP_2_THEN_GO_TO_catacombs") == 1
    assert instructions.count("GO_UP_2_THEN_GO_TO_labyrinth") == 1
    assert (final / "VICTORY.marker").is_file()


def test_every_maze_instruction_resolves_to_an_existing_directory(tmp_path: Path) -> None:
    """A GO_ file that cannot be followed is a soft-lock dressed up as a puzzle."""
    level = MazeLevel()
    level.prepare(tmp_path)

    maze_root = tmp_path / "level-1" / "maze"
    broken: list[str] = []

    for instruction in maze_root.rglob("GO_*"):
        if not instruction.is_file():
            continue
        destination = resolve_maze_instruction(instruction.name, instruction.parent)
        if destination is None or not destination.is_dir():
            broken.append(f"{instruction.relative_to(maze_root)} -> {destination}")

    assert not broken, "Unfollowable maze instructions:\n  " + "\n  ".join(broken)


def test_maze_instruction_files_are_not_blank(tmp_path: Path) -> None:
    """Empty GO_ files make `cat` look like the level is broken."""
    level = MazeLevel()
    level.prepare(tmp_path)

    blank = [
        str(path.relative_to(tmp_path))
        for path in (tmp_path / "level-1" / "maze").rglob("GO_*")
        if path.is_file() and not path.read_text(encoding="utf-8").strip()
    ]
    assert not blank, f"Blank instruction files: {blank}"
