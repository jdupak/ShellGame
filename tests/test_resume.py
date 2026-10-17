"""Tests for `shellgame resume LEVEL`.

The contract a player relies on: resuming moves them to the requested level
*and* records it, so quitting and launching ShellGame again continues there.
"""

from __future__ import annotations

from datetime import datetime
from pathlib import Path

import pytest

from shellgame.core.session import GameSession
from shellgame.state.manager import GameState
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
) -> tuple[GameSession, _FakeDisplay, _FakeStateManager]:
    display = _FakeDisplay()
    state_manager = _FakeStateManager(state)
    workspace_root = tmp_path / "ws"
    workspace_root.mkdir(parents=True, exist_ok=True)
    session = GameSession(
        console=_FakeConsole(),
        display=display,  # type: ignore[arg-type]
        state_manager=state_manager,  # type: ignore[arg-type]
        level_registry=_FakeRegistry(levels),  # type: ignore[arg-type]
        teleport_notice=lambda _destination: None,
        shell_client=_FakeShell(),  # type: ignore[arg-type]
        workspace_factory=lambda _workspace: _FakeWorkspace(workspace_root),  # type: ignore[arg-type]
    )
    return session, display, state_manager


def _levels() -> list[_FakeLevel]:
    return [_FakeLevel("1.1"), _FakeLevel("1.2"), _FakeLevel("2.0"), _FakeLevel("2.1")]


def test_resume_moves_to_requested_level_and_persists_it(tmp_path: Path) -> None:
    state = _make_state(tmp_path, current_level="1.1")
    session, display, state_manager = _make_session(tmp_path, _levels(), state)

    session.resume("2.1")

    assert state.current_level == "2.1"
    assert state_manager.state is not None
    assert state_manager.state.current_level == "2.1"
    assert state_manager.saves >= 1
    assert "show_resumed" in display.names()


def test_relaunch_continues_from_resumed_level(tmp_path: Path) -> None:
    """The point of the feature: the next launch starts where resume left off."""
    state = _make_state(tmp_path, current_level="1.1")
    session, _, state_manager = _make_session(tmp_path, _levels(), state)

    session.resume("2.0")

    # A fresh process: new session, same persisted state.
    relaunched, display, _ = _make_session(tmp_path, _levels(), state_manager.state)
    relaunched.show_current_level()

    args, _kwargs = display.call("show_instructions")
    assert args[0].id == "2.0"


def test_resume_prepares_the_target_level(tmp_path: Path) -> None:
    levels = _levels()
    target = next(level for level in levels if level.id == "2.1")
    session, _, _ = _make_session(tmp_path, levels, _make_state(tmp_path))

    session.resume("2.1")

    assert target.prepared >= 1


def test_resume_reopens_a_finished_run(tmp_path: Path) -> None:
    """Without clearing `completed_at`, every command prints the end screen."""
    state = _make_state(tmp_path, current_level="2.1")
    state.completed_at = datetime.now()
    session, display, _ = _make_session(tmp_path, _levels(), state)

    session.resume("1.2")

    assert state.completed_at is None
    assert state.current_level == "1.2"
    assert "show_game_complete" not in display.names()


def test_resume_starts_the_clock_for_an_unvisited_level(tmp_path: Path) -> None:
    state = _make_state(tmp_path, current_level="1.1")
    session, _, _ = _make_session(tmp_path, _levels(), state)

    session.resume("2.1")

    assert "2.1" in state.level_started_at


def test_resume_keeps_the_original_start_time_when_revisiting(tmp_path: Path) -> None:
    state = _make_state(tmp_path, current_level="2.1")
    first_seen = datetime(2020, 1, 1, 12, 0, 0)
    state.level_started_at["1.2"] = first_seen
    session, _, _ = _make_session(tmp_path, _levels(), state)

    session.resume("1.2")

    assert state.level_started_at["1.2"] == first_seen


@pytest.mark.parametrize("level_id", ["9.9", "nope", "1", "1.x", ""])
def test_resume_rejects_unusable_level_ids_without_touching_state(tmp_path: Path, level_id: str) -> None:
    state = _make_state(tmp_path, current_level="1.1")
    session, _, state_manager = _make_session(tmp_path, _levels(), state)

    session.resume(level_id)

    assert state.current_level == "1.1"
    assert state_manager.saves == 0


def test_resume_without_a_save_reports_not_initialized(tmp_path: Path) -> None:
    session, display, state_manager = _make_session(tmp_path, _levels(), None)

    session.resume("1.2")

    assert "show_not_initialized" in display.names()
    assert state_manager.saves == 0
