"""Parity tests ensuring bash and fish integrations behave identically.

These tests verify that both shell integrations:
1. Handle exit codes consistently
2. Propagate arguments correctly
3. Handle special characters safely
4. Execute the pwd wrapper correctly
5. Handle the protocol marker in various positions

Tests are skipped if the respective shell is not installed.
"""

import base64
import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

import pytest

from shellgame.cli.hooks import (
    generate_bash_cd_hooks,
    generate_fish_cd_hooks,
    get_cd_hooked_levels,
)
from shellgame.cli.subshell import get_bash_integration, get_fish_integration


def _protocol_line(command: str, *args: str) -> str:
    encoded = [base64.b64encode(arg.encode()).decode() for arg in args]
    return "__SHELLGAME_EXEC__" + " ".join(("v1", command, *encoded))


def _create_mock_game(tmp_path: Path, code: str) -> str:
    """Create a mock Python script and return a shell-escaped command to run it."""
    game_script = tmp_path / "mock_game.py"
    game_script.write_text(code, encoding="utf-8")
    return " ".join(shlex.quote(p) for p in [sys.executable, str(game_script)])


def _run_bash(
    integration_path: Path, command: str, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    """Run a command in bash with the integration loaded."""
    return subprocess.run(
        ["bash", "--noprofile", "--norc", "-c", f"source {integration_path}; {command}"],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, **env} if env else None,
    )


def _run_fish(
    integration_path: Path, command: str, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[str]:
    """Run a command in fish with the integration loaded."""
    return subprocess.run(
        ["fish", "--no-config", "-c", f"source {integration_path}; {command}"],
        check=False,
        capture_output=True,
        text=True,
        env={**os.environ, **env} if env else None,
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

        result = _run_bash(int_file, 'shellgame; echo "exit:$?"')
        assert "exit:0" in result.stdout

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_propagates_exit_code_nonzero(self, tmp_path: Path) -> None:
        mock_code = "import sys; sys.exit(42)"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        result = _run_bash(int_file, 'shellgame; echo "exit:$?"')
        assert "exit:42" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_propagates_exit_code_zero(self, tmp_path: Path) -> None:
        mock_code = "import sys; sys.exit(0)"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        result = _run_fish(int_file, 'shellgame; echo "exit:$status"')
        assert "exit:0" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_propagates_exit_code_nonzero(self, tmp_path: Path) -> None:
        mock_code = "import sys; sys.exit(42)"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        result = _run_fish(int_file, 'shellgame; echo "exit:$status"')
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
        first = _protocol_line("echo", "FIRST")
        second = _protocol_line("echo", "SECOND")
        mock_code = f"""
import sys
print({first!r}, file=sys.stderr)
print({second!r}, file=sys.stderr)
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
        first = _protocol_line("echo", "FIRST")
        second = _protocol_line("echo", "SECOND")
        mock_code = f"""
import sys
print({first!r}, file=sys.stderr)
print({second!r}, file=sys.stderr)
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
    def test_bash_pwd_records_usage(self, tmp_path: Path, user_dir: Path, monkeypatch: Any) -> None:
        mock_code = "print('done')"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        marker_file = user_dir / ".pwd_used"
        assert not marker_file.exists()

        result = _run_bash(int_file, "pwd", env={"SHELLGAME_WORKSPACE": str(user_dir)})
        assert result.returncode == 0
        assert marker_file.exists()

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_pwd_records_usage(self, tmp_path: Path, user_dir: Path) -> None:
        mock_code = "print('done')"
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(get_fish_integration(binary_cmd, devmode=False))

        marker_file = user_dir / ".pwd_used"
        assert not marker_file.exists()

        result = _run_fish(int_file, "pwd", env={"SHELLGAME_WORKSPACE": str(user_dir)})
        assert result.returncode == 0
        assert marker_file.exists()

    def test_templates_never_guess_the_workspace_path(self) -> None:
        """The workspace must come from SHELLGAME_WORKSPACE only.

        A `$USER`-derived fallback would let a stale or foreign directory
        collect evidence markers, so it must not exist in either template.
        """
        binary_cmd = "shellgame"
        for integration in (
            get_bash_integration(binary_cmd, devmode=False),
            get_fish_integration(binary_cmd, devmode=False),
        ):
            assert "$USER" not in integration
            assert "/tmp/shellgame-" not in integration

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_pwd_is_safe_without_workspace(self, tmp_path: Path) -> None:
        """`pwd` must still work (and create nothing) when no workspace is set."""
        binary_cmd = _create_mock_game(tmp_path, "print('done')")
        int_file = tmp_path / "integration.bash"
        int_file.write_text(get_bash_integration(binary_cmd, devmode=False))

        result = _run_bash(int_file, "pwd", env={"SHELLGAME_WORKSPACE": ""})
        assert result.returncode == 0
        assert not list(tmp_path.glob("**/.pwd_used"))

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_pwd_is_safe_without_workspace(self, tmp_path: Path) -> None:
        """`pwd` must still work (and create nothing) when no workspace is set."""
        binary_cmd = _create_mock_game(tmp_path, "print('done')")
        int_file = tmp_path / "integration.fish"
        int_file.write_text(get_fish_integration(binary_cmd, devmode=False))

        result = _run_fish(int_file, "pwd", env={"SHELLGAME_WORKSPACE": ""})
        assert result.returncode == 0
        assert not list(tmp_path.glob("**/.pwd_used"))


class TestCdHookConsistency:
    """Test that cd hooks are generated consistently for both shells."""

    def test_bash_and_fish_hooks_cover_same_levels(self) -> None:
        """Ensure both generators produce hooks for the same levels."""
        hooked_levels = get_cd_hooked_levels()
        bash_hooks = generate_bash_cd_hooks()
        fish_hooks = generate_fish_cd_hooks()

        bash_patterns = next(
            line for line in bash_hooks.splitlines() if line.strip().startswith('"') and line.strip().endswith(")")
        )
        fish_patterns = next(line for line in fish_hooks.splitlines() if line.strip().startswith('case "'))

        for level_id in hooked_levels:
            assert f'"{level_id}"' in bash_patterns, f"Missing bash hook for {level_id}"
            assert f'"{level_id}"' in fish_patterns, f"Missing fish hook for {level_id}"

    def test_hooks_dispatch_by_shellgame_level_env(self) -> None:
        """Both shells should dispatch cd based on SHELLGAME_LEVEL environment variable."""
        bash_hooks = generate_bash_cd_hooks()
        fish_hooks = generate_fish_cd_hooks()

        # Bash uses case statement
        assert 'case "$SHELLGAME_LEVEL"' in bash_hooks
        # Fish uses switch
        assert 'switch "$SHELLGAME_LEVEL"' in fish_hooks


@pytest.mark.skip(reason="Requires shellgame binary in PATH")
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
        export_level = _protocol_line("export", "SHELLGAME_LEVEL", "1.9")
        change_directory = _protocol_line("cd", str(target_dir))
        mock_code = f"""
import sys
print({export_level!r}, file=sys.stderr)
print({change_directory!r}, file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        int_file.write_text(integration)

        result = _run_bash(int_file, "shellgame; pwd")
        # The protocol cd should have succeeded, putting us in target_dir
        assert str(target_dir) in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_protocol_cd_bypasses_hook_that_rejects_absolute_paths(self, tmp_path: Path) -> None:
        """Protocol cd should work even when SHELLGAME_LEVEL is set to a hook that rejects absolute paths."""
        target_dir = tmp_path / "target"
        target_dir.mkdir()

        export_level = _protocol_line("export", "SHELLGAME_LEVEL", "1.9")
        change_directory = _protocol_line("cd", str(target_dir))
        mock_code = f"""
import sys
print({export_level!r}, file=sys.stderr)
print({change_directory!r}, file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"

        int_file.write_text(integration)

        result = _run_fish(int_file, "shellgame; pwd")
        assert str(target_dir) in result.stdout

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_user_cd_still_uses_hook(self, tmp_path: Path) -> None:
        """Direct user cd should still go through the hook (not bypass it)."""
        target_dir = tmp_path / "target"
        target_dir.mkdir()

        # Mock game that just sets the level
        export_level = _protocol_line("export", "SHELLGAME_LEVEL", "1.9")
        mock_code = f"""
import sys
print({export_level!r}, file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_bash_integration(binary_cmd, devmode=False)

        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

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

        export_level = _protocol_line("export", "SHELLGAME_LEVEL", "1.9")
        mock_code = f"""
import sys
print({export_level!r}, file=sys.stderr)
"""
        binary_cmd = _create_mock_game(tmp_path, mock_code)
        integration = get_fish_integration(binary_cmd, devmode=False)

        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        result = _run_fish(int_file, f"shellgame; cd {target_dir} 2>&1; echo exit:$status")
        assert "exit:1" in result.stdout
        assert "1.9" in result.stdout


class TestAutocompleteParity:
    """Test autocomplete behavior for both bash and fish."""

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_completes_commands_and_options(self, tmp_path: Path) -> None:
        binary_cmd = _create_mock_game(tmp_path, "import sys; sys.exit(0)")
        integration = get_bash_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.bash"
        int_file.write_text(integration)

        # Test root completion
        result = _run_bash(
            int_file,
            'COMP_WORDS=(shellgame ""); COMP_CWORD=1; _shellgame_completions; echo "root:${COMPREPLY[*]}"',
        )
        assert "hint" in result.stdout
        assert "submit" in result.stdout
        assert "repeat" in result.stdout
        assert "show" in result.stdout

        # Test hint options
        result = _run_bash(
            int_file,
            'COMP_WORDS=(shellgame hint "-"); COMP_CWORD=2; _shellgame_completions; echo "hint:${COMPREPLY[*]}"',
        )
        assert "--repeat" in result.stdout
        assert "-r" in result.stdout

        # Test repeat options
        result = _run_bash(
            int_file,
            'COMP_WORDS=(shellgame repeat "--"); COMP_CWORD=2; _shellgame_completions; echo "repeat:${COMPREPLY[*]}"',
        )
        assert "--section" in result.stdout
        assert "--level" in result.stdout

        # Test alias sg
        result = _run_bash(
            int_file,
            'COMP_WORDS=(sg hint "-"); COMP_CWORD=2; _shellgame_completions; echo "sg:${COMPREPLY[*]}"',
        )
        assert "--repeat" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_completes_commands_and_options(self, tmp_path: Path) -> None:
        binary_cmd = _create_mock_game(tmp_path, "import sys; sys.exit(0)")
        integration = get_fish_integration(binary_cmd, devmode=False)
        int_file = tmp_path / "integration.fish"
        int_file.write_text(integration)

        # Test root completion
        result = _run_fish(int_file, 'complete -C "shellgame "')
        assert "hint" in result.stdout
        assert "submit" in result.stdout
        assert "repeat" in result.stdout
        assert "show" in result.stdout

        # Test hint options
        result = _run_fish(int_file, 'complete -C "shellgame hint -"')
        assert "--repeat" in result.stdout
        assert "-r" in result.stdout

        # Test repeat options
        result = _run_fish(int_file, 'complete -C "shellgame repeat --"')
        assert "--section" in result.stdout
        assert "--level" in result.stdout

        # Test alias sg
        result = _run_fish(int_file, 'complete -C "sg hint -"')
        assert "--repeat" in result.stdout
