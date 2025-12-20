"""Section 6: Copy and Move."""

from typing import Tuple, Any, Optional
from pathlib import Path
from shellgame.levels.base import Level
from shellgame.validation.validators import (
    StringValidator,
    CopyValidator,
    MoveValidator,
    MultiValidator,
    FileExistsValidator,
)


class Level6_0(Level):
    """Level 6.0: Section 6 Introduction."""

    def __init__(self) -> None:
        super().__init__(
            id="6.0",
            section=6,
            title="Sekce 6: Kopírování a přesouvání",
            instructions_file="section6_intro.md",
            hints=["Přečtěte si úvod a pokračujte stisknutím Enter."],
        )

    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        pass

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Always valid."""
        return True, "Jdeme na to!"


class Level6_1(Level):
    """Level 6.1: Simple File Copy."""

    def __init__(self) -> None:
        super().__init__(
            id="6.1",
            section=6,
            title="Kopírování souboru",
            instructions="""# Level 6.1: Kopírování souboru

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
`shellgame submit -f dulezite.bak`""",
            hints=[
                "Příkaz cp má dva argumenty: odkud a kam kopírujete.",
                "Syntaxe je: cp zdrojový_soubor cílový_soubor",
                "Použijte 'cp dulezite.txt dulezite.bak'.",
            ],
            start_directory="level-6/copying",
        )

    def setup(self, workspace: Path) -> None:
        """Create source file."""
        level_dir = workspace / "level-6" / "copying"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Clean up previous attempts
        if (level_dir / "dulezite.bak").exists():
            (level_dir / "dulezite.bak").unlink()

        (level_dir / "dulezite.txt").write_text("Very important data.")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate copy."""
        if answer is None:
            return False, "Musíte zadat název kopie."

        str_val = StringValidator("dulezite.bak")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        copy_val = CopyValidator("level-6/copying/dulezite.txt", "level-6/copying/dulezite.bak")
        return copy_val.validate(answer, state.workspace)


class Level6_2(Level):
    """Level 6.2: Directory Copy."""

    def __init__(self) -> None:
        super().__init__(
            id="6.2",
            section=6,
            title="Kopírování adresáře",
            instructions="""# Level 6.2: Kopírování adresáře

Pro kopírování adresářů musíte použít přepínač `-r` (recursive), aby se zkopíroval i jejich obsah.

## Úkol:
Zkopírujte celý adresář `projekt` do nového adresáře `projekt_zaloha`.

## Příkazy:
- `cp -r <zdroj> <cíl>`: Zkopíruje adresář

## Odevzdání:
Odevzdejte název nového adresáře.
`shellgame submit -f projekt_zaloha`""",
            hints=["Použijte 'cp -r projekt projekt_zaloha'.", "Bez -r to nepůjde."],
            start_directory="level-6/copying",
        )

    def setup(self, workspace: Path) -> None:
        """Create source directory."""
        level_dir = workspace / "level-6" / "copying"

        # Clean up previous attempts
        import shutil

        if (level_dir / "projekt_zaloha").exists():
            shutil.rmtree(level_dir / "projekt_zaloha")

        project_dir = level_dir / "projekt"
        project_dir.mkdir(parents=True, exist_ok=True)
        (project_dir / "main.py").write_text("print('hello')")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate directory copy."""
        if answer is None:
            return False, "Musíte zadat název adresáře."

        str_val = StringValidator("projekt_zaloha")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        # Check if destination exists and is a directory
        # CopyValidator handles basic existence, but let's be specific
        copy_val = CopyValidator("level-6/copying/projekt", "level-6/copying/projekt_zaloha")
        return copy_val.validate(answer, state.workspace)


class Level6_3(Level):
    """Level 6.3: File Rename."""

    def __init__(self) -> None:
        super().__init__(
            id="6.3",
            section=6,
            title="Přejmenování souboru",
            instructions="""# Level 6.3: Přejmenování souboru

Příkaz `mv` (move) se používá k přesouvání, ale pokud přesouváte soubor ve stejném adresáři na nové jméno, jde o přejmenování.

## Úkol:
Soubor `spatne_jmeno.txt` má překlep. Přejmenujte ho na `spravne_jmeno.txt`.

## Příkazy:
- `mv <staré_jméno> <nové_jméno>`: Přejmenuje soubor

## Odevzdání:
Odevzdejte nový název souboru.
`shellgame submit -f spravne_jmeno.txt`""",
            hints=[
                "Použijte 'mv spatne_jmeno.txt spravne_jmeno.txt'.",
                "Příkaz mv slouží i k přejmenování.",
            ],
            start_directory="level-6/moving",
        )

    def setup(self, workspace: Path) -> None:
        """Create file with wrong name."""
        level_dir = workspace / "level-6" / "moving"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Reset state
        if (level_dir / "spravne_jmeno.txt").exists():
            (level_dir / "spravne_jmeno.txt").unlink()

        (level_dir / "spatne_jmeno.txt").write_text("content")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate rename."""
        if answer is None:
            return False, "Musíte zadat nový název."

        str_val = StringValidator("spravne_jmeno.txt")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        move_val = MoveValidator(
            "level-6/moving/spatne_jmeno.txt", "level-6/moving/spravne_jmeno.txt"
        )
        return move_val.validate(answer, state.workspace)


class Level6_4(Level):
    """Level 6.4: File Move."""

    def __init__(self) -> None:
        super().__init__(
            id="6.4",
            section=6,
            title="Přesun souboru",
            instructions="""# Level 6.4: Přesun souboru

Pokud jako cíl příkazu `mv` uvedete existující adresář, soubor se do něj přesune (a zachová si své jméno, pokud neuvedete jiné).

## Úkol:
Přesuňte soubor `report.pdf` do adresáře `dokumenty`.

## Příkazy:
- `mv <soubor> <adresář>/`: Přesune soubor do adresáře

## Odevzdání:
Odevzdejte název adresáře, kam jste soubor přesunuli.
`shellgame submit -f dokumenty`""",
            hints=[
                "Použijte 'mv report.pdf dokumenty/'.",
                "Lomítko na konci není nutné, ale je dobrým zvykem.",
            ],
            start_directory="level-6/moving",
        )

    def setup(self, workspace: Path) -> None:
        """Create file and destination."""
        level_dir = workspace / "level-6" / "moving"

        # Reset state
        if (level_dir / "dokumenty" / "report.pdf").exists():
            (level_dir / "dokumenty" / "report.pdf").unlink()

        (level_dir / "dokumenty").mkdir(exist_ok=True)
        (level_dir / "report.pdf").write_text("report data")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate move."""
        if answer is None:
            return False, "Musíte zadat cíl."

        str_val = StringValidator("dokumenty")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        move_val = MoveValidator("level-6/moving/report.pdf", "level-6/moving/dokumenty/report.pdf")
        return move_val.validate(answer, state.workspace)


class Level6_5(Level):
    """Level 6.5: Directory Rename."""

    def __init__(self) -> None:
        super().__init__(
            id="6.5",
            section=6,
            title="Přejmenování adresáře",
            instructions="""# Level 6.5: Přejmenování adresáře

Stejně jako soubory, i adresáře se přejmenovávají pomocí `mv`.

## Úkol:
Adresář `tmp_data` už není dočasný. Přejmenujte ho na `data`.

## Příkazy:
- `mv <starý_adresář> <nový_adresář>`

## Odevzdání:
Odevzdejte nový název adresáře.
`shellgame submit -f data`""",
            hints=["Použijte 'mv tmp_data data'.", "Funguje to stejně jako u souborů."],
            extension=True,
            start_directory="level-6/renaming",
        )

    def setup(self, workspace: Path) -> None:
        """Create directory to rename."""
        level_dir = workspace / "level-6" / "renaming"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Reset state
        import shutil

        if (level_dir / "data").exists():
            shutil.rmtree(level_dir / "data")

        (level_dir / "tmp_data").mkdir(exist_ok=True)
        (level_dir / "tmp_data" / "file.txt").write_text("content")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate rename."""
        if answer is None:
            return False, "Musíte zadat nový název."

        str_val = StringValidator("data")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        move_val = MoveValidator("level-6/renaming/tmp_data", "level-6/renaming/data")
        return move_val.validate(answer, state.workspace)


class Level6_6(Level):
    """Level 6.6: Multi-File Move."""

    def __init__(self) -> None:
        super().__init__(
            id="6.6",
            section=6,
            title="Úklid logů",
            instructions="""# Level 6.6: Úklid logů

V adresáři je nepořádek. Všechny soubory s příponou `.log` by měly být v adresáři `logs`.

## Úkol:
Přesuňte všechny `.log` soubory (`app.log`, `error.log`) do adresáře `logs`.
Můžete to udělat jedním příkazem pomocí hvězdičky.

## Příkazy:
- `mv *.log logs/`

## Odevzdání:
Odevzdejte název adresáře, kam jste soubory přesunuli.
`shellgame submit -f logs`""",
            hints=[
                "Použijte 'mv *.log logs/'.",
                "Hvězdička vybere všechny soubory končící na .log.",
            ],
            optional=True,
            start_directory="level-6/organize",
        )

    def setup(self, workspace: Path) -> None:
        """Create logs and destination."""
        level_dir = workspace / "level-6" / "organize"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Reset state
        import shutil

        if (level_dir / "logs").exists():
            shutil.rmtree(level_dir / "logs")

        (level_dir / "logs").mkdir(exist_ok=True)
        (level_dir / "app.log").write_text("log1")
        (level_dir / "error.log").write_text("log2")
        (level_dir / "other.txt").write_text("keep me")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate organization."""
        if answer is None:
            return False, "Musíte zadat cíl."

        str_val = StringValidator("logs")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        validator = MultiValidator(
            [
                MoveValidator("level-6/organize/app.log", "level-6/organize/logs/app.log"),
                MoveValidator("level-6/organize/error.log", "level-6/organize/logs/error.log"),
                FileExistsValidator("level-6/organize/other.txt", should_exist=True),
            ]
        )
        return validator.validate(answer, state.workspace)


class Level6_7(Level):
    """Level 6.7: Section 6 Summary."""

    def __init__(self) -> None:
        super().__init__(
            id="6.7",
            section=6,
            title="Souhrn Sekce 6",
            instructions="""### 🎯 Výzva: Organizátor souborů

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
`shellgame submit organizer`""",
            hints=[
                "Kopírování: 'cp original.txt backup/'. Přejmenování: 'mv temp_data.csv data.csv'.",
                "Přesun do složky: 'mv misplaced.log logs/'.",
                "Zkontrolujte: 'ls backup/' (je tam original.txt?), 'ls' (je tam data.csv?), 'ls logs/' (je tam misplaced.log?).",
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create test environment."""
        import shutil

        test_dir = workspace / "level-6" / "final_test"

        if test_dir.exists():
            shutil.rmtree(test_dir)

        test_dir.mkdir(parents=True, exist_ok=True)
        (test_dir / "backup").mkdir()
        (test_dir / "logs").mkdir()

        (test_dir / "original.txt").write_text("Important data - do not delete!\n")
        (test_dir / "temp_data.csv").write_text("col1,col2\n1,2\n")
        (test_dir / "misplaced.log").write_text("Log entry\n")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate all operations."""
        if answer is None:
            return False, "Musíte zadat heslo."

        if answer.strip().lower() != "organizer":
            return False, "Heslo není správné. Splňte úkoly a odevzdejte: organizer"

        test_dir = state.workspace / "level-6" / "final_test"

        # Check copy (original must still exist + copy in backup)
        if not (test_dir / "original.txt").exists():
            return False, "Smazali jste original.txt! Měli jste ho zkopírovat, ne přesunout."
        if not (test_dir / "backup" / "original.txt").exists():
            return False, "Chybí kopie v backup/. Použijte 'cp original.txt backup/'."

        # Check rename
        if (test_dir / "temp_data.csv").exists():
            return False, "temp_data.csv stále existuje. Přejmenujte ho na data.csv pomocí 'mv'."
        if not (test_dir / "data.csv").exists():
            return (
                False,
                "Chybí data.csv. Přejmenujte temp_data.csv pomocí 'mv temp_data.csv data.csv'.",
            )

        # Check move
        if (test_dir / "misplaced.log").exists():
            return False, "misplaced.log stále v hlavní složce. Přesuňte do logs/ pomocí 'mv'."
        if not (test_dir / "logs" / "misplaced.log").exists():
            return False, "Chybí misplaced.log v logs/. Přesuňte pomocí 'mv misplaced.log logs/'."

        return (
            True,
            "🎉 Výborně! Dokončili jste Sekci 6. Umíte kopírovat, přesouvat i přejmenovávat!",
        )


def get_levels() -> list:
    """
    Return all Section 6 level classes.

    Returns:
        List of Level instances for Section 6
    """
    return [
        Level6_0(),
        Level6_1(),
        Level6_2(),
        Level6_3(),
        Level6_4(),
        Level6_5(),
        Level6_6(),
        Level6_7(),
    ]
