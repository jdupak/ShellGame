"""Level registry for managing all game levels."""

from shellgame.levels.base import Level


def parse_level_id(level_id: str) -> tuple[int, int] | None:
    """Split a persistent level ID into `(section, number)`, or None if malformed."""
    section, separator, number = level_id.partition(".")
    if not separator or not section.isdigit() or not number.isdigit():
        return None
    return (int(section), int(number))


class UnknownLevelError(LookupError):
    """Raised when a level ID is not registered at all."""


class LevelRegistry:
    def __init__(self) -> None:
        self.levels: dict[str, Level] = {}
        self._level_order: list[str] = []

    def register(self, level: Level) -> None:
        if not level.id:
            raise ValueError("Cannot register level without id")
        if level.id in self.levels and self.levels[level.id] is not level:
            raise ValueError(f"Duplicate level ID: {level.id}")

        self.levels[level.id] = level
        if level.id not in self._level_order:
            self._level_order.append(level.id)

    def register_section(self, section_num: int, levels: list[Level]) -> None:
        registered: list[Level] = []
        for level in levels:
            level_id = level.id
            id_section, _, id_number = level_id.partition(".")
            if (
                level.section != section_num
                or not id_section.isdigit()
                or int(id_section) != section_num
                or not id_number.isdigit()
            ):
                raise ValueError(f"Invalid stable ID {level_id!r} for section {section_num}")
            registered.append(level)

        for level in sorted(registered, key=lambda item: parse_level_id(item.id) or (0, 0)):
            self.register(level)

    def get(self, level_id: str) -> Level | None:
        return self.levels.get(level_id)

    def next_level(self, current_id: str) -> str | None:
        """The level after `current_id`, or None when it is the last one.

        An unregistered ID is a different situation entirely and raises, so that
        a stale saved ID can never be mistaken for "the player finished".
        """
        try:
            current_index = self._level_order.index(current_id)
        except ValueError as error:
            raise UnknownLevelError(current_id) from error

        if current_index + 1 < len(self._level_order):
            return self._level_order[current_index + 1]
        return None

    def resolve_or_nearest(self, level_id: str) -> str | None:
        """Map a possibly stale saved ID onto a level that still exists.

        Levels get renumbered between versions, which would otherwise leave a
        saved game permanently unplayable. Resolution keeps the player as close
        as possible to where they were: the same section if it still exists,
        otherwise the next section that does.
        """
        if level_id in self.levels:
            return level_id
        if not self._level_order:
            return None

        target = parse_level_id(level_id)
        if target is None:
            return self._level_order[0]

        candidates = [(key, known) for known in self._level_order if (key := parse_level_id(known)) is not None]
        if not candidates:
            return self._level_order[0]

        later = [known for key, known in candidates if key >= target]
        if later:
            return later[0]
        return candidates[-1][1]

    def list_levels(self) -> list[Level]:
        return [self.levels[lid] for lid in self._level_order]


_registry = LevelRegistry()


def get_registry() -> LevelRegistry:
    return _registry
