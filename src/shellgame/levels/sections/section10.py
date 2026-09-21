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

section = Section(10, root="level-10")

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
    title = "Sekce 10: Chybové výstupy"
    instructions_file = "section10_intro.md"
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
        `shellgame submit <název>`
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
        Stejně jako u normálního výstupu se i u chyb rozhodujete mezi dvěma operátory:
        jeden soubor před zápisem vyprázdní, druhý zapisuje na jeho konec.

        ### Úkol
        V souboru `errors.log` už je z minula záznam `Old error 1`.
        Spusťte `buggy.sh` znovu tak, aby v souboru zůstal starý záznam i nová chyba.

        Než příkaz spustíte, rozmyslete si: který z obou operátorů soubor nejdřív
        vyprázdní a co by se v takovém případě stalo s řádkem `Old error 1`?

        ### Příkazy
        - `./script 2> soubor` - chybový výstup přepíše obsah souboru
        - `./script 2>> soubor` - chybový výstup přidá na konec souboru

        ### Odevzdání
        Odevzdejte název souboru.
        `shellgame submit <název>`
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
    success_message = "Správně! Dvojitá šipka zapisuje na konec, takže předchozí záznamy v logu zůstaly zachovány."


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
        `shellgame submit <název>`
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
                error_message=(
                    "Soubor neobsahuje oba typy výstupů. "
                    "Chybí-li normální výstup, přesměrovali jste jen chyby (2>). "
                    "Chybí-li chyby, zůstaly na obrazovce, protože samotné > bere jen stdout."
                ),
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
        `shellgame submit <název>`
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
        answer=ExactAnswer(
            "/dev/null",
            mistakes={
                "null": "Skoro. Odevzdejte celou cestu k tomu speciálnímu souboru, ne jen jeho název.",
                "dev/null": "Téměř. Jde o absolutní cestu, začíná tedy lomítkem od kořenového adresáře.",
            },
        ),
        requirements=(
            Evidence(
                MarkerManager.LEVEL9_4_DEV_NULL,
                "Spusťte `./buggy.sh` a přesměrujte stdout i stderr do `/dev/null`.",
            ),
        ),
    )
    success_message = "Správně! Zápis do /dev/null se rovnou zahodí, takže příkaz umlčíte bez zakládání logu."

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
        steps=(RunShell("./mixed.sh > output.log 2> errors.log"),),
        answer="2,3",
    )
    title = "Souhrn Sekce 10"
    instructions = """
        ### Výzva: Oddělení streamů

        Ukažte, že umíte uložit stdout a stderr odděleně.

        ### Úkol
        V `level-10/challenge` je skript `mixed.sh` který vypisuje:
        - normální výstup na stdout
        - chyby na stderr

        Spusťte skript **jednou** a současně uložte:
        1. pouze normální výstup do `output.log`
        2. pouze chyby do `errors.log`

        Odpovězte: kolik řádků má errors.log a kolik output.log?
        Formát: `chyby,výstup` (např. `3,5`)

        ### Připomenutí
        - `>` přesměruje standardní výstup.
        - `2>` přesměruje chybový výstup.

        V této výzvě potřebujete oba proudy zachytit zvlášť.

        ### Odevzdání
        `shellgame submit <chyby>,<výstup>`
        """
    hints = [
        (
            "Chyby se zapisují na stderr (descriptor 2), standardní výstup na stdout (descriptor 1). "
            "Každý proud může mít vlastní cíl."
        ),
        (
            "Za jeden příkaz lze zapsat přesměrování '>' i '2>'; pořadí zde nevadí, "
            "protože oba proudy míří do různých souborů."
        ),
        "Spusťte './mixed.sh > output.log 2> errors.log'. Počty zjistěte pomocí 'wc -l errors.log output.log'.",
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
                        "Počet chyb není správně. Zkontrolujte, že do errors.log směřuje pouze descriptor 2."
                    ),
                    invalid_message="Obě hodnoty musí být čísla.",
                ),
                IntegerAnswer(
                    3,
                    error_message=(
                        "Počet normálních řádků není správně. "
                        "Zkontrolujte, že do output.log směřuje pouze standardní výstup."
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
                (
                    "errors.log neobsahuje přesně pouze chybový výstup skriptu. "
                    "Buď do něj spadl i standardní výstup (oba proudy míří do jednoho souboru), "
                    "nebo jste skript spustili víckrát a záznamy se nasčítaly."
                ),
            ),
            FileLineCount(
                "challenge/output.log",
                3,
                (
                    "output.log neobsahuje přesně pouze standardní výstup skriptu. "
                    "Buď v něm skončily i chyby (chybí samostatné přesměrování druhého proudu), "
                    "nebo jste skript spustili víckrát a záznamy se nasčítaly."
                ),
            ),
        ),
    )
    success_message = "Perfektní! Dokončili jste Sekci 10. Stdout a stderr jsou pro vás jako otevřená kniha!"
