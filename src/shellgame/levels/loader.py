"""Initialize and register all game levels."""

from __future__ import annotations

import importlib
import pkgutil
import re
from types import ModuleType

from shellgame.levels import sections
from shellgame.levels.base import Level
from shellgame.levels.collector import Section
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
    if registry.list_levels():
        return

    for section_num, module in _discover_sections():
        registry.register_section(section_num, _section_levels(module))


def _section_levels(module: ModuleType) -> list[Level]:
    """Read the levels a section module declared via its `Section` object.

    Every section already builds one through `@section.level(...)`, so requiring
    an extra `get_levels()` at the bottom of each module was boilerplate that
    could silently go missing or return the wrong list.
    """
    section = getattr(module, "section", None)
    if isinstance(section, Section):
        return section.levels

    raise ValueError(f"Section module declares no `section` object: {module.__name__}")
