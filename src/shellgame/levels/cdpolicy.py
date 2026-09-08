"""Declarative `cd` policies for strict navigation levels.

Several levels teach one exact movement command and must reject anything else.
Hand-written hooks all repeated the same five steps, and each one had to
remember the anti-soft-lock contract: skip post-move calls, consult
`cd_enforcement_lifted()` before rejecting anything, reject only through
`block_cd()`, and record evidence on success.

A `CdPolicy` states only what the level requires. `Level` performs the
contract, so the guarantee is structural instead of a convention every author
has to re-implement correctly.

A policy has two kinds of clause:

* **scope** - decides whether the policy has anything to say about this move.
  A move outside the scope is allowed silently.
* **rules** - decide whether an in-scope move is acceptable. The first failing
  rule rejects the move with its own message.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Protocol

from shellgame.levels.completion import Evidence
from shellgame.markers import MarkerManager
from shellgame.protocols import GameStateProtocol

#: Placeholder for "this level's own `cd` evidence marker". `Section.level()`
#: replaces it with a name derived from the level ID, so a marker cannot be
#: typo'd or accidentally shared between levels.
CD_EVIDENCE = "\x00shellgame.cd-evidence"


def CdEvidence(message: str) -> Evidence:  # noqa: N802 - reads as a requirement constructor
    """Require that this level's own `cd` drill has been performed.

    The marker name is bound from the level ID at registration time.
    """
    return Evidence(CD_EVIDENCE, message)


def cd_marker(level_id: str) -> str:
    """The evidence marker recording that a level's `cd` drill was performed."""
    return f"cd-{level_id}"


def _normalise(command: str | None) -> str | None:
    """Compare `cd` targets without tripping over spacing or a trailing slash."""
    if command is None:
        return None
    stripped = command.strip()
    if len(stripped) > 1:
        stripped = stripped.rstrip("/") or "/"
    return stripped


@dataclass(frozen=True, slots=True)
class CdRequest:
    """One attempted `cd`, with everything a clause may need to judge it."""

    target: str | None
    pwd: str | None
    state: GameStateProtocol
    #: the level's section root; every declared path resolves against it
    root: Path

    def at(self, path: str) -> bool:
        """True when the move starts in the given section-relative directory."""
        if not self.pwd:
            return False
        expected = self.root / path if path else self.root
        try:
            return Path(self.pwd).resolve() == expected.resolve()
        except (OSError, RuntimeError):
            return False


class CdScope(Protocol):
    def applies(self, request: CdRequest) -> bool: ...


class CdRule(Protocol):
    def reject(self, request: CdRequest) -> str | None: ...


@dataclass(frozen=True, slots=True)
class FromDirectory:
    """Restrict the policy to moves that start in one directory."""

    path: str

    def applies(self, request: CdRequest) -> bool:
        return request.at(self.path)


@dataclass(frozen=True, slots=True)
class WithTarget:
    """Restrict the policy to one literal `cd` argument, such as `-`."""

    value: str

    def applies(self, request: CdRequest) -> bool:
        return request.target == self.value


@dataclass(frozen=True, slots=True)
class RequireExactCommand:
    """Accept exactly one `cd` argument."""

    command: str
    message: str

    def reject(self, request: CdRequest) -> str | None:
        if _normalise(request.target) == _normalise(self.command):
            return None
        return self.message


@dataclass(frozen=True, slots=True)
class RequireAbsolutePath:
    """Accept any absolute target, and nothing relative."""

    message: str
    missing_message: str | None = None

    def reject(self, request: CdRequest) -> str | None:
        if not request.target:
            return self.missing_message or self.message
        if request.target.startswith("/"):
            return None
        return self.message


@dataclass(frozen=True, slots=True)
class RequireSourceDirectory:
    """Accept the move only when it starts in the given directory."""

    path: str
    message: str

    def reject(self, request: CdRequest) -> str | None:
        return None if request.at(self.path) else self.message


@dataclass(frozen=True, slots=True)
class RequireEvidence:
    """Accept the move only after an earlier drill has been performed."""

    marker: str
    message: str

    def reject(self, request: CdRequest) -> str | None:
        if MarkerManager.from_state(request.state).exists(self.marker):
            return None
        return self.message


@dataclass(frozen=True, slots=True)
class CdPolicy:
    rules: tuple[CdRule, ...]
    scope: tuple[CdScope, ...] = ()
    #: bound by `Section.level()` from the level ID; never set by hand
    marker: str = CD_EVIDENCE

    def applies(self, request: CdRequest) -> bool:
        return all(clause.applies(request) for clause in self.scope)

    def rejection(self, request: CdRequest) -> str | None:
        for rule in self.rules:
            if (message := rule.reject(request)) is not None:
                return message
        return None
