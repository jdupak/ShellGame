from __future__ import annotations

import shutil
from pathlib import Path

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    CopyValidator,
    FileExistsValidator,
    MoveValidator,
    MultiValidator,
    StringValidator,
    ValidationResult,
)

section = Section()


@section.level
class SectionIntro(Level):
    title = "Sekce 6: Kopírování a přesouvání"
    instructions_file = "section6_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"

    def setup(self, workspace: Path) -> None:
        pass

    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


@section.level
class BackupImportantFileLevel(Level):
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
        `shellgame submit -f dulezite.bak`
        """
    hints = [
        "Příkaz cp má dva argumenty: odkud a kam kopírujete.",
        "Syntaxe je: cp zdrojový_soubor cílový_soubor",
        "Použijte 'cp dulezite.txt dulezite.bak'.",
    ]
    start_directory = "level-6/copying"
    require_answer = True
    validators = [
        StringValidator("dulezite.bak"),
        CopyValidator("level-6/copying/dulezite.txt", "level-6/copying/dulezite.bak"),
    ]

    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-6" / "copying"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "dulezite.txt").write_text("Very important data.")

        bak = level_dir / "dulezite.bak"
        if bak.exists():
            bak.unlink()


@section.level
class BackupProjectDirectoryLevel(Level):
    title = "Kopírování adresáře"
    instructions = """
        Pro kopírování adresářů musíte použít přepínač `-r` (recursive), aby se zkopíroval i jejich obsah.

        ## Úkol:
        Zkopírujte celý adresář `projekt` do nového adresáře `projekt_zaloha`.

        ## Příkazy:
        - `cp -r <zdroj> <cíl>`: Zkopíruje adresář

        ## Odevzdání:
        Odevzdejte název nového adresáře.
        `shellgame submit -f projekt_zaloha`
        """
    hints = ["Použijte 'cp -r projekt projekt_zaloha'.", "Bez -r to nepůjde."]
    start_directory = "level-6/copying"
    require_answer = True
    validators = [
        StringValidator("projekt_zaloha"),
        CopyValidator("level-6/copying/projekt", "level-6/copying/projekt_zaloha"),
    ]

    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-6" / "copying"
        level_dir.mkdir(parents=True, exist_ok=True)

        dst = level_dir / "projekt_zaloha"
        if dst.exists():
            shutil.rmtree(dst)

        project_dir = level_dir / "projekt"
        project_dir.mkdir(parents=True, exist_ok=True)
        (project_dir / "main.py").write_text("print('hello')")


@section.level
class RenameFileLevel(Level):
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
        `shellgame submit -f spravne_jmeno.txt`
        """
    hints = [
        "Použijte 'mv spatne_jmeno.txt spravne_jmeno.txt'.",
        "Příkaz mv slouží i k přejmenování.",
    ]
    start_directory = "level-6/moving"
    require_answer = True
    validators = [
        StringValidator("spravne_jmeno.txt"),
        MoveValidator("level-6/moving/spatne_jmeno.txt", "level-6/moving/spravne_jmeno.txt"),
    ]

    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-6" / "moving"
        level_dir.mkdir(parents=True, exist_ok=True)

        correct = level_dir / "spravne_jmeno.txt"
        if correct.exists():
            correct.unlink()

        (level_dir / "spatne_jmeno.txt").write_text("content")


@section.level
class MoveReportToDocumentsLevel(Level):
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
        `shellgame submit -f dokumenty`
        """
    hints = [
        "Použijte 'mv report.pdf dokumenty/'.",
        "Lomítko na konci není nutné, ale je dobrým zvykem.",
    ]
    start_directory = "level-6/moving"
    require_answer = True
    validators = [
        StringValidator("dokumenty"),
        MoveValidator("level-6/moving/report.pdf", "level-6/moving/dokumenty/report.pdf"),
    ]

    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-6" / "moving"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "dokumenty").mkdir(exist_ok=True)

        moved = level_dir / "dokumenty" / "report.pdf"
        if moved.exists():
            moved.unlink()

        (level_dir / "report.pdf").write_text("report data")


@section.level
class RenameDirectoryLevel(Level):
    title = "Přejmenování adresáře"
    instructions = """
        Stejně jako soubory, i adresáře se přejmenovávají pomocí `mv`.

        ## Úkol:
        Adresář `tmp_data` už není dočasný. Přejmenujte ho na `data`.

        ## Příkazy:
        - `mv <starý_adresář> <nový_adresář>`

        ## Odevzdání:
        Odevzdejte nový název adresáře.
        `shellgame submit -f data`
        """
    hints = ["Použijte 'mv tmp_data data'.", "Funguje to stejně jako u souborů."]
    extension = True
    start_directory = "level-6/renaming"
    require_answer = True
    validators = [
        StringValidator("data"),
        MoveValidator("level-6/renaming/tmp_data", "level-6/renaming/data"),
    ]

    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-6" / "renaming"
        level_dir.mkdir(parents=True, exist_ok=True)

        dst = level_dir / "data"
        if dst.exists():
            shutil.rmtree(dst)

        src = level_dir / "tmp_data"
        src.mkdir(exist_ok=True)
        (src / "file.txt").write_text("content")


@section.level
class OrganizeLogsLevel(Level):
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
        `shellgame submit -f logs`
        """
    hints = [
        "Použijte 'mv *.log logs/'.",
        "Hvězdička vybere všechny soubory končící na .log.",
    ]
    optional = True
    start_directory = "level-6/organize"
    require_answer = True
    validators = [
        StringValidator("logs"),
        MultiValidator(
            [
                MoveValidator("level-6/organize/app.log", "level-6/organize/logs/app.log"),
                MoveValidator("level-6/organize/error.log", "level-6/organize/logs/error.log"),
                FileExistsValidator("level-6/organize/other.txt", should_exist=True),
            ]
        ),
    ]

    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-6" / "organize"
        level_dir.mkdir(parents=True, exist_ok=True)

        logs_dir = level_dir / "logs"
        if logs_dir.exists():
            shutil.rmtree(logs_dir)

        logs_dir.mkdir(exist_ok=True)
        (level_dir / "app.log").write_text("log1")
        (level_dir / "error.log").write_text("log2")
        (level_dir / "other.txt").write_text("keep me")


@section.level
class FileOrganizerChallengeLevel(Level):
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
    require_answer = True
    expected_answer = "organizer"

    def setup(self, workspace: Path) -> None:
        test_dir = workspace / "level-6" / "final_test"

        if test_dir.exists():
            shutil.rmtree(test_dir)

        test_dir.mkdir(parents=True, exist_ok=True)
        (test_dir / "backup").mkdir()
        (test_dir / "logs").mkdir()

        (test_dir / "original.txt").write_text("Important data - do not delete!\n")
        (test_dir / "temp_data.csv").write_text("col1,col2\n1,2\n")
        (test_dir / "misplaced.log").write_text("Log entry\n")

    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:  # noqa: PLR0911
        ok, msg = super().validate(answer, state)
        if not ok:
            return ok, msg

        test_dir = state.workspace / "level-6" / "final_test"

        if not (test_dir / "original.txt").exists():
            return False, "Smazali jste original.txt! Měli jste ho zkopírovat, ne přesunout."
        if not (test_dir / "backup" / "original.txt").exists():
            return False, "Chybí kopie v backup/. Použijte 'cp original.txt backup/'."

        if (test_dir / "temp_data.csv").exists():
            return False, "temp_data.csv stále existuje. Přejmenujte ho na data.csv pomocí 'mv'."
        if not (test_dir / "data.csv").exists():
            return False, "Chybí data.csv. Přejmenujte temp_data.csv pomocí 'mv temp_data.csv data.csv'."

        if (test_dir / "misplaced.log").exists():
            return False, "misplaced.log stále v hlavní složce. Přesuňte do logs/ pomocí 'mv'."
        if not (test_dir / "logs" / "misplaced.log").exists():
            return False, "Chybí misplaced.log v logs/. Přesuňte pomocí 'mv misplaced.log logs/'."

        return True, "Výborně! Dokončili jste Sekci 6. Umíte kopírovat, přesouvat i přejmenovávat!"


def get_levels() -> list[Level]:
    return section.levels
