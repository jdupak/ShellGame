"""Shell client for communicating with the shell wrapper."""

import shlex
import sys
from pathlib import Path
from typing import TextIO, Union


class ShellClient:
    def __init__(self, stream: TextIO = sys.stderr):
        self._stream = stream

    def _emit(self, command: str, *args: Union[str, Path]) -> None:
        str_args = [str(arg) for arg in args]

        if str_args:
            escaped_args = " ".join(shlex.quote(arg) for arg in str_args)
            safe_command = f"{command} {escaped_args}"
        else:
            safe_command = command

        print(f"__SHELLGAME_EXEC__{safe_command}", file=self._stream, flush=True)

    def cd(self, path: Union[str, Path]) -> None:
        self._emit("cd", path)

    def export(self, var_name: str, value: str) -> None:
        self._emit("export", f"{var_name}={value}")

    def echo(self, message: str) -> None:
        self._emit("echo", message)

    def pwd(self) -> None:
        self._emit("pwd")

    def exit(self) -> None:
        self._emit("exit")
