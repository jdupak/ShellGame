"""State management for game persistence."""

from pathlib import Path
from datetime import datetime
from typing import Dict, Optional
from pydantic import BaseModel, Field, ConfigDict, field_serializer
import json
import tempfile


class LevelCompletion(BaseModel):
    """Record of a completed level."""

    time_sec: int
    hints: int
    attempts: int
    completed_at: datetime


class GameState(BaseModel):
    """Game state model with all player progress."""

    version: str = "1.0"
    username: str
    workspace: Path
    current_level: str
    start_time: datetime

    # Completion stats (what the player sees in `status`)
    levels_complete: Dict[str, LevelCompletion] = Field(default_factory=dict)

    # Tracking (used to compute completion stats)
    level_attempts: Dict[str, int] = Field(default_factory=dict)
    level_hints_used: Dict[str, int] = Field(default_factory=dict)
    level_started_at: Dict[str, datetime] = Field(default_factory=dict)

    # Pydantic v2 configuration (replaces deprecated class-based Config)
    model_config = ConfigDict(
        # Path is a supported type, but keep this permissive to avoid surprises
        arbitrary_types_allowed=True,
    )

    @field_serializer("workspace")
    def _serialize_workspace(self, v: Path) -> str:
        return str(v)

    @field_serializer("start_time")
    def _serialize_start_time(self, v: datetime) -> str:
        return v.isoformat()

    @field_serializer("level_started_at")
    def _serialize_level_started_at(self, v: Dict[str, datetime]) -> Dict[str, str]:
        return {k: dt.isoformat() for k, dt in v.items()}

    def model_dump_json(self, **kwargs) -> str:  # type: ignore
        """Custom JSON serialization."""
        # Use JSON-mode dump so field serializers run.
        # Keep JSON structure stable and human-readable.
        data = self.model_dump(mode="json")
        return json.dumps(data, indent=2)


class StateManager:
    """Manages game state persistence."""

    def __init__(self) -> None:
        """Initialize state manager with default paths."""
        self.state_dir = Path.home() / ".config" / "shellgame"
        self.state_file = self.state_dir / "state.json"

    def load(self) -> Optional[GameState]:
        """
        Load state from disk.

        Returns:
            GameState if file exists, None otherwise
        """
        if not self.state_file.exists():
            return None

        try:
            with open(self.state_file, "r") as f:
                data = json.load(f)

            # Convert string paths and dates back to proper types
            data["workspace"] = Path(data["workspace"])
            data["start_time"] = datetime.fromisoformat(data["start_time"])

            # Convert completed_at strings back to datetime
            for level_data in data.get("levels_complete", {}).values():
                level_data["completed_at"] = datetime.fromisoformat(
                    level_data["completed_at"]
                )

            # Convert level_started_at strings back to datetime
            if "level_started_at" in data and isinstance(
                data["level_started_at"], dict
            ):
                data["level_started_at"] = {
                    k: datetime.fromisoformat(v)
                    for k, v in data["level_started_at"].items()
                }

            return GameState(**data)
        except (json.JSONDecodeError, KeyError, ValueError, TypeError) as e:
            # Corrupted state file
            print(f"Varování: Poškozený soubor stavu. Chyba: {e}")
            return None

    def save(self, state: GameState) -> None:
        """
        Atomically save state to disk.

        Args:
            state: GameState to persist
        """
        # Create state directory if it doesn't exist
        self.state_dir.mkdir(parents=True, exist_ok=True)

        # Atomic write: temp file + rename
        with tempfile.NamedTemporaryFile(
            mode="w", dir=self.state_dir, delete=False, suffix=".json"
        ) as tmp:
            tmp.write(state.model_dump_json())
            tmp_path = tmp.name

        # Atomic rename
        Path(tmp_path).rename(self.state_file)

    def init(self, username: str) -> GameState:
        """
        Initialize new game state.

        Args:
            username: Player username

        Returns:
            New GameState instance
        """
        workspace = Path(f"/tmp/shellgame-{username}")
        state = GameState(
            username=username,
            workspace=workspace,
            current_level="0.0",
            start_time=datetime.now(),
        )
        # Start current level timer immediately
        state.level_started_at[state.current_level] = datetime.now()
        self.save(state)
        return state

    def exists(self) -> bool:
        """Check if state file exists."""
        return self.state_file.exists()

    def delete(self) -> None:
        """Delete state file."""
        if self.state_file.exists():
            self.state_file.unlink()

    def remove(self) -> None:
        """Remove state file (alias for delete)."""
        self.delete()

    def record_attempt(self, state: GameState, *, level_id: str) -> None:
        """Increment attempt counter for a level."""
        state.level_attempts[level_id] = state.level_attempts.get(level_id, 0) + 1

    def record_hint_used(
        self, state: GameState, *, level_id: str, count: int = 1
    ) -> None:
        """Increment hint counter for a level."""
        if count <= 0:
            return
        state.level_hints_used[level_id] = (
            state.level_hints_used.get(level_id, 0) + count
        )

    def ensure_level_started(
        self, state: GameState, *, level_id: str, now: Optional[datetime] = None
    ) -> None:
        """Ensure a per-level start time exists (for time tracking)."""
        if level_id in state.level_started_at:
            return
        state.level_started_at[level_id] = now or datetime.now()

    def record_completion(
        self,
        state: GameState,
        *,
        level_id: str,
        completed_at: Optional[datetime] = None,
    ) -> LevelCompletion:
        """Record completion stats for a level.

        Computes:
        - time_sec: based on `level_started_at[level_id]` if present; otherwise 0
        - hints: from `level_hints_used[level_id]`
        - attempts: from `level_attempts[level_id]`
        """
        completed_at = completed_at or datetime.now()

        started_at = state.level_started_at.get(level_id)
        time_sec = 0
        if started_at is not None:
            delta = completed_at - started_at
            time_sec = max(0, int(delta.total_seconds()))

        completion = LevelCompletion(
            time_sec=time_sec,
            hints=state.level_hints_used.get(level_id, 0),
            attempts=state.level_attempts.get(level_id, 0),
            completed_at=completed_at,
        )
        state.levels_complete[level_id] = completion
        return completion
