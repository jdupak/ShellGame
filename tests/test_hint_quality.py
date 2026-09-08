"""Pedagogical guarantees about hints and assessment copy.

A hint ladder should scaffold, not hand over the answer. Assessment levels
(section summaries and challenges) are where students demonstrate what they
learned, so a hint there must never contain the literal expected answer - that
turns the assessment into a lookup.

Levels whose *instructions* already state the answer (for example "submit the
password: builder") are exempt from the hint check: repeating it reveals
nothing. ``error_message`` and discovery ``shellgame submit …`` examples are
never exempt — those surfaces must not print the value.
"""

from __future__ import annotations

import re
from datetime import datetime
from pathlib import Path

import pytest

from shellgame.levels.base import Level
from shellgame.levels.completion import (
    AtDirectory,
    ExactAnswer,
    IntegerAnswer,
    SuffixAnswer,
    TupleAnswer,
)
from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import get_registry
from shellgame.messages import Messages
from shellgame.state.manager import GameState

#: Answers shorter than this match too many unrelated words to be meaningful.
_MIN_ANSWER_LENGTH = 4

_ASSESSMENT_MARKERS = ("souhrn", "výzva", "challenge", "finální")


@pytest.fixture(scope="module")
def levels() -> list:
    initialize_levels()
    return list(get_registry().list_levels())


def _expected_answers(rule: object) -> list[str]:
    if isinstance(rule, TupleAnswer):
        parts = [value for part in rule.parts for value in _expected_answers(part)]
        raw = [str(part.expected) for part in rule.parts if isinstance(part, (ExactAnswer, IntegerAnswer))]
        if len(raw) == len(rule.parts) and raw:
            parts.append(rule.separator.join(raw))
        return parts
    if isinstance(rule, SuffixAnswer):
        return [rule.expected]
    if isinstance(rule, (ExactAnswer, IntegerAnswer)):
        return [str(rule.expected)]
    return []


def _error_messages(rule: object) -> list[str]:
    if isinstance(rule, TupleAnswer):
        return [message for part in rule.parts for message in _error_messages(part)]
    message = getattr(rule, "error_message", None)
    messages = [message] if message else []
    if isinstance(rule, (ExactAnswer, IntegerAnswer)):
        messages.extend(rule.mistakes.values())
    return messages


def _is_assessment(level: object) -> bool:
    title = str(getattr(level, "title", "")).lower()
    return any(marker in title for marker in _ASSESSMENT_MARKERS)


def test_assessment_hints_do_not_contain_the_answer(levels: list) -> None:
    leaks: list[str] = []

    for level in levels:
        if not _is_assessment(level):
            continue

        completion = getattr(level, "completion", None)
        if completion is None or completion.answer is None:
            continue

        instructions = str(getattr(level, "instructions", "")).lower()
        answers = [
            answer
            for answer in _expected_answers(completion.answer)
            if len(answer) >= _MIN_ANSWER_LENGTH or ("," in answer and len(answer) >= 3)
        ]

        for index, hint in enumerate(getattr(level, "hints", []) or [], start=1):
            hint_text = str(hint).lower()
            for answer in answers:
                if "," not in answer and answer.lower() in instructions:
                    continue  # the task itself states this value
                if answer.lower() in hint_text:
                    leaks.append(f"{level.id} ({level.title}) hint #{index} reveals {answer!r}: {hint}")

    assert not leaks, "Assessment hints must scaffold, not reveal:\n" + "\n".join(leaks)


def test_error_message_never_contains_the_expected_answer(levels: list) -> None:
    """A wrong submit must not print the value the player was looking for."""
    leaks: list[str] = []

    for level in levels:
        completion = getattr(level, "completion", None)
        if completion is None or completion.answer is None:
            continue

        answers = [answer for answer in _expected_answers(completion.answer) if len(answer) >= _MIN_ANSWER_LENGTH]
        for message in _error_messages(completion.answer):
            lowered = message.lower()
            for answer in answers:
                if answer.lower() in lowered:
                    leaks.append(f"{level.id} ({level.title}) feedback reveals {answer!r}: {message}")

    assert not leaks, "Feedback must not contain the expected answer:\n" + "\n".join(leaks)


@pytest.mark.parametrize("level_id", ["1.1", "2.1", "2.2", "2.3", "2.4", "8.8"])
def test_discovery_final_hint_does_not_give_the_answer(levels: list[Level], level_id: str) -> None:
    level = next(level for level in levels if level.id == level_id)
    assert level.completion is not None
    assert isinstance(level.completion.answer, (ExactAnswer, IntegerAnswer))
    expected = str(level.completion.answer.expected)

    assert not re.search(rf"(?<!\w){re.escape(expected)}(?!\w)", level.hints[-1], re.IGNORECASE)


def test_navigation_feedback_does_not_reveal_target(
    levels: list[Level], tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    workspace = tmp_path / "workspace"
    workspace.mkdir()
    monkeypatch.chdir(tmp_path)
    state = GameState(username="tester", workspace=workspace, current_level="test", start_time=datetime.now())

    for level in levels:
        completion = level.completion
        if completion is None:
            continue
        requirements = [*completion.requirements, completion.allow_empty_when]
        directories = [requirement for requirement in requirements if isinstance(requirement, AtDirectory)]
        if not directories:
            continue
        level.prepare(workspace)
        root = level.section_path(workspace)
        for requirement in directories:
            expected = (root / requirement.path).name
            success, message = requirement.check(state, root)
            assert not success, level.id
            assert not re.search(rf"(?<!\w){re.escape(expected)}(?!\w)", message, re.IGNORECASE), (
                f"{level.id}: {message}"
            )


@pytest.mark.parametrize("answer", ["inside.txt", "other.csv"])
def test_extension_feedback_preserves_location_guard_and_does_not_echo_answer(
    levels: list[Level], tmp_path: Path, monkeypatch: pytest.MonkeyPatch, answer: str
) -> None:
    level = next(level for level in levels if level.id == "1.3")
    level.prepare(tmp_path)
    state = GameState(username="tester", workspace=tmp_path, current_level=level.id, start_time=datetime.now())
    root = level.section_path(tmp_path)
    monkeypatch.chdir(root)
    assert level.validate(answer, state) == (False, Messages.WRONG_DIRECTORY)

    monkeypatch.chdir(root / "alpha")
    assert level.validate(answer, state) == (False, Messages.L1_3_INCLUDED_EXTENSION)
    assert "inside" not in Messages.L1_3_INCLUDED_EXTENSION
    assert level.validate("inside", state)[0]


def test_discovery_submit_line_does_not_include_the_path(levels: list) -> None:
    """Finding a path is the task; the submit example must stay a placeholder."""
    leaks: list[str] = []

    for level in levels:
        completion = getattr(level, "completion", None)
        if completion is None or not isinstance(completion.answer, SuffixAnswer):
            continue

        needle = f"shellgame submit {completion.answer.expected}"
        if needle.lower() in str(getattr(level, "instructions", "")).lower():
            leaks.append(f"{level.id} ({level.title}) instructions contain {needle!r}")

    assert not leaks, "Discovery submit examples must use a placeholder:\n" + "\n".join(leaks)


@pytest.mark.parametrize("level_id", ["5.1", "7.1", "8.6"])
def test_discovery_submit_examples_do_not_reveal_values(levels: list[Level], level_id: str) -> None:
    level = next(level for level in levels if level.id == level_id)
    assert level.completion is not None
    examples = re.findall(r"`shellgame submit ([^`]+)`", level.instructions)
    assert examples
    for example in examples:
        assert "<" in example and ">" in example
        for expected in _expected_answers(level.completion.answer):
            assert expected.lower() not in example.lower()


def test_intro_hints_continue_with_enter(levels: list) -> None:
    """Intros auto-advance on Enter. Telling the player to submit contradicts the engine."""
    broken: list[str] = []

    for level in levels:
        if not getattr(level, "is_intro", False):
            continue
        blob = " ".join(getattr(level, "hints", []) or []).lower()
        if "enter" not in blob or "submit" in blob:
            broken.append(f"{level.id}: {blob!r}")

    assert not broken, "Intro hints must say Enter, not submit:\n" + "\n".join(broken)


def test_every_level_has_usable_hints(levels: list) -> None:
    """Empty or whitespace-only hints would burn a hint with no help."""
    broken = [
        f"{level.id} hint #{index}"
        for level in levels
        for index, hint in enumerate(getattr(level, "hints", []) or [], start=1)
        if not str(hint).strip()
    ]
    assert not broken, f"Empty hints: {broken}"


def test_hint_progression_scaffolding(levels: list[Level]) -> None:
    """Verify that multi-step hints in key operational levels introduce concept first."""
    checked_levels = {
        "2.1",
        "2.2",
        "2.3",
        "2.4",
        "2.6",
        "4.2",
        "4.7",
        "6.2",
        "6.3",
        "6.4",
        "6.5",
        "6.6",
        "7.1",
        "7.3",
        "7.4",
        "8.2",
        "8.3",
        "8.4",
        "10.1",
        "10.5",
        "11.1",
    }
    for level in levels:
        if level.id in checked_levels:
            assert len(level.hints) >= 2, f"{level.id} should have at least 2 hints"
            # Hint 1 should explain concept/syntax, not start with 'Spusťte' or 'Použijte'
            hint1 = level.hints[0]
            assert not hint1.startswith("Použijte '"), f"{level.id} hint 1 gives command immediately: {hint1}"
            assert not hint1.startswith("Spusťte '"), f"{level.id} hint 1 gives command immediately: {hint1}"
