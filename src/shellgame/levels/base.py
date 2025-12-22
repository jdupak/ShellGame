"""Base level interface and abstract class."""

from abc import ABC, abstractmethod
from collections.abc import Callable
from pathlib import Path
from textwrap import dedent
from typing import Optional

from shellgame.markers import MarkerManager
from shellgame.messages import Messages
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    ValidationResult,
    Validator,
)


class Level(ABC):
    def __init__(  # noqa: PLR0913
        self,
        id: Optional[str] = None,
        section: Optional[int] = None,
        title: Optional[str] = None,
        instructions: Optional[str] = None,
        hints: Optional[list[str]] = None,
        optional: Optional[bool] = None,
        extension: Optional[bool] = None,
        instructions_file: Optional[str] = None,
        validators: Optional[list[Validator]] = None,
        start_directory: Optional[str] = None,
        required_cwd: Optional[str] = None,
        require_answer: Optional[bool] = None,
        marker_name: Optional[str] = None,
        marker_error: Optional[str] = None,
        expected_answer: Optional[str] = None,
        success_message: Optional[str] = None,
        allow_cwd_as_answer: Optional[bool] = None,
    ):
        self.id = id if id is not None else getattr(self, "id", None)
        
        self.section = section if section is not None else getattr(self, "section", None)

        self.title = title if title is not None else getattr(self, "title", None)
        if self.title is None:
            raise ValueError("Level must have a title")

        _hints = hints if hints is not None else getattr(self, "hints", [])
        self.hints = list(_hints)

        self.optional = optional if optional is not None else getattr(self, "optional", False)
        self.extension = extension if extension is not None else getattr(self, "extension", False)
        
        _validators = validators if validators is not None else getattr(self, "validators", [])
        self.validators = list(_validators)

        self.start_directory = start_directory if start_directory is not None else getattr(
            self, "start_directory", None
        )
        self.required_cwd = required_cwd if required_cwd is not None else getattr(self, "required_cwd", None)
        self.require_answer = require_answer if require_answer is not None else getattr(self, "require_answer", False)
        self.marker_name = marker_name if marker_name is not None else getattr(self, "marker_name", None)
        self.marker_error = marker_error if marker_error is not None else getattr(
            self, "marker_error", Messages.MARKER_NOT_FOUND
        )
        self.expected_answer = expected_answer if expected_answer is not None else getattr(
            self, "expected_answer", None
        )
        self.success_message = success_message if success_message is not None else getattr(
            self, "success_message", Messages.CORRECT
        )
        self.allow_cwd_as_answer = allow_cwd_as_answer if allow_cwd_as_answer is not None else getattr(
            self, "allow_cwd_as_answer", False
        )

        inst_file = instructions_file if instructions_file is not None else getattr(self, "instructions_file", None)
        inst_text = instructions if instructions is not None else getattr(self, "instructions", "")

        if inst_file:
            content_path = Path(__file__).parent / "content" / inst_file
            if content_path.exists():
                self.instructions = dedent(content_path.read_text()).strip()
            else:
                self.instructions = f"Error: Instructions file {inst_file} not found."
        else:
            self.instructions = dedent(inst_text).strip()

    @abstractmethod
    def setup(self, workspace: Path) -> None:
        pass

    def _check_marker(self, state: GameStateProtocol) -> Optional[ValidationResult]:
        if not self.marker_name:
            return None
        username = getattr(state, "username", None)
        if username and not MarkerManager(username).exists(self.marker_name):
            return False, self.marker_error
        return None

    def _check_cwd(self) -> Optional[ValidationResult]:
        if not self.required_cwd:
            return None
        cwd_name = Path.cwd().name
        if cwd_name != self.required_cwd:
            return False, Messages.wrong_directory(cwd_name, self.required_cwd)
        return None

    def _check_answer_presence(self, answer: Optional[str]) -> Optional[ValidationResult]:
        if self.require_answer and answer is None:
            return False, Messages.ANSWER_REQUIRED
        return None

    def _check_cwd_as_answer(self, answer: Optional[str]) -> Optional[ValidationResult]:
        if answer is None and self.allow_cwd_as_answer and self.expected_answer:
            if Path.cwd().name == self.expected_answer:
                return True, self.success_message
            return False, Messages.wrong_directory(Path.cwd().name, self.expected_answer)
        return None

    def _check_expected_answer(self, answer: Optional[str]) -> Optional[ValidationResult]:
        if self.expected_answer and answer is not None:
            if answer.strip() == self.expected_answer:
                return True, self.success_message
            return False, Messages.INCORRECT
        return None

    def validate(self, answer: Optional[str], state: GameStateProtocol) -> ValidationResult:
        checks = [
            lambda: self._check_marker(state),
            lambda: self._check_cwd(),
            lambda: self._check_answer_presence(answer),
            lambda: self._check_cwd_as_answer(answer),
            lambda: self._check_expected_answer(answer),
        ]

        for check in checks:
            if result := check():
                return result

        for validator in self.validators:
            success, message = validator.validate(answer, state)
            if not success:
                return False, message

        return True, self.success_message

    def reset(self, workspace: Path) -> None:
        self.setup(workspace)

    def get_start_directory(self, workspace: Path) -> Optional[Path]:
        if self.start_directory is None:
            return None

        if not self.start_directory:
            return workspace

        return workspace / self.start_directory

    @property
    def hooks(self) -> dict[str, Callable[..., object]]:
        return {}

