from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    Completion,
    ExactAnswer,
    FileExists,
    PathMoved,
    PathsMatch,
    TextFileContent,
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
        answer=ExactAnswer(
            "dulezite.bak",
            mistakes={
                "dulezite.txt": (
                    "'dulezite.txt' je zdroj, ne vytvořená kopie. "
                    "Příkaz cp bere nejdřív zdroj a až potom cíl - odevzdejte jméno cíle."
                ),
            },
        ),
        requirements=(
            PathsMatch(
                "copying/dulezite.txt",
                "copying/dulezite.bak",
            ),
        ),
    )
    success_message = "Správně! Záloha je samostatná kopie pod novým jménem, originál zůstává nedotčený."


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
        "Tvar je 'cp -r <zdroj> <cíl>': přepínač stojí před oběma argumenty, "
        "zdrojový adresář je první a název nové kopie druhý.",
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
    success_message = "Správně! Přepínač -r říká příkazu cp, že má sestoupit do celého stromu adresáře."


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
        "Oba argumenty jsou jména souborů v aktuálním adresáři: první je to současné, druhé to požadované.",
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
    success_message = "Správně! Přejmenování je jen přesun na novou cestu ve stejném adresáři."


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

        ## Proč to lomítko na konci?
        Pokud cílový adresář **neexistuje**, `mv` soubor tiše přejmenuje na zadané jméno.
        S lomítkem `mv` místo toho ohlásí chybu, takže se o překlepu v názvu adresáře dozvíte hned.

        ## Odevzdání:
        Odevzdejte název adresáře, kam jste soubor přesunuli.
        `shellgame submit dokumenty`
        """
    hints = [
        "O tom, co 'mv' udělá, rozhoduje podoba cíle: soubor znamená přejmenování, existující adresář přesun.",
        "Tvar je 'mv <soubor> <cílový_adresář>/': první argument je přesouvaný soubor, "
        "druhý cílový adresář zakončený lomítkem.",
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
                source_error=(
                    "report.pdf je stále na původním místě. Kopie originál ponechává, přesun ho odtud odebere."
                ),
                destination_error=(
                    "report.pdf z původního místa zmizel, ale v cílovém adresáři není. "
                    "Když cíl neexistuje, 'mv' soubor jen přejmenuje - zkontrolujte 'ls'."
                ),
            ),
        ),
    )
    success_message = "Správně! Podoba cíle rozhoduje, jestli mv přejmenovává, nebo přesouvá."


@section.level(5)
class MoveDirectoryLevel(Level):
    solution = Solution(steps=(RunShell("mv projekt archiv/"),), answer=None)
    title = "Přesun adresáře"
    instructions = """
        Příkaz `mv` přesouvá celé adresáře včetně jejich obsahu; přepínač `-r` nepotřebuje.

        ## Úkol:
        Přesuňte celý adresář `projekt` do existujícího adresáře `archiv`.

        ## Příkazy:
        - `mv <adresář> <cílový_adresář>/`

        ## Odevzdání:
        Až bude `projekt` uvnitř `archiv`, spusťte:
        `shellgame submit`
        """
    hints = [
        "Adresář se přesouvá stejným příkazem jako soubor: 'mv' vezme celý podstrom a přepínač '-r' nepotřebuje.",
        "Tvar je 'mv <adresář> <cílový_adresář>/': první argument je přesouvaný adresář, "
        "druhý už existující rodič, do kterého má vklouznout.",
        "Spusťte 'mv projekt archiv/' a ověřte výsledek pomocí 'ls archiv/'.",
    ]
    start_directory = "moving"
    fixture = WorkspaceFixture(
        directories=("moving/archiv",),
        files=(FileFixture("moving/projekt/data/file.txt", "content"),),
        clean=("moving/projekt", "moving/archiv/projekt"),
    )
    completion = Completion(
        requirements=(
            PathMoved(
                "moving/projekt",
                "moving/archiv/projekt",
                source_error="Adresář projekt je stále na původním místě - zatím se nic nepřesunulo.",
                destination_error=(
                    "Projekt z původního místa zmizel, ale v archivu není. "
                    "Když cílový adresář neexistuje, 'mv' zdroj jen přejmenuje - porovnejte 'ls' a 'ls archiv/'."
                ),
            ),
            TextFileContent(
                "moving/archiv/projekt/data/file.txt",
                exact="content",
                error_message="Přesunutý adresář nemá původní obsah.",
            ),
        ),
    )
    success_message = "Správně! Přesun adresáře vezme celý jeho obsah s sebou jedním krokem."


@section.level(6)
class OrganizeLogsLevel(Level):
    solution = Solution(steps=(RunShell("mv app.log error.log logs/"),), answer="logs")
    title = "Úklid logů"
    instructions = """
        V adresáři je nepořádek. Všechny soubory s příponou `.log` by měly být v adresáři `logs`.

        ## Úkol:
        Přesuňte všechny `.log` soubory (`app.log`, `error.log`) do adresáře `logs`.
        Zvládnete to jedním příkazem, když místo výčtu jmen zadáte vzor se žolíkem `*`.

        ## Příkazy:
        - `mv VZOR logs/`: za `VZOR` dosaďte vzor, který vybere právě `.log` soubory

        Vzor si nejdřív ověřte pomocí `ls VZOR` - uvidíte přesně ty soubory, které by `mv` přesunul.
        Znak `*` je žolík (zástupný znak); celou rodinu žolíků probereme hned v následující Sekci 7.

        ## Odevzdání:
        Odevzdejte název adresáře, kam jste soubory přesunuli.
        `shellgame submit logs`
        """
    hints = [
        "Žolík '*' nahradí libovolný počet znaků, takže vzor '*.log' vybere všechny soubory s touto příponou.",
        "Příkaz 'mv' dokáže přesunout více souborů najednou do cílového adresáře: 'mv <soubory> <cíl>/'.",
        "Spusťte 'mv *.log logs/' a odevzdejte 'logs'.",
    ]
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
    success_message = "Správně! Vzor se žolíkem rozbalí shell na seznam jmen ještě před spuštěním příkazu."


@section.level(7)
class ShellgameAliasLevel(Level):
    title = "Rychlejší práce: alias"
    instructions = """
        ### Cíl
        Zkraťte si časté psaní příkazu `shellgame` pomocí aliasu `sg`.

        ### Nastavení
        Použijte příkaz pro svůj shell:
        - **Bash:** `alias sg='shellgame'`
        - **Fish:** `alias sg shellgame`

        Alias ověřte příkazem `type sg`.
        Platí jen v aktuálním herním shellu; po jeho ukončení zmizí.

        ### Odevzdání
        Dokončete level přes vytvořený alias:
        `sg submit alias-ready`

        V dalších levelech můžete místo dlouhého `shellgame` používat kratší `sg`.
        Samotné použití `sg` je součást tohoto cvičení.
        """
    hints = [
        "Alias dává dlouhému příkazu kratší jméno. Vyberte syntaxi pro svůj shell.",
        "Po nastavení spusťte 'type sg' a ověřte, že sg odkazuje na shellgame.",
        "Level dokončete přes alias příkazem 'sg submit alias-ready'.",
    ]
    completion = Completion(
        answer=ExactAnswer(
            "alias-ready",
            required_message="Nejprve nastavte a ověřte alias, potom spusťte: sg submit alias-ready",
        )
    )
    success_message = "Výborně! Alias je jen krátké jméno pro delší příkaz — v závěrečné výzvě se vám bude hodit."


@section.level(8)
class FileOrganizerChallengeLevel(Level):
    solution = Solution(
        steps=(RunShell("cp original.txt backup/ && mv temp_data.csv data.csv && mv misplaced.log logs/"),),
        answer=None,
    )
    title = "Výzva: Organizátor souborů"
    instructions = """
        Ukažte, že ovládáte kopírování a přesouvání!

        ### Úkol
        V aktuálním adresáři:

        1. **Zkopírujte** `original.txt` do složky `backup/` (zachovejte originál)
        2. **Přejmenujte** `temp_data.csv` na `data.csv`
        3. **Přesuňte** `misplaced.log` do složky `logs/`

        ### Přehled příkazů
        ```
        cp zdroj cíl       → Kopíruje soubor
        cp zdroj dir/      → Kopíruje do adresáře
        cp -r dir1 dir2    → Kopíruje celý adresář
        mv zdroj cíl       → Přesune/přejmenuje
        mv soubor dir/     → Přesune do adresáře
        ```

        ### Odevzdání
        Po splnění všech bodů spusťte `shellgame submit`.
        """
    hints = [
        "Rozhodněte u každého souboru, zda musí původní cesta zůstat zachovaná. Podle toho zvolte 'cp' nebo 'mv'.",
        "Přejmenování i přesun používají 'mv'; rozdíl určuje podoba cíle. Kopie do adresáře zachová původní soubor.",
        "Použijte 'cp original.txt backup/', 'mv temp_data.csv data.csv' a 'mv misplaced.log logs/'.",
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
        requirements=(
            FileExists(
                "final_test/original.txt",
                error_message="Smazali jste original.txt! Měli jste ho zkopírovat, ne přesunout.",
            ),
            PathsMatch(
                "final_test/original.txt",
                "final_test/backup/original.txt",
                error_message="Kopie v backup/ neodpovídá originálu.",
                destination_error="Chybí kopie original.txt v adresáři backup/.",
            ),
            PathMoved(
                "final_test/temp_data.csv",
                "final_test/data.csv",
                source_error="temp_data.csv stále existuje. Přejmenujte ho na data.csv pomocí 'mv'.",
                destination_error="Chybí data.csv. Zkontrolujte, zda jste soubor přejmenovali místo kopírování.",
            ),
            PathMoved(
                "final_test/misplaced.log",
                "final_test/logs/misplaced.log",
                source_error="misplaced.log stále v hlavní složce. Přesuňte do logs/ pomocí 'mv'.",
                destination_error="Chybí misplaced.log v adresáři logs/.",
            ),
        ),
    )
    success_message = "Výborně! Dokončili jste Sekci 6 — kopírování, přesun i přejmenování máte pod kontrolou."
