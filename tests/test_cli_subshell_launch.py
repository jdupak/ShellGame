"""Tests for subshell launching behavior.

CRITICAL REQUIREMENTS (do not break):

- bash must be launched with:
  `bash --rcfile <integration_script> -i`

  NEVER use --norc (it disables --rcfile entirely!)
  NEVER use --noprofile (let users keep their PATH/env setup)

  The integration script is used directly as the rcfile - there is NO separate
  rc template. Autostart is at the END of the integration template.

- fish must suppress greeting and source the integration script:
  `fish --init-command "function fish_greeting; end; source <script>"`

  Autostart is inside the integration template (not in init-command).

Both templates follow the same pattern:
1. Shell setup
2. Function definitions (shellgame, pwd, cd hooks)
3. Autostart block at the END (after functions are defined)

These tests intentionally do not spawn real shells; they assert the subprocess
invocations and temporary file behavior.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

import shellgame.cli.subshell as subshell


@dataclass
class _RunCall:
    args: list[str]
    env: dict[str, str] | None


class _FakeNamedTemp:
    """Minimal tempfile.NamedTemporaryFile stand-in.

    Supports context-manager protocol and provides:
    - `.name` path
    - `.write(str)` for collecting written content
    """

    def __init__(self, name: str) -> None:
        self.name = name
        self._writes: list[str] = []

    def __enter__(self) -> "_FakeNamedTemp":
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        return None

    def write(self, s: str) -> int:
        self._writes.append(s)
        return len(s)

    @property
    def content(self) -> str:
        return "".join(self._writes)


class _TempFactory:
    """Factory that returns deterministic fake tempfiles in call order."""

    def __init__(self) -> None:
        self.created: list[_FakeNamedTemp] = []
        self._counter = 0

    def __call__(self, *args: Any, **kwargs: Any) -> _FakeNamedTemp:  # noqa: ANN401
        self._counter += 1
        suffix = kwargs.get("suffix", "")
        name = f"/tmp/fake-tmp-{self._counter}{suffix}"
        f = _FakeNamedTemp(name)
        self.created.append(f)
        return f


class _UnlinkRecorder:
    def __init__(self) -> None:
        self.paths: list[str] = []

    def __call__(self, path: str) -> None:
        self.paths.append(path)


class _Console:
    def __init__(self) -> None:
        self.printed: list[str] = []

    def print(self, msg: str) -> None:
        self.printed.append(msg)


def test_launch_subshell_bash_critical_flags(monkeypatch) -> None:
    """CRITICAL: Verify bash is launched correctly.

    - Must use --rcfile with integration script directly
    - Must NOT use --norc (disables --rcfile!)
    - Must NOT use --noprofile (breaks user PATH/env)
    - Must use -i for interactive mode
    """
    calls: list[_RunCall] = []
    temp_factory = _TempFactory()
    unlink = _UnlinkRecorder()
    console = _Console()

    def fake_run(args: list[str], env: dict[str, str] | None = None) -> None:
        calls.append(_RunCall(args=args, env=env))

    monkeypatch.setattr(subshell.tempfile, "NamedTemporaryFile", temp_factory)
    monkeypatch.setattr(subshell.subprocess, "run", fake_run)
    monkeypatch.setattr(subshell.os, "unlink", unlink)
    monkeypatch.setattr(subshell.os.path, "abspath", lambda _: "/fake/shellgame")
    monkeypatch.setattr(subshell.sys, "argv", ["/fake/shellgame"])
    monkeypatch.setattr(subshell.os, "environ", {"USER": "u", "PATH": "/bin"})

    subshell.launch_subshell("bash", devmode=False)

    # One temp file: integration script (now used directly as rcfile).
    assert len(temp_factory.created) == 1
    script_tmp = temp_factory.created[0]

    # bash invocation: no special suppression flags.
    # NOTE: We do NOT use --norc (disables --rcfile) or --noprofile (breaks user PATH/env).
    assert len(calls) == 1

    args = calls[0].args
    assert args[0] == "bash"
    assert "--norc" not in args
    assert "--noprofile" not in args

    # Must include: --rcfile <script_tmp.name> and -i
    assert "-i" in args

    rc_idx = args.index("--rcfile")
    assert args[rc_idx + 1] == script_tmp.name

    # Ensure env is passed and wrapper flag is set.
    assert calls[0].env is not None
    assert calls[0].env.get("SHELLGAME_WRAPPER") == "1"

    # Temp file is cleaned up.
    assert script_tmp.name in unlink.paths

    # console should not be used for supported shell flow
    assert console.printed == []


def test_launch_subshell_fish_suppresses_greeting_and_sources_script(
    monkeypatch,
) -> None:
    calls: list[_RunCall] = []
    temp_factory = _TempFactory()
    unlink = _UnlinkRecorder()
    console = _Console()

    def fake_run(args: list[str], env: dict[str, str] | None = None) -> None:
        calls.append(_RunCall(args=args, env=env))

    monkeypatch.setattr(subshell.tempfile, "NamedTemporaryFile", temp_factory)
    monkeypatch.setattr(subshell.subprocess, "run", fake_run)
    monkeypatch.setattr(subshell.os, "unlink", unlink)
    monkeypatch.setattr(subshell.os.path, "abspath", lambda _: "/fake/shellgame")
    monkeypatch.setattr(subshell.sys, "argv", ["/fake/shellgame"])
    monkeypatch.setattr(subshell.os, "environ", {"USER": "u", "PATH": "/bin"})

    subshell.launch_subshell("fish", devmode=False)

    # One temp file: integration script
    assert len(temp_factory.created) == 1
    script_tmp = temp_factory.created[0]

    assert len(calls) == 1
    assert calls[0].args[0:2] == ["fish", "--init-command"]

    init_cmd = calls[0].args[2]
    # Must override fish_greeting and source integration.
    # Autostart is now handled inside the integration template (not in init_cmd).
    assert "function fish_greeting; end;" in init_cmd
    assert f"source {script_tmp.name}" in init_cmd

    # Ensure env is passed and wrapper flag is set.
    assert calls[0].env is not None
    assert calls[0].env.get("SHELLGAME_WRAPPER") == "1"

    # Temp file is cleaned up.
    assert unlink.paths == [script_tmp.name]

    assert console.printed == []


def test_launch_subshell_unknown_shell_raises_and_cleans_script(monkeypatch) -> None:
    calls: list[_RunCall] = []
    temp_factory = _TempFactory()
    unlink = _UnlinkRecorder()

    def fake_run(args: list[str], env: dict[str, str] | None = None) -> None:
        calls.append(_RunCall(args=args, env=env))

    monkeypatch.setattr(subshell.tempfile, "NamedTemporaryFile", temp_factory)
    monkeypatch.setattr(subshell.subprocess, "run", fake_run)
    monkeypatch.setattr(subshell.os, "unlink", unlink)
    monkeypatch.setattr(subshell.os.path, "abspath", lambda _: "/fake/shellgame")
    monkeypatch.setattr(subshell.sys, "argv", ["/fake/shellgame"])
    monkeypatch.setattr(subshell.os, "environ", {"USER": "u", "PATH": "/bin"})

    try:
        subshell.launch_subshell("zsh", devmode=False)
        assert False, "expected ValueError"
    except ValueError as e:
        assert "Nepodporovaný shell" in str(e)

    # No subprocess call for unsupported shell.
    assert calls == []

    # Script tempfile is still created and cleaned up.
    assert len(temp_factory.created) == 1
    script_tmp = temp_factory.created[0]
    assert unlink.paths == [script_tmp.name]
