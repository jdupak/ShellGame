"""Tests for CLI commands `repeat` and `reset`.

We patch the global singletons in `shellgame.cli.commands` to avoid
filesystem side-effects and to assert the right display calls happen.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Optional

import shellgame.cli.commands as commands


@dataclass
class _FakeState:
    username: str
    workspace: Path
    current_level: str


class _FakeLevel:
    def __init__(self, level_id: str) -> None:
        self.id = level_id
        self.reset_called_with: list[Path] = []

    def reset(self, workspace: Path) -> None:
        self.reset_called_with.append(workspace)


class _FakeRegistry:
    def __init__(self, levels: dict[str, _FakeLevel]) -> None:
        self._levels = levels

    def get(self, level_id: str) -> Optional[_FakeLevel]:
        return self._levels.get(level_id)


class _FakeDisplay:
    def __init__(self) -> None:
        self.reset_calls: list[str] = []
        self.instructions_calls: list[str] = []

    def show_reset(self, level_id: str) -> None:
        self.reset_calls.append(level_id)

    def show_instructions(self, level: _FakeLevel) -> None:
        self.instructions_calls.append(level.id)

    def show_not_initialized(self) -> None:
        raise AssertionError("should not be called in these tests")


def test_reset_shows_assignment_again(monkeypatch, tmp_path: Path) -> None:
    state = _FakeState(username="u", workspace=tmp_path / "ws", current_level="1.2")
    level = _FakeLevel("1.2")

    fake_display = _FakeDisplay()
    fake_registry = _FakeRegistry({"1.2": level})

    monkeypatch.setattr(commands, "display", fake_display)
    monkeypatch.setattr(commands, "level_registry", fake_registry)
    monkeypatch.setattr(commands.state_manager, "load", lambda: state)

    commands.reset.callback()  # type: ignore[attr-defined]

    assert level.reset_called_with == [state.workspace]
    assert fake_display.reset_calls == ["1.2"]
    assert fake_display.instructions_calls == ["1.2"]


def test_repeat_defaults_to_current_level(monkeypatch, tmp_path: Path) -> None:
    state = _FakeState(username="u", workspace=tmp_path / "ws", current_level="1.7")
    level = _FakeLevel("1.7")

    fake_display = _FakeDisplay()
    fake_registry = _FakeRegistry({"1.7": level})

    monkeypatch.setattr(commands, "display", fake_display)
    monkeypatch.setattr(commands, "level_registry", fake_registry)
    monkeypatch.setattr(commands.state_manager, "load", lambda: state)

    commands.repeat.callback(section_num=None, level_id=None)  # type: ignore[attr-defined]

    assert fake_display.instructions_calls == ["1.7"]


def test_repeat_by_section_uses_intro_level(monkeypatch, tmp_path: Path) -> None:
    state = _FakeState(username="u", workspace=tmp_path / "ws", current_level="9.9")
    level = _FakeLevel("2.0")

    fake_display = _FakeDisplay()
    fake_registry = _FakeRegistry({"2.0": level})

    monkeypatch.setattr(commands, "display", fake_display)
    monkeypatch.setattr(commands, "level_registry", fake_registry)
    monkeypatch.setattr(commands.state_manager, "load", lambda: state)

    commands.repeat.callback(section_num=2, level_id=None)  # type: ignore[attr-defined]

    assert fake_display.instructions_calls == ["2.0"]


def test_repeat_by_level_overrides_section(monkeypatch, tmp_path: Path) -> None:
    state = _FakeState(username="u", workspace=tmp_path / "ws", current_level="9.9")
    level = _FakeLevel("1.1")

    fake_display = _FakeDisplay()
    fake_registry = _FakeRegistry({"1.1": level, "2.0": _FakeLevel("2.0")})

    monkeypatch.setattr(commands, "display", fake_display)
    monkeypatch.setattr(commands, "level_registry", fake_registry)
    monkeypatch.setattr(commands.state_manager, "load", lambda: state)

    commands.repeat.callback(section_num=2, level_id="1.1")  # type: ignore[attr-defined]

    assert fake_display.instructions_calls == ["1.1"]
