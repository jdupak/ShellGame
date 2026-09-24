"""Tests for `shellgame levels` and the choices `resume` offers.

A player who does not know the level IDs has no way to guess them, so the
listing is the entry point to `resume` and must work before the game shell
exists — and without dragging one up behind it.
"""

from __future__ import annotations

from datetime import datetime
from pathlib import Path
from typing import Any

import pytest
from click.testing import CliRunner

from shellgame.cli import commands
from shellgame.core.session import GameSession
from shellgame.state.manager import GameState, LevelCompletion
from tests.fakes import (
    _FakeConsole,
    _FakeDisplay,
    _FakeLevel,
    _FakeRegistry,
    _FakeShell,
    _FakeStateManager,
    _FakeWorkspace,
)


def _make_state(tmp_path: Path, current_level: str = "1.1") -> GameState:
    return GameState(
        username="student",
        current_level=current_level,
        workspace=tmp_path / "ws",
        start_time=datetime.now(),
    )


def _make_session(
    tmp_path: Path, levels: list[_FakeLevel], state: GameState | None
) -> tuple[GameSession, _FakeDisplay]:
    workspace_root = tmp_path / "ws"
    workspace_root.mkdir(parents=True, exist_ok=True)
    display = _FakeDisplay()
    session = GameSession(
        console=_FakeConsole(),
        display=display,  # type: ignore[arg-type]
        state_manager=_FakeStateManager(state),  # type: ignore[arg-type]
        level_registry=_FakeRegistry(levels),  # type: ignore[arg-type]
        teleport_notice=lambda _destination: None,
        shell_client=_FakeShell(),  # type: ignore[arg-type]
        workspace_factory=lambda _workspace: _FakeWorkspace(workspace_root),  # type: ignore[arg-type]
    )
    return session, display


def _levels() -> list[_FakeLevel]:
    return [_FakeLevel("1.1"), _FakeLevel("1.2"), _FakeLevel("2.0"), _FakeLevel("2.1")]


def test_levels_lists_every_registered_level(tmp_path: Path) -> None:
    session, display = _make_session(tmp_path, _levels(), _make_state(tmp_path))

    session.levels()

    args, _kwargs = display.call("show_levels")
    assert [level.id for level in args[0]] == ["1.1", "1.2", "2.0", "2.1"]


def test_levels_marks_where_the_player_is(tmp_path: Path) -> None:
    state = _make_state(tmp_path, current_level="2.0")
    state.levels_complete["1.1"] = LevelCompletion(time_sec=1, hints=0, attempts=0, completed_at=datetime.now())
    session, display = _make_session(tmp_path, _levels(), state)

    session.levels()

    _args, kwargs = display.call("show_levels")
    assert kwargs["current_level"] == "2.0"
    assert kwargs["completed"] == {"1.1"}


def test_levels_works_before_the_game_is_initialized(tmp_path: Path) -> None:
    """The listing is how a player finds a level, so it cannot require a save."""
    session, display = _make_session(tmp_path, _levels(), None)

    session.levels()

    _args, kwargs = display.call("show_levels")
    assert kwargs["current_level"] is None
    assert kwargs["completed"] == set()
    assert "show_not_initialized" not in display.names()


def test_resume_without_a_level_reports_the_usage_and_the_listing(tmp_path: Path) -> None:
    """Missing ID is a usage error, but the IDs cannot be guessed either."""
    session, display = _make_session(tmp_path, _levels(), _make_state(tmp_path))

    session.resume(None)

    assert "show_missing_level_id" in display.names()
    assert "show_levels" in display.names()


def test_resume_without_a_level_changes_nothing(tmp_path: Path) -> None:
    state = _make_state(tmp_path, current_level="1.1")
    session, _display = _make_session(tmp_path, _levels(), state)

    session.resume(None)

    assert state.current_level == "1.1"


@pytest.mark.parametrize("level_id", [None, "nonsense", "9.9"])
def test_resume_reports_failure_to_the_caller(tmp_path: Path, level_id: str | None) -> None:
    """The CLI turns this into a non-zero exit status."""
    session, _display = _make_session(tmp_path, _levels(), _make_state(tmp_path))

    assert session.resume(level_id) is False


def test_resume_reports_success_to_the_caller(tmp_path: Path) -> None:
    session, _display = _make_session(tmp_path, _levels(), _make_state(tmp_path))

    assert session.resume("2.1") is True


def test_cli_resume_without_a_level_exits_non_zero(monkeypatch: Any) -> None:
    runner = CliRunner()

    class _StubSession:
        def resume(self, level_id: str | None) -> bool:
            assert level_id is None
            return False

    monkeypatch.setattr(commands, "_get_session", lambda: _StubSession())

    result = runner.invoke(commands.cli, ["resume"], env={"SHELLGAME_WRAPPER": "1"})

    assert result.exit_code == 1


def test_cli_resume_with_a_level_exits_zero(monkeypatch: Any) -> None:
    runner = CliRunner()

    class _StubSession:
        def resume(self, level_id: str | None) -> bool:
            return True

    monkeypatch.setattr(commands, "_get_session", lambda: _StubSession())

    result = runner.invoke(commands.cli, ["resume", "2.1"], env={"SHELLGAME_WRAPPER": "1"})

    assert result.exit_code == 0


def test_unknown_level_points_at_the_listing(tmp_path: Path) -> None:
    session, display = _make_session(tmp_path, _levels(), _make_state(tmp_path))

    session.resume("9.9")

    assert "show_unknown_level" in display.names()


class _FakeContext:
    def __init__(self, invoked_subcommand: str | None) -> None:
        self.invoked_subcommand = invoked_subcommand


@pytest.fixture
def _registry(monkeypatch: Any) -> None:
    monkeypatch.setattr(commands, "level_registry", _FakeRegistry(_levels()))


def _skips_shell(argv: list[str], invoked: str, monkeypatch: Any) -> bool:
    monkeypatch.setattr(commands.sys, "argv", ["shellgame", *argv])
    return commands._runs_without_game_shell(_FakeContext(invoked))  # type: ignore[arg-type]


@pytest.mark.usefixtures("_registry")
@pytest.mark.parametrize(
    ("argv", "invoked"),
    [
        (["levels"], "levels"),
        (["resume"], "resume"),
        (["resume", "9.9"], "resume"),
        (["resume", "nonsense"], "resume"),
        (["resume", "--help"], "resume"),
        (["resume", "1.1", "2.1"], "resume"),
    ],
)
def test_printing_only_invocations_never_launch_a_subshell(argv: list[str], invoked: str, monkeypatch: Any) -> None:
    """These only print, so trapping the player in a subshell would be wrong."""
    assert _skips_shell(argv, invoked, monkeypatch) is True


@pytest.mark.usefixtures("_registry")
@pytest.mark.parametrize(
    ("argv", "invoked"),
    [
        (["resume", "2.1"], "resume"),
        (["status"], "status"),
        (["hint"], "hint"),
        (["submit", "answer"], "submit"),
    ],
)
def test_gameplay_invocations_still_launch_the_game_shell(argv: list[str], invoked: str, monkeypatch: Any) -> None:
    assert _skips_shell(argv, invoked, monkeypatch) is False


def test_bare_launch_still_starts_the_game(monkeypatch: Any) -> None:
    monkeypatch.setattr(commands.sys, "argv", ["shellgame"])
    assert commands._runs_without_game_shell(_FakeContext(None)) is False  # type: ignore[arg-type]
