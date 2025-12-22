"""Centralized marker file management for shell wrapper communication.

Marker files are used to verify that the user performed specific shell actions
(like using `pwd`, `cd` with absolute path, etc.) that cannot be detected
directly from Python.

The markers live in /tmp/shellgame-{username}/ and are created by shell
wrappers defined in commands.py.
"""

from pathlib import Path
from typing import Optional


class MarkerManager:
    PWD_USED = "pwd_used"
    LEVEL1_8_ABSOLUTE_CD = "level1_8_absolute_cd_used"
    LEVEL1_9_CD_WALK_PROGRESS = "level1_9_cd_walk_progress"
    LEVEL1_9_CD_WALK_COMPLETED = "level1_9_cd_walk_completed"

    def __init__(self, username: str):
        self.username = username
        self.base_dir = Path(f"/tmp/shellgame-{username}")

    @classmethod
    def from_state(cls, state: "GameStateProtocol") -> "MarkerManager":
        username = getattr(state, "username", None)
        if not username:
            raise ValueError("State must have a username attribute")
        return cls(username)

    def _marker_path(self, name: str) -> Path:
        marker_name = f".{name}" if not name.startswith(".") else name
        return self.base_dir / marker_name

    def exists(self, name: str) -> bool:
        return self._marker_path(name).exists()

    def create(self, name: str, content: str = "") -> None:
        self.base_dir.mkdir(parents=True, exist_ok=True)
        self._marker_path(name).write_text(content)

    def remove(self, name: str) -> None:
        self._marker_path(name).unlink(missing_ok=True)

    def read(self, name: str) -> Optional[str]:
        path = self._marker_path(name)
        if path.exists():
            return path.read_text()
        return None

    def clear_all(self) -> None:
        if self.base_dir.exists():
            for marker in self.base_dir.glob(".*"):
                if marker.is_file():
                    marker.unlink()


class GameStateProtocol:
    username: str
