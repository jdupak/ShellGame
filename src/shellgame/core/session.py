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
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any, Callable, Protocol

from shellgame.core.navigation import NavigationManager
from shellgame.core.progress import ProgressTracker
from shellgame.levels.registry import LevelRegistry
from shellgame.shell.client import ShellClient
from shellgame.state.manager import GameState, StateManager
from shellgame.ui.display import Display
from shellgame.workspace.builder import WorkspaceManager


class SupportsPrint(Protocol):
    def print(self, *args: Any, **kwargs: Any) -> None: ...


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
        shell_client: ShellClient,
        workspace_factory: Callable[[str], WorkspaceManager],
    ) -> None:
        self._console = console
        self._display = display
        self._state_manager = state_manager
        self._level_registry = level_registry
        self._shell_client = shell_client
        self._workspace_factory = workspace_factory

        self._progress_tracker = ProgressTracker(state_manager)
        self._navigation_manager = NavigationManager(
            level_registry, shell_client, teleport_notice
        )

    # ---- helpers ---------------------------------------------------------

    def _get_level(self, level_id: str) -> Any:
        level = self._level_registry.get(level_id)
        if level is None:
            self._console.print(f"[red]Chyba: Level {level_id} nenalezen[/red]\n")
        return level

    def _export_shell_context(self, *, workspace: Path, level_id: str) -> None:
        """Export env for shell-side instrumentation (cd/pwd wrappers)."""
        self._shell_client.export("SHELLGAME_WORKSPACE", str(workspace))
        self._shell_client.export("SHELLGAME_LEVEL", level_id)

    def _initialize_game(self, username: str) -> GameState:
        """Initialize new game state and workspace."""
        state = self._state_manager.init(username)

        workspace_manager = self._workspace_factory(username)
        workspace_manager.init()

        first_level = self._level_registry.get(state.current_level)
        if first_level:
            first_level.setup(state.workspace)

        self._display.show_init_success(username, str(state.workspace))

        start_dir = self._navigation_manager.get_level_start_directory(
            state.current_level, state.workspace
        )
        if start_dir:
            self._navigation_manager.maybe_teleport(start_dir)
            self._display.show_current_directory(start_dir)

        return state

    def _auto_init_if_needed(self) -> GameState:
        """Load state or initialize everything if missing.

        Returns the loaded/created state object.
        """
        state = self._state_manager.load()
        if state:
            return state

        username = os.environ.get("USER", "player")
        return self._initialize_game(username)

    def _restore_workspace_if_missing(self, state: GameState) -> None:
        """If workspace is missing, rebuild and re-setup current level."""
        if state.workspace.exists():
            return

        workspace_manager = self._workspace_factory(state.username)
        workspace_manager.init()

        current_level = self._level_registry.get(state.current_level)
        if current_level:
            current_level.setup(state.workspace)

        self._display.show_workspace_restored(state.workspace)

    # ---- boot / show -----------------------------------------------------

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
            self._progress_tracker.ensure_level_started(
                state, level_id=level.id, now=datetime.now()
            )
        except Exception:
            # Timing is optional; gameplay should continue even if tracking fails.
            pass

        # Export instrumentation context
        self._export_shell_context(workspace=state.workspace, level_id=level.id)

        # Possibly teleport (only if outside workspace; level-dependent)
        self._navigation_manager.ensure_user_in_reasonable_place(state)

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
        self._initialize_game(username)

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

        hints = getattr(level, "hints", []) or []
        total_hints = len(hints)

        if total_hints == 0:
            self._display.show_no_more_hints()
            return

        if repeat:
            revealed = self._progress_tracker.get_hint_status(state, level.id, total_hints)
            self._display.show_repeated_hints(hints, revealed)
        else:
            # Reveal next hint
            revealed_idx = self._progress_tracker.reveal_next_hint(
                state, level.id, total_hints
            )
            if revealed_idx == -1:
                self._display.show_no_more_hints()
            else:
                self._display.show_hint(hints[revealed_idx], revealed_idx, total_hints)

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
        self._workspace_factory(state.username).remove()

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
                self._progress_tracker.record_attempt(state, level_id=level.id)
            except Exception:
                # Keep gameplay responsive even if tracking fails.
                pass

            self._display.show_failure(msg)
            return

        # Success: record completion (time/hints/attempts) centrally.
        with contextlib.suppress(Exception):
            self._progress_tracker.record_completion(
                state, level_id=level.id, completed_at=datetime.now()
            )

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

        # Section intros (.0) should not show a success panel after the intro text.
        if level.id.endswith(".0"):
            self.show_current_level()
            return

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

    def handle_cd_hook(
        self, *, target: str | None, pwd: str | None, post_move: bool
    ) -> None:
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
