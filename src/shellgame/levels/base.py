"""Base level interface and abstract class."""

import contextlib
import stat
import sys
from collections.abc import Sequence
from pathlib import Path
from textwrap import dedent
from typing import NoReturn, final

from shellgame.levels.cdpolicy import CdPolicy, CdRequest
from shellgame.levels.completion import Completion
from shellgame.levels.fixture import WorkspaceFixture
from shellgame.levels.solution import Solution
from shellgame.markers import MarkerManager
from shellgame.messages import Messages
from shellgame.paths import WorkspaceRoot, current_directory, resolve_within
from shellgame.protocols import CdHookCallback, GameStateProtocol, ValidationResult


def block_cd(level_id: str, message: str) -> NoReturn:
    """Reject a `cd` attempt and always tell the player how to recover.

    Strict navigation levels only accept one command shape. Without an explicit
    recovery path a player who moves one step too far can be left in a directory
    from which every permitted move leads further away.
    """
    sys.stderr.write(f"ShellGame ({level_id}): {message}\n")
    sys.stderr.write(f"ShellGame ({level_id}): {Messages.CD_RECOVERY_TIP}\n")
    sys.exit(1)


class Level:
    id = ""
    section: int | None = None
    title = ""
    instructions = ""
    instructions_file: str | None = None
    hints: Sequence[str] = ()
    optional = False
    extension = False
    start_directory: str | WorkspaceRoot | None = None
    reset_markers: Sequence[str] = ()
    success_message = Messages.CORRECT
    is_intro = False
    enforce_start_directory = True
    section_root = ""
    section_fixture: WorkspaceFixture | None = None
    fixture: WorkspaceFixture | None = None
    completion: Completion | None = None
    cd_policy: CdPolicy | None = None
    solution: Solution | None = None

    def __init__(self) -> None:
        if not self.title:
            raise ValueError("Level must have a title")
        if not self.is_intro and self.completion is None and type(self).validate is Level.validate:
            raise ValueError(f"{type(self).__name__}: regular levels require Completion or a custom validate()")

        self.hints = list(self.hints)

        if self.instructions_file:
            content_path = Path(__file__).parent / "content" / self.instructions_file
            self.instructions = dedent(content_path.read_text(encoding="utf-8")).strip()
        else:
            self.instructions = dedent(self.instructions).strip()

    def setup(self, workspace: Path) -> None:
        return

    @final
    def prepare(self, workspace: Path) -> None:
        markers = MarkerManager(workspace)
        completion_markers = self.completion.evidence_markers if self.completion else ()
        marker_names = [name for name in (*self.reset_markers, *completion_markers) if name]
        for marker_name in dict.fromkeys(marker_names):
            markers.remove(marker_name)
        if self.section_root:
            target = workspace / self.section_root
            if target.exists() and not target.is_dir() and not target.is_symlink():
                with contextlib.suppress(OSError):
                    target.chmod(stat.S_IRUSR | stat.S_IWUSR)
                target.unlink(missing_ok=True)
            elif target.is_dir() and not target.is_symlink():
                with contextlib.suppress(OSError):
                    target.chmod(stat.S_IRWXU)
        fixture_root = self.section_path(workspace)
        if self.section_fixture:
            self.section_fixture.apply(fixture_root)
        else:
            fixture_root.mkdir(parents=True, exist_ok=True)
        if self.fixture:
            self.fixture.apply(fixture_root)
        self.setup(workspace)

    def section_path(self, workspace: Path) -> Path:
        """The directory every path this level declares is resolved against."""
        return resolve_within(workspace, self.section_root, kind="Section root")

    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        if self.completion:
            return self.completion.validate(
                answer,
                state,
                root=self.section_path(state.workspace),
                success_message=self.success_message,
            )
        return True, self.success_message

    def reset(self, workspace: Path) -> None:
        self.prepare(workspace)

    def get_start_directory(self, workspace: Path) -> Path | None:
        if self.start_directory is None:
            return None

        if isinstance(self.start_directory, WorkspaceRoot):
            return workspace

        return self.section_path(workspace) / self.start_directory

    @property
    def is_bonus(self) -> bool:
        """Extension and optional levels must never gate advancement."""
        return bool(self.optional or self.extension)

    def returns_to_start(self, *, target: str | None, pwd: str | None, state: GameStateProtocol) -> bool:
        """True when this `cd` would bring the player back to the level start directory.

        Strict `cd` hooks use this to always permit the recovery move, so a
        player can never be locked out of the level's own workspace.
        """
        if not target:
            return False

        start = self.get_start_directory(state.workspace)
        if start is None:
            return False

        candidate = Path(target).expanduser()
        if not candidate.is_absolute():
            base = Path(pwd) if pwd else current_directory()
            if base is None:
                return False
            candidate = base / candidate

        try:
            return candidate.resolve() == start.resolve()
        except (OSError, RuntimeError):
            return False

    def cd_enforcement_lifted(
        self,
        marker: str,
        *,
        target: str | None,
        pwd: str | None,
        state: GameStateProtocol,
    ) -> bool:
        """True when a strict `cd` rule must not block this move.

        Enforcement is lifted once the player has demonstrated the required
        command (evidence marker exists) and for any move that returns them to
        the level start directory. Without this, a single extra step could leave
        a player in a directory from which every permitted move leads away.
        """
        if MarkerManager.from_state(state).exists(marker):
            return True
        return self.returns_to_start(target=target, pwd=pwd, state=state)

    @property
    def hooks(self) -> dict[str, CdHookCallback]:
        if self.cd_policy is not None:
            return {"cd": self._enforce_cd_policy}
        return {}

    @final
    def _enforce_cd_policy(
        self,
        *,
        target: str | None,
        pwd: str | None,
        post_move: bool,
        state: GameStateProtocol,
    ) -> None:
        """Apply this level's `cd_policy` under the anti-soft-lock contract.

        The order here is the contract: out-of-scope moves pass silently,
        enforcement is always consulted before any rejection, rejections always
        go through `block_cd()` so the player is told how to recover, and
        evidence is recorded only for a move that satisfied every rule.
        """
        policy = self.cd_policy
        if policy is None or post_move:
            return

        request = CdRequest(target=target, pwd=pwd, state=state, root=self.section_path(state.workspace))
        if not policy.applies(request):
            return

        if self.cd_enforcement_lifted(policy.marker, target=target, pwd=pwd, state=state):
            return

        if (message := policy.rejection(request)) is not None:
            block_cd(self.id, message)

        MarkerManager.from_state(state).create(policy.marker)

    def record_fd_evidence(
        self,
        *,
        stdout_target: str,
        stderr_target: str,
        state: GameStateProtocol,
    ) -> None:
        return
