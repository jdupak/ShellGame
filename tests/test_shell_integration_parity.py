"""Parity tests ensuring bash and fish integrations behave identically.

These tests verify that both shell integrations:
1. Handle exit codes consistently
2. Propagate arguments correctly
3. Handle special characters safely
4. Execute the pwd wrapper correctly
5. Handle the protocol marker in various positions

Tests are skipped if the respective shell is not installed.
"""
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Optional

import pytest

from shellgame.cli.subshell import get_bash_integration, get_fish_integration


def _create_mock_game(tmp_path: Path, code: str) -> str:
    """Create a mock Python script and return a shell-escaped command to run it."""
    import shlex

    game_script = tmp_path / "mock_game.py"
    game_script.write_text(code, encoding="utf-8")
    return " ".join(shlex.quote(p) for p in [sys.executable, str(game_script)])


def _run_bash(integration_path: Path, command: str) -> subprocess.CompletedProcess[str]:
    """Run a command in bash with the integration loaded."""
    return subprocess.run(
        ["bash", "--noprofile", "--norc", "-c", f"source {integration_path}; {command}"],
        capture_output=True,
        text=True,
    )


def _run_fish(integration_path: Path, command: str) -> subprocess.CompletedProcess[str]:
    """Run a command in fish with the integration loaded."""
    return subprocess.run(
        ["fish", "--no-config", "-c", f"source {integration_path}; {command}"],
        capture_output=True,
        text=True,
    )


class TestExitCodePropagation:
    """Test that exit codes from the game are propagated through the wrapper."""

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_propagates_exit_code_zero(self, tmp_path: Path) -> None:
        mock_code = "import sys; sys.exit(0)"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        result = _run_bash(int_file, "shellgame; echo \"exit:$?\"")
        assert "exit:0" in result.stdout

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_propagates_exit_code_nonzero(self, tmp_path: Path) -> None:
        mock_code = "import sys; sys.exit(42)"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        result = _run_bash(int_file, "shellgame; echo \"exit:$?\"")
        assert "exit:42" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_propagates_exit_code_zero(self, tmp_path: Path) -> None:
        mock_code = "import sys; sys.exit(0)"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        result = _run_fish(int_file, "shellgame; echo \"exit:$status\"")
        assert "exit:0" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_propagates_exit_code_nonzero(self, tmp_path: Path) -> None:
        mock_code = "import sys; sys.exit(42)"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        result = _run_fish(int_file, "shellgame; echo \"exit:$status\"")
        assert "exit:42" in result.stdout


class TestArgumentPropagation:
    """Test that arguments are passed correctly to the game."""

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_passes_simple_arguments(self, tmp_path: Path) -> None:
        mock_code = "import sys; print('ARGS:', sys.argv[1:])"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        result = _run_bash(int_file, "shellgame submit answer123")
        assert "['submit', 'answer123']" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_passes_simple_arguments(self, tmp_path: Path) -> None:
        mock_code = "import sys; print('ARGS:', sys.argv[1:])"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        result = _run_fish(int_file, "shellgame submit answer123")
        assert "['submit', 'answer123']" in result.stdout

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_handles_arguments_with_spaces(self, tmp_path: Path) -> None:
        mock_code = "import sys; print('ARGS:', sys.argv[1:])"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        result = _run_bash(int_file, 'shellgame submit "hello world"')
        assert "['submit', 'hello world']" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_handles_arguments_with_spaces(self, tmp_path: Path) -> None:
        mock_code = "import sys; print('ARGS:', sys.argv[1:])"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        result = _run_fish(int_file, 'shellgame submit "hello world"')
        assert "['submit', 'hello world']" in result.stdout


class TestProtocolEdgeCases:
    """Test protocol handling edge cases."""

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_handles_protocol_marker_in_middle_of_line(self, tmp_path: Path) -> None:
        # Protocol marker should ONLY be recognized at start of line
        mock_code = """
import sys
print("text before __SHELLGAME_EXEC__echo SHOULD_NOT_RUN", file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        result = _run_bash(int_file, "shellgame")
        assert "SHOULD_NOT_RUN" not in result.stdout
        assert "__SHELLGAME_EXEC__" in result.stderr

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_handles_protocol_marker_in_middle_of_line(self, tmp_path: Path) -> None:
        mock_code = """
import sys
print("text before __SHELLGAME_EXEC__echo SHOULD_NOT_RUN", file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        result = _run_fish(int_file, "shellgame")
        assert "SHOULD_NOT_RUN" not in result.stdout
        assert "__SHELLGAME_EXEC__" in result.stderr

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_handles_multiple_protocol_commands(self, tmp_path: Path) -> None:
        mock_code = """
import sys
print("__SHELLGAME_EXEC__echo FIRST", file=sys.stderr)
print("__SHELLGAME_EXEC__echo SECOND", file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        result = _run_bash(int_file, "shellgame")
        assert "FIRST" in result.stdout
        assert "SECOND" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_handles_multiple_protocol_commands(self, tmp_path: Path) -> None:
        mock_code = """
import sys
print("__SHELLGAME_EXEC__echo FIRST", file=sys.stderr)
print("__SHELLGAME_EXEC__echo SECOND", file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        result = _run_fish(int_file, "shellgame")
        assert "FIRST" in result.stdout
        assert "SECOND" in result.stdout

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_handles_empty_protocol_command(self, tmp_path: Path) -> None:
        mock_code = """
import sys
print("__SHELLGAME_EXEC__", file=sys.stderr)
print("normal output", file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        result = _run_bash(int_file, "shellgame")
        # Should not crash
        assert result.returncode == 0
        assert "normal output" in result.stderr

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_handles_empty_protocol_command(self, tmp_path: Path) -> None:
        mock_code = """
import sys
print("__SHELLGAME_EXEC__", file=sys.stderr)
print("normal output", file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        result = _run_fish(int_file, "shellgame")
        assert result.returncode == 0
        assert "normal output" in result.stderr


class TestPwdWrapper:
    """Test that pwd wrapper records usage correctly."""

    @pytest.fixture
    def user_dir(self, tmp_path: Path) -> Path:
        """Create a fake user directory for testing."""
        user = os.environ.get("USER", "test")
        user_dir = tmp_path / f"shellgame-{user}"
        user_dir.mkdir()
        return user_dir

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_pwd_records_usage(self, tmp_path: Path, user_dir: Path, monkeypatch) -> None:
        mock_code = "print('done')"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        # Override /tmp with our tmp_path for the marker file location
        env = os.environ.copy()
        marker_file = user_dir / ".pwd_used"
        assert not marker_file.exists()

        # Inject custom user_dir path into the script
        modified_integration = integration.replace(
            '/tmp/shellgame-$USER',
            str(user_dir)
        )
        int_file.write_text(modified_integration)

        result = _run_bash(int_file, "pwd")
        assert result.returncode == 0
        assert marker_file.exists()

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_pwd_records_usage(self, tmp_path: Path, user_dir: Path) -> None:
        mock_code = "print('done')"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"

        marker_file = user_dir / ".pwd_used"
        assert not marker_file.exists()

        modified_integration = integration.replace(
            '/tmp/shellgame-$USER',
            str(user_dir)
        )
        int_file.write_text(modified_integration)

        result = _run_fish(int_file, "pwd")
        assert result.returncode == 0
        assert marker_file.exists()


class TestCdHookConsistency:
    """Test that cd hooks are generated consistently for both shells."""

    def test_bash_and_fish_hooks_cover_same_levels(self) -> None:
        """Ensure both generators produce hooks for the same levels."""
        from shellgame.cli.hooks import (
            generate_bash_cd_hooks,
            generate_fish_cd_hooks,
            get_cd_hooked_levels,
        )

        hooked_levels = get_cd_hooked_levels()
        bash_hooks = generate_bash_cd_hooks()
        fish_hooks = generate_fish_cd_hooks()

        for level_id in hooked_levels:
            level_marker = level_id.replace(".", "_")
            assert f"__shellgame_cd_{level_marker}" in bash_hooks, f"Missing bash hook for {level_id}"
            assert f"__shellgame_cd_{level_marker}" in fish_hooks, f"Missing fish hook for {level_id}"

    def test_hooks_dispatch_by_shellgame_level_env(self) -> None:
        """Both shells should dispatch cd based on SHELLGAME_LEVEL environment variable."""
        from shellgame.cli.hooks import generate_bash_cd_hooks, generate_fish_cd_hooks

        bash_hooks = generate_bash_cd_hooks()
        fish_hooks = generate_fish_cd_hooks()

        # Bash uses case statement
        assert 'case "$SHELLGAME_LEVEL"' in bash_hooks
        # Fish uses switch
        assert 'switch "$SHELLGAME_LEVEL"' in fish_hooks


class TestProtocolCdBypassesHooks:
    """Test that protocol cd commands bypass user-facing cd hooks.

    This is critical: when the game teleports the user between levels,
    the protocol cd must use builtin cd, not the hooked cd function.
    Otherwise, hooks like level 1.9 (which reject absolute paths) would
    break level transitions.
    """

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_protocol_cd_bypasses_hook_that_rejects_absolute_paths(self, tmp_path: Path) -> None:
        """Protocol cd should work even when SHELLGAME_LEVEL is set to a hook that rejects absolute paths."""
        # Create a target directory
        target_dir = tmp_path / "target"
        target_dir.mkdir()

        # Mock game that sets SHELLGAME_LEVEL=1.9 (which rejects absolute paths)
        # and then tries to cd to an absolute path via protocol
        mock_code = f"""
import sys
print("__SHELLGAME_EXEC__export SHELLGAME_LEVEL=1.9", file=sys.stderr)
print("__SHELLGAME_EXEC__cd {target_dir}", file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        # Set up user_dir so the hook is active
        user = os.environ.get("USER", "test")
        user_dir = tmp_path / f"shellgame-{user}"
        user_dir.mkdir()

        # Modify integration to use our test user_dir
        modified = integration.replace('/tmp/shellgame-$USER', str(user_dir))
        int_file.write_text(modified)

        result = _run_bash(int_file, f"shellgame; pwd")
        # The protocol cd should have succeeded, putting us in target_dir
        assert str(target_dir) in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_protocol_cd_bypasses_hook_that_rejects_absolute_paths(self, tmp_path: Path) -> None:
        """Protocol cd should work even when SHELLGAME_LEVEL is set to a hook that rejects absolute paths."""
        target_dir = tmp_path / "target"
        target_dir.mkdir()

        mock_code = f"""
import sys
print("__SHELLGAME_EXEC__export SHELLGAME_LEVEL=1.9", file=sys.stderr)
print("__SHELLGAME_EXEC__cd {target_dir}", file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"

        user = os.environ.get("USER", "test")
        user_dir = tmp_path / f"shellgame-{user}"
        user_dir.mkdir()

        modified = integration.replace('/tmp/shellgame-$USER', str(user_dir))
        int_file.write_text(modified)

        result = _run_fish(int_file, "shellgame; pwd")
        assert str(target_dir) in result.stdout

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_user_cd_still_uses_hook(self, tmp_path: Path) -> None:
        """Direct user cd should still go through the hook (not bypass it)."""
        target_dir = tmp_path / "target"
        target_dir.mkdir()

        # Mock game that just sets the level
        mock_code = """
import sys
print("__SHELLGAME_EXEC__export SHELLGAME_LEVEL=1.9", file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)

        user = os.environ.get("USER", "test")
        user_dir = tmp_path / f"shellgame-{user}"
        user_dir.mkdir()

        modified = integration.replace('/tmp/shellgame-$USER', str(user_dir))
        int_file = tmp_path / "integration.bash"
        int_file.write_text(modified)

        # After shellgame sets SHELLGAME_LEVEL=1.9, a direct user cd to absolute path should fail
        result = _run_bash(int_file, f"shellgame; cd {target_dir} 2>&1; echo exit:$?")
        # The hook should reject this (exit code 1)
        assert "exit:1" in result.stdout
        # And show the rejection message
        assert "1.9" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_user_cd_still_uses_hook(self, tmp_path: Path) -> None:
        """Direct user cd should still go through the hook (not bypass it)."""
        target_dir = tmp_path / "target"
        target_dir.mkdir()

        mock_code = """
import sys
print("__SHELLGAME_EXEC__export SHELLGAME_LEVEL=1.9", file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)

        user = os.environ.get("USER", "test")
        user_dir = tmp_path / f"shellgame-{user}"
        user_dir.mkdir()

        modified = integration.replace('/tmp/shellgame-$USER', str(user_dir))
        int_file = tmp_path / "integration.fish"
        int_file.write_text(modified)

        result = _run_fish(int_file, f"shellgame; cd {target_dir} 2>&1; echo exit:$status")
        assert "exit:1" in result.stdout
        assert "1.9" in result.stdout
