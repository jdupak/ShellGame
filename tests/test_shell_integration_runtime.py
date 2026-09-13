"""Runtime tests for shell integration scripts.

These tests actually execute the generated shell scripts using `bash` and `fish`
(if available) to verify that the protocol handling (stderr filtering and
command execution) works as expected in a real shell environment.
"""

import base64
import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

from shellgame.cli.subshell import (
    _create_fd_hook_script,
    get_bash_integration,
    get_fish_integration,
)


def _protocol_line(command: str, *args: str) -> str:
    encoded = [base64.b64encode(arg.encode()).decode() for arg in args]
    return "__SHELLGAME_EXEC__" + " ".join(("v1", command, *encoded))


# Helper to create a mock game binary (python script)
def create_mock_game(tmp_path: Path, content: str) -> str:
    game_script = tmp_path / "mock_game.py"
    game_script.write_text(content, encoding="utf-8")
    # Return a command string that runs this script
    return f"{sys.executable} {game_script}"


def test_fd_hook_launcher_preserves_redirected_descriptors(tmp_path: Path) -> None:
    report = tmp_path / "fd-report.txt"
    game_script = tmp_path / "fd_probe.py"
    game_script.write_text(
        """
import os
import sys
from pathlib import Path

if sys.argv[1:] == ["fd-hook"]:
    Path(os.environ["FD_REPORT"]).write_text(
        os.readlink("/proc/self/fd/1") + "\\n" + os.readlink("/proc/self/fd/2"),
        encoding="utf-8",
    )
""",
        encoding="utf-8",
    )
    launcher = _create_fd_hook_script([sys.executable, str(game_script)])

    try:
        with Path("/dev/null").open("wb") as dev_null:
            result = subprocess.run(
                [launcher],
                check=False,
                stdout=dev_null,
                stderr=dev_null,
                env={**os.environ, "FD_REPORT": str(report)},
            )
    finally:
        Path(launcher).unlink()

    assert result.returncode == 0
    assert report.read_text(encoding="utf-8").splitlines() == [
        "/dev/null",
        "/dev/null",
    ]


@pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
def test_bash_integration_runtime_protocol(tmp_path: Path) -> None:
    # 1. Create a mock game that emits a protocol command via stderr
    #    The protocol command will be 'echo "PROTOCOL_WORKED"'
    protocol_echo = _protocol_line("echo", "PROTOCOL_WORKED")
    mock_game_code = f"""
import sys
print({protocol_echo!r}, file=sys.stderr)
print("normal stderr", file=sys.stderr)
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)

    # 2. Generate Bash integration script
    #    We need to be careful about how binary_path is passed.
    #    get_bash_integration expects a string that will be put into `SHELLGAME_BINARY_ARR=($binary_path)`
    #    So we should quote the parts.
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())

    integration_script = get_bash_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.bash"
    integration_file.write_text(integration_script, encoding="utf-8")

    # 3. Run bash
    #    Source the integration, then run `shellgame`
    #    We capture stdout to verify 'PROTOCOL_WORKED' appears
    cmd = ["bash", "--noprofile", "--norc", "-c", f"source {integration_file}; shellgame"]

    result = subprocess.run(cmd, check=False, capture_output=True, text=True)

    assert result.returncode == 0
    assert "PROTOCOL_WORKED" in result.stdout
    assert "normal stderr" in result.stderr
    # The protocol line itself should NOT be in stderr
    assert "__SHELLGAME_EXEC__" not in result.stderr


@pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
def test_fish_integration_runtime_protocol(tmp_path: Path) -> None:
    # 1. Create a mock game
    protocol_echo = _protocol_line("echo", "PROTOCOL_WORKED")
    mock_game_code = f"""
import sys
print({protocol_echo!r}, file=sys.stderr)
print("normal stderr", file=sys.stderr)
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)

    # 2. Generate Fish integration script
    #    Fish integration uses `eval $SHELLGAME_BINARY $argv`
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())

    integration_script = get_fish_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.fish"
    integration_file.write_text(integration_script, encoding="utf-8")

    # 3. Run fish
    cmd = ["fish", "--no-config", "-c", f"source {integration_file}; shellgame"]

    result = subprocess.run(cmd, check=False, capture_output=True, text=True)

    assert result.returncode == 0
    assert "PROTOCOL_WORKED" in result.stdout
    assert "normal stderr" in result.stderr
    assert "__SHELLGAME_EXEC__" not in result.stderr


@pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
def test_bash_protocol_preserves_cd_path_with_spaces(tmp_path: Path) -> None:
    target = tmp_path / "directory with spaces"
    target.mkdir()
    protocol_cd = _protocol_line("cd", str(target))
    mock_game_code = f"""
import sys
print({protocol_cd!r}, file=sys.stderr)
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())
    integration = get_bash_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.bash"
    integration_file.write_text(integration, encoding="utf-8")

    result = subprocess.run(
        ["bash", "--noprofile", "--norc", "-c", f"source {integration_file}; pwd"],
        check=False,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0
    assert str(target) in result.stdout


@pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
def test_bash_protocol_exit_closes_wrapper(tmp_path: Path) -> None:
    protocol_exit = _protocol_line("exit")
    mock_game_code = f"""
import sys
if sys.argv[1:] == ["exit"]:
    print({protocol_exit!r}, file=sys.stderr)
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())
    integration = get_bash_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.bash"
    integration_file.write_text(integration, encoding="utf-8")
    shell_tmp = tmp_path / "shell-tmp"
    shell_tmp.mkdir()

    result = subprocess.run(
        [
            "bash",
            "--noprofile",
            "--norc",
            "-c",
            f"source {integration_file}; shellgame exit; echo SHOULD_NOT_RUN",
        ],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "TMPDIR": str(shell_tmp)},
    )

    assert result.returncode == 0
    assert "SHOULD_NOT_RUN" not in result.stdout
    assert list(shell_tmp.iterdir()) == []


@pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
def test_fish_protocol_exit_closes_wrapper_without_leaking_temp_file(tmp_path: Path) -> None:
    protocol_exit = _protocol_line("exit")
    mock_game_code = f"""
import sys
if sys.argv[1:] == ["exit"]:
    print({protocol_exit!r}, file=sys.stderr)
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())
    integration = get_fish_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.fish"
    integration_file.write_text(integration, encoding="utf-8")
    shell_tmp = tmp_path / "shell-tmp"
    shell_tmp.mkdir()

    result = subprocess.run(
        [
            "fish",
            "--no-config",
            "-c",
            f"source {integration_file}; shellgame exit; echo SHOULD_NOT_RUN",
        ],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "TMPDIR": str(shell_tmp)},
    )

    assert result.returncode == 0
    assert "SHOULD_NOT_RUN" not in result.stdout
    assert list(shell_tmp.iterdir()) == []


@pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
def test_fish_integration_injection_vulnerability(tmp_path: Path) -> None:
    # Mock game that just prints args
    mock_game_code = """
import sys
print(f"ARGS: {sys.argv[1:]}")
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)

    # Generate Fish integration script
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())

    integration_script = get_fish_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.fish"
    integration_file.write_text(integration_script, encoding="utf-8")

    # Attempt injection: pass a command separator and another command
    # We want to see if 'echo PWNED' gets executed by the shell
    injection_arg = "; echo PWNED"

    cmd = ["fish", "--no-config", "-c", f"source {integration_file}; shellgame '{injection_arg}'"]

    result = subprocess.run(cmd, check=False, capture_output=True, text=True)

    # If fixed, 'PWNED' should NOT be printed by the shell echo,
    # but it SHOULD be printed by the python script as an argument.
    assert "PWNED" not in result.stdout or "ARGS: ['; echo PWNED']" in result.stdout


@pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
def test_bash_integration_injection_vulnerability(tmp_path: Path) -> None:
    # Mock game that just prints args
    mock_game_code = """
import sys
print(f"ARGS: {sys.argv[1:]}")
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)

    # Generate Bash integration script
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())

    integration_script = get_bash_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.bash"
    integration_file.write_text(integration_script, encoding="utf-8")

    # Attempt injection: pass a command separator and another command
    # We want to see if 'echo PWNED' gets executed by the shell
    injection_arg = "; echo PWNED"

    cmd = [
        "bash",
        "--noprofile",
        "--norc",
        "-c",
        f"source {integration_file}; shellgame '{injection_arg}'",
    ]

    result = subprocess.run(cmd, check=False, capture_output=True, text=True)

    # If fixed, 'PWNED' should NOT be printed by the shell echo,
    # but it SHOULD be printed by the python script as an argument.
    assert "PWNED" not in result.stdout or "ARGS: ['; echo PWNED']" in result.stdout


@pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
def test_bash_relocates_from_deleted_cwd_before_invoking_python(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    sub = workspace / "sub"
    sub.mkdir()

    mock_game_code = """
import os
import sys
print(f"CWD: {os.getcwd()}")
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())
    integration = get_bash_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.bash"
    integration_file.write_text(integration, encoding="utf-8")

    cmd = [
        "bash",
        "--noprofile",
        "--norc",
        "-c",
        f"source {integration_file}; cd {sub}; rm -rf {sub}; shellgame check",
    ]

    result = subprocess.run(
        cmd,
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "SHELLGAME_WORKSPACE": str(workspace)},
    )

    assert result.returncode == 0
    assert f"CWD: {workspace.resolve()}" in result.stdout


@pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
def test_fish_relocates_from_deleted_cwd_before_invoking_python(tmp_path: Path) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    sub = workspace / "sub"
    sub.mkdir()

    mock_game_code = """
import os
import sys
print(f"CWD: {os.getcwd()}")
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())
    integration = get_fish_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.fish"
    integration_file.write_text(integration, encoding="utf-8")

    cmd = [
        "fish",
        "--no-config",
        "-c",
        f"source {integration_file}; cd {sub}; rm -rf {sub}; shellgame check",
    ]

    result = subprocess.run(
        cmd,
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, "SHELLGAME_WORKSPACE": str(workspace)},
    )

    assert result.returncode == 0
    assert f"CWD: {workspace.resolve()}" in result.stdout
