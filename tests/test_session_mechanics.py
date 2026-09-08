"""Robustness tests for :class:`GameSession` game mechanics.

These cover the contracts a player relies on and that used to be silently
broken:

* the level's own ``success_message`` reaches the player (not a generic one)
* success statistics (time / hints / failed attempts) are reported
* advancing performs exactly one atomic save and never loses progress
* finishing the last level is a real terminal state
* bonus levels are skippable, core levels are not
* a broken level fixture cannot destroy the player's progress
"""

from __future__ import annotations

import shutil
from datetime import datetime, timedelta
from pathlib import Path

import pytest

from shellgame.core.session import GameSession
from shellgame.levels.base import Level
from shellgame.levels.registry import LevelRegistry
from shellgame.levels.sections.section5 import FindCriticalCodeInLogLevel
from shellgame.levels.sections.section6 import BackupImportantFileLevel
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


@pytest.mark.parametrize(
    ("level_type", "relative_file"),
    [
        (BackupImportantFileLevel, "copying/dulezite.txt"),
        (FindCriticalCodeInLogLevel, "logs/server.log"),
    ],
)
def test_reset_rescues_player_inside_directory_replacing_a_fixture_file(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, level_type: type[Level], relative_file: str
) -> None:
    level = level_type()
    state = _make_state(tmp_path, current_level=level.id)
    state.workspace.mkdir()
    state.level_hints_used[level.id] = 2
    level.prepare(state.workspace)
    damaged = level.section_path(state.workspace) / relative_file
    damaged.unlink()
    damaged.mkdir()
    monkeypatch.chdir(damaged)
    registry = LevelRegistry()
    registry.register(level)
    display = _FakeDisplay()
    state_manager = _FakeStateManager(state)
    shell = _FakeShell()
    session = GameSession(
        console=_FakeConsole(),
        display=display,  # type: ignore[arg-type]
        state_manager=state_manager,  # type: ignore[arg-type]
        level_registry=registry,
        teleport_notice=lambda _destination: None,
        shell_client=shell,  # type: ignore[arg-type]
        workspace_factory=lambda workspace: _FakeWorkspace(workspace),  # type: ignore[arg-type]
    )

    session.reset()

    assert damaged.is_file()
    assert Path.cwd() == level.get_start_directory(state.workspace)
    assert shell.exported["SHELLGAME_LEVEL"] == level.id
    assert shell.exported["SHELLGAME_WORKSPACE"] == str(state.workspace)
    assert state.level_hints_used[level.id] == 2
    assert state_manager.saves == 0
    assert "show_reset" in display.names()
    assert level.solution is not None
    level.solution.perform(level, state)
    session.submit(level.solution.answer)
    assert level.id in state.levels_complete


class TestSuccessFeedback:
    def test_level_success_message_reaches_the_player(self, tmp_path: Path) -> None:
        """A hand-written per-level message must not be replaced by a generic one."""
        levels = [
            _FakeLevel("1.1", result=(True, "Skvěle! Našli jste adresář delta.")),
            _FakeLevel("1.2"),
        ]
        state = _make_state(tmp_path)
        session, display, _ = _make_session(tmp_path, levels, state)

        session.submit("delta")

        args, _ = display.call("show_success")
        assert args[0] == "Skvěle! Našli jste adresář delta."

    def test_success_reports_time_hints_and_failed_attempts(self, tmp_path: Path) -> None:
        levels = [_FakeLevel("1.1"), _FakeLevel("1.2")]
        state = _make_state(tmp_path)
        state.level_started_at["1.1"] = datetime.now() - timedelta(seconds=42)
        state.level_hints_used["1.1"] = 2
        state.level_attempts["1.1"] = 3
        session, display, _ = _make_session(tmp_path, levels, state)

        session.submit("delta")

        _, kwargs = display.call("show_success")
        assert kwargs["hints_used"] == 2
        assert kwargs["attempts"] == 3
        assert kwargs["time_sec"] is not None
        assert kwargs["time_sec"] >= 42

    def test_failure_shows_the_validators_message(self, tmp_path: Path) -> None:
        levels = [_FakeLevel("1.1", result=(False, "To je soubor, ne adresář."))]
        state = _make_state(tmp_path)
        session, display, _ = _make_session(tmp_path, levels, state)

        session.submit("data.txt")

        args, _ = display.call("show_failure")
        assert args[0] == "To je soubor, ne adresář."
        assert "show_success" not in display.names()


class TestAdvancing:
    def test_advance_saves_state_exactly_once(self, tmp_path: Path) -> None:
        levels = [_FakeLevel("1.1"), _FakeLevel("1.2")]
        state = _make_state(tmp_path)
        session, _, state_manager = _make_session(tmp_path, levels, state)

        session.submit("x")

        assert state_manager.saves == 1
        assert state.current_level == "1.2"

    def test_progress_is_kept_when_the_next_level_fails_to_prepare(self, tmp_path: Path) -> None:
        """A broken fixture must never cost the player their completed level."""
        levels = [
            _FakeLevel("1.1"),
            _FakeLevel("1.2", prepare_error=OSError("disk on fire")),
        ]
        state = _make_state(tmp_path)
        session, display, state_manager = _make_session(tmp_path, levels, state)

        session.submit("x")

        assert state_manager.state is not None
        assert state_manager.state.current_level == "1.2"
        assert "1.1" in state.levels_complete
        assert "show_level_setup_error" in display.names()

    def test_intro_levels_do_not_show_a_success_panel(self, tmp_path: Path) -> None:
        intro = _FakeLevel("1.0")
        intro.is_intro = True
        levels = [intro, _FakeLevel("1.1")]
        state = _make_state(tmp_path, current_level="1.0")
        session, display, _ = _make_session(tmp_path, levels, state)

        session.submit(None)

        assert "show_success" not in display.names()


class TestTerminalState:
    def test_finishing_the_last_level_marks_the_game_complete(self, tmp_path: Path) -> None:
        levels = [_FakeLevel("11.9")]
        state = _make_state(tmp_path, current_level="11.9")
        session, display, _ = _make_session(tmp_path, levels, state)

        session.submit("x")

        assert state.completed_at is not None
        assert "show_game_complete" in display.names()

    @pytest.mark.parametrize("action", ["submit", "skip", "hint", "show_current_level"])
    def test_actions_after_completion_are_inert(self, tmp_path: Path, action: str) -> None:
        """A finished game must not re-run the last level or crash."""
        levels = [_FakeLevel("11.9")]
        state = _make_state(tmp_path, current_level="11.9")
        state.completed_at = datetime.now()
        session, display, state_manager = _make_session(tmp_path, levels, state)

        if action == "submit":
            session.submit("x")
        elif action == "skip":
            session.skip()
        elif action == "hint":
            session.hint(repeat=False)
        else:
            session.show_current_level()

        assert "show_game_complete" in display.names()
        assert state_manager.saves == 0


class TestSkip:
    def test_optional_level_can_be_skipped(self, tmp_path: Path) -> None:
        levels = [_FakeLevel("1.11", optional=True), _FakeLevel("2.0")]
        state = _make_state(tmp_path, current_level="1.11")
        session, display, _ = _make_session(tmp_path, levels, state)

        session.skip()

        assert state.current_level == "2.0"
        assert "show_skipped" in display.names()

    def test_extension_level_can_be_skipped(self, tmp_path: Path) -> None:
        levels = [_FakeLevel("1.8", extension=True), _FakeLevel("1.9")]
        state = _make_state(tmp_path, current_level="1.8")
        session, display, _ = _make_session(tmp_path, levels, state)

        session.skip()

        assert state.current_level == "1.9"
        assert "show_skipped" in display.names()

    def test_core_level_cannot_be_skipped(self, tmp_path: Path) -> None:
        """Skipping must never be a way around required material."""
        levels = [_FakeLevel("1.1"), _FakeLevel("1.2")]
        state = _make_state(tmp_path)
        session, display, state_manager = _make_session(tmp_path, levels, state)

        session.skip()

        assert state.current_level == "1.1"
        assert "show_not_skippable" in display.names()
        assert state_manager.saves == 0

    def test_skipping_does_not_count_as_a_completion(self, tmp_path: Path) -> None:
        levels = [_FakeLevel("1.11", optional=True), _FakeLevel("2.0")]
        state = _make_state(tmp_path, current_level="1.11")
        session, display, _ = _make_session(tmp_path, levels, state)

        session.skip()

        assert "1.11" not in state.levels_complete
        assert "show_success" not in display.names()


class TestNotInitialized:
    @pytest.mark.parametrize("action", ["submit", "skip", "reset", "hint"])
    def test_actions_without_state_are_safe(self, tmp_path: Path, action: str) -> None:
        session, display, _ = _make_session(tmp_path, [_FakeLevel("1.1")], None)

        if action == "submit":
            session.submit("x")
        elif action == "skip":
            session.skip()
        elif action == "reset":
            session.reset()
        else:
            session.hint(repeat=False)

        assert "show_not_initialized" in display.names()


class TestWorkspaceRestore:
    """A cleared `/tmp` must rebuild from every gameplay command, not just `shellgame`."""

    @pytest.mark.parametrize("action", ["submit", "skip", "reset", "hint"])
    def test_missing_workspace_is_restored(self, tmp_path: Path, action: str) -> None:
        levels = [_FakeLevel("1.1", optional=True), _FakeLevel("1.2")]
        state = _make_state(tmp_path)
        session, display, _ = _make_session(tmp_path, levels, state)

        shutil.rmtree(state.workspace)
        assert not state.workspace.exists()

        if action == "submit":
            session.submit("x")
        elif action == "skip":
            session.skip()
        elif action == "reset":
            session.reset()
        else:
            session.hint(repeat=False)

        assert state.workspace.is_dir()
        assert "show_workspace_restored" in display.names()

    def test_present_workspace_is_not_reported_as_restored(self, tmp_path: Path) -> None:
        levels = [_FakeLevel("1.1"), _FakeLevel("1.2")]
        state = _make_state(tmp_path)
        session, display, _ = _make_session(tmp_path, levels, state)

        session.submit("x")

        assert "show_workspace_restored" not in display.names()
