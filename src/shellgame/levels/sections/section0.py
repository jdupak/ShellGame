"""Section 0: Introduction and Game Mechanics."""

from __future__ import annotations

from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import StringValidator


class Level0_0(Level):
    """Level 0.0: Introduction - How to play."""

    def __init__(self) -> None:
        super().__init__(
            id="0.0",
            section=0,
            title="Vítejte v ShellGame",
            instructions_file="section0_intro.md",
            hints=["Přečtěte si instrukce a pokračujte příkazem 'shellgame submit'."],
            start_directory="",
        )

    @override
    def setup(self, workspace: Path) -> None:
        """No setup needed for intro."""
        pass

    @override
    def validate(
        self, answer: str | None, state: GameStateProtocol
    ) -> tuple[bool, str]:
        """Always valid, just moving to next level."""
        return True, "Vítejte ve hře!"


class Level0_1(Level):
    """Level 0.1: Warmup Task."""

    def __init__(self) -> None:
        super().__init__(
            id="0.1",
            section=0,
            title="Zahřívací kolo",
            instructions="""
### Zahřívací kolo

Toto je první testovací úkol, abychom si ověřili, že vše funguje.

### Úkol
Pro postup do první sekce stačí odevzdat heslo `start`.

Odevzdejte pomocí: `shellgame submit start`
            """.strip(),
            hints=[
                "Opravdu jen napište: shellgame submit start",
                "Nic víc v tom nehledejte :)",
            ],
            start_directory="",
        )

    @override
    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        pass

    @override
    def validate(
        self, answer: str | None, state: GameStateProtocol
    ) -> tuple[bool, str]:
        """Validate that answer is 'start'."""
        if answer is None:
            return False, "Musíte zadat heslo: shellgame submit start"

        validator = StringValidator("start", case_sensitive=False)
        return validator.validate(answer, state.workspace)


def get_levels() -> list[Level]:
    """
    Return all Section 0 level classes.

    Returns:
        List of Level instances for Section 0
    """
    return [
        Level0_0(),
        Level0_1(),
    ]
