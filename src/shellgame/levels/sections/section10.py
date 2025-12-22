"""Section 10: Wildcards."""

from __future__ import annotations

import shutil
from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import FileExistsValidator, ValidationResult


class SectionIntro(Level):
    title = "Sekce 10: Žolíky (Wildcards)"
    instructions_file = "section10_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"

    @override
    def setup(self, workspace: Path) -> None:
        return

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


class StarWildcardCopyLevel(Level):
    title = "Hvězdička *"
    instructions = """
        # Hvězdička *

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
        Po splnění úkolu odevzdejte: `shellgame submit`
        """
    hints = [
        "Hvězdička (*) nahrazuje libovolný počet znaků.",
        "Příkaz 'ls *.txt' vypíše všechny soubory s příponou .txt.",
    ]
    # NOTE: preserve original UX (start dir at section hub), not the actual task dir
    start_directory = "level-10/wildcards"
    validators = [
        FileExistsValidator("level-10/star/images/photo1.jpg", should_exist=True),
        FileExistsValidator("level-10/star/images/photo2.jpg", should_exist=True),
        FileExistsValidator("level-10/star/images/notes.txt", should_exist=False),
    ]

    @override
    def setup(self, workspace: Path) -> None:
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


class QuestionMarkWildcardCopyLevel(Level):
    title = "Otazník ?"
    instructions = """
        # Otazník ?

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
        Po splnění úkolu odevzdejte: `shellgame submit`
        """
    hints = [
        "Otazník nahradí právě jeden znak. Kolik znaků je mezi 'data' a '.txt' v data1.txt?",
        "data?.txt zachytí data1.txt a data2.txt, ale ne data10.txt (tam jsou dva znaky).",
        "Použijte 'cp data?.txt short_data/'.",
    ]
    start_directory = "level-10/wildcards"
    validators = [
        FileExistsValidator("level-10/question/short_data/data1.txt", should_exist=True),
        FileExistsValidator("level-10/question/short_data/data2.txt", should_exist=True),
        FileExistsValidator("level-10/question/short_data/data10.txt", should_exist=False),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-10" / "question"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "short_data").mkdir(exist_ok=True)

        (level_dir / "data1.txt").touch()
        (level_dir / "data2.txt").touch()
        (level_dir / "data10.txt").touch()

        # Clean up destination
        for f in (level_dir / "short_data").glob("*"):
            f.unlink()


class CharacterClassWildcardCopyLevel(Level):
    title = "Výběr znaků []"
    instructions = """
        # Výběr znaků []

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
        Po splnění úkolu odevzdejte: `shellgame submit`
        """
    hints = [
        "Hranaté závorky definují množinu povolených znaků na dané pozici.",
        "[ab] znamená 'a nebo b', takže file_[ab].txt zachytí file_a.txt a file_b.txt.",
        "Použijte 'cp file_[ab].txt ab_files/'.",
    ]
    start_directory = "level-10/wildcards"
    validators = [
        FileExistsValidator("level-10/brackets/ab_files/file_a.txt", should_exist=True),
        FileExistsValidator("level-10/brackets/ab_files/file_b.txt", should_exist=True),
        FileExistsValidator("level-10/brackets/ab_files/file_c.txt", should_exist=False),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-10" / "brackets"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "ab_files").mkdir(exist_ok=True)

        (level_dir / "file_a.txt").touch()
        (level_dir / "file_b.txt").touch()
        (level_dir / "file_c.txt").touch()

        # Clean up destination
        for f in (level_dir / "ab_files").glob("*"):
            f.unlink()


class RangeWildcardCopyLevel(Level):
    title = "Rozsahy [a-z]"
    instructions = """
        # Rozsahy znaků [a-z]

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
        Po splnění úkolu odevzdejte: `shellgame submit`
        """
    hints = [
        "Rozsah [a-z] vybere všechna malá písmena od 'a' do 'z'.",
        "Vzor [a-z]* znamená: začíná malým písmenem, pak cokoliv.",
        "Použijte 'cp [a-z]* lowercase/'.",
    ]
    start_directory = "level-10/wildcards"
    validators = [
        FileExistsValidator("level-10/ranges/lowercase/apple.txt", should_exist=True),
        FileExistsValidator("level-10/ranges/lowercase/cherry.txt", should_exist=True),
        FileExistsValidator("level-10/ranges/lowercase/Banana.txt", should_exist=False),
        FileExistsValidator("level-10/ranges/lowercase/Date.txt", should_exist=False),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-10" / "ranges"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "lowercase").mkdir(exist_ok=True)

        (level_dir / "apple.txt").touch()
        (level_dir / "Banana.txt").touch()
        (level_dir / "cherry.txt").touch()
        (level_dir / "Date.txt").touch()

        # Clean up destination
        for f in (level_dir / "lowercase").glob("*"):
            f.unlink()


class WildcardsChallengeLevel(Level):
    title = "Souhrn Sekce 10"
    instructions = """
        ### Výzva: Mistr wildcardů

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
        `shellgame submit <log>,<txt>,<csv>`
        """
    hints = [
        "'ls *.log' a spočítejte řádky. 'ls *.txt | wc -l' pro počet txt souborů.",
        "'ls report*.csv' pro nalezení CSV souboru.",
        "Jsou tam 3 log soubory, 4 txt soubory, a CSV je 'report_final.csv'.",
    ]
    require_answer = True

    @override
    def setup(self, workspace: Path) -> None:
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

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        # No declarative validators/expected_answer here, but keep the standard entrypoint.
        _success, _msg = super().validate(answer, state)

        ok = False
        msg = "Zadejte odpověď ve formátu: počet_log,počet_txt,název_csv"

        if answer is None:
            return ok, msg

        text = answer.strip()
        parts = text.split(",")

        if len(parts) != 3:
            return False, "Formát: počet_log,počet_txt,název_csv (např. 5,3,report.csv)"

        try:
            log_count = int(parts[0].strip())
            txt_count = int(parts[1].strip())
        except ValueError:
            return False, "První dvě hodnoty musí být čísla."

        csv_name = parts[2].strip().lower()

        if log_count != 3:
            msg = f"Počet .log souborů není {log_count}. Použijte 'ls *.log'."
        elif txt_count != 4:
            msg = f"Počet .txt souborů není {txt_count}. Použijte 'ls *.txt | wc -l'."
        elif csv_name != "report_final.csv":
            msg = f"CSV soubor není {csv_name}. Použijte 'ls report*.csv'."
        else:
            ok = True
            msg = "Výborně! Dokončili jste Sekci 10. Wildcards jsou váš nejlepší přítel!"

        return ok, msg


def get_levels() -> list[Level]:
    levels: list[Level] = [
        SectionIntro(),
        StarWildcardCopyLevel(),
        QuestionMarkWildcardCopyLevel(),
        CharacterClassWildcardCopyLevel(),
        RangeWildcardCopyLevel(),
        WildcardsChallengeLevel(),
    ]

    section_num = 10
    for i, level in enumerate(levels):
        level.section = section_num
        level.id = f"{section_num}.{i}"

    return levels
