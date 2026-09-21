"""Grade answers obtained from the commands actually taught by discovery levels."""

import os
import shutil
import subprocess
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.base import Level
from shellgame.levels.sections.section1 import LsLevel
from shellgame.levels.sections.section2 import HelpDiscoveryLevel
from shellgame.levels.sections.section3 import HiddenDirCountLevel, HiddenFilesSummaryChallengeLevel
from shellgame.levels.sections.section5 import FileDetectiveChallengeLevel
from shellgame.levels.sections.section8 import MakeExecutableLevel, MakeReadOnlyLevel, PermissionsChallengeLevel
from shellgame.levels.sections.section9 import WordAndLineCountLevel
from shellgame.levels.sections.section11 import GrepConfigLineToFileLevel, RecursiveGrepFindFileLevel
from shellgame.levels.solution import RunShell
from shellgame.state.manager import GameState


@pytest.fixture
def command_env(tmp_path: Path) -> dict[str, str]:
    home = tmp_path / "home"
    home.mkdir()
    scratch = tmp_path / "shell-scratch"
    scratch.mkdir()
    env = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("SHELLGAME_", "BASH_FUNC_"))
        and key not in {"BASH_ENV", "ENV", "BASHOPTS", "SHELLOPTS", "CDPATH", "OLDPWD"}
    }
    env.update(
        HOME=str(home),
        XDG_CONFIG_HOME=str(home / "config"),
        XDG_DATA_HOME=str(home / "data"),
        XDG_CACHE_HOME=str(home / "cache"),
        TMPDIR=str(scratch),
        LC_ALL="C",
    )
    return env


def _run(args: list[str], start: Path, env: dict[str, str]) -> str:
    result = subprocess.run(args, cwd=start, env=env, capture_output=True, text=True, check=True, timeout=10)
    assert not result.stderr
    return result.stdout


def _state(workspace: Path, level: Level) -> GameState:
    return GameState(username="tester", workspace=workspace, current_level=level.id, start_time=datetime.now())


@pytest.mark.skipif(shutil.which("file") is None, reason="file not installed")
def test_file_detective_answer_can_be_derived_from_file_output(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, command_env: dict[str, str]
) -> None:
    level = FileDetectiveChallengeLevel()
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    state = _state(workspace, level)
    assert "bez skriptů" in level.instructions

    for prepare in (level.prepare, level.reset):
        prepare(workspace)
        start = level.get_start_directory(workspace)
        assert start is not None
        monkeypatch.chdir(start)
        names = sorted(path.name for path in start.iterdir())
        descriptions = _run(["file", "-b", "--", *names], start, command_env).splitlines()
        assert len(names) == len(descriptions) == 5
        plain = sum(description.startswith("ASCII text") for description in descriptions)
        images = sum("image" in description for description in descriptions)
        scripts = [
            name for name, description in zip(names, descriptions, strict=True) if "Python script" in description
        ]
        assert len(scripts) == 1
        assert images == 1
        assert sum("ASCII text" in description for description in descriptions) > plain
        success, message = level.validate(f"{plain},{images},{scripts[0]}", state)
        assert success, message

        (start / "config.txt").write_bytes(b"not an image")
        (start / "extra.txt").write_text("extra", encoding="utf-8")


@pytest.mark.parametrize(
    "level_type", [HiddenDirCountLevel, HiddenFilesSummaryChallengeLevel], ids=lambda level: level.id
)
def test_hidden_counts_are_distinguishable_and_resettable(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, command_env: dict[str, str], level_type: type[Level]
) -> None:
    level = level_type()
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    state = _state(workspace, level)
    assert "`ls -la`" in level.instructions

    for prepare in (level.prepare, level.reset):
        prepare(workspace)
        start = level.get_start_directory(workspace)
        assert start is not None
        monkeypatch.chdir(start)
        listing = _run(["ls", "-la"], start, command_env).splitlines()
        entries = [parts for line in listing if len(parts := line.split(maxsplit=8)) == 9]
        hidden = [parts for parts in entries if parts[8].startswith(".") and parts[8] not in {".", ".."}]
        directories = sum(parts[0].startswith("d") for parts in hidden)
        files = len(hidden) - directories
        suffix = ""
        if isinstance(level, HiddenFilesSummaryChallengeLevel):
            suffix = "," + _run(["cat", ".secret_code"], start, command_env).strip()
        assert files != directories
        assert level.validate(f"{directories}{suffix}", state)[0]
        assert not level.validate(f"{files}{suffix}", state)[0]
        assert not level.validate(f"{len(hidden)}{suffix}", state)[0]

        (start / ".extra-directory").mkdir()
        (start / ".extra-file").write_text("extra", encoding="utf-8")


@pytest.mark.parametrize("level_type", [MakeExecutableLevel, MakeReadOnlyLevel, PermissionsChallengeLevel])
@pytest.mark.parametrize("mask", ["000", "022", "077", "777"])
@pytest.mark.parametrize("shell", ["bash", "fish"])
def test_taught_permission_commands_are_independent_of_umask(
    tmp_path: Path,
    command_env: dict[str, str],
    level_type: type[Level],
    mask: str,
    shell: str,
) -> None:
    if not shutil.which(shell):
        pytest.skip(f"{shell} not installed")
    level = level_type()
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    level.prepare(workspace)
    state = _state(workspace, level)
    start = level.get_start_directory(workspace)
    assert start is not None
    assert level.solution is not None
    assert not level.validate(level.solution.answer, state)[0]
    commands = [step.command for step in level.solution.steps if isinstance(step, RunShell)]
    assert commands
    for recipe in commands:
        for command in recipe.split(" && "):
            assert command in " ".join(level.hints)
    flags = ["--no-config"] if shell == "fish" else []
    _run([shell, *flags, "-c", f"umask {mask}; " + " && ".join(commands)], start, command_env)
    success, message = level.validate(level.solution.answer, state)
    assert success, message


@pytest.mark.skipif(shutil.which("wc") is None, reason="wc not installed")
def test_word_count_answer_matches_wc_output(tmp_path: Path, command_env: dict[str, str]) -> None:
    level = WordAndLineCountLevel()
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    state = _state(workspace, level)

    for prepare in (level.prepare, level.reset):
        prepare(workspace)
        start = level.get_start_directory(workspace)
        assert start is not None
        lines = _run(["wc", "-l", "article.txt"], start, command_env).split()[0]
        words = _run(["wc", "-w", "article.txt"], start, command_env).split()[0]
        success, message = level.validate(f"{lines},{words}", state)
        assert success, message

        (start / "article.txt").write_text("changed\n", encoding="utf-8")


@pytest.mark.skipif(shutil.which("grep") is None, reason="grep not installed")
def test_recursive_grep_accepts_paths_printed_from_either_directory(
    tmp_path: Path, command_env: dict[str, str]
) -> None:
    level = RecursiveGrepFindFileLevel()
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    level.prepare(workspace)
    state = _state(workspace, level)
    start = level.get_start_directory(workspace)
    assert start is not None

    from_start = _run(["grep", "-r", "SECRET_KEY", "project/"], start, command_env).split(":", maxsplit=1)[0]
    from_project = _run(["grep", "-r", "SECRET_KEY", "."], start / "project", command_env).split(":", maxsplit=1)[0]

    assert level.validate(from_start, state)[0]
    assert level.validate(from_project, state)[0]


def test_grep_hints_do_not_reveal_the_matching_line(tmp_path: Path) -> None:
    level = GrepConfigLineToFileLevel()
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    level.prepare(workspace)
    start = level.get_start_directory(workspace)
    assert start is not None
    matching_line = next(
        line
        for line in (start / "config.txt").read_text(encoding="utf-8").splitlines()
        if "ACTIVE_PROFILE" in line
    )

    assert all(matching_line not in hint for hint in level.hints)


def test_level1_2_accepts_trailing_slash_from_tab_completion(tmp_path: Path) -> None:
    level = LsLevel()
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    state = _state(workspace, level)

    # delta/ from tab-completion must succeed
    ok, msg = level.validate("delta/", state)
    assert ok is True

    # data/ must trigger the specific file mistake, not a pattern mismatch
    ok, msg = level.validate("data/", state)
    assert ok is False
    assert "není to adresář" in msg

    # dog/ must trigger the pattern mismatch without ugly trailing slash in the diagnostic
    ok, msg = level.validate("dog/", state)
    assert ok is False
    assert "'dog' začíná na 'd'" in msg


def test_level2_8_hints_do_not_reveal_the_answer() -> None:
    level = HelpDiscoveryLevel()
    for hint in level.hints:
        assert "odpověď je 'human'" not in hint.lower()
        assert "odpoved je 'human'" not in hint.lower()


def test_bash_template_enforces_globasciiranges() -> None:
    template_path = Path("src/shellgame/cli/templates/bash_integration.template")
    content = template_path.read_text(encoding="utf-8")
    assert "shopt -s globasciiranges" in content
