"""Workspace management for directory structure creation and management."""

import os
import shutil
from pathlib import Path


class FileSpec:
    """Specification for a file to create."""

    def __init__(self, path: str, content: str = "", mode: int = 0o644):
        """
        Initialize file specification.

        Args:
            path: Relative path to file
            content: File content
            mode: File permissions (octal)
        """
        self.path = path
        self.content = content
        self.mode = mode


class DirSpec:
    """Specification for a directory to create."""

    def __init__(self, path: str, mode: int = 0o755):
        """
        Initialize directory specification.

        Args:
            path: Relative path to directory
            mode: Directory permissions (octal)
        """
        self.path = path
        self.mode = mode


class WorkspaceManager:
    """Manages game workspace directory structure."""

    def __init__(self, username: str):
        """
        Initialize workspace manager.

        Args:
            username: Player username
        """
        self.workspace_root = Path(f"/tmp/shellgame-{username}")

    def init(self) -> Path:
        """
        Create workspace directory.

        Returns:
            Path to workspace root
        """
        self.workspace_root.mkdir(parents=True, exist_ok=True)
        return self.workspace_root

    def cleanup(self) -> None:
        """Remove entire workspace."""
        if self.workspace_root.exists():
            shutil.rmtree(self.workspace_root)

    def remove(self) -> None:
        """Remove entire workspace (alias for cleanup)."""
        self.cleanup()

    def build_structure(self, dirs: list[DirSpec], files: list[FileSpec]) -> None:
        """
        Build directory structure with files.

        Args:
            dirs: List of directory specifications
            files: List of file specifications
        """
        # Create directories first
        for dir_spec in dirs:
            dir_path = self.workspace_root / dir_spec.path
            dir_path.mkdir(parents=True, exist_ok=True)
            os.chmod(dir_path, dir_spec.mode)

        # Create files
        for file_spec in files:
            file_path = self.workspace_root / file_spec.path
            file_path.parent.mkdir(parents=True, exist_ok=True)
            file_path.write_text(file_spec.content)
            os.chmod(file_path, file_spec.mode)

    def validate_integrity(self) -> bool:
        """
        Check workspace still exists and is valid.

        Returns:
            True if workspace is valid
        """
        return (
            self.workspace_root.exists()
            and self.workspace_root.is_dir()
            and str(self.workspace_root).startswith("/tmp/")
        )

    def clear_directory(self, relative_path: str) -> None:
        """
        Clear all contents of a directory.

        Args:
            relative_path: Path relative to workspace root
        """
        dir_path = self.workspace_root / relative_path
        if dir_path.exists() and dir_path.is_dir():
            shutil.rmtree(dir_path)
            dir_path.mkdir(parents=True, exist_ok=True)
