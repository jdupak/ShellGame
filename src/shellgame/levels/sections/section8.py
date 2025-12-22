from __future__ import annotations

import shutil
from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    CommonMistakeValidator,
    IntegerValidator,
    StringValidator,
    ValidationResult,
)

section = Section()


def _setup_access_log(workspace: Path) -> None:
    level_dir = workspace / "level-8" / "pipes"
    level_dir.mkdir(parents=True, exist_ok=True)

    log_content = """2024-01-01 10:00:00 INFO Server started
2024-01-01 10:05:23 ERROR Connection refused
2024-01-01 10:10:45 INFO User logged in
2024-01-01 10:15:00 WARNING Low memory
2024-01-01 10:20:12 ERROR Database timeout
2024-01-01 10:25:00 INFO Request processed
2024-01-01 10:30:33 ERROR File not found
2024-01-01 10:35:00 INFO Cache cleared
2024-01-01 10:40:55 ERROR Permission denied
2024-01-01 10:45:00 DEBUG Verbose output
2024-01-01 10:50:18 ERROR Network unreachable
2024-01-01 10:55:00 INFO Backup completed
2024-01-01 11:00:00 ERROR Disk full
2024-01-01 11:05:00 INFO Server shutdown
2024-01-01 11:10:42 ERROR Service unavailable
"""
    (level_dir / "access.log").write_text(log_content)


def _setup_long_file(workspace: Path) -> None:
    level_dir = workspace / "level-8" / "headtail"
    level_dir.mkdir(parents=True, exist_ok=True)

    lines = ["START of the file - this is line 1"]
    for i in range(2, 50):
        lines.append(f"Line number {i} with some content")
    lines.append("END of the file - this is line 50")
    (level_dir / "long_file.txt").write_text("\n".join(lines) + "\n")


def _setup_article_file(workspace: Path) -> None:
    level_dir = workspace / "level-8" / "wc"
    level_dir.mkdir(parents=True, exist_ok=True)

    article = """Linux je svobodný operační systém.
Byl vytvořen Linusem Torvaldsem v roce 1991.
Dnes pohání většinu serverů na internetu.
Je základem systému Android a mnoha dalších.
Open source komunita ho neustále vylepšuje.
"""
    (level_dir / "article.txt").write_text(article)


def _setup_visitors_file(workspace: Path) -> None:
    level_dir = workspace / "level-8" / "sort"
    level_dir.mkdir(parents=True, exist_ok=True)

    visitors = """Alice
Bob
Charlie
Alice
David
Bob
Eve
Alice
"""
    (level_dir / "visitors.txt").write_text(visitors)


def _setup_section8_challenge(workspace: Path) -> None:
    challenge_dir = workspace / "level-8" / "challenge"

    if challenge_dir.exists():
        shutil.rmtree(challenge_dir)

    challenge_dir.mkdir(parents=True, exist_ok=True)

    (challenge_dir / "sample1.txt").write_text("sample")
    (challenge_dir / "sample2.txt").write_text("sample")


@section.level
class SectionIntro(Level):
    title = "Sekce 8: Přesměrování výstupu"
    instructions_file = "section8_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"

    @override
    def setup(self, workspace: Path) -> None:
        return

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


@section.level
class RedirectLsToFileLevel(Level):
    title = "Uložení výstupu"
    instructions = """\
        # Uložení výstupu

        Operátor `>` přesměruje výstup příkazu do souboru. Pokud soubor neexistuje, vytvoří se.
        Pokud existuje, **přepíše se**.

        ## Úkol
        Uložte seznam souborů v aktuálním adresáři (výstup `ls`) do souboru `seznam.txt`.

        ## Příkazy
        - `ls > seznam.txt`

        ## Odevzdání
        Odevzdejte název vytvořeného souboru.
        `shellgame submit -f seznam.txt`
        """
    hints = [
        "Použijte operátor '>' pro přesměrování výstupu.",
        "Příkaz 'ls' vypíše obsah adresáře.",
        "Zkuste: 'ls > seznam.txt'.",
    ]
    start_directory = "level-8/redirect"
    require_answer = True
    validators = [StringValidator("seznam.txt")]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-8" / "redirection"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "file1").touch()
        (level_dir / "file2").touch()

        target = level_dir / "seznam.txt"
        if target.exists():
            target.unlink()

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        target = state.workspace / "level-8" / "redirection" / "seznam.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "file1" in content and "file2" in content:
            return True, "Správně!"
        return False, "Soubor neobsahuje očekávaný výstup příkazu ls."


@section.level
class AppendWithRedirectLevel(Level):
    title = "Přidání na konec"
    instructions = """\
        # Přidání na konec

        Operátor `>>` (append) přidá výstup na konec souboru, aniž by smazal původní obsah.

        ## Úkol
        Máte soubor `log.txt` s nějakým obsahem. Přidejte na jeho konec text "Konec logu"
        pomocí příkazu `echo`.

        ## Příkazy
        - `echo "Text" >> soubor`

        ## Odevzdání
        Odevzdejte název souboru.
        `shellgame submit -f log.txt`
        """
    hints = [
        "Použijte 'echo \"Konec logu\" >> log.txt'.",
        "Dvě šipky >> znamenají append.",
    ]
    start_directory = "level-8/redirect"
    require_answer = True
    validators = [StringValidator("log.txt")]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-8" / "redirection"
        level_dir.mkdir(parents=True, exist_ok=True)
        (level_dir / "log.txt").write_text("Start logu\nZaznam 1\n")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        target = state.workspace / "level-8" / "redirection" / "log.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "Start logu" in content and "Konec logu" in content:
            return True, "Správně!"
        if "Konec logu" in content:
            return False, "Zdá se, že jste přepsali původní obsah (použili jste > místo >>?)."
        return False, "Soubor neobsahuje nový text."


@section.level
class ConcatenatePartsLevel(Level):
    title = "Spojování souborů"
    instructions = """\
        # Spojování souborů

        Příkaz `cat` (concatenate) umí vypsat obsah více souborů za sebou.
        Když to zkombinujete s přesměrováním, můžete spojit více souborů do jednoho.

        ## Úkol
        Spojte obsah souborů `part1.txt` a `part2.txt` do nového souboru `full.txt`.

        ## Příkazy
        - `cat soubor1 soubor2 > novy_soubor`

        ## Odevzdání
        Odevzdejte název nového souboru.
        `shellgame submit -f full.txt`
        """
    hints = [
        "Použijte 'cat part1.txt part2.txt > full.txt'.",
        "Pořadí argumentů určuje pořadí v cílovém souboru.",
    ]
    start_directory = "level-8/redirect"
    require_answer = True
    validators = [StringValidator("full.txt")]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-8" / "concat"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "part1.txt").write_text("First part.\n")
        (level_dir / "part2.txt").write_text("Second part.\n")

        target = level_dir / "full.txt"
        if target.exists():
            target.unlink()

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        target = state.workspace / "level-8" / "concat" / "full.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "First part." in content and "Second part." in content:
            return True, "Správně!"
        return False, "Soubor neobsahuje text z obou částí."


@section.level
class EchoCreateFileLevel(Level):
    title = "Vytvoření souboru s obsahem"
    instructions = """\
        # Vytvoření souboru s obsahem

        Místo editoru můžete pro vytvoření krátkého souboru použít `echo` a přesměrování.

        ## Úkol
        Vytvořte soubor `pozdrav.txt`, který bude obsahovat text "Ahoj svete".

        ## Příkazy
        - `echo "Ahoj svete" > pozdrav.txt`

        ## Odevzdání
        Odevzdejte název souboru.
        `shellgame submit -f pozdrav.txt`
        """
    hints = [
        "Použijte 'echo \"Ahoj svete\" > pozdrav.txt'.",
        "Uvozovky jsou důležité, pokud text obsahuje mezery.",
    ]
    start_directory = "level-8/redirect"
    require_answer = True
    validators = [StringValidator("pozdrav.txt")]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-8" / "echo"
        level_dir.mkdir(parents=True, exist_ok=True)

        target = level_dir / "pozdrav.txt"
        if target.exists():
            target.unlink()

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        target = state.workspace / "level-8" / "echo" / "pozdrav.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text().strip()
        if content == "Ahoj svete":
            return True, "Správně!"
        return False, f"Očekáváno 'Ahoj svete', nalezeno '{content}'."


@section.level
class PipeGrepAndCountLevel(Level):
    title = "Propojení příkazů (Pipes)"
    instructions = """\
        # Propojení příkazů pomocí rour (pipes)

        Znak `|` (pipe/roura) pošle výstup jednoho příkazu jako vstup druhému.

        ## Úkol
        V aktuálním adresáři je soubor `access.log` s mnoha řádky.
        Spočítejte, kolik řádků obsahuje slovo "ERROR".

        Použijte: `grep "ERROR" access.log | wc -l`

        ## Odevzdání
        Odevzdejte nalezený počet (číslo).
        `shellgame submit -f <číslo>`
        """
    hints = [
        "Pipe (|) propojuje výstup prvního příkazu se vstupem druhého.",
        "grep najde řádky s 'ERROR', wc -l je spočítá. Spojte je pomocí |.",
        'Použijte: grep "ERROR" access.log | wc -l',
    ]
    require_answer = True
    validators = [
        IntegerValidator(7),
        CommonMistakeValidator(
            {
                "15": "Spočítali jste všechny řádky. Potřebujete jen ty s 'ERROR'. Použijte grep před wc.",
            }
        ),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_access_log(workspace)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        # Parent validation handles IntegerValidator + common mistakes.
        return super().validate(answer, state)


@section.level
class HeadTailFirstAndLastWordLevel(Level):
    title = "Začátek a konec souboru"
    instructions = """\
        # Head a Tail - prohlížení částí souboru

        ## Úkol
        V souboru `long_file.txt` je 50 řádků.
        1. Zjistěte první slovo na 1. řádku (pomocí `head -n 1`)
        2. Zjistěte první slovo na posledním řádku (pomocí `tail -n 1`)

        ## Odevzdání
        Odevzdejte obě slova oddělená čárkou: `první,poslední`
        `shellgame submit -f START,END`
        """
    hints = [
        "head -n 1 zobrazí první řádek, tail -n 1 zobrazí poslední.",
        "První řádek začíná slovem 'START', poslední slovem 'END'.",
        "Odpověď je: START,END",
    ]
    require_answer = True
    expected_answer = "START,END"
    success_message = "Správně! Head a tail jsou skvělé pro rychlý náhled do souborů."
    validators = [
        CommonMistakeValidator(
            {
                "START END": "Použijte čárku: START,END",
                "START;END": "Použijte čárku: START,END",
                "START|END": "Použijte čárku: START,END",
            }
        )
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_long_file(workspace)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if success:
            return True, msg

        if answer is None:
            return False, "Zadejte odpověď ve formátu: první_slovo,poslední_slovo"

        normalized = answer.strip().upper()
        if "," not in normalized:
            return False, "Formát: první_slovo,poslední_slovo (např. AHOJ,SVET)"

        parts = normalized.split(",")
        first = parts[0].strip()
        last = parts[1].strip() if len(parts) > 1 else ""

        if first != "START":
            return False, f"První slovo není '{first}'. Použijte 'head -n 1 long_file.txt'."
        return False, f"Poslední slovo není '{last}'. Použijte 'tail -n 1 long_file.txt'."


@section.level
class WordAndLineCountLevel(Level):
    title = "Počítání (wc)"
    instructions = """\
        # Příkaz wc (word count)

        ## Úkol
        Zjistěte o souboru `article.txt`:
        1. Kolik má řádků? (`wc -l`)
        2. Kolik má slov? (`wc -w`)

        ## Odevzdání
        Odevzdejte: `řádky,slova` (např. `10,50`)
        `shellgame submit -f <řádky>,<slova>`
        """
    hints = [
        "wc -l počítá řádky, wc -w počítá slova.",
        "Článek má 5 řádků a 25 slov.",
        "Odpověď je: 5,25",
    ]
    require_answer = True
    expected_answer = "5,25"
    success_message = "Správně! Příkaz wc je nepostradatelný pro rychlou analýzu souborů."
    validators = [
        CommonMistakeValidator(
            {
                "5 25": "Použijte čárku: 5,25",
                "5;25": "Použijte čárku: 5,25",
            }
        )
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_article_file(workspace)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if success:
            return True, msg

        if answer is None:
            return False, "Zadejte odpověď ve formátu: řádky,slova"

        if "," not in answer:
            return False, "Formát: řádky,slova (např. 10,50)"

        parts = answer.split(",")
        try:
            lines = int(parts[0].strip())
            words = int(parts[1].strip())
        except ValueError:
            return False, "Obě hodnoty musí být čísla."

        if lines != 5:
            return False, f"Počet řádků není {lines}. Použijte 'wc -l article.txt'."
        return False, f"Počet slov není {words}. Použijte 'wc -w article.txt'."


@section.level
class SortUniqCountUniqueLevel(Level):
    title = "Řazení a odstranění duplicit"
    instructions = """\
        # Sort a Uniq - řazení a deduplikace

        Příkazy `sort` a `uniq` jsou mocné nástroje pro zpracování textových dat.

        ## Úkol
        V souboru `visitors.txt` jsou jména návštěvníků (někteří přišli vícekrát).
        Zjistěte, kolik je UNIKÁTNÍCH návštěvníků.

        Použijte: `sort visitors.txt | uniq | wc -l`

        ## Odevzdání
        Odevzdejte počet unikátních návštěvníků.
        `shellgame submit -f <číslo>`
        """
    hints = [
        "Příkaz uniq odstraní duplikáty, ale jen sousedící! Proto nejdřív sort.",
        "Řetězec: sort → uniq → wc -l spočítá unikátní řádky.",
        "V souboru je 5 unikátních jmen.",
    ]
    extension = True
    require_answer = True
    validators = [
        IntegerValidator(5),
        CommonMistakeValidator(
            {
                "8": "Spočítali jste všechny řádky, ne unikátní. Zkuste: sort visitors.txt | uniq | wc -l",
                "3": "Možná jste spočítali jen duplikáty. Hledáme počet unikátních jmen.",
            }
        ),
    ]
    success_message = "Správně! Sort | uniq je klasická kombinace pro práci s daty."

    @override
    def setup(self, workspace: Path) -> None:
        _setup_visitors_file(workspace)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


@section.level
class SectionSummaryChallengeLevel(Level):
    title = "Souhrn Sekce 8"
    instructions = """\
        ### Výzva: Mistr přesměrování a pipes

        Ukažte, že ovládáte přesměrování i roury!

        ### Úkol
        V `level-8/challenge`:

        1. Vytvořte `message.txt` s textem "Hello World" pomocí echo
        2. Přidejte na konec souboru další řádek "Goodbye" (append)
        3. Spočítejte, kolik `.txt` souborů je v adresáři pomocí `ls *.txt | wc -l`

        Odevzdejte: **<počet_txt_souborů>**

        ### Odevzdání
        `shellgame submit <počet>`
        """
    hints = [
        "První řádek: 'echo \"Hello World\" > message.txt'. Druhý: 'echo \"Goodbye\" >> message.txt' (dva >>).",
        "Pro počítání: 'ls *.txt | wc -l'. Nezapomeňte vytvořit message.txt!",
        "Po vytvoření message.txt budou v adresáři 3 .txt soubory (sample1.txt, sample2.txt, message.txt).",
    ]
    require_answer = True
    validators = [
        IntegerValidator(3),
        CommonMistakeValidator(
            {
                "2": "Spočítali jste jen sample1.txt a sample2.txt. Vytvořili jste message.txt?",
            }
        ),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_section8_challenge(workspace)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        # First, run declarative checks (require_answer + integer validator and common mistakes).
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        challenge_dir = state.workspace / "level-8" / "challenge"
        message_file = challenge_dir / "message.txt"
        if not message_file.exists():
            return False, "Chybí message.txt. Vytvořte pomocí 'echo \"Hello World\" > message.txt'."

        content = message_file.read_text()
        lines = content.strip().split("\n")

        if len(lines) < 2:
            return (
                False,
                "message.txt má jen jeden řádek. Přidejte druhý pomocí 'echo \"Goodbye\" >> message.txt' (dva >>).",
            )

        if "Hello World" not in lines[0]:
            return False, "První řádek message.txt nemá 'Hello World'."

        if "Goodbye" not in lines[1]:
            return False, "Druhý řádek message.txt nemá 'Goodbye'."

        return True, "Skvělé! Dokončili jste Sekci 8. Přesměrování i pipes máte v malíku!"


def get_levels() -> list[Level]:
    return section.levels
