from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import Completion, FileExists, IntegerAnswer, TupleAnswer
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import RunShell, Solution

section = Section(7, root="level-7")


@section.level(0)
class SectionIntro(Level):
    is_intro = True
    title = "Sekce 7: Žolíky (Wildcards)"
    instructions_file = "section7_intro.md"
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

        Potkali jste ji už dříve (`file *`, `mv *.log logs/`); tady si její pravidla poprvé pojmenujeme přesně.

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
    success_message = (
        "Správně! Jediný vzor s hvězdičkou zastoupil libovolný zbytek názvu, "
        "takže jste nemuseli soubory vypisovat po jednom."
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
        - V Bashi: `cp VZOR short_data/` - VZOR sestavte sami z `data`, žolíku a `.txt`
        - Z fish: `bash -c 'cp VZOR short_data/'` - uvozovky ponechte

        Vzor nejdřív ověřte pomocí `ls VZOR`: vypsat se smějí jen soubory, které chcete zkopírovat.

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
    success_message = "Správně! Otazník zastupuje právě jeden znak, proto se dvouciferný název do výběru nedostal."


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
        - `zprava_[xy].txt` → zprava_x.txt, zprava_y.txt
        - `log[123].txt` → log1.txt, log2.txt, log3.txt

        ### Úkol
        Zkopírujte `file_a.txt` a `file_b.txt` do adresáře `ab_files/`.
        NEKOPÍRUJTE `file_c.txt`.

        ### Příkazy
        - V Bashi: `cp VZOR ab_files/` - VZOR sestavte sami z `file_`, závorek s povolenými znaky a `.txt`
        - Z fish: `bash -c 'cp VZOR ab_files/'` - uvozovky ponechte

        Vzor nejdřív ověřte pomocí `ls VZOR`: vypsat se smějí jen soubory, které chcete zkopírovat.

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
    success_message = (
        "Správně! Závorky povolují na jedné pozici jen vyjmenované znaky, takže vzor vybral přesně požadovanou dvojici."
    )


@section.level(4)
class RangeWildcardCopyLevel(Level):
    solution = Solution(steps=(RunShell("cp [[:lower:]]*.txt lowercase/"),), answer=None)
    title = "Třídy znaků [[:lower:]]"
    instructions = """
        # Třídy znaků [[:lower:]]

        Rozsahy jako `[a-z]` závisejí na řazení nastaveného jazyka (locale).
        POSIX třída `[[:lower:]]` místo pořadí vybírá jeden znak klasifikovaný
        jako malé písmeno. Pro tento úkol je proto spolehlivější napříč locale.

        Podobně existují `[[:upper:]]` pro velká písmena a `[[:digit:]]` pro číslice.

        ### Požadavek: Bash
        Tento level vyžaduje **Bash**. Ve fish použijte variantu s `bash -c` níže.
        Ta spustí pouze kopírování v Bashi; odevzdávejte dál ve svém herním shellu.

        ### Úkol
        Zkopírujte všechny soubory `.txt` začínající malým písmenem do adresáře `lowercase/`.
        NEKOPÍRUJTE soubory začínající velkým písmenem.

        ### Příkazy
        - V Bashi: `cp VZOR lowercase/` - VZOR sestavte sami z třídy znaků, `*` a `.txt`
        - Z fish: `bash -c 'cp VZOR lowercase/'` - uvozovky ponechte

        Vzor nejdřív ověřte pomocí `ls VZOR`: vypsat se smějí jen soubory, které chcete zkopírovat.

        Přípona `.txt` vyloučí cílový adresář `lowercase`, který také začíná malým písmenem.

        ### Odevzdání
        Po splnění úkolu odevzdejte: `shellgame submit`
        """
    hints = [
        "Třída [[:lower:]] vybere na dané pozici právě jedno malé písmeno bez závislosti na pořadí znaků v locale.",
        "Vzor [[:lower:]]*.txt vybere názvy začínající malým písmenem a končící příponou .txt.",
        "V Bashi použijte `cp [[:lower:]]*.txt lowercase/`. Z fish: `bash -c 'cp [[:lower:]]*.txt lowercase/'`.",
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
    success_message = (
        "Správně! Třída znaků vybírá podle klasifikace písmene, "
        "takže výsledek nezávisí na řazení znaků v nastaveném locale."
    )


@section.level(5)
class WildcardsChallengeLevel(Level):
    solution = Solution(
        steps=(
            RunShell("cp *.log logs/"),
            RunShell("cp data?.txt short_data/"),
            RunShell("cp report_[ab].csv selected_reports/"),
        ),
        answer="3,2,2",
    )
    title = "Souhrn Sekce 7"
    instructions = """
        ### Výzva: Tři druhy žolíků

        ### Úkol
        V aktuálním adresáři použijte pro každý výběr jiný druh vzoru.
        Než začnete kopírovat, nejprve si předpovězte odpovídající názvy a vzor ověřte pomocí `ls VZOR`.

        1. Pomocí `*` zkopírujte všechny `.log` soubory do `logs/`.
        2. Pomocí `?` zkopírujte do `short_data/` jen názvy `data`, jeden znak a `.txt`.
        3. Pomocí znakové třídy zkopírujte do `selected_reports/` reporty s písmenem `a` nebo `b`.

        Nakonec odevzdejte počty souborů v těchto třech cílových adresářích.
        Formát: `<logy>,<kratka_data>,<vybrane_reporty>`

        `?` a `[...]` zde používají syntaxi Bashe. Ve fish spusťte kopírování přes `bash -c`.

        ### Odevzdání
        `shellgame submit <logy>,<data>,<reporty>`
        """
    hints = [
        "Každý krok má procvičit jiný vzor. Nejdřív si pomocí 'ls' ověřte, zda vzor nevybírá některý z decoy souborů.",
        "Logy potřebují žolík pro libovolný zbytek názvu, krátká data žolík pro právě jeden znak "
        "a reporty znakovou třídu se dvěma povolenými písmeny; každý sestavený vzor ověřte pomocí `ls`.",
        "Bash: `cp *.log logs/; cp data?.txt short_data/; cp report_[ab].csv selected_reports/`. "
        "Fish: `bash -c 'cp *.log logs/; cp data?.txt short_data/; cp report_[ab].csv selected_reports/'`.",
    ]
    start_directory = "challenge"
    fixture = WorkspaceFixture(
        directories=(
            "challenge/logs",
            "challenge/short_data",
            "challenge/selected_reports",
        ),
        files=(
            FileFixture("challenge/app.log", "log1"),
            FileFixture("challenge/error.log", "log2"),
            FileFixture("challenge/debug.log", "log3"),
            FileFixture("challenge/notes.txt", "txt"),
            FileFixture("challenge/data1.txt", "short1"),
            FileFixture("challenge/data2.txt", "short2"),
            FileFixture("challenge/data10.txt", "long"),
            FileFixture("challenge/report_a.csv", "a\n"),
            FileFixture("challenge/report_b.csv", "b\n"),
            FileFixture("challenge/report_c.csv", "c\n"),
            FileFixture("challenge/script.sh", "#!/bin/bash\n"),
        ),
        clean=("challenge",),
    )
    completion = Completion(
        answer=TupleAnswer(
            (
                IntegerAnswer(
                    3,
                    error_message="Počet zkopírovaných logů není správně. Zkontrolujte vzor s `*`.",
                    invalid_message="Všechny tři hodnoty musí být čísla.",
                ),
                IntegerAnswer(
                    2,
                    error_message="Počet krátkých datových názvů není správně. Zkontrolujte vzor s `?`.",
                    invalid_message="Všechny tři hodnoty musí být čísla.",
                ),
                IntegerAnswer(
                    2,
                    error_message="Počet vybraných reportů není správně. Zkontrolujte znakovou třídu.",
                    invalid_message="Všechny tři hodnoty musí být čísla.",
                ),
            ),
            format_message="Formát: logy,data,reporty (tři čísla oddělená čárkou)",
        ),
        requirements=(
            FileExists("challenge/logs/app.log"),
            FileExists("challenge/logs/error.log"),
            FileExists("challenge/logs/debug.log"),
            FileExists("challenge/short_data/data1.txt"),
            FileExists("challenge/short_data/data2.txt"),
            FileExists("challenge/short_data/data10.txt", should_exist=False),
            FileExists("challenge/selected_reports/report_a.csv"),
            FileExists("challenge/selected_reports/report_b.csv"),
            FileExists("challenge/selected_reports/report_c.csv", should_exist=False),
        ),
    )
    success_message = "Výborně! Dokončili jste Sekci 7. Wildcards jsou váš nejlepší přítel!"
