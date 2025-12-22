"""Initialize and register all game levels."""

from __future__ import annotations

import importlib
import pkgutil
import re
from types import ModuleType

from shellgame.levels import sections
from shellgame.levels.base import Level
from shellgame.levels.registry import get_registry


def _get_section_number(module_name: str) -> int | None:
    match = re.search(r"section(\d+)$", module_name)
    if match:
        return int(match.group(1))
    return None


def _discover_sections() -> list[tuple[int, ModuleType]]:
    found_sections = []
    
    for _, name, _ in pkgutil.iter_modules(sections.__path__):
        section_num = _get_section_number(name)
        if section_num is not None:
            module_name = f"shellgame.levels.sections.{name}"
            module = importlib.import_module(module_name)
            found_sections.append((section_num, module))
    
    found_sections.sort(key=lambda x: x[0])
    return found_sections


def initialize_levels() -> None:
    registry = get_registry()
    
    for section_num, module in _discover_sections():
        if hasattr(module, "get_levels"):
            levels = module.get_levels()
            registry.register_section(section_num, levels)


class LevelLoader:
    def __init__(self) -> None:
        self._levels: dict[str, Level] = {}
        self._sections: dict[int, list[Level]] = {}
        self._load_levels()

    def _register_section(self, levels: list[Level]) -> None:
        for level in levels:
            if level.id is None or level.section is None:
                raise ValueError("Levels must have id and section set before registration")

            self._levels[level.id] = level
            self._sections.setdefault(level.section, []).append(level)

    def _load_levels(self) -> None:
        for _, module in _discover_sections():
            if hasattr(module, "get_levels"):
                self._register_section(module.get_levels())
