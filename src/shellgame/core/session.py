"""
Core game session orchestration.

This module encapsulates "what the game does" (boot + game actions) independently
from Click command parsing. The CLI layer should be a thin wrapper around this.

Design goals:
- Keep all side-effectful gameplay operations in one place.
- Preserve existing UX/gameplay behavior.
- Make it easier to test/maintain commands without turning `commands.py` into a god file.

This file intentionally does NOT define Click commands.
"""

from __future__ import annotations

import contextlib
import os
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Callable, Protocol

from shellgame import shell
from shellgame.cli.subshell import detect_interactive_shell, launch_subshell
from shellgame.levels.registry import LevelRegistry
from shellgame.markers import MarkerManager
from shellgame.state.manager import GameState, StateManager
from shellgame.ui.display import Display
from shellgame.workspace.builder import WorkspaceManager


class SupportsPrint(Protocol):
    def print(self, *args: Any, **kwargs: Any) -> None: ...


@dataclass(frozen=True, slots=True)
class BootResult:
    """Result of boot orchestration.

    `should_exit` is primarily used by the CLI group entrypoint: if we launched
    a wrapped subshell, the outer process should exit.
    """

    should_exit: bool
    exit_code: int = 0


class GameSession:
    """High-level orchestration for ShellGame.

    This class centralizes:
    - Initialization (state + workspace creation)
    - Showing current assignment
    - Exporting shell-side instrumentation env vars
    - Level start directory enforcement / teleport behavior
    - Core actions: hint / submit / reset / repeat / status / remove

    The CLI should:
    - Construct `GameSession` with the established singletons
    - Delegate to these methods
    """

    def __init__(
        self,
        *,
        console: SupportsPrint,
        display: Display,
        state_manager: StateManager,
        level_registry: LevelRegistry,
        teleport_notice: Callable[[Path], None],
    ) -> None:
        self._console = console
        self._display = display
        self._state_manager = state_manager
        self._level_registry = level_registry
        self._teleport_notice = teleport_notice

    # ---- helpers ---------------------------------------------------------

    def _get_level(self, level_id: str) -> Any:
        level = self._level_registry.get(level_id)
        if level is None:
            self._console.print(f"[red]Chyba: Level {level_id} nenalezen[/red]\n")
        return level

    def _get_level_start_directory(self, level_id: str, workspace: Path) -> Path | None:
        """Resolve start dir via the level definition (preferred)."""
        level = self._level_registry.get(level_id)
        if level:
            return level.get_start_directory(workspace)
        return None

    def _export_shell_context(self, *, workspace: Path, level_id: str) -> None:
        """Export env for shell-side instrumentation (cd/pwd wrappers)."""
        shell.export("SHELLGAME_WORKSPACE", str(workspace))
        shell.export("SHELLGAME_LEVEL", level_id)

    def _maybe_teleport(self, *, start_dir: Path | None) -> None:
        """Teleport to a start directory if requested and needed."""
        if not start_dir:
            return

        before_dir = Path.cwd()
        shell.cd(start_dir)
        after_dir = Path.cwd()
        if after_dir != before_dir:
            self._teleport_notice(start_dir)

    def _ensure_user_in_reasonable_place(self, *, state: GameState) -> None:
        """Ensure player is in the configured level start directory when appropriate.

        Policy:
        - If a level defines `start_directory`, we normally place the player there.
        - Some navigation-focused levels should NOT auto-teleport because it would
          trivialize the task (explicit allowlist/denylist).
        """
        # Some navigation-focused levels should NOT auto-teleport.
        no_autocd_levels = {"1.5"}

        start_dir = self._get_level_start_directory(state.current_level, state.workspace)
        if not start_dir or state.current_level in no_autocd_levels:
            return

        # Always attempt to enter the level's start directory; notice is only shown
        # if cwd actually changed (handled inside _maybe_teleport).
        self._maybe_teleport(start_dir=start_dir)

    def _auto_init_if_needed(self) -> GameState:
        """Load state or initialize everything if missing.

        Returns the loaded/created state object.
        """
        state = self._state_manager.load()
        if state:
            return state

        username = os.environ.get("USER", "player")
        state = self._state_manager.init(username)

        workspace_manager = WorkspaceManager(username)
        workspace_manager.init()

        first_level = self._level_registry.get(state.current_level)
        if first_level:
            first_level.setup(state.workspace)

        self._display.show_init_success(username, str(state.workspace))

        # Auto-teleport to first start dir (if defined) and show current dir.
        start_dir = self._get_level_start_directory(state.current_level, state.workspace)
        if start_dir:
            self._maybe_teleport(start_dir=start_dir)
            self._console.print(f"[dim]Aktuální adresář: {start_dir}[/dim]")

        return state

    def _restore_workspace_if_missing(self, state: GameState) -> None:
        """If workspace is missing, rebuild and re-setup current level."""
        if state.workspace.exists():
            return

        self._console.print("[yellow]⚠ Pracovní prostor byl smazán (např. restart systému). Obnovuji...[/yellow]")

        workspace_manager = WorkspaceManager(state.username)
        workspace_manager.init()

        current_level = self._level_registry.get(state.current_level)
        if current_level:
            current_level.setup(state.workspace)

        self._console.print(f"[green]✓ Pracovní prostor obnoven: {state.workspace}[/green]\n")

    # ---- boot / show -----------------------------------------------------

    def boot_if_needed(self, *, wrapped: bool, devmode: bool, parent_shell: str) -> BootResult:
        """Handle the 'outer process' boot check.

        If not already wrapped (`SHELLGAME_WRAPPER` not set), the CLI should call this,
        then exit if it returns `should_exit=True`.

        Shell selection policy:
        - If an explicit override is provided via `SHELLGAME_FORCE_SHELL`, it is respected.
        - Otherwise, prefer robust interactive-shell detection (works even under wrappers like `uv`/`make`).
        - Fallback to bash if the shell cannot be determined.

        `parent_shell` is accepted for backwards compatibility with the CLI layer but is
        not used as the primary signal.
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

    def show_current_level(self) -> None:
        """Default action when user runs `shellgame` with no subcommand."""
        state = self._auto_init_if_needed()
        self._restore_workspace_if_missing(state)

        level = self._get_level(state.current_level)
        if level is None:
            return

        self._display.show_instructions(level)

        # Start per-level timer as soon as we "enter" the level (show it).
        try:
            self._state_manager.ensure_level_started(state, level_id=level.id, now=datetime.now())
            self._state_manager.save(state)
        except Exception:
            # Timing is optional; gameplay should continue even if tracking fails.
            pass

        # Export instrumentation context
        self._export_shell_context(workspace=state.workspace, level_id=level.id)

        # Possibly teleport (only if outside workspace; level-dependent)
        self._ensure_user_in_reasonable_place(state=state)

        # Auto-advance for intro levels
        if str(state.current_level).endswith(".0"):
            self._display.wait_for_continue()
            # Mirror CLI behavior: intro levels auto-submit with no args
            self.submit(answer=None)

    # ---- CLI command actions --------------------------------------------

    def init(self) -> None:
        """Initialize a new game state + workspace (legacy command)."""
        existing = self._state_manager.load()
        if existing:
            self._display.show_already_initialized()
            return

        username = os.environ.get("USER", "player")
        state = self._state_manager.init(username)

        workspace_manager = WorkspaceManager(username)
        workspace_manager.init()

        first_level = self._level_registry.get(state.current_level)
        if first_level:
            first_level.setup(state.workspace)

        self._display.show_init_success(username, str(state.workspace))

        start_dir = self._get_level_start_directory(state.current_level, state.workspace)
        if start_dir:
            self._maybe_teleport(start_dir=start_dir)
            self._console.print(f"[dim]Aktuální adresář: {start_dir}[/dim]")

    def status(self) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        self._display.show_status(state)

    def hint(self, *, repeat: bool = False) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        level = self._get_level(state.current_level)
        if level is None:
            return

        # Delegate hint semantics + rendering to Display (progressive + --repeat).
        # Display mutates `state.level_hints_used[level_id]` when a new hint is consumed.
        before_hints = dict(getattr(state, "level_hints_used", {}) or {})
        self._display.show_level_hint(level, state, repeat=repeat)
        after_hints = dict(getattr(state, "level_hints_used", {}) or {})

        # Persist only if we actually consumed a new hint.
        if after_hints != before_hints:
            self._state_manager.save(state)

    def reset(self) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        level = self._get_level(state.current_level)
        if level is None:
            return

        level.reset(state.workspace)
        self._display.show_reset(state.current_level)
        self._display.show_instructions(level)

    def repeat(self, *, section_num: int | None, level_id: str | None) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        # Behavior: level_id overrides section_num; if neither, use current level.
        resolved: str
        if level_id:
            resolved = level_id
        elif section_num is not None:
            resolved = f"{section_num}.0"
        else:
            resolved = state.current_level

        level = self._get_level(resolved)
        if level is None:
            return

        self._display.show_instructions(level)

    def remove(self) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        # Confirmation is handled by Click at the CLI layer.
        #
        # The shell wrapper (fish/bash) detects `remove` command success and exits
        # the subshell directly. We don't emit shell.exit() here because Click's
        # confirmation prompt can interfere with stderr capture timing, causing
        # protocol directive leakage.

        # Remove workspace + state
        self._state_manager.remove()
        WorkspaceManager(state.username).remove()

        self._display.show_removed()

    def submit(self, answer: str | None) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        level = self._get_level(state.current_level)
        if level is None:
            return

        # Validate via level, preserving per-level semantics.
        ok, msg = level.validate(answer, state)

        if not ok:
            # Centralize attempt tracking: every failed validation counts as an attempt.
            try:
                self._state_manager.record_attempt(state, level_id=level.id)
                self._state_manager.save(state)
            except Exception:
                # Keep gameplay responsive even if tracking fails.
                pass

            self._display.show_failure(msg)
            return

        # Success: record completion (time/hints/attempts) centrally in StateManager.
        with contextlib.suppress(Exception):
            self._state_manager.record_completion(state, level_id=level.id, completed_at=datetime.now())

        # Advance level
        next_level_id = self._level_registry.next_level(level.id)
        if next_level_id is None:
            self._display.show_success("Hotovo! Dokončili jste všechny levely.")
            self._state_manager.save(state)
            return

        state.current_level = next_level_id
        self._state_manager.save(state)

        # Setup next level
        next_level = self._level_registry.get(next_level_id)
        if next_level:
            next_level.setup(state.workspace)

        self._display.show_success("Správně!")
        self._display.wait_for_continue()
        # After success, show next instructions immediately (existing behavior)
        self.show_current_level()

    # ---- dev helpers (optional; leave for CLI to call) -------------------

    def dev_jump_to(self, level_id: str) -> None:
        """Developer helper: jump to a level (if devmode enables it)."""
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        level = self._level_registry.get(level_id)
        if not level:
            self._console.print(f"[red]Neznámý level: {level_id}[/red]")
            return

        state.current_level = level_id
        self._state_manager.save(state)
        level.setup(state.workspace)
        self.show_current_level()

    def handle_cd_hook(self, *, target: str | None, pwd: str | None, post_move: bool) -> None:
        """Handle shell directory change hooks.

        This logic was previously embedded in shell templates.
        """
        state = self._state_manager.load()
        if not state:
            return

        level = self._get_level(state.current_level)
        if not level:
            return

        hook = level.hooks.get("cd")
        if hook:
            hook(target=target, pwd=pwd, post_move=post_move, state=state)

