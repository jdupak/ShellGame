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
    should_exit: bool
    exit_code: int = 0


def boot_if_needed(*, wrapped: bool, devmode: bool) -> BootResult:
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
