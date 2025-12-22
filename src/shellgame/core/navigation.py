"""Navigation manager for player movement and teleportation."""

from pathlib import Path
from typing import Callable, Optional, Set

from shellgame.levels.registry import LevelRegistry
from shellgame.shell.client import ShellClient
from shellgame.state.manager import GameState


class NavigationManager:
    """Manages player navigation and teleportation."""

    def __init__(
        self,
        level_registry: LevelRegistry,
        shell_client: ShellClient,
        teleport_notice: Callable[[Path], None],
    ):
        """Initialize navigation manager.

        Args:
            level_registry: Registry to look up level start directories
            shell_client: Client to emit cd commands
            teleport_notice: Callback to notify user of teleportation
        """
        self._level_registry = level_registry
        self._shell = shell_client
        self._teleport_notice = teleport_notice
        # Levels that should NOT auto-teleport
        self._no_autocd_levels: Set[str] = {"1.5"}

    def get_level_start_directory(self, level_id: str, workspace: Path) -> Optional[Path]:
        """Resolve start dir via the level definition."""
        level = self._level_registry.get(level_id)
        if level:
            return level.get_start_directory(workspace)
        return None

    def maybe_teleport(self, start_dir: Optional[Path]) -> None:
        """Teleport to a start directory if requested and needed."""
        if not start_dir:
            return

        # Resolve paths to handle symlinks (e.g. /tmp vs /private/tmp on macOS)
        # This prevents unnecessary teleport notices when the user is effectively
        # in the correct directory but the path string differs.
        current_dir = Path.cwd().resolve()
        target_dir = start_dir.resolve()

        # Only teleport if we are not already there
        if current_dir != target_dir:
            self._shell.cd(start_dir)
            self._teleport_notice(start_dir)

    def ensure_user_in_reasonable_place(self, state: GameState) -> None:
        """Ensure player is in the configured level start directory when appropriate."""
        if state.current_level in self._no_autocd_levels:
            return

        start_dir = self.get_level_start_directory(state.current_level, state.workspace)
        if not start_dir:
            return

        self.maybe_teleport(start_dir)
