"""Marker file management for shell wrapper communication.

Marker files are used to verify that the user performed specific shell actions
(like using `pwd`, `cd` with absolute path, etc.) that cannot be detected
directly from Python.

Markers live directly in the configured workspace.
"""

from pathlib import Path

from shellgame.protocols import GameStateProtocol


class MarkerManager:
    #: Markers shared across levels. A level's own `cd` evidence marker is
    #: derived from its ID instead (see `levels.cdpolicy.cd_marker`), so it
    #: cannot be misspelled or reused by another level.
    PWD_USED = "pwd_used"
    LEVEL1_9_CD_WALK_PROGRESS = "level1_9_cd_walk_progress"
    LEVEL1_9_CD_WALK_COMPLETED = "level1_9_cd_walk_completed"
    LEVEL9_4_DEV_NULL = "level9_4_dev_null_used"

    def __init__(self, workspace: Path):
        self.base_dir = workspace

    @classmethod
    def from_state(cls, state: GameStateProtocol) -> "MarkerManager":
        return cls(state.workspace)

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

    def read(self, name: str) -> str | None:
        path = self._marker_path(name)
        if path.exists():
            return path.read_text()
        return None
