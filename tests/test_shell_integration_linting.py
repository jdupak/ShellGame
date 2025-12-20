import os
import shutil
import subprocess
from typing import Optional

import pytest

from shellgame.cli.subshell import get_bash_integration, get_fish_integration


def _binary_path_for_tests() -> str:
    # We don't actually execute `shellgame` in these tests; we just validate
    # that the generated shell integration scripts are syntactically valid.
    return "shellgame"


def _run(
    cmd: list[str], *, input_text: Optional[str] = None
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        input=input_text,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


@pytest.mark.optional_shell_lint
def test_bash_integration_passes_shellcheck() -> None:
    shellcheck = shutil.which("shellcheck")
    if not shellcheck:
        pytest.skip("shellcheck not installed")

    script = get_bash_integration(_binary_path_for_tests(), devmode=False)
    # -s bash: treat as bash
    # -: read from stdin
    proc = _run([shellcheck, "-S", "error", "-s", "bash", "-"], input_text=script)
    assert proc.returncode == 0, proc.stdout + proc.stderr


@pytest.mark.optional_shell_lint
def test_bash_integration_devmode_passes_shellcheck() -> None:
    shellcheck = shutil.which("shellcheck")
    if not shellcheck:
        pytest.skip("shellcheck not installed")

    script = get_bash_integration(_binary_path_for_tests(), devmode=True)
    proc = _run([shellcheck, "-S", "error", "-s", "bash", "-"], input_text=script)
    assert proc.returncode == 0, proc.stdout + proc.stderr


@pytest.mark.optional_shell_lint
def test_fish_integration_has_valid_syntax() -> None:
    fish = shutil.which("fish")
    if not fish:
        pytest.skip("fish not installed")

    script = get_fish_integration(_binary_path_for_tests(), devmode=False)

    # `fish -n` parses the script for syntax errors without executing it.
    # We pipe the script via stdin.
    proc = _run([fish, "-n"], input_text=script)
    assert proc.returncode == 0, proc.stdout + proc.stderr


@pytest.mark.optional_shell_lint
def test_fish_integration_devmode_has_valid_syntax() -> None:
    fish = shutil.which("fish")
    if not fish:
        pytest.skip("fish not installed")

    script = get_fish_integration(_binary_path_for_tests(), devmode=True)
    proc = _run([fish, "-n"], input_text=script)
    assert proc.returncode == 0, proc.stdout + proc.stderr
