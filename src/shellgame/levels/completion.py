"""Declarative level completion rules."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field
from pathlib import Path
from stat import S_IMODE
from typing import Protocol

from shellgame.markers import MarkerManager
from shellgame.messages import Messages
from shellgame.paths import ContainedPathError, current_directory, relative_path, resolve_within
from shellgame.protocols import GameStateProtocol, ValidationResult

_KIND = "Completion"


def _relative_path(value: str) -> Path:
    return relative_path(value, kind=_KIND)


def _resolve(root: Path, value: str) -> Path:
    return resolve_within(root, value, kind=_KIND)


def _entry_exists(
    root: Path,
    path: str,
    *,
    directory: bool,
    should_exist: bool,
    error_message: str | None,
) -> ValidationResult:
    target = _resolve(root, path)
    present = target.exists() and (target.is_dir() if directory else target.is_file())
    if should_exist:
        if present:
            message = Messages.DIR_EXISTS if directory else Messages.FILE_EXISTS
            return True, message.format(path=path)
        fallback = Messages.DIR_NOT_EXISTS if directory else Messages.FILE_NOT_EXISTS
        return False, error_message or fallback.format(path=path)

    if not (target.exists() or target.is_symlink()):
        message = Messages.DIR_NOT_EXISTS if directory else Messages.FILE_NOT_EXISTS
        return True, message.format(path=path)
    fallback = Messages.DIR_STILL_EXISTS if directory else Messages.FILE_STILL_EXISTS
    return False, error_message or fallback.format(path=path)


class AnswerRule(Protocol):
    @property
    def required_message(self) -> str: ...

    def validate(self, answer: str) -> ValidationResult: ...


class Requirement(Protocol):
    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult: ...


@dataclass(frozen=True, slots=True)
class ExactAnswer:
    expected: str
    case_sensitive: bool = True
    mistakes: Mapping[str, str] = field(default_factory=dict)
    error_message: str | None = None
    required_message: str = Messages.ANSWER_REQUIRED

    def validate(self, answer: str) -> ValidationResult:
        actual = answer.strip()
        compared_actual = actual if self.case_sensitive else actual.lower()
        for mistake, message in self.mistakes.items():
            compared_mistake = mistake if self.case_sensitive else mistake.lower()
            if compared_actual == compared_mistake:
                return False, message

        compared_expected = self.expected if self.case_sensitive else self.expected.lower()
        if compared_actual == compared_expected:
            return True, Messages.CORRECT
        return False, self.error_message or Messages.INCORRECT


@dataclass(frozen=True, slots=True)
class IntegerAnswer:
    expected: int
    mistakes: Mapping[int, str] = field(default_factory=dict)
    error_message: str | None = None
    invalid_message: str = Messages.EXPECTED_INTEGER
    required_message: str = Messages.ANSWER_REQUIRED_NUMBER

    def validate(self, answer: str) -> ValidationResult:
        try:
            actual = int(answer.strip())
        except ValueError:
            return False, self.invalid_message
        if actual in self.mistakes:
            return False, self.mistakes[actual]
        if actual == self.expected:
            return True, Messages.CORRECT
        return False, self.error_message or Messages.INCORRECT


@dataclass(frozen=True, slots=True)
class IntegerRangeAnswer:
    minimum: int
    maximum: int
    error_message: str | None = None
    invalid_message: str = Messages.EXPECTED_INTEGER
    required_message: str = Messages.ANSWER_REQUIRED_NUMBER

    def __post_init__(self) -> None:
        if self.minimum > self.maximum:
            raise ValueError("Integer answer minimum cannot exceed maximum")

    def validate(self, answer: str) -> ValidationResult:
        try:
            actual = int(answer.strip())
        except ValueError:
            return False, self.invalid_message
        if self.minimum <= actual <= self.maximum:
            return True, Messages.CORRECT
        return False, self.error_message or Messages.INCORRECT


@dataclass(frozen=True, slots=True)
class ChoiceAnswer:
    accepted: tuple[str, ...]
    case_sensitive: bool = True
    error_message: str | None = None
    required_message: str = Messages.ANSWER_REQUIRED

    def __post_init__(self) -> None:
        if not self.accepted:
            raise ValueError("Choice answer requires at least one accepted value")

    def validate(self, answer: str) -> ValidationResult:
        actual = answer.strip()
        if not self.case_sensitive:
            actual = actual.lower()
            accepted = {value.lower() for value in self.accepted}
        else:
            accepted = set(self.accepted)
        if actual in accepted:
            return True, Messages.CORRECT
        return False, self.error_message or Messages.INCORRECT


@dataclass(frozen=True, slots=True)
class SuffixAnswer:
    expected: str
    case_sensitive: bool = True
    error_message: str | None = None
    required_message: str = Messages.ANSWER_REQUIRED

    def validate(self, answer: str) -> ValidationResult:
        actual = answer.strip()
        if not self.case_sensitive:
            actual = actual.lower()
            expected = self.expected.lower()
        else:
            expected = self.expected
        if actual == expected or actual.endswith(f"/{expected}"):
            return True, Messages.CORRECT
        return False, self.error_message or Messages.INCORRECT


@dataclass(frozen=True, slots=True)
class TupleAnswer:
    parts: tuple[AnswerRule, ...]
    separator: str = ","
    format_message: str = Messages.ANSWER_FORMAT
    required_message: str = Messages.ANSWER_REQUIRED

    def validate(self, answer: str) -> ValidationResult:
        values = [value.strip() for value in answer.split(self.separator)]
        if len(values) != len(self.parts):
            return False, self.format_message
        for value, rule in zip(values, self.parts, strict=True):
            success, message = rule.validate(value)
            if not success:
                return False, message
        return True, Messages.CORRECT


@dataclass(frozen=True, slots=True)
class OrderedListAnswer:
    expected: tuple[str, ...]
    separator: str = ","
    case_sensitive: bool = True
    error_message: str | None = None
    required_message: str = Messages.ANSWER_REQUIRED_LIST

    def validate(self, answer: str) -> ValidationResult:
        actual = tuple(item.strip() for item in answer.split(self.separator) if item.strip())
        if self.case_sensitive:
            matches = actual == self.expected
        else:
            matches = tuple(item.lower() for item in actual) == tuple(item.lower() for item in self.expected)
        if matches:
            return True, Messages.CORRECT
        return False, self.error_message or Messages.INCORRECT


@dataclass(frozen=True, slots=True)
class AtDirectory:
    path: str
    error_message: str | None = None

    def __post_init__(self) -> None:
        _relative_path(self.path)

    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        expected = _resolve(root, self.path).resolve()
        current = current_directory()
        if current is None:
            return False, Messages.CWD_MISSING
        if current == expected:
            return True, Messages.CORRECT
        message = self.error_message or Messages.WRONG_DIRECTORY
        return False, message


@dataclass(frozen=True, slots=True)
class AtHome:
    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        del root
        current = current_directory()
        if current is None:
            return False, Messages.CWD_MISSING
        expected = Path.home().resolve()
        if current == expected:
            return True, Messages.CORRECT
        return False, Messages.NOT_AT_HOME.format(actual=current)


@dataclass(frozen=True, slots=True)
class Evidence:
    marker: str
    error_message: str = Messages.MARKER_NOT_FOUND

    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        del root
        if MarkerManager.from_state(state).exists(self.marker):
            return True, Messages.CORRECT
        return False, self.error_message


@dataclass(frozen=True, slots=True)
class FileExists:
    path: str
    should_exist: bool = True
    error_message: str | None = None

    def __post_init__(self) -> None:
        _relative_path(self.path)

    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        del state
        return _entry_exists(
            root,
            self.path,
            directory=False,
            should_exist=self.should_exist,
            error_message=self.error_message,
        )


@dataclass(frozen=True, slots=True)
class DirectoryExists:
    path: str
    should_exist: bool = True
    error_message: str | None = None

    def __post_init__(self) -> None:
        _relative_path(self.path)

    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        del state
        return _entry_exists(
            root,
            self.path,
            directory=True,
            should_exist=self.should_exist,
            error_message=self.error_message,
        )


@dataclass(frozen=True, slots=True)
class TextFileContent:
    path: str
    exact: str | None = None
    contains: tuple[str, ...] = ()
    excludes: tuple[str, ...] = ()
    strip: bool = False
    error_message: str = Messages.FILE_CONTENT_MISMATCH
    missing_message: str | None = None
    unreadable_message: str | None = None

    def __post_init__(self) -> None:
        _relative_path(self.path)

    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        target = _resolve(root, self.path)
        if not target.is_file():
            message = self.missing_message or Messages.FILE_NOT_EXISTS.format(path=self.path)
            return False, message
        try:
            content = target.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as error:
            return False, self.unreadable_message or Messages.FILE_READ_ERROR.format(error=str(error))
        if self.strip:
            content = content.strip()
        if (
            (self.exact is not None and content != self.exact)
            or any(value not in content for value in self.contains)
            or any(value in content for value in self.excludes)
        ):
            return False, self.error_message
        return True, Messages.FILE_CONTENT_CORRECT


@dataclass(frozen=True, slots=True)
class FileLineCount:
    path: str
    expected: int
    error_message: str
    missing_message: str | None = None

    def __post_init__(self) -> None:
        _relative_path(self.path)
        if self.expected < 0:
            raise ValueError("File line count cannot be negative")

    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        target = _resolve(root, self.path)
        if not target.is_file():
            message = self.missing_message or Messages.FILE_NOT_EXISTS.format(path=self.path)
            return False, message
        try:
            actual = len(target.read_text(encoding="utf-8").splitlines())
        except (OSError, UnicodeError) as error:
            return False, Messages.FILE_READ_ERROR.format(error=str(error))
        if actual == self.expected:
            return True, Messages.CORRECT
        return False, self.error_message


@dataclass(frozen=True, slots=True)
class PathsMatch:
    source: str
    destination: str
    error_message: str = Messages.COPY_CONTENT_MISMATCH
    source_error: str | None = None
    destination_error: str | None = None

    def __post_init__(self) -> None:
        _relative_path(self.source)
        _relative_path(self.destination)

    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        source = _resolve(root, self.source)
        destination = _resolve(root, self.destination)
        if not source.exists():
            message = self.source_error or Messages.COPY_SOURCE_MISSING.format(path=self.source)
            return False, message
        if not destination.exists():
            message = self.destination_error or Messages.COPY_DEST_MISSING.format(path=self.destination)
            return False, message
        try:
            matches = self._paths_match(source, destination)
        except OSError as error:
            return False, Messages.COMPARE_FAILED.format(error=error)
        return (True, Messages.COPY_SUCCESS) if matches else (False, self.error_message)

    @classmethod
    def _paths_match(cls, source: Path, destination: Path) -> bool:
        if source.is_symlink() or destination.is_symlink():
            return source.is_symlink() and destination.is_symlink() and source.readlink() == destination.readlink()
        if source.is_file() or destination.is_file():
            return source.is_file() and destination.is_file() and source.read_bytes() == destination.read_bytes()
        if not source.is_dir() or not destination.is_dir():
            return False

        source_entries = {entry.relative_to(source) for entry in source.rglob("*")}
        destination_entries = {entry.relative_to(destination) for entry in destination.rglob("*")}
        if source_entries != destination_entries:
            return False
        return all(cls._paths_match(source / relative, destination / relative) for relative in source_entries)


@dataclass(frozen=True, slots=True)
class PathMoved:
    source: str
    destination: str
    source_error: str | None = None
    destination_error: str | None = None

    def __post_init__(self) -> None:
        _relative_path(self.source)
        _relative_path(self.destination)

    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        if _resolve(root, self.source).exists():
            message = self.source_error or Messages.MOVE_SOURCE_EXISTS.format(path=self.source)
            return False, message
        if not _resolve(root, self.destination).exists():
            message = self.destination_error or Messages.MOVE_DEST_MISSING.format(path=self.destination)
            return False, message
        return True, Messages.MOVE_SUCCESS


@dataclass(frozen=True, slots=True)
class PermissionBits:
    path: str
    required: int = 0
    forbidden: int = 0
    error_message: str = Messages.PERMISSION_WRONG

    def __post_init__(self) -> None:
        _relative_path(self.path)
        if self.required & self.forbidden:
            raise ValueError("Permission bits cannot be both required and forbidden")

    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        target = _resolve(root, self.path)
        if not target.is_file():
            return False, Messages.FILE_NOT_EXISTS.format(path=self.path)
        mode = S_IMODE(target.stat().st_mode)
        if mode & self.required == self.required and not mode & self.forbidden:
            return True, Messages.PERMISSION_CORRECT
        return False, self.error_message


@dataclass(frozen=True, slots=True)
class PermissionMode:
    path: str
    expected: int
    error_message: str | None = None

    def __post_init__(self) -> None:
        _relative_path(self.path)
        if not 0 <= self.expected <= 0o7777:
            raise ValueError("Permission mode must be between 0o0000 and 0o7777")

    def check(self, state: GameStateProtocol, root: Path) -> ValidationResult:
        target = _resolve(root, self.path)
        if not target.is_file():
            return False, Messages.FILE_NOT_EXISTS.format(path=self.path)
        actual = S_IMODE(target.stat().st_mode)
        if actual == self.expected:
            return True, Messages.PERMISSION_CORRECT
        return False, self.error_message or Messages.PERMISSION_WRONG


@dataclass(frozen=True, slots=True)
class Completion:
    answer: AnswerRule | None = None
    requirements: tuple[Requirement, ...] = ()
    allow_empty: bool = False
    allow_empty_when: Requirement | None = None

    @property
    def evidence_markers(self) -> tuple[str, ...]:
        markers = [requirement.marker for requirement in self.requirements if isinstance(requirement, Evidence)]
        if isinstance(self.allow_empty_when, Evidence):
            markers.append(self.allow_empty_when.marker)
        return tuple(markers)

    def _check_requirements(self, state: GameStateProtocol, root: Path) -> ValidationResult | None:
        for requirement in self.requirements:
            try:
                success, message = requirement.check(state, root)
            except ContainedPathError:
                return False, Messages.PATH_ESCAPES_WORKSPACE
            if not success:
                return False, message
        return None

    def _validate_empty(self, state: GameStateProtocol, root: Path, success_message: str) -> ValidationResult:
        if self.allow_empty:
            return True, success_message
        if self.allow_empty_when is not None:
            try:
                success, message = self.allow_empty_when.check(state, root)
            except ContainedPathError:
                return False, Messages.PATH_ESCAPES_WORKSPACE
            return (True, success_message) if success else (False, message)
        if self.answer is None:
            return True, success_message
        return False, self.answer.required_message

    def validate(
        self,
        answer: str | None,
        state: GameStateProtocol,
        *,
        root: Path,
        success_message: str,
    ) -> ValidationResult:
        if result := self._check_requirements(state, root):
            return result

        if answer is None:
            return self._validate_empty(state, root, success_message)

        if self.answer is not None:
            success, message = self.answer.validate(answer)
            if not success:
                return False, message

        return True, success_message
