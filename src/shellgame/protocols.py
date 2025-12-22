from pathlib import Path
from typing import Optional, Protocol, runtime_checkable


@runtime_checkable
class GameStateProtocol(Protocol):
    @property
    def workspace(self) -> Path:
        ...

    @property
    def username(self) -> str:
        ...

    @property
    def current_level(self) -> str:
        ...


@runtime_checkable
class LevelProtocol(Protocol):
    @property
    def id(self) -> str:
        ...

    @property
    def section(self) -> int:
        ...

    @property
    def title(self) -> str:
        ...

    @property
    def instructions(self) -> str:
        ...

    @property
    def start_directory(self) -> Optional[str]:
        ...
