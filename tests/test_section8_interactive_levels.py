"""Focused tests for the interactive terminal lessons in section 8."""

from __future__ import annotations

import shutil
import subprocess
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.sections.section8 import (
    CommandStatusLevel,
    EchoCreateFileLevel,
    InteractiveCatInputLevel,
    section,
)
from shellgame.state.manager import GameState


def _state(workspace: Path, level_id: str) -> GameState:
    return GameState(
        username="tester",
        workspace=workspace,
        current_level=level_id,
        start_time=datetime.now(),
    )


def _run_shell(
    shell: str,
    command: str,
    cwd: Path,
    *,
    input_text: str | None = None,
) -> None:
    executable = shutil.which(shell)
    if executable is None:
        pytest.skip(f"{shell} is not installed")
    _ = subprocess.run(  # noqa: S603
        [executable, "-c", command],
        cwd=cwd,
        input=input_text,
        text=True,
        check=True,
        capture_output=True,
    )


def test_level_8_4_quotes_real_output_filename_and_multiword_text(tmp_path: Path) -> None:
    level = EchoCreateFileLevel()
    level.prepare(tmp_path)
    start = level.get_start_directory(tmp_path)
    assert start is not None

    _run_shell("bash", 'echo "Ahoj svete" > "muj pozdrav.txt"', start)

    output = start / "muj pozdrav.txt"
    assert level.id == "8.4"
    assert output.read_text(encoding="utf-8") == "Ahoj svete\n"
    assert level.validate("muj pozdrav.txt", _state(tmp_path, level.id))[0]
    assert '"Ahoj svete"' in level.instructions
    assert '"muj pozdrav.txt"' in level.instructions


def test_level_8_10_reads_stdin_until_eof_and_requires_exact_content(tmp_path: Path) -> None:
    level = InteractiveCatInputLevel()
    level.prepare(tmp_path)
    start = level.get_start_directory(tmp_path)
    assert start is not None
    state = _state(tmp_path, level.id)

    _run_shell(
        "bash",
        "cat > poznamka.txt",
        start,
        input_text="První řádek\nDruhý řádek\n",
    )

    assert level.id == "8.10"
    assert level.validate("poznamka.txt", state)[0]
    assert "Ctrl+D" in level.instructions and "EOF" in level.instructions
    assert "Ctrl+C" in level.instructions and "interrupt" in level.instructions

    _ = (start / "poznamka.txt").write_text("První řádek\nDruhý řádek\nnavíc\n", encoding="utf-8")
    assert not level.validate("poznamka.txt", state)[0]


@pytest.mark.parametrize("shell", ["bash", "fish"])
def test_level_8_11_status_chains_work_in_supported_shells(shell: str, tmp_path: Path) -> None:
    level = CommandStatusLevel()
    level.prepare(tmp_path)
    start = level.get_start_directory(tmp_path)
    assert start is not None

    command = """\
true && echo "stav: uspech" > status.txt
false || echo "stav: neuspech" >> status.txt
"""
    _run_shell(shell, command, start)

    assert level.id == "8.11"
    assert (start / "status.txt").read_text(encoding="utf-8") == "stav: uspech\nstav: neuspech\n"
    assert level.validate("status.txt", _state(tmp_path, level.id))[0]


def test_level_8_11_is_the_section_completion() -> None:
    levels = {level.id: level for level in section.levels}

    assert section.levels[-1].id == "8.11"
    assert "souhrn" not in levels["8.9"].title.lower()
    assert "Dokončili jste Sekci 8" not in levels["8.9"].success_message
    assert "Dokončili jste Sekci 8" in levels["8.11"].success_message
    assert levels["8.10"].solution is not None
    assert levels["8.11"].solution is not None
