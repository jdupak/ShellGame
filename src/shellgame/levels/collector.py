"""Declarative section/level registration helpers."""

from __future__ import annotations

from typing import TypeVar, overload

from shellgame.levels.base import Level

T = TypeVar("T", bound=type[Level])


class Section:
    """
    Collects level instances via decorator.

    New API (preferred):
        section = Section()

        @section.level
        class MyLevel(Level):
            ...

    Backward-compatible API (legacy):
        levels = LevelCollector()

        @levels.register
        class MyLevel(Level):
            ...
    """

    def __init__(self) -> None:
        self._levels: list[Level] = []

    def _register(self, cls: T) -> T:
        """Register a level class and instantiate it immediately."""
        self._levels.append(cls())
        return cls

    @property
    def levels(self) -> list[Level]:
        """Get the list of collected level instances."""
        return list(self._levels)

    @property
    def level(self):
        """
        Decorator used as `@section.level`.

        Implemented as a property so it can be used without parentheses.
        """
        return self._register

    # Backward compatibility: allow `@section.register` too.
    @property
    def register(self):
        return self._register


class LevelCollector(Section):
    """Backward-compatible alias for the old name."""
