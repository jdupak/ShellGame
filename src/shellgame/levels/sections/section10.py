from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import Completion, ExactAnswer, FileExists, IntegerAnswer, TupleAnswer
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import RunShell, Solution

section = Section(10, root="level-10")


@section.level(0)
class SectionIntro(Level):
    is_intro = True
    title = "Sekce 10: Žolíky (Wildcards)"
    instructions_file = "section10_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"


@section.level(1)
class StarWildcardCopyLevel(Level):
    solution = Solution(steps=(RunShell("cp *.jpg images/"),), answer=None)
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
        "Příkaz 'ls *.jpg' vypíše všechny soubory s příponou .jpg.",
        "Použijte 'cp *.jpg images/' pro zkopírování všech souborů s příponou .jpg do adresáře images.",
    ]
    start_directory = "star"
    fixture = WorkspaceFixture(
        directories=("star/images",),
        files=(
            FileFixture("star/photo1.jpg"),
            FileFixture("star/photo2.jpg"),
            FileFixture("star/notes.txt"),
        ),
        clean=("star/images",),
    )
    completion = Completion(
        requirements=(
            FileExists("star/images/photo1.jpg"),
            FileExists("star/images/photo2.jpg"),
            FileExists("star/images/notes.txt", should_exist=False),
        )
    )


@section.level(2)
class QuestionMarkWildcardCopyLevel(Level):
    solution = Solution(steps=(RunShell("cp data?.txt short_data/"),), answer=None)
    title = "Otazník ?"
    instructions = """
        # Otazník ?

        Otazník `?` nahradí PRÁVĚ JEDEN znak.
        Je užitečný, když chcete být přesnější než s hvězdičkou.

        ### Požadavek: Bash
        Tento level vyžaduje **Bash**. Ve fish použijte variantu s `bash -c` níže.
        Ta spustí pouze kopírování v Bashi; odevzdávejte dál ve svém herním shellu.

        ### Rozdíl od hvězdičky
        - `*` = libovolný počet znaků (0 nebo více)
        - `?` = přesně jeden znak

        ### Úkol
        Zkopírujte `data1.txt` a `data2.txt` do adresáře `short_data/`.
        NEKOPÍRUJTE `data10.txt` (má dvouciferné číslo).

        ### Příkazy
        - V Bashi: `cp data?.txt short_data/` - ? nahradí právě jeden znak
        - Z fish: `bash -c 'cp data?.txt short_data/'` - uvozovky ponechte

        ### Odevzdání
        Po splnění úkolu odevzdejte: `shellgame submit`
        """
    hints = [
        "Otazník nahradí právě jeden znak. Kolik znaků je mezi 'data' a '.txt' v data1.txt?",
        "data?.txt zachytí data1.txt a data2.txt, ale ne data10.txt (tam jsou dva znaky).",
        "V Bashi použijte `cp data?.txt short_data/`. Z fish: `bash -c 'cp data?.txt short_data/'`.",
    ]
    start_directory = "question"
    fixture = WorkspaceFixture(
        directories=("question/short_data",),
        files=(
            FileFixture("question/data1.txt"),
            FileFixture("question/data2.txt"),
            FileFixture("question/data10.txt"),
        ),
        clean=("question/short_data",),
    )
    completion = Completion(
        requirements=(
            FileExists("question/short_data/data1.txt"),
            FileExists("question/short_data/data2.txt"),
            FileExists("question/short_data/data10.txt", should_exist=False),
        )
    )


@section.level(3)
class CharacterClassWildcardCopyLevel(Level):
    solution = Solution(steps=(RunShell("cp file_[ab].txt ab_files/"),), answer=None)
    title = "Výběr znaků []"
    instructions = """
        # Výběr znaků []

        Hranaté závorky `[...]` nahradí JEDEN ze znaků uvnitř.
        Například `[abc]` odpovídá znaku 'a', 'b' nebo 'c'.

        ### Požadavek: Bash
        Tento level vyžaduje **Bash**. Ve fish použijte variantu s `bash -c` níže.
        Ta spustí pouze kopírování v Bashi; odevzdávejte dál ve svém herním shellu.

        ### Příklady
        - `file_[ab].txt` → file_a.txt, file_b.txt
        - `log[123].txt` → log1.txt, log2.txt, log3.txt

        ### Úkol
        Zkopírujte `file_a.txt` a `file_b.txt` do adresáře `ab_files/`.
        NEKOPÍRUJTE `file_c.txt`.

        ### Příkazy
        - V Bashi: `cp file_[ab].txt ab_files/`
        - Z fish: `bash -c 'cp file_[ab].txt ab_files/'` - uvozovky ponechte

        ### Odevzdání
        Po splnění úkolu odevzdejte: `shellgame submit`
        """
    hints = [
        "Hranaté závorky definují množinu povolených znaků na dané pozici.",
        "[ab] znamená 'a nebo b', takže file_[ab].txt zachytí file_a.txt a file_b.txt.",
        "V Bashi použijte `cp file_[ab].txt ab_files/`. Z fish: `bash -c 'cp file_[ab].txt ab_files/'`.",
    ]
    start_directory = "brackets"
    fixture = WorkspaceFixture(
        directories=("brackets/ab_files",),
        files=(
            FileFixture("brackets/file_a.txt"),
            FileFixture("brackets/file_b.txt"),
            FileFixture("brackets/file_c.txt"),
        ),
        clean=("brackets/ab_files",),
    )
    completion = Completion(
        requirements=(
            FileExists("brackets/ab_files/file_a.txt"),
            FileExists("brackets/ab_files/file_b.txt"),
            FileExists("brackets/ab_files/file_c.txt", should_exist=False),
        )
    )


@section.level(4)
class RangeWildcardCopyLevel(Level):
    solution = Solution(steps=(RunShell("cp [a-z]*.txt lowercase/"),), answer=None)
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

        ### Požadavek: Bash
        Tento level vyžaduje **Bash**. Ve fish použijte variantu s `bash -c` níže.
        Ta spustí pouze kopírování v Bashi; odevzdávejte dál ve svém herním shellu.

        ### Úkol
        Zkopírujte všechny soubory `.txt` začínající malým písmenem do adresáře `lowercase/`.
        NEKOPÍRUJTE soubory začínající velkým písmenem.

        ### Příkazy
        - V Bashi: `cp [a-z]*.txt lowercase/`
        - Z fish: `bash -c 'cp [a-z]*.txt lowercase/'` - uvozovky ponechte

        Přípona `.txt` vyloučí cílový adresář `lowercase`, který také začíná malým písmenem.

        ### Odevzdání
        Po splnění úkolu odevzdejte: `shellgame submit`
        """
    hints = [
        "Rozsah [a-z] vybere všechna malá písmena od 'a' do 'z'.",
        "Vzor [a-z]*.txt vybere názvy začínající malým písmenem a končící příponou .txt.",
        "V Bashi použijte `cp [a-z]*.txt lowercase/`. Z fish: `bash -c 'cp [a-z]*.txt lowercase/'`.",
    ]
    start_directory = "ranges"
    fixture = WorkspaceFixture(
        directories=("ranges/lowercase",),
        files=(
            FileFixture("ranges/apple.txt"),
            FileFixture("ranges/Banana.txt"),
            FileFixture("ranges/cherry.txt"),
            FileFixture("ranges/Date.txt"),
        ),
        clean=("ranges/lowercase",),
    )
    completion = Completion(
        requirements=(
            FileExists("ranges/lowercase/apple.txt"),
            FileExists("ranges/lowercase/cherry.txt"),
            FileExists("ranges/lowercase/Banana.txt", should_exist=False),
            FileExists("ranges/lowercase/Date.txt", should_exist=False),
        )
    )


@section.level(5)
class WildcardsChallengeLevel(Level):
    solution = Solution(answer="3,4,report_final.csv")
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

        ### Shrnutí příkazů Sekce 10 (Bash)
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
        "Zástupný znak '*' vybere všechny soubory s danou příponou (např. *.log nebo *.txt).",
        "Příkazy 'ls *.log' a 'ls *.txt' vypíší hledané soubory. Spočítat je můžete i přes '| wc -l'.",
        "Hledaný CSV soubor najdete přes 'ls report*.csv'. Odpověď odevzdejte jako tři hodnoty oddělené čárkou.",
    ]
    start_directory = "challenge"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("challenge/app.log", "log1"),
            FileFixture("challenge/error.log", "log2"),
            FileFixture("challenge/debug.log", "log3"),
            FileFixture("challenge/notes.txt", "txt1"),
            FileFixture("challenge/readme.txt", "txt2"),
            FileFixture("challenge/todo.txt", "txt3"),
            FileFixture("challenge/data.txt", "txt4"),
            FileFixture("challenge/report_final.csv", "col1,col2\n"),
            FileFixture("challenge/script.sh", "#!/bin/bash\n"),
            FileFixture("challenge/config.json", "{}"),
        ),
        clean=("challenge",),
    )
    completion = Completion(
        answer=TupleAnswer(
            (
                IntegerAnswer(
                    3,
                    error_message="Počet .log souborů není správně. Použijte 'ls *.log'.",
                    invalid_message="První dvě hodnoty musí být čísla.",
                ),
                IntegerAnswer(
                    4,
                    error_message="Počet .txt souborů není správně. Použijte 'ls *.txt | wc -l'.",
                    invalid_message="První dvě hodnoty musí být čísla.",
                ),
                ExactAnswer(
                    "report_final.csv",
                    case_sensitive=False,
                    error_message="CSV soubor není správně. Použijte 'ls report*.csv'.",
                ),
            ),
            format_message="Formát: počet_log,počet_txt,název_csv (např. 5,3,report.csv)",
        )
    )
    success_message = "Výborně! Dokončili jste Sekci 10. Wildcards jsou váš nejlepší přítel!"
