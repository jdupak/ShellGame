"""Type protocols for ShellGame.

Defines Protocol classes to enable proper type checking without
creating circular dependencies.
"""

from pathlib import Path
from typing import Optional, Protocol, runtime_checkable


@runtime_checkable
class GameStateProtocol(Protocol):
    """Protocol for game state objects.

    This protocol defines the interface that validators and other
    components expect from a game state object.
    """

    @property
    def workspace(self) -> Path:
        """Path to the game workspace directory."""
        ...

    @property
    def username(self) -> str:
        """Player's username."""
        ...

    @property
    def current_level(self) -> str:
        """Current level ID (e.g., '1.1')."""
        ...


@runtime_checkable
class LevelProtocol(Protocol):
    """Protocol for level objects.

    Defines the minimal interface expected from a Level.
    """

    @property
    def id(self) -> str:
        """Level identifier (e.g., '1.1')."""
        ...

    @property
    def section(self) -> int:
        """Section number."""
        ...

    @property
    def title(self) -> str:
        """Level title."""
        ...

    @property
    def instructions(self) -> str:
        """Level instructions text."""
        ...

    @property
    def start_directory(self) -> Optional[str]:
        """Starting directory relative to workspace, or None."""
        ...
