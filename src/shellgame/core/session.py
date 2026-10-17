"""
Core game session orchestration.

This module encapsulates "what the game does" (boot + game actions) independently
from Click command parsing. The CLI layer should be a thin wrapper around this.
"""

from __future__ import annotations

import os
from collections.abc import Callable
from datetime import datetime
from enum import Enum, auto
from pathlib import Path
from typing import Any, Protocol

from shellgame.core.navigation import NavigationManager
from shellgame.core.progress import ProgressTracker
from shellgame.levels.base import Level
from shellgame.levels.registry import LevelRegistry, UnknownLevelError, parse_level_id
from shellgame.paths import ContainedPathError
from shellgame.shell.client import ShellClient
from shellgame.state.manager import GameState, LevelCompletion, StateLoadError, StateManager
from shellgame.ui.display import Display
from shellgame.workspace.builder import WorkspaceManager


class SupportsPrint(Protocol):
    def print(self, *args: Any, **kwargs: Any) -> None: ...


class _LevelSync(Enum):
    """Outcome of checking a saved `current_level` against the registry."""

    VALID = auto()
    RESYNCED = auto()
    UNAVAILABLE = auto()


class GameSession:
    def __init__(  # noqa: PLR0913
        self,
        *,
        console: SupportsPrint,
        display: Display,
        state_manager: StateManager,
        level_registry: LevelRegistry,
        teleport_notice: Callable[[Path], None],
        shell_client: ShellClient,
        workspace_factory: Callable[[Path], WorkspaceManager],
    ) -> None:
        self._console = console
        self._display = display
        self._state_manager = state_manager
        self._level_registry = level_registry
        self._shell_client = shell_client
        self._workspace_factory = workspace_factory

        self._progress_tracker = ProgressTracker()
        self._navigation_manager = NavigationManager(level_registry, shell_client, teleport_notice)

    def _get_level(self, level_id: str) -> Level | None:
        level = self._level_registry.get(level_id)
        if level is None:
            self._console.print(f"[red]Chyba: Level {level_id} nenalezen[/red]\n")
        return level

    def _export_shell_context(self, *, workspace: Path, level_id: str) -> None:
        self._shell_client.export("SHELLGAME_WORKSPACE", str(workspace))
        self._shell_client.export("SHELLGAME_LEVEL", level_id)

    def _initialize_game(self, username: str) -> GameState:
        state = self._state_manager.create(username)

        workspace_manager = self._workspace_factory(state.workspace)
        workspace_manager.init()

        first_level = self._level_registry.get(state.current_level)
        if first_level:
            self._prepare_level(state.current_level, state)

        self._state_manager.save(state)
        self._display.show_init_success(username, str(state.workspace))

        start_dir = self._navigation_manager.get_level_start_directory(state.current_level, state.workspace)
        if start_dir:
            self._navigation_manager.maybe_teleport(start_dir)
            self._display.show_current_directory(start_dir)

        return state

    def _auto_init_if_needed(self) -> GameState:
        state = self._state_manager.load()
        if state:
            return state

        username = os.environ.get("USER", "player")
        return self._initialize_game(username)

    def _restore_workspace_if_missing(self, state: GameState) -> None:
        if state.workspace.exists():
            return

        workspace_manager = self._workspace_factory(state.workspace)
        workspace_manager.init()

        current_level = self._level_registry.get(state.current_level)
        if current_level:
            self._prepare_level(state.current_level, state)

        self._display.show_workspace_restored(state.workspace)

    def _resync_stale_level(self, state: GameState) -> _LevelSync:
        """Rescue a save whose `current_level` no longer exists.

        Levels get renumbered between versions. Without this, every gameplay
        command would report "level not found" and the only way out would be
        `shellgame remove`, which destroys all progress.
        """
        if self._level_registry.get(state.current_level) is not None:
            return _LevelSync.VALID

        stale_id = state.current_level
        resolved_id = self._level_registry.resolve_or_nearest(stale_id)
        if resolved_id is None:
            self._display.show_no_levels_available()
            return _LevelSync.UNAVAILABLE

        self._display.show_level_missing(stale_id, resolved_id)
        state.current_level = resolved_id
        self._progress_tracker.ensure_level_started(state, level_id=resolved_id)
        self._state_manager.save(state)
        self._prepare_level(resolved_id, state)
        return _LevelSync.RESYNCED

    def _active_state(self) -> GameState | None:
        """Shared gameplay preamble: load state and rebuild a missing workspace.

        Every gameplay command needs this. Without it, a player whose `/tmp` was
        cleared gets confusing validation failures instead of a rebuild.
        """
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return None

        sync = self._resync_stale_level(state)
        if sync is _LevelSync.UNAVAILABLE:
            return None
        if sync is _LevelSync.RESYNCED:
            # The in-flight command was aimed at a level that no longer exists.
            # Applying it to the replacement level would be wrong, so show the
            # new assignment instead.
            self.show_current_level()
            return None

        self._restore_workspace_if_missing(state)
        return state

    def show_current_level(self) -> None:
        state = self._auto_init_if_needed()

        if self._handle_finished(state):
            return

        if self._resync_stale_level(state) is _LevelSync.UNAVAILABLE:
            return

        self._restore_workspace_if_missing(state)

        level = self._get_level(state.current_level)
        if level is None:
            return

        self._display.show_instructions(level)

        if self._progress_tracker.ensure_level_started(state, level_id=level.id, now=datetime.now()):
            self._state_manager.save(state)

        self._export_shell_context(workspace=state.workspace, level_id=level.id)

        self._navigation_manager.ensure_user_in_reasonable_place(state)

        if level.is_intro:
            self._display.wait_for_continue()
            self.submit(answer=None)

    def init(self) -> None:
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
        state = self._active_state()
        if not state:
            return

        if self._handle_finished(state):
            return

        level = self._get_level(state.current_level)
        if level is None:
            return

        hints = list(level.hints)
        total_hints = len(hints)

        if total_hints == 0:
            self._display.show_no_hints_available()
            return

        if repeat:
            revealed = self._progress_tracker.get_hint_status(state, level.id, total_hints)
            self._display.show_repeated_hints(hints, revealed)
        else:
            revealed_idx = self._progress_tracker.reveal_next_hint(state, level.id, total_hints)
            if revealed_idx == -1:
                self._display.show_no_more_hints()
            else:
                self._state_manager.save(state)
                self._display.show_hint(hints[revealed_idx], revealed_idx, total_hints)

    def reset(self) -> None:
        state = self._active_state()
        if not state:
            return

        if self._handle_finished(state):
            return

        level = self._get_level(state.current_level)
        if level is None:
            return

        try:
            level.reset(state.workspace)
        except ContainedPathError as exc:
            self._console.print(f"[red]Chyba při obnově levelu: {exc}[/red]\n")
            return

        self._display.show_reset(state.current_level)
        self._display.show_instructions(level)
        self._export_shell_context(workspace=state.workspace, level_id=level.id)
        self._navigation_manager.ensure_user_in_reasonable_place(state)

    def levels(self) -> None:
        """List every level a player can resume into."""
        registered = self._level_registry.list_levels()
        if not registered:
            self._display.show_no_levels_available()
            return

        state = self._state_manager.load()
        self._display.show_levels(
            registered,
            current_level=state.current_level if state else None,
            completed=set(state.levels_complete) if state else set(),
        )

    def resume(self, level_id: str | None) -> bool:
        """Move the player to `level_id` and make it their saved position.

        Unlike `repeat`, which only reprints an assignment, this rewrites
        `current_level`, so quitting and launching ShellGame again starts here.

        Returns whether the player was actually moved, so the CLI can exit
        non-zero when they were not.
        """
        # Missing ID is a usage error, but the IDs are the one thing a player
        # cannot guess, so the listing comes with it rather than just a refusal.
        if level_id is None:
            self._display.show_missing_level_id()
            self.levels()
            return False

        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return False

        # Resolved against the request, never against the saved level: an
        # explicit target must win even when the save points at a level that
        # this version renumbered away.
        if parse_level_id(level_id) is None:
            self._display.show_invalid_level_id(level_id)
            return False

        if self._level_registry.get(level_id) is None:
            self._display.show_unknown_level(level_id)
            return False

        self._restore_workspace_if_missing(state)

        if not self._prepare_level(level_id, state):
            return False

        state.current_level = level_id
        # A finished run makes every gameplay command print the completion
        # screen instead. Resuming into a level reopens the run, otherwise the
        # player would land on the level but be unable to play it.
        state.completed_at = None
        self._progress_tracker.ensure_level_started(state, level_id=level_id)
        self._state_manager.save(state)

        self._display.show_resumed(level_id)
        self.show_current_level()
        return True

    def repeat(self, *, section_num: int | None, level_id: str | None) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

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

    def show(self, *, section: bool, level: bool) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        if self._handle_finished(state):
            return

        if section == level:
            self._console.print("[yellow]Použití: shellgame show --level  nebo  shellgame show --section[/yellow]\n")
            return

        target_id = state.current_level
        if section:
            section_prefix, separator, _ = state.current_level.partition(".")
            if not separator or not section_prefix.isdigit():
                self._console.print(f"[red]Chyba: Neplatný formát aktuálního levelu: {state.current_level}[/red]\n")
                return
            target_id = f"{section_prefix}.0"

        target_level = self._get_level(target_id)
        if target_level is not None:
            self._display.show_instructions(target_level)

    def remove(self) -> None:
        try:
            state = self._state_manager.load()
        except StateLoadError:
            # The state file is unreadable, so the workspace path cannot be read
            # from it. Fall back to the default location for this user.
            username = os.environ.get("USER", "player")
            self._workspace_factory(StateManager.default_workspace(username)).remove()
            self._state_manager.remove()
            self._display.show_removed()
            return

        if not state:
            self._display.show_not_initialized()
            return

        self._workspace_factory(state.workspace).remove()
        self._state_manager.remove()

        self._display.show_removed()

    def submit(self, answer: str | None) -> None:
        state = self._active_state()
        if not state:
            return

        if self._handle_finished(state):
            return

        level = self._get_level(state.current_level)
        if level is None:
            return

        ok, msg = level.validate(answer, state)

        if not ok:
            self._progress_tracker.record_attempt(state, level_id=level.id)
            self._state_manager.save(state)
            self._display.show_failure(msg)
            return

        completion = self._progress_tracker.record_completion(state, level_id=level.id, completed_at=datetime.now())
        self._advance(state, level=level, success_message=msg, completion=completion)

    def skip(self) -> None:
        state = self._active_state()
        if not state:
            return

        if self._handle_finished(state):
            return

        level = self._get_level(state.current_level)
        if level is None:
            return

        if not level.is_bonus:
            self._display.show_not_skippable(level.id)
            return

        self._display.show_skipped(level.id)
        self._advance(state, level=level, success_message=None, completion=None)

    def _advance(
        self,
        state: GameState,
        *,
        level: Level,
        success_message: str | None,
        completion: LevelCompletion | None,
    ) -> None:
        """Move to the next level with exactly one atomic state save."""
        try:
            next_level_id = self._level_registry.next_level(level.id)
        except UnknownLevelError:
            # Never mistake an unregistered ID for "the player finished".
            if self._resync_stale_level(state) is _LevelSync.RESYNCED:
                self.show_current_level()
            return

        if next_level_id is None:
            state.completed_at = datetime.now()
            self._state_manager.save(state)
            if success_message is not None:
                self._show_success(success_message, completion)
            self._display.show_game_complete(state)
            return

        state.current_level = next_level_id
        self._progress_tracker.ensure_level_started(state, level_id=next_level_id)
        self._state_manager.save(state)

        prepared = self._prepare_level(next_level_id, state)

        if success_message is not None and not level.is_intro:
            self._show_success(success_message, completion)
            self._display.wait_for_continue()

        if not prepared:
            return

        self.show_current_level()

    def _show_success(self, message: str, completion: LevelCompletion | None) -> None:
        if completion is None:
            self._display.show_success(message)
            return
        self._display.show_success(
            message,
            time_sec=completion.time_sec,
            hints_used=completion.hints,
            attempts=completion.attempts,
        )

    def _prepare_level(self, level_id: str, state: GameState) -> bool:
        level = self._level_registry.get(level_id)
        if level is None:
            return True

        try:
            level.prepare(state.workspace)
        except (OSError, ValueError) as error:
            self._display.show_level_setup_error(level_id, str(error))
            return False
        return True

    def _handle_finished(self, state: GameState) -> bool:
        if getattr(state, "completed_at", None) is None:
            return False
        self._display.show_game_complete(state)
        return True

    def dev_jump_to(self, level_id: str) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        level = self._level_registry.get(level_id)
        if not level:
            self._console.print(f"[red]Neznámý level: {level_id}[/red]")
            return

        if not self._prepare_level(level_id, state):
            return
        state.current_level = level_id
        self._state_manager.save(state)
        self.show_current_level()

    def dev_next(self) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        try:
            next_id = self._level_registry.next_level(state.current_level)
        except UnknownLevelError:
            if self._resync_stale_level(state) is _LevelSync.RESYNCED:
                self.show_current_level()
            return
        if next_id is None:
            self._console.print("[yellow]Žádný další level.[/yellow]")
            return
        self.dev_jump_to(next_id)

    def dev_previous(self) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        previous_id: str | None = None
        for registered_level in self._level_registry.list_levels():
            if registered_level.id == state.current_level:
                break
            previous_id = registered_level.id

        if previous_id is None:
            self._console.print("[yellow]Žádný předchozí level.[/yellow]")
            return
        self.dev_jump_to(previous_id)

    def dev_reload(self) -> None:
        state = self._state_manager.load()
        if not state:
            self._display.show_not_initialized()
            return

        level = self._get_level(state.current_level)
        if level is None:
            return

        level.reset(state.workspace)
        self._display.wait_for_continue()
        self._display.show_instructions(level)
        self._console.print(f"[green]✓ Level {state.current_level} znovu načten.[/green]")

    def dev_start(self) -> None:
        levels = self._level_registry.list_levels()
        if not levels:
            self._console.print("[red]Nejsou registrovány žádné levely.[/red]")
            return
        self.dev_jump_to(str(levels[0].id))

    def exit(self) -> None:
        self._console.print("[yellow]Ukončuji ShellGame...[/yellow]")
        self._shell_client.exit()

    def handle_cd_hook(self, *, target: str | None, pwd: str | None, post_move: bool) -> None:
        state = self._state_manager.load()
        if not state:
            return

        level = self._get_level(state.current_level)
        if not level:
            return

        hook = level.hooks.get("cd")
        if hook:
            hook(target=target, pwd=pwd, post_move=post_move, state=state)

    def handle_fd_hook(self) -> None:
        state = self._state_manager.load()
        if not state:
            return

        level = self._get_level(state.current_level)
        if level is None:
            return

        try:
            stdout_target = os.readlink("/proc/self/fd/1")
            stderr_target = os.readlink("/proc/self/fd/2")
        except OSError:
            return

        level.record_fd_evidence(
            stdout_target=stdout_target,
            stderr_target=stderr_target,
            state=state,
        )
