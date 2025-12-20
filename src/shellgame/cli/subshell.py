"""Subshell integration (bash/fish) for ShellGame.

Responsibilities:
- Generate per-shell integration scripts (bash / fish) from templates
- Inject generated shell hooks (e.g. `cd` wrappers) into integration templates
- Launch a wrapped subshell with startup/banner suppression

Error-handling policy:
- If launching the subshell fails (binary missing, permission issues, etc.),
  raise an exception and let the caller decide how to present the error.

Important notes:
- Bash must be launched only with bash-compatible flags. Never pass fish-style flags such
  as `--init-command` to bash.
- The integration script may contain a `shellgame` function; the subshell startup should
  call that function directly (simple and robust), rather than re-invoking the Python
  entrypoint again from bash startup.
- Determining the *actual* current interactive shell is tricky when wrappers (e.g. `uv`,
  `make`) get into the process tree. Prefer interactive-shell signals first, then fall back.

Debugging:
- If `SHELLGAME_SUBSHELL_DEBUG=1` is set, the subshell launcher prints:
  - the exact argv used to launch bash/fish
  - rcfile path and rcfile contents (bash)
  - integration script path and a short preview of its contents
to help diagnose wrapper/flag parsing issues.

Bash autostart policy:
- Bash wrapper autostart is triggered from the generated bash rcfile (not from the integration template),
  guarded by `SHELLGAME_AUTO_STARTED=1` and only in interactive shells. This avoids brittle behavior
  across different bash startup modes.
"""

from __future__ import annotations

from importlib import resources
from string import Template

import os
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Optional

from shellgame.cli.hooks import generate_bash_cd_hooks, generate_fish_cd_hooks


def _load_template(relative_path: str) -> str:
    """Load a bundled template from `shellgame.cli.templates`."""
    package = "shellgame.cli.templates"
    return resources.files(package).joinpath(relative_path).read_text(encoding="utf-8")


def _render_template(
    template_text: str,
    *,
    binary_path: str,
    dev_shortcuts: str,
    cd_hook: str,
) -> str:
    """Render a shell template using `string.Template`.

    We intentionally avoid Python `.format(...)` because shell code frequently uses `{}`.
    Templates should use these placeholders:
    - `$binary_path`
    - `$dev_shortcuts`
    - `$cd_hook`
    """
    return Template(template_text).safe_substitute(
        binary_path=binary_path,
        dev_shortcuts=dev_shortcuts,
        cd_hook=cd_hook,
    )


def _read_proc_comm(pid: int) -> Optional[str]:
    """Best-effort read of `/proc/<pid>/comm` (lowercased)."""
    try:
        with open(f"/proc/{pid}/comm", "r") as f:
            return f.read().strip().lower()
    except Exception:
        return None


def detect_interactive_shell() -> str:
    """Detect the *current* interactive shell (bash/fish) as robustly as possible.

    We prefer signals that reflect the *current* shell session:
    1) Environment variable `0` (often set by interactive shells to their name/path).
    2) Process-tree walk (find first bash/fish/zsh above us).
    3) Fallback to `$SHELL` (login shell; can be misleading inside nested shells).

    Returns:
        "fish", "bash", "zsh", or "unknown"
    """
    # 1) `0` env var (best indicator of the current shell in many interactive setups)
    try:
        zero = (os.environ.get("0") or "").lower()
        if "fish" in zero:
            return "fish"
        if "bash" in zero:
            return "bash"
        if "zsh" in zero:
            return "zsh"
    except Exception:
        pass

    # 2) Process tree (can be obscured by wrappers, but still useful)
    parent = get_parent_shell()
    if parent in ("fish", "bash", "zsh"):
        return parent

    # 3) Login shell fallback
    try:
        shell_env = (os.environ.get("SHELL") or "").lower()
        if shell_env.endswith("/fish") or shell_env == "fish":
            return "fish"
        if shell_env.endswith("/bash") or shell_env == "bash":
            return "bash"
        if shell_env.endswith("/zsh") or shell_env == "zsh":
            return "zsh"
    except Exception:
        pass

    return "unknown"


def _read_proc_stat_ppid(pid: int) -> Optional[int]:
    """Best-effort parse of parent pid from `/proc/<pid>/stat`.

    Linux `/proc/<pid>/stat` format (simplified):
      pid (comm) state ppid ...

    The tricky part is that `comm` is wrapped in parentheses and may contain spaces,
    so we must locate the *matching* closing `)` and parse fields after it.
    """
    try:
        raw = Path(f"/proc/{pid}/stat").read_text(encoding="utf-8", errors="replace")
        # Find the *first* '(' and the *last* ')' which closes comm.
        # Everything after that is space-separated fields starting with:
        #   state ppid ...
        close = raw.rfind(")")
        if close == -1:
            return None

        tail = raw[close + 1 :].strip()
        if not tail:
            return None

        fields = tail.split()
        # fields[0] = state, fields[1] = ppid
        if len(fields) < 2:
            return None

        return int(fields[1])
    except Exception:
        return None


def get_parent_shell() -> str:
    """Detect the parent shell process name.

    We walk up the process tree because the direct parent may be a wrapper
    (e.g. `uv`, `make`, `python`), not the interactive shell.
    """
    try:
        pid = os.getppid()
        # Hard cap to avoid infinite loops; process trees are shallow here.
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
    except Exception:
        pass

    return "unknown"


def get_fish_integration(binary_path: str, devmode: bool = False) -> str:
    """Return Fish shell integration script content."""
    template_text = _load_template("fish_integration.template")

    dev_shortcuts = ""
    if devmode:
        dev_shortcuts = _load_template("fish_devmode_abbrs.template")

    cd_hook = generate_fish_cd_hooks()

    return _render_template(
        template_text,
        binary_path=binary_path,
        dev_shortcuts=dev_shortcuts,
        cd_hook=cd_hook,
    )


def get_bash_integration(binary_path: str, devmode: bool = False) -> str:
    """Return Bash shell integration script content."""
    template_text = _load_template("bash_integration.template")

    dev_shortcuts = ""
    if devmode:
        dev_shortcuts = _load_template("bash_devmode_aliases.template")

    cd_hook = generate_bash_cd_hooks()

    return _render_template(
        template_text,
        binary_path=binary_path,
        dev_shortcuts=dev_shortcuts,
        cd_hook=cd_hook,
    )


def _generate_fish_init_command(script_path: str) -> str:
    """Generate the init command for fish."""
    # Suppress greeting and source integration.
    # Autostart is handled inside the integration template (at the end, after the
    # shellgame function is defined), so we don't call shellgame here.
    return f"function fish_greeting; end; source {script_path}"


def launch_subshell(shell_name: str, devmode: bool = False) -> None:
    """Launch a subshell with integration loaded.

    UX / gameplay requirements:
    - fish: suppress greeting by overriding fish_greeting in-session
    - bash: do not source user startup scripts
      - launch with: --noprofile --norc -i --rcfile <tempfile>

    NOTE:
    - Bash startup must NOT use fish-style flags (e.g. `--init-command`).
    - For bash, we source the integration and then call the `shellgame` shell function
      (defined by the integration). This avoids fragile command-string evaluation.

    Debug:
    - Set `SHELLGAME_SUBSHELL_DEBUG=1` to print the exact invocation + rcfile contents.
    """
    # Determine how the current process was invoked.
    # Prefer `sys.argv[0]` only when it looks like a real script/binary path;
    # otherwise fall back to the installed module entrypoint (`python -m shellgame`).
    argv0 = (sys.argv[0] or "").strip()
    if argv0 and argv0 not in ("-c", "-m"):
        launcher_argv = [os.path.abspath(argv0)]
        # If it is a Python file, prefix it with the interpreter.
        if launcher_argv[0].endswith(".py"):
            launcher_argv = [sys.executable, launcher_argv[0]]
    else:
        launcher_argv = [sys.executable, "-m", "shellgame"]

    # Append devmode flag if requested.
    if devmode:
        launcher_argv.append("--devmode")

    # Store as a space-separated, shell-escaped argv string.
    # If bash integration prefers calling the `shellgame` function directly, this value
    # is still useful (e.g. for debugging or future templates).
    binary_path = " ".join(shlex.quote(p) for p in launcher_argv)

    debug = (os.environ.get("SHELLGAME_SUBSHELL_DEBUG") or "").strip() == "1"

    # Create temp file for integration script
    with tempfile.NamedTemporaryFile(
        mode="w", delete=False, suffix=f".{shell_name}"
    ) as f:
        script_path = f.name
        if shell_name == "fish":
            f.write(get_fish_integration(binary_path, devmode))
        else:
            f.write(get_bash_integration(binary_path, devmode))

    if debug:
        # Print minimal, actionable debug info before launching the subshell.
        print(
            f"[shellgame][debug] launch_subshell(shell_name={shell_name!r}, devmode={devmode})"
        )
        print(f"[shellgame][debug] integration_script_path={script_path}")
        try:
            preview = Path(script_path).read_text(encoding="utf-8", errors="replace")
            preview_lines = preview.splitlines()
            head = "\n".join(preview_lines[:40])
            print("[shellgame][debug] integration_script_preview (first 40 lines):")
            print(head)
        except Exception as e:
            print(f"[shellgame][debug] integration_script_preview_error={e!r}")

    try:
        env = os.environ.copy()
        env["SHELLGAME_WRAPPER"] = "1"

        if shell_name == "fish":
            argv = [
                "fish",
                "--init-command",
                _generate_fish_init_command(script_path),
            ]
            if debug:
                print(f"[shellgame][debug] fish_argv={argv!r}")
            proc = subprocess.run(argv, env=env)
            # In tests we stub subprocess.run() to return None.
            if proc is not None and getattr(proc, "returncode", 0) != 0:
                raise RuntimeError(
                    f"Fish subshell exited with code {getattr(proc, 'returncode', 'unknown')}"
                )
        elif shell_name == "bash":
            # Use the integration script directly as rcfile (it now includes
            # shell options and autostart, matching the fish approach).
            # NOTE: Do NOT use --norc here - it disables --rcfile entirely.
            # We also don't use --noprofile - let users keep their PATH/env setup.
            argv = ["bash", "--rcfile", script_path, "-i"]
            if debug:
                print(f"[shellgame][debug] bash_argv={argv!r}")
            proc = subprocess.run(argv, env=env)
            # In tests we stub subprocess.run() to return None.
            if proc is not None and getattr(proc, "returncode", 0) != 0:
                raise RuntimeError(
                    f"Bash subshell exited with code {getattr(proc, 'returncode', 'unknown')}"
                )
        else:
            raise ValueError(f"Nepodporovaný shell: {shell_name}")
    finally:
        os.unlink(script_path)
