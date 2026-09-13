from __future__ import annotations

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    Completion,
    Evidence,
    ExactAnswer,
    FileLineCount,
    IntegerAnswer,
    TextFileContent,
    TupleAnswer,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import RecordFdEvidence, RunShell, Solution
from shellgame.markers import MarkerManager
from shellgame.protocols import GameStateProtocol

section = Section(9, root="level-9")

_BUGGY_SCRIPT = "#!/bin/bash\necho 'This is normal output'\necho 'This is an error message' >&2\n"
_MIXED_SCRIPT = """#!/bin/bash
echo "Line 1 - normal output"
echo "ERROR: Something went wrong" >&2
echo "Line 2 - more output"
echo "ERROR: Another problem" >&2
echo "Line 3 - final output"
"""


@section.level(0)
class SectionIntro(Level):
    is_intro = True
    title = "Sekce 9: Chybové výstupy"
    instructions_file = "section9_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"


@section.level(1)
class StderrToFileLevel(Level):
    solution = Solution(steps=(RunShell("./buggy.sh 2> errors.log"),), answer="errors.log")
    title = "Přesměrování chyb"
    instructions = """
        Standardní chybový výstup (stderr) používá deskriptor souboru 2.
        Pro přesměrování pouze chyb použijte `2>`.

        ### Proč je to důležité
        Při běhu programů často chcete zachytit chybové hlášky do logu,
        zatímco normální výstup zobrazíte uživateli. Oddělení stdout a stderr
        je klíčové pro diagnostiku problémů.

        ### Úkol
        V adresáři je skript `buggy.sh`, který vypisuje normální text i chybové zprávy.
        Spusťte ho a přesměrujte POUZE chybové zprávy do souboru `errors.log`.

        ### Příkazy
        - `./script 2> soubor` - přesměruje stderr do souboru

        ### Odevzdání
        Odevzdejte název vytvořeného souboru.
        `shellgame submit errors.log`
        """
    hints = [
        "Běžný výstup jde na stdout (1), chyby na stderr (2). Jak přesměrujete jen dvojku?",
        "Syntaxe je: příkaz 2> soubor. Zkuste to se skriptem buggy.sh.",
        "Použijte './buggy.sh 2> errors.log'.",
    ]
    start_directory = ""
    fixture = WorkspaceFixture(
        files=(FileFixture("buggy.sh", _BUGGY_SCRIPT, mode=0o755),),
        clean=("errors.log",),
    )
    completion = Completion(
        answer=ExactAnswer("errors.log"),
        requirements=(
            TextFileContent(
                "errors.log",
                excludes=("This is normal output",),
                error_message="Soubor obsahuje i normální výstup (použili jste &> nebo chybí 2?).",
                missing_message="Soubor neexistuje.",
            ),
            TextFileContent(
                "errors.log",
                contains=("This is an error message",),
                error_message="Soubor neobsahuje očekávanou chybu.",
            ),
        ),
    )
    success_message = "Správně! Soubor obsahuje pouze chyby."


@section.level(2)
class AppendStderrToFileLevel(Level):
    solution = Solution(steps=(RunShell("./buggy.sh 2>> errors.log"),), answer="errors.log")
    title = "Přidávání chyb"
    instructions = """
        Stejně jako u normálního výstupu můžete chyby přidávat na konec souboru pomocí `2>>`.

        ### Úkol
        Spusťte `buggy.sh` znovu, ale tentokrát PŘIDEJTE chybové zprávy na konec `errors.log`.
        Nepřepisujte existující chyby!

        ### Příkazy
        - `./script 2>> soubor` - přidá stderr na konec souboru

        ### Odevzdání
        Odevzdejte název souboru.
        `shellgame submit errors.log`
        """
    hints = [
        "Jaký je rozdíl mezi > a >>? Jeden přepisuje, druhý přidává.",
        "Pro přidání chyb na konec použijte dvě šipky: 2>>",
        "Použijte './buggy.sh 2>> errors.log'.",
    ]
    start_directory = ""
    fixture = WorkspaceFixture(
        files=(
            FileFixture("buggy.sh", _BUGGY_SCRIPT, mode=0o755),
            FileFixture("errors.log", "Old error 1\n"),
        )
    )
    completion = Completion(
        answer=ExactAnswer("errors.log"),
        requirements=(
            TextFileContent(
                "errors.log",
                contains=("Old error 1",),
                error_message="Původní obsah zmizel (použili jste 2> místo 2>>?).",
                missing_message="Soubor neexistuje.",
            ),
            TextFileContent(
                "errors.log",
                contains=("This is an error message",),
                error_message="Soubor neobsahuje novou chybu.",
            ),
        ),
    )


@section.level(3)
class AllOutputToFileLevel(Level):
    solution = Solution(steps=(RunShell("./buggy.sh &> all_output.log"),), answer="all_output.log")
    title = "Všechny výstupy"
    instructions = """
        Někdy chcete zachytit VŠECHNO - normální výstup i chyby do jednoho souboru.
        K tomu slouží `&>`.

        ### Proč je to užitečné
        Při ladění skriptů nebo automatizaci často potřebujete kompletní log
        všeho, co program vypsal - ať už to byla informace nebo chyba.

        ### Úkol
        Spusťte `buggy.sh` a přesměrujte OBOJÍ (stdout i stderr) do `all_output.log`.

        ### Příkazy
        - `./script &> soubor` - přesměruje stdout i stderr

        ### Odevzdání
        Odevzdejte název souboru.
        `shellgame submit all_output.log`
        """
    hints = [
        "Ampersand (&) v tomto kontextu znamená 'obojí' - stdout i stderr.",
        "Kombinace &> je zkratka pro přesměrování obou výstupů.",
        "Použijte './buggy.sh &> all_output.log'.",
    ]
    start_directory = ""
    fixture = WorkspaceFixture(
        files=(FileFixture("buggy.sh", _BUGGY_SCRIPT, mode=0o755),),
        clean=("all_output.log",),
    )
    completion = Completion(
        answer=ExactAnswer("all_output.log"),
        requirements=(
            TextFileContent(
                "all_output.log",
                contains=("This is normal output", "This is an error message"),
                error_message="Soubor neobsahuje oba typy výstupů.",
                missing_message="Soubor neexistuje.",
            ),
        ),
    )
    success_message = "Správně! Máme všechno."


@section.level(4)
class DevNullLevel(Level):
    solution = Solution(steps=(RecordFdEvidence(),), answer="/dev/null")
    title = "Černá díra"
    instructions = """
        `/dev/null` je speciální soubor, který zahodí všechno, co do něj pošlete.
        Je užitečný pro umlčení hlučných příkazů.

        ### Proč je to užitečné
        Některé příkazy vypisují spoustu informací, které nepotřebujete.
        Místo zahlcení obrazovky je můžete "poslat do černé díry".

        ### Úkol
        Spusťte `buggy.sh` a umlčte VŠECHNY výstupy (stdout i stderr) přesměrováním do `/dev/null`.

        ### Příkazy
        - `./script &> /dev/null` - zahodí veškerý výstup

        ### Odevzdání
        Odevzdejte název speciálního souboru, který jste použili.
        `shellgame submit /dev/null`
        """
    hints = [
        "Kam v Linuxu 'vyhodíte' data, která nechcete? Existuje speciální soubor...",
        "Soubor /dev/null je jako černá díra - vše pohltí a nic nevrátí.",
        "Použijte './buggy.sh &> /dev/null'.",
    ]
    start_directory = ""
    fixture = WorkspaceFixture(
        files=(
            FileFixture(
                "buggy.sh",
                _BUGGY_SCRIPT + '"$SHELLGAME_FD_HOOK"\n',
                mode=0o755,
            ),
        )
    )
    completion = Completion(
        answer=ExactAnswer("/dev/null"),
        requirements=(
            Evidence(
                MarkerManager.LEVEL9_4_DEV_NULL,
                "Spusťte `./buggy.sh` a přesměrujte stdout i stderr do `/dev/null`.",
            ),
        ),
    )

    @override
    def record_fd_evidence(
        self,
        *,
        stdout_target: str,
        stderr_target: str,
        state: GameStateProtocol,
    ) -> None:
        if stdout_target == "/dev/null" and stderr_target == "/dev/null":
            MarkerManager.from_state(state).create(MarkerManager.LEVEL9_4_DEV_NULL)


@section.level(5)
class StreamsChallengeLevel(Level):
    solution = Solution(
        steps=(
            RunShell("./mixed.sh 2> errors.log"),
            RunShell("./mixed.sh > output.log"),
        ),
        answer="2,3",
    )
    title = "Souhrn Sekce 9"
    instructions = """
        ### Výzva: Mistr streamů

        Ukažte, že rozumíte stdout, stderr a /dev/null!

        ### Úkol
        V `level-9/challenge` je skript `mixed.sh` který vypisuje:
        - normální výstup na stdout
        - chyby na stderr

        1. Spusťte skript a uložte **pouze chyby** do `errors.log`
        2. Spusťte znovu a uložte **pouze normální výstup** do `output.log`

        Odpovězte: kolik řádků má errors.log a kolik output.log?
        Formát: `chyby,výstup` (např. `3,5`)

        ### Shrnutí příkazů Sekce 9
        ```
        ./skript > out.txt       → stdout do souboru
        ./skript 2> err.txt      → stderr do souboru
        ./skript &> all.txt      → vše do souboru
        ./skript 2>&1            → stderr do stdout
        ./skript > /dev/null     → zahodit stdout
        ```

        ### Odevzdání
        `shellgame submit <chyby>,<výstup>`
        """
    hints = [
        "Chyby se zapisují na chybový výstup (stderr, descriptor 2), standardní výstup na stdout (descriptor 1).",
        "Spusťte './mixed.sh 2> errors.log' pro uložení chyb a './mixed.sh > output.log' pro běžný výstup.",
        "Počet řádků spočítejte pomocí 'wc -l errors.log output.log' a odevzdejte dvě čísla oddělená čárkou.",
    ]
    start_directory = "challenge"
    fixture = WorkspaceFixture(
        files=(FileFixture("challenge/mixed.sh", _MIXED_SCRIPT, mode=0o755),),
        clean=("challenge",),
    )
    completion = Completion(
        answer=TupleAnswer(
            (
                IntegerAnswer(
                    2,
                    error_message=(
                        "Počet chyb není správně. Spusťte './mixed.sh 2> errors.log' a pak 'wc -l errors.log'."
                    ),
                    invalid_message="Obě hodnoty musí být čísla.",
                ),
                IntegerAnswer(
                    3,
                    error_message=(
                        "Počet normálních řádků není správně. "
                        "Spusťte './mixed.sh > output.log' a pak 'wc -l output.log'."
                    ),
                    invalid_message="Obě hodnoty musí být čísla.",
                ),
            ),
            format_message="Formát odpovědi: chyby,výstup (např. 3,5)",
        ),
        requirements=(
            FileLineCount(
                "challenge/errors.log",
                2,
                "errors.log nemá přesně dva řádky chyb.",
            ),
            FileLineCount(
                "challenge/output.log",
                3,
                "output.log nemá přesně tři řádky normálního výstupu.",
            ),
        ),
    )
    success_message = "Perfektní! Dokončili jste Sekci 9. Stdout a stderr jsou pro vás jako otevřená kniha!"
