"""Validators for answer checking.

This module provides reusable validation components for checking
user answers in ShellGame levels.
"""

import os
import stat
import subprocess
from collections.abc import Sequence
from pathlib import Path
from typing import (
    Optional,
    Protocol,
    Union,
    runtime_checkable,
)

from shellgame.markers import MarkerManager
from shellgame.messages import Messages
from shellgame.protocols import GameStateProtocol

ValidationResult = tuple[bool, str]

StateOrPath = Union[GameStateProtocol, Path]


@runtime_checkable
class _HasWorkspace(Protocol):
    workspace: Path


def _get_workspace_path(state: GameStateProtocol | Path) -> Path:
    if isinstance(state, Path):
        return state
    if isinstance(state, _HasWorkspace):
        return state.workspace

    raise TypeError(f"Expected Path or object with `.workspace`, got: {type(state)!r}")


class Validator:
    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        raise NotImplementedError


class AnswerRequiredValidator(Validator):
    def __init__(self, message: str = Messages.ANSWER_REQUIRED):
        self.message = message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        if answer is None or not answer.strip():
            return False, self.message
        return True, ""


class StringValidator(Validator):
    def __init__(
        self,
        expected: str,
        case_sensitive: bool = True,
        error_message: Optional[str] = None,
    ):
        self.expected = expected
        self.case_sensitive = case_sensitive
        self.error_message = error_message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
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
    def __init__(self, expected: int):
        self.expected = expected

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        if answer is None:
            return False, Messages.ANSWER_REQUIRED_NUMBER

        try:
            actual = int(answer.strip())
        except ValueError:
            return False, Messages.EXPECTED_INTEGER

        if actual == self.expected:
            return True, Messages.CORRECT
        return False, Messages.EXPECTED_GOT_INT.format(expected=self.expected, actual=actual)


class BasenameValidator(Validator):
    def __init__(
        self,
        expected: str,
        strip_ext: bool = False,
        error_message: Optional[str] = None,
    ):
        self.expected = expected
        self.strip_ext = strip_ext
        self.error_message = error_message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
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
    def __init__(self, relative_path: str, should_exist: bool = True):
        self.relative_path = relative_path
        self.should_exist = should_exist

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
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
    def __init__(self, relative_path: str, should_exist: bool = True):
        self.relative_path = relative_path
        self.should_exist = should_exist

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
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
    def __init__(self, file_path: str, expected_content: str):
        self.file_path = file_path
        self.expected_content = expected_content

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
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
    def __init__(self, validators: Sequence[Validator]):
        self.validators = validators

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        for validator in self.validators:
            success, message = validator.validate(answer, state)
            if not success:
                return False, message
        return True, Messages.CORRECT
class OrderedListValidator(Validator):
    def __init__(self, expected: list[str], case_sensitive: bool = True):
        self.expected = expected
        self.case_sensitive = case_sensitive

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        if answer is None:
            return False, Messages.ANSWER_REQUIRED_LIST

        items = [item.strip() for item in answer.split(",")]
        items = [item for item in items if item]

        expected = self.expected

        if not self.case_sensitive:
            items = [item.lower() for item in items]
            expected = [item.lower() for item in expected]

        if items == expected:
            return True, Messages.CORRECT
        return False, Messages.expected_got(",".join(self.expected), ",".join(items))


class FileTypeValidator(Validator):
    def __init__(self, file_path: str, expected_type: str):
        self.file_path = file_path
        self.expected_type = expected_type

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
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
            return False, Messages.FILE_TYPE_MISMATCH.format(expected=self.expected_type, actual=output)
        except subprocess.CalledProcessError:
            return False, "Nepodařilo se určit typ souboru."
        except FileNotFoundError:
            return False, Messages.FILE_TYPE_ERROR


class CopyValidator(Validator):
    def __init__(self, source_path: str, dest_path: str):
        self.source_path = source_path
        self.dest_path = dest_path

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        workspace = _get_workspace_path(state)
        source = workspace / self.source_path
        dest = workspace / self.dest_path

        if not source.exists():
            return False, Messages.COPY_SOURCE_MISSING.format(path=self.source_path)

        if not dest.exists():
            return False, Messages.COPY_DEST_MISSING.format(path=self.dest_path)

        try:
            if not source.is_dir() and source.read_bytes() != dest.read_bytes():
                return False, Messages.COPY_CONTENT_MISMATCH
        except Exception as e:
            return False, f"Chyba při kontrole souborů: {e}"

        return True, Messages.COPY_SUCCESS


class MoveValidator(Validator):
    def __init__(self, source_path: str, dest_path: str):
        self.source_path = source_path
        self.dest_path = dest_path

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        workspace = _get_workspace_path(state)
        source = workspace / self.source_path
        dest = workspace / self.dest_path

        if source.exists():
            return False, Messages.MOVE_SOURCE_EXISTS.format(path=self.source_path)

        if not dest.exists():
            return False, Messages.MOVE_DEST_MISSING.format(path=self.dest_path)

        return True, Messages.MOVE_SUCCESS


class PermissionValidator(Validator):
    def __init__(self, file_path: str, expected_mode: str):
        self.file_path = file_path
        self.expected_mode = expected_mode

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        workspace = _get_workspace_path(state)
        target = workspace / self.file_path

        if not target.exists():
            return False, Messages.FILE_NOT_EXISTS.format(path=self.file_path)

        mode = target.stat().st_mode
        mode_str = stat.filemode(mode)
        mode_octal = oct(mode)[-3:]

        if self.expected_mode.isdigit():
            if mode_octal == self.expected_mode:
                return True, Messages.PERMISSION_CORRECT
            return False, Messages.PERMISSION_MISMATCH.format(expected=self.expected_mode, actual=mode_octal)

        current_perms = mode_str[1:]

        if self.expected_mode == current_perms:
            return True, Messages.PERMISSION_CORRECT
        return False, Messages.PERMISSION_MISMATCH.format(expected=self.expected_mode, actual=current_perms)


class ExecutableValidator(Validator):
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
    def __init__(
        self,
        expected_name: str,
        success_message: Optional[str] = None,
        error_message: Optional[str] = None,
    ):
        self.expected_name = expected_name
        self.success_message = success_message
        self.error_message = error_message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        cwd_name = Path.cwd().name
        if cwd_name == self.expected_name:
            msg = self.success_message or f"Správně! Jste v adresáři '{self.expected_name}'."
            return True, msg

        if self.error_message:
            return False, self.error_message
        return False, Messages.wrong_directory(cwd_name, self.expected_name)


class MarkerValidator(Validator):
    def __init__(self, marker_name: str, error_message: str):
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
    def __init__(self, mistakes: dict[Union[str, tuple[str, ...]], str]):
        self.mistakes = {}
        for key, msg in mistakes.items():
            if isinstance(key, tuple):
                for k in key:
                    self.mistakes[k] = msg
            else:
                self.mistakes[key] = msg

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        if answer and answer.strip() in self.mistakes:
            return False, self.mistakes[answer.strip()]
        return True, ""


class HomeDirectoryValidator(Validator):
    def __init__(self, check_basename: bool = True):
        self.check_basename = check_basename

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        home = Path.home()
        current = Path.cwd()

        if current != home:
            return False, f"Nejste doma. Jste v: {current}"

        if self.check_basename and answer:
            if answer.strip() == home.name:
                return True, "Správně! Jste doma."
            return False, Messages.expected_got(home.name, answer.strip())

        return True, "Správně! Jste doma."


class DirectoryAndAnswerValidator(Validator):
    def __init__(self, expected_dirname: str, success_message: str = Messages.CORRECT):
        self.expected_dirname = expected_dirname
        self.success_message = success_message

    def validate(self, answer: Optional[str], state: StateOrPath) -> ValidationResult:
        if answer is None:
            if Path.cwd().name == self.expected_dirname:
                return True, self.success_message
            return False, Messages.wrong_directory(Path.cwd().name, self.expected_dirname)

        if answer.strip() == self.expected_dirname:
            return True, self.success_message
        return False, Messages.expected_got(self.expected_dirname, answer.strip())
