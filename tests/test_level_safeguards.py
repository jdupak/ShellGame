from dataclasses import dataclass
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.base import Level
from shellgame.levels.cdpolicy import cd_marker
from shellgame.levels.completion import Completion, ExactAnswer
from shellgame.levels.sections.section1 import (
    CdUpLevel,
    MultiLevelAscentLevel,
    SummaryLevel,
)
from shellgame.levels.sections.section2 import (
    DeepRelativeNavigationLevel,
    PreviousDirectoryToggleLevel,
)
from shellgame.levels.sections.section9 import SectionSummaryChallengeLevel
from shellgame.levels.sections.section10 import DevNullLevel, StreamsChallengeLevel
from shellgame.levels.sections.section11 import (
    FinalChallengeLevel,
    GrepConfigLineToFileLevel,
)
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


@dataclass(frozen=True)
class _NavigationCase:
    level: Level
    source: str
    destination: str
    target: str
    answer: str | None
    marker_name: str
    needs_pwd: bool = False


class _ExpectedAnswerLevel(Level):
    title = "Test"
    completion = Completion(answer=ExactAnswer("correct", mistakes={"near": "specific feedback"}))


@pytest.mark.parametrize(
    "case",
    [
        _NavigationCase(
            level=CdUpLevel(),
            source="level-1/alpha",
            destination="level-1",
            target="..",
            answer="level-1",
            marker_name=cd_marker("1.4"),
        ),
        _NavigationCase(
            level=MultiLevelAscentLevel(),
            source="level-1/gamma/deep/a/b/c",
            destination="level-1/gamma/deep",
            target="../../..",
            answer=None,
            marker_name=cd_marker("1.6"),
        ),
        _NavigationCase(
            level=PreviousDirectoryToggleLevel(),
            source="level-2/location-B",
            destination="level-2/location-A",
            target="-",
            answer=None,
            marker_name=cd_marker("2.2"),
        ),
        _NavigationCase(
            level=DeepRelativeNavigationLevel(),
            source="level-2/deep/structure/start",
            destination="level-2/deep/other/target",
            target="../../other/target",
            answer=None,
            marker_name=cd_marker("2.3"),
        ),
        _NavigationCase(
            level=SummaryLevel(),
            source="level-1/gamma/deep/a/b/c",
            destination="level-1/gamma",
            target="../../../..",
            answer=None,
            marker_name=cd_marker("1.12"),
            needs_pwd=True,
        ),
    ],
)
def test_navigation_command_evidence_and_exact_destination(
    case: _NavigationCase,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    case.level.prepare(workspace)
    state = _state(workspace)
    markers = MarkerManager(workspace)
    if case.needs_pwd:
        markers.create(MarkerManager.PWD_USED)

    source_path = workspace / case.source
    monkeypatch.chdir(source_path)
    assert case.level.validate(case.answer, state)[0] is False

    hook = case.level.hooks["cd"]
    hook(target=case.target, pwd=str(source_path), post_move=False, state=state)

    assert markers.exists(case.marker_name)

    destination_path = workspace / case.destination
    monkeypatch.chdir(destination_path)
    assert case.level.validate(case.answer, state)[0] is True

    decoy = tmp_path / "outside" / destination_path.name
    decoy.mkdir(parents=True)
    monkeypatch.chdir(decoy)
    assert case.level.validate(case.answer, state)[0] is False


def test_specific_mistake_feedback_runs_before_generic_answer_error(tmp_path: Path) -> None:
    level = _ExpectedAnswerLevel()

    success, message = level.validate("near", _state(tmp_path))

    assert success is False
    assert message == "specific feedback"


def test_dev_null_level_requires_suppressed_execution(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    level = DevNullLevel()
    level.prepare(workspace)
    state = _state(workspace)
    script = workspace / "level-10" / "buggy.sh"

    assert '"$SHELLGAME_FD_HOOK"' in script.read_text()
    level.record_fd_evidence(
        stdout_target="/dev/null",
        stderr_target="/dev/null",
        state=state,
    )

    monkeypatch.chdir(workspace / "level-10")
    assert level.validate("/dev/null", state)[0] is True


def test_dev_null_level_rejects_answer_without_execution(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    level = DevNullLevel()
    level.prepare(workspace)
    state = _state(workspace)

    monkeypatch.chdir(workspace / "level-10")
    assert level.validate("/dev/null", state)[0] is False


def test_grep_level_requires_exact_matching_line(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    level = GrepConfigLineToFileLevel()
    level.prepare(workspace)
    state = _state(workspace)
    level_dir = workspace / "level-11" / "grep"
    config_lines = (level_dir / "config.txt").read_text(encoding="utf-8").splitlines()
    expected_line = next(line for line in config_lines if "ACTIVE_PROFILE" in line)
    target = level_dir / "profile.txt"
    target.write_text(expected_line + "\n", encoding="utf-8")

    monkeypatch.chdir(level_dir)
    assert level.validate("profile.txt", state)[0] is True

    target.write_text(expected_line.split("=", 1)[-1], encoding="utf-8")
    assert level.validate("profile.txt", state)[0] is False


def test_grep_level_reports_missing_source_without_crashing(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    level = GrepConfigLineToFileLevel()
    level.prepare(workspace)
    state = _state(workspace)
    level_dir = workspace / "level-11" / "grep"
    (level_dir / "profile.txt").write_text("anything", encoding="utf-8")
    (level_dir / "config.txt").unlink()

    monkeypatch.chdir(level_dir)
    success, message = level.validate("profile.txt", state)

    assert success is False
    assert "shellgame reset" in message


@pytest.mark.parametrize("path", ["profile.txt", "config.txt"])
@pytest.mark.parametrize("damage", ["directory", "invalid-utf8", "symlink"])
def test_grep_level_rejects_malformed_files_and_reset_restores_them(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, path: str, damage: str
) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    level = GrepConfigLineToFileLevel()
    level.prepare(workspace)
    level_dir = level.section_path(workspace) / "grep"
    state = _state(workspace)
    source = level_dir / "config.txt"
    output = level_dir / "profile.txt"
    output.write_text(
        next(line for line in source.read_text().splitlines() if "ACTIVE_PROFILE" in line),
        encoding="utf-8",
    )
    target = level_dir / path
    target.unlink()
    outside = tmp_path / "untouched.txt"
    outside.write_text("keep", encoding="utf-8")
    if damage == "directory":
        target.mkdir()
        (target / "nested.txt").write_text("mistake", encoding="utf-8")
    elif damage == "invalid-utf8":
        target.write_bytes(b"\xff")
    else:
        target.symlink_to(outside)
    monkeypatch.chdir(level_dir)

    success, message = level.validate("profile.txt", state)
    assert not success
    assert Messages.CWD_MISSING not in message
    assert "shellgame reset" in message or message == Messages.PATH_ESCAPES_WORKSPACE

    level.reset(workspace)
    assert source.is_file() and not source.is_symlink()
    assert not output.exists()
    assert outside.read_text() == "keep"
    assert not level.validate("profile.txt", state)[0]
    assert level.solution is not None
    level.solution.perform(level, state)
    assert level.validate("profile.txt", state)[0]


def test_grep_level_handles_unreadable_output(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    level = GrepConfigLineToFileLevel()
    level.prepare(tmp_path)
    state = _state(tmp_path)
    output = level.section_path(tmp_path) / "grep" / "profile.txt"
    output.write_text("anything", encoding="utf-8")
    original_read = Path.read_text

    def read_text(path: Path, encoding: str | None = None, errors: str | None = None) -> str:
        if path == output:
            raise PermissionError("Permission denied")
        return original_read(path, encoding=encoding, errors=errors)

    monkeypatch.setattr(Path, "read_text", read_text)
    success, message = level.validate("profile.txt", state)
    assert not success
    assert "shellgame reset" in message
    assert Messages.CWD_MISSING not in message


def test_redirection_challenge_requires_created_content(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    level = SectionSummaryChallengeLevel()
    level.prepare(tmp_path)
    state = _state(tmp_path)
    challenge = tmp_path / "level-9" / "challenge"
    monkeypatch.chdir(challenge)

    assert level.validate("3", state)[0] is False

    (challenge / "message.txt").write_text("Hello World\nGoodbye\n", encoding="utf-8")
    assert level.validate("3", state)[0] is True


def test_streams_challenge_requires_both_output_files(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    level = StreamsChallengeLevel()
    level.prepare(tmp_path)
    state = _state(tmp_path)
    challenge = tmp_path / "level-10" / "challenge"
    monkeypatch.chdir(challenge)

    assert level.validate("2,3", state)[0] is False

    (challenge / "errors.log").write_text("error 1\nerror 2\n", encoding="utf-8")
    (challenge / "output.log").write_text("line 1\nline 2\nline 3\n", encoding="utf-8")
    assert level.validate("2,3", state)[0] is True


def test_final_challenge_requires_secret_script_copy(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    level = FinalChallengeLevel()
    level.prepare(tmp_path)
    state = _state(tmp_path)
    final = tmp_path / "level-11" / "final"
    monkeypatch.chdir(final)

    assert level.validate("4,NINJA2024", state)[0] is False

    source = final / "hidden" / "secret.sh"
    (final / "found" / "secret.txt").write_bytes(source.read_bytes())
    assert level.validate("4,NINJA2024", state)[0] is True
