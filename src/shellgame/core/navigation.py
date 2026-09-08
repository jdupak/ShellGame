"""Navigation manager for player movement and teleportation."""

import os
from collections.abc import Callable
from pathlib import Path

from shellgame.levels.registry import LevelRegistry
from shellgame.paths import current_directory
from shellgame.shell.client import ShellClient
from shellgame.state.manager import GameState


class NavigationManager:
    def __init__(
        self,
        level_registry: LevelRegistry,
        shell_client: ShellClient,
        teleport_notice: Callable[[Path], None],
    ) -> None:
        self._level_registry = level_registry
        self._shell = shell_client
        self._teleport_notice = teleport_notice

    def get_level_start_directory(self, level_id: str, workspace: Path) -> Path | None:
        level = self._level_registry.get(level_id)
        if level:
            return level.get_start_directory(workspace)
        return None

    def maybe_teleport(self, start_dir: Path | None) -> None:
        if not start_dir:
            return

        # A missing cwd (the player deleted it) always needs teleporting.
        current_dir = current_directory()
        target_dir = start_dir.resolve()

        if current_dir != target_dir:
            try:
                os.chdir(target_dir)
            except OSError:
                self._shell.cd(start_dir)
                return
            self._shell.cd(start_dir)
            self._teleport_notice(start_dir)

    def ensure_user_in_reasonable_place(self, state: GameState) -> None:
        level = self._level_registry.get(state.current_level)
        if level is None or not level.enforce_start_directory:
            return

        start_dir = level.get_start_directory(state.workspace)
        if not start_dir:
            return

        self.maybe_teleport(start_dir)
