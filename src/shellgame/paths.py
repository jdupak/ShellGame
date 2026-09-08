"""Filesystem location helpers that tolerate a deleted working directory.

A player can delete the directory they are standing in (sections 3 and 4 teach
`rm`, `rmdir` and `mv`). After that, `getcwd()` fails for every process the shell
spawns, so `Path.cwd()` raises. Game rules must report that as a normal,
recoverable failure instead of crashing.
"""

from __future__ import annotations

from pathlib import Path
from typing import Final


def current_directory() -> Path | None:
    """Return the resolved working directory, or None when it no longer exists."""
    try:
        return Path.cwd().resolve()
    except (OSError, RuntimeError):
        return None


class WorkspaceRoot:
    """Sentinel for the rare level that starts at the workspace root.

    Level-declared paths are relative to the level's section root, so an empty
    path means "the section root". The handful of levels that deliberately place
    the player above their own section (the game "home") say so explicitly
    rather than by overloading the empty string.
    """

    __slots__ = ()

    def __repr__(self) -> str:
        return "WORKSPACE_ROOT"


WORKSPACE_ROOT: Final = WorkspaceRoot()


class ContainedPathError(ValueError):
    """A level-declared path does not stay inside the root it is resolved against."""


def relative_path(value: str, *, kind: str) -> Path:
    """Reject anything that is not a plain relative path.

    Level authors declare paths, not locations: an absolute path or a `..`
    segment would let a level read or write outside the workspace the game owns.
    """
    path = Path(value)
    if path.is_absolute() or ".." in path.parts:
        raise ContainedPathError(f"{kind} path must be relative to the section root: {value!r}")
    return path


def resolve_within(
    root: Path,
    value: str,
    *,
    kind: str,
    allow_leaf_symlink: bool = False,
) -> Path:
    """Resolve a level-declared path against `root`, refusing to escape it.

    Fixtures, requirements, start directories and solution steps all speak the
    same vocabulary and enforce the same containment, so a level can never
    disagree with itself about where it lives - nor reach outside the workspace
    by way of `..`, an absolute path, or a symlinked parent.
    """
    if root.is_symlink():
        raise ContainedPathError(f"{kind} root cannot be a symlink")
    relative = relative_path(value, kind=kind)
    candidate = root / relative
    containment_target = candidate.parent if allow_leaf_symlink and candidate.is_symlink() else candidate
    try:
        root_resolved = root.resolve()
        candidate_resolved = containment_target.resolve()
    except (OSError, RuntimeError) as error:
        raise ContainedPathError(f"{kind} path cannot be resolved: {value!r}") from error
    if not candidate_resolved.is_relative_to(root_resolved):
        raise ContainedPathError(f"{kind} path escapes its root: {value!r}")

    parts = relative.parts[:-1] if allow_leaf_symlink else relative.parts
    current = root
    for part in parts:
        current /= part
        if current.is_symlink():
            raise ContainedPathError(f"{kind} path contains a symlink: {value!r}")
    return candidate
