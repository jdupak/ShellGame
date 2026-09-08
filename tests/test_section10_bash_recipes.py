"""Replay the displayed Bash-only recipes from both supported game shells."""

import os
import re
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.base import Level
from shellgame.levels.sections.section10 import (
    CharacterClassWildcardCopyLevel,
    QuestionMarkWildcardCopyLevel,
    RangeWildcardCopyLevel,
    SectionIntro,
    StarWildcardCopyLevel,
    WildcardsChallengeLevel,
)
from shellgame.state.manager import GameState

BASH_LEVELS = (QuestionMarkWildcardCopyLevel, CharacterClassWildcardCopyLevel, RangeWildcardCopyLevel)


@pytest.mark.parametrize("level_type", BASH_LEVELS, ids=lambda level: level.id)
@pytest.mark.parametrize("shell", ["bash", "fish"])
@pytest.mark.parametrize("source", ["instructions", "hint"])
def test_displayed_recipe_completes_fresh_and_reset_levels(
    level_type: type[Level], shell: str, source: str, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    if not shutil.which(shell) or not shutil.which("bash"):
        pytest.skip(f"{shell} and bash are required")
    workspace = tmp_path / "workspace with spaces"
    workspace.mkdir()
    home = tmp_path / "home"
    home.mkdir()
    env = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("SHELLGAME_", "BASH_FUNC_"))
        and key not in {"BASH_ENV", "ENV", "BASHOPTS", "SHELLOPTS", "CDPATH", "OLDPWD"}
    }
    env.update(
        HOME=str(home),
        XDG_CONFIG_HOME=str(home / "config"),
        XDG_DATA_HOME=str(home / "data"),
        XDG_CACHE_HOME=str(home / "cache"),
    )
    level = level_type()
    assert "vyžaduje **Bash**" in level.instructions
    text = level.instructions if source == "instructions" else level.hints[-1]
    prefix = "cp " if shell == "bash" else "bash -c "
    commands = re.findall(rf"`({re.escape(prefix)}[^`]+)`", text)
    assert len(commands) == 1
    flags = ["--no-config"] if shell == "fish" else []
    state = GameState(username="tester", workspace=workspace, current_level=level.id, start_time=datetime.now())

    for prepare in (level.prepare, level.reset):
        prepare(workspace)
        start = level.get_start_directory(workspace)
        assert start is not None
        monkeypatch.chdir(start)
        assert not level.validate(None, state)[0]
        result = subprocess.run(
            [shell, *flags, "-c", commands[0]],
            cwd=start,
            env=env,
            capture_output=True,
            text=True,
            check=True,
            timeout=10,
        )
        assert result.stderr == ""
        success, message = level.validate(None, state)
        assert success, message


def test_intro_scopes_bash_requirement_and_preserves_game_shell() -> None:
    instructions = SectionIntro().instructions
    assert "10.2–10.4 vyžadují Bash" in instructions
    assert "bash -c" in instructions
    assert "shellgame submit" in instructions
    assert "stiskněte Enter" in instructions
    for level in (StarWildcardCopyLevel(), WildcardsChallengeLevel()):
        assert "vyžaduje **Bash**" not in level.instructions
