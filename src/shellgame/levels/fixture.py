"""Declarative workspace fixtures for level setup."""

from __future__ import annotations

import contextlib
import os
import shutil
import stat
from dataclasses import dataclass
from pathlib import Path

from shellgame.paths import relative_path, resolve_within

_KIND = "Fixture"


def _safe_relative_path(value: str) -> Path:
    return relative_path(value, kind=_KIND)


def _fixture_path(root: Path, value: str, *, allow_leaf_symlink: bool = False) -> Path:
    return resolve_within(root, value, kind=_KIND, allow_leaf_symlink=allow_leaf_symlink)


def _safe_rmtree(path: Path) -> None:
    """Recursively remove a directory tree, recovering from restricted permissions."""
    if path.is_symlink():
        path.unlink(missing_ok=True)
        return
    if not path.exists():
        return
    with contextlib.suppress(OSError):
        path.chmod(stat.S_IRWXU)
    for root, dirs, files in os.walk(path, followlinks=False):
        for name in dirs:
            item = Path(root) / name
            if not item.is_symlink():
                with contextlib.suppress(OSError):
                    item.chmod(stat.S_IRWXU)
        for name in files:
            item = Path(root) / name
            if not item.is_symlink():
                with contextlib.suppress(OSError):
                    item.chmod(stat.S_IRWXU)
    shutil.rmtree(path)


def _ensure_directory(root: Path, relative: str) -> None:
    if root.exists() and root.is_dir():
        with contextlib.suppress(OSError):
            root.chmod(stat.S_IRWXU)
    _fixture_path(root, relative)
    root.mkdir(parents=True, exist_ok=True)
    current = Path()
    for part in _safe_relative_path(relative).parts:
        current /= part
        directory = _fixture_path(root, str(current))
        if directory.exists():
            if directory.is_symlink():
                directory.unlink(missing_ok=True)
            elif not directory.is_dir():
                with contextlib.suppress(OSError):
                    directory.chmod(stat.S_IRUSR | stat.S_IWUSR)
                directory.unlink()
            else:
                with contextlib.suppress(OSError):
                    directory.chmod(stat.S_IRWXU)
        directory.mkdir(exist_ok=True)


@dataclass(frozen=True, slots=True)
class FileFixture:
    path: str
    content: str | bytes = ""
    mode: int | None = None
    overwrite: bool = True

    def __post_init__(self) -> None:
        if not _safe_relative_path(self.path).parts:
            raise ValueError("File fixture cannot target its root")

    def apply(self, root: Path) -> None:
        destination = _fixture_path(root, self.path)
        _ensure_directory(root, str(Path(self.path).parent))
        if self.overwrite or not destination.is_file():
            self._replace(destination)
        if self.mode is not None:
            destination.chmod(self.mode)

    def _replace(self, destination: Path) -> None:
        """Restore the file after permission or file/directory mistakes.

        Section 7 teaches `chmod`, so a player can legitimately leave a fixture
        file read-only. Reset has to hand back a clean level regardless of what
        the previous attempt did to it, and a fixture that cannot overwrite its
        own file would fail the one command meant to recover from that.
        """
        if destination.is_symlink():
            destination.unlink(missing_ok=True)
        elif destination.is_dir():
            _safe_rmtree(destination)
        elif destination.exists():
            with contextlib.suppress(OSError):
                destination.chmod(stat.S_IRUSR | stat.S_IWUSR)
            destination.unlink()
        if isinstance(self.content, bytes):
            destination.write_bytes(self.content)
        else:
            destination.write_text(self.content, encoding="utf-8")


@dataclass(frozen=True, slots=True)
class WorkspaceFixture:
    directories: tuple[str, ...] = ()
    files: tuple[FileFixture, ...] = ()
    clean: tuple[str, ...] = ()

    def __post_init__(self) -> None:
        for relative in self.directories:
            _safe_relative_path(relative)
        for relative in self.clean:
            path = _safe_relative_path(relative)
            if not path.parts:
                raise ValueError("Fixture cleanup cannot target its root")

    def apply(self, root: Path) -> None:
        _ensure_directory(root, "")

        for relative in self.clean:
            target = _fixture_path(root, relative, allow_leaf_symlink=True)
            if target.is_symlink():
                target.unlink(missing_ok=True)
            elif target.is_file():
                with contextlib.suppress(OSError):
                    target.chmod(stat.S_IRUSR | stat.S_IWUSR)
                target.unlink(missing_ok=True)
            elif target.is_dir():
                _safe_rmtree(target)

        for relative in self.directories:
            _ensure_directory(root, relative)

        for file_fixture in self.files:
            file_fixture.apply(root)
