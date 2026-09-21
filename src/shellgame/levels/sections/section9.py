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

section = Section(9, root="level-9")


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
    title = "Sekce 9: Vstup, výstup a stav příkazů"
    instructions_file = "section9_intro.md"
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
        Po vytvoření souboru spusťte:
        `shellgame submit`
        (můžete také zadat: `shellgame submit seznam.txt`)
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
        allow_empty=True,
    )
    success_message = "Správně! Výstup příkazu nemusí skončit na obrazovce — dá se uložit a dál s ním pracovat."


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
        Po přidání textu spusťte:
        `shellgame submit`
        (můžete také zadat: `shellgame submit log.txt`)
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
                exact="Start logu\nZaznam 1\nKonec logu\n",
                error_message=(
                    "Soubor musí zachovat původní obsah a přidat nový řádek přesně na konec. "
                    "Zmizely-li původní řádky, použili jste jednoduchou šipku `>`, která soubor přepíše; "
                    "připojení zajistí až zdvojená šipka. Původní stav vrátí `shellgame reset`."
                ),
                missing_message="Soubor neexistuje.",
            ),
        ),
        allow_empty=True,
    )
    success_message = "Správně! Zdvojená šipka připojuje, takže předchozí obsah souboru zůstane zachovaný."


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
        Po vytvoření spojeného souboru spusťte:
        `shellgame submit`
        (můžete také zadat: `shellgame submit full.txt`)
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
                exact="First part.\nSecond part.\n",
                error_message=(
                    "Soubor musí obsahovat obě části přesně v zadaném pořadí. "
                    "Jsou-li obsahy na místě, ale prohozené, prohodili jste argumenty příkazu cat: "
                    "jejich pořadí určuje pořadí řádků ve výstupu."
                ),
                missing_message="Soubor neexistuje.",
            ),
        ),
        allow_empty=True,
    )
    success_message = "Správně! Příkaz cat čte soubory v pořadí argumentů a stejné pořadí má i výsledek."


@section.level(4)
class EchoCreateFileLevel(Level):
    solution = Solution(
        steps=(RunShell('echo "Ahoj svete" > "muj pozdrav.txt"'),),
        answer="muj pozdrav.txt",
    )
    title = "Uvozovky v textu i názvu"
    instructions = """\
        # Uvozovky v textu i názvu souboru

        Shell dělí příkaz podle mezer. Uvozovky proto chrání víceslovný text i název
        souboru s mezerami — každou část uzavřete zvlášť.

        ## Úkol
        Pomocí `echo` vytvořte soubor `muj pozdrav.txt` s jediným řádkem `Ahoj svete`.

        ## Příkaz
        - `echo "Ahoj svete" > "muj pozdrav.txt"`

        První dvojice uvozovek chrání text, druhá název výstupního souboru.

        ## Odevzdání
        Uvozovky potřebuje i název předaný příkazu `shellgame`:
        `shellgame submit "muj pozdrav.txt"`
        """
    hints = [
        "Mezery oddělují argumenty. Uvozovky udrží více slov pohromadě jako jeden text nebo jednu cestu.",
        "Uzavřete do uvozovek text za `echo` a zvlášť také název za `>`.",
        'Spusťte `echo "Ahoj svete" > "muj pozdrav.txt"` a název v uvozovkách také odevzdejte.',
    ]
    start_directory = "echo"
    fixture = WorkspaceFixture(
        clean=("echo/pozdrav.txt", "echo/muj pozdrav.txt"),
        directories=("echo",),
    )
    completion = Completion(
        answer=ExactAnswer(
            "muj pozdrav.txt",
            required_message='Odevzdejte název v uvozovkách: shellgame submit "muj pozdrav.txt"',
        ),
        requirements=(
            TextFileContent(
                "echo/muj pozdrav.txt",
                exact="Ahoj svete\n",
                error_message="Soubor nemá přesně požadovaný jeden řádek.",
                missing_message="Chybí soubor s požadovaným názvem obsahujícím mezeru.",
            ),
        ),
    )
    success_message = "Správně! Uvozovky drží pohromadě to, co by shell jinak rozdělil podle mezer."


@section.level(5)
class PipeGrepAndCountLevel(Level):
    title = "Propojení příkazů (Pipes)"
    instructions = """\
        # Propojení příkazů pomocí rour (pipes)

        Znak `|` (pipe/roura) pošle výstup jednoho příkazu jako vstup druhému.

        ## Stavební kameny
        - `grep "vzor" soubor` vypíše jen ty řádky souboru, které vzor obsahují.
          Vyhledávání se naplno věnuje Sekce 11; tady `grep` použijeme jako hotový filtr.
        - `wc -l` spočítá řádky, které dostane na vstupu.
        - `|` pošle výstup levého příkazu na vstup pravého.

        ## Úkol
        V aktuálním adresáři je soubor `access.log` s mnoha řádky.
        Spočítejte, kolik řádků obsahuje slovo "ERROR". Příkaz si složte sami ze tří dílů výše.

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
    success_message = "Správně! Roura spojí jednoduché příkazy v nástroj, jakým žádný z nich sám o sobě není."


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
                    mistakes={
                        31: (
                            "První číslo má být počet řádků, ne slov — vypadá to, že máte hodnoty prohozené. "
                            "Pořadí je řádky,slova."
                        ),
                    },
                    error_message="Počet řádků není správně. Použijte 'wc -l article.txt'.",
                    invalid_message="Obě hodnoty musí být čísla.",
                ),
                IntegerAnswer(
                    31,
                    mistakes={
                        5: "Druhé číslo má být počet slov, ne řádků. Ten už jste zapsali jako první hodnotu.",
                    },
                    error_message="Počet slov není správně. Použijte 'wc -w article.txt'.",
                    invalid_message="Obě hodnoty musí být čísla.",
                ),
            ),
            format_message="Formát: řádky,slova (např. 10,50) — na pořadí obou čísel záleží.",
            required_message="Zadejte odpověď ve formátu: řádky,slova",
        )
    )


@section.level(8)
class SortUniqCountUniqueLevel(Level):
    title = "Řazení a odstranění duplicit"
    instructions = """\
        # Sort a Uniq - řazení a deduplikace

        Příkazy `sort` a `uniq` jsou mocné nástroje pro zpracování textových dat.

        ## Stavební kameny
        - `sort soubor` vypíše řádky seřazeně, takže stejné hodnoty skončí vedle sebe.
        - `uniq` zahodí opakující se řádky, ale pozná jen ty **sousední**.
        - `wc -l` spočítá řádky, které dostane na vstupu.

        Pořadí proto není libovolné: rozmyslete si, co musí `uniq` dostat na vstup, aby fungoval.

        ## Úkol
        V souboru `visitors.txt` jsou jména návštěvníků (někteří přišli vícekrát).
        Zjistěte, kolik je UNIKÁTNÍCH návštěvníků. Příkazy si pospojujte rourami sami.

        ## Odevzdání
        Odevzdejte počet unikátních návštěvníků.
        `shellgame submit <číslo>`
        """
    hints = [
        "Příkaz uniq odstraní duplikáty, ale jen sousedící! Proto nejdřív sort.",
        "Řetězec: sort → uniq → wc -l spočítá unikátní řádky.",
        "Spusťte 'sort visitors.txt | uniq | wc -l' a odevzdejte číslo z výstupu.",
    ]
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
            RunShell("ls | wc -l"),
        ),
        answer="3",
    )
    title = "Výzva: Přesměrování a roury"
    instructions = """\
        ### Výzva: Přesměrování a roury

        ### Úkol
        V aktuálním adresáři:

        1. Vytvořte `message.txt` s prvním řádkem `Hello World`.
        2. Přidejte na konec druhý řádek `Goodbye`, aniž by první zmizel.
        3. Propojte výpis obsahu adresáře s počítáním řádků a zjistěte počet položek.

        Vystačíte si s `echo`, `>`, `>>`, `ls`, `|` a `wc -l`.
        Odevzdejte zjištěný počet položek.

        ### Odevzdání
        `shellgame submit <počet>`
        """
    hints = [
        "První přesměrování má soubor vytvořit, druhé musí zachovat jeho obsah. "
        "Výpis pak pošlete rourou do počítadla řádků.",
        'Soubor vytvoříte pomocí `echo "Hello World" > message.txt` a druhý řádek přidáte přes `>>`.',
        "Počet položek zjistíte příkazem `ls | wc -l`.",
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
    success_message = "Skvělé! Přesměrování i roury máte v malíku!"


@section.level(10)
class InteractiveCatInputLevel(Level):
    solution = Solution(
        steps=(RunShell("printf '%s\\n' 'První řádek' 'Druhý řádek' | cat > poznamka.txt"),),
        answer="poznamka.txt",
    )
    title = "Interaktivní vstup a EOF"
    instructions = """\
        # Interaktivní vstup a EOF

        Když spustíte `cat > soubor`, příkaz čte řádky z klávesnice a zapisuje je do souboru.
        Na prázdném řádku stiskněte **Ctrl+D**: terminál tím oznámí EOF (konec vstupu) a `cat`
        řádně skončí. **Ctrl+C** místo toho běžící příkaz přeruší (interrupt).

        ## Úkol
        Spusťte `cat > poznamka.txt` a zadejte přesně tyto dva řádky:

        ```text
        První řádek
        Druhý řádek
        ```

        Po druhém řádku stiskněte Enter a potom na prázdném řádku Ctrl+D.

        ## Odevzdání
        Po ukončení zápisu pomocí Ctrl+D spusťte:
        `shellgame submit`
        (můžete také zadat: `shellgame submit poznamka.txt`)
        """
    hints = [
        "`cat` bez názvu vstupního souboru čte standardní vstup; EOF mu oznámí, že už žádná data nepřijdou.",
        "Po `cat > poznamka.txt` napište oba řádky. Ctrl+D použijte až na novém prázdném řádku.",
        "Jestli jste použili Ctrl+C nebo udělali překlep, spusťte `shellgame reset` a zopakujte zápis s Ctrl+D.",
    ]
    start_directory = "stdin"
    fixture = WorkspaceFixture(
        directories=("stdin",),
        clean=("stdin/poznamka.txt",),
    )
    completion = Completion(
        answer=ExactAnswer(
            "poznamka.txt",
            required_message="Odevzdejte název souboru: shellgame submit poznamka.txt",
        ),
        requirements=(
            TextFileContent(
                "stdin/poznamka.txt",
                exact="První řádek\nDruhý řádek\n",
                error_message=(
                    "Soubor neobsahuje přesně oba zadané řádky. Častá příčina je Ctrl+C: "
                    "to příkaz přeruší uprostřed práce, takže text zůstane useknutý nebo vůbec nedojde na disk. "
                    "Vstup řádně ukončí až Ctrl+D na prázdném řádku. Začněte znovu po `shellgame reset`."
                ),
                missing_message=(
                    "Soubor chybí. Začněte příkazem `cat > poznamka.txt`; pokud jste ho přerušili pomocí Ctrl+C, "
                    "nemusel vzniknout vůbec."
                ),
            ),
        ),
        allow_empty=True,
    )
    success_message = "Správně! EOF ukončilo vstup a `cat` soubor uzavřel."


@section.level(11)
class CommandStatusLevel(Level):
    solution = Solution(
        steps=(
            RunShell('true && echo "stav: uspech" > status.txt'),
            RunShell('false || echo "stav: neuspech" >> status.txt'),
        ),
        answer="status.txt",
    )
    title = "Návratový kód: && a ||"
    instructions = """\
        # Návratový kód: `&&` a `||`

        Každý příkaz skončí návratovým kódem (exit code): **0 znamená úspěch**, nenulová
        hodnota neúspěch. V interaktivním Bash i Fish podle něj můžete spojovat příkazy:

        - `první && druhý` spustí druhý jen po úspěchu prvního,
        - `první || druhý` spustí druhý jen po neúspěchu prvního.

        Příkazy `true` a `false` vracejí právě stav 0 a nenulový stav.

        ## Úkol
        Spusťte postupně oba řetězce; zapíšou do souboru `status.txt` dva řádky:

        ```bash
        true  && echo "stav: uspech"   ___ status.txt
        false || echo "stav: neuspech" ___ status.txt
        ```

        Na místo `___` doplňte přesměrování `>` nebo `>>`. Rozhodněte se podle toho,
        že první řádek soubor zakládá a druhý se musí přidat za něj — výsledek má mít
        právě dva řádky v tomto pořadí.

        ## Odevzdání
        `shellgame submit status.txt`
        """
    hints = [
        "Návratový kód 0 značí úspěch, nenulový kód neúspěch. Operátory sledují právě tento stav.",
        (
            "Za `&&` pokračuje úspěšný příkaz; za `||` pokračuje neúspěšný. "
            "První řetězec soubor zakládá (stačí přepsání), druhý k němu jen přidává."
        ),
        'Spusťte `true && echo "stav: uspech" > status.txt` a potom `false || echo "stav: neuspech" >> status.txt`.',
    ]
    start_directory = "status"
    fixture = WorkspaceFixture(
        directories=("status",),
        clean=("status/status.txt",),
    )
    completion = Completion(
        answer=ExactAnswer(
            "status.txt",
            required_message="Odevzdejte název souboru: shellgame submit status.txt",
        ),
        requirements=(
            TextFileContent(
                "status/status.txt",
                exact="stav: uspech\nstav: neuspech\n",
                error_message=(
                    "status.txt musí obsahovat přesně oba řádky ve správném pořadí. "
                    "Zbyl-li jen jeden řádek, přepsalo druhé přesměrování to první."
                ),
                missing_message="Chybí status.txt. Spusťte oba zadané řetězce příkazů.",
            ),
        ),
    )
    success_message = "Výborně! Dokončili jste Sekci 9 a umíte reagovat na stav příkazu."
