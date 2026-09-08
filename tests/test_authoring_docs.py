"""Keep the authoring guide honest.

A guide that drifts is worse than none: an author follows it, gets a
`TypeError`, and stops trusting the docs. Every Python example in
`docs/AUTHORING.md` is compiled and executed against the real API here, so a
renamed rule or a changed signature fails the build instead of the reader.
"""

from __future__ import annotations

import re
import textwrap
from pathlib import Path
from typing import Any

import pytest

from shellgame.cli.commands import cli
from shellgame.levels import cdpolicy, completion, solution

DOCS = Path(__file__).resolve().parents[1] / "docs"
GUIDE = DOCS / "AUTHORING.md"

_PREAMBLE = textwrap.dedent("""
    from shellgame.levels.base import Level
    from shellgame.levels.cdpolicy import (
        CdEvidence, CdPolicy, FromDirectory, RequireAbsolutePath, RequireEvidence,
        RequireExactCommand, RequireSourceDirectory, WithTarget,
    )
    from shellgame.levels.collector import Section
    from shellgame.levels.completion import (
        AtDirectory, AtHome, ChoiceAnswer, Completion, DirectoryExists, Evidence,
        ExactAnswer, FileExists, FileLineCount, IntegerAnswer, IntegerRangeAnswer,
        OrderedListAnswer, PathMoved, PathsMatch, PermissionBits, PermissionMode,
        SuffixAnswer, TextFileContent, TupleAnswer,
    )
    from shellgame.levels.fixture import FileFixture, WorkspaceFixture
    from shellgame.levels.solution import (
        AppendFile, Chdir, Chmod, CopyPath, GoHome, MakeDirectory, MovePath,
        PerformCd, RecordEvidence, RecordFdEvidence, RemovePath, RunShell,
        Solution, WalkHome, WriteFile,
    )
    from shellgame.markers import MarkerManager
    from shellgame.paths import WORKSPACE_ROOT

    N = 99
    section = Section(99, root="level-99")
""")

# The section template deliberately points at a file an author has yet to write.
_PLACEHOLDER_CONTENT = "sectionN_intro.md"


def _examples() -> list[tuple[int, str]]:
    blocks = re.findall(r"```python\n(.*?)```", GUIDE.read_text(encoding="utf-8"), re.S)
    return list(enumerate(blocks))


@pytest.mark.parametrize(("index", "example"), _examples(), ids=[f"block-{i}" for i, _ in _examples()])
def test_authoring_example_matches_the_real_api(index: int, example: str) -> None:
    """Every documented snippet must import, construct and validate for real."""
    namespace: dict[str, Any] = {}
    try:
        exec(compile(_PREAMBLE + textwrap.dedent(example), f"<AUTHORING.md block {index}>", "exec"), namespace)  # noqa: S102
    except FileNotFoundError as error:
        if _PLACEHOLDER_CONTENT in str(error):
            return
        raise


def test_guide_documents_every_rule_the_engine_offers() -> None:
    """A rule nobody documents is a rule nobody uses correctly."""
    text = GUIDE.read_text(encoding="utf-8")
    undocumented = [
        f"{module.__name__.rsplit('.', 1)[1]}.{name}"
        for module in (completion, cdpolicy, solution)
        for name in _public_rules(module)
        if name not in text
    ]
    assert not undocumented, "These are part of the authoring API but absent from AUTHORING.md:\n  " + "\n  ".join(
        undocumented
    )


def _public_rules(module: Any) -> list[str]:
    exported = getattr(module, "__all__", None)
    names = exported if exported is not None else dir(module)
    return [
        name
        for name in names
        if name[:1].isupper()
        and name not in _NOT_AUTHORING_SURFACE
        and getattr(getattr(module, name, None), "__module__", None) == module.__name__
        and not getattr(getattr(module, name), "_is_protocol", False)
    ]


# Values the engine hands *to* a rule, rather than vocabulary an author writes.
_NOT_AUTHORING_SURFACE = frozenset({"CdRequest", "SolutionContext"})


def test_spec_does_not_describe_commands_the_game_does_not_have() -> None:
    """`docs/spec.md` is read as the source of truth, so its drift is expensive."""
    text = (DOCS / "spec.md").read_text(encoding="utf-8")
    documented = set(re.findall(r"`shellgame ([a-z-]+)", text))
    documented.discard("submit")  # documented with an argument placeholder

    unknown = sorted(name for name in documented if name not in cli.commands)
    assert not unknown, f"docs/spec.md documents commands that do not exist: {unknown}"
