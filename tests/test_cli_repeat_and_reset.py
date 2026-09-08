"""Tests for CLI commands `repeat` and `reset`.

We patch the global singletons in `shellgame.cli.commands` to avoid
filesystem side-effects and to assert the right display calls happen.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from shellgame.cli import commands


@dataclass
class _FakeState:
    username: str
    workspace: Path
    current_level: str


class _FakeLevel:
    def __init__(self, level_id: str, *, enforce_start_directory: bool = False) -> None:
        self.id = level_id
        self.reset_called_with: list[Path] = []
        self.prepare_called_with: list[Path] = []
        self.enforce_start_directory = enforce_start_directory
        self.start_directory = ""

    def reset(self, workspace: Path) -> None:
        self.reset_called_with.append(workspace)

    def prepare(self, workspace: Path) -> None:
        self.prepare_called_with.append(workspace)

    def get_start_directory(self, workspace: Path) -> Path:
        return workspace / self.start_directory if self.start_directory else workspace


class _FakeRegistry:
    def __init__(self, levels: dict[str, _FakeLevel]) -> None:
        self._levels = levels

    def get(self, level_id: str) -> _FakeLevel | None:
        return self._levels.get(level_id)


class _FakeDisplay:
    def __init__(self) -> None:
        self.reset_calls: list[str] = []
        self.instructions_calls: list[str] = []
        self.workspace_restored_calls: list[Path] = []

    def show_workspace_restored(self, workspace: Path) -> None:
        self.workspace_restored_calls.append(workspace)

    def show_reset(self, level_id: str) -> None:
        self.reset_calls.append(level_id)

    def show_instructions(self, level: _FakeLevel) -> None:  # type: ignore[override]
        self.instructions_calls.append(level.id)

    def show_not_initialized(self) -> None:
        raise AssertionError("should not be called in these tests")


def test_reset_shows_assignment_again(monkeypatch: Any, tmp_path: Path) -> None:
    state = _FakeState(username="u", workspace=tmp_path / "ws", current_level="1.2")
    level = _FakeLevel("1.2")

    fake_display = _FakeDisplay()
    fake_registry = _FakeRegistry({"1.2": level})

    monkeypatch.setattr(commands, "display", fake_display)
    monkeypatch.setattr(commands, "level_registry", fake_registry)
    monkeypatch.setattr(commands.state_manager, "load", lambda: state)

    assert commands.reset.callback is not None
    commands.reset.callback()

    assert level.reset_called_with == [state.workspace]
    assert fake_display.reset_calls == ["1.2"]
    assert fake_display.instructions_calls == ["1.2"]


def test_reset_teleports_lost_player_back_to_start(monkeypatch: Any, tmp_path: Path) -> None:
    """`reset` must rescue a player who wandered outside the level start directory."""
    workspace = tmp_path / "ws"
    start = workspace / "level-1"
    start.mkdir(parents=True)
    elsewhere = tmp_path / "elsewhere"
    elsewhere.mkdir()

    state = _FakeState(username="u", workspace=workspace, current_level="1.4")
    level = _FakeLevel("1.4", enforce_start_directory=True)
    level.start_directory = "level-1"

    fake_display = _FakeDisplay()
    fake_registry = _FakeRegistry({"1.4": level})

    monkeypatch.setattr(commands, "display", fake_display)
    monkeypatch.setattr(commands, "level_registry", fake_registry)
    monkeypatch.setattr(commands.state_manager, "load", lambda: state)
    monkeypatch.chdir(elsewhere)

    assert commands.reset.callback is not None
    commands.reset.callback()

    assert Path.cwd().resolve() == start.resolve()


def test_repeat_defaults_to_current_level(monkeypatch: Any, tmp_path: Path) -> None:
    state = _FakeState(username="u", workspace=tmp_path / "ws", current_level="1.7")
    level = _FakeLevel("1.7")

    fake_display = _FakeDisplay()
    fake_registry = _FakeRegistry({"1.7": level})

    monkeypatch.setattr(commands, "display", fake_display)
    monkeypatch.setattr(commands, "level_registry", fake_registry)
    monkeypatch.setattr(commands.state_manager, "load", lambda: state)

    assert commands.repeat.callback is not None
    commands.repeat.callback(section_num=None, level_id=None)

    assert fake_display.instructions_calls == ["1.7"]


def test_repeat_by_section_uses_intro_level(monkeypatch: Any, tmp_path: Path) -> None:
    state = _FakeState(username="u", workspace=tmp_path / "ws", current_level="9.9")
    level = _FakeLevel("2.0")

    fake_display = _FakeDisplay()
    fake_registry = _FakeRegistry({"2.0": level})

    monkeypatch.setattr(commands, "display", fake_display)
    monkeypatch.setattr(commands, "level_registry", fake_registry)
    monkeypatch.setattr(commands.state_manager, "load", lambda: state)

    assert commands.repeat.callback is not None
    commands.repeat.callback(section_num=2, level_id=None)

    assert fake_display.instructions_calls == ["2.0"]


def test_repeat_by_level_overrides_section(monkeypatch: Any, tmp_path: Path) -> None:
    state = _FakeState(username="u", workspace=tmp_path / "ws", current_level="9.9")
    level = _FakeLevel("1.1")

    fake_display = _FakeDisplay()
    fake_registry = _FakeRegistry({"1.1": level, "2.0": _FakeLevel("2.0")})

    monkeypatch.setattr(commands, "display", fake_display)
    monkeypatch.setattr(commands, "level_registry", fake_registry)
    monkeypatch.setattr(commands.state_manager, "load", lambda: state)

    assert commands.repeat.callback is not None
    commands.repeat.callback(section_num=2, level_id="1.1")

    assert fake_display.instructions_calls == ["1.1"]
