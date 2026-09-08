"""A saved game must survive levels being renumbered between versions.

Level IDs are persisted in `state.json`. Before this, a saved `current_level`
that no longer existed made every gameplay command print "level not found",
leaving `shellgame remove` — total progress loss — as the only way out.
"""

from __future__ import annotations

from datetime import datetime
from pathlib import Path

import pytest

from shellgame.core.session import GameSession
from shellgame.levels.base import Level
from shellgame.levels.completion import Completion
from shellgame.levels.registry import LevelRegistry, UnknownLevelError
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


def _level(level_id: str) -> Level:
    class _Registered(Level):
        id = level_id
        title = f"Level {level_id}"
        completion = Completion()

    return _Registered()


def _registry(*ids: str) -> LevelRegistry:
    registry = LevelRegistry()
    for level_id in ids:
        registry.register(_level(level_id))
    return registry


class TestNextLevel:
    def test_last_level_returns_none(self) -> None:
        assert _registry("1.1", "1.2").next_level("1.2") is None

    def test_unknown_level_raises_instead_of_looking_finished(self) -> None:
        """Returning None here would silently declare the game complete."""
        with pytest.raises(UnknownLevelError):
            _registry("1.1", "1.2").next_level("9.9")

    def test_known_level_advances(self) -> None:
        assert _registry("1.1", "1.2").next_level("1.1") == "1.2"


class TestResolveOrNearest:
    def test_existing_id_is_returned_unchanged(self) -> None:
        assert _registry("1.1", "2.1").resolve_or_nearest("2.1") == "2.1"

    def test_removed_level_resolves_forward_within_the_section(self) -> None:
        assert _registry("1.1", "1.5", "2.1").resolve_or_nearest("1.3") == "1.5"

    def test_removed_section_resolves_to_the_next_section(self) -> None:
        assert _registry("1.1", "3.1").resolve_or_nearest("2.4") == "3.1"

    def test_id_past_the_end_resolves_to_the_last_level(self) -> None:
        assert _registry("1.1", "1.2").resolve_or_nearest("9.9") == "1.2"

    def test_malformed_id_resolves_to_the_first_level(self) -> None:
        assert _registry("1.1", "1.2").resolve_or_nearest("garbage") == "1.1"

    def test_empty_registry_resolves_to_nothing(self) -> None:
        assert LevelRegistry().resolve_or_nearest("1.1") is None


class TestSessionRecovery:
    def _session(self, tmp_path: Path, current_level: str) -> tuple[GameSession, _FakeDisplay, GameState]:
        workspace = tmp_path / "ws"
        workspace.mkdir(parents=True, exist_ok=True)
        state = GameState(
            username="student",
            current_level=current_level,
            workspace=workspace,
            start_time=datetime.now(),
        )
        display = _FakeDisplay()
        registry = _FakeRegistry([_FakeLevel("1.1"), _FakeLevel("1.2")], nearest="1.1")

        session = GameSession(
            console=_FakeConsole(),
            display=display,  # type: ignore[arg-type]
            state_manager=_FakeStateManager(state),  # type: ignore[arg-type]
            level_registry=registry,  # type: ignore[arg-type]
            teleport_notice=lambda _destination: None,
            shell_client=_FakeShell(),  # type: ignore[arg-type]
            workspace_factory=lambda _workspace: _FakeWorkspace(workspace),  # type: ignore[arg-type]
        )
        return session, display, state

    @pytest.mark.parametrize("action", ["submit", "skip", "reset", "hint"])
    def test_stale_level_resyncs_instead_of_bricking(self, tmp_path: Path, action: str) -> None:
        session, display, state = self._session(tmp_path, "7.3")

        if action == "submit":
            session.submit("x")
        elif action == "skip":
            session.skip()
        elif action == "reset":
            session.reset()
        else:
            session.hint(repeat=False)

        assert "show_level_missing" in display.names()
        assert state.current_level == "1.1"

    def test_resync_does_not_apply_the_stale_command_to_the_new_level(self, tmp_path: Path) -> None:
        """An answer typed for level 7.3 must not be graded against level 1.1."""
        session, display, state = self._session(tmp_path, "7.3")

        session.submit("an answer for a level that no longer exists")

        assert "show_success" not in display.names()
        assert state.current_level == "1.1"
        assert "1.1" not in state.levels_complete

    def test_resync_preserves_completed_levels(self, tmp_path: Path) -> None:
        session, _, state = self._session(tmp_path, "7.3")
        state.levels_complete["1.1"] = LevelCompletion(time_sec=10, hints=0, attempts=1, completed_at=datetime.now())

        session.submit("x")

        assert "1.1" in state.levels_complete

    def test_resync_never_declares_the_game_complete(self, tmp_path: Path) -> None:
        session, display, state = self._session(tmp_path, "7.3")

        session.submit("x")

        assert state.completed_at is None
        assert "show_game_complete" not in display.names()

    def test_valid_level_is_untouched(self, tmp_path: Path) -> None:
        session, display, state = self._session(tmp_path, "1.1")

        session.submit("x")

        assert "show_level_missing" not in display.names()
        assert state.current_level == "1.2"
