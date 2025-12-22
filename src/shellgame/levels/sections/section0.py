"""Section 0: Introduction and Game Mechanics."""

from __future__ import annotations

from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import StringValidator, ValidationResult

section = Section()


@section.level
class IntroLevel(Level):
    title = "Vítejte v ShellGame"
    instructions_file = "section0_intro.md"
    hints = ["Přečtěte si instrukce a pokračujte příkazem 'shellgame submit'."]
    start_directory = ""

    @override
    def setup(self, workspace: Path) -> None:
        pass

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        super().validate(answer, state)
        return True, "Vítejte ve hře!"


@section.level
class WarmupPasswordLevel(Level):
    title = "Zahřívací kolo"
    instructions = """
        ### Zahřívací kolo

        Toto je první testovací úkol, abychom si ověřili, že vše funguje.

        ### Úkol
        Pro postup do první sekce stačí odevzdat heslo `start`.

        Odevzdejte pomocí: `shellgame submit start`
        """
    hints = [
        "Opravdu jen napište: shellgame submit start",
        "Nic víc v tom nehledejte :)",
    ]
    start_directory = ""
    require_answer = True
    validators = [StringValidator("start", case_sensitive=False)]

    @override
    def setup(self, workspace: Path) -> None:
        pass


def get_levels() -> list[Level]:
    return section.levels
