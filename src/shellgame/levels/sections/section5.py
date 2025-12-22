from __future__ import annotations

import shutil
from pathlib import Path

from shellgame.levels.base import Level
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    IntegerValidator,
    StringValidator,
    ValidationResult,
)


def _ensure_section_dir(workspace: Path) -> Path:
    """Ensure `level-5/` exists and return it."""
    section_dir = workspace / "level-5"
    section_dir.mkdir(parents=True, exist_ok=True)
    return section_dir


class SectionIntroLevel(Level):
    title = "Sekce 5: Zkoumání souborů"
    instructions_file = "section5_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"

    def setup(self, workspace: Path) -> None:
        pass

    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


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
        `shellgame submit 12345`
        """
    hints = [
        "Příkaz 'ls -l' zobrazí podrobnosti o souborech. Který sloupec obsahuje velikost?",
        "Ve výstupu ls -l je velikost v bajtech - hledejte číslo před datem.",
        "Použijte 'ls -l database.db' a podívejte se na pátý sloupec.",
    ]
    start_directory = "level-5/sizes"
    require_answer = True
    validators = [IntegerValidator(12345)]

    def setup(self, workspace: Path) -> None:
        _ensure_section_dir(workspace)
        level_dir = workspace / "level-5" / "sizes"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Recreate deterministically
        (level_dir / "database.db").write_bytes(b"x" * 12345)


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
        "Použijte 'ls -l' a hledejte číslo 1337.",
        ("Ve výpisu hledejte řádek, kde je velikost přesně 1337 (pátý sloupec). Název souboru je na konci řádku."),
    ]
    start_directory = "level-5/search"
    require_answer = True
    validators = [StringValidator("target_file")]

    def setup(self, workspace: Path) -> None:
        _ensure_section_dir(workspace)
        level_dir = workspace / "level-5" / "search"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Distractions
        (level_dir / "file_a").write_bytes(b"x" * 1000)
        (level_dir / "file_b").write_bytes(b"x" * 2000)
        (level_dir / "file_c").write_bytes(b"x" * 1338)

        # Target
        (level_dir / "target_file").write_bytes(b"x" * 1337)


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
        `shellgame submit fileX`
        """
    hints = [
        "Příkaz 'file' zkoumá obsah souboru, ne jeho název. Jak zjistíte typ všech souborů najednou?",
        "Zkuste 'file *' nebo 'file file1 file2 file3'. Hledejte 'JPEG' ve výstupu.",
        "Použijte 'file *' a najděte soubor označený jako 'JPEG image data'.",
    ]
    start_directory = "level-5/types"
    require_answer = True
    validators = [StringValidator("file3")]

    def setup(self, workspace: Path) -> None:
        _ensure_section_dir(workspace)
        level_dir = workspace / "level-5" / "types"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "file1").write_text("This is a text file.")
        (level_dir / "file2").write_bytes(b"\x00\x01\x02\x03")
        # Minimal JPEG header
        (level_dir / "file3").write_bytes(b"\xff\xd8\xff\xe0\x00\x10JFIF")


class FindCriticalCodeInLogLevel(Level):
    title = "Prohlížení velkých souborů (less)"
    instructions = """
        Příkaz `cat` vypíše celý soubor najednou. U velkých souborů to není praktické!
        Příkaz `less` umožňuje procházet soubor interaktivně.

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
        Soubor `server.log` má 200 řádků. Najděte řádek, který obsahuje "CRITICAL".

        1. Otevřete soubor: `less server.log`
        2. Hledejte: stiskněte `/`, napište `CRITICAL`, Enter
        3. Zjistěte, jaké číslo je na konci nalezeného řádku

        ## Odevzdání
        Odevzdejte číslo z CRITICAL řádku.
        `shellgame submit <číslo>`
        """
    hints = [
        "V less použijte / pro vyhledávání. Napište /CRITICAL a stiskněte Enter.",
        "Nalezený řádek obsahuje číslo na konci. Přečtěte ho.",
        "Pozor: neodevzdáváte číslo řádku (v závorkách na začátku), ale kód na konci věty.",
    ]
    start_directory = "level-5/logs"
    require_answer = True
    # We need custom feedback (not just IntegerValidator) due to common mistake "137".

    def setup(self, workspace: Path) -> None:
        _ensure_section_dir(workspace)
        level_dir = workspace / "level-5" / "logs"
        level_dir.mkdir(parents=True, exist_ok=True)

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

        (level_dir / "server.log").write_text("\n".join(lines) + "\n")

    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        ok, msg = super().validate(answer, state)
        if not ok:
            return ok, msg

        assert answer is not None
        try:
            num = int(answer.strip())
        except ValueError:
            return False, "Odpověď musí být číslo."

        if num == 42:
            return True, "Správně! Less je nezbytný pro práci s velkými soubory."
        if num == 137:
            return False, "137 je číslo řádku, ne kód na konci. Přečtěte celý CRITICAL řádek."
        return False, "Tohle není správný kód. Najděte řádek s 'CRITICAL' a přečtěte číslo na konci."


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
        `shellgame submit fake.jpg`
        """
    hints = ["Použijte 'file *.jpg'.", "Hledejte ten, který je 'ASCII text'."]
    extension = True
    start_directory = "level-5/downloads"
    require_answer = True
    validators = [StringValidator("secret.jpg")]

    def setup(self, workspace: Path) -> None:
        _ensure_section_dir(workspace)
        level_dir = workspace / "level-5" / "downloads"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "photo1.jpg").write_bytes(b"\xff\xd8\xff\xe0\x00\x10JFIF")
        (level_dir / "photo2.jpg").write_bytes(b"\xff\xd8\xff\xe0\x00\x10JFIF")
        (level_dir / "secret.jpg").write_text("This is actually a text file.")


class IdentifyPythonScriptLevel(Level):
    title = "Spustitelný skript"
    instructions = """
        Některé textové soubory jsou skripty, které lze spustit.
        Poznáte je podle toho, že příkaz `file` o nich řekne např. "Python script" nebo "Bourne-Again shell script".

        ## Úkol:
        Najděte v adresáři `bin` soubor, který je Python skriptem.

        ## Odevzdání:
        Odevzdejte název skriptu.
        `shellgame submit script.py`
        """
    hints = ["Použijte 'file *' v adresáři bin.", "Hledejte 'Python script'."]
    optional = True
    start_directory = "level-5/bin"
    require_answer = True
    validators = [StringValidator("calc.py")]

    def setup(self, workspace: Path) -> None:
        _ensure_section_dir(workspace)
        level_dir = workspace / "level-5" / "bin"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "readme.txt").write_text("Just text.")
        (level_dir / "run.sh").write_text("#!/bin/bash\necho hello")
        (level_dir / "calc.py").write_text("#!/usr/bin/env python3\nprint(1+1)\n")
        (level_dir / "program").write_bytes(b"\x7fELF")


class FileDetectiveChallengeLevel(Level):
    title = "Souhrn Sekce 5"
    instructions = """
        ### Výzva: Detektiv souborů

        Ukažte, že umíte identifikovat typy souborů!

        ### Úkol
        V `level-5/mystery` je 5 souborů s podivnými názvy.
        Zjistěte typ každého a odpovězte na otázky:

        1. Kolik je tam **textových** souborů (ASCII text)?
        2. Kolik je tam **obrázků** (image)?
        3. Jaký je název jediného **Python** skriptu (bez cesty)?

        ### Formát odpovědi
        `<text_count>,<image_count>,<script_name>`

        Příklad: `2,1,mujskript.py`

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
        "Hledejte: 'ASCII text' pro textové, 'image' pro obrázky, 'Python' pro skripty.",
        (
            "Když si nejste jistí, počítejte jen podle klíčových slov ve výstupu 'file'. "
            "Např. 'PNG image data' berte jako obrázek."
        ),
    ]
    require_answer = True

    def setup(self, workspace: Path) -> None:
        _ensure_section_dir(workspace)
        mystery_dir = workspace / "level-5" / "mystery"

        if mystery_dir.exists():
            shutil.rmtree(mystery_dir)
        mystery_dir.mkdir(parents=True, exist_ok=True)

        # Text files (2)
        (mystery_dir / "data.bin").write_text("This is just plain text.\n")
        (mystery_dir / "notes.xyz").write_text("More text content.\n")

        # Image file (1) - minimal PNG header
        (mystery_dir / "config.txt").write_bytes(b"\x89PNG\r\n\x1a\n\x00\x00\x00")

        # Python script (1) with misleading name
        (mystery_dir / "analyzer.dat").write_text("#!/usr/bin/env python3\nimport sys\nprint('Hello')\n")

        # Binary file (not counted)
        (mystery_dir / "readme.doc").write_bytes(b"\x7fELF\x02\x01\x01")

    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        ok, msg = super().validate(answer, state)
        if not ok:
            return ok, msg

        assert answer is not None

        success = False
        message = "Formát: počet_textových,počet_obrázků,název_skriptu (např. 2,1,skript.py)"

        parts = [p.strip() for p in answer.strip().split(",")]
        if len(parts) == 3:
            try:
                text_count = int(parts[0])
                img_count = int(parts[1])
                script_name = parts[2].strip().lower()
            except ValueError:
                message = "První dvě hodnoty musí být čísla."
            else:
                if text_count != 2:
                    message = "Počet textových souborů není správně. Hledejte 'ASCII text' ve výstupu 'file *'."
                elif img_count != 1:
                    message = "Počet obrázků není správně. Hledejte 'image' ve výstupu 'file *'."
                elif script_name != "analyzer.dat":
                    message = "Název Python skriptu není správně. Hledejte 'Python' ve výstupu 'file *'."
                else:
                    success = True
                    message = "Skvělá detektivní práce! Dokončili jste Sekci 5. Příponám se už nedáte zmást!"

        return success, message


def get_levels() -> list[Level]:
    levels: list[Level] = [
        SectionIntroLevel(),
        FileSizeInBytesLevel(),
        FindFileByExactSizeLevel(),
        IdentifyJpegAmongFilesLevel(),
        FindCriticalCodeInLogLevel(),
        FindFakeJpgLevel(),
        IdentifyPythonScriptLevel(),
        FileDetectiveChallengeLevel(),
    ]

    section_num = 5
    for i, level in enumerate(levels):
        level.section = section_num
        level.id = f"{section_num}.{i}"

    return levels
