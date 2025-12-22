"""Progress tracker for game state business logic."""

from datetime import datetime
from typing import Optional

from shellgame.state.manager import GameState, LevelCompletion, StateManager


class ProgressTracker:
    def __init__(self, state_manager: StateManager):
        self._state_manager = state_manager

    def record_attempt(self, state: GameState, level_id: str) -> None:
        state.level_attempts[level_id] = state.level_attempts.get(level_id, 0) + 1
        self._state_manager.save(state)

    def record_hint_used(self, state: GameState, level_id: str, count: int = 1) -> None:
        if count <= 0:
            return
        state.level_hints_used[level_id] = state.level_hints_used.get(level_id, 0) + count
        self._state_manager.save(state)

    def ensure_level_started(
        self, state: GameState, level_id: str, now: Optional[datetime] = None
    ) -> None:
        if level_id in state.level_started_at:
            return
        state.level_started_at[level_id] = now or datetime.now()
        self._state_manager.save(state)

    def record_completion(
        self,
        state: GameState,
        level_id: str,
        completed_at: Optional[datetime] = None,
    ) -> LevelCompletion:
        completed_at = completed_at or datetime.now()

        started_at = state.level_started_at.get(level_id)
        time_sec = 0
        if started_at is not None:
            delta = completed_at - started_at
            time_sec = max(0, int(delta.total_seconds()))

        completion = LevelCompletion(
            time_sec=time_sec,
            hints=state.level_hints_used.get(level_id, 0),
            attempts=state.level_attempts.get(level_id, 0),
            completed_at=completed_at,
        )
        state.levels_complete[level_id] = completion
        self._state_manager.save(state)
        return completion

    def get_hint_status(self, state: GameState, level_id: str, total_hints: int) -> int:
        revealed = state.level_hints_used.get(level_id, 0)
        return max(0, min(revealed, total_hints))

    def reveal_next_hint(self, state: GameState, level_id: str, total_hints: int) -> int:
        revealed = self.get_hint_status(state, level_id, total_hints)
        if revealed >= total_hints:
            return -1

        state.level_hints_used[level_id] = revealed + 1
        self._state_manager.save(state)
        return revealed
