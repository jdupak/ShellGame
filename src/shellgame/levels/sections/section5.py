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
    solution = Solution(answer="12345")
    title = "Velikost souboru"
    instructions = """
        ### Cíl
        Zjistěte přesnou velikost souboru v bajtech.

        Příkaz `ls -l` (long listing) zobrazí podrobné informace o souborech včetně jejich
        velikosti v bajtech (5. sloupec). Přepínač `-h` ji převede do čitelnější podoby (KB, MB).

        ### Příkazy
        - `ls -l` (zobrazí detaily; velikost je v pátém sloupci)

        ### Úkol
        1. V aktuálním adresáři je soubor `database.db`.
        2. Pomocí `ls -l` zjistěte jeho přesnou velikost v bajtech.
        3. Odevzdejte tuto velikost jako číslo.

        ### Odevzdání
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
    solution = Solution(answer="file_d")
    title = "Hledání podle velikosti"
    instructions = """
        ### Cíl
        Najděte v podrobném výpisu soubor s konkrétní velikostí.

        ### Příkazy
        - `ls -l` (zobrazí podrobnosti včetně velikosti v 5. sloupci)

        ### Úkol
        V adresáři je několik souborů (`file_a`, `file_b`, `file_c`, `file_d`).
        Pouze jeden z nich má velikost přesně **1337 bajtů**.

        1. Spusťte `ls -l` a zkontrolujte sloupec s velikostí.
        2. Najděte soubor s velikostí přesně 1337 bajtů.
        3. Odevzdejte název tohoto souboru.

        ### Odevzdání
        `shellgame submit <název_souboru>`
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
            FileFixture("search/file_d", b"x" * 1337),
        )
    )
    completion = Completion(
        answer=ExactAnswer(
            "file_d",
            mistakes={
                "file_c": "Soubor 'file_c' má 1338 bajtů (o 1 bajt více). Hledejte přesně 1337 bajtů.",
                "file_a": "Soubor 'file_a' má 1000 bajtů. Hledejte přesně 1337 bajtů.",
                "file_b": "Soubor 'file_b' má 2000 bajtů. Hledejte přesně 1337 bajtů.",
            },
        )
    )
    success_message = "Správně! Podrobný výpis se čte po sloupcích — stačí porovnat ten jeden, který vás zajímá."


@section.level(3)
class IdentifyJpegAmongFilesLevel(Level):
    solution = Solution(answer="file3")
    title = "Typ souboru"
    instructions = """
        ### Cíl
        Rozpoznejte obrázek mezi soubory bez přípony podle jejich obsahu.

        V Linuxu přípona souboru (např. `.txt`, `.jpg`) neurčuje jeho typ. O tom rozhoduje obsah.
        Příkaz `file` prozkoumá obsah souboru a řekne vám, o jaký typ se jedná.

        ### Příkazy
        - `file <soubor>` (zjistí typ konkrétního souboru)
        - `file *` (zjistí typ všech souborů v aktuálním adresáři)

        ### Úkol
        V adresáři jsou tři soubory bez přípony: `file1`, `file2`, `file3`.
        Jeden z nich je obrázek (`JPEG image data`). Zjistěte který a odevzdejte jeho název.

        ### Odevzdání
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
        ### Cíl
        Naučte se prohlížet a prohledávat velké soubory pomocí interaktivního nástroje `less`.

        Příkaz `cat` vypíše celý soubor najednou, což je u dlouhých souborů nepraktické.
        Interaktivní stránkovač `less` umožní souborem pohodlně listovat a hledat v něm.

        ### 🚪 Jak z less odejít: klávesa `q` (Quit)
        Nástroj `less` zabere celou obrazovku terminálu. Během jeho běhu nelze zadávat
        běžné příkazy shellu.
        **Pro ukončení prohlížeče a návrat do příkazové řádky stiskněte klávesu `q`.**

        ### Ovládání less
        ```
        q                     → UKONČIT prohlížení (návrat do shellu)
        /hledany_text         → Hledat text v souboru (potvrdit Enterem)
        n                     → Další výskyt hledaného textu
        Enter / šipka dolů    → Posun o jeden řádek dolů
        Mezerník / Page Down  → Posun o celou stránku dolů
        b / Page Up           → Posun o stránku nahoru
        šipka nahoru          → Posun o řádek nahoru
        g / G                 → Na začátek / na konec souboru
        ```

        ### Úkol
        Soubor `server.log` má 200 řádků. Najděte v něm chybový kód na řádku s textem `CRITICAL`:

        1. Otevřete soubor: `less server.log`
        2. Hledejte: stiskněte `/`, napište `CRITICAL` a stiskněte **Enter**
        3. Přečtěte chybový kód (číslo za slovem `Code`) na konci nalezeného řádku
        4. **Stiskněte klávesu `q`** pro ukončení `less` a návrat do příkazové řádky
        5. Odešlete nalezený kód pomocí `shellgame submit`

        ### Odevzdání
        `shellgame submit <číslo>`
        """
    hints = [
        "V less použijte / pro vyhledávání. Napište /CRITICAL a stiskněte Enter.",
        "Nalezený řádek obsahuje číslo na konci. Přečtěte ho.",
        "Pro návrat do shellu stiskněte klávesu 'q' (quit).",
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
    solution = Solution(answer="secret.jpg")
    title = "Zamaskovaný soubor"
    instructions = """
        ### Cíl
        Najděte soubor s příponou `.jpg`, který ve skutečnosti není obrázkem.

        V adresáři `downloads` je mnoho různých stažených souborů (dokumenty, archivy,
        skripty i logy). Někdo se pokusil ukrýt tajnou textovou zprávu tím, že ji pojmenoval
        s příponou `.jpg`.

        ### Příkazy
        - `file *.jpg` (zkontroluje pouze soubory s příponou .jpg místo všech položek)

        ### Úkol
        1. V adresáři `downloads` je mnoho různých souborů. Spuštění `file *` by vypsalo
           desítky položek bez užitku.
        2. Pomocí `file *.jpg` zkontrolujte pouze soubory s příponou `.jpg`.
        3. Najděte, který soubor je ve skutečnosti textový (`ASCII text`).
        4. Odevzdejte název tohoto souboru.

        ### Odevzdání
        `shellgame submit <soubor>`
        """
    hints = [
        "Příkaz 'file' zkoumá skutečný obsah souboru bez ohledu na jeho příponu.",
        "Místo 'file *' použijte 'file *.jpg' — zkontrolujete jen soubory s příponou .jpg.",
        "Hledejte mezi soubory .jpg ten, u kterého 'file' vypíše 'ASCII text'. Odevzdejte jeho název.",
    ]
    start_directory = "downloads"
    fixture = WorkspaceFixture(
        files=(
            # Real JPEGs
            FileFixture("downloads/photo1.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/photo2.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/photo3.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/photo4.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/photo5.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/vacation.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/sunset.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/wallpaper.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/avatar.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            FileFixture("downloads/nature.jpg", b"\xff\xd8\xff\xe0\x00\x10JFIF"),
            # Disguised secret text file
            FileFixture("downloads/secret.jpg", "This is actually a secret text file.\n"),
            # Smoke files of various types (documents, archives, text, data)
            FileFixture("downloads/manual.pdf", b"%PDF-1.4\n%EOF\n"),
            FileFixture("downloads/archive.zip", b"PK\x03\x04\x14\x00\x00\x00\x08\x00"),
            FileFixture("downloads/backup.tar.gz", b"\x1f\x8b\x08\x00\x00\x00\x00\x00"),
            FileFixture("downloads/music.mp3", b"ID3\x03\x00\x00\x00"),
            FileFixture("downloads/notes.txt", "Random notes and meeting minutes.\n"),
            FileFixture("downloads/install.sh", "#!/bin/sh\necho 'installing...'\n"),
            FileFixture("downloads/setup.log", "[INFO] Setup initialized.\n"),
            FileFixture("downloads/table.csv", "id,name,value\n1,alpha,100\n"),
            FileFixture("downloads/data.json", '{"status": "ok", "count": 42}\n'),
            FileFixture("downloads/config.yaml", "env: production\ndebug: false\n"),
            FileFixture("downloads/readme.md", "# Downloads Readme\nJust downloads.\n"),
            FileFixture("downloads/styles.css", "body { margin: 0; padding: 0; }\n"),
            FileFixture("downloads/index.html", "<!DOCTYPE html><html><body>Test</body></html>\n"),
            FileFixture("downloads/script.py", "#!/usr/bin/env python3\nprint('hello')\n"),
            FileFixture(
                "downloads/checksums.sha256", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\n"
            ),
            FileFixture("downloads/report.doc", b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1"),
            FileFixture("downloads/app.bin", b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00"),
        )
    )
    completion = Completion(answer=ExactAnswer("secret.jpg"))
    success_message = "Správně! Přípona je jen dohoda — `file` čte skutečný obsah, a tak odhalí i zamaskovaný soubor."


@section.level(6)
class IdentifyPythonScriptLevel(Level):
    solution = Solution(answer="calc.py")
    title = "Rozpoznání Python skriptu"
    instructions = """
        ### Cíl
        Rozpoznejte Python skript mezi různými typy souborů.

        Příkaz `file` dokáže podle obsahu spolehlivě rozpoznat Python skript i bez ohledu na jeho název.
        Popis `Python script` ale neříká, zda má soubor právo ke spuštění (`x`) — typ zjišťuje `file`,
        kdežto oprávnění zobrazuje `ls -l` a mění `chmod` (tomu se věnuje Sekce 8).

        ### Příkazy
        - `file *` (vypíše typy všech souborů v aktuálním adresáři)

        ### Úkol
        Nacházíte se přímo v adresáři `bin` (nikam nemusíte přecházet):
        1. Prozkoumejte soubory v aktuálním adresáři pomocí `file *`.
        2. Najděte soubor, který příkaz `file` označí jako `Python script`.
        3. Odevzdejte název tohoto souboru.

        ### Odevzdání
        `shellgame submit <soubor>`
        """
    hints = [
        "Jste přímo v adresáři 'bin'. Spusťte 'file *' pro zjištění typů všech souborů.",
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
