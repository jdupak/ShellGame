#!/usr/bin/env python3
"""Regenerate the golden state files that tests/state_compat.rs compares against.

Every ``python_*.json`` file is written by the REAL Python implementation: the state
is built with ``ProgressTracker`` and saved with ``StateManager.save``, i.e.
``GameState.model_dump_json(indent=2)`` plus the atomic write. The Rust port has to
reproduce those bytes exactly, because students switch binaries mid-semester.
``python_v1_0_migrated.json`` and ``python_v2_0_migrated.json`` are what Python makes of the
hand-written ``v1_0.json`` / ``v2_0.json`` (migrated, then saved), and ``migration_table.json``
is computed with the real ``_migrate_level_id_to_2_1``. ``timestamp_table.json`` records how the
real pydantic reads a few hundred timestamp strings, valid and invalid.

Run it from the repository root; only pydantic is needed::

    PYTHONPATH=src uv run --no-project --with 'pydantic==2.12.5' python tests/state/generate_golden.py

All timestamps are fixed, so the output is deterministic and re-running it leaves
``git diff`` empty. The hand-written fixtures (``v1_0.json``, ``v2_0.json``,
``corrupt.json``, ``version_99.json``, ``not_object.json``, ``missing_workspace.json``)
are never touched. Once ``src/shellgame`` is deleted, run this from a checkout of the
last Python release: the generated files only change when the Python schema does.
"""

from __future__ import annotations

import json
import os
import shutil
import sys
import tempfile
from dataclasses import dataclass
from datetime import datetime, timedelta
from pathlib import Path

from pydantic import TypeAdapter, ValidationError

from shellgame.core.progress import ProgressTracker
from shellgame.state.manager import GameState, StateManager, _migrate_level_id_to_2_1, _remap_level_ids

HERE = Path(__file__).resolve().parent


@dataclass(frozen=True)
class Step:
    """One level as a player plays it."""

    level_id: str
    seconds: float
    attempts: int = 0
    hints: int = 0
    #: Forces the fraction of the completion time, so that both timestamp shapes occur.
    microsecond: int | None = None


#: ``1.11`` is absent on purpose: that level was retired and its ID is never reused.
EARLY_GAME = [
    Step("0.0", 7.5),
    Step("1.0", 4.25),
    Step("1.1", 38.9, microsecond=0),
    Step("1.2", 72.5, attempts=2, hints=1),
    Step("1.3", 25.0),
    Step("1.4", 140.75, attempts=3, hints=2, microsecond=0),
    Step("1.10", 61.0, attempts=1, microsecond=0),
    Step("1.12", 95.123456, hints=3),
    Step("2.0", 3.0, microsecond=0),
    Step("2.1", 44.0, microsecond=0),
    Step("2.2", 120.5, attempts=1, hints=1),
]


class Scenario:
    """A game driven the way ``GameSession`` drives it, on a clock that moves only when told to."""

    def __init__(self, username: str, workspace: Path, clock: datetime) -> None:
        self.tracker = ProgressTracker()
        self.clock = clock
        # What ``StateManager.create`` produces, minus its two ``datetime.now()`` calls.
        self.state = GameState(username=username, workspace=workspace, current_level="0.0", start_time=clock)
        self.state.level_started_at["0.0"] = clock

    def tick(self, seconds: float, microsecond: int | None = None) -> datetime:
        self.clock += timedelta(seconds=seconds)
        if microsecond is not None:
            self.clock = self.clock.replace(microsecond=microsecond)
        return self.clock

    def play(self, step: Step, next_level: str) -> None:
        """Start the level, fail, ask for hints, complete it and advance to ``next_level``."""
        level_id, state = step.level_id, self.state
        self.tracker.ensure_level_started(state, level_id, now=self.clock)
        for _ in range(step.attempts):
            self.tracker.record_attempt(state, level_id)
        for _ in range(step.hints):
            self.tracker.reveal_next_hint(state, level_id, total_hints=3)
        completed_at = self.tick(step.seconds, step.microsecond)
        self.tracker.record_completion(state, level_id, completed_at=completed_at)
        state.current_level = next_level
        self.tracker.ensure_level_started(state, next_level, now=completed_at)


def save_as(manager: StateManager, state: GameState, name: str) -> None:
    """Write ``state`` with the real ``StateManager.save`` and copy the bytes into the golden folder."""
    manager.save(state)
    data = manager.state_file.read_bytes()
    (HERE / name).write_bytes(data)
    print(f"wrote {name} ({len(data)} bytes)")


def migrate_fixture(manager: StateManager, source: str, target: str) -> None:
    """Load a hand-written old-format fixture with the real migration, then save it as Python would."""
    shutil.copyfile(HERE / source, manager.state_file)
    save_as(manager, manager.load(), target)


def build_mid_game() -> GameState:
    """A player part-way through the course who also used ``dev jump`` once (out-of-order IDs)."""
    # ``start_time`` has no fraction, the first ``level_started_at`` entry has one.
    game = Scenario("Jůlie Novák", Path("/tmp/shellgame-Jůlie Novák"), datetime(2026, 2, 16, 9, 15, 42, 417))
    game.state.start_time = datetime(2026, 2, 16, 9, 15, 42)

    followers = [step.level_id for step in EARLY_GAME[1:]] + ["10.3"]
    for step, next_level in zip(EARLY_GAME, followers, strict=True):
        game.play(step, next_level)

    # ``dev jump 10.3``: the player lands in section 10 with sections 3-9 never played.
    game.tick(600, microsecond=0)
    game.state.level_started_at["10.3"] = game.clock
    game.play(Step("10.3", 51.0, attempts=3), "10.4")
    game.play(Step("10.4", 213.5, hints=3), "10.5")

    # The current level has been started and failed once, but is not complete.
    game.tracker.record_attempt(game.state, "10.5")
    return game.state


def build_finished() -> GameState:
    """A player who reached the end: ``completed_at`` is set and ``current_level`` stays on the last level."""
    game = Scenario("Eliška", Path("/tmp/shellgame-eliska"), datetime(2026, 3, 2, 8, 0, 0))
    game.play(Step("0.0", 6.0), "1.0")
    game.play(Step("1.0", 3.5), "11.6")
    game.tracker.record_attempt(game.state, "11.6")
    game.play(Step("11.6", 310.25), "11.6")
    game.state.completed_at = datetime(2026, 3, 2, 8, 5, 22, 654321)
    return game.state


def build_fresh() -> GameState:
    """The file ``shellgame`` writes on the very first run: every per-level map but one is empty."""
    game = Scenario("player", Path("/tmp/shellgame-player"), datetime(2026, 9, 1, 12, 0, 0, 250000))
    game.state.level_started_at["0.0"] = datetime(2026, 9, 1, 12, 0, 0, 250009)
    return game.state


def build_escapes() -> GameState:
    """Strings that exercise JSON escaping: pydantic escapes controls and writes everything else as-is."""
    username = 'Quote " back\\slash tab\t newline\n bell\x07 unit-sep\x1f del\x7f line-sep  ünï čödé 😀 日本'
    game = Scenario(username, Path("/tmp/shell game/sp ace ✓"), datetime(2026, 4, 1, 0, 0, 0, 1))
    game.play(Step("0.0", 1.000001, attempts=1), "1.0")
    return game.state


# Level IDs that are malformed or sit at an edge, next to the well-formed ones.
ODD_LEVEL_IDS = [
    *["", ".", "7", "7.", ".7", "7.1.2", "07.1", "007.5", "+7.1", "-7.1", " 7.1", "7 .1", "7.1 ", "x.1", "7.x"],
    *["10", "10.", "10.x.y", "100.1", "99999999999999999999.1", "1.12", "Ahoj.7"],
]


def build_migration_table() -> dict[str, list[list[object]]]:
    """Pairs from the real ID migration, plus what ``_remap_level_ids`` makes of a whole map.

    The map is keyed by every ID below, with the ID's position as its value. Several old IDs can
    land on one new ID (``07.1`` and ``7.1`` both become ``8.1``), and the real code decides
    which entry survives and where it ends up.
    """
    ids = [f"{section}.{number}" for section in range(13) for number in range(21)] + ODD_LEVEL_IDS
    ids = list(dict.fromkeys(ids))  # the range above already holds some of the extras

    pairs = [[old, _migrate_level_id_to_2_1(old)] for old in ids]
    keyed = {"level_attempts": {old: position for position, old in enumerate(ids)}}
    remapped = _remap_level_ids(keyed, _migrate_level_id_to_2_1)["level_attempts"]
    return {"pairs": pairs, "remapped_map": [[key, value] for key, value in remapped.items()]}


def dump_migration_table(table: dict[str, list[list[object]]]) -> str:
    """One row per line, so that a change to the table shows up as a readable diff."""

    def rows(items: list[list[object]]) -> str:
        return ",\n".join("    " + json.dumps(item, ensure_ascii=False) for item in items)

    return (
        '{\n  "pairs": [\n' + rows(table["pairs"]) + "\n  ],\n"
        '  "remapped_map": [\n' + rows(table["remapped_map"]) + "\n  ]\n}\n"
    )


# The variations of a valid timestamp that build_timestamp_table tries, one part at a time.
# fmt: off
TIMESTAMP_DATES = [
    "2026-03-01", "0001-01-01", "9999-12-31", "2026-02-29", "2024-02-29", "2026-13-01", "2026-00-10", "2026-04-31",
    "20260301", "2026-3-1", "2026/03/01", "+2026-03-01", "", "2026-03-1", "2026-03-011", "0000-12-31",
    "２０２６-03-01",
]
TIMESTAMP_SEPARATORS = ["T", "t", " ", "_", "", "-", ":", "  ", "TT", "\t"]
TIMESTAMP_TIMES = [
    "10:20:30", "10:20", "10", "1020", "102030", "24:00:00", "23:59:59", "23:59:60", "00:00:00", "10:60:00",
    "10:20:61", "1:20:30", "10:2:30", "10:20:3", "10:20:", "10:20:30:", "10.20.30", "10:20:30 ", "", "٣:20:30",
]
TIMESTAMP_FRACTIONS = [
    "", ".", ".1", ".12", ".123", ".123456", ".1234567", ".123456789", ".1234567890123", ",5", ",", ".x", ".0",
    ".000000", ".999999", ".9999999", "..1", ".1.2",
]
TIMESTAMP_OFFSETS = [
    "", "Z", "z", "+02:00", "-05:30", "+0200", "+02", "+2:00", "+24:00", "+23:59", "-23:59", "+00:60", "-00:00", " Z",
    "z ", "+02:00:30", "UTC", "+02:0", "Zz", "+", "-", "+02:00Z", "+2", "+020", "+02000",
]
# fmt: on


def build_timestamp_table() -> list[dict[str, object]]:
    """How the real pydantic reads timestamp text, one row per string.

    A row is ``{"text", "wall", "offset"}``: ``wall`` is ``[year, month, day, hour, minute,
    second, microsecond]`` and ``offset`` the seconds east of UTC (``null`` for a naive value),
    or ``wall`` is ``null`` when pydantic refuses the text. Rows vary one part of a valid
    timestamp at a time, plus every fraction next to every kind of offset.

    Left out on purpose, because the Rust reader does not accept them: unix timestamps written
    as text (``"1772360430"``) and a Unicode minus sign in offsets.
    """
    adapter = TypeAdapter(datetime)

    texts = [f"{date}T10:20:30" for date in TIMESTAMP_DATES]
    texts += [f"2026-03-01{separator}10:20:30" for separator in TIMESTAMP_SEPARATORS]
    texts += [f"2026-03-01T{time}" for time in TIMESTAMP_TIMES]
    texts += [f"2026-03-01T10:20:30{offset}" for offset in TIMESTAMP_OFFSETS]
    for fraction in TIMESTAMP_FRACTIONS:
        texts += [f"2026-03-01T10:20:30{fraction}{offset}" for offset in ("", "Z", "+02:00", "-05:30")]
    texts += ["2026-03-01", "2026-03-01T10:20", "2026-03-01 10:20Z", "", "yesterday", "0001-01-01T00:00:00"]
    texts += ["9999-12-31T23:59:59.999999"]

    rows: list[dict[str, object]] = []
    for text in dict.fromkeys(texts):
        try:
            value = adapter.validate_python(text)
        except ValidationError:
            rows.append({"text": text, "wall": None, "offset": None})
            continue
        offset = value.utcoffset()
        wall = [value.year, value.month, value.day, value.hour, value.minute, value.second, value.microsecond]
        rows.append({"text": text, "wall": wall, "offset": None if offset is None else int(offset.total_seconds())})
    return rows


def dump_timestamp_table(rows: list[dict[str, object]]) -> str:
    """One row per line, keys in a fixed order."""
    return "[\n" + ",\n".join("  " + json.dumps(row, ensure_ascii=False) for row in rows) + "\n]\n"


def check_coverage(state: GameState) -> None:
    """Fail loudly if a regenerated fixture stops covering what the Rust tests rely on."""
    text = (HERE / "python_v2_1.json").read_text(encoding="utf-8")
    stamps = [word for word in text.split('"') if word.startswith("20") and "T" in word]
    assert any("." not in stamp for stamp in stamps), "need a timestamp without microseconds"
    assert any("." in stamp for stamp in stamps), "need a timestamp with microseconds"
    assert {"1.12", "10.4"} <= set(state.levels_complete), "need the double-digit IDs"
    assert state.completed_at is None
    assert "ů" in text
    assert "\\u" not in text, "non-ASCII must be written as-is"


def main() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        # StateManager reads this when it is constructed, so set it first.
        os.environ["SHELLGAME_STATE_DIR"] = tmp
        manager = StateManager()

        mid_game = build_mid_game()
        save_as(manager, mid_game, "python_v2_1.json")
        save_as(manager, build_finished(), "python_v2_1_finished.json")
        save_as(manager, build_fresh(), "python_v2_1_fresh.json")
        save_as(manager, build_escapes(), "python_v2_1_escapes.json")
        migrate_fixture(manager, "v1_0.json", "python_v1_0_migrated.json")
        migrate_fixture(manager, "v2_0.json", "python_v2_0_migrated.json")

        table = build_migration_table()
        (HERE / "migration_table.json").write_text(dump_migration_table(table), encoding="utf-8")
        print(f"wrote migration_table.json ({len(table['pairs'])} pairs)")

        timestamps = build_timestamp_table()
        (HERE / "timestamp_table.json").write_text(dump_timestamp_table(timestamps), encoding="utf-8")
        accepted = sum(1 for row in timestamps if row["wall"] is not None)
        print(f"wrote timestamp_table.json ({len(timestamps)} strings, {accepted} accepted)")

    check_coverage(mid_game)
    return 0


if __name__ == "__main__":
    sys.exit(main())
