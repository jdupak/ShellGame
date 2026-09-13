"""State management for game persistence."""

import json
import os
import tempfile
from datetime import datetime
from pathlib import Path

from pydantic import BaseModel, Field, ValidationError

CURRENT_STATE_VERSION = "2.0"


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

        if version == "1.0":
            migrated["version"] = CURRENT_STATE_VERSION
        elif version != CURRENT_STATE_VERSION:
            raise ValueError(f"Nepodporovaná verze stavu: {version}")

        return migrated
