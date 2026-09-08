"""Shared test doubles for :class:`GameSession` tests.

Kept in one place so that every session-level test exercises the same contract
surface as the real collaborators.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from shellgame.levels.registry import UnknownLevelError
from shellgame.state.manager import GameState


class _FakeDisplay:
    def __init__(self) -> None:
        self.calls: list[tuple[str, tuple[Any, ...], dict[str, Any]]] = []

    def __getattr__(self, name: str) -> Any:
        def recorder(*args: Any, **kwargs: Any) -> None:
            self.calls.append((name, args, kwargs))

        return recorder

    def names(self) -> list[str]:
        return [name for name, _, _ in self.calls]

    def call(self, name: str) -> tuple[tuple[Any, ...], dict[str, Any]]:
        for called, args, kwargs in self.calls:
            if called == name:
                return args, kwargs
        raise AssertionError(f"{name} was never called; got {self.names()}")


class _FakeConsole:
    def __init__(self) -> None:
        self.lines: list[str] = []

    def print(self, *args: Any, **kwargs: Any) -> None:
        self.lines.append(" ".join(str(a) for a in args))


class _FakeShell:
    def __init__(self) -> None:
        self.exported: dict[str, str] = {}

    def export(self, name: str, value: str) -> None:
        self.exported[name] = value

    def cd(self, path: Path) -> None: ...

    def exit(self) -> None: ...


class _FakeLevel:
    is_intro = False
    enforce_start_directory = False

    def __init__(
        self,
        level_id: str,
        *,
        result: tuple[bool, str] = (True, "Level-specific success!"),
        optional: bool = False,
        extension: bool = False,
        prepare_error: Exception | None = None,
    ) -> None:
        self.id = level_id
        self.title = f"Level {level_id}"
        self.hints: list[str] = []
        self.optional = optional
        self.extension = extension
        self._result = result
        self._prepare_error = prepare_error
        self.prepared = 0

    @property
    def is_bonus(self) -> bool:
        return self.optional or self.extension

    def validate(self, answer: str | None, state: GameState) -> tuple[bool, str]:
        return self._result

    def prepare(self, workspace: Path) -> None:
        if self._prepare_error is not None:
            raise self._prepare_error
        self.prepared += 1

    def reset(self, workspace: Path) -> None: ...

    def get_start_directory(self, workspace: Path) -> Path:
        return workspace


class _FakeRegistry:
    def __init__(self, levels: list[_FakeLevel], *, nearest: str | None = None) -> None:
        self._levels = levels
        self._by_id = {level.id: level for level in levels}
        self._nearest = nearest

    def get(self, level_id: str) -> _FakeLevel | None:
        return self._by_id.get(level_id)

    def list_levels(self) -> list[_FakeLevel]:
        return list(self._levels)

    def next_level(self, level_id: str) -> str | None:
        ids = [level.id for level in self._levels]
        if level_id not in ids:
            raise UnknownLevelError(level_id)
        index = ids.index(level_id)
        return ids[index + 1] if index + 1 < len(ids) else None

    def resolve_or_nearest(self, level_id: str) -> str | None:
        if level_id in self._by_id:
            return level_id
        if self._nearest is not None:
            return self._nearest
        return self._levels[0].id if self._levels else None


class _FakeWorkspace:
    def __init__(self, root: Path) -> None:
        self.root = root

    def init(self) -> Path:
        self.root.mkdir(parents=True, exist_ok=True)
        return self.root

    def remove(self) -> None: ...


class _FakeStateManager:
    def __init__(self, state: GameState | None) -> None:
        self.state = state
        self.saves = 0

    def load(self) -> GameState | None:
        return self.state

    def save(self, state: GameState) -> None:
        self.saves += 1
        self.state = state

    def remove(self) -> None:
        self.state = None
