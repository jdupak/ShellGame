"""
Boot orchestration for ShellGame.

This module handles the process wrapping and subshell launching logic.
It decides whether to launch a subshell or run the game logic in the current process.
"""

import os
from dataclasses import dataclass

from shellgame.cli.subshell import detect_interactive_shell, launch_subshell


@dataclass(frozen=True, slots=True)
class BootResult:
    """Result of boot orchestration.

    `should_exit` is primarily used by the CLI group entrypoint: if we launched
    a wrapped subshell, the outer process should exit.
    """

    should_exit: bool
    exit_code: int = 0


def boot_if_needed(*, wrapped: bool, devmode: bool) -> BootResult:
    """Handle the 'outer process' boot check.

    If not already wrapped (`SHELLGAME_WRAPPER` not set), the CLI should call this,
    then exit if it returns `should_exit=True`.

    Shell selection policy:
    - If an explicit override is provided via `SHELLGAME_FORCE_SHELL`, it is respected.
    - Otherwise, prefer robust interactive-shell detection (works even under wrappers like `uv`/`make`).
    - Fallback to bash if the shell cannot be determined.
    """
    if wrapped:
        return BootResult(should_exit=False)

    forced = (os.environ.get("SHELLGAME_FORCE_SHELL") or "").strip().lower()
    if forced in ("bash", "fish"):
        target_shell = forced
    else:
        detected = detect_interactive_shell()
        target_shell = detected if detected in ("fish", "bash") else "bash"

    launch_subshell(target_shell, devmode)
    return BootResult(should_exit=True, exit_code=0)
