"""Anti-soft-lock tests for strict `cd` hooks.

Several navigation levels only accept one exact `cd` command. Before these
guarantees existed, a player who took one wrong step could end up in a
directory where *every* permitted move led further away, with no in-game way
back - the level became unwinnable without deleting the whole workspace.

The rules asserted here are:

1. A blocked move always tells the player how to recover.
2. A move back to the level start directory is always allowed.
3. Once the required command has been demonstrated (evidence marker written),
   free movement is restored - the drill is over.

Completion is still gated by each level's declarative requirements, so lifting
the *movement* restriction never lets a player pass a level they did not solve.
"""

from __future__ import annotations

import inspect
import io
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, NamedTuple

import pytest

from shellgame.levels.cdpolicy import cd_marker
from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import get_registry
from shellgame.markers import MarkerManager
from shellgame.messages import Messages


class _Case(NamedTuple):
    level_id: str
    marker: str
    #: a `cd` target this level rejects while the drill is unfinished
    rejected_target: str
    #: markers that must already exist for the level to reach its strict branch
    prerequisites: tuple[str, ...] = ()
    #: cwd (relative to workspace) the hook must see for its strict branch
    pwd_relative: str | None = None


STRICT_CD_CASES = [
    _Case("1.4", cd_marker("1.4"), rejected_target="/etc"),
    _Case("1.6", cd_marker("1.6"), rejected_target="/etc"),
    _Case("1.8", cd_marker("1.8"), rejected_target="relative/path"),
    _Case("1.9", MarkerManager.LEVEL1_9_CD_WALK_COMPLETED, rejected_target="/etc"),
    _Case(
        "1.12",
        cd_marker("1.12"),
        rejected_target="/etc",
        prerequisites=(MarkerManager.PWD_USED,),
        pwd_relative="level-1/gamma/deep/a/b/c",
    ),
    _Case("2.3", cd_marker("2.3"), rejected_target="/etc"),
]

_IDS = [case.level_id for case in STRICT_CD_CASES]


@dataclass
class _FakeState:
    workspace: Path
    username: str = "student"
    current_level: str = "1.1"
    level_hints_used: dict[str, int] = field(default_factory=dict)


@pytest.fixture(scope="module")
def registry() -> Any:
    initialize_levels()
    return get_registry()


def _run_hook(level: Any, state: _FakeState, target: str | None, pwd: str) -> tuple[bool, str]:
    """Run a level's cd hook. Returns ``(blocked, stderr_text)``."""
    hook = level.hooks.get("cd")
    assert hook is not None, f"level {level.id} has no cd hook"

    captured = io.StringIO()
    original = sys.stderr
    sys.stderr = captured
    try:
        hook(target=target, pwd=pwd, post_move=False, state=state)
    except SystemExit:
        return True, captured.getvalue()
    finally:
        sys.stderr = original
    return False, captured.getvalue()


def _setup(registry: Any, tmp_path: Path, case: _Case) -> tuple[Any, _FakeState, Path, Path]:
    level = registry.get(case.level_id)
    assert level is not None, f"unknown level {case.level_id}"
    state = _FakeState(workspace=tmp_path, current_level=case.level_id)
    level.prepare(tmp_path)

    markers = MarkerManager.from_state(state)
    for prerequisite in case.prerequisites:
        markers.create(prerequisite)

    start = level.get_start_directory(tmp_path)
    assert start is not None
    start.mkdir(parents=True, exist_ok=True)

    strict_pwd = tmp_path / case.pwd_relative if case.pwd_relative else start
    strict_pwd.mkdir(parents=True, exist_ok=True)

    return level, state, start, strict_pwd


@pytest.mark.parametrize("case", STRICT_CD_CASES, ids=_IDS)
class TestStrictCdHooksNeverSoftLock:
    def test_blocked_move_explains_how_to_recover(self, registry: Any, tmp_path: Path, case: _Case) -> None:
        level, state, _start, strict_pwd = _setup(registry, tmp_path, case)

        blocked, err = _run_hook(level, state, target=case.rejected_target, pwd=str(strict_pwd))

        assert blocked, f"{case.level_id} unexpectedly allowed cd {case.rejected_target}"
        assert Messages.CD_RECOVERY_TIP in err
        assert "shellgame reset" in err

    def test_returning_to_the_start_directory_is_always_allowed(
        self, registry: Any, tmp_path: Path, case: _Case
    ) -> None:
        level, state, start, _strict_pwd = _setup(registry, tmp_path, case)

        stray = tmp_path / "somewhere-far-away"
        stray.mkdir(parents=True, exist_ok=True)

        blocked, _ = _run_hook(level, state, target=str(start), pwd=str(stray))

        assert not blocked, f"{case.level_id} blocked the move back to its own start directory"

    def test_movement_is_free_once_the_skill_is_demonstrated(self, registry: Any, tmp_path: Path, case: _Case) -> None:
        level, state, _start, strict_pwd = _setup(registry, tmp_path, case)

        blocked_before, _ = _run_hook(level, state, target=case.rejected_target, pwd=str(strict_pwd))
        assert blocked_before, f"{case.level_id} is not actually strict; the test case is stale"

        # The player demonstrates the required command.
        MarkerManager.from_state(state).create(case.marker)

        blocked_after, _ = _run_hook(level, state, target=case.rejected_target, pwd=str(strict_pwd))
        assert not blocked_after, f"{case.level_id} kept restricting movement after the drill was completed"


def test_every_blocking_level_is_covered() -> None:
    """Guard against a new strict hook being added without soft-lock protection."""
    covered = {case.level_id for case in STRICT_CD_CASES}
    # `2.2` only intercepts the literal `cd -`; every other move stays free,
    # so it cannot strand a player and needs no recovery rule.
    known_narrow = {"2.2"}

    initialize_levels()
    for level in get_registry().list_levels():
        if not _can_reject_a_move(level):
            continue
        assert level.id in covered or level.id in known_narrow, (
            f"Level {level.id} rejects cd but has no soft-lock test coverage"
        )


def _can_reject_a_move(level: Any) -> bool:
    """A level rejects moves via a declared `cd_policy` or a hand-written hook."""
    if getattr(level, "cd_policy", None) is not None:
        return True
    handler = getattr(level, "_handle_cd", None)
    return handler is not None and "block_cd" in inspect.getsource(handler)
