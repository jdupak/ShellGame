"""Replay the displayed Bash-only recipes from both supported game shells.

Section 10 teaches wildcard *construction*, so the task text deliberately shows
only the shape of the command (`cp VZOR short_data/`). The ready-made pattern
lives in the final hint, which is the surface this module replays: a recipe the
game prints must actually work, fresh and after `reset()`, in bash and in fish.
"""

import os
import re
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.base import Level
from shellgame.levels.sections.section7 import (
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
def test_displayed_recipe_completes_fresh_and_reset_levels(
    level_type: type[Level], shell: str, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
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
    text = level.hints[-1]
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


@pytest.mark.parametrize("level_type", BASH_LEVELS, ids=lambda level: level.id)
def test_task_text_shows_the_shape_not_the_finished_pattern(level_type: type[Level]) -> None:
    """Constructing the glob is the whole lesson, so the task must not hand it over.

    Without this, section 10 degrades into transcription: the learner copies the
    pattern out of the instructions and never has to reason about `?`, `[...]`
    or a character class at all.
    """
    level = level_type()
    solution_step = level.solution.steps[0]
    pattern = str(solution_step.command).split()[1]

    assert "VZOR" in level.instructions, f"{level.id}: task text must show the placeholder shape"
    assert pattern not in level.instructions, f"{level.id}: task text reveals the finished pattern {pattern!r}"
    assert pattern in level.hints[-1], f"{level.id}: final hint must still carry the finished pattern"


def test_intro_scopes_bash_requirement_and_preserves_game_shell() -> None:
    instructions = SectionIntro().instructions
    assert "7.2–7.4 vyžadují Bash" in instructions
    assert "bash -c" in instructions
    assert "shellgame submit" in instructions
    assert "stiskněte Enter" in instructions
    for level in (StarWildcardCopyLevel(), WildcardsChallengeLevel()):
        assert "vyžaduje **Bash**" not in level.instructions
