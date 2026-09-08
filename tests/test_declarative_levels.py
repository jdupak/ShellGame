import shutil
import stat
from collections.abc import Callable
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    AtDirectory,
    ChoiceAnswer,
    Completion,
    DirectoryExists,
    Evidence,
    ExactAnswer,
    FileExists,
    FileLineCount,
    IntegerAnswer,
    IntegerRangeAnswer,
    OrderedListAnswer,
    PathMoved,
    PathsMatch,
    PermissionBits,
    PermissionMode,
    SuffixAnswer,
    TextFileContent,
    TupleAnswer,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import get_registry
from shellgame.markers import MarkerManager
from shellgame.messages import Messages
from shellgame.state.manager import GameState


def _state(workspace: Path) -> GameState:
    return GameState(
        username="tester",
        workspace=workspace,
        current_level="test",
        start_time=datetime.now(),
    )


class _FixtureLevel(Level):
    title = "Fixture test"
    completion = Completion()
    section_root = "section"
    section_fixture = WorkspaceFixture(
        files=(
            FileFixture("ordered.txt", "section"),
            FileFixture("persistent.txt", "initial", overwrite=False),
        ),
    )
    fixture = WorkspaceFixture(files=(FileFixture("ordered.txt", "level"),))


def test_level_prepare_applies_section_then_level_fixture(tmp_path: Path) -> None:
    level = _FixtureLevel()

    level.prepare(tmp_path)

    root = tmp_path / "section"
    assert (root / "ordered.txt").read_text(encoding="utf-8") == "level"


def test_fixture_overwrite_policy_preserves_shared_state(tmp_path: Path) -> None:
    level = _FixtureLevel()
    level.prepare(tmp_path)
    root = tmp_path / "section"
    (root / "persistent.txt").write_text("player change", encoding="utf-8")
    (root / "ordered.txt").write_text("player change", encoding="utf-8")

    level.prepare(tmp_path)

    assert (root / "persistent.txt").read_text(encoding="utf-8") == "player change"
    assert (root / "ordered.txt").read_text(encoding="utf-8") == "level"


@pytest.mark.parametrize("overwrite", [False, True])
@pytest.mark.parametrize("content", ["restored", b"restored"])
def test_file_fixture_replaces_an_accidental_directory(tmp_path: Path, overwrite: bool, content: str | bytes) -> None:
    root = tmp_path / "fixture"
    root.mkdir()
    target = root / "input.txt"
    target.mkdir()
    (target / "mistake.txt").write_text("mistake", encoding="utf-8")
    outside = tmp_path / "outside"
    outside.mkdir()
    keep = outside / "keep.txt"
    keep.write_text("keep", encoding="utf-8")
    (target / "link").symlink_to(outside, target_is_directory=True)

    FileFixture("input.txt", content, overwrite=overwrite, mode=0o640).apply(root)

    assert target.is_file()
    assert target.read_bytes() == b"restored"
    assert stat.S_IMODE(target.stat().st_mode) == 0o640
    assert keep.read_text(encoding="utf-8") == "keep"


@pytest.mark.parametrize("blocker", ["nested", "nested/deep"])
@pytest.mark.parametrize("file_fixture", [False, True])
def test_fixture_replaces_files_blocking_expected_directories(tmp_path: Path, blocker: str, file_fixture: bool) -> None:
    target = tmp_path / blocker
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text("wrong type", encoding="utf-8")

    if file_fixture:
        FileFixture("nested/deep/input.txt", "restored").apply(tmp_path)
        assert (tmp_path / "nested/deep/input.txt").read_text(encoding="utf-8") == "restored"
    else:
        WorkspaceFixture(directories=("nested/deep",)).apply(tmp_path)
    assert (tmp_path / "nested/deep").is_dir()


@pytest.mark.parametrize("path", ["", ".", "./"])
def test_file_fixture_cannot_replace_its_root(path: str) -> None:
    with pytest.raises(ValueError, match="root"):
        FileFixture(path, "data")


@pytest.mark.parametrize("file_fixture", [False, True])
def test_fixture_does_not_replace_a_file_used_as_its_root(tmp_path: Path, file_fixture: bool) -> None:
    root = tmp_path / "root"
    root.write_text("keep", encoding="utf-8")
    with pytest.raises(OSError):
        if file_fixture:
            FileFixture("input.txt", "data").apply(root)
        else:
            WorkspaceFixture(directories=("nested",)).apply(root)
    assert root.read_text(encoding="utf-8") == "keep"


def test_fixture_cleanup_removes_files_directories_and_symlinks(tmp_path: Path) -> None:
    root = tmp_path / "root"
    (root / "directory").mkdir(parents=True)
    (root / "directory" / "file.txt").write_text("data", encoding="utf-8")
    (root / "file.txt").write_text("data", encoding="utf-8")
    (root / "link").symlink_to(root / "file.txt")

    WorkspaceFixture(clean=("directory", "file.txt", "link")).apply(root)

    assert not (root / "directory").exists()
    assert not (root / "file.txt").exists()
    assert not (root / "link").exists()


def test_fixture_cleanup_and_replace_recovers_from_chmod_000(tmp_path: Path) -> None:
    root = tmp_path / "root"
    root.mkdir()
    restricted = root / "challenge" / "myproject" / "src"
    restricted.mkdir(parents=True)
    (restricted / "file.txt").write_text("code", encoding="utf-8")
    (restricted / "file.txt").chmod(0)
    restricted.chmod(0)
    (root / "challenge" / "myproject").chmod(0)

    WorkspaceFixture(clean=("challenge",)).apply(root)
    assert not (root / "challenge").exists()

    accidental_dir = root / "input.txt"
    accidental_dir.mkdir()
    (accidental_dir / "child").mkdir()
    (accidental_dir / "child").chmod(0)
    accidental_dir.chmod(0)

    FileFixture("input.txt", "restored").apply(root)
    assert (root / "input.txt").is_file()
    assert (root / "input.txt").read_text(encoding="utf-8") == "restored"


def test_level_prepare_recovers_from_corrupted_section_root(tmp_path: Path) -> None:
    initialize_levels()
    level = get_registry().get("3.2")
    assert level is not None

    # Normal prepare
    level.prepare(tmp_path)
    secret = tmp_path / "level-3" / ".secret_config"
    assert secret.is_file()

    # Case 1: section root made chmod 000
    (tmp_path / "level-3").chmod(0)
    level.prepare(tmp_path)
    assert secret.is_file()

    # Case 2: section root replaced by a file
    shutil.rmtree(tmp_path / "level-3")
    (tmp_path / "level-3").write_text("i am a file")
    level.prepare(tmp_path)
    assert secret.is_file()

    # Case 3: section root replaced by a symlink is rejected for safety
    shutil.rmtree(tmp_path / "level-3")
    (tmp_path / "level-3").symlink_to(tmp_path / "nonexistent")
    with pytest.raises(ValueError, match="Section root"):
        level.prepare(tmp_path)


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("directories", ("../outside",)),
        ("clean", ("",)),
    ],
)
def test_fixture_rejects_unsafe_paths(field: str, value: tuple[str, ...]) -> None:
    with pytest.raises(ValueError):
        WorkspaceFixture(**{field: value})


def test_file_fixture_rejects_absolute_path() -> None:
    with pytest.raises(ValueError):
        FileFixture("/tmp/outside")


def test_file_fixture_rejects_symlink_escape(tmp_path: Path) -> None:
    root = tmp_path / "root"
    outside = tmp_path / "outside"
    root.mkdir()
    outside.mkdir()
    (root / "link").symlink_to(outside, target_is_directory=True)

    with pytest.raises(ValueError):
        FileFixture("link/file.txt", "data").apply(root)

    assert not (outside / "file.txt").exists()


@pytest.mark.parametrize("file_fixture", [False, True])
def test_fixture_repair_does_not_replace_or_follow_leaf_symlinks(tmp_path: Path, file_fixture: bool) -> None:
    root = tmp_path / "root"
    root.mkdir()
    outside = tmp_path / "outside"
    outside.mkdir()
    keep = outside / "keep.txt"
    keep.write_text("keep", encoding="utf-8")
    link = root / "link"
    link.symlink_to(outside, target_is_directory=True)

    with pytest.raises(ValueError, match="Fixture"):
        if file_fixture:
            FileFixture("link", "replacement").apply(root)
        else:
            WorkspaceFixture(directories=("link",)).apply(root)

    assert link.is_symlink()
    assert keep.read_text(encoding="utf-8") == "keep"


def test_workspace_fixture_rejects_symlinked_root(tmp_path: Path) -> None:
    outside = tmp_path / "outside"
    outside.mkdir()
    root = tmp_path / "root"
    root.symlink_to(outside, target_is_directory=True)

    with pytest.raises(ValueError, match="root cannot be a symlink"):
        WorkspaceFixture(files=(FileFixture("file.txt", "data"),)).apply(root)

    assert not (outside / "file.txt").exists()


def test_fixture_cleanup_unlinks_external_symlink_without_following_it(
    tmp_path: Path,
) -> None:
    root = tmp_path / "root"
    outside = tmp_path / "outside"
    root.mkdir()
    outside.mkdir()
    target = outside / "keep.txt"
    target.write_text("keep", encoding="utf-8")
    (root / "link").symlink_to(target)

    WorkspaceFixture(clean=("link",)).apply(root)

    assert not (root / "link").exists()
    assert target.read_text(encoding="utf-8") == "keep"


def test_completion_requirements_are_unconditional(tmp_path: Path) -> None:
    completion = Completion(
        answer=ExactAnswer("correct"),
        requirements=(Evidence("required", "missing evidence"),),
        allow_empty=True,
    )
    state = _state(tmp_path)

    assert completion.validate("correct", state, root=tmp_path, success_message="done") == (
        False,
        "missing evidence",
    )
    assert completion.validate(None, state, root=tmp_path, success_message="done") == (
        False,
        "missing evidence",
    )

    MarkerManager(tmp_path).create("required")
    assert completion.validate(None, state, root=tmp_path, success_message="done") == (True, "done")


def test_completion_allows_empty_answer_only_when_requirement_matches(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    target = tmp_path / "target"
    target.mkdir()
    completion = Completion(
        answer=ExactAnswer("target"),
        allow_empty_when=AtDirectory("target"),
    )
    state = _state(tmp_path)
    monkeypatch.chdir(tmp_path)

    assert completion.validate("target", state, root=tmp_path, success_message="done") == (True, "done")
    assert completion.validate(None, state, root=tmp_path, success_message="done")[0] is False

    monkeypatch.chdir(target)
    assert completion.validate(None, state, root=tmp_path, success_message="done") == (True, "done")


def test_directory_feedback_is_non_revealing_and_keeps_custom_messages(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    root = tmp_path / "workspace"
    target = root / "target"
    target.mkdir(parents=True)
    same_named_directory = tmp_path / "outside" / "target"
    same_named_directory.mkdir(parents=True)
    state = _state(root)
    monkeypatch.chdir(same_named_directory)

    assert AtDirectory("target").check(state, root) == (False, Messages.WRONG_DIRECTORY)
    assert AtDirectory("target", error_message="Follow the route.").check(state, root) == (
        False,
        "Follow the route.",
    )

    monkeypatch.chdir(target)
    assert AtDirectory("target").check(state, root)[0]


def test_tuple_answer_parses_parts_and_preserves_specific_feedback() -> None:
    answer = TupleAnswer(
        (
            ExactAnswer("alpha", mistakes={"alfa": "Použijte anglický zápis."}),
            IntegerAnswer(3),
        )
    )

    assert answer.validate("alpha, 3")[0] is True
    assert answer.validate("alpha")[0] is False
    assert answer.validate("alfa, 3") == (False, "Použijte anglický zápis.")
    assert answer.validate("alpha, three")[0] is False


def test_prepare_clears_declarative_evidence(tmp_path: Path) -> None:
    class EvidenceLevel(Level):
        title = "Evidence test"
        completion = Completion(requirements=(Evidence("temporary"),))

    markers = MarkerManager(tmp_path)
    markers.create("temporary")

    EvidenceLevel().prepare(tmp_path)

    assert not markers.exists("temporary")


def test_regular_level_requires_completion_or_custom_validation() -> None:
    class IncompleteLevel(Level):
        title = "Incomplete"

    with pytest.raises(ValueError, match="require Completion"):
        IncompleteLevel()


def test_level_prepare_rejects_symlinked_section_root(tmp_path: Path) -> None:
    class EscapingLevel(Level):
        title = "Escaping"
        completion = Completion()
        section_root = "section"
        fixture = WorkspaceFixture(files=(FileFixture("created.txt", "data"),))

    workspace = tmp_path / "workspace"
    outside = tmp_path / "outside"
    workspace.mkdir()
    outside.mkdir()
    (workspace / "section").symlink_to(outside, target_is_directory=True)

    with pytest.raises(ValueError, match="Section root"):
        EscapingLevel().prepare(workspace)

    assert not (outside / "created.txt").exists()


def test_section_assigns_local_stable_ids_and_rejects_duplicates() -> None:
    section = Section(12)

    @section.level(3)
    class RegisteredLevel(Level):
        title = "Registered"
        completion = Completion()

    assert RegisteredLevel.id == "12.3"
    assert RegisteredLevel.section == 12
    assert section.levels[0].id == "12.3"

    with pytest.raises(ValueError, match="Duplicate level ID"):

        @section.level(3)
        class DuplicateLevel(Level):
            title = "Duplicate"
            completion = Completion()


def test_section_rejects_unsafe_root() -> None:
    with pytest.raises(ValueError, match="workspace-relative"):
        Section(1, root="../outside")


def test_common_answer_rules() -> None:
    assert ChoiceAnswer(("yes", "ano"), case_sensitive=False).validate("ANO")[0] is True
    assert IntegerRangeAnswer(1, 5, "range").validate("3")[0] is True
    assert IntegerRangeAnswer(1, 5, "range").validate("6") == (False, "range")
    assert IntegerRangeAnswer(1, 5).validate("6") == (False, Messages.INCORRECT)
    assert OrderedListAnswer(("a", "b")).validate("a, b")[0] is True
    assert SuffixAnswer("path/file.txt").validate("/tmp/path/file.txt")[0] is True
    assert SuffixAnswer("path/file.txt").validate("wrong-path/file.txt")[0] is False


@pytest.mark.parametrize(
    "factory",
    [
        lambda: AtDirectory("../outside"),
        lambda: FileExists("/tmp/outside"),
        lambda: TextFileContent("../outside"),
        lambda: PathsMatch("source", "../outside"),
    ],
)
def test_completion_rules_reject_unsafe_paths(factory: Callable[[], object]) -> None:
    with pytest.raises(ValueError):
        factory()


def test_filesystem_requirements(tmp_path: Path) -> None:
    state = _state(tmp_path)
    source = tmp_path / "source"
    destination = tmp_path / "destination"
    source.mkdir()
    destination.mkdir()
    (source / "file.txt").write_text("first\nsecond\n", encoding="utf-8")
    (destination / "file.txt").write_text("first\nsecond\n", encoding="utf-8")
    moved = tmp_path / "moved.txt"
    moved.write_text("moved", encoding="utf-8")

    assert DirectoryExists("source").check(state, tmp_path)[0] is True
    assert FileExists("source/file.txt").check(state, tmp_path)[0] is True
    assert TextFileContent("source/file.txt", contains=("first", "second")).check(state, tmp_path)[0] is True
    assert FileLineCount("source/file.txt", 2, "wrong lines").check(state, tmp_path)[0] is True
    assert PathsMatch("source", "destination").check(state, tmp_path)[0] is True
    assert PathMoved("old.txt", "moved.txt").check(state, tmp_path)[0] is True

    (destination / "extra.txt").write_text("extra", encoding="utf-8")
    assert PathsMatch("source", "destination").check(state, tmp_path)[0] is False


def test_filesystem_requirements_should_not_exist(tmp_path: Path) -> None:
    state = _state(tmp_path)

    # Completely absent path passes should_exist=False
    assert FileExists("absent.txt", should_exist=False).check(state, tmp_path)[0] is True
    assert DirectoryExists("absent_dir", should_exist=False).check(state, tmp_path)[0] is True

    # Regular file exists: both should fail should_exist=False
    (tmp_path / "actual_file.txt").write_text("data", encoding="utf-8")
    assert FileExists("actual_file.txt", should_exist=False).check(state, tmp_path)[0] is False
    assert DirectoryExists("actual_file.txt", should_exist=False).check(state, tmp_path)[0] is False

    # Directory exists: both should fail should_exist=False
    (tmp_path / "actual_dir").mkdir()
    assert FileExists("actual_dir", should_exist=False).check(state, tmp_path)[0] is False
    assert DirectoryExists("actual_dir", should_exist=False).check(state, tmp_path)[0] is False


def test_completion_rejects_symlinked_requirement_paths(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    outside = tmp_path / "outside"
    outside.mkdir()
    (outside / "file.txt").write_text("data", encoding="utf-8")
    (tmp_path / "link").symlink_to(outside, target_is_directory=True)
    state = _state(tmp_path)
    completion = Completion(
        requirements=(
            FileExists("link/file.txt"),
            AtDirectory("link"),
        )
    )
    monkeypatch.chdir(outside)

    success, message = completion.validate(None, state, root=tmp_path, success_message="done")

    assert success is False
    assert "pracovní prostor" in message


def test_permission_requirements(tmp_path: Path) -> None:
    target = tmp_path / "script.sh"
    target.write_text("#!/bin/sh\n", encoding="utf-8")
    target.chmod(0o755)
    state = _state(tmp_path)

    assert PermissionMode("script.sh", 0o755).check(state, tmp_path)[0] is True
    wrong_mode, wrong_message = PermissionMode("script.sh", 0o644).check(state, tmp_path)
    assert wrong_mode is False
    assert "644" not in wrong_message
    assert "755" not in wrong_message
    assert PermissionBits("script.sh", required=stat.S_IXUSR).check(state, tmp_path)[0] is True
    assert PermissionBits("script.sh", forbidden=stat.S_IWOTH).check(state, tmp_path)[0] is True


def test_fixture_overwrites_a_file_the_player_made_read_only(tmp_path: Path) -> None:
    """Reset must restore a level even after the player revoked write permission.

    Section 7 teaches `chmod`, so leaving a fixture file read-only is a correct
    solution, not misuse. Before this was handled, re-applying the fixture raised
    `PermissionError` and `shellgame reset` could never hand the level back.
    """
    fixture = WorkspaceFixture(files=(FileFixture("permissions/config.readonly", content="Do not touch", mode=0o644),))
    fixture.apply(tmp_path)

    target = tmp_path / "permissions" / "config.readonly"
    target.write_text("player edit", encoding="utf-8")
    target.chmod(0o444)

    fixture.apply(tmp_path)

    assert target.read_text(encoding="utf-8") == "Do not touch"
    assert target.stat().st_mode & 0o777 == 0o644
