from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    Completion,
    ExactAnswer,
    IntegerAnswer,
    TextFileContent,
    TupleAnswer,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import RunShell, Solution

section = Section(8, root="level-8")


_ACCESS_LOG = """2024-01-01 10:00:00 INFO Server started
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
_LONG_FILE = "\n".join(
    [
        "START of the file - this is line 1",
        *(f"Line number {index} with some content" for index in range(2, 50)),
        "END of the file - this is line 50",
        "",
    ]
)
_ARTICLE = """Linux je svobodný operační systém.
Byl vytvořen Linusem Torvaldsem v roce 1991.
Dnes pohání většinu serverů na internetu.
Je základem systému Android a mnoha dalších.
Open source komunita ho neustále vylepšuje.
"""
_VISITORS = """Alice
Bob
Charlie
Alice
David
Bob
Eve
Alice
"""


@section.level(0)
class SectionIntro(Level):
    is_intro = True
    title = "Sekce 8: Přesměrování výstupu"
    instructions_file = "section8_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"


@section.level(1)
class RedirectLsToFileLevel(Level):
    solution = Solution(steps=(RunShell("ls > seznam.txt"),), answer="seznam.txt")
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
        `shellgame submit seznam.txt`
        """
    hints = [
        "Použijte operátor '>' pro přesměrování výstupu.",
        "Příkaz 'ls' vypíše obsah adresáře.",
        "Zkuste: 'ls > seznam.txt'.",
    ]
    start_directory = "redirection"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("redirection/file1"),
            FileFixture("redirection/file2"),
        ),
        clean=("redirection/seznam.txt",),
    )
    completion = Completion(
        answer=ExactAnswer("seznam.txt"),
        requirements=(
            TextFileContent(
                "redirection/seznam.txt",
                contains=("file1", "file2"),
                error_message="Soubor neobsahuje očekávaný výstup příkazu ls.",
                missing_message="Soubor neexistuje.",
            ),
        ),
    )


@section.level(2)
class AppendWithRedirectLevel(Level):
    solution = Solution(steps=(RunShell("echo 'Konec logu' >> log.txt"),), answer="log.txt")
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
        `shellgame submit log.txt`
        """
    hints = [
        "Dvě šipky '>>' znamenají append (připojení na konec souboru bez přepsání obsahu).",
        "Spusťte 'echo \"Konec logu\" >> log.txt'.",
    ]
    start_directory = "redirection"
    fixture = WorkspaceFixture(files=(FileFixture("redirection/log.txt", "Start logu\nZaznam 1\n"),))
    completion = Completion(
        answer=ExactAnswer("log.txt"),
        requirements=(
            TextFileContent(
                "redirection/log.txt",
                contains=("Start logu",),
                error_message="Zdá se, že jste přepsali původní obsah (použili jste > místo >>?).",
                missing_message="Soubor neexistuje.",
            ),
            TextFileContent(
                "redirection/log.txt",
                contains=("Konec logu",),
                error_message="Soubor neobsahuje nový text.",
            ),
        ),
    )


@section.level(3)
class ConcatenatePartsLevel(Level):
    solution = Solution(steps=(RunShell("cat part1.txt part2.txt > full.txt"),), answer="full.txt")
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
        `shellgame submit full.txt`
        """
    hints = [
        "Příkaz 'cat' umí přijmout více souborů najednou a vypsat jejich obsahy za sebou.",
        "Výstup více souborů z 'cat' můžete přesměrovat pomocí '>' do cílového souboru.",
        "Spusťte 'cat part1.txt part2.txt > full.txt'. Pořadí argumentů určuje pořadí v souboru.",
    ]
    start_directory = "concat"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("concat/part1.txt", "First part.\n"),
            FileFixture("concat/part2.txt", "Second part.\n"),
        ),
        clean=("concat/full.txt",),
    )
    completion = Completion(
        answer=ExactAnswer("full.txt"),
        requirements=(
            TextFileContent(
                "concat/full.txt",
                contains=("First part.", "Second part."),
                error_message="Soubor neobsahuje text z obou částí.",
                missing_message="Soubor neexistuje.",
            ),
        ),
    )


@section.level(4)
class EchoCreateFileLevel(Level):
    solution = Solution(steps=(RunShell("echo 'Ahoj svete' > pozdrav.txt"),), answer="pozdrav.txt")
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
        `shellgame submit pozdrav.txt`
        """
    hints = [
        "Příkaz 'echo' vypisuje zadaný text. Pomocí operátoru '>' můžete výstup přesměrovat do souboru.",
        "Text obsahující mezery uzavřete do uvozovek, aby se předal jako jeden argument.",
        "Spusťte 'echo \"Ahoj svete\" > pozdrav.txt' a odevzdejte 'pozdrav.txt'.",
    ]
    start_directory = "echo"
    fixture = WorkspaceFixture(clean=("echo/pozdrav.txt",), directories=("echo",))
    completion = Completion(
        answer=ExactAnswer("pozdrav.txt"),
        requirements=(
            TextFileContent(
                "echo/pozdrav.txt",
                exact="Ahoj svete",
                strip=True,
                error_message="Soubor neobsahuje přesně text 'Ahoj svete'.",
                missing_message="Soubor neexistuje.",
            ),
        ),
    )


@section.level(5)
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
        `shellgame submit <číslo>`
        """
    hints = [
        "Pipe (|) propojuje výstup prvního příkazu se vstupem druhého.",
        "grep najde řádky s 'ERROR', wc -l je spočítá. Spojte je pomocí |.",
        'Použijte: grep "ERROR" access.log | wc -l',
    ]
    start_directory = "pipes"
    fixture = WorkspaceFixture(files=(FileFixture("pipes/access.log", _ACCESS_LOG),))
    completion = Completion(
        answer=IntegerAnswer(
            7,
            mistakes={
                15: "Spočítali jste všechny řádky. Potřebujete jen ty s 'ERROR'. Použijte grep před wc.",
            },
        )
    )


@section.level(6)
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
        `shellgame submit <první>,<poslední>`
        """
    hints = [
        "head -n 1 zobrazí první řádek, tail -n 1 zobrazí poslední.",
        "Odevzdejte první slovo z každého z těchto dvou řádků.",
        "Formát odpovědi je `prvni,posledni` - dvě slova oddělená čárkou, bez mezery.",
    ]
    start_directory = "headtail"
    success_message = "Správně! Head a tail jsou skvělé pro rychlý náhled do souborů."
    fixture = WorkspaceFixture(files=(FileFixture("headtail/long_file.txt", _LONG_FILE),))
    completion = Completion(
        answer=TupleAnswer(
            (
                ExactAnswer(
                    "START",
                    error_message="První slovo není správně. Použijte 'head -n 1 long_file.txt'.",
                ),
                ExactAnswer(
                    "END",
                    error_message="Poslední slovo není správně. Použijte 'tail -n 1 long_file.txt'.",
                ),
            ),
            format_message="Formát: první_slovo,poslední_slovo (např. AHOJ,SVET)",
            required_message="Zadejte odpověď ve formátu: první_slovo,poslední_slovo",
        )
    )


@section.level(7)
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
        `shellgame submit <řádky>,<slova>`
        """
    hints = [
        "wc -l počítá řádky, wc -w počítá slova.",
        "Spusťte oba příkazy na `article.txt` a zapište si obě čísla.",
        "Odevzdejte je v pořadí řádky,slova — bez mezery za čárkou.",
    ]
    start_directory = "wc"
    success_message = "Správně! Příkaz wc je nepostradatelný pro rychlou analýzu souborů."
    fixture = WorkspaceFixture(files=(FileFixture("wc/article.txt", _ARTICLE),))
    completion = Completion(
        answer=TupleAnswer(
            (
                IntegerAnswer(
                    5,
                    error_message="Počet řádků není správně. Použijte 'wc -l article.txt'.",
                    invalid_message="Obě hodnoty musí být čísla.",
                ),
                IntegerAnswer(
                    31,
                    error_message="Počet slov není správně. Použijte 'wc -w article.txt'.",
                    invalid_message="Obě hodnoty musí být čísla.",
                ),
            ),
            format_message="Formát: řádky,slova (např. 10,50)",
            required_message="Zadejte odpověď ve formátu: řádky,slova",
        )
    )


@section.level(8)
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
        `shellgame submit <číslo>`
        """
    hints = [
        "Příkaz uniq odstraní duplikáty, ale jen sousedící! Proto nejdřív sort.",
        "Řetězec: sort → uniq → wc -l spočítá unikátní řádky.",
        "Spusťte 'sort visitors.txt | uniq | wc -l' a odevzdejte číslo z výstupu.",
    ]
    extension = True
    start_directory = "sort"
    success_message = "Správně! Sort | uniq je klasická kombinace pro práci s daty."
    fixture = WorkspaceFixture(files=(FileFixture("sort/visitors.txt", _VISITORS),))
    completion = Completion(
        answer=IntegerAnswer(
            5,
            mistakes={
                8: "Spočítali jste všechny řádky, ne unikátní. Zkuste: sort visitors.txt | uniq | wc -l",
                3: "Možná jste spočítali jen duplikáty. Hledáme počet unikátních jmen.",
            },
        )
    )


@section.level(9)
class SectionSummaryChallengeLevel(Level):
    solution = Solution(
        steps=(
            RunShell('echo "Hello World" > message.txt'),
            RunShell('echo "Goodbye" >> message.txt'),
        ),
        answer="3",
    )
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
        "Nejprve vytvořte soubor přesměrováním '>' a další řádek přidejte přes append '>>'.",
        "Použijte 'echo \"Hello World\" > message.txt' a pak 'echo \"Goodbye\" >> message.txt'.",
        "Spočítejte všechny .txt soubory (včetně nového message.txt) příkazem 'ls *.txt | wc -l'.",
    ]
    start_directory = "challenge"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("challenge/sample1.txt", "sample"),
            FileFixture("challenge/sample2.txt", "sample"),
        ),
        clean=("challenge",),
    )
    completion = Completion(
        answer=IntegerAnswer(
            3,
            mistakes={
                2: "Spočítali jste jen sample1.txt a sample2.txt. Vytvořili jste message.txt?",
            },
        ),
        requirements=(
            TextFileContent(
                "challenge/message.txt",
                exact="Hello World\nGoodbye",
                strip=True,
                error_message=("message.txt musí obsahovat řádky 'Hello World' a 'Goodbye' v tomto pořadí."),
                missing_message=("Chybí message.txt. Vytvořte pomocí 'echo \"Hello World\" > message.txt'."),
            ),
        ),
    )
    success_message = "Skvělé! Dokončili jste Sekci 8. Přesměrování i pipes máte v malíku!"
