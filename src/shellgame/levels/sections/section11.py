from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    Completion,
    ExactAnswer,
    IntegerAnswer,
    PathsMatch,
    SuffixAnswer,
    TextFileContent,
    TupleAnswer,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import RunShell, Solution

section = Section(11, root="level-11")


@section.level(0)
class SectionIntro(Level):
    is_intro = True
    title = "Sekce 11: Vyhledávání"
    instructions_file = "section11_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"


@section.level(1)
class GrepConfigLineToFileLevel(Level):
    solution = Solution(
        steps=(RunShell("grep ACTIVE_PROFILE config.txt > profile.txt"),),
        answer="profile.txt",
    )
    title = "Hledání v souboru (grep)"
    instructions = """
        ### Cíl
        Najděte řádek obsahující `ACTIVE_PROFILE` v souboru `config.txt` a uložte ho do `profile.txt`.

        ### Příkazy
        - `grep "vzor" soubor` - hledá vzor v souboru
        - `grep "vzor" soubor > výstup` - uloží nalezené řádky do souboru

        ### Úkol
        1. Najděte řádek s `ACTIVE_PROFILE`
        2. Přesměrujte výsledek do `profile.txt`

        ### Odevzdání
        Odevzdejte název vytvořeného souboru:
        `shellgame submit <název-souboru>`
        """
    hints = [
        "Příkaz `grep` hledá zadaný vzor v souborech.",
        "Vyhledejte klíč 'ACTIVE_PROFILE' v souboru 'config.txt' a výstup přesměrujte pomocí `>`.",
        "Spusťte `grep ACTIVE_PROFILE config.txt > profile.txt` a výsledek ověřte pomocí `cat profile.txt`.",
    ]
    start_directory = "grep"
    fixture = WorkspaceFixture(
        clean=("grep/profile.txt", "grep/config.txt"),
        files=(
            FileFixture(
                "grep/config.txt",
                "user=admin\nhost=localhost\nport=8080\nACTIVE_PROFILE=production\ndebug=true\n",
            ),
        ),
    )
    completion = Completion(
        answer=ExactAnswer("profile.txt"),
        requirements=(
            TextFileContent(
                "grep/config.txt",
                contains=("ACTIVE_PROFILE=production",),
                error_message="Zdrojový soubor config.txt chybí. Obnovte level příkazem `shellgame reset`.",
                missing_message="Zdrojový soubor config.txt chybí. Obnovte level příkazem `shellgame reset`.",
                unreadable_message=(
                    "Zdrojový soubor config.txt nelze přečíst. Obnovte level příkazem `shellgame reset`."
                ),
            ),
            TextFileContent(
                "grep/profile.txt",
                exact="ACTIVE_PROFILE=production",
                strip=True,
                error_message="Soubor neobsahuje přesně hledaný řádek.",
                missing_message="Výstup profile.txt musí být soubor. Obnovte level příkazem `shellgame reset`.",
                unreadable_message="Soubor profile.txt nelze přečíst. Obnovte level příkazem `shellgame reset`.",
            ),
        ),
    )
    success_message = "Správně! `grep` vybere jen potřebné řádky a `>` je uloží pro další použití."


@section.level(2)
class RecursiveGrepFindFileLevel(Level):
    title = "Rekurzivní hledání"
    instructions = """
        ### Cíl
        Pomocí `grep -r` zjistěte, který soubor v adresáři `project` obsahuje text `SECRET_KEY`.

        ### Příkazy
        - `grep -r "vzor" adresář/` - rekurzivní hledání

        ### Úkol
        Najděte soubor obsahující `SECRET_KEY` a odevzdejte jeho relativní cestu.

        ### Odevzdání
        `shellgame submit <cesta>`
        """
    hints = [
        "Přepínač `-r` hledá rekurzivně ve všech podadresářích.",
        'Zkuste `grep -r "SECRET_KEY" project/`.',
        "grep -r vypíše cestu před dvojtečkou. Odevzdejte právě tu cestu.",
    ]
    start_directory = "recursive"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("recursive/project/README.md", "# Project\n"),
            FileFixture("recursive/project/src/main.py", "print('Hello')\n"),
            FileFixture(
                "recursive/project/config/settings.py",
                "SECRET_KEY = 'xyz'\nDEBUG = True\n",
            ),
        )
    )
    completion = Completion(
        answer=SuffixAnswer(
            "config/settings.py",
            error_message=(
                "To není správná odpověď. `grep -r` vypisuje před dvojtečkou celou relativní cestu — "
                "odevzdejte ji i s adresáři, ne jen samotný název souboru. Úvodní `./` nevadí."
            ),
        )
    )
    success_message = "Správně! S přepínačem -r prochází grep celý strom adresářů a u každého nálezu uvede cestu."


@section.level(3)
class CaseInsensitiveWarningCountLevel(Level):
    title = "Hledání bez ohledu na velikost písmen"
    instructions = """
        ### Cíl
        Najděte všechny řádky obsahující slovo `warning` bez ohledu na velikost písmen a spočítejte je.

        ### Příkazy
        - `grep -i "vzor" soubor` - ignoruje velikost písmen
        - `grep -i "vzor" soubor | wc -l` - spočítá nalezené řádky

        ### Úkol
        Spočítejte, kolik řádků v `messages.log` odpovídá `warning` (case-insensitive).

        ### Odevzdání
        `shellgame submit <číslo>`
        """
    hints = [
        "Bez `-i` najdete jen některé varianty. S `-i` najdete `warning`, `WARNING`, `Warning`…",
        'Zkuste `grep -i "warning" messages.log` a spočítejte vypsané řádky.',
        "Počítejte každý řádek, který grep -i vypíše — ne jen jednu velikost písmen.",
    ]
    start_directory = "case"
    fixture = WorkspaceFixture(
        files=(
            FileFixture(
                "case/messages.log",
                """2024-01-01 10:00:00 INFO: Server started
2024-01-01 10:05:00 WARNING: Memory usage high
2024-01-01 10:10:00 ERROR: Connection lost
2024-01-01 10:15:00 warning: Disk space low
2024-01-01 10:20:00 INFO: User logged in
2024-01-01 10:25:00 Warning: CPU temperature elevated
2024-01-01 10:30:00 DEBUG: Cache cleared
2024-01-01 10:35:00 WARNING: Network latency detected
2024-01-01 10:40:00 INFO: Backup completed
""",
            ),
        )
    )
    completion = Completion(
        answer=IntegerAnswer(
            4,
            mistakes={
                2: "Našli jste jen 'WARNING'. Použijte -i pro nalezení všech variant (warning, Warning...).",
                1: "Našli jste jen jednu variantu. Přepínač -i ignoruje velikost písmen.",
            },
            error_message="Počet není správně. Zkontrolujte, že hledáte bez ohledu na velikost písmen.",
            invalid_message="Odpověď musí být číslo.",
        )
    )
    success_message = "Správně! Přepínač -i je nepostradatelný pro robustní hledání."


@section.level(4)
class FindLostFilePathLevel(Level):
    title = "Hledání souborů (find)"
    instructions = """
        ### Cíl
        Pomocí `find` najděte soubor `lost_file.txt` někde uvnitř `messy_dir`.

        ### Příkazy
        - `find adresář -type f -name "název"` - hledá pouze soubory podle názvu

        ### Úkol
        Najděte `lost_file.txt` a odevzdejte celou relativní cestu.

        ### Odevzdání
        `shellgame submit <cesta>`
        """
    hints = [
        "Příkaz `find` prohledá všechny podadresáře automaticky.",
        'Pro hledání souborů použijte: `find kde_hledat -type f -name "co_hledat"`.',
        'Použijte: `find messy_dir -type f -name "lost_file.txt"`.',
    ]
    start_directory = "find"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("find/messy_dir/a/b/c/d/lost_file.txt"),
            FileFixture("find/messy_dir/other.txt"),
            FileFixture("find/messy_dir/a/junk.txt"),
        )
    )
    completion = Completion(
        answer=SuffixAnswer(
            "messy_dir/a/b/c/d/lost_file.txt",
            error_message=(
                "To není správná cesta. Odevzdejte ji přesně tak, jak ji vypsal `find` — "
                "včetně všech adresářů, ne jen název souboru. Úvodní `./` nevadí."
            ),
        )
    )
    success_message = "Správně! `find` hledá podle vlastností souboru, takže si poradí i s neznámým umístěním."


@section.level(5)
class FindPythonFilesToListLevel(Level):
    solution = Solution(
        steps=(RunShell('find src_code -type f -name "*.py" | sort > python_files.txt'),),
        answer="python_files.txt",
    )
    title = "Hledání podle přípony"
    instructions = """
        ### Cíl
        Najděte všechny běžné `.py` soubory v `src_code` a uložte jejich cesty do `python_files.txt`.
        Výstup seřaďte, aby měl vždy stejné pořadí.

        **Důležité:** Vzor musí být v uvozovkách: `"*.py"`.
        Bez uvozovek by se ho nejprve pokusil rozbalit shell. S uvozovkami dostane vzor doslova `find`.

        ### Stavební prvky
        - `find adresář -type f -name "*.py"` - hledá pouze soubory s příponou `.py`
        - `sort` - seřadí řádky
        - `>` - uloží výsledek

        ### Úkol
        Vytvořte `python_files.txt` obsahující přesně seřazené cesty nalezených souborů.

        ### Odevzdání
        `shellgame submit <název-souboru>`
        """
    hints = [
        "Omezte `find` na běžné soubory pomocí `-type f`; adresář s příponou `.py` se počítat nemá.",
        "Vzor dejte do uvozovek a výstup pošlete přes `sort` před uložením pomocí `>`.",
        'Použijte: `find src_code -type f -name "*.py" | sort > python_files.txt`.',
    ]
    start_directory = "extension"
    fixture = WorkspaceFixture(
        directories=("extension/src_code/archive.py",),
        files=(
            FileFixture("extension/src_code/main.py"),
            FileFixture("extension/src_code/utils.py"),
            FileFixture("extension/src_code/README.txt"),
            FileFixture("extension/src_code/data.csv"),
        ),
        clean=("extension/python_files.txt",),
    )
    completion = Completion(
        answer=ExactAnswer("python_files.txt"),
        requirements=(
            TextFileContent(
                "extension/python_files.txt",
                exact="src_code/main.py\nsrc_code/utils.py",
                strip=True,
                error_message=(
                    "Obsah souboru nesedí. Bez `-type f` se do výpisu dostane i adresář s příponou `.py`; "
                    "bez uvozovek kolem vzoru ho rozbalí shell ještě před spuštěním `find`. "
                    "Nezapomeňte také na `sort`."
                ),
                missing_message=(
                    "Soubor zatím neexistuje. Výstup příkazu uložte pomocí `>` do souboru zadaného v úkolu."
                ),
            ),
        ),
    )
    success_message = "Správně! Spojení `find`, `sort` a `>` dává výsledek, který vyjde pokaždé stejně."


@section.level(6)
class FinalChallengeLevel(Level):
    solution = Solution(
        steps=(
            RunShell('find . -type f -name "*.sh"'),
            RunShell('grep -r "SECRET" .'),
            RunShell("cp hidden/secret.sh found/secret.txt"),
            RunShell('find . -type f -name "*.sh" | wc -l'),
        ),
        answer="4,NINJA2024",
    )
    title = "Finální výzva"
    instructions = """
        ### 🏆 Finální výzva: Terminálový ninja

        Gratulujeme! Dostali jste se na konec ShellGame.
        Tato výzva kombinuje vyhledávání, filtrování, kopírování a počítání z posledních sekcí.

        ### Úkol
        V `level-11/final`:

        1. Najděte **všechny soubory** s příponou `.sh` rekurzivně pomocí `find`.
        2. Jeden z nich obsahuje tajný kód — najděte ho pomocí `grep`.
        3. Obsah skriptu s kódem zkopírujte do `found/secret.txt`.
        4. Počet `.sh` souborů zjistěte až po kopírování; díky příponě `.txt` zůstane stabilní.

        ### Struktura odpovědi
        `<počet_skriptů>,<tajný_kód>`

        ### Odevzdání
        `shellgame submit <počet>,<kód>`
        """
    hints = [
        "Pro hledání skriptů spojte `find` s podmínkami `-type f` a `-name`; obsah pak prohledejte pomocí `grep`.",
        'Skripty vypíše `find . -type f -name "*.sh"`. V nalezených souborech hledejte řádek se slovem `SECRET`.',
        'Kopii vytvořte jako `found/secret.txt` a počet zjistěte přes `find . -type f -name "*.sh" | wc -l`. ',
        "Odevzdejte počet a samotný kód.",
    ]
    start_directory = "final"
    fixture = WorkspaceFixture(
        directories=("final/found",),
        files=(
            FileFixture("final/run.sh", "#!/bin/bash\necho 'Running...'\n"),
            FileFixture("final/scripts/build.sh", "#!/bin/bash\nmake all\n"),
            FileFixture("final/scripts/test.sh", "#!/bin/bash\npytest\n"),
            FileFixture(
                "final/hidden/secret.sh",
                "#!/bin/bash\n# SECRET_CODE=NINJA2024\necho 'You found me!'\n",
            ),
            FileFixture("final/readme.txt", "Look for shell scripts!\n"),
            FileFixture("final/data.csv", "col1,col2\n"),
        ),
        clean=("final",),
    )
    completion = Completion(
        answer=TupleAnswer(
            (
                IntegerAnswer(
                    4,
                    error_message=("Počet .sh souborů není správně. Použijte 'find . -type f -name \"*.sh\"'."),
                    invalid_message="První hodnota musí být číslo.",
                ),
                ExactAnswer(
                    "NINJA2024",
                    case_sensitive=False,
                    error_message=("Tajný kód není správně. Hledejte 'SECRET' v obsahu skriptů pomocí 'grep'."),
                ),
            ),
            format_message="Formát: počet,kód (např. 5,SECRET123)",
        ),
        requirements=(
            PathsMatch(
                "final/hidden/secret.sh",
                "final/found/secret.txt",
                error_message="Do složky found/ zkopírujte skript s tajným kódem.",
                destination_error="Do složky found/ zkopírujte skript s tajným kódem.",
            ),
        ),
    )
    success_message = """
GRATULUJEME!

Úspěšně jste dokončili ShellGame!

Nyní ovládáte základy práce s terminálem:
✓ Navigace v souborovém systému
✓ Práce se soubory a adresáři
✓ Oprávnění a typy souborů
✓ Přesměrování a streamy
✓ Wildcards a vyhledávání

Jste připraveni na další dobrodružství v Linuxu!
""".strip()
