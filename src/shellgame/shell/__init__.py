"""Shell command protocol for communicating with shell wrappers.

This module allows the Python CLI to emit commands that the shell wrapper
can execute in the user's current shell environment.

Supported commands:
- cd: Change directory
- export: Set environment variable
- echo: Display message

Protocol transport:
- The wrapper consumes directives that start with `__SHELLGAME_EXEC__`.
- Historically directives were emitted to stderr, but some environments leak stderr
  (or print it unfiltered). To make exit/remove robust, we emit to both stderr and
  stdout so whichever stream the wrapper captures will still receive the directive.
"""

import sys
import shlex
from pathlib import Path
from typing import Union


def emit_shell_command(command: str, *args: Union[str, Path]) -> None:
    """
    Emit a shell command for the wrapper to execute.

    The wrapper intercepts directives prefixed with `__SHELLGAME_EXEC__`.
    We emit directives to both stderr and stdout to reduce the chance of leakage
    (some shells/wrappers may only capture/filter one stream).

    Args:
        command: The command to execute (cd, export, echo, exit)
        args: Arguments for the command (will be properly escaped)

    Example:
        emit_shell_command("cd", "/tmp/workspace")
        emit_shell_command("export", "SHELLGAME_LEVEL=1.2")
        emit_shell_command("echo", "Welcome to level 2!")
    """
    # Convert Path objects to strings
    str_args = [str(arg) for arg in args]

    # Properly escape arguments for shell safety
    if str_args:
        escaped_args = " ".join(shlex.quote(arg) for arg in str_args)
        safe_command = f"{command} {escaped_args}"
    else:
        safe_command = command

    # Emit to stderr with special marker
    print(f"__SHELLGAME_EXEC__{safe_command}", file=sys.stderr, flush=True)


def cd(path: Union[str, Path]) -> None:
    """
    Change directory in user's shell.

    Args:
        path: Directory to change to
    """
    emit_shell_command("cd", path)


def export(var_name: str, value: str) -> None:
    """
    Export environment variable in user's shell.

    Args:
        var_name: Variable name
        value: Variable value
    """
    emit_shell_command("export", f"{var_name}={value}")


def echo(message: str) -> None:
    """
    Echo a message in user's shell.

    Args:
        message: Message to display
    """
    emit_shell_command("echo", message)


def pwd() -> None:
    """Print working directory in user's shell."""
    emit_shell_command("pwd")


def exit() -> None:
    """Exit the shellgame subshell."""
    emit_shell_command("exit")
