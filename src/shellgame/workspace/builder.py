"""Workspace management for directory structure creation and management."""

import shutil
import tempfile
from pathlib import Path


class WorkspaceManager:
    """Owns one workspace directory.

    The path is always supplied by the caller (normally `GameState.workspace`)
    so that initialising, restoring and removing a workspace can never operate
    on a different directory than the one the player is actually playing in.
    """

    def __init__(self, workspace: Path):
        self.workspace_root = Path(workspace)

    def init(self) -> Path:
        self.workspace_root.mkdir(parents=True, exist_ok=True)
        return self.workspace_root

    def remove(self) -> None:
        if self.workspace_root.exists():
            if not self.validate_integrity():
                raise RuntimeError(f"Refusing to remove unsafe workspace path: {self.workspace_root}")
            shutil.rmtree(self.workspace_root)

    def validate_integrity(self) -> bool:
        temp_root = Path(tempfile.gettempdir()).resolve()
        workspace = self.workspace_root.resolve()
        return workspace != temp_root and workspace.is_relative_to(temp_root)
