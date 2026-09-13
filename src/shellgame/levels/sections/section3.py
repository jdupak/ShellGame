"""Section 3: Hidden Files."""

from __future__ import annotations

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    AtDirectory,
    Completion,
    ExactAnswer,
    IntegerAnswer,
    IntegerRangeAnswer,
    TupleAnswer,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import Chdir, Solution
from shellgame.paths import WORKSPACE_ROOT
from shellgame.protocols import GameStateProtocol, ValidationResult

section = Section(3, root="level-3")


@section.level(0)
class SectionIntroLevel(Level):
    is_intro = True
    title = "Sekce 3: Skryté soubory"
    instructions_file = "section3_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"


@section.level(1)
class HiddenDirCountLevel(Level):
    title = "Počítání skrytých adresářů"
    instructions = """
        ### Cíl
        Najděte a spočítejte skryté adresáře.

        ### Příkazy k naučení
        - `ls -a` (zobrazí všechny soubory včetně skrytých)
        - `ls -aF` (navíc označí adresáře lomítkem `/` na konci)

        ### Úkol
        Nacházíte se v adresáři `level-3/hub`.
        1. Použijte `ls -aF` pro zobrazení všech položek i jejich typů.
        2. Spočítejte **skryté adresáře**: začínají tečkou a ve výpisu končí lomítkem.
        3. **Důležité:** Do počtu NEZAHRNUJTE speciální adresáře `.` (aktuální) a `..` (nadřazený).

        Odevzdejte počet nalezených skrytých adresářů (číslo).

        Odevzdejte pomocí: `shellgame submit [číslo]`
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Použijte 'ls -aF': -a zobrazí skryté položky, -F označí adresáře lomítkem.",
        "Počítejte jen názvy začínající tečkou a končící lomítkem.",
        "Nepočítejte './' ani '../'. Skryté soubory bez lomítka také vynechte.",
    ]
    start_directory = "hub"
    fixture = WorkspaceFixture(
        clean=("hub",),
        directories=("hub/.beta", "hub/.gamma", "hub/visible_dir"),
        files=(FileFixture("hub/.config"), FileFixture("hub/visible_file.txt")),
    )
    completion = Completion(answer=IntegerAnswer(2))


@section.level(2)
class HiddenFileReadLevel(Level):
    title = "Čtení skrytého souboru"
    instructions = """
        ### Cíl
        Přečtěte obsah skrytého souboru.

        ### Úkol
        V aktuálním adresáři je skrytý soubor `.secret_config`.
        1. Ověřte jeho existenci pomocí `ls -a`.
        2. Přečtěte jeho obsah pomocí `cat`.
        3. Odevzdejte obsah souboru.

        Odevzdejte pomocí: `shellgame submit [obsah]`
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Skryté soubory začínají tečkou. Jak je zobrazíte pomocí ls?",
        "Příkaz 'cat' funguje i na skryté soubory - stačí zadat správný název včetně tečky.",
        "Zkuste: cat .secret_config",
    ]
    start_directory = ""
    fixture = WorkspaceFixture(files=(FileFixture(".secret_config", "mode=stealth\n"),))
    completion = Completion(answer=ExactAnswer("mode=stealth"))


@section.level(3)
class HiddenVaultKeyLevel(Level):
    solution = Solution(steps=(Chdir("hub/.vault"),), answer="platinum")
    title = "Uvnitř skrytého adresáře"
    instructions = """
        ### Cíl
        Vstupte do skrytého adresáře.

        ### Úkol
        1. Najděte skrytý adresář `.vault`.
        2. Vstupte do něj (`cd .vault`).
        3. Uvnitř najděte soubor `key.txt` a přečtěte ho.
        4. Odevzdejte nalezený klíč.

        Odevzdejte pomocí: `shellgame submit [klíč]`
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Jak byste vstoupili do běžného adresáře? Stejně to funguje i se skrytými.",
        "Skrytý adresář .vault - jak se do něj dostanete pomocí cd?",
        "Jděte do .vault pomocí 'cd .vault', pak přečtěte key.txt.",
    ]
    start_directory = "hub"
    fixture = WorkspaceFixture(files=(FileFixture("hub/.vault/key.txt", "platinum\n"),))
    completion = Completion(
        answer=ExactAnswer("platinum", case_sensitive=False),
        requirements=(AtDirectory("hub/.vault"),),
    )


@section.level(4)
class HiddenBackupSuffixLevel(Level):
    title = "Skrytá záloha"
    instructions = """
        ### Cíl
        Identifikujte skrytý soubor podle přípony.

        ### Úkol
        V adresáři `level-3/backup` je několik skrytých souborů.
        Najděte ten, který má příponu `.bak` (záloha).
        Odevzdejte jeho celý název.

        Odevzdejte pomocí: `shellgame submit [název-souboru]`
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Použijte 'ls -a' v adresáři backup.",
        "Hledejte soubor začínající tečkou a končící .bak.",
        "Odevzdejte celý název včetně tečky na začátku.",
    ]
    start_directory = "backup"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("backup/.config"),
            FileFixture("backup/.data.bak"),
            FileFixture("backup/normal.txt"),
        )
    )
    completion = Completion(answer=ExactAnswer(".data.bak"))


@section.level(5)
class HiddenFilesSummaryChallengeLevel(Level):
    solution = Solution(answer="2,hidden_master")
    title = "Souhrn Sekce 3"
    instructions = """
        ### Výzva: Mistři skrytých souborů

        Ukažte, že ovládáte práci se skrytými soubory!

        ### Úkol
        V adresáři `level-3/final_test` jsou normální i skryté položky.

        1. Pomocí `ls -aF` spočítejte **skryté adresáře** (tečka na začátku, lomítko na konci; bez ./ a ../)
        2. Najděte skrytý soubor `.secret_code`
        3. Přečtěte jeho obsah
        4. Odevzdejte: `<počet>,<obsah>` (např. `3,tajne123`)

        ### Shrnutí příkazů Sekce 3
        ```
        ls -a         → Zobrazí vše včetně skrytých
        ls -aF        → Navíc označí adresáře lomítkem
        ls -la        → Detailní výpis všeho
        cat .soubor   → Přečíst skrytý soubor
        cd .adresar   → Vstoupit do skrytého adresáře
        ```

        ### Odevzdání
        `shellgame submit <počet>,<obsah>`
        """
    hints = [
        "Skryté položky začínají tečkou. Použijte 'ls -la' pro zobrazení všeho včetně typů.",
        "Adresáře poznáte podle 'd' na začátku řádku v ls -l, nebo podle / na konci v ls -F.",
        "Řádky pro '.' a '..' nepočítejte. Obsah souboru zobrazíte pomocí 'cat .secret_code'.",
    ]
    start_directory = "final_test"
    fixture = WorkspaceFixture(
        clean=("final_test",),
        directories=("final_test/.hidden_dir1", "final_test/.hidden_dir2", "final_test/visible_dir"),
        files=(
            FileFixture("final_test/.secret_code", "hidden_master\n"),
            FileFixture("final_test/.config", "not this one\n"),
            FileFixture("final_test/.notes", "Poznámky nejsou adresář.\n"),
            FileFixture("final_test/readme.txt", "Look for hidden items!\n"),
        ),
    )
    completion = Completion(
        answer=TupleAnswer(
            (
                IntegerAnswer(
                    2,
                    mistakes={4: "Možná počítáte i ./ a ../. Ty vynechte; počítejte jen skryté adresáře."},
                    error_message=(
                        "Počet skrytých adresářů není správně. Použijte 'ls -aF' a rozlište soubory a adresáře."
                    ),
                    invalid_message="První část musí být číslo (počet skrytých adresářů).",
                ),
                ExactAnswer(
                    "hidden_master",
                    case_sensitive=False,
                    mistakes={"not this one": "To je obsah .config, ne .secret_code."},
                    error_message="Kód není správný. Přečtěte .secret_code.",
                ),
            ),
            format_message="Formát odpovědi je: počet,kód (např. 3,tajne123)",
        )
    )
    success_message = "Výborně! Dokončili jste Sekci 3. Skryté soubory před vámi nic neskryjí!"


@section.level(6)
class SelfReflectionCheckpointLevel(Level):
    solution = Solution(answer="4")
    title = "Kontrolní bod: Sebehodnocení"
    instructions = """
        ### Čas na zamyšlení!

        Právě jste se naučili základy práce v terminálu:
        - **Sekce 1**: Navigace (`pwd`, `cd`, `ls`)
        - **Sekce 2**: Čtení souborů (`cat`, `ls -l`, `man`/`--help`)
        - **Sekce 3**: Skryté soubory (`ls -a`, soubory začínající `.`)

        ### Úkol: Sebehodnocení

        Na stupnici **1-5** ohodnoťte svou jistotu:
        - **1** = Potřebuji víc procvičování
        - **3** = Rozumím základům, ale občas váhám
        - **5** = Cítím se jistě, mohu pokračovat

        ### Otázky k zamyšlení
        1. Umím se pohybovat mezi adresáři pomocí `cd`?
        2. Dokážu zobrazit skryté soubory?
        3. Vím, jak přečíst obsah souboru?
        4. Umím najít nápovědu k příkazu?

        ### Odevzdání
        Odevzdejte číslo 1-5 podle vaší jistoty.
        - Pokud je vaše hodnocení **1-2**, projděte si znovu úvod předchozí sekce
        - Pokud je **3-5**, pokračujte dál!

        `shellgame submit <1-5>`
        """
    hints = [
        "Toto je sebehodnocení - neexistuje špatná odpověď!",
        "Buďte k sobě upřímní. Pokud váháte, vraťte se k předchozím levelům.",
        "Odevzdejte jakékoliv číslo od 1 do 5.",
    ]
    start_directory = WORKSPACE_ROOT
    completion = Completion(
        answer=IntegerRangeAnswer(
            1,
            5,
            error_message="Hodnocení musí být od 1 do 5.",
            invalid_message="Odevzdejte číslo od 1 do 5.",
            required_message="Odevzdejte číslo od 1 do 5.",
        ),
    )

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success or answer is None:
            return success, msg

        rating = int(answer.strip())
        if rating <= 2:
            return True, (
                "Děkujeme za upřímnost! Doporučujeme vrátit se k "
                "předchozím materiálům. Úvod sekce si zobrazíte například "
                "příkazem 'shellgame repeat --section 1'."
            )
        if rating == 3:
            return True, (
                "Dobrý základ! Pokud si nejste jisti konkrétním příkazem, "
                "můžete se kdykoliv vrátit. Pokračujte na Sekci 4!"
            )
        return True, (
            "Skvělé! Máte solidní základy. Pokračujte na Sekci 4, kde se naučíte vytvářet a organizovat soubory!"
        )
