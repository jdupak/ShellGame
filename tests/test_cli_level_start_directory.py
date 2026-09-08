"""Unit tests for CLI helper functions.

These tests lock in path-mapping behavior so it doesn't regress.
"""

import os
from pathlib import Path
from unittest.mock import MagicMock

from shellgame.cli.commands import level_registry
from shellgame.core.navigation import NavigationManager
from shellgame.levels.sections.section1 import PwdLevel


def test_get_level_start_directory_selected_levels(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"

    # Intro levels start in the workspace root.
    assert level_registry.get("0.0").get_start_directory(workspace) == workspace

    # A representative "starts somewhere specific" level.
    assert level_registry.get("1.4").get_start_directory(workspace) == workspace / "level-1" / "alpha"

    # 1.5 is a navigation task; it must not start at the goal directory.
    assert level_registry.get("1.5").get_start_directory(workspace) == workspace / "level-1"

    # Section intro levels ("x.0") default to "no forced start dir".
    assert level_registry.get("2.0").get_start_directory(workspace) is None

    # Unknown levels also default to "no forced start dir".
    assert level_registry.get("999.9") is None


def test_failed_local_teleport_is_delegated_without_notice(tmp_path: Path, monkeypatch) -> None:
    shell_client = MagicMock()
    teleport_notice = MagicMock()
    navigation = NavigationManager(level_registry, shell_client, teleport_notice)
    destination = tmp_path / "missing"
    monkeypatch.setattr(os, "chdir", MagicMock(side_effect=FileNotFoundError(destination)))

    navigation.maybe_teleport(destination)

    shell_client.cd.assert_called_once_with(destination)
    teleport_notice.assert_not_called()


def test_level1_1_requires_pwd_marker(tmp_path: Path) -> None:
    """Level 1.1 should require evidence that `pwd` was used."""

    # Fake state object with just the fields Level1_1.validate expects.
    class _State:
        def __init__(self, username: str, workspace: Path) -> None:
            self.username = username
            self.workspace = workspace
            self.current_level = "1.1"

    state = _State(username="testuser", workspace=tmp_path / "workspace")
    state.workspace.mkdir()
    marker = state.workspace / ".pwd_used"

    level = PwdLevel()

    ok, msg = level.validate("level-1", state)
    assert ok is False
    assert "pwd" in msg

    marker.parent.mkdir(parents=True, exist_ok=True)
    marker.write_text("")
    try:
        ok2, _ = level.validate("level-1", state)
        assert ok2 is True
    finally:
        marker.unlink(missing_ok=True)


def test_level1_1_wrong_answer_does_not_spoil_expected(tmp_path: Path) -> None:
    """If the pwd marker exists, a wrong answer should fail without revealing the expected value."""

    class _State:
        def __init__(self, username: str, workspace: Path) -> None:
            self.username = username
            self.workspace = workspace
            self.current_level = "1.1"

    state = _State(username="testuser", workspace=tmp_path / "workspace")
    state.workspace.mkdir()
    marker = state.workspace / ".pwd_used"
    marker.write_text("")

    level = PwdLevel()
    try:
        ok, msg = level.validate("shellgame-viper", state)
        assert ok is False
        assert "Očekáváno" not in msg
        assert "level-1" not in msg
    finally:
        marker.unlink(missing_ok=True)
