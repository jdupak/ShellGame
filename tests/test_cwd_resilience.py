"""The game must survive the player deleting the directory they stand in.

Sections 3 and 4 teach `rm`, `rmdir` and `mv`, so a player can delete their own
working directory. After that `getcwd()` fails for every process the shell
spawns. Nothing may escape as a traceback.
"""

from __future__ import annotations

import os
from collections.abc import Iterator
from datetime import datetime
from pathlib import Path

import click
import pytest
from click.testing import CliRunner

from shellgame.cli import commands
from shellgame.core.navigation import NavigationManager
from shellgame.levels.base import Level
from shellgame.levels.completion import AtDirectory, AtHome, Completion
from shellgame.messages import Messages
from shellgame.paths import current_directory
from shellgame.state.manager import GameState


@pytest.fixture
def deleted_cwd(tmp_path: Path) -> Iterator[None]:
    """Stand inside a directory and then delete it."""
    original = Path.cwd()
    doomed = tmp_path / "doomed"
    doomed.mkdir()
    os.chdir(doomed)
    doomed.rmdir()
    try:
        yield
    finally:
        os.chdir(original)


def _state(workspace: Path) -> GameState:
    return GameState(
        username="tester",
        workspace=workspace,
        current_level="1.1",
        start_time=datetime.now(),
    )


def test_current_directory_returns_none_when_cwd_is_gone(deleted_cwd: None) -> None:
    assert current_directory() is None


def test_current_directory_resolves_a_live_directory(tmp_path: Path) -> None:
    assert current_directory() is not None


def test_at_directory_reports_missing_cwd_instead_of_raising(tmp_path: Path, deleted_cwd: None) -> None:
    workspace = tmp_path / "workspace"
    (workspace / "level-1").mkdir(parents=True)

    success, message = AtDirectory("level-1").check(_state(workspace), workspace)

    assert success is False
    assert message == Messages.CWD_MISSING


def test_at_home_reports_missing_cwd_instead_of_raising(tmp_path: Path, deleted_cwd: None) -> None:
    success, message = AtHome().check(_state(tmp_path), tmp_path)

    assert success is False
    assert message == Messages.CWD_MISSING


def test_completion_validate_survives_missing_cwd(tmp_path: Path, deleted_cwd: None) -> None:
    workspace = tmp_path / "workspace"
    (workspace / "level-1").mkdir(parents=True)
    completion = Completion(requirements=(AtDirectory("level-1"),))

    success, message = completion.validate(None, _state(workspace), root=workspace, success_message="ok")

    assert success is False
    assert message == Messages.CWD_MISSING


def test_returns_to_start_is_false_when_cwd_is_gone(tmp_path: Path, deleted_cwd: None) -> None:
    class _Level(Level):
        title = "Relative move"
        completion = Completion()
        start_directory = "level-1"

    level = _Level()

    # A relative target with no `pwd` must resolve against the cwd, which is gone.
    assert level.returns_to_start(target="level-1", pwd=None, state=_state(tmp_path)) is False


def test_returns_to_start_still_works_when_pwd_is_supplied(tmp_path: Path, deleted_cwd: None) -> None:
    class _Level(Level):
        title = "Relative move"
        completion = Completion()
        start_directory = "level-1"

    workspace = tmp_path / "workspace"
    (workspace / "level-1").mkdir(parents=True)
    level = _Level()

    assert level.returns_to_start(target="level-1", pwd=str(workspace), state=_state(workspace)) is True


@pytest.mark.parametrize("cwd_missing", [False, True])
@pytest.mark.parametrize(
    "error",
    [
        FileNotFoundError(2, "No such file or directory"),
        IsADirectoryError(21, "Is a directory", "pass.txt"),
        PermissionError(13, "Permission denied", "pass.txt"),
    ],
)
def test_cli_renders_os_errors_instead_of_a_traceback(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, cwd_missing: bool, error: OSError
) -> None:
    shown: list[str] = []

    class _Display:
        def show_state_error(self, message: str) -> None:
            shown.append(message)

    monkeypatch.setattr(commands, "display", _Display())
    monkeypatch.setattr(commands, "current_directory", lambda: None if cwd_missing else tmp_path)

    @click.group(cls=commands.CzechGroup)
    def group() -> None: ...

    @group.command()
    def boom() -> None:
        raise error

    result = CliRunner().invoke(group, ["boom"])

    assert result.exit_code == 1
    assert shown and str(error) in shown[0]
    if cwd_missing:
        assert Messages.CWD_MISSING in shown[0]
    else:
        assert shown[0] == Messages.FILESYSTEM_ERROR.format(error=error)
        assert Messages.CWD_MISSING not in shown[0]


def test_navigation_teleports_when_cwd_is_gone(tmp_path: Path, deleted_cwd: None) -> None:
    class _Shell:
        def __init__(self) -> None:
            self.destinations: list[Path] = []

        def cd(self, destination: Path) -> None:
            self.destinations.append(destination)

    notices: list[Path] = []
    shell = _Shell()
    manager = NavigationManager(None, shell, notices.append)  # type: ignore[arg-type]

    target = tmp_path / "start"
    target.mkdir()
    manager.maybe_teleport(target)

    assert shell.destinations == [target]
    assert notices == [target]
