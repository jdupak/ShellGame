"""Tests for cd hook behavior in both bash and fish.

These tests verify that the cd hooks for levels 1.8 (absolute path) and 1.9 (step-by-step)
work consistently across both shell implementations.
"""
import os
import shutil
import subprocess
from pathlib import Path

import pytest

from shellgame.cli.hooks import (
    generate_bash_cd_hook,
    generate_fish_cd_hook,
    get_cd_hooked_levels,
)


class TestCdHookGeneration:
    """Test that cd hooks are generated correctly."""

    def test_level_1_8_has_bash_hook(self) -> None:
        hook = generate_bash_cd_hook("1.8")
        assert hook, "Level 1.8 should have a bash cd hook"
        assert "builtin cd" in hook
        assert "absolute" in hook.lower() or "absolutní" in hook.lower()

    def test_level_1_8_has_fish_hook(self) -> None:
        hook = generate_fish_cd_hook("1.8")
        assert hook, "Level 1.8 should have a fish cd hook"
        assert "builtin cd" in hook
        assert "absolute" in hook.lower() or "absolutní" in hook.lower()

    def test_level_1_9_has_bash_hook(self) -> None:
        hook = generate_bash_cd_hook("1.9")
        assert hook, "Level 1.9 should have a bash cd hook"
        assert "builtin cd" in hook
        assert "segment" in hook.lower()

    def test_level_1_9_has_fish_hook(self) -> None:
        hook = generate_fish_cd_hook("1.9")
        assert hook, "Level 1.9 should have a fish cd hook"
        assert "builtin cd" in hook
        assert "segment" in hook.lower()

    def test_non_hooked_level_returns_empty(self) -> None:
        assert generate_bash_cd_hook("1.1") == ""
        assert generate_fish_cd_hook("1.1") == ""
        assert generate_bash_cd_hook("2.0") == ""
        assert generate_fish_cd_hook("2.0") == ""


class TestLevel18CdHookBehavior:
    """Test level 1.8 cd hook: requires absolute paths."""

    @pytest.fixture
    def workspace(self, tmp_path: Path) -> Path:
        """Create a mock workspace with the absolute-target directory."""
        ws = tmp_path / "shellgame-workspace"
        target = ws / "level-1" / "absolute-target"
        target.mkdir(parents=True)
        return ws

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_1_8_rejects_empty_cd(self, workspace: Path, tmp_path: Path) -> None:
        hook = generate_bash_cd_hook("1.8")
        script = tmp_path / "test.bash"
        script.write_text(f"""
export SHELLGAME_WORKSPACE="{workspace}"
{hook}
cd 2>&1
echo "exit:$?"
""")
        result = subprocess.run(
            ["bash", "--noprofile", "--norc", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:1" in result.stdout
        assert "absolutní" in result.stdout.lower() or "absolute" in result.stdout.lower()

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_1_8_rejects_empty_cd(self, workspace: Path, tmp_path: Path) -> None:
        hook = generate_fish_cd_hook("1.8")
        script = tmp_path / "test.fish"
        script.write_text(f"""
set -gx SHELLGAME_WORKSPACE "{workspace}"
{hook}
cd 2>&1
echo "exit:$status"
""")
        result = subprocess.run(
            ["fish", "--no-config", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:1" in result.stdout
        assert "absolutní" in result.stdout.lower() or "absolute" in result.stdout.lower()

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_1_8_rejects_relative_path(self, workspace: Path, tmp_path: Path) -> None:
        hook = generate_bash_cd_hook("1.8")
        script = tmp_path / "test.bash"
        script.write_text(f"""
export SHELLGAME_WORKSPACE="{workspace}"
{hook}
cd level-1 2>&1
echo "exit:$?"
""")
        result = subprocess.run(
            ["bash", "--noprofile", "--norc", str(script)],
            capture_output=True,
            text=True,
            cwd=str(workspace),
        )
        assert "exit:1" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_1_8_rejects_relative_path(self, workspace: Path, tmp_path: Path) -> None:
        hook = generate_fish_cd_hook("1.8")
        script = tmp_path / "test.fish"
        script.write_text(f"""
set -gx SHELLGAME_WORKSPACE "{workspace}"
{hook}
cd level-1 2>&1
echo "exit:$status"
""")
        result = subprocess.run(
            ["fish", "--no-config", str(script)],
            capture_output=True,
            text=True,
            cwd=str(workspace),
        )
        assert "exit:1" in result.stdout

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_1_8_accepts_absolute_path(self, workspace: Path, tmp_path: Path) -> None:
        target = workspace / "level-1" / "absolute-target"
        hook = generate_bash_cd_hook("1.8")
        script = tmp_path / "test.bash"
        script.write_text(f"""
export SHELLGAME_WORKSPACE="{workspace}"
{hook}
cd "{target}" 2>&1
echo "exit:$?"
echo "pwd:$(pwd)"
""")
        result = subprocess.run(
            ["bash", "--noprofile", "--norc", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:0" in result.stdout
        assert f"pwd:{target}" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_1_8_accepts_absolute_path(self, workspace: Path, tmp_path: Path) -> None:
        target = workspace / "level-1" / "absolute-target"
        hook = generate_fish_cd_hook("1.8")
        script = tmp_path / "test.fish"
        script.write_text(f"""
set -gx SHELLGAME_WORKSPACE "{workspace}"
{hook}
cd "{target}" 2>&1
echo "exit:$status"
echo "pwd:"(pwd)
""")
        result = subprocess.run(
            ["fish", "--no-config", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:0" in result.stdout
        assert f"pwd:{target}" in result.stdout


class TestLevel19CdHookBehavior:
    """Test level 1.9 cd hook: requires step-by-step navigation."""

    @pytest.fixture
    def user_dir(self, tmp_path: Path) -> Path:
        """Create a fake user directory for testing."""
        user = os.environ.get("USER", "test")
        user_dir = tmp_path / f"shellgame-{user}"
        user_dir.mkdir()
        return user_dir

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_1_9_rejects_empty_cd(self, tmp_path: Path, user_dir: Path) -> None:
        hook = generate_bash_cd_hook("1.9")
        # Replace /tmp/shellgame-$USER with our test dir
        hook = hook.replace('/tmp/shellgame-$USER', str(user_dir))
        script = tmp_path / "test.bash"
        script.write_text(f"""
export HOME="{tmp_path / 'home' / 'user'}"
{hook}
cd 2>&1
echo "exit:$?"
""")
        (tmp_path / "home" / "user").mkdir(parents=True)
        result = subprocess.run(
            ["bash", "--noprofile", "--norc", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:1" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_1_9_rejects_empty_cd(self, tmp_path: Path, user_dir: Path) -> None:
        hook = generate_fish_cd_hook("1.9")
        hook = hook.replace('/tmp/shellgame-$USER', str(user_dir))
        script = tmp_path / "test.fish"
        script.write_text(f"""
set -gx HOME "{tmp_path / 'home' / 'user'}"
{hook}
cd 2>&1
echo "exit:$status"
""")
        (tmp_path / "home" / "user").mkdir(parents=True)
        result = subprocess.run(
            ["fish", "--no-config", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:1" in result.stdout

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_1_9_rejects_literal_tilde(self, tmp_path: Path, user_dir: Path) -> None:
        """Test that literal '~' string is rejected."""
        hook = generate_bash_cd_hook("1.9")
        hook = hook.replace('/tmp/shellgame-$USER', str(user_dir))
        script = tmp_path / "test.bash"
        # Use single quotes to prevent expansion - tests the hook's literal tilde detection
        script.write_text(f"""
export HOME="{tmp_path / 'home' / 'user'}"
{hook}
builtin cd /  # Start at root
cd '~' 2>&1
echo "exit:$?"
""")
        (tmp_path / "home" / "user").mkdir(parents=True)
        result = subprocess.run(
            ["bash", "--noprofile", "--norc", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:1" in result.stdout
        assert "~" in result.stdout or "zkratky" in result.stdout.lower()

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_1_9_rejects_expanded_tilde(self, tmp_path: Path, user_dir: Path) -> None:
        """Test that expanded ~ (which becomes $HOME path) is rejected."""
        hook = generate_bash_cd_hook("1.9")
        hook = hook.replace('/tmp/shellgame-$USER', str(user_dir))
        home_dir = tmp_path / "home" / "user"
        home_dir.mkdir(parents=True)
        script = tmp_path / "test.bash"
        # Without quotes, bash expands ~ to $HOME before the function is called
        script.write_text(f"""
export HOME="{home_dir}"
{hook}
builtin cd /  # Start at root
cd ~ 2>&1
echo "exit:$?"
""")
        result = subprocess.run(
            ["bash", "--noprofile", "--norc", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:1" in result.stdout
        # Should show the tilde message, not the generic absolute path message
        assert "~" in result.stdout or "zkratky" in result.stdout.lower()

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_1_9_rejects_tilde(self, tmp_path: Path, user_dir: Path) -> None:
        hook = generate_fish_cd_hook("1.9")
        hook = hook.replace('/tmp/shellgame-$USER', str(user_dir))
        script = tmp_path / "test.fish"
        script.write_text(f"""
set -gx HOME "{tmp_path / 'home' / 'user'}"
{hook}
builtin cd /
cd "~" 2>&1
echo "exit:$status"
""")
        (tmp_path / "home" / "user").mkdir(parents=True)
        result = subprocess.run(
            ["fish", "--no-config", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:1" in result.stdout

    @pytest.mark.skipif(not shutil.which("bash"), reason="bash not installed")
    def test_bash_1_9_accepts_cd_to_root(self, tmp_path: Path, user_dir: Path) -> None:
        hook = generate_bash_cd_hook("1.9")
        hook = hook.replace('/tmp/shellgame-$USER', str(user_dir))
        script = tmp_path / "test.bash"
        script.write_text(f"""
export HOME="{tmp_path / 'home' / 'user'}"
{hook}
cd / 2>&1
echo "exit:$?"
echo "pwd:$(pwd)"
""")
        (tmp_path / "home" / "user").mkdir(parents=True)
        result = subprocess.run(
            ["bash", "--noprofile", "--norc", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:0" in result.stdout
        assert "pwd:/" in result.stdout

    @pytest.mark.skipif(not shutil.which("fish"), reason="fish not installed")
    def test_fish_1_9_accepts_cd_to_root(self, tmp_path: Path, user_dir: Path) -> None:
        hook = generate_fish_cd_hook("1.9")
        hook = hook.replace('/tmp/shellgame-$USER', str(user_dir))
        script = tmp_path / "test.fish"
        script.write_text(f"""
set -gx HOME "{tmp_path / 'home' / 'user'}"
{hook}
cd / 2>&1
echo "exit:$status"
echo "pwd:"(pwd)
""")
        (tmp_path / "home" / "user").mkdir(parents=True)
        result = subprocess.run(
            ["fish", "--no-config", str(script)],
            capture_output=True,
            text=True,
        )
        assert "exit:0" in result.stdout
        assert "pwd:/" in result.stdout


class TestCdHookErrorMessages:
    """Test that error messages are consistent between bash and fish."""

    def test_level_1_8_error_messages_are_similar(self) -> None:
        """Both shells should show similar error messages for 1.8."""
        bash_hook = generate_bash_cd_hook("1.8")
        fish_hook = generate_fish_cd_hook("1.8")

        # Both should mention "ShellGame (1.8)"
        assert "ShellGame (1.8)" in bash_hook
        assert "ShellGame (1.8)" in fish_hook

        # Both should mention absolute path requirement in Czech
        assert "absolutní" in bash_hook.lower()
        assert "absolutní" in fish_hook.lower()

    def test_level_1_9_error_messages_are_similar(self) -> None:
        """Both shells should show similar error messages for 1.9."""
        bash_hook = generate_bash_cd_hook("1.9")
        fish_hook = generate_fish_cd_hook("1.9")

        # Both should mention "ShellGame (1.9)"
        assert "ShellGame (1.9)" in bash_hook
        assert "ShellGame (1.9)" in fish_hook

        # Both should have consistent messaging about segments
        assert "segment" in bash_hook.lower()
        assert "segment" in fish_hook.lower()
