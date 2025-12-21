"""Section 10: Wildcards."""

import shutil
from pathlib import Path
from typing import Any, Optional

from shellgame.levels.base import Level
from shellgame.validation.validators import FileExistsValidator, MultiValidator


class Level10_0(Level):
    """Level 10.0: Section 10 Introduction."""

    def __init__(self) -> None:
        super().__init__(
            id="10.0",
            section=10,
            title="Sekce 10: Žolíky (Wildcards)",
            instructions_file="section10_intro.md",
            hints=["Přečtěte si úvod a pokračujte stisknutím Enter."],
        )

    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        pass

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Always valid."""
        return True, "Jdeme na to!"


class Level10_1(Level):
    """Level 10.1: The Star Wildcard."""

    def __init__(self) -> None:
        super().__init__(
            id="10.1",
            section=10,
            title="Hvězdička *",
            instructions="""# Level 10.1: Hvězdička *

Hvězdička `*` nahradí JAKOUKOLIV sekvenci znaků (včetně prázdné).
Je velmi užitečná pro výběr souborů se specifickou příponou.

### Proč je to užitečné
Představte si, že máte 100 fotografií a chcete je všechny zkopírovat.
Místo `cp foto1.jpg foto2.jpg foto3.jpg ...` stačí `cp *.jpg cíl/`.

### Úkol
Zkopírujte všechny soubory s příponou `.jpg` do adresáře `images`.
(Adresář `images` již existuje).

### Příkazy
- `cp *.jpg adresář/` - zkopíruje všechny .jpg soubory

### Odevzdání
Po splnění úkolu odevzdejte: `shellgame submit`""",
            hints=[
                "Hvězdička (*) nahrazuje libovolný počet znaků.",
                "Příkaz 'ls *.txt' vypíše všechny soubory s příponou .txt.",
            ],
            start_directory="level-10/wildcards",
        )

    def setup(self, workspace: Path) -> None:
        """Create files and directory."""
        level_dir = workspace / "level-10" / "star"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "images").mkdir(exist_ok=True)

        # Create source files
        (level_dir / "photo1.jpg").touch()
        (level_dir / "photo2.jpg").touch()
        (level_dir / "notes.txt").touch()

        # Clean up destination
        for f in (level_dir / "images").glob("*"):
            f.unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate copy."""
        base = "level-10/star/images"

        validators = [
            FileExistsValidator(f"{base}/photo1.jpg", should_exist=True),
            FileExistsValidator(f"{base}/photo2.jpg", should_exist=True),
            FileExistsValidator(f"{base}/notes.txt", should_exist=False),
        ]

        validator = MultiValidator(validators)
        return validator.validate(answer or "", state.workspace)


class Level10_2(Level):
    """Level 10.2: The Question Mark."""

    def __init__(self) -> None:
        super().__init__(
            id="10.2",
            section=10,
            title="Otazník ?",
            instructions="""# Level 10.2: Otazník ?

Otazník `?` nahradí PRÁVĚ JEDEN znak.
Je užitečný, když chcete být přesnější než s hvězdičkou.

### Rozdíl od hvězdičky
- `*` = libovolný počet znaků (0 nebo více)
- `?` = přesně jeden znak

### Úkol
Zkopírujte `data1.txt` a `data2.txt` do adresáře `short_data/`.
NEKOPÍRUJTE `data10.txt` (má dvouciferné číslo).

### Příkazy
- `cp data?.txt adresář/` - ? nahradí právě jeden znak

### Odevzdání
Po splnění úkolu odevzdejte: `shellgame submit`""",
            hints=[
                "Otazník nahradí právě jeden znak. Kolik znaků je mezi 'data' a '.txt' v data1.txt?",
                "data?.txt zachytí data1.txt a data2.txt, ale ne data10.txt (tam jsou dva znaky).",
                "Použijte 'cp data?.txt short_data/'.",
            ],
            start_directory="level-10/wildcards",
        )

    def setup(self, workspace: Path) -> None:
        """Create files."""
        level_dir = workspace / "level-10" / "question"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "short_data").mkdir(exist_ok=True)

        (level_dir / "data1.txt").touch()
        (level_dir / "data2.txt").touch()
        (level_dir / "data10.txt").touch()

        # Clean up
        for f in (level_dir / "short_data").glob("*"):
            f.unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate copy."""
        base = "level-10/question/short_data"

        validators = [
            FileExistsValidator(f"{base}/data1.txt", should_exist=True),
            FileExistsValidator(f"{base}/data2.txt", should_exist=True),
            FileExistsValidator(f"{base}/data10.txt", should_exist=False),
        ]

        validator = MultiValidator(validators)
        return validator.validate(answer or "", state.workspace)


class Level10_3(Level):
    """Level 10.3: Character Classes."""

    def __init__(self) -> None:
        super().__init__(
            id="10.3",
            section=10,
            title="Výběr znaků []",
            instructions="""# Level 10.3: Výběr znaků []

Hranaté závorky `[...]` nahradí JEDEN ze znaků uvnitř.
Například `[abc]` odpovídá znaku 'a', 'b' nebo 'c'.

### Příklady
- `file_[ab].txt` → file_a.txt, file_b.txt
- `log[123].txt` → log1.txt, log2.txt, log3.txt

### Úkol
Zkopírujte `file_a.txt` a `file_b.txt` do adresáře `ab_files/`.
NEKOPÍRUJTE `file_c.txt`.

### Příkazy
- `cp file_[ab].txt adresář/`

### Odevzdání
Po splnění úkolu odevzdejte: `shellgame submit`""",
            hints=[
                "Hranaté závorky definují množinu povolených znaků na dané pozici.",
                "[ab] znamená 'a nebo b', takže file_[ab].txt zachytí file_a.txt a file_b.txt.",
                "Použijte 'cp file_[ab].txt ab_files/'.",
            ],
            start_directory="level-10/wildcards",
        )

    def setup(self, workspace: Path) -> None:
        """Create files."""
        level_dir = workspace / "level-10" / "brackets"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "ab_files").mkdir(exist_ok=True)

        (level_dir / "file_a.txt").touch()
        (level_dir / "file_b.txt").touch()
        (level_dir / "file_c.txt").touch()

        # Clean up
        for f in (level_dir / "ab_files").glob("*"):
            f.unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate copy."""
        base = "level-10/brackets/ab_files"

        validators = [
            FileExistsValidator(f"{base}/file_a.txt", should_exist=True),
            FileExistsValidator(f"{base}/file_b.txt", should_exist=True),
            FileExistsValidator(f"{base}/file_c.txt", should_exist=False),
        ]

        validator = MultiValidator(validators)
        return validator.validate(answer or "", state.workspace)


class Level10_4(Level):
    """Level 10.4: Ranges."""

    def __init__(self) -> None:
        super().__init__(
            id="10.4",
            section=10,
            title="Rozsahy [a-z]",
            instructions="""# Level 10.4: Rozsahy znaků [a-z]

Uvnitř hranatých závorek můžete zadat rozsah znaků pomocí pomlčky `-`.
- `[a-z]` odpovídá jakémukoliv malému písmenu
- `[0-9]` odpovídá jakékoliv číslici
- `[A-Z]` odpovídá jakémukoliv velkému písmenu

### Příklady
- `[a-c]` = a, b, nebo c
- `[0-5]` = 0, 1, 2, 3, 4, nebo 5

### Úkol
Zkopírujte všechny soubory začínající malým písmenem do adresáře `lowercase/`.
NEKOPÍRUJTE soubory začínající velkým písmenem.

### Příkazy
- `cp [a-z]* lowercase/`

### Odevzdání
Po splnění úkolu odevzdejte: `shellgame submit`""",
            hints=[
                "Rozsah [a-z] vybere všechna malá písmena od 'a' do 'z'.",
                "Vzor [a-z]* znamená: začíná malým písmenem, pak cokoliv.",
                "Použijte 'cp [a-z]* lowercase/'.",
            ],
            start_directory="level-10/wildcards",
        )

    def setup(self, workspace: Path) -> None:
        """Create files."""
        level_dir = workspace / "level-10" / "ranges"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "lowercase").mkdir(exist_ok=True)

        (level_dir / "apple.txt").touch()
        (level_dir / "Banana.txt").touch()
        (level_dir / "cherry.txt").touch()
        (level_dir / "Date.txt").touch()

        # Clean up
        for f in (level_dir / "lowercase").glob("*"):
            f.unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate copy."""
        base = "level-10/ranges/lowercase"

        validators = [
            FileExistsValidator(f"{base}/apple.txt", should_exist=True),
            FileExistsValidator(f"{base}/cherry.txt", should_exist=True),
            FileExistsValidator(f"{base}/Banana.txt", should_exist=False),
            FileExistsValidator(f"{base}/Date.txt", should_exist=False),
        ]

        validator = MultiValidator(validators)
        return validator.validate(answer or "", state.workspace)


class Level10_5(Level):
    """Level 10.5: Section 10 Summary."""

    def __init__(self) -> None:
        super().__init__(
            id="10.5",
            section=10,
            title="Souhrn Sekce 10",
            instructions="""### 🎯 Výzva: Mistr wildcardů

Ukažte, že ovládáte zástupné znaky!

### Úkol
V `level-10/challenge`:

1. Vypište **pouze** soubory končící na `.log` pomocí `ls *.log`
2. Spočítejte, kolik je `.txt` souborů
3. Najděte soubor, který začíná na `report` a má příponu `.csv`

### Formát odpovědi
`<pocet_log>,<pocet_txt>,<nazev_csv>`

Příklad: `5,3,report_2024.csv`

### Shrnutí příkazů Sekce 10
```
*           → libovolné znaky (i žádné)
?           → právě jeden znak
[abc]       → jeden znak z množiny
[a-z]       → jeden znak z rozsahu
ls *.txt    → soubory končící na .txt
rm temp*    → smaže vše začínající na temp
```

### Odevzdání
`shellgame submit <log>,<txt>,<csv>`""",
            hints=[
                "'ls *.log' a spočítejte řádky. 'ls *.txt | wc -l' pro počet txt souborů.",
                "'ls report*.csv' pro nalezení CSV souboru.",
                "Jsou tam 3 log soubory, 4 txt soubory, a CSV je 'report_final.csv'.",
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create challenge files."""
        challenge_dir = workspace / "level-10" / "challenge"

        if challenge_dir.exists():
            shutil.rmtree(challenge_dir)

        challenge_dir.mkdir(parents=True, exist_ok=True)

        # Log files (3)
        (challenge_dir / "app.log").write_text("log1")
        (challenge_dir / "error.log").write_text("log2")
        (challenge_dir / "debug.log").write_text("log3")

        # Text files (4)
        (challenge_dir / "notes.txt").write_text("txt1")
        (challenge_dir / "readme.txt").write_text("txt2")
        (challenge_dir / "todo.txt").write_text("txt3")
        (challenge_dir / "data.txt").write_text("txt4")

        # CSV file
        (challenge_dir / "report_final.csv").write_text("col1,col2\n")

        # Distractors
        (challenge_dir / "script.sh").write_text("#!/bin/bash\n")
        (challenge_dir / "config.json").write_text("{}")

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:  # noqa: PLR0911
        """Validate wildcard knowledge."""
        if answer is None:
            return False, "Zadejte odpověď ve formátu: počet_log,počet_txt,název_csv"

        answer = answer.strip()
        parts = answer.split(",")

        if len(parts) != 3:
            return False, "Formát: počet_log,počet_txt,název_csv (např. 5,3,report.csv)"

        try:
            log_count = int(parts[0].strip())
            txt_count = int(parts[1].strip())
            csv_name = parts[2].strip().lower()
        except ValueError:
            return False, "První dvě hodnoty musí být čísla."

        if log_count != 3:
            return False, f"Počet .log souborů není {log_count}. Použijte 'ls *.log'."

        if txt_count != 4:
            return False, f"Počet .txt souborů není {txt_count}. Použijte 'ls *.txt | wc -l'."

        if csv_name != "report_final.csv":
            return False, f"CSV soubor není {csv_name}. Použijte 'ls report*.csv'."

        return True, "🎉 Výborně! Dokončili jste Sekci 10. Wildcards jsou váš nejlepší přítel!"


def get_levels() -> list:
    """
    Return all Section 10 level classes.

    Returns:
        List of Level instances for Section 10
    """
    return [
        Level10_0(),
        Level10_1(),
        Level10_2(),
        Level10_3(),
        Level10_4(),
        Level10_5(),
    ]
