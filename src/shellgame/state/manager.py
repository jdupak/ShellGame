"""State management for game persistence."""

import json
import os
import tempfile
from datetime import datetime
from pathlib import Path
from typing import Optional

from pydantic import BaseModel, ConfigDict, Field, field_serializer


class LevelCompletion(BaseModel):
    time_sec: int
    hints: int
    attempts: int
    completed_at: datetime


class GameState(BaseModel):
    version: str = "1.0"
    username: str
    workspace: Path
    current_level: str
    start_time: datetime

    levels_complete: dict[str, LevelCompletion] = Field(default_factory=dict)

    level_attempts: dict[str, int] = Field(default_factory=dict)
    level_hints_used: dict[str, int] = Field(default_factory=dict)
    level_started_at: dict[str, datetime] = Field(default_factory=dict)

    model_config = ConfigDict(
        arbitrary_types_allowed=True,
    )

    @field_serializer("workspace")
    def _serialize_workspace(self, v: Path) -> str:
        return str(v)

    @field_serializer("start_time")
    def _serialize_start_time(self, v: datetime) -> str:
        return v.isoformat()

    @field_serializer("level_started_at")
    def _serialize_level_started_at(self, v: dict[str, datetime]) -> dict[str, str]:
        return {k: dt.isoformat() for k, dt in v.items()}

    def model_dump_json(self, **kwargs) -> str:  # type: ignore
        data = self.model_dump(mode="json")
        return json.dumps(data, indent=2)


class StateManager:
    def __init__(self) -> None:
        xdg_config = os.environ.get("XDG_CONFIG_HOME")
        if xdg_config:
            self.state_dir = Path(xdg_config) / "shellgame"
        else:
            self.state_dir = Path.home() / ".config" / "shellgame"
        self.state_file = self.state_dir / "state.json"

    def load(self) -> Optional[GameState]:
        if not self.state_file.exists():
            return None

        try:
            with open(self.state_file) as f:
                data = json.load(f)

            data["workspace"] = Path(data["workspace"])
            data["start_time"] = datetime.fromisoformat(data["start_time"])

            for level_data in data.get("levels_complete", {}).values():
                level_data["completed_at"] = datetime.fromisoformat(level_data["completed_at"])

            if "level_started_at" in data and isinstance(data["level_started_at"], dict):
                data["level_started_at"] = {k: datetime.fromisoformat(v) for k, v in data["level_started_at"].items()}

            return GameState(**data)
        except (json.JSONDecodeError, KeyError, ValueError, TypeError) as e:
            print(f"Varování: Poškozený soubor stavu. Chyba: {e}")
            return None

    def save(self, state: GameState) -> None:
        self.state_dir.mkdir(parents=True, exist_ok=True)

        with tempfile.NamedTemporaryFile(mode="w", dir=self.state_dir, delete=False, suffix=".json") as tmp:
            tmp.write(state.model_dump_json())
            tmp_path = tmp.name

        Path(tmp_path).rename(self.state_file)

    def init(self, username: str, workspace_path: Optional[Path] = None) -> GameState:
        workspace = workspace_path if workspace_path else Path(f"/tmp/shellgame-{username}")
        state = GameState(
            username=username,
            workspace=workspace,
            current_level="0.0",
            start_time=datetime.now(),
        )
        state.level_started_at[state.current_level] = datetime.now()
        self.save(state)
        return state

    def exists(self) -> bool:
        return self.state_file.exists()

    def delete(self) -> None:
        if self.state_file.exists():
            self.state_file.unlink()

    def remove(self) -> None:
        self.delete()
