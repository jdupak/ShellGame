"""Replay navigation directions from the player's declared starting location."""

import re
import subprocess
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.sections.section1 import AbsoluteCdLevel, SummaryLevel
from shellgame.levels.sections.section2 import SectionChallengeLevel
from shellgame.state.manager import GameState


@pytest.mark.parametrize("reset", [False, True], ids=["fresh", "reset"])
def test_summary_final_hint_path_is_reachable_from_start(tmp_path: Path, reset: bool) -> None:
    level = SummaryLevel()
    level.prepare(tmp_path)
    if reset:
        level.reset(tmp_path)
    start = level.get_start_directory(tmp_path)
    assert start is not None
    commands = re.findall(r"`(cd [^`]+)`", level.hints[-1])
    assert len(commands) == 2

    result = subprocess.run(
        ["bash", "-c", f"{commands[0]} && pwd -P"],
        cwd=start,
        capture_output=True,
        text=True,
        check=True,
        timeout=10,
    )

    assert Path(result.stdout.strip()) == level.section_path(tmp_path) / "gamma/deep/a/b/c"


def test_absolute_hint_uses_actual_start_not_example_username(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    level = AbsoluteCdLevel()
    workspace = tmp_path / "workspace with spaces"
    workspace.mkdir()
    level.prepare(workspace)
    start = level.get_start_directory(workspace)
    assert start is not None
    hint_text = " ".join(level.hints)
    assert "shellgame-user" not in hint_text
    assert "/tmp/" not in hint_text
    assert "pwd" in hint_text
    assert "'/absolute-target'" in hint_text
    destination = Path(f"{start}/absolute-target")
    state = GameState(username="tester", workspace=workspace, current_level=level.id, start_time=datetime.now())
    monkeypatch.chdir(start)
    assert not level.validate(None, state)[0]

    level.hooks["cd"](target=str(destination), pwd=str(start), post_move=False, state=state)
    result = subprocess.run(
        ["bash", "-c", 'cd "$1" && pwd -P', "bash", str(destination)],
        cwd=start,
        capture_output=True,
        text=True,
        check=True,
        timeout=10,
    )
    assert Path(result.stdout.strip()) == destination
    monkeypatch.chdir(destination)
    assert level.validate(None, state)[0]


@pytest.mark.parametrize("reset", [False, True], ids=["fresh", "reset"])
def test_section_two_challenge_clues_lead_to_required_destination(tmp_path: Path, reset: bool) -> None:
    level = SectionChallengeLevel()
    level.prepare(tmp_path)
    if reset:
        level.reset(tmp_path)
    start = level.get_start_directory(tmp_path)
    assert start == level.section_path(tmp_path)
    assert "challenge/room1" in level.instructions
    assert "room2" in level.instructions
    assert "password.txt" in level.instructions
    assert "`cd -`" not in level.instructions

    destination = level.section_path(tmp_path) / "challenge/room2"
    state = GameState(username="tester", workspace=tmp_path, current_level=level.id, start_time=datetime.now())
    assert not level.validate("navigator", state)[0]

    with pytest.MonkeyPatch.context() as monkeypatch:
        monkeypatch.chdir(destination)
        assert level.validate("navigator", state)[0]
