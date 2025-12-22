"""Initialize and register all game levels."""

from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.registry import get_registry
from shellgame.levels.sections import (
    section0,
    section1,
    section2,
    section3,
    section4,
    section5,
    section6,
    section7,
    section8,
    section9,
    section10,
    section11,
)


def initialize_levels() -> None:
    """Register all levels from all sections."""
    registry = get_registry()

    # Register Section 0 levels
    for level in section0.get_levels():
        registry.register(level)

    # Register Section 1 levels
    for level in section1.get_levels():
        registry.register(level)

    # Register Section 2 levels
    for level in section2.get_levels():
        registry.register(level)

    # Register Section 3 levels
    for level in section3.get_levels():
        registry.register(level)

    # Register Section 4 levels
    for level in section4.get_levels():
        registry.register(level)

    # Register Section 5 levels
    for level in section5.get_levels():
        registry.register(level)

    # Register Section 6 levels
    for level in section6.get_levels():
        registry.register(level)

    # Register Section 7 levels
    for level in section7.get_levels():
        registry.register(level)

    # Register Section 8 levels
    for level in section8.get_levels():
        registry.register(level)

    # Register Section 9 levels
    for level in section9.get_levels():
        registry.register(level)

    # Register Section 10 levels
    for level in section10.get_levels():
        registry.register(level)

    # Register Section 11 levels
    for level in section11.get_levels():
        registry.register(level)


class LevelLoader:
    """Loads and manages game levels."""

    def __init__(self) -> None:
        self._levels: dict[str, Level] = {}
        self._sections: dict[int, list[Level]] = {}
        self._load_levels()

    def _register_section(self, levels: list[Level]) -> None:
        """Index a batch of levels by id and section."""
        for level in levels:
            if level.id is None or level.section is None:
                raise ValueError("Levels must have id and section set before registration")

            self._levels[level.id] = level
            self._sections.setdefault(level.section, []).append(level)

    def _load_levels(self) -> None:
        """Load all levels from section modules."""
        # Load levels from each section
        self._register_section(section0.get_levels())
        self._register_section(section1.get_levels())
        self._register_section(section2.get_levels())
        self._register_section(section3.get_levels())
        self._register_section(section4.get_levels())
        self._register_section(section5.get_levels())
        self._register_section(section6.get_levels())
        self._register_section(section7.get_levels())
        self._register_section(section8.get_levels())
        self._register_section(section9.get_levels())
        self._register_section(section10.get_levels())
        self._register_section(section11.get_levels())
