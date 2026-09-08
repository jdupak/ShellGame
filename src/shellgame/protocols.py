from pathlib import Path
from typing import Protocol, runtime_checkable

ValidationResult = tuple[bool, str]


@runtime_checkable
class GameStateProtocol(Protocol):
    @property
    def workspace(self) -> Path: ...

    @property
    def username(self) -> str: ...

    @property
    def current_level(self) -> str: ...


class CdHookCallback(Protocol):
    def __call__(
        self,
        *,
        target: str | None,
        pwd: str | None,
        post_move: bool,
        state: GameStateProtocol,
    ) -> None: ...
