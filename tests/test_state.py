import json
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.state.manager import (
    CURRENT_STATE_VERSION,
    GameState,
    LevelCompletion,
    StateLoadError,
    StateManager,
    StateSaveError,
)


class TestGameState:
    def test_create_state(self) -> None:
        workspace = Path("/tmp/test-workspace")
        state = GameState(
            username="testuser",
            workspace=workspace,
            current_level="1.1",
            start_time=datetime.now(),
        )

        assert state.username == "testuser"
        assert state.workspace == workspace
        assert state.current_level == "1.1"
        assert isinstance(state.start_time, datetime)

        # Tracking defaults
        assert state.level_attempts == {}
        assert state.level_hints_used == {}
        assert state.level_started_at == {}
        assert state.levels_complete == {}

    def test_level_completion(self) -> None:
        state = GameState(
            username="testuser",
            workspace=Path("/tmp/test"),
            current_level="1.1",
            start_time=datetime.now(),
        )

        completion = LevelCompletion(time_sec=45, hints=1, attempts=2, completed_at=datetime.now())

        state.levels_complete["1.1"] = completion
        assert "1.1" in state.levels_complete
        assert state.levels_complete["1.1"].time_sec == 45


class TestStateManager:
    def test_init_state(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"

        state = manager.init("testuser")

        assert state.username == "testuser"
        assert state.current_level == "0.0"
        assert manager.state_file.exists()

        # Current level timer starts immediately
        assert state.current_level in state.level_started_at

    def test_save_and_load(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"

        # Create and save state
        state = manager.init("testuser")
        state.current_level = "2.3"
        manager.save(state)

        # Load state
        loaded = manager.load()

        assert loaded is not None
        assert loaded.username == "testuser"
        assert loaded.current_level == "2.3"

    def test_save_removes_temp_file_when_serialization_fails(
        self, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
    ) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"
        state = manager.create("testuser")

        def fail_serialization(self: GameState, *, indent: int) -> str:
            raise OSError("disk full")

        monkeypatch.setattr(GameState, "model_dump_json", fail_serialization)

        with pytest.raises(StateSaveError, match="disk full"):
            manager.save(state)

        assert list(tmp_path.iterdir()) == []

    def test_load_nonexistent(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "nonexistent.json"

        state = manager.load()
        assert state is None

    def test_delete_state(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"

        manager.init("testuser")
        assert manager.state_file.exists()

        manager.remove()
        assert not manager.state_file.exists()

    def test_exists(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"

        assert not manager.exists()

        manager.init("testuser")
        assert manager.exists()

    def test_save_and_load_persists_tracking_fields(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"

        state = manager.init("testuser")
        state.current_level = "1.1"
        state.level_attempts["1.1"] = 3
        state.level_hints_used["1.1"] = 2
        state.level_started_at["1.1"] = datetime.now()

        manager.save(state)
        loaded = manager.load()

        assert loaded is not None
        assert loaded.level_attempts["1.1"] == 3
        assert loaded.level_hints_used["1.1"] == 2
        assert isinstance(loaded.level_started_at["1.1"], datetime)

    def test_load_migrates_version_1_state(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"
        manager.state_file.write_text(
            json.dumps(
                {
                    "version": "1.0",
                    "username": "testuser",
                    "workspace": "/tmp/test",
                    "current_level": "1.1",
                    "start_time": datetime.now().isoformat(),
                }
            )
        )

        loaded = manager.load()

        assert loaded is not None
        assert loaded.version == CURRENT_STATE_VERSION

    def test_corrupt_state_raises_without_overwriting_file(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"
        manager.state_file.write_text("{not-json")

        with pytest.raises(StateLoadError):
            manager.load()

        assert manager.state_file.read_text() == "{not-json"

    def test_unsupported_state_version_raises(self, tmp_path: Path) -> None:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"
        manager.state_file.write_text(
            json.dumps(
                {
                    "version": "99.0",
                    "username": "testuser",
                    "workspace": "/tmp/test",
                    "current_level": "1.1",
                    "start_time": datetime.now().isoformat(),
                }
            )
        )

        with pytest.raises(StateLoadError, match="Nepodporovaná verze"):
            manager.load()
