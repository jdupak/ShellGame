from __future__ import annotations

from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import Completion, ExactAnswer, IntegerAnswer, TupleAnswer
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import Solution

section = Section(5, root="level-5")


@section.level(0)
class SectionIntroLevel(Level):
    is_intro = True
    title = "Sekce 5: Zkoumání souborů"
    instructions_file = "section5_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"


@section.level(1)
class FileSizeInBytesLevel(Level):
    title = "Velikost souboru"
    instructions = """
        Příkaz `ls -l` (long listing) zobrazí podrobné informace o souborech, včetně jejich velikosti v bajtech.
        Pokud chcete velikost v čitelnějším formátu (KB, MB), použijte `ls -lh` (human readable).

        ## Úkol:
        Zjistěte přesnou velikost souboru `database.db` v bajtech.

        ## Příkazy:
        - `ls -l`: Zobrazí detaily (velikost je pátý sloupec)

        ## Odevzdání:
        Odevzdejte velikost souboru jako číslo.
        `shellgame submit <bajty>`
        """
    hints = [
        "Příkaz 'ls -l' zobrazí podrobnosti o souborech. Který sloupec obsahuje velikost?",
        "Ve výstupu ls -l je velikost v bajtech - hledejte číslo před datem.",
        "Použijte 'ls -l database.db' a podívejte se na pátý sloupec.",
    ]
    start_directory = "sizes"
    fixture = WorkspaceFixture(files=(FileFixture("sizes/database.db", b"x" * 12345),))
    completion = Completion(answer=IntegerAnswer(12345))
    success_message = "Správně! Pátý sloupec `ls -l` udává velikost v bajtech; `-h` ji převede do čitelnější podoby."


@section.level(2)
class FindFileByExactSizeLevel(Level):
    title = "Hledání podle velikosti"
    instructions = """
        V adresáři je mnoho souborů, ale jen jeden má specifickou velikost.

        ## Úkol:
        Najděte soubor, který má přesně **1337 bajtů**.

        ## Příkazy:
        - `ls -l`: Projděte seznam a hledejte velikost 1337.

        ## Odevzdání:
        Odevzdejte název nalezeného souboru.
        `shellgame submit nazev_souboru`
        """
    hints = [
        "Příkaz 'ls -l' zobrazuje podrobný výpis souborů včetně velikosti v bajtech v pátém sloupci.",
        "Spusťte 'ls -l' a hledejte řádek, kde je velikost přesně 1337.",
        "Název souboru je na konci příslušného řádku. Odevzdejte ho příkazem 'shellgame submit <soubor>'.",
    ]
    start_directory = "search"
    #: Extra practice of the 5.1 size column rather than a new skill, so a
    #: confident player may skip it.
    optional = True
    fixture = WorkspaceFixture(
        files=(
            FileFixture("search/file_a", b"x" * 1000),
            FileFixture("search/file_b", b"x" * 2000),
            FileFixture("search/file_c", b"x" * 1338),
            FileFixture("search/target_file", b"x" * 1337),
        )
    )
    completion = Completion(answer=ExactAnswer("target_file"))
    success_message = "Správně! Podrobný výpis se čte po sloupcích — stačí porovnat ten jeden, který vás zajímá."


@section.level(3)
class IdentifyJpegAmongFilesLevel(Level):
    title = "Typ souboru"
    instructions = """
        V Linuxu přípona souboru (např. `.txt`, `.jpg`) neurčuje jeho typ. O tom rozhoduje obsah.
        Příkaz `file` prozkoumá obsah souboru a řekne vám, o jaký typ se jedná.

        ## Úkol:
        V adresáři jsou tři soubory bez přípony: `file1`, `file2`, `file3`.
        Jeden z nich je obrázek (JPEG image data). Zjistěte který.

        ## Příkazy:
        - `file <soubor>`: Zjistí typ souboru
        - `file *`: Zjistí typ všech souborů v adresáři

        ## Odevzdání:
        Odevzdejte název souboru, který je obrázkem.
        `shellgame submit <soubor>`
        """
    hints = [
        "Příkaz 'file' zkoumá obsah souboru, ne jeho název. Jak zjistíte typ všech souborů najednou?",
        "Zkuste 'file *' nebo 'file file1 file2 file3'. Hledejte 'JPEG' ve výstupu.",
        "Použijte 'file *' a najděte soubor označený jako 'JPEG image data'.",
    ]
    start_directory = "types"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("types/file1", "This is a text file."),
            FileFixture("types/file2", b"\x00\x01\x02\x03"),
            FileFixture("types/file3", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
        )
    )
    completion = Completion(answer=ExactAnswer("file3"))
    success_message = "Správně! O typu souboru rozhoduje jeho obsah, ne název ani přípona."


@section.level(4)
class FindCriticalCodeInLogLevel(Level):
    solution = Solution(answer="42")
    title = "Hledání ve velkém souboru"
    instructions = """
        Příkaz `cat` vypíše celý soubor najednou, což je u velkých souborů nepraktické.
        Pro procházení a hledání informací ve velkém souboru se hodí interaktivní nástroj `less`.

        ### Ovládání less
        ```
        Mezerník / Page Down  → O stránku dolů
        b / Page Up           → O stránku nahoru
        j / šipka dolů        → O řádek dolů
        k / šipka nahoru      → O řádek nahoru
        g                     → Na začátek souboru
        G                     → Na konec souboru
        /hledany_text         → Hledat (n = další výskyt)
        q                     → Ukončit
        ```

        ## Úkol
        Soubor `server.log` má 200 řádků. Najděte v něm informaci o chybovém kódu
        na řádku obsahujícím `CRITICAL`. Doporučený nástroj je `less`.

        1. Otevřete soubor: `less server.log`
        2. Hledejte: stiskněte `/`, napište `CRITICAL` a potvrďte Enterem
        3. Přečtěte chybový kód (číslo za `Code`) na konci nalezeného řádku
        4. Stisknutím `q` ukončete `less` a teprve potom odešlete odpověď

        ## Odevzdání
        Odevzdejte chybový kód z CRITICAL řádku.
        `shellgame submit <číslo>`
        """
    hints = [
        "V less použijte / pro vyhledávání. Napište /CRITICAL a stiskněte Enter.",
        "Nalezený řádek obsahuje číslo na konci. Přečtěte ho.",
        "Pozor: neodevzdáváte číslo řádku (v závorkách na začátku), ale kód na konci věty.",
    ]
    start_directory = "logs"
    completion = Completion(
        answer=IntegerAnswer(
            42,
            mistakes={137: "137 je číslo řádku, ne kód na konci. Přečtěte celý CRITICAL řádek."},
            error_message="Tohle není správný kód. Najděte řádek s 'CRITICAL' a přečtěte číslo na konci.",
            invalid_message="Odpověď musí být číslo.",
        )
    )
    success_message = "Správně! Less usnadňuje hledání ve velkých souborech."

    @override
    def setup(self, workspace: Path) -> None:
        lines: list[str] = []
        for i in range(1, 201):
            if i == 137:
                lines.append(f"[{i:03d}] CRITICAL: System failure detected - Code 42")
            elif i % 10 == 0:
                lines.append(f"[{i:03d}] WARNING: High memory usage")
            elif i % 7 == 0:
                lines.append(f"[{i:03d}] ERROR: Connection timeout")
            else:
                lines.append(f"[{i:03d}] INFO: Normal operation")

        FileFixture("logs/server.log", "\n".join(lines) + "\n").apply(self.section_path(workspace))


@section.level(5)
class FindFakeJpgLevel(Level):
    title = "Zamaskovaný soubor"
    instructions = """
        Někdo se pokusil skrýt tajnou zprávu tím, že soubor pojmenoval jako obrázek.

        ## Úkol:
        V adresáři `downloads` je několik souborů s příponou `.jpg`.
        Jeden z nich je ale ve skutečnosti textový soubor (ASCII text). Najděte ho.

        ## Příkazy:
        - `file *.jpg`: Zkontroluje všechny soubory s příponou .jpg

        ## Odevzdání:
        Odevzdejte název falešného obrázku.
        `shellgame submit <soubor>`
        """
    hints = [
        "Příkaz 'file' určí skutečný typ souboru bez ohledu na jeho příponu.",
        "Spusťte 'file *.jpg' a prozkoumejte typy jednotlivých souborů.",
        "Hledejte soubor, u kterého je uvedeno 'ASCII text'. Jeho název odevzdejte.",
    ]
    start_directory = "downloads"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("downloads/photo1.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/photo2.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/secret.jpg", "This is actually a text file."),
        )
    )
    completion = Completion(answer=ExactAnswer("secret.jpg"))
    success_message = "Správně! Přípona je jen dohoda — `file` čte skutečný obsah, a tak odhalí i zamaskovaný soubor."


@section.level(6)
class IdentifyPythonScriptLevel(Level):
    title = "Rozpoznání Python skriptu"
    instructions = """
        Příkaz `file` dokáže podle obsahu rozpoznat Python skript, i když má neobvyklý název.
        Popis `Python script` ale neříká, zda má soubor právo ke spuštění (`x`). Typ zjišťuje `file`,
        oprávnění zobrazuje `ls -l` a mění `chmod` — tomu se věnuje Sekce 8.

        ## Úkol:
        Najděte v adresáři `bin` soubor, který příkaz `file` označí jako Python skript.

        ## Odevzdání:
        Odevzdejte název skriptu.
        `shellgame submit <soubor>`
        """
    hints = [
        "Příkaz 'file *' vypíše typ pro všechny soubory v aktuálním adresáři.",
        "Spusťte 'file *' a hledejte soubor, u kterého výstup uvádí 'Python script'.",
        "Odevzdejte název souboru označeného jako 'Python script'; samotný popis neověřuje právo x.",
    ]
    start_directory = "bin"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("bin/readme.txt", "Just text."),
            FileFixture("bin/run.sh", "#!/bin/bash\necho hello"),
            FileFixture("bin/calc.py", "#!/usr/bin/env python3\nprint(1+1)\n"),
            FileFixture("bin/program", b"\x7fELF"),
        )
    )
    completion = Completion(
        answer=ExactAnswer(
            "calc.py",
            mistakes={
                "readme.txt": "To je prostý text, ne skript. Řiďte se přesným popisem z `file *`, ne příponou názvu.",
                "run.sh": "To je shellový skript (`shell script`), ne Python. Přečtěte popis z `file *` celý.",
                "program": "To je zkompilovaný program (ELF), ne skript. Porovnejte popisy z `file *`.",
            },
            error_message=(
                "To není Python skript. Rozhoduje přesný popis z `file *`; "
                "právo `x` z něj nevyčtete — to ukazuje až `ls -l`."
            ),
        )
    )
    success_message = "Správně! `file` určí typ souboru podle obsahu, ale právo ke spuštění ukáže až `ls -l`."


@section.level(7)
class FileDetectiveChallengeLevel(Level):
    title = "Souhrn Sekce 5"
    instructions = """
        ### Výzva: Detektiv souborů

        Ukažte, že umíte identifikovat typy souborů!

        ### Úkol
        V `level-5/mystery` je 5 souborů s podivnými názvy.
        Zjistěte typ každého a odpovězte na otázky:

        1. Kolik je tam **prostých textových** souborů (popis začíná `ASCII text`, bez skriptů)?
        2. Kolik je tam **obrázků** (image)?
        3. Jaký je název jediného **Python** skriptu (bez cesty)?

        ### Formát odpovědi
        `<text_count>,<image_count>,<script_name>`

        Skript počítejte samostatně, i když jeho popis také obsahuje `ASCII text`.

        ### Shrnutí příkazů Sekce 5
        ```
        file soubor     → Zjistí typ souboru
        file *          → Typy všech souborů
        file -b soubor  → Jen typ bez názvu
        ```

        ### Odevzdání
        `shellgame submit <text>,<img>,<script>`
        """
    hints = [
        "Použijte 'file *' k zobrazení typů všech souborů najednou.",
        "Pro prostý text hledejte popis začínající 'ASCII text'. Skripty do tohoto počtu nepatří.",
        ("Popis s 'image' znamená obrázek. Popis s 'Python script' znamená Python skript, ne prostý text."),
    ]
    start_directory = "mystery"
    fixture = WorkspaceFixture(
        clean=("mystery",),
        files=(
            FileFixture("mystery/data.bin", "This is just plain text.\n"),
            FileFixture("mystery/notes.xyz", "More text content.\n"),
            FileFixture(
                "mystery/config.txt",
                bytes.fromhex(
                    "89504e470d0a1a0a0000000d4948445200000001000000010802000000907753de"
                    "0000000c49444154789c63606060000000040001f61738550000000049454e44ae426082"
                ),
            ),
            FileFixture(
                "mystery/analyzer.dat",
                "#!/usr/bin/env python3\nimport sys\nprint('Hello')\n",
            ),
            FileFixture("mystery/readme.doc", b"\x7fELF\x02\x01\x01"),
        ),
    )
    completion = Completion(
        answer=TupleAnswer(
            (
                IntegerAnswer(
                    2,
                    error_message=(
                        "Počet prostých textových souborů není správně. "
                        "Počítejte popisy začínající 'ASCII text', nikoli skripty."
                    ),
                    invalid_message="První dvě hodnoty musí být čísla.",
                ),
                IntegerAnswer(
                    1,
                    error_message="Počet obrázků není správně. Hledejte 'image' ve výstupu 'file *'.",
                    invalid_message="První dvě hodnoty musí být čísla.",
                ),
                ExactAnswer(
                    "analyzer.dat",
                    case_sensitive=False,
                    error_message="Název Python skriptu není správně. Hledejte 'Python' ve výstupu 'file *'.",
                ),
            ),
            format_message="Formát: počet_textových,počet_obrázků,název_skriptu",
        )
    )
    success_message = "Skvělá detektivní práce! Dokončili jste Sekci 5. Příponám se už nedáte zmást!"
