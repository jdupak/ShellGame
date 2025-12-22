"""Workspace management for directory structure creation and management."""

import os
import shutil
from pathlib import Path


class FileSpec:
    def __init__(self, path: str, content: str = "", mode: int = 0o644):
        self.path = path
        self.content = content
        self.mode = mode


class DirSpec:
    def __init__(self, path: str, mode: int = 0o755):
        self.path = path
        self.mode = mode


class WorkspaceManager:
    def __init__(self, username: str):
        self.workspace_root = Path(f"/tmp/shellgame-{username}")

    def init(self) -> Path:
        self.workspace_root.mkdir(parents=True, exist_ok=True)
        return self.workspace_root

    def cleanup(self) -> None:
        if self.workspace_root.exists():
            shutil.rmtree(self.workspace_root)

    def remove(self) -> None:
        self.cleanup()

    def build_structure(self, dirs: list[DirSpec], files: list[FileSpec]) -> None:
        for dir_spec in dirs:
            dir_path = self.workspace_root / dir_spec.path
            dir_path.mkdir(parents=True, exist_ok=True)
            os.chmod(dir_path, dir_spec.mode)

        for file_spec in files:
            file_path = self.workspace_root / file_spec.path
            file_path.parent.mkdir(parents=True, exist_ok=True)
            file_path.write_text(file_spec.content)
            os.chmod(file_path, file_spec.mode)

    def validate_integrity(self) -> bool:
        return (
            self.workspace_root.exists()
            and self.workspace_root.is_dir()
            and str(self.workspace_root).startswith("/tmp/")
        )

    def clear_directory(self, relative_path: str) -> None:
        dir_path = self.workspace_root / relative_path
        if dir_path.exists() and dir_path.is_dir():
            shutil.rmtree(dir_path)
            dir_path.mkdir(parents=True, exist_ok=True)
