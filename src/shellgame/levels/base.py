"""Base level interface and abstract class."""

from abc import ABC, abstractmethod
from typing import List, Tuple, Optional
from pathlib import Path

from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    Validator,
    ValidationResult,
    MarkerValidator,
    CurrentDirectoryValidator,
    BasenameValidator,
)
from shellgame.messages import Messages
from shellgame.markers import MarkerManager


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

    def __init__(
        self,
        id: str,
        section: int,
        title: str,
        instructions: str = "",
        hints: Optional[List[str]] = None,
        optional: bool = False,
        extension: bool = False,
        instructions_file: Optional[str] = None,
        validators: Optional[List[Validator]] = None,
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
                self.instructions = (
                    f"Error: Instructions file {instructions_file} not found."
                )
        else:
            self.instructions = instructions

    @abstractmethod
    def setup(self, workspace: Path) -> None:
        """Generate files/directories for this level.

        Args:
            workspace: Path to workspace root directory
        """
        pass

    def validate(
        self, answer: Optional[str], state: GameStateProtocol
    ) -> ValidationResult:
        """Validate answer using declarative config and validators.

        Validation order:
        1. Check marker file (if configured)
        2. Check required_cwd (if configured)
        3. Check require_answer (if True and no answer)
        4. Check allow_cwd_as_answer (if True and no answer, check cwd)
        5. Check expected_answer (if configured)
        6. Run configured validators

        Args:
            answer: User's submitted answer (or None if checking environment)
            state: Current game state

        Returns:
            Tuple of (success: bool, message: str)
        """
        # 1. Check marker if required
        if self.marker_name:
            username = getattr(state, "username", None)
            if username:
                markers = MarkerManager(username)
                if not markers.exists(self.marker_name):
                    return False, self.marker_error

        # 2. Check required current directory
        if self.required_cwd:
            cwd_name = Path.cwd().name
            if cwd_name != self.required_cwd:
                return False, Messages.wrong_directory(cwd_name, self.required_cwd)

        # 3. Check if answer is required
        if self.require_answer and answer is None:
            return False, Messages.ANSWER_REQUIRED

        # 4. If no answer but allow_cwd_as_answer, check current directory
        if answer is None and self.allow_cwd_as_answer and self.expected_answer:
            if Path.cwd().name == self.expected_answer:
                return True, self.success_message
            return False, Messages.wrong_directory(
                Path.cwd().name, self.expected_answer
            )

        # 5. Check expected answer
        if self.expected_answer and answer is not None:
            if answer.strip() == self.expected_answer:
                return True, self.success_message
            # Do not leak the expected answer in the error message for simple
            # expected-vs-got checks. Keep feedback actionable but non-spoiling.
            return False, Messages.INCORRECT

        # 6. Run configured validators
        if self.validators:
            for validator in self.validators:
                success, message = validator.validate(answer, state)
                if not success:
                    return False, message
            return True, self.success_message

        # Default: if nothing configured, level completes
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
