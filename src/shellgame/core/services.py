"""
Service container and factories for ShellGame.

This module centralizes the creation of core services and dependencies.
"""

from typing import Callable

from rich.console import Console

from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import LevelRegistry, get_registry
from shellgame.shell.client import ShellClient
from shellgame.state.manager import StateManager
from shellgame.ui.display import Display
from shellgame.workspace.builder import WorkspaceManager


class GameServices:
    def __init__(self) -> None:
        self.console = Console()
        self.state_manager = StateManager()
        self.level_registry = get_registry()
        self.display = Display(self.console)
        self.shell_client = ShellClient()

        initialize_levels()

    def get_workspace_manager_factory(self) -> Callable[[str], WorkspaceManager]:
        return WorkspaceManager

