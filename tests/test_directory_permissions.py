"""Focused tests for declarative directory fixtures and permission requirements."""

from __future__ import annotations

import shutil
import stat
from dataclasses import dataclass
from pathlib import Path

import pytest

from shellgame.levels.base import Level
from shellgame.levels.completion import Completion, DirectoryPermissionMode, PermissionMode
from shellgame.levels.fixture import DirectoryFixture, FileFixture, WorkspaceFixture
from shellgame.messages import Messages


@dataclass
class _State:
    workspace: Path
    username: str = "tester"
    current_level: str = "test"


class _DirectoryFixtureLevel(Level):
    title = "Directory fixture test"
    section_root = "section"
    fixture = WorkspaceFixture(
        directory_fixtures=(DirectoryFixture("project/private", mode=0o500),),
        files=(FileFixture("project/private/config.txt", content="restored"),),
    )
    completion = Completion()


def _mode(path: Path) -> int:
    return stat.S_IMODE(path.stat().st_mode)


def _remove_fixture_tree(path: Path, target: Path) -> None:
    target.chmod(0o700)
    target.parent.chmod(0o700)
    if path != target:
        path.chmod(0o700)
    shutil.rmtree(path)


def test_workspace_fixture_applies_exact_directory_modes_after_child_files(tmp_path: Path) -> None:
    fixture = WorkspaceFixture(
        directory_fixtures=(
            DirectoryFixture("vault", mode=0o000),
            DirectoryFixture("vault/records", mode=0o750),
        ),
        files=(FileFixture("vault/records/report.txt", content="data"),),
    )

    fixture.apply(tmp_path)

    vault = tmp_path / "vault"
    assert _mode(vault) == 0o000
    vault.chmod(0o100)
    assert _mode(vault / "records") == 0o750
    assert (vault / "records/report.txt").read_text(encoding="utf-8") == "data"
    vault.chmod(0o700)


@pytest.mark.parametrize(
    "corruption",
    ["chmod-000", "target-file", "parent-file", "target-symlink", "parent-symlink"],
)
def test_level_reset_repairs_directory_fixture_without_escaping_root(tmp_path: Path, corruption: str) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    level = _DirectoryFixtureLevel()
    level.prepare(workspace)

    parent = workspace / "section/project"
    target = parent / "private"
    outside = tmp_path / "outside"
    outside.mkdir()
    keep = outside / "keep.txt"
    keep.write_text("keep", encoding="utf-8")

    if corruption == "chmod-000":
        target.chmod(0)
        parent.chmod(0)
    elif corruption == "target-file":
        _remove_fixture_tree(target, target)
        target.write_text("wrong type", encoding="utf-8")
    elif corruption == "parent-file":
        _remove_fixture_tree(parent, target)
        parent.write_text("wrong type", encoding="utf-8")
    elif corruption == "target-symlink":
        _remove_fixture_tree(target, target)
        target.symlink_to(outside, target_is_directory=True)
    else:
        _remove_fixture_tree(parent, target)
        parent.symlink_to(outside, target_is_directory=True)

    level.reset(workspace)

    assert target.is_dir()
    assert not target.is_symlink()
    assert _mode(target) == 0o500
    assert (target / "config.txt").read_text(encoding="utf-8") == "restored"
    assert keep.read_text(encoding="utf-8") == "keep"
    assert not (outside / "private").exists()
    assert not (outside / "config.txt").exists()


def test_directory_fixture_rejects_unsafe_paths_and_modes() -> None:
    with pytest.raises(ValueError, match="cannot target its root"):
        DirectoryFixture("", mode=0o755)

    for path in ("../outside", "/tmp/outside"):
        with pytest.raises(ValueError, match="relative"):
            DirectoryFixture(path, mode=0o755)

    for mode in (-1, 0o10000):
        with pytest.raises(ValueError, match="between"):
            DirectoryFixture("private", mode=mode)


def test_directory_fixture_rejects_a_symlinked_root(tmp_path: Path) -> None:
    outside = tmp_path / "outside"
    outside.mkdir()
    root = tmp_path / "root"
    root.symlink_to(outside, target_is_directory=True)

    with pytest.raises(ValueError, match="root cannot be a symlink"):
        DirectoryFixture("private", mode=0o700).apply(root)

    assert not (outside / "private").exists()


def test_directory_permission_mode_accepts_only_the_exact_directory_mode(tmp_path: Path) -> None:
    target = tmp_path / "private"
    target.mkdir()
    target.chmod(0o750)
    state = _State(tmp_path)

    assert DirectoryPermissionMode("private", 0o750).check(state, tmp_path) == (
        True,
        Messages.PERMISSION_CORRECT,
    )

    passed, message = DirectoryPermissionMode("private", 0o700).check(state, tmp_path)
    assert passed is False
    assert message == Messages.PERMISSION_WRONG
    assert "700" not in message
    assert "750" not in message

    assert DirectoryPermissionMode("private", 0o700, error_message="Upravte oprávnění.").check(state, tmp_path) == (
        False,
        "Upravte oprávnění.",
    )


@pytest.mark.parametrize("entry_type", ["missing", "file"])
def test_directory_permission_mode_rejects_missing_or_non_directory_paths(tmp_path: Path, entry_type: str) -> None:
    target = tmp_path / "private"
    if entry_type == "file":
        target.write_text("not a directory", encoding="utf-8")

    assert DirectoryPermissionMode("private", 0o700).check(_State(tmp_path), tmp_path) == (
        False,
        Messages.DIR_NOT_EXISTS.format(path="private"),
    )


def test_directory_permission_mode_validates_path_and_mode_declarations() -> None:
    for path in ("../outside", "/tmp/outside"):
        with pytest.raises(ValueError, match="relative"):
            DirectoryPermissionMode(path, 0o700)

    for mode in (-1, 0o10000):
        with pytest.raises(ValueError, match="between"):
            DirectoryPermissionMode("private", mode)


def test_directory_permission_mode_does_not_follow_symlinks(tmp_path: Path) -> None:
    root = tmp_path / "root"
    outside = tmp_path / "outside"
    root.mkdir()
    outside.mkdir()
    outside.chmod(0o700)
    (root / "private").symlink_to(outside, target_is_directory=True)
    completion = Completion(requirements=(DirectoryPermissionMode("private", 0o700),))

    assert completion.validate(None, _State(root), root=root, success_message="done") == (
        False,
        Messages.PATH_ESCAPES_WORKSPACE,
    )


def test_directory_permission_mode_participates_in_permission_mode_invariants() -> None:
    requirement = DirectoryPermissionMode("private", 0o700)

    assert isinstance(requirement, PermissionMode)
