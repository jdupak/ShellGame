"""Unit tests for progress tracking."""

from datetime import datetime, timedelta
from pathlib import Path

from shellgame.core.progress import ProgressTracker
from shellgame.state.manager import StateManager


class TestProgressTracker:
    """Test ProgressTracker."""

    def test_record_attempt_increments(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"
        tracker = ProgressTracker()

        state = manager.init("testuser")
        tracker.record_attempt(state, level_id="1.1")
        tracker.record_attempt(state, level_id="1.1")
        tracker.record_attempt(state, level_id="2.0")

        assert state.level_attempts["1.1"] == 2
        assert state.level_attempts["2.0"] == 1

    def test_ensure_level_started_sets_once(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"
        tracker = ProgressTracker()

        state = manager.init("testuser")

        now = datetime.now()
        tracker.ensure_level_started(state, level_id="1.1", now=now)
        assert state.level_started_at["1.1"] == now

        later = now + timedelta(seconds=10)
        tracker.ensure_level_started(state, level_id="1.1", now=later)
        assert state.level_started_at["1.1"] == now

    def test_record_completion_uses_tracking(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"
        tracker = ProgressTracker()

        state = manager.init("testuser")

        started = datetime.now() - timedelta(seconds=12)
        state.level_started_at["1.1"] = started
        state.level_attempts["1.1"] = 2
        state.level_hints_used["1.1"] = 1

        completed_at = datetime.now()
        completion = tracker.record_completion(state, level_id="1.1", completed_at=completed_at)

        assert state.levels_complete["1.1"] == completion
        assert completion.attempts == 2
        assert completion.hints == 1
        assert completion.completed_at == completed_at
        assert completion.time_sec >= 12
