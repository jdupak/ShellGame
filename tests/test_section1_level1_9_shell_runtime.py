"""Exercise home-walk grading through the real bash/fish wrappers and CLI."""

from __future__ import annotations

import os
import shlex
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.cli.subshell import get_bash_integration, get_fish_integration
from shellgame.markers import MarkerManager
from shellgame.state.manager import GameState, StateManager


@dataclass
class _HomeWalkShell:
    shell: str
    integration: Path
    home: Path
    state: GameState
    manager: StateManager
    env: dict[str, str]

    @property
    def markers(self) -> MarkerManager:
        return MarkerManager.from_state(self.state)

    def run(self, commands: str) -> subprocess.CompletedProcess[str]:
        flags = ["--no-config"] if self.shell == "fish" else []
        result = subprocess.run(
            [self.shell, *flags, "-c", f"source {shlex.quote(str(self.integration))}; {commands}"],
            cwd=self.state.workspace,
            env=self.env,
            check=False,
            capture_output=True,
            text=True,
            input="\n",
            timeout=30,
        )
        assert result.returncode == 0, result.stdout + result.stderr
        assert "__SHELLGAME_EXEC__" not in result.stdout + result.stderr
        return result

    def probe_move(self, command: str) -> subprocess.CompletedProcess[str]:
        status = "$status" if self.shell == "fish" else "$?"
        return self.run(f"{command}; printf 'CD_STATUS=%s\\n' {status}; command pwd -P; shellgame submit")

    def assert_incomplete(self) -> None:
        assert not self.markers.exists(MarkerManager.LEVEL1_9_CD_WALK_COMPLETED)
        saved = self.manager.load()
        assert saved is not None
        assert saved.current_level == "1.9"
        assert "1.9" not in saved.levels_complete
        assert saved.level_attempts["1.9"] == 1


@pytest.fixture(params=["bash", "fish"])
def home_walk_shell(request: pytest.FixtureRequest, tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> _HomeWalkShell:
    shell = request.param
    if not shutil.which(shell):
        pytest.skip(f"{shell} not installed")

    home = tmp_path / "home" / "student"
    home.mkdir(parents=True)
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    scratch = tmp_path / "shell-scratch"
    scratch.mkdir()
    config = tmp_path / "config"
    monkeypatch.setenv("XDG_CONFIG_HOME", str(config))
    manager = StateManager()
    state = GameState(
        username="student",
        workspace=workspace,
        current_level="1.9",
        start_time=datetime.now(),
        level_started_at={"1.9": datetime.now()},
    )
    manager.save(state)
    env = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("SHELLGAME_", "BASH_FUNC_")) and key not in {"BASH_ENV", "ENV", "CDPATH", "OLDPWD"}
    }
    env.update(
        HOME=str(home),
        XDG_CONFIG_HOME=str(config),
        XDG_DATA_HOME=str(tmp_path / "data"),
        XDG_CACHE_HOME=str(tmp_path / "cache"),
        TMPDIR=str(scratch),
        PYTHONPATH=str(Path(__file__).resolve().parents[1] / "src"),
        SHELLGAME_WRAPPER="1",
        SHELLGAME_AUTO_STARTED="1",
        SHELLGAME_LEVEL="1.9",
        SHELLGAME_WORKSPACE=str(workspace),
    )
    renderer = get_fish_integration if shell == "fish" else get_bash_integration
    integration = tmp_path / f"integration.{shell}"
    integration.write_text(renderer(f"{shlex.quote(sys.executable)} -m shellgame"), encoding="utf-8")
    return _HomeWalkShell(shell, integration, home, state, manager, env)


@pytest.mark.parametrize("target", ["'~'", "'$HOME'"])
def test_quoted_home_names_do_not_expand(home_walk_shell: _HomeWalkShell, target: str) -> None:
    result = home_walk_shell.probe_move(f"cd {target}")

    assert "CD_STATUS=1" in result.stdout
    assert str(home_walk_shell.state.workspace) in result.stdout.splitlines()
    assert not home_walk_shell.markers.exists(MarkerManager.LEVEL1_9_CD_WALK_PROGRESS)
    home_walk_shell.assert_incomplete()


@pytest.mark.parametrize("name", ["~", "$HOME"])
def test_quoted_home_names_can_enter_literal_directories(home_walk_shell: _HomeWalkShell, name: str) -> None:
    literal = home_walk_shell.state.workspace / name
    literal.mkdir()

    result = home_walk_shell.probe_move(f"cd {shlex.quote(name)}")

    assert "CD_STATUS=0" in result.stdout
    assert str(literal) in result.stdout.splitlines()
    home_walk_shell.assert_incomplete()


@pytest.mark.parametrize("command", ["cd", "cd ~", "cd $HOME", 'cd "$HOME"'])
def test_expanded_home_shortcuts_are_blocked(home_walk_shell: _HomeWalkShell, command: str) -> None:
    result = home_walk_shell.probe_move(command)

    assert "CD_STATUS=1" in result.stdout
    assert str(home_walk_shell.state.workspace) in result.stdout.splitlines()
    assert "shellgame reset" in result.stdout + result.stderr
    home_walk_shell.assert_incomplete()


def test_single_segment_symlink_jump_cannot_complete_walk(home_walk_shell: _HomeWalkShell) -> None:
    workspace = home_walk_shell.state.workspace
    (workspace / "shortcut").symlink_to(home_walk_shell.home, target_is_directory=True)
    walk_to_workspace = "; ".join(["cd /", *(f"cd {shlex.quote(part)}" for part in workspace.parts[1:])])
    status = "$status" if home_walk_shell.shell == "fish" else "$?"

    result = home_walk_shell.run(
        f"{walk_to_workspace}; cd shortcut; printf 'CD_STATUS=%s\\n' {status}; command pwd -P; shellgame submit"
    )

    assert "CD_STATUS=0" in result.stdout
    assert str(home_walk_shell.home) in result.stdout.splitlines()
    home_walk_shell.assert_incomplete()


def test_real_root_to_home_walk_completes(home_walk_shell: _HomeWalkShell) -> None:
    walk_home = "; ".join(["cd /", *(f"cd {shlex.quote(part)}" for part in home_walk_shell.home.parts[1:])])

    home_walk_shell.run(f"{walk_home}; shellgame submit")

    assert home_walk_shell.markers.exists(MarkerManager.LEVEL1_9_CD_WALK_COMPLETED)
    saved = home_walk_shell.manager.load()
    assert saved is not None
    assert "1.9" in saved.levels_complete
    assert saved.current_level == "1.10"
