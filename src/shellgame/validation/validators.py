"""Validators for answer checking.

This module provides reusable validation components for checking
user answers in ShellGame levels.
"""

from typing import (
    Tuple,
    Optional,
    List,
    Dict,
    Union,
    Sequence,
    Protocol,
    runtime_checkable,
)
from pathlib import Path
import os
import stat
import subprocess

from shellgame.protocols import GameStateProtocol
from shellgame.messages import Messages
from shellgame.markers import MarkerManager


# Type alias for validation results
ValidationResult = Tuple[bool, str]

# Type alias for state parameter (can be GameStateProtocol or raw Path)
StateOrPath = Union[GameStateProtocol, Path]


@runtime_checkable
class _HasWorkspace(Protocol):
    workspace: Path


def _get_workspace_path(state: GameStateProtocol | Path) -> Path:
    """Resolve workspace path from a GameState-like object or a raw Path.

    Why: some call sites/tests pass `state.workspace`, while others pass a bare
    `Path` (e.g., `tmp_path`). This function normalizes both shapes in a way
    that is friendly to mypy.
    """
    if isinstance(state, Path):
        return state
    if isinstance(state, _HasWorkspace):
        return state.workspace

    # If we got here, we have neither a Path nor an object with `.workspace`.
    # Don't attempt to coerce arbitrary objects to Path (mypy will reject it).
    raise TypeError(f"Expected Path or object with `.workspace`, got: {type(state)!r}")


class Validator:
    """Base validator class.

    All validators should inherit from this class and implement
    the validate() method.
    """

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate an answer.

        Args:
            answer: User's answer (can be None)
            state: Game state object (has .workspace) or raw Path to workspace

        Returns:
            Tuple of (success, message)
        """
        raise NotImplementedError


class AnswerRequiredValidator(Validator):
    """Validates that an answer is provided."""

    def __init__(self, message: str = Messages.ANSWER_REQUIRED):
        self.message = message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        if answer is None or not answer.strip():
            return False, self.message
        return True, ""


class StringValidator(Validator):
    """Validates exact string match."""

    def __init__(
        self,
        expected: str,
        case_sensitive: bool = True,
        error_message: Optional[str] = None,
    ):
        """Initialize string validator.

        Args:
            expected: Expected answer
            case_sensitive: Whether comparison is case-sensitive
            error_message: Custom error message (optional)
        """
        self.expected = expected
        self.case_sensitive = case_sensitive
        self.error_message = error_message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate string match."""
        if answer is None:
            return False, Messages.ANSWER_REQUIRED

        actual = answer.strip()
        expected = self.expected

        if not self.case_sensitive:
            actual = actual.lower()
            expected = expected.lower()

        if actual == expected:
            return True, Messages.CORRECT

        if self.error_message:
            return False, self.error_message
        return False, Messages.expected_got(self.expected, answer.strip())


class IntegerValidator(Validator):
    """Validates integer answer."""

    def __init__(self, expected: int):
        """Initialize integer validator.

        Args:
            expected: Expected integer value
        """
        self.expected = expected

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate integer match."""
        if answer is None:
            return False, Messages.ANSWER_REQUIRED_NUMBER

        try:
            actual = int(answer.strip())
        except ValueError:
            return False, Messages.EXPECTED_INTEGER

        if actual == self.expected:
            return True, Messages.CORRECT
        return False, Messages.EXPECTED_GOT_INT.format(
            expected=self.expected, actual=actual
        )


class BasenameValidator(Validator):
    """Validates directory or file basename."""

    def __init__(
        self,
        expected: str,
        strip_ext: bool = False,
        error_message: Optional[str] = None,
    ):
        """Initialize basename validator.

        Args:
            expected: Expected basename
            strip_ext: Whether to strip file extension before comparing
            error_message: Custom error message (optional)
        """
        self.expected = expected
        self.strip_ext = strip_ext
        self.error_message = error_message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate basename match."""
        if answer is None:
            return False, Messages.ANSWER_REQUIRED_NAME

        actual = answer.strip()

        if self.strip_ext and "." in actual:
            actual = actual.rsplit(".", 1)[0]

        if actual == self.expected:
            return True, Messages.CORRECT

        if self.error_message:
            return False, self.error_message
        return False, Messages.expected_got(self.expected, actual)


class FileExistsValidator(Validator):
    """Validates file existence in workspace."""

    def __init__(self, relative_path: str, should_exist: bool = True):
        """Initialize file existence validator.

        Args:
            relative_path: Path relative to workspace
            should_exist: Whether file should exist (True) or not exist (False)
        """
        self.relative_path = relative_path
        self.should_exist = should_exist

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate file existence."""
        workspace = _get_workspace_path(state)
        file_path = workspace / self.relative_path
        exists = file_path.exists() and file_path.is_file()

        if exists == self.should_exist:
            if self.should_exist:
                return True, Messages.FILE_EXISTS.format(path=self.relative_path)
            return True, Messages.FILE_NOT_EXISTS.format(path=self.relative_path)

        if self.should_exist:
            return False, Messages.FILE_NOT_EXISTS.format(path=self.relative_path)
        return False, Messages.FILE_STILL_EXISTS.format(path=self.relative_path)


class DirectoryExistsValidator(Validator):
    """Validates directory existence in workspace."""

    def __init__(self, relative_path: str, should_exist: bool = True):
        """Initialize directory existence validator.

        Args:
            relative_path: Path relative to workspace
            should_exist: Whether directory should exist (True) or not (False)
        """
        self.relative_path = relative_path
        self.should_exist = should_exist

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate directory existence."""
        workspace = _get_workspace_path(state)
        dir_path = workspace / self.relative_path
        exists = dir_path.exists() and dir_path.is_dir()

        if exists == self.should_exist:
            if self.should_exist:
                return True, Messages.DIR_EXISTS.format(path=self.relative_path)
            return True, Messages.DIR_NOT_EXISTS.format(path=self.relative_path)

        if self.should_exist:
            return False, Messages.DIR_NOT_EXISTS.format(path=self.relative_path)
        return False, Messages.DIR_STILL_EXISTS.format(path=self.relative_path)


class FileContentValidator(Validator):
    """Validates file content matches expected value."""

    def __init__(self, file_path: str, expected_content: str):
        """Initialize file content validator.

        Args:
            file_path: Path to file relative to workspace
            expected_content: Expected file content
        """
        self.file_path = file_path
        self.expected_content = expected_content

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate file content."""
        workspace = _get_workspace_path(state)
        full_path = workspace / self.file_path

        if not full_path.exists():
            return False, Messages.FILE_NOT_EXISTS.format(path=self.file_path)

        try:
            actual_content = full_path.read_text().strip()
        except Exception as e:
            return False, Messages.FILE_READ_ERROR.format(error=str(e))

        if actual_content == self.expected_content.strip():
            return True, Messages.FILE_CONTENT_CORRECT
        return False, Messages.FILE_CONTENT_MISMATCH


class MultiValidator(Validator):
    """Combines multiple validators with AND logic."""

    def __init__(self, validators: Sequence[Validator]):
        """Initialize multi-validator.

        Args:
            validators: Sequence of Validator instances
        """
        self.validators = validators

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate with all validators."""
        for validator in self.validators:
            success, message = validator.validate(answer, state)
            if not success:
                return False, message
        return True, Messages.CORRECT
        return True, Messages.ALL_CHECKS_PASSED


class OrderedListValidator(Validator):
    """Validates an ordered list of strings."""

    def __init__(self, expected: List[str], case_sensitive: bool = True):
        """Initialize ordered list validator.

        Args:
            expected: List of expected strings
            case_sensitive: Whether comparison is case-sensitive
        """
        self.expected = expected
        self.case_sensitive = case_sensitive

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate ordered list match."""
        if answer is None:
            return False, Messages.ANSWER_REQUIRED_LIST

        # Split by comma and strip whitespace from each item
        items = [item.strip() for item in answer.split(",")]
        # Remove empty strings if any (e.g. trailing comma)
        items = [item for item in items if item]

        expected = self.expected

        if not self.case_sensitive:
            items = [item.lower() for item in items]
            expected = [item.lower() for item in expected]

        if items == expected:
            return True, Messages.CORRECT
        return False, Messages.expected_got(",".join(self.expected), ",".join(items))


class FileTypeValidator(Validator):
    """Validates file type using 'file' command output."""

    def __init__(self, file_path: str, expected_type: str):
        """Initialize file type validator.

        Args:
            file_path: Path to file relative to workspace
            expected_type: Expected string in 'file' command output (e.g. "ASCII text")
        """
        self.file_path = file_path
        self.expected_type = expected_type

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate file type."""
        if answer is None:
            return False, Messages.ANSWER_REQUIRED_FILE

        workspace = _get_workspace_path(state)
        target_file = workspace / answer.strip()

        if not target_file.exists():
            return False, Messages.FILE_NOT_EXISTS.format(path=answer)

        try:
            result = subprocess.run(
                ["file", "-b", str(target_file)],
                capture_output=True,
                text=True,
                check=True,
            )
            output = result.stdout.strip()

            if self.expected_type.lower() in output.lower():
                return True, Messages.CORRECT
            return False, Messages.FILE_TYPE_MISMATCH.format(
                expected=self.expected_type, actual=output
            )
        except subprocess.CalledProcessError:
            return False, "Nepodařilo se určit typ souboru."
        except FileNotFoundError:
            return False, Messages.FILE_TYPE_ERROR


class CopyValidator(Validator):
    """Validates file copy operation."""

    def __init__(self, source_path: str, dest_path: str):
        """Initialize copy validator.

        Args:
            source_path: Path to source file (relative to workspace)
            dest_path: Path to destination file (relative to workspace)
        """
        self.source_path = source_path
        self.dest_path = dest_path

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate copy."""
        workspace = _get_workspace_path(state)
        source = workspace / self.source_path
        dest = workspace / self.dest_path

        if not source.exists():
            return False, Messages.COPY_SOURCE_MISSING.format(path=self.source_path)

        if not dest.exists():
            return False, Messages.COPY_DEST_MISSING.format(path=self.dest_path)

        # Check content matches
        try:
            if not source.is_dir():
                if source.read_bytes() != dest.read_bytes():
                    return False, Messages.COPY_CONTENT_MISMATCH
        except Exception as e:
            return False, f"Chyba při kontrole souborů: {e}"

        return True, Messages.COPY_SUCCESS


class MoveValidator(Validator):
    """Validates move/rename operation."""

    def __init__(self, source_path: str, dest_path: str):
        """Initialize move validator.

        Args:
            source_path: Original path (should not exist anymore)
            dest_path: New path (should exist)
        """
        self.source_path = source_path
        self.dest_path = dest_path

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate move."""
        workspace = _get_workspace_path(state)
        source = workspace / self.source_path
        dest = workspace / self.dest_path

        if source.exists():
            return False, Messages.MOVE_SOURCE_EXISTS.format(path=self.source_path)

        if not dest.exists():
            return False, Messages.MOVE_DEST_MISSING.format(path=self.dest_path)

        return True, Messages.MOVE_SUCCESS


class PermissionValidator(Validator):
    """Validates file permissions."""

    def __init__(self, file_path: str, expected_mode: str):
        """Initialize permission validator.

        Args:
            file_path: Path to file relative to workspace
            expected_mode: Expected mode string (e.g. "rwxr-xr-x") OR octal (e.g. "755")
        """
        self.file_path = file_path
        self.expected_mode = expected_mode

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate permissions."""
        workspace = _get_workspace_path(state)
        target = workspace / self.file_path

        if not target.exists():
            return False, Messages.FILE_NOT_EXISTS.format(path=self.file_path)

        mode = target.stat().st_mode
        mode_str = stat.filemode(mode)
        mode_octal = oct(mode)[-3:]

        # If expected is octal
        if self.expected_mode.isdigit():
            if mode_octal == self.expected_mode:
                return True, Messages.PERMISSION_CORRECT
            return False, Messages.PERMISSION_MISMATCH.format(
                expected=self.expected_mode, actual=mode_octal
            )

        # If expected is string (e.g. rwxr-xr-x)
        current_perms = mode_str[1:]  # Skip type char

        if self.expected_mode == current_perms:
            return True, Messages.PERMISSION_CORRECT
        return False, Messages.PERMISSION_MISMATCH.format(
            expected=self.expected_mode, actual=current_perms
        )


class ExecutableValidator(Validator):
    """Validates if file is executable by user."""

    def __init__(self, file_path: str):
        self.file_path = file_path

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        workspace = _get_workspace_path(state)
        target = workspace / self.file_path

        if not target.exists():
            return False, Messages.FILE_NOT_EXISTS.format(path=self.file_path)

        if os.access(target, os.X_OK):
            return True, Messages.EXECUTABLE_SUCCESS
        return False, Messages.EXECUTABLE_FAIL


class CurrentDirectoryValidator(Validator):
    """Validates that the user is in a specific directory."""

    def __init__(
        self,
        expected_name: str,
        success_message: Optional[str] = None,
        error_message: Optional[str] = None,
    ):
        """Initialize current directory validator.

        Args:
            expected_name: Expected directory basename
            success_message: Custom success message (optional)
            error_message: Custom error message (optional)
        """
        self.expected_name = expected_name
        self.success_message = success_message
        self.error_message = error_message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        cwd_name = Path.cwd().name
        if cwd_name == self.expected_name:
            msg = (
                self.success_message
                or f"Správně! Jste v adresáři '{self.expected_name}'."
            )
            return True, msg

        if self.error_message:
            return False, self.error_message
        return False, Messages.wrong_directory(cwd_name, self.expected_name)


class MarkerValidator(Validator):
    """Validates that a marker file exists (created by shell wrapper)."""

    def __init__(self, marker_name: str, error_message: str):
        """Initialize marker validator.

        Args:
            marker_name: Name of the marker file (without leading dot)
            error_message: Error message if marker doesn't exist
        """
        self.marker_name = marker_name
        self.error_message = error_message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        username = getattr(state, "username", None)
        if not username:
            return False, Messages.UNKNOWN_USER

        markers = MarkerManager(username)
        if markers.exists(self.marker_name):
            return True, ""
        return False, self.error_message


class CommonMistakeValidator(Validator):
    """Checks for common mistakes and provides specific feedback."""

    def __init__(self, mistakes: Dict[Union[str, Tuple[str, ...]], str]):
        """Initialize common mistake validator.

        Args:
            mistakes: Dictionary mapping wrong answer (or tuple of wrong answers)
                     to feedback message.
        """
        self.mistakes = {}
        for key, msg in mistakes.items():
            if isinstance(key, tuple):
                for k in key:
                    self.mistakes[k] = msg
            else:
                self.mistakes[key] = msg

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Check if answer matches a known mistake."""
        if answer and answer.strip() in self.mistakes:
            return False, self.mistakes[answer.strip()]
        return True, ""


class HomeDirectoryValidator(Validator):
    """Validates that user is in home directory and optionally checks basename."""

    def __init__(self, check_basename: bool = True):
        """Initialize home directory validator.

        Args:
            check_basename: If True, also validates that answer matches home dir name.
        """
        self.check_basename = check_basename

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        """Validate home directory location."""
        home = Path.home()
        current = Path.cwd()

        if current != home:
            return False, f"Nejste doma. Jste v: {current}"

        if self.check_basename and answer:
            if answer.strip() == home.name:
                return True, "Správně! Jste doma."
            return False, Messages.expected_got(home.name, answer.strip())

        return True, "Správně! Jste doma."


# === Composite Validators for Common Patterns ===


class DirectoryAndAnswerValidator(Validator):
    """Validates user is in correct directory OR submits correct answer."""

    def __init__(self, expected_dirname: str, success_message: str = Messages.CORRECT):
        """Initialize validator.

        Args:
            expected_dirname: Expected directory basename
            success_message: Message on success
        """
        self.expected_dirname = expected_dirname
        self.success_message = success_message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        # If no answer provided, check current directory
        if answer is None:
            if Path.cwd().name == self.expected_dirname:
                return True, self.success_message
            return False, Messages.wrong_directory(
                Path.cwd().name, self.expected_dirname
            )

        # Check answer
        if answer.strip() == self.expected_dirname:
            return True, self.success_message
        return False, Messages.expected_got(self.expected_dirname, answer.strip())
