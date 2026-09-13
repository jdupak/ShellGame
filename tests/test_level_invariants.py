"""Cross-section invariants for declarative level configuration."""

from datetime import datetime
from pathlib import Path
from typing import Any

import pytest

from shellgame.levels.base import Level
from shellgame.levels.completion import AtDirectory
from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import get_registry
from shellgame.markers import MarkerManager
from shellgame.messages import Messages
from shellgame.state.manager import GameState


def _all_levels() -> list[Level]:
    initialize_levels()
    return get_registry().list_levels()


@pytest.mark.parametrize("level", _all_levels(), ids=lambda level: level.id)
def test_level_setup_creates_declared_start_directory(level: Level, tmp_path: Path) -> None:
    workspace = tmp_path / str(level.id).replace(".", "_")
    workspace.mkdir()

    level.prepare(workspace)

    start_directory = level.get_start_directory(workspace)
    if start_directory is not None:
        assert start_directory.is_dir()


@pytest.mark.parametrize(
    "level",
    [
        level
        for level in _all_levels()
        if any(fixture and fixture.files for fixture in (level.section_fixture, level.fixture))
    ],
    ids=lambda level: level.id,
)
def test_reset_repairs_declared_files_replaced_by_directories(level: Level, tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    level.prepare(workspace)
    root = level.section_path(workspace)
    paths = {
        root / file.path
        for fixture in (level.section_fixture, level.fixture)
        if fixture is not None
        for file in fixture.files
    }
    for target in paths:
        assert target.is_file()
        target.unlink()
        target.mkdir()
        (target / "mistake.txt").write_text("wrong type", encoding="utf-8")

    level.reset(workspace)

    assert all(target.is_file() for target in paths)


def test_generated_log_reset_repairs_directory_in_place_of_file(tmp_path: Path) -> None:
    level = next(level for level in _all_levels() if level.id == "5.4")
    level.prepare(tmp_path)
    log = level.section_path(tmp_path) / "logs" / "server.log"
    expected = log.read_bytes()
    log.unlink()
    log.mkdir()
    (log / "mistake.txt").write_text("wrong type", encoding="utf-8")

    level.reset(tmp_path)

    assert log.is_file()
    assert log.read_bytes() == expected


@pytest.mark.parametrize("level", _all_levels(), ids=lambda level: level.id)
def test_completion_directories_are_created(level: Level, tmp_path: Path) -> None:
    if level.completion is None:
        return

    requirements = [*level.completion.requirements]
    if level.completion.allow_empty_when is not None:
        requirements.append(level.completion.allow_empty_when)
    directory_requirements = [requirement for requirement in requirements if isinstance(requirement, AtDirectory)]
    if not directory_requirements:
        return

    workspace = tmp_path / str(level.id).replace(".", "_")
    workspace.mkdir()
    level.prepare(workspace)
    root = level.section_path(workspace)
    assert all((root / requirement.path).is_dir() for requirement in directory_requirements)


@pytest.mark.parametrize("level", _all_levels(), ids=lambda level: level.id)
def test_fresh_regular_level_rejects_empty_submit(
    level: Level, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    if level.is_intro:
        return

    workspace = tmp_path / str(level.id).replace(".", "_")
    workspace.mkdir()
    level.prepare(workspace)
    monkeypatch.chdir(workspace)
    state = GameState(
        username="tester",
        workspace=workspace,
        current_level=str(level.id),
        start_time=datetime.now(),
    )

    success, _ = level.validate(None, state)

    assert success is False


@pytest.mark.parametrize(
    "level",
    [
        level
        for level in _all_levels()
        if level.reset_markers or (level.completion and level.completion.evidence_markers)
    ],
    ids=lambda level: level.id,
)
def test_level_prepare_clears_all_command_evidence(level: Level, tmp_path: Path) -> None:
    workspace = tmp_path / str(level.id).replace(".", "_")
    workspace.mkdir()
    markers = MarkerManager(workspace)
    completion_markers = level.completion.evidence_markers if level.completion else ()
    marker_names = [name for name in (*level.reset_markers, *completion_markers) if name]
    for marker_name in marker_names:
        markers.create(marker_name)

    level.prepare(workspace)

    assert all(not markers.exists(marker_name) for marker_name in marker_names)


@pytest.mark.parametrize("level", _all_levels(), ids=lambda level: level.id)
def test_level_instructions_use_supported_submit_syntax(level: Level) -> None:
    assert "shellgame submit -f" not in level.instructions


def test_registered_level_ids_are_unique() -> None:
    levels = _all_levels()
    assert len({level.id for level in levels}) == len(levels)


def test_no_level_is_optional_or_extension() -> None:
    levels = _all_levels()
    for level in levels:
        assert not level.optional, f"Level {level.id} has optional=True"
        assert not level.extension, f"Level {level.id} has extension=True"
        assert not level.is_bonus, f"Level {level.id} has is_bonus=True"


@pytest.mark.parametrize("level", _all_levels(), ids=lambda level: level.id)
def test_intro_metadata_matches_public_id(level: Level) -> None:
    assert level.is_intro is str(level.id).endswith(".0")


@pytest.mark.parametrize("level", _all_levels(), ids=lambda level: level.id)
def test_regular_levels_have_declarative_or_custom_completion(level: Level) -> None:
    if level.is_intro:
        return

    assert level.completion is not None or type(level).validate is not Level.validate


def test_default_answer_feedback_never_reveals_the_answer() -> None:
    """A rule the author left unconfigured must reject without giving the game away.

    Integer and list answers used to fall back to "Očekáváno X, obdrženo Y",
    which handed the solution to any player who guessed once. The default is now
    uniformly terse; an author who wants to narrow the search writes the hint
    themselves.
    """
    initialize_levels()
    offenders: list[str] = []

    for level in get_registry().list_levels():
        completion = level.completion
        if completion is None or completion.answer is None:
            continue
        for rule in _answer_rules(completion.answer):
            expected = getattr(rule, "expected", None)
            if expected is None or getattr(rule, "error_message", None) is not None:
                continue
            wrong = "9999999" if isinstance(expected, int) else "\u017e\u017e\u017e-\u0161patn\u011b"
            passed, message = rule.validate(wrong)
            assert not passed
            if message != Messages.INCORRECT:
                offenders.append(f"{level.id}: {message}")

    assert not offenders, "These rules use a default rejection that is not the terse one:\n  " + "\n  ".join(offenders)


def _answer_rules(rule: Any) -> list[Any]:
    parts = getattr(rule, "parts", None)
    if parts is None:
        return [rule]
    return [nested for part in parts for nested in _answer_rules(part)]
