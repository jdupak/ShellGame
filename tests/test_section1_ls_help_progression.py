"""Focused checks for the early ``ls`` and help progression."""

from pathlib import Path

from shellgame.levels.sections.section1 import LsLevel


def test_level1_2_moves_from_plain_ls_to_discovering_long_listing() -> None:
    level = LsLevel()
    instructions = level.instructions

    assert instructions.index("`ls`") < instructions.index("`ls --help`")
    assert "`man ls`" in instructions
    assert "dlouhý (podrobný) výpis" in instructions
    assert "ls -F" not in instructions


def test_level1_2_hints_reveal_exact_commands_late() -> None:
    hints = LsLevel().hints

    assert all(command not in " ".join(hints[:2]) for command in ("ls --help", "man ls", "ls -l"))
    assert "ls --help" in hints[-2]
    assert "man ls" in hints[-2]
    assert "ls -l" in hints[-1]
    assert all("ls -F" not in hint for hint in hints)


def test_section1_intro_lists_plain_ls_before_help_tools() -> None:
    intro_path = Path(__file__).parents[1] / "src/shellgame/levels/content/section1_intro.md"
    intro = intro_path.read_text(encoding="utf-8")

    assert intro.index("`ls`") < intro.index("`<příkaz> --help`") < intro.index("`man <příkaz>`")
    assert "ls -F" not in intro
