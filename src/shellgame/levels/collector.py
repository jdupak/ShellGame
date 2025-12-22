"""Declarative section/level registration helpers."""

from __future__ import annotations

from typing import TypeVar, overload

from shellgame.levels.base import Level

T = TypeVar("T", bound=type[Level])


class Section:
    def __init__(self) -> None:
        self._levels: list[Level] = []

    def _register(self, cls: T) -> T:
        self._levels.append(cls())
        return cls

    @property
    def levels(self) -> list[Level]:
        return list(self._levels)

    @property
    def level(self):
        return self._register

    # Backward compatibility: allow `@section.register` too.
    @property
    def register(self):
        return self._register


class LevelCollector(Section):
    pass
