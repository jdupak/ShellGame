"""Shell client for communicating with the shell wrapper."""

import shlex
import sys
from pathlib import Path
from typing import TextIO, Union


class ShellClient:
    """Client for communicating with the shell wrapper via protocol directives.

    Encapsulates the __SHELLGAME_EXEC__ protocol.
    """

    def __init__(self, stream: TextIO = sys.stderr):
        """Initialize shell client.

        Args:
            stream: Output stream to write directives to (default: stderr)
        """
        self._stream = stream

    def _emit(self, command: str, *args: Union[str, Path]) -> None:
        """Emit a command directive to the configured stream."""
        str_args = [str(arg) for arg in args]

        if str_args:
            escaped_args = " ".join(shlex.quote(arg) for arg in str_args)
            safe_command = f"{command} {escaped_args}"
        else:
            safe_command = command

        print(f"__SHELLGAME_EXEC__{safe_command}", file=self._stream, flush=True)

    def cd(self, path: Union[str, Path]) -> None:
        """Change directory in user's shell."""
        self._emit("cd", path)

    def export(self, var_name: str, value: str) -> None:
        """Export environment variable in user's shell."""
        self._emit("export", f"{var_name}={value}")

    def echo(self, message: str) -> None:
        """Echo a message in user's shell."""
        self._emit("echo", message)

    def pwd(self) -> None:
        """Print working directory in user's shell."""
        self._emit("pwd")

    def exit(self) -> None:
        """Exit the shellgame subshell."""
        self._emit("exit")
