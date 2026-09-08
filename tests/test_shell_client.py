"""Tests for the versioned shell protocol encoder."""

import base64
from io import StringIO

import pytest

from shellgame.shell.client import PROTOCOL_PREFIX, ShellClient


def _decode_directive(line: str) -> tuple[str, list[str]]:
    payload = line.removeprefix(PROTOCOL_PREFIX).split()
    assert payload[0] == "v1"
    return payload[1], [base64.b64decode(arg).decode() for arg in payload[2:]]


def test_cd_encodes_path_without_shell_parsing() -> None:
    stream = StringIO()
    client = ShellClient(stream)

    client.cd("/tmp/path with spaces;$(ignored)")

    assert _decode_directive(stream.getvalue().strip()) == (
        "cd",
        ["/tmp/path with spaces;$(ignored)"],
    )


def test_export_encodes_name_and_value_separately() -> None:
    stream = StringIO()
    client = ShellClient(stream)

    client.export("SHELLGAME_LEVEL", "value with spaces")

    assert _decode_directive(stream.getvalue().strip()) == (
        "export",
        ["SHELLGAME_LEVEL", "value with spaces"],
    )


def test_export_rejects_invalid_variable_name() -> None:
    with pytest.raises(ValueError, match="Invalid environment variable"):
        ShellClient(StringIO()).export("BAD-NAME", "value")


def test_protocol_rejects_unknown_command() -> None:
    with pytest.raises(ValueError, match="Unsupported shell protocol"):
        ShellClient(StringIO())._emit("eval", "echo unsafe")
