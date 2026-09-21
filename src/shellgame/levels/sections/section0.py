"""Section 0: Introduction and Game Mechanics."""

from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import Completion, ExactAnswer

section = Section(0)


@section.level(0)
class IntroLevel(Level):
    is_intro = True
    title = "Vítejte v ShellGame"
    instructions_file = "section0_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    start_directory = ""
    success_message = "Vítejte ve hře!"


@section.level(1)
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
        "Heslo je uvedeno přímo v zadání úkolu výše.",
        "Zadejte příkaz 'shellgame submit' následovaný tímto heslem.",
        "Opravdu jen napište: shellgame submit start",
    ]
    start_directory = ""
    completion = Completion(answer=ExactAnswer("start", case_sensitive=False))
    success_message = "Správně! Takhle se odevzdává každý úkol: `shellgame submit` a vaše odpověď."
