from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    Completion,
    ExactAnswer,
    FileExists,
    PathMoved,
    PathsMatch,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import RunShell, Solution

section = Section(6, root="level-6")


@section.level(0)
class SectionIntro(Level):
    is_intro = True
    title = "Sekce 6: Kopírování a přesouvání"
    instructions_file = "section6_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"


@section.level(1)
class BackupImportantFileLevel(Level):
    solution = Solution(steps=(RunShell("cp dulezite.txt dulezite.bak"),), answer="dulezite.bak")
    title = "Kopírování souboru"
    instructions = """
        Příkaz `cp` (copy) vytvoří kopii souboru.

        ### Proč zálohovat?
        Před úpravou důležitého souboru je dobré si udělat zálohu.
        Když něco pokazíte, máte se kam vrátit!

        V praxi uvidíte:
        - `cp config.yaml config.yaml.bak` (před úpravou konfigurace)
        - `cp report.docx report_v1.docx` (verzování dokumentů)

        ## Úkol:
        Vytvořte zálohu souboru `dulezite.txt`. Kopii pojmenujte `dulezite.bak`.

        ## Příkazy:
        - `cp <zdroj> <cíl>`: Zkopíruje zdroj do cíle

        ## Odevzdání:
        Odevzdejte název vytvořené kopie.
        `shellgame submit dulezite.bak`
        """
    hints = [
        "Příkaz cp má dva argumenty: odkud a kam kopírujete.",
        "Syntaxe je: cp zdrojový_soubor cílový_soubor",
        "Použijte 'cp dulezite.txt dulezite.bak'.",
    ]
    start_directory = "copying"
    fixture = WorkspaceFixture(
        files=(FileFixture("copying/dulezite.txt", "Very important data."),),
        clean=("copying/dulezite.bak",),
    )
    completion = Completion(
        answer=ExactAnswer("dulezite.bak"),
        requirements=(
            PathsMatch(
                "copying/dulezite.txt",
                "copying/dulezite.bak",
            ),
        ),
    )


@section.level(2)
class BackupProjectDirectoryLevel(Level):
    solution = Solution(steps=(RunShell("cp -r projekt projekt_zaloha"),), answer="projekt_zaloha")
    title = "Kopírování adresáře"
    instructions = """
        Pro kopírování adresářů musíte použít přepínač `-r` (recursive), aby se zkopíroval i jejich obsah.

        ## Úkol:
        Zkopírujte celý adresář `projekt` do nového adresáře `projekt_zaloha`.

        ## Příkazy:
        - `cp -r <zdroj> <cíl>`: Zkopíruje adresář

        ## Odevzdání:
        Odevzdejte název nového adresáře.
        `shellgame submit projekt_zaloha`
        """
    hints = [
        "Pro kopírování celého adresáře včetně obsahu je nutné použít rekurzivní přepínač '-r'.",
        "Spusťte 'cp -r projekt projekt_zaloha'.",
    ]
    start_directory = "copying"
    fixture = WorkspaceFixture(
        files=(FileFixture("copying/projekt/main.py", "print('hello')"),),
        clean=("copying/projekt_zaloha",),
    )
    completion = Completion(
        answer=ExactAnswer("projekt_zaloha"),
        requirements=(
            PathsMatch(
                "copying/projekt",
                "copying/projekt_zaloha",
            ),
        ),
    )


@section.level(3)
class RenameFileLevel(Level):
    solution = Solution(steps=(RunShell("mv spatne_jmeno.txt spravne_jmeno.txt"),), answer="spravne_jmeno.txt")
    title = "Přejmenování souboru"
    instructions = """
        Příkaz `mv` (move) se používá k přesouvání,
        ale pokud přesouváte soubor ve stejném adresáři na nové jméno, jde o přejmenování.

        ## Úkol:
        Soubor `spatne_jmeno.txt` má překlep. Přejmenujte ho na `spravne_jmeno.txt`.

        ## Příkazy:
        - `mv <staré_jméno> <nové_jméno>`: Přejmenuje soubor

        ## Odevzdání:
        Odevzdejte nový název souboru.
        `shellgame submit spravne_jmeno.txt`
        """
    hints = [
        "Příkaz 'mv' slouží nejen k přesunu, ale i k přejmenování souboru: 'mv staré nové'.",
        "Spusťte 'mv spatne_jmeno.txt spravne_jmeno.txt'.",
    ]
    start_directory = "moving"
    fixture = WorkspaceFixture(
        files=(FileFixture("moving/spatne_jmeno.txt", "content"),),
        clean=("moving/spravne_jmeno.txt",),
    )
    completion = Completion(
        answer=ExactAnswer("spravne_jmeno.txt"),
        requirements=(
            PathMoved(
                "moving/spatne_jmeno.txt",
                "moving/spravne_jmeno.txt",
            ),
        ),
    )


@section.level(4)
class MoveReportToDocumentsLevel(Level):
    solution = Solution(steps=(RunShell("mv report.pdf dokumenty/"),), answer="dokumenty")
    title = "Přesun souboru"
    instructions = """
        Pokud jako cíl příkazu `mv` uvedete existující adresář,
        soubor se do něj přesune (a zachová si své jméno, pokud neuvedete jiné).

        ## Úkol:
        Přesuňte soubor `report.pdf` do adresáře `dokumenty`.

        ## Příkazy:
        - `mv <soubor> <adresář>/`: Přesune soubor do adresáře

        ## Odevzdání:
        Odevzdejte název adresáře, kam jste soubor přesunuli.
        `shellgame submit dokumenty`
        """
    hints = [
        "Syntaxe pro přesun do adresáře: 'mv <soubor> <cílový_adresář>/'.",
        "Spusťte 'mv report.pdf dokumenty/'. Lomítko na konci značí adresář.",
    ]
    start_directory = "moving"
    fixture = WorkspaceFixture(
        directories=("moving/dokumenty",),
        files=(FileFixture("moving/report.pdf", "report data"),),
        clean=("moving/dokumenty/report.pdf",),
    )
    completion = Completion(
        answer=ExactAnswer("dokumenty"),
        requirements=(
            PathMoved(
                "moving/report.pdf",
                "moving/dokumenty/report.pdf",
            ),
        ),
    )


@section.level(5)
class RenameDirectoryLevel(Level):
    solution = Solution(steps=(RunShell("mv tmp_data data"),), answer="data")
    title = "Přejmenování adresáře"
    instructions = """
        Stejně jako soubory, i adresáře se přejmenovávají pomocí `mv`.

        ## Úkol:
        Adresář `tmp_data` už není dočasný. Přejmenujte ho na `data`.

        ## Příkazy:
        - `mv <starý_adresář> <nový_adresář>`

        ## Odevzdání:
        Odevzdejte nový název adresáře.
        `shellgame submit data`
        """
    hints = [
        "Přejmenování adresáře funguje stejně jako u souborů: 'mv <starý_název> <nový_název>'.",
        "Spusťte 'mv tmp_data data'.",
    ]
    extension = True
    start_directory = "renaming"
    fixture = WorkspaceFixture(
        files=(FileFixture("renaming/tmp_data/file.txt", "content"),),
        clean=("renaming/data",),
    )
    completion = Completion(
        answer=ExactAnswer("data"),
        requirements=(
            PathMoved(
                "renaming/tmp_data",
                "renaming/data",
            ),
        ),
    )


@section.level(6)
class OrganizeLogsLevel(Level):
    solution = Solution(steps=(RunShell("mv app.log error.log logs/"),), answer="logs")
    title = "Úklid logů"
    instructions = """
        V adresáři je nepořádek. Všechny soubory s příponou `.log` by měly být v adresáři `logs`.

        ## Úkol:
        Přesuňte všechny `.log` soubory (`app.log`, `error.log`) do adresáře `logs`.
        Můžete to udělat jedním příkazem pomocí hvězdičky.

        ## Příkazy:
        - `mv *.log logs/`

        ## Odevzdání:
        Odevzdejte název adresáře, kam jste soubory přesunuli.
        `shellgame submit logs`
        """
    hints = [
        "Žolík '*' nahradí libovolný počet znaků, takže vzor '*.log' vybere všechny soubory s touto příponou.",
        "Příkaz 'mv' dokáže přesunout více souborů najednou do cílového adresáře: 'mv <soubory> <cíl>/'.",
        "Spusťte 'mv *.log logs/' a odevzdejte 'logs'.",
    ]
    optional = True
    start_directory = "organize"
    fixture = WorkspaceFixture(
        directories=("organize/logs",),
        files=(
            FileFixture("organize/app.log", "log1"),
            FileFixture("organize/error.log", "log2"),
            FileFixture("organize/other.txt", "keep me"),
        ),
        clean=("organize/logs",),
    )
    completion = Completion(
        answer=ExactAnswer("logs"),
        requirements=(
            PathMoved(
                "organize/app.log",
                "organize/logs/app.log",
            ),
            PathMoved(
                "organize/error.log",
                "organize/logs/error.log",
            ),
            FileExists("organize/other.txt"),
        ),
    )


@section.level(7)
class FileOrganizerChallengeLevel(Level):
    solution = Solution(
        steps=(RunShell("cp original.txt backup/ && mv temp_data.csv data.csv && mv misplaced.log logs/"),),
        answer="organizer",
    )
    title = "Souhrn Sekce 6"
    instructions = """
        ### Výzva: Organizátor souborů

        Ukažte, že ovládáte kopírování a přesouvání!

        ### Úkol
        V `level-6/final_test`:

        1. **Zkopírujte** `original.txt` do složky `backup/` (zachovejte originál)
        2. **Přejmenujte** `temp_data.csv` na `data.csv`
        3. **Přesuňte** `misplaced.log` do složky `logs/`

        Pak odevzdejte heslo: **organizer**

        ### Shrnutí příkazů Sekce 6
        ```
        cp zdroj cíl       → Kopíruje soubor
        cp zdroj dir/      → Kopíruje do adresáře
        cp -r dir1 dir2    → Kopíruje celý adresář
        mv zdroj cíl       → Přesune/přejmenuje
        mv soubor dir/     → Přesune do adresáře
        ```

        ### Odevzdání
        `shellgame submit organizer`
        """
    hints = [
        "Kopírování: 'cp original.txt backup/'. Přejmenování: 'mv temp_data.csv data.csv'.",
        "Přesun do složky: 'mv misplaced.log logs/'.",
        (
            "Zkontrolujte: 'ls backup/' (je tam original.txt?), "
            "'ls' (je tam data.csv?), 'ls logs/' (je tam misplaced.log?)."
        ),
    ]
    start_directory = "final_test"
    fixture = WorkspaceFixture(
        directories=("final_test/backup", "final_test/logs"),
        files=(
            FileFixture(
                "final_test/original.txt",
                "Important data - do not delete!\n",
            ),
            FileFixture("final_test/temp_data.csv", "col1,col2\n1,2\n"),
            FileFixture("final_test/misplaced.log", "Log entry\n"),
        ),
        clean=("final_test",),
    )
    completion = Completion(
        answer=ExactAnswer("organizer"),
        requirements=(
            FileExists(
                "final_test/original.txt",
                error_message="Smazali jste original.txt! Měli jste ho zkopírovat, ne přesunout.",
            ),
            PathsMatch(
                "final_test/original.txt",
                "final_test/backup/original.txt",
                error_message="Kopie v backup/ neodpovídá originálu.",
                destination_error="Chybí kopie v backup/. Použijte 'cp original.txt backup/'.",
            ),
            PathMoved(
                "final_test/temp_data.csv",
                "final_test/data.csv",
                source_error="temp_data.csv stále existuje. Přejmenujte ho na data.csv pomocí 'mv'.",
                destination_error="Chybí data.csv. Přejmenujte temp_data.csv pomocí 'mv temp_data.csv data.csv'.",
            ),
            PathMoved(
                "final_test/misplaced.log",
                "final_test/logs/misplaced.log",
                source_error="misplaced.log stále v hlavní složce. Přesuňte do logs/ pomocí 'mv'.",
                destination_error="Chybí misplaced.log v logs/. Přesuňte pomocí 'mv misplaced.log logs/'.",
            ),
        ),
    )
    success_message = "Výborně! Dokončili jste Sekci 6. Umíte kopírovat, přesouvat i přejmenovávat!"
