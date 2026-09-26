"""A command typed outside the game shell must survive the subshell launch.

`shellgame resume 6.2` in an ordinary terminal has to start the game subshell
first, which ends the original process before Click ever reaches the
subcommand. Without forwarding, what the player typed is silently dropped and
they land on their old level instead.
"""

from __future__ import annotations

import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

import pytest

from shellgame.cli import boot, commands
from shellgame.cli.subshell import _autostart_command, get_bash_integration, get_fish_integration


class _FakeContext:
    def __init__(self, invoked_subcommand: str | None) -> None:
        self.invoked_subcommand = invoked_subcommand


def _pending(argv: list[str], invoked: str | None, monkeypatch: Any) -> list[str]:
    monkeypatch.setattr(sys, "argv", ["shellgame", *argv])
    return commands._pending_command(_FakeContext(invoked))  # type: ignore[arg-type]


def test_pending_command_captures_subcommand_and_arguments(monkeypatch: Any) -> None:
    assert _pending(["resume", "6.2"], "resume", monkeypatch) == ["resume", "6.2"]


def test_pending_command_skips_group_level_options(monkeypatch: Any) -> None:
    assert _pending(["--shell", "bash", "resume", "6.2"], "resume", monkeypatch) == ["resume", "6.2"]


def test_pending_command_is_empty_for_a_bare_launch(monkeypatch: Any) -> None:
    assert _pending([], None, monkeypatch) == []


def test_pending_command_gives_up_rather_than_guessing(monkeypatch: Any) -> None:
    """A resolved name that was never typed must not be forwarded as a guess."""
    assert _pending(["--shell", "bash"], "resume", monkeypatch) == []


@pytest.mark.parametrize(
    ("shell_name", "expected"),
    [
        ("bash", "resume 6.2"),
        ("fish", "'resume' '6.2'"),
    ],
)
def test_autostart_command_quotes_per_shell(shell_name: str, expected: str) -> None:
    assert _autostart_command(shell_name, ["resume", "6.2"]) == expected


def test_autostart_command_is_empty_without_a_pending_command() -> None:
    assert _autostart_command("bash", None) == ""
    assert _autostart_command("bash", []) == ""


@pytest.mark.skipif(shutil.which("bash") is None, reason="bash not installed")
def test_bash_quoting_survives_spaces_and_quotes() -> None:
    """The rendered string must re-split into the exact original arguments."""
    argv = ["submit", "a b 'c'", "d\\e"]

    rendered = _autostart_command("bash", argv)

    assert shlex.split(rendered) == argv


@pytest.mark.skipif(shutil.which("fish") is None, reason="fish not installed")
def test_fish_quoting_survives_spaces_and_quotes(tmp_path: Path) -> None:
    """fish escapes differently from POSIX, so real fish has to confirm it."""
    argv = ["submit", "a b 'c'", "d\\e"]
    recorder = tmp_path / "args.txt"

    rendered = _autostart_command("fish", argv)
    script = (
        f"function shellgame; printf '%s\\n' $argv > {shlex.quote(str(recorder))}; end; eval \"shellgame {rendered}\""
    )
    subprocess.run(["fish", "-c", script], check=True, capture_output=True, text=True)

    assert recorder.read_text(encoding="utf-8").splitlines() == argv


def test_boot_forwards_the_pending_command_to_the_subshell(monkeypatch: Any) -> None:
    launched: dict[str, Any] = {}

    def fake_launch(shell_name: str, devmode: bool = False, *, pending_command: list[str] | None = None) -> None:
        launched["shell"] = shell_name
        launched["pending"] = pending_command

    monkeypatch.setattr(boot, "launch_subshell", fake_launch)
    monkeypatch.setenv("SHELLGAME_FORCE_SHELL", "bash")

    result = boot.boot_if_needed(wrapped=False, devmode=False, pending_command=["resume", "6.2"])

    assert result.should_exit is True
    assert launched["pending"] == ["resume", "6.2"]


def test_boot_inside_the_game_shell_does_not_relaunch(monkeypatch: Any) -> None:
    def fail_launch(*args: Any, **kwargs: Any) -> None:
        raise AssertionError("must not launch a subshell when already wrapped")

    monkeypatch.setattr(boot, "launch_subshell", fail_launch)

    result = boot.boot_if_needed(wrapped=True, devmode=False, pending_command=["resume", "6.2"])

    assert result.should_exit is False


def _recording_binary(tmp_path: Path, recorder: Path) -> str:
    stub = tmp_path / "stub.py"
    stub.write_text(
        "import sys, pathlib\n"
        f"pathlib.Path({str(recorder)!r}).write_text(chr(10).join(sys.argv[1:]), encoding='utf-8')\n",
        encoding="utf-8",
    )
    return " ".join(shlex.quote(part) for part in [sys.executable, str(stub)])


def _source_in_bash(script: Path) -> None:
    subprocess.run(
        ["bash", "--noprofile", "--norc", "-c", f"source {shlex.quote(str(script))}"],
        check=False,
        capture_output=True,
        text=True,
        env=os.environ.copy(),
    )


@pytest.mark.skipif(shutil.which("bash") is None, reason="bash not installed")
@pytest.mark.parametrize("argv", [["resume", "6.2"], ["submit", "a b 'c'"], ["levels"]])
def test_bash_autostart_runs_the_forwarded_command(tmp_path: Path, argv: list[str]) -> None:
    """End to end in real bash: the forwarded args reach the game binary intact."""
    recorder = tmp_path / "args.txt"
    script = tmp_path / "integration.bash"
    script.write_text(
        get_bash_integration(_recording_binary(tmp_path, recorder), pending_command=argv),
        encoding="utf-8",
    )

    _source_in_bash(script)

    assert recorder.read_text(encoding="utf-8").splitlines() == argv


@pytest.mark.skipif(shutil.which("bash") is None, reason="bash not installed")
def test_bash_autostart_without_forwarding_starts_the_game_plainly(tmp_path: Path) -> None:
    recorder = tmp_path / "args.txt"
    script = tmp_path / "integration.bash"
    script.write_text(get_bash_integration(_recording_binary(tmp_path, recorder)), encoding="utf-8")

    _source_in_bash(script)

    assert recorder.read_text(encoding="utf-8") == ""


@pytest.mark.skipif(shutil.which("fish") is None, reason="fish not installed")
@pytest.mark.parametrize("argv", [["resume", "6.2"], ["submit", "a b 'c'"]])
def test_fish_autostart_runs_the_forwarded_command(tmp_path: Path, argv: list[str]) -> None:
    recorder = tmp_path / "args.txt"
    script = tmp_path / "integration.fish"
    script.write_text(
        get_fish_integration(_recording_binary(tmp_path, recorder), pending_command=argv),
        encoding="utf-8",
    )

    subprocess.run(
        ["fish", "-c", f"source {shlex.quote(str(script))}"],
        check=False,
        capture_output=True,
        text=True,
        env=os.environ.copy(),
    )

    assert recorder.read_text(encoding="utf-8").splitlines() == argv
