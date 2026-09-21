"""Subshell integration (bash/fish) for ShellGame.

Responsibilities:
- Generate per-shell integration scripts (bash / fish) from templates
- Inject generated shell hooks (e.g. `cd` wrappers) into integration templates
- Launch a wrapped subshell with startup/banner suppression
"""

from __future__ import annotations

import os
import shlex
import subprocess
import sys
import tempfile
from importlib import resources
from pathlib import Path
from string import Template

from shellgame.cli.hooks import generate_bash_cd_hooks, generate_fish_cd_hooks

DEV_SHORTCUTS = {
    "r": "shellgame dev reload",
    "n": "shellgame dev next",
    "p": "shellgame dev prev",
    "j": "shellgame dev jump",
    "s": "shellgame dev start",
    "e": "shellgame exit",
    "g": "shellgame",
}


def _generate_bash_aliases(shortcuts: dict[str, str]) -> str:
    lines = ["# Dev shortcuts"]
    for alias, command in shortcuts.items():
        lines.append(f'alias {alias}="{command}"')
    return "\n".join(lines)


def _generate_fish_abbrs(shortcuts: dict[str, str]) -> str:
    lines = ["# Dev shortcuts"]
    for abbr, command in shortcuts.items():
        lines.append(f'abbr -a {abbr} "{command}"')
    return "\n".join(lines)


def _load_template(relative_path: str) -> str:
    package = "shellgame.cli.templates"
    return resources.files(package).joinpath(relative_path).read_text(encoding="utf-8")


def _render_template(
    template_text: str,
    *,
    binary_path: str,
    dev_shortcuts: str,
    cd_hook: str,
) -> str:
    return Template(template_text).safe_substitute(
        binary_path=binary_path,
        dev_shortcuts=dev_shortcuts,
        cd_hook=cd_hook,
    )


def _read_proc_comm(pid: int) -> str | None:
    try:
        with open(f"/proc/{pid}/comm") as f:
            return f.read().strip().lower()
    except OSError:
        return None


def detect_interactive_shell() -> str:
    zero = (os.environ.get("0") or "").lower()
    for shell in ("fish", "bash", "zsh"):
        if shell in zero:
            return shell

    parent = get_parent_shell()
    if parent in ("fish", "bash", "zsh"):
        return parent

    shell_env = (os.environ.get("SHELL") or "").lower()
    for shell in ("fish", "bash", "zsh"):
        if shell_env.endswith(f"/{shell}") or shell_env == shell:
            return shell

    return "unknown"


def _read_proc_stat_ppid(pid: int) -> int | None:
    try:
        raw = Path(f"/proc/{pid}/stat").read_text(encoding="utf-8", errors="replace")
        close = raw.rfind(")")
        if close == -1:
            return None

        tail = raw[close + 1 :].strip()
        if not tail:
            return None

        fields = tail.split()
        if len(fields) < 2:
            return None

        return int(fields[1])
    except (OSError, ValueError):
        return None


def get_parent_shell() -> str:
    pid = os.getppid()
    for _ in range(25):
        comm = _read_proc_comm(pid) or ""
        if "fish" in comm:
            return "fish"
        if "bash" in comm:
            return "bash"
        if "zsh" in comm:
            return "zsh"

        ppid = _read_proc_stat_ppid(pid)
        if not ppid or ppid <= 1 or ppid == pid:
            break
        pid = ppid

    return "unknown"


def get_fish_integration(binary_path: str, devmode: bool = False) -> str:
    template_text = _load_template("fish_integration.template")

    dev_shortcuts = ""
    if devmode:
        dev_shortcuts = _generate_fish_abbrs(DEV_SHORTCUTS)

    cd_hook = generate_fish_cd_hooks()

    return _render_template(
        template_text,
        binary_path=binary_path,
        dev_shortcuts=dev_shortcuts,
        cd_hook=cd_hook,
    )


def get_bash_integration(binary_path: str, devmode: bool = False) -> str:
    template_text = _load_template("bash_integration.template")

    dev_shortcuts = ""
    if devmode:
        dev_shortcuts = _generate_bash_aliases(DEV_SHORTCUTS)

    cd_hook = generate_bash_cd_hooks()

    return _render_template(
        template_text,
        binary_path=binary_path,
        dev_shortcuts=dev_shortcuts,
        cd_hook=cd_hook,
    )


def _generate_fish_init_command(script_path: str) -> str:
    return f"function fish_greeting; end; source {script_path}"


def _get_launcher_argv(devmode: bool) -> list[str]:
    argv0 = (sys.argv[0] or "").strip()
    if argv0 and argv0 not in ("-c", "-m"):
        launcher_argv = [os.path.abspath(argv0)]
        if launcher_argv[0].endswith(".py"):
            launcher_argv = [sys.executable, launcher_argv[0]]
    else:
        launcher_argv = [sys.executable, "-m", "shellgame"]

    if devmode:
        launcher_argv.append("--devmode")

    return launcher_argv


def _create_integration_script(shell_name: str, binary_path: str, devmode: bool) -> str:
    with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=f".{shell_name}") as f:
        if shell_name == "fish":
            f.write(get_fish_integration(binary_path, devmode))
        else:
            f.write(get_bash_integration(binary_path, devmode))
        return f.name


def _create_fd_hook_script(launcher_argv: list[str]) -> str:
    with tempfile.NamedTemporaryFile(mode="w", delete=False, suffix=".sh") as file:
        command = shlex.join([*launcher_argv, "fd-hook"])
        file.write(f"#!/bin/sh\nexec {command}\n")
        path = file.name
    os.chmod(path, 0o700)
    return path


def _print_debug_info(shell_name: str, devmode: bool, script_path: str) -> None:
    print(f"[shellgame][debug] launch_subshell(shell_name={shell_name!r}, devmode={devmode})")
    print(f"[shellgame][debug] integration_script_path={script_path}")
    try:
        preview = Path(script_path).read_text(encoding="utf-8", errors="replace")
        preview_lines = preview.splitlines()
        head = "\n".join(preview_lines[:40])
        print("[shellgame][debug] integration_script_preview (first 40 lines):")
        print(head)
    except OSError as e:
        print(f"[shellgame][debug] integration_script_preview_error={e!r}")


def _run_subshell(argv: list[str], env: dict[str, str], debug: bool, label: str) -> None:
    if debug:
        print(f"[shellgame][debug] {label}_argv={argv!r}")
    proc = subprocess.run(argv, check=False, env=env)
    if proc.returncode != 0:
        raise RuntimeError(f"{label.capitalize()} subshell exited with code {proc.returncode}")


def launch_subshell(shell_name: str, devmode: bool = False) -> None:
    launcher_argv = _get_launcher_argv(devmode)

    binary_path = " ".join(shlex.quote(p) for p in launcher_argv)

    debug = (os.environ.get("SHELLGAME_SUBSHELL_DEBUG") or "").strip() == "1"

    script_path = _create_integration_script(shell_name, binary_path, devmode)
    fd_hook_path: str | None = None

    try:
        fd_hook_path = _create_fd_hook_script(launcher_argv)
        if debug:
            _print_debug_info(shell_name, devmode, script_path)

        env = os.environ.copy()
        env["SHELLGAME_WRAPPER"] = "1"
        env["SHELLGAME_FD_HOOK"] = fd_hook_path

        # In PyInstaller onefile bundles, LD_LIBRARY_PATH is set to the extraction dir (_MEIPASS).
        # We must restore the original LD_LIBRARY_PATH so spawned shells and system binaries
        # (like flatpak, git, curl) use system libraries instead of bundled ones.
        if "LD_LIBRARY_PATH_ORIG" in env:
            env["LD_LIBRARY_PATH"] = env.pop("LD_LIBRARY_PATH_ORIG")
        else:
            env.pop("LD_LIBRARY_PATH", None)
        if "DYLD_LIBRARY_PATH_ORIG" in env:
            env["DYLD_LIBRARY_PATH"] = env.pop("DYLD_LIBRARY_PATH_ORIG")
        else:
            env.pop("DYLD_LIBRARY_PATH", None)

        if shell_name == "fish":
            _run_subshell(
                ["fish", "--init-command", _generate_fish_init_command(script_path)],
                env,
                debug,
                "fish",
            )
        elif shell_name == "bash":
            _run_subshell(["bash", "--rcfile", script_path, "-i"], env, debug, "bash")
        else:
            raise ValueError(f"Nepodporovaný shell: {shell_name}")
    finally:
        os.unlink(script_path)
        if fd_hook_path is not None:
            os.unlink(fd_hook_path)
