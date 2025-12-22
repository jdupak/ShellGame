"""Focused tests for Display hint repeat rules.

These tests validate the UX contract around:
- progressive hint consumption (`shellgame hint`)
- repeat semantics (`shellgame hint --repeat`) without consuming new hints
- footer printing rules (repeat tip once at end, next-help only if more hints exist)
- no-more-hints panel
"""

from __future__ import annotations

from rich.console import Console

from shellgame.ui.display import Display


def _make_display() -> tuple[Display, Console]:
    console = Console(record=True, width=120)
    display = Display(console)
    return display, console


def _output(console: Console) -> str:
    return console.export_text()


def test_show_hint_output() -> None:
    display, console = _make_display()
    
    display.show_hint("H1", 0, 3)

    out = _output(console)
    assert "H1" in out
    assert "shellgame hint --repeat" in out  # repeat tip under normal hint
    assert "Potřebujete další pomoc? Napište: shellgame hint" in out  # next-help shown


def test_repeat_with_no_revealed_hints_shows_first_hint() -> None:
    display, console = _make_display()
    hints = ["H1", "H2"]

    display.show_repeated_hints(hints, 0)

    out = _output(console)
    # It shows the first hint as a preview of "already revealed" (0 -> none)
    assert "H1" in out
    assert "H2" not in out
    # Repeat tip should appear once
    assert out.count("shellgame hint --repeat") == 1
    # Since more hints exist, we can show the "need more help" line once at end
    assert out.count("Potřebujete další pomoc? Napište: shellgame hint") == 1


def test_repeat_reprints_only_revealed_hints() -> None:
    display, console = _make_display()
    hints = ["H1", "H2", "H3"]

    display.show_repeated_hints(hints, 2)

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
    hints = ["H1", "H2", "H3"]

    display.show_repeated_hints(hints, 3)

    out = _output(console)
    assert "H1" in out and "H2" in out and "H3" in out
    # Repeat tip once at end
    assert out.count("shellgame hint --repeat") == 1
    # No more hints exist -> do not show next-help
    assert "Potřebujete další pomoc? Napište: shellgame hint" not in out


def test_no_more_hints_panel() -> None:
    display, console = _make_display()
    
    display.show_no_more_hints()

    out = _output(console)
    assert "Pro tento level již nejsou k dispozici žádné další nápovědy." in out
    # The UX rule says to show repeat tip below this panel (note with violet command)
    assert "shellgame hint --repeat" in out


def test_repeat_does_not_print_next_help_multiple_times() -> None:
    display, console = _make_display()
    hints = ["H1", "H2", "H3", "H4"]

    display.show_repeated_hints(hints, 2)

    out = _output(console)
    # Ensure helper line is not repeated per-hint
    assert out.count("Potřebujete další pomoc? Napište: shellgame hint") == 1
