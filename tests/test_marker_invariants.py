"""Invariants that keep evidence markers from silently breaking a level.

Markers are flat dotfiles in the workspace root. Two authoring mistakes are
invisible at runtime and impossible to spot in review:

* a marker that is *required* but never *written* makes its level permanently
  unsatisfiable;
* a marker that is written but never cleared by its owning level's
  ``prepare()`` leaks across a ``reset`` and pre-satisfies the level.

Both are structural properties of the source, so they are checked statically.
"""

from __future__ import annotations

import ast
from pathlib import Path

import pytest

from shellgame.levels.base import Level
from shellgame.levels.cdpolicy import CD_EVIDENCE, cd_marker
from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import LevelRegistry, get_registry
from shellgame.markers import MarkerManager

SECTIONS_DIR = Path(__file__).resolve().parent.parent / "src" / "shellgame" / "levels" / "sections"
TEMPLATES_DIR = Path(__file__).resolve().parent.parent / "src" / "shellgame" / "cli" / "templates"

MARKER_CONSTANTS = {
    name: value for name, value in vars(MarkerManager).items() if name.isupper() and isinstance(value, str)
}


def _marker_value(node: ast.expr) -> str | None:
    """Resolve ``MarkerManager.SOME_CONST`` (or a literal) to its marker name."""
    if isinstance(node, ast.Attribute) and node.attr in MARKER_CONSTANTS:
        return MARKER_CONSTANTS[node.attr]
    if isinstance(node, ast.Constant) and isinstance(node.value, str):
        return node.value if node.value in MARKER_CONSTANTS.values() else None
    return None


def _class_markers(class_node: ast.ClassDef) -> tuple[set[str], set[str]]:
    """Return the markers a level class writes and the ones it merely reads."""
    written: set[str] = set()
    referenced: set[str] = set()

    for node in ast.walk(class_node):
        if isinstance(node, ast.Attribute) and node.attr in MARKER_CONSTANTS:
            referenced.add(MARKER_CONSTANTS[node.attr])
        if (
            isinstance(node, ast.Call)
            and isinstance(node.func, ast.Attribute)
            and node.func.attr == "create"
            and node.args
            and (marker := _marker_value(node.args[0])) is not None
        ):
            written.add(marker)

    return written, referenced


def _level_classes() -> dict[str, ast.ClassDef]:
    """Map every level class name to its AST node, across all section modules."""
    classes: dict[str, ast.ClassDef] = {}
    for module_path in sorted(SECTIONS_DIR.glob("section*.py")):
        tree = ast.parse(module_path.read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            if isinstance(node, ast.ClassDef):
                classes[f"{module_path.stem}.{node.name}"] = node
    return classes


def _template_markers() -> set[str]:
    """Markers written directly by the shell integration templates."""
    written: set[str] = set()
    for template in TEMPLATES_DIR.glob("*_integration.template"):
        text = template.read_text(encoding="utf-8")
        for marker in MARKER_CONSTANTS.values():
            if f".{marker}" in text:
                written.add(marker)
    return written


@pytest.fixture(scope="module")
def registry() -> LevelRegistry:
    initialize_levels()
    return get_registry()


@pytest.fixture(scope="module")
def written_by_class(registry: LevelRegistry) -> dict[str, set[str]]:
    """Markers each level writes, whether by hand or through its `cd_policy`."""
    written = {name: _class_markers(node)[0] for name, node in _level_classes().items()}
    by_class_name = {type(level).__name__: level for level in registry.list_levels()}
    for qualified_name, markers in written.items():
        level = by_class_name.get(qualified_name.split(".", 1)[1])
        if level is not None and level.cd_policy is not None:
            markers.add(level.cd_policy.marker)
    return written


def test_policy_markers_are_derived_from_the_level_id(registry: LevelRegistry) -> None:
    """Auto-derived names make collisions impossible by construction."""
    policy_levels = [level for level in registry.list_levels() if level.cd_policy is not None]

    assert policy_levels, "No level uses cd_policy; the derivation is untested"
    for level in policy_levels:
        assert level.cd_policy is not None
        assert level.cd_policy.marker == cd_marker(level.id), (
            f"Level {level.id} has a hand-written cd marker; let the ID derive it"
        )


def test_policy_evidence_placeholders_are_all_bound(registry: LevelRegistry) -> None:
    """An unbound placeholder would be checked as a literal, never-written name."""
    for level in registry.list_levels():
        markers = level.completion.evidence_markers if level.completion else ()
        assert CD_EVIDENCE not in markers, f"Level {level.id} has an unbound CdEvidence placeholder"
        if level.cd_policy is not None:
            assert level.cd_policy.marker != CD_EVIDENCE, f"Level {level.id} has an unbound cd_policy marker"


def test_marker_constants_are_unique() -> None:
    """Two constants sharing a value would let one level satisfy another."""
    values = list(MARKER_CONSTANTS.values())
    duplicates = {value for value in values if values.count(value) > 1}
    assert not duplicates, f"Marker values reused across constants: {sorted(duplicates)}"


def test_every_required_marker_is_written_somewhere(
    registry: LevelRegistry, written_by_class: dict[str, set[str]]
) -> None:
    """A required marker nobody writes makes its level impossible to finish."""
    produced = set(_template_markers())
    for markers in written_by_class.values():
        produced |= markers

    unsatisfiable: list[tuple[str, str]] = []
    for level in registry.list_levels():
        for marker in _required_markers(level):
            if marker not in produced:
                unsatisfiable.append((level.id, marker))

    assert not unsatisfiable, (
        "These levels require evidence markers that nothing ever creates, so they "
        f"can never be completed: {unsatisfiable}"
    )


def test_every_written_marker_is_cleared_by_its_level(
    registry: LevelRegistry, written_by_class: dict[str, set[str]]
) -> None:
    """A marker that survives ``prepare()`` pre-satisfies the level after a reset."""
    by_class_name = {type(level).__name__: level for level in registry.list_levels()}

    leaked: list[tuple[str, str]] = []
    for qualified_name, markers in written_by_class.items():
        class_name = qualified_name.split(".", 1)[1]
        level = by_class_name.get(class_name)
        if level is None:
            continue
        cleared = _cleared_markers(level)
        for marker in markers:
            if marker not in cleared:
                leaked.append((level.id, marker))

    assert not leaked, (
        "These levels write evidence markers that their own prepare() does not clear, "
        "so the evidence survives `shellgame reset` and pre-satisfies the level: "
        f"{leaked}. Add the marker to `reset_markers` or to an `Evidence` requirement."
    )


def test_each_marker_has_at_most_one_writing_level(written_by_class: dict[str, set[str]]) -> None:
    """Shared write sites make one level's actions satisfy a different level."""
    owners: dict[str, list[str]] = {}
    for qualified_name, markers in written_by_class.items():
        for marker in markers:
            owners.setdefault(marker, []).append(qualified_name)

    shared = {marker: names for marker, names in owners.items() if len(names) > 1}
    assert not shared, f"Markers written by more than one level class: {shared}"


def _required_markers(level: Level) -> set[str]:
    return set(level.completion.evidence_markers) if level.completion else set()


def _cleared_markers(level: Level) -> set[str]:
    completion_markers = level.completion.evidence_markers if level.completion else ()
    return {name for name in (*level.reset_markers, *completion_markers) if name}
