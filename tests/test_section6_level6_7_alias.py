import shutil
import subprocess
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.sections.section6 import (
    FileOrganizerChallengeLevel,
    ShellgameAliasLevel,
    section,
)
from shellgame.state.manager import GameState


def _state(workspace: Path) -> GameState:
    return GameState(
        username="tester",
        workspace=workspace,
        current_level="6.7",
        start_time=datetime.now(),
    )


def test_level6_7_teaches_alias_for_both_shells_and_its_scope() -> None:
    level = ShellgameAliasLevel()

    assert level.id == "6.7"
    assert "alias sg='shellgame'" in level.instructions
    assert "alias sg shellgame" in level.instructions
    assert "type sg" in level.instructions
    assert "jen v aktuálním herním shellu" in level.instructions
    assert "po jeho ukončení zmizí" in level.instructions
    assert "sg submit alias-ready" in level.instructions
    assert "místo dlouhého `shellgame` používat kratší `sg`" in level.instructions
    assert "použití `sg` je součást tohoto cvičení" in level.instructions


@pytest.mark.parametrize(
    ("shell", "command"),
    [
        (
            "bash",
            "function shellgame(){ printf '%s\\n' \"$*\"; }; "
            "shopt -s expand_aliases; alias sg='shellgame'; eval 'sg submit alias-ready'",
        ),
        (
            "fish",
            "function shellgame; string join ' ' $argv; end; "
            "alias sg shellgame; sg submit alias-ready",
        ),
    ],
)
def test_level6_7_displayed_alias_commands_work(shell: str, command: str) -> None:
    executable = shutil.which(shell)
    if executable is None:
        pytest.skip(f"{shell} is not installed")

    result = subprocess.run(  # noqa: S603
        [executable, "--no-config" if shell == "fish" else "--noprofile", "-c", command],
        capture_output=True,
        text=True,
        check=True,
        timeout=10,
    )

    assert result.stdout.strip() == "submit alias-ready"


def test_level6_7_completion_requires_explicit_alias_ready_answer(tmp_path: Path) -> None:
    level = ShellgameAliasLevel()
    level.prepare(tmp_path)
    state = _state(tmp_path)

    success, message = level.validate("alias-ready", state)
    assert success is True
    assert message == level.success_message

    missing, missing_message = level.validate(None, state)
    assert missing is False
    assert "sg submit alias-ready" in missing_message

    wrong, _ = level.validate("ready", state)
    assert wrong is False


def test_section6_ends_on_the_challenge_not_the_alias_aside() -> None:
    """Every section closes on its summary/challenge.

    The alias level is a shell-ergonomics aside, so it was moved ahead of the
    challenge. Only the last level may claim the section is finished.
    """
    levels = {level.id: level for level in section.levels}

    assert section.levels[-1].id == FileOrganizerChallengeLevel.id
    assert "Dokončili jste Sekci 6" in levels[FileOrganizerChallengeLevel.id].success_message
    assert "Dokončili jste Sekci 6" not in levels[ShellgameAliasLevel.id].success_message
