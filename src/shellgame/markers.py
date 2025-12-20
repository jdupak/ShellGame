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
    """Manages marker files for shell-wrapper verification.

    Marker files allow the Python CLI to verify that users performed
    specific shell commands correctly (e.g., using absolute paths,
    step-by-step navigation, etc.).

    Example:
        markers = MarkerManager("alice")
        if markers.exists("pwd_used"):
            print("User ran pwd command")
        markers.create("level_complete")
    """

    # Well-known marker names used throughout the application
    PWD_USED = "pwd_used"
    LEVEL1_8_ABSOLUTE_CD = "level1_8_absolute_cd_used"
    LEVEL1_9_CD_WALK_PROGRESS = "level1_9_cd_walk_progress"
    LEVEL1_9_CD_WALK_COMPLETED = "level1_9_cd_walk_completed"

    def __init__(self, username: str):
        """Initialize marker manager.

        Args:
            username: Player's username (used to construct marker directory path)
        """
        self.username = username
        self.base_dir = Path(f"/tmp/shellgame-{username}")

    @classmethod
    def from_state(cls, state: "GameStateProtocol") -> "MarkerManager":
        """Create a MarkerManager from game state.

        Args:
            state: Game state object with username attribute

        Returns:
            MarkerManager instance
        """
        username = getattr(state, "username", None)
        if not username:
            raise ValueError("State must have a username attribute")
        return cls(username)

    def _marker_path(self, name: str) -> Path:
        """Get the full path for a marker file.

        Args:
            name: Marker name (without leading dot)

        Returns:
            Full path to marker file
        """
        # Normalize: ensure marker name starts with dot for hidden file
        marker_name = f".{name}" if not name.startswith(".") else name
        return self.base_dir / marker_name

    def exists(self, name: str) -> bool:
        """Check if a marker file exists.

        Args:
            name: Marker name

        Returns:
            True if marker exists
        """
        return self._marker_path(name).exists()

    def create(self, name: str, content: str = "") -> None:
        """Create a marker file.

        Args:
            name: Marker name
            content: Optional content to write to marker
        """
        self.base_dir.mkdir(parents=True, exist_ok=True)
        self._marker_path(name).write_text(content)

    def remove(self, name: str) -> None:
        """Remove a marker file if it exists.

        Args:
            name: Marker name
        """
        self._marker_path(name).unlink(missing_ok=True)

    def read(self, name: str) -> Optional[str]:
        """Read content from a marker file.

        Args:
            name: Marker name

        Returns:
            Content of marker file, or None if it doesn't exist
        """
        path = self._marker_path(name)
        if path.exists():
            return path.read_text()
        return None

    def clear_all(self) -> None:
        """Remove all marker files (but keep the directory)."""
        if self.base_dir.exists():
            for marker in self.base_dir.glob(".*"):
                if marker.is_file():
                    marker.unlink()


# Type stub for avoiding circular import
class GameStateProtocol:
    """Protocol stub - actual definition in protocols.py."""

    username: str
