"""Level registry for managing all game levels."""

from typing import Optional

from shellgame.levels.base import Level


class LevelRegistry:
    def __init__(self) -> None:
        self.levels: dict[str, Level] = {}
        self._level_order: list[str] = []

    def register(self, level: Level) -> None:
        if level.id is None:
            raise ValueError("Cannot register level without id")

        self.levels[level.id] = level
        if level.id not in self._level_order:
            self._level_order.append(level.id)

    def register_all(self, levels: list[Level]) -> None:
        for level in levels:
            self.register(level)

    def register_section(self, section_num: int, levels: list[Level]) -> None:
        for i, level in enumerate(levels):
            level.section = section_num
            level.id = f"{section_num}.{i}"
            self.register(level)

    def get(self, level_id: str) -> Optional[Level]:
        return self.levels.get(level_id)

    def next_level(self, current_id: str) -> Optional[str]:
        try:
            current_index = self._level_order.index(current_id)
            if current_index + 1 < len(self._level_order):
                return self._level_order[current_index + 1]
            return None
        except ValueError:
            return None

    def get_all_in_section(self, section: int) -> list[Level]:
        return [level for level in self.levels.values() if level.section == section]

    def get_core_levels(self) -> list[Level]:
        return [level for level in self.levels.values() if not level.optional and not level.extension]

    def list_levels(self) -> list[Level]:
        return [self.levels[lid] for lid in self._level_order]


_registry = LevelRegistry()


def get_registry() -> LevelRegistry:
    return _registry
