"""Shell client for communicating with the shell wrapper."""

import base64
import re
import sys
from pathlib import Path
from typing import TextIO

PROTOCOL_PREFIX = "__SHELLGAME_EXEC__"
PROTOCOL_VERSION = "v1"
_SUPPORTED_COMMANDS = frozenset({"cd", "export", "echo", "pwd", "exit"})
_ENV_NAME_PATTERN = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")


class ShellClient:
    def __init__(self, stream: TextIO = sys.stderr):
        self._stream = stream

    def _emit(self, command: str, *args: str | Path) -> None:
        if command not in _SUPPORTED_COMMANDS:
            raise ValueError(f"Unsupported shell protocol command: {command}")

        encoded_args = [base64.b64encode(str(arg).encode("utf-8")).decode("ascii") for arg in args]
        payload = " ".join((PROTOCOL_VERSION, command, *encoded_args))

        print(f"{PROTOCOL_PREFIX}{payload}", file=self._stream, flush=True)

    def cd(self, path: str | Path) -> None:
        self._emit("cd", path)

    def export(self, var_name: str, value: str) -> None:
        if not _ENV_NAME_PATTERN.fullmatch(var_name):
            raise ValueError(f"Invalid environment variable name: {var_name!r}")
        self._emit("export", var_name, value)

    def exit(self) -> None:
        self._emit("exit")
