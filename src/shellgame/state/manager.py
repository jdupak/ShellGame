"""State management for game persistence."""

import json
import os
import tempfile
from collections.abc import Callable
from datetime import datetime
from pathlib import Path

from pydantic import BaseModel, Field, ValidationError

CURRENT_STATE_VERSION = "2.1"

#: Section numbers changed in state 2.1: wildcards moved ahead of permissions and
#: redirection, so globbing is taught before the earlier sections rely on it.
_SECTION_ROTATION_2_1 = {7: 8, 8: 9, 9: 10, 10: 7}

#: Levels swapped inside their section in 2.1 so every section ends on its
#: challenge: the help lesson moved ahead of 2's challenge, the alias aside ahead
#: of 6's. Both directions are listed, which makes the mapping its own inverse.
_LEVEL_SWAPS_2_1 = {"2.7": "2.8", "2.8": "2.7", "6.7": "6.8", "6.8": "6.7"}

#: Keys in a saved state whose values are keyed by level ID.
_LEVEL_KEYED_FIELDS = ("levels_complete", "level_attempts", "level_hints_used", "level_started_at")


class StatePersistenceError(RuntimeError):
    """Base error for state loading and saving failures."""


class StateLoadError(StatePersistenceError):
    """Raised when an existing state file cannot be loaded safely."""


class StateSaveError(StatePersistenceError):
    """Raised when state cannot be written atomically."""


class LevelCompletion(BaseModel):
    time_sec: int
    hints: int
    attempts: int
    completed_at: datetime


class GameState(BaseModel):
    version: str = CURRENT_STATE_VERSION
    username: str
    workspace: Path
    current_level: str
    start_time: datetime

    levels_complete: dict[str, LevelCompletion] = Field(default_factory=dict)

    level_attempts: dict[str, int] = Field(default_factory=dict)
    level_hints_used: dict[str, int] = Field(default_factory=dict)
    level_started_at: dict[str, datetime] = Field(default_factory=dict)

    completed_at: datetime | None = None


class StateManager:
    def __init__(self) -> None:
        state_dir_env = os.environ.get("SHELLGAME_STATE_DIR")
        if state_dir_env:
            self.state_dir = Path(state_dir_env)
        else:
            xdg_config = os.environ.get("XDG_CONFIG_HOME")
            if xdg_config:
                self.state_dir = Path(xdg_config) / "shellgame"
            else:
                self.state_dir = Path.home() / ".config" / "shellgame"
        self.state_file = self.state_dir / "state.json"

    def load(self) -> GameState | None:
        if not self.state_file.exists():
            return None

        try:
            with self.state_file.open(encoding="utf-8") as f:
                data = json.load(f)
            return GameState.model_validate(self._migrate(data))
        except (OSError, json.JSONDecodeError, ValidationError, TypeError, ValueError) as exc:
            raise StateLoadError(f"Stav ShellGame nelze načíst z {self.state_file}: {exc}") from exc

    def save(self, state: GameState) -> None:
        self.state_dir.mkdir(parents=True, exist_ok=True)
        tmp_path: Path | None = None

        try:
            with tempfile.NamedTemporaryFile(
                mode="w",
                encoding="utf-8",
                dir=self.state_dir,
                delete=False,
                suffix=".json",
            ) as tmp:
                tmp_path = Path(tmp.name)
                tmp.write(state.model_dump_json(indent=2))
                tmp.flush()
                os.fsync(tmp.fileno())

            os.replace(tmp_path, self.state_file)
        except OSError as exc:
            raise StateSaveError(f"Stav ShellGame nelze uložit do {self.state_file}: {exc}") from exc
        finally:
            if tmp_path is not None:
                tmp_path.unlink(missing_ok=True)

    def create(self, username: str, workspace_path: Path | None = None) -> GameState:
        workspace = workspace_path if workspace_path else self.default_workspace(username)
        state = GameState(
            username=username,
            workspace=workspace,
            current_level="0.0",
            start_time=datetime.now(),
        )
        state.level_started_at[state.current_level] = datetime.now()
        return state

    @staticmethod
    def default_workspace(username: str) -> Path:
        """The only place that derives a workspace path from a username."""
        env_ws = os.environ.get("SHELLGAME_WORKSPACE")
        if env_ws:
            return Path(env_ws)
        return Path(f"/tmp/shellgame-{username}")

    def init(self, username: str, workspace_path: Path | None = None) -> GameState:
        state = self.create(username, workspace_path)
        self.save(state)
        return state

    def exists(self) -> bool:
        return self.state_file.exists()

    def remove(self) -> None:
        self.state_file.unlink(missing_ok=True)

    def _migrate(self, data: object) -> dict[str, object]:
        if not isinstance(data, dict):
            raise ValueError("Kořen souboru stavu musí být objekt.")

        migrated = dict(data)
        version = str(migrated.get("version", "1.0"))

        if version in {"1.0", "2.0"}:
            # Both predate the section reordering, so their level IDs point at
            # whatever now happens to carry that number. Remap them explicitly:
            # without this the registry's nearest-match fallback would silently
            # resume a player on unrelated content and keep their completed
            # levels credited to the wrong section.
            migrated = _remap_level_ids(migrated, _migrate_level_id_to_2_1)
            migrated["version"] = CURRENT_STATE_VERSION
        elif version != CURRENT_STATE_VERSION:
            raise ValueError(f"Nepodporovaná verze stavu: {version}")

        return migrated


def _migrate_level_id_to_2_1(level_id: str) -> str:
    """Map a pre-reorder level ID onto the level that now holds the same content."""
    if level_id in _LEVEL_SWAPS_2_1:
        return _LEVEL_SWAPS_2_1[level_id]

    section, separator, number = level_id.partition(".")
    if not separator or not section.isdigit():
        return level_id

    rotated = _SECTION_ROTATION_2_1.get(int(section))
    return f"{rotated}.{number}" if rotated is not None else level_id


def _remap_level_ids(data: dict[str, object], remap: Callable[[str], str]) -> dict[str, object]:
    migrated = dict(data)

    current = migrated.get("current_level")
    if isinstance(current, str):
        migrated["current_level"] = remap(current)

    for field in _LEVEL_KEYED_FIELDS:
        value = migrated.get(field)
        if isinstance(value, dict):
            migrated[field] = {remap(str(key)): item for key, item in value.items()}

    return migrated
