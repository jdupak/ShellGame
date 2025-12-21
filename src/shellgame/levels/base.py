"""Base level interface and abstract class."""

from abc import ABC, abstractmethod
from pathlib import Path
from typing import Optional

from shellgame.markers import MarkerManager
from shellgame.messages import Messages
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    ValidationResult,
    Validator,
)


class Level(ABC):
    """Abstract base class for all game levels.

    Supports declarative validation through attributes:
    - start_directory: Where to place user at level start (relative to workspace)
    - required_cwd: Directory user must be in to complete level
    - require_answer: Whether an explicit answer argument is required
    - marker_name: Shell wrapper marker that must exist
    - marker_error: Error message if marker doesn't exist
    - expected_answer: Simple expected answer string
    - success_message: Custom success message
    """

    def __init__(  # noqa: PLR0913
        self,
        id: str,
        section: int,
        title: str,
        instructions: str = "",
        hints: Optional[list[str]] = None,
        optional: bool = False,
        extension: bool = False,
        instructions_file: Optional[str] = None,
        validators: Optional[list[Validator]] = None,
        # New declarative attributes
        start_directory: Optional[str] = None,
        required_cwd: Optional[str] = None,
        require_answer: bool = False,
        marker_name: Optional[str] = None,
        marker_error: Optional[str] = None,
        expected_answer: Optional[str] = None,
        success_message: Optional[str] = None,
        allow_cwd_as_answer: bool = False,
    ):
        """Initialize a level.

        Args:
            id: Level identifier (e.g., "1.1")
            section: Section number
            title: Level title
            instructions: Instructions text to display (optional if file provided)
            hints: List of progressive hints
            optional: Whether this is an optional level
            extension: Whether this is an extension level
            instructions_file: Filename of markdown instructions in content/ directory
            validators: List of validators to check answer against
            start_directory: Starting directory relative to workspace
            required_cwd: Required current directory name for validation
            require_answer: Whether answer argument is mandatory
            marker_name: Shell marker file that must exist
            marker_error: Error message when marker is missing
            expected_answer: Simple expected answer for validation
            success_message: Custom success message
            allow_cwd_as_answer: If True, accept being in target dir as valid answer
        """
        self.id = id
        self.section = section
        self.title = title
        self.hints = hints or []
        self.optional = optional
        self.extension = extension
        self.validators = validators or []

        # Declarative validation attributes
        self.start_directory = start_directory
        self.required_cwd = required_cwd
        self.require_answer = require_answer
        self.marker_name = marker_name
        self.marker_error = marker_error or Messages.MARKER_NOT_FOUND
        self.expected_answer = expected_answer
        self.success_message = success_message or Messages.CORRECT
        self.allow_cwd_as_answer = allow_cwd_as_answer

        if instructions_file:
            content_path = Path(__file__).parent / "content" / instructions_file
            if content_path.exists():
                self.instructions = content_path.read_text()
            else:
                self.instructions = f"Error: Instructions file {instructions_file} not found."
        else:
            self.instructions = instructions

    @abstractmethod
    def setup(self, workspace: Path) -> None:
        """Generate files/directories for this level.

        Args:
            workspace: Path to workspace root directory
        """
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
        """Validate answer using declarative config and validators."""
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

        # Run configured validators
        for validator in self.validators:
            success, message = validator.validate(answer, state)
            if not success:
                return False, message

        return True, self.success_message

    def reset(self, workspace: Path) -> None:
        """Rebuild level structure (default: call setup).

        Args:
            workspace: Path to workspace root directory
        """
        self.setup(workspace)

    def get_start_directory(self, workspace: Path) -> Optional[Path]:
        """Get the absolute starting directory for this level.

        Args:
            workspace: Path to workspace root directory

        Returns:
            Absolute path to start directory, or None if not configured.
        """
        if self.start_directory is None:
            return None

        # Handle empty string as workspace root
        if not self.start_directory:
            return workspace

        return workspace / self.start_directory

    @property
    def hooks(self) -> dict[str, callable]:
        """Return a dictionary of command hooks.

        Returns:
            Dict mapping command name (e.g. 'cd') to a handler method.
            The handler signature depends on the command.
            For 'cd': (target: str|None, pwd: str|None, post_move: bool, state: GameStateProtocol) -> None
        """
        return {}

