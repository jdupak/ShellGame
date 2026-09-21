"""Section 3: Hidden Files."""

from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    AtDirectory,
    Completion,
    ExactAnswer,
    IntegerAnswer,
    TupleAnswer,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import Chdir, Solution

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
        - `ls -a` zobrazí všechny položky včetně skrytých
        - `ls -la` navíc zobrazí podrobnosti a typ položky

        ### Úkol
        Nacházíte se v adresáři `level-3/hub`.
        1. Použijte `ls -la` pro zobrazení všech položek i jejich typů.
        2. Spočítejte **skryté adresáře**: jejich název začíná tečkou a řádek znakem `d`.
        3. **Důležité:** Do počtu NEZAHRNUJTE speciální adresáře `.` (aktuální) a `..` (nadřazený).

        Odevzdejte počet nalezených skrytých adresářů (číslo).

        Odevzdejte pomocí: `shellgame submit <hodnota>`
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Použijte 'ls -la': -a zobrazí skryté položky a -l přidá podrobnosti.",
        "Adresář poznáte podle znaku 'd' na začátku řádku; běžný soubor začíná '-'.",
        "Počítejte skryté názvy v řádcích začínajících 'd', ale vynechte položky '.' a '..'.",
    ]
    start_directory = "hub"
    fixture = WorkspaceFixture(
        clean=("hub",),
        directories=("hub/.beta", "hub/.gamma", "hub/visible_dir"),
        files=(FileFixture("hub/.config"), FileFixture("hub/visible_file.txt")),
    )
    completion = Completion(
        answer=IntegerAnswer(
            2,
            mistakes={
                3: "Nejspíš počítáte i skrytý soubor '.config'. Jeho řádek v 'ls -la' nezačíná 'd'.",
                4: "Nejspíš počítáte i položky '.' a '..'. Jsou to speciální odkazy, ne hledané adresáře.",
                5: "Nejspíš počítáte '.', '..' i skrytý soubor. Sledujte první znak řádku a přesný název.",
            },
        )
    )
    success_message = "Správně! Skryté položky zobrazí `ls -a` a typ položky poznáte podle prvního znaku řádku `ls -l`."


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

        Odevzdejte pomocí: `shellgame submit <obsah>`
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
    success_message = "Správně! Tečka na začátku názvu soubor jen skryje ve výpisu, práci s ním nijak neomezuje."


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

        Odevzdejte pomocí: `shellgame submit <klíč>`
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
    success_message = "Správně! Do skrytého adresáře se vstupuje příkazem `cd` úplně stejně jako do viditelného."


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

        Odevzdejte pomocí: `shellgame submit <název-souboru>`
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
    success_message = "Správně! Úvodní tečka i přípona jsou součástí názvu, takže se soubor odevzdává celým jménem."


@section.level(5)
class HiddenFilesSummaryChallengeLevel(Level):
    solution = Solution(answer="2,hidden_master")
    title = "Souhrn Sekce 3"
    instructions = """
        ### Výzva: Mistři skrytých souborů

        Ukažte, že ovládáte práci se skrytými soubory!

        ### Úkol
        V adresáři `level-3/final_test` jsou normální i skryté položky.

        1. Pomocí `ls -la` spočítejte **skryté adresáře** (název začíná tečkou, řádek znakem `d`; bez `.` a `..`)
        2. Najděte skrytý soubor `.secret_code`
        3. Přečtěte jeho obsah
        4. Odevzdejte dvojici ve schématu `POČET,KÓD`

        ### Shrnutí příkazů Sekce 3
        ```
        ls -a         → Zobrazí vše včetně skrytých
        ls -la        → Přidá podrobnosti; adresář má na začátku řádku d
        cat .soubor   → Přečíst skrytý soubor
        cd .adresar   → Vstoupit do skrytého adresáře
        ```

        ### Odevzdání
        `shellgame submit <hodnota>`
        """
    hints = [
        "Skryté položky začínají tečkou. Použijte 'ls -la' pro zobrazení všech položek i jejich typů.",
        "Adresáře poznáte podle 'd' na začátku řádku; běžné soubory začínají '-'.",
        "Položky '.' a '..' nepočítejte. Obsah souboru zobrazíte pomocí 'cat .secret_code'.",
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
                        "Počet skrytých adresářů není správně. V 'ls -la' sledujte první znak řádku a název."
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
            format_message="Formát odpovědi je: POČET,KÓD",
        )
    )
    success_message = "Výborně! Dokončili jste Sekci 3. Skryté soubory před vámi nic neskryjí!"
