"""Level registry for managing all game levels."""

from typing import Optional

from shellgame.levels.base import Level


class LevelRegistry:
    """Registry for all game levels."""

    def __init__(self) -> None:
        """Initialize level registry."""
        self.levels: dict[str, Level] = {}
        self._level_order: list[str] = []

    def register(self, level: Level) -> None:
        """
        Register a level.

        Args:
            level: Level instance to register
        """
        if level.id is None:
            raise ValueError("Cannot register level without id")

        self.levels[level.id] = level
        if level.id not in self._level_order:
            self._level_order.append(level.id)

    def register_all(self, levels: list[Level]) -> None:
        """
        Register multiple levels.

        Args:
            levels: List of Level instances
        """
        for level in levels:
            self.register(level)

    def register_section(self, section_num: int, levels: list[Level]) -> None:
        """
        Register a list of levels for a specific section.
        Assigns IDs automatically as `{section_num}.{index}`.

        Args:
            section_num: The section number (e.g. 1, 2)
            levels: List of Level instances to register
        """
        for i, level in enumerate(levels):
            level.section = section_num
            level.id = f"{section_num}.{i}"
            self.register(level)

    def get(self, level_id: str) -> Optional[Level]:
        """
        Get level by ID.

        Args:
            level_id: Level identifier

        Returns:
            Level instance or None if not found
        """
        return self.levels.get(level_id)

    def next_level(self, current_id: str) -> Optional[str]:
        """
        Get next level ID based on progression rules.

        Args:
            current_id: Current level ID

        Returns:
            Next level ID or None if at end
        """
        try:
            current_index = self._level_order.index(current_id)
            if current_index + 1 < len(self._level_order):
                return self._level_order[current_index + 1]
            return None
        except ValueError:
            return None

    def get_all_in_section(self, section: int) -> list[Level]:
        """
        Get all levels in a section.

        Args:
            section: Section number

        Returns:
            List of levels in that section
        """
        return [level for level in self.levels.values() if level.section == section]

    def get_core_levels(self) -> list[Level]:
        """
        Get all core (non-optional, non-extension) levels.

        Returns:
            List of core levels
        """
        return [level for level in self.levels.values() if not level.optional and not level.extension]

    def list_levels(self) -> list[Level]:
        """
        Get all levels in order.

        Returns:
            List of all Level instances in order
        """
        return [self.levels[lid] for lid in self._level_order]


# Global registry instance
_registry = LevelRegistry()


def get_registry() -> LevelRegistry:
    """
    Get the global level registry.

    Returns:
        Global LevelRegistry instance
    """
    return _registry
