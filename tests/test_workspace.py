"""Unit tests for workspace management."""

from pathlib import Path

import pytest

from shellgame.state.manager import StateManager
from shellgame.workspace.builder import WorkspaceManager


class TestWorkspaceManager:
    def test_init_workspace(self, tmp_path: Path) -> None:
        manager = WorkspaceManager(tmp_path / "workspace")

        result = manager.init()

        assert result.is_dir()

    def test_validate_integrity(self, tmp_path: Path) -> None:
        manager = WorkspaceManager(tmp_path / "workspace")
        manager.init()

        assert manager.validate_integrity() is True

    def test_remove(self, tmp_path: Path) -> None:
        manager = WorkspaceManager(tmp_path / "workspace")
        manager.init()

        manager.remove()

        assert not manager.workspace_root.exists()

    def test_remove_rejects_path_outside_temp_directory(self) -> None:
        manager = WorkspaceManager(Path("/"))

        with pytest.raises(RuntimeError, match="unsafe workspace"):
            manager.remove()


def test_workspace_manager_uses_the_path_it_is_given(tmp_path: Path) -> None:
    """Restore and remove must act on `state.workspace`, never on a re-derived path."""
    explicit = tmp_path / "custom-location"

    manager = WorkspaceManager(explicit)
    manager.init()

    assert manager.workspace_root == explicit
    assert explicit.is_dir()


def test_default_workspace_is_derived_in_exactly_one_place() -> None:
    assert StateManager.default_workspace("ada") == Path("/tmp/shellgame-ada")
    assert StateManager().create("ada").workspace == StateManager.default_workspace("ada")
