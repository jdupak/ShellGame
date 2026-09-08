"""Declarative section/level registration helpers."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import replace
from pathlib import Path
from typing import TypeVar

from shellgame.levels.base import Level
from shellgame.levels.cdpolicy import CD_EVIDENCE, cd_marker
from shellgame.levels.completion import Evidence, Requirement
from shellgame.levels.fixture import WorkspaceFixture

L = TypeVar("L", bound=Level)


class Section:
    def __init__(
        self,
        number: int,
        *,
        root: str = "",
        fixture: WorkspaceFixture | None = None,
    ) -> None:
        if number < 0:
            raise ValueError("Section number cannot be negative")
        root_path = Path(root)
        if root_path.is_absolute() or ".." in root_path.parts:
            raise ValueError(f"Section root must be workspace-relative: {root!r}")
        self._levels: list[Level] = []
        self.number = number
        self.root = root
        self.fixture = fixture

    def _register(self, number: int, cls: type[L]) -> type[L]:
        level_id = f"{self.number}.{number}"
        if any(level.id == level_id for level in self._levels):
            raise ValueError(f"Duplicate level ID: {level_id}")
        cls.id = level_id
        cls.section = self.number
        cls.section_root = self.root
        cls.section_fixture = self.fixture
        _bind_cd_evidence(cls, cd_marker(level_id))
        self._levels.append(cls())
        return cls

    @property
    def levels(self) -> list[Level]:
        return list(self._levels)

    def level(self, number: int) -> Callable[[type[L]], type[L]]:
        if number < 0:
            raise ValueError("Level number cannot be negative")
        return lambda cls: self._register(number, cls)


def _bind_cd_evidence(cls: type[Level], marker: str) -> None:
    """Give the level's `cd` evidence marker a name derived from its ID.

    Authors write `CdEvidence(...)` and a `CdPolicy` without naming a marker,
    so a marker can be neither misspelled nor shared with another level.
    """
    if cls.cd_policy is not None and cls.cd_policy.marker == CD_EVIDENCE:
        cls.cd_policy = replace(cls.cd_policy, marker=marker)

    completion = cls.completion
    if completion is None:
        return

    requirements = tuple(_bind_requirement(requirement, marker) for requirement in completion.requirements)
    allow_empty_when = (
        _bind_requirement(completion.allow_empty_when, marker) if completion.allow_empty_when is not None else None
    )
    if requirements != completion.requirements or allow_empty_when is not completion.allow_empty_when:
        cls.completion = replace(completion, requirements=requirements, allow_empty_when=allow_empty_when)


def _bind_requirement(requirement: Requirement, marker: str) -> Requirement:
    if isinstance(requirement, Evidence) and requirement.marker == CD_EVIDENCE:
        return replace(requirement, marker=marker)
    return requirement
