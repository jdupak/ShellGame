"""Focused tests for Display hint repeat rules.

These tests validate the UX contract around:
- progressive hint consumption (`shellgame hint`)
- repeat semantics (`shellgame hint --repeat`) without consuming new hints
- footer printing rules (repeat tip once at end, next-help only if more hints exist)
- no-more-hints panel
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List

from rich.console import Console

from shellgame.ui.display import Display


@dataclass
class _FakeState:
    """Minimal state shape required by Display.show_level_hint()."""

    level_hints_used: Dict[str, int] = field(default_factory=dict)


@dataclass
class _FakeLevel:
    id: str
    hints: List[str]


def _make_display() -> tuple[Display, Console]:
    console = Console(record=True, width=120)
    display = Display(console)
    return display, console


def _output(console: Console) -> str:
    return console.export_text()


def test_progressive_hint_consumes_one_and_updates_state() -> None:
    display, console = _make_display()
    state = _FakeState()
    level = _FakeLevel(id="1.1", hints=["H1", "H2", "H3"])

    display.show_level_hint(level, state, repeat=False)

    assert state.level_hints_used["1.1"] == 1
    out = _output(console)
    assert "H1" in out
    assert "H2" not in out
    assert "shellgame hint --repeat" in out  # repeat tip under normal hint
    assert "Potřebujete další pomoc? Napište: shellgame hint" in out  # next-help shown


def test_repeat_with_no_revealed_hints_does_not_consume() -> None:
    display, console = _make_display()
    state = _FakeState(level_hints_used={})
    level = _FakeLevel(id="1.1", hints=["H1", "H2"])

    display.show_level_hint(level, state, repeat=True)

    # Must not consume new hints
    assert state.level_hints_used.get("1.1", 0) == 0

    out = _output(console)
    # It still shows the first hint as a preview of "already revealed" (0 -> none),
    # but does not mark it consumed.
    assert "H1" in out
    assert "H2" not in out
    # Repeat tip should appear once
    assert out.count("shellgame hint --repeat") == 1
    # Since more hints exist, we can show the "need more help" line once at end
    assert out.count("Potřebujete další pomoc? Napište: shellgame hint") == 1


def test_repeat_reprints_only_revealed_hints_without_consuming_new_ones() -> None:
    display, console = _make_display()
    state = _FakeState(level_hints_used={"1.1": 2})
    level = _FakeLevel(id="1.1", hints=["H1", "H2", "H3"])

    display.show_level_hint(level, state, repeat=True)

    # Must not consume new hints
    assert state.level_hints_used["1.1"] == 2

    out = _output(console)
    assert "H1" in out
    assert "H2" in out
    assert "H3" not in out

    # In --repeat flow: no per-hint footer spam; repeat tip once at end
    assert out.count("shellgame hint --repeat") == 1
    # Another hint still exists (3rd), so next-help may be shown once at end
    assert out.count("Potřebujete další pomoc? Napište: shellgame hint") == 1


def test_repeat_when_all_hints_revealed_does_not_show_next_help() -> None:
    display, console = _make_display()
    state = _FakeState(level_hints_used={"1.1": 3})
    level = _FakeLevel(id="1.1", hints=["H1", "H2", "H3"])

    display.show_level_hint(level, state, repeat=True)

    assert state.level_hints_used["1.1"] == 3

    out = _output(console)
    assert "H1" in out and "H2" in out and "H3" in out
    # Repeat tip once at end
    assert out.count("shellgame hint --repeat") == 1
    # No more hints exist -> do not show next-help
    assert "Potřebujete další pomoc? Napište: shellgame hint" not in out


def test_no_more_hints_panel_is_used_when_progressive_exhausted() -> None:
    display, console = _make_display()
    state = _FakeState(level_hints_used={"1.1": 2})
    level = _FakeLevel(id="1.1", hints=["H1", "H2"])

    display.show_level_hint(level, state, repeat=False)

    # Must not change (already at end, nothing consumed)
    assert state.level_hints_used["1.1"] == 2

    out = _output(console)
    assert "Pro tento level již nejsou k dispozici žádné další nápovědy." in out
    # The UX rule says to show repeat tip below this panel (note with violet command)
    assert "shellgame hint --repeat" in out


def test_no_hints_defined_behaves_like_no_more_hints() -> None:
    display, console = _make_display()
    state = _FakeState(level_hints_used={})
    level = _FakeLevel(id="1.1", hints=[])

    display.show_level_hint(level, state, repeat=False)

    out = _output(console)
    assert "Pro tento level již nejsou k dispozici žádné další nápovědy." in out
    assert "shellgame hint --repeat" in out


def test_repeat_does_not_print_next_help_multiple_times() -> None:
    display, console = _make_display()
    state = _FakeState(level_hints_used={"1.1": 2})
    level = _FakeLevel(id="1.1", hints=["H1", "H2", "H3", "H4"])

    display.show_level_hint(level, state, repeat=True)

    out = _output(console)
    # Ensure helper line is not repeated per-hint
    assert out.count("Potřebujete další pomoc? Napište: shellgame hint") == 1
