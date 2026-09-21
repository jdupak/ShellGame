import json
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import get_registry
from shellgame.state.manager import (
    _LEVEL_SWAPS_2_1,
    _SECTION_ROTATION_2_1,
    CURRENT_STATE_VERSION,
    GameState,
    LevelCompletion,
    StateLoadError,
    StateManager,
    StateSaveError,
    _migrate_level_id_to_2_1,
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


class TestSectionReorderMigration:
    """State 2.1 renumbered sections; a pre-2.1 save must keep its real progress.

    Without an explicit remap the registry's nearest-match fallback would resume
    the player on whatever level now carries their saved number - different
    content - and credit their finished levels to the wrong section.
    """

    def _load(self, tmp_path: Path, payload: dict) -> GameState:
        manager = StateManager()
        manager.state_dir = tmp_path
        manager.state_file = tmp_path / "state.json"
        manager.state_file.write_text(json.dumps(payload))
        loaded = manager.load()
        assert loaded is not None
        return loaded

    def _payload(self, **overrides: object) -> dict:
        payload: dict = {
            "version": "2.0",
            "username": "testuser",
            "workspace": "/tmp/test",
            "current_level": "1.1",
            "start_time": datetime.now().isoformat(),
        }
        payload.update(overrides)
        return payload

    @pytest.mark.parametrize(
        ("saved", "expected"),
        [
            ("1.1", "1.1"),  # untouched sections keep their IDs
            ("5.4", "5.4"),
            ("11.6", "11.6"),
            ("7.2", "8.2"),  # permissions moved back one slot
            ("8.5", "9.5"),  # redirection moved back one slot
            ("9.4", "10.4"),  # error streams moved back one slot
            ("10.3", "7.3"),  # wildcards moved forward
            ("2.7", "2.8"),  # help lesson and challenge swapped
            ("2.8", "2.7"),
            ("6.7", "6.8"),  # alias aside and challenge swapped
            ("6.8", "6.7"),
        ],
    )
    def test_current_level_follows_its_content(self, tmp_path: Path, saved: str, expected: str) -> None:
        loaded = self._load(tmp_path, self._payload(current_level=saved))

        assert loaded.current_level == expected
        assert loaded.version == CURRENT_STATE_VERSION

    def test_progress_keyed_by_level_id_is_remapped_too(self, tmp_path: Path) -> None:
        completion = {"time_sec": 12, "hints": 1, "attempts": 0, "completed_at": datetime.now().isoformat()}
        loaded = self._load(
            tmp_path,
            self._payload(
                current_level="10.1",
                levels_complete={"7.1": completion},
                level_attempts={"8.5": 3},
                level_hints_used={"9.4": 2},
                level_started_at={"10.3": datetime.now().isoformat()},
            ),
        )

        assert loaded.current_level == "7.1"
        assert set(loaded.levels_complete) == {"8.1"}
        assert loaded.level_attempts == {"9.5": 3}
        assert loaded.level_hints_used == {"10.4": 2}
        assert set(loaded.level_started_at) == {"7.3"}

    def test_every_migrated_id_resolves_to_a_registered_level(self) -> None:
        """Round-trip: rebuild each level's pre-2.1 ID, then migrate it forward."""
        initialize_levels()
        registry = get_registry()
        inverse_rotation = {new: old for old, new in _SECTION_ROTATION_2_1.items()}

        def previous_id(level_id: str) -> str:
            section, _, number = level_id.partition(".")
            old = f"{inverse_rotation.get(int(section), int(section))}.{number}"
            # The swaps are their own inverse, so reapplying them rebuilds the old ID.
            return _LEVEL_SWAPS_2_1.get(old, old)

        old_ids = [previous_id(str(level.id)) for level in registry.list_levels()]
        migrated = [_migrate_level_id_to_2_1(old_id) for old_id in old_ids]

        assert migrated == [str(level.id) for level in registry.list_levels()]
        assert len(set(migrated)) == len(old_ids), "migration must not collapse two levels onto one"

    def test_already_migrated_state_is_left_alone(self, tmp_path: Path) -> None:
        loaded = self._load(tmp_path, self._payload(version=CURRENT_STATE_VERSION, current_level="7.2"))

        assert loaded.current_level == "7.2"
