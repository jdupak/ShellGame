"""Runtime tests for shell integration scripts.

These tests actually execute the generated shell scripts using `bash` and `fish`
(if available) to verify that the protocol handling (stderr filtering and
command execution) works as expected in a real shell environment.
"""
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
import pytest
from shellgame.cli.subshell import get_bash_integration, get_fish_integration

# Helper to create a mock game binary (python script)
def create_mock_game(tmp_path: Path, content: str) -> str:
    game_script = tmp_path / "mock_game.py"
    game_script.write_text(content, encoding="utf-8")
    # Return a command string that runs this script
    return f"{sys.executable} {game_script}"

@pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
def test_bash_integration_runtime_protocol(tmp_path: Path):
    # 1. Create a mock game that emits a protocol command via stderr
    #    The protocol command will be 'echo "PROTOCOL_WORKED"'
    mock_game_code = """
import sys
print("__SHELLGAME_EXEC__echo PROTOCOL_WORKED", file=sys.stderr)
print("normal stderr", file=sys.stderr)
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)

    # 2. Generate Bash integration script
    #    We need to be careful about how binary_path is passed. 
    #    get_bash_integration expects a string that will be put into `SHELLGAME_BINARY_ARR=($binary_path)`
    #    So we should quote the parts.
    import shlex
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())
    
    integration_script = get_bash_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.bash"
    integration_file.write_text(integration_script, encoding="utf-8")

    # 3. Run bash
    #    Source the integration, then run `shellgame`
    #    We capture stdout to verify 'PROTOCOL_WORKED' appears
    cmd = [
        "bash",
        "--noprofile",
        "--norc",
        "-c",
        f"source {integration_file}; shellgame"
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)
    
    assert result.returncode == 0
    assert "PROTOCOL_WORKED" in result.stdout
    assert "normal stderr" in result.stderr
    # The protocol line itself should NOT be in stderr
    assert "__SHELLGAME_EXEC__" not in result.stderr

@pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
def test_fish_integration_runtime_protocol(tmp_path: Path):
    # 1. Create a mock game
    mock_game_code = """
import sys
print("__SHELLGAME_EXEC__echo PROTOCOL_WORKED", file=sys.stderr)
print("normal stderr", file=sys.stderr)
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)

    # 2. Generate Fish integration script
    #    Fish integration uses `eval $SHELLGAME_BINARY $argv`
    import shlex
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())

    integration_script = get_fish_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.fish"
    integration_file.write_text(integration_script, encoding="utf-8")

    # 3. Run fish
    cmd = [
        "fish",
        "--no-config",
        "-c",
        f"source {integration_file}; shellgame"
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)

    assert result.returncode == 0
    assert "PROTOCOL_WORKED" in result.stdout
    assert "normal stderr" in result.stderr
    assert "__SHELLGAME_EXEC__" not in result.stderr

@pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
def test_fish_integration_injection_vulnerability(tmp_path: Path):
    # Mock game that just prints args
    mock_game_code = """
import sys
print(f"ARGS: {sys.argv[1:]}")
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)

    # Generate Fish integration script
    import shlex
    quoted_binary = " ".join(shlex.quote(p) for p in binary_cmd.split())

    integration_script = get_fish_integration(quoted_binary, devmode=False)
    integration_file = tmp_path / "integration.fish"
    integration_file.write_text(integration_script, encoding="utf-8")

    # Attempt injection: pass a command separator and another command
    # We want to see if 'echo PWNED' gets executed by the shell
    injection_arg = "; echo PWNED"
    
    cmd = [
        "fish",
        "--no-config",
        "-c",
        f"source {integration_file}; shellgame '{injection_arg}'"
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)
    
    # If fixed, 'PWNED' should NOT be printed by the shell echo, 
    # but it SHOULD be printed by the python script as an argument.
    assert "PWNED" not in result.stdout or "ARGS: ['; echo PWNED']" in result.stdout

@pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
def test_bash_integration_injection_vulnerability(tmp_path: Path):
    # Mock game that just prints args
    mock_game_code = """
import sys
print(f"ARGS: {sys.argv[1:]}")
"""
    binary_cmd = create_mock_game(tmp_path, mock_game_code)

    # Generate Bash integration script
    import shlex
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
        f"source {integration_file}; shellgame '{injection_arg}'"
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)
    
    # If fixed, 'PWNED' should NOT be printed by the shell echo, 
    # but it SHOULD be printed by the python script as an argument.
    assert "PWNED" not in result.stdout or "ARGS: ['; echo PWNED']" in result.stdout
