"""Section 5: File Inspection."""

from typing import Tuple, Any, Optional
from pathlib import Path
from shellgame.levels.base import Level
from shellgame.validation.validators import StringValidator, IntegerValidator, FileTypeValidator


def _setup_level5_common(workspace: Path) -> None:
    """Common setup for Level 5 section."""
    level_dir = workspace / "level-5"
    level_dir.mkdir(parents=True, exist_ok=True)


class Level5_0(Level):
    """Level 5.0: Section 5 Introduction."""

    def __init__(self) -> None:
        super().__init__(
            id="5.0",
            section=5,
            title="Sekce 5: Zkoumání souborů",
            instructions_file="section5_intro.md",
            hints=["Přečtěte si úvod a pokračujte stisknutím Enter."],
        )

    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        pass

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Always valid."""
        return True, "Jdeme na to!"


class Level5_1(Level):
    """Level 5.1: Reading File Sizes."""

    def __init__(self) -> None:
        super().__init__(
            id="5.1",
            section=5,
            title="Velikost souboru",
            instructions="""# Level 5.1: Velikost souboru

Příkaz `ls -l` (long listing) zobrazí podrobné informace o souborech, včetně jejich velikosti v bajtech.
Pokud chcete velikost v čitelnějším formátu (KB, MB), použijte `ls -lh` (human readable).

## Úkol:
Zjistěte přesnou velikost souboru `database.db` v bajtech.

## Příkazy:
- `ls -l`: Zobrazí detaily (velikost je pátý sloupec)

## Odevzdání:
Odevzdejte velikost souboru jako číslo.
`shellgame submit 12345`""",
            hints=[
                "Příkaz 'ls -l' zobrazí podrobnosti o souborech. Který sloupec obsahuje velikost?",
                "Ve výstupu ls -l je velikost v bajtech - hledejte číslo před datem.",
                "Použijte 'ls -l database.db' a podívejte se na pátý sloupec.",
            ],
            start_directory="level-5/sizes",
        )

    def setup(self, workspace: Path) -> None:
        """Create file with specific size."""
        _setup_level5_common(workspace)
        level_dir = workspace / "level-5" / "sizes"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Create a file with exactly 12345 bytes
        with open(level_dir / "database.db", "wb") as f:
            f.write(b"x" * 12345)

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate size."""
        if answer is None:
            return False, "Musíte zadat velikost."

        validator = IntegerValidator(12345)
        return validator.validate(answer, state.workspace)


class Level5_2(Level):
    """Level 5.2: Identifying by Size."""

    def __init__(self) -> None:
        super().__init__(
            id="5.2",
            section=5,
            title="Hledání podle velikosti",
            instructions="""# Level 5.2: Hledání podle velikosti

V adresáři je mnoho souborů, ale jen jeden má specifickou velikost.

## Úkol:
Najděte soubor, který má přesně **1337 bajtů**.

## Příkazy:
- `ls -l`: Projděte seznam a hledejte velikost 1337.

## Odevzdání:
Odevzdejte název nalezeného souboru.
`shellgame submit nazev_souboru`""",
            hints=[
                "Použijte 'ls -l' a hledejte číslo 1337.",
                "Ve výpisu hledejte řádek, kde je velikost přesně 1337 (pátý sloupec). Název souboru je na konci řádku.",
            ],
            start_directory="level-5/search",
        )

    def setup(self, workspace: Path) -> None:
        """Create multiple files with different sizes."""
        _setup_level5_common(workspace)
        level_dir = workspace / "level-5" / "search"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Distractions
        (level_dir / "file_a").write_bytes(b"x" * 1000)
        (level_dir / "file_b").write_bytes(b"x" * 2000)
        (level_dir / "file_c").write_bytes(b"x" * 1338)

        # Target
        (level_dir / "target_file").write_bytes(b"x" * 1337)

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate filename."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        validator = StringValidator("target_file")
        return validator.validate(answer, state.workspace)


class Level5_3(Level):
    """Level 5.3: File Type Detection."""

    def __init__(self) -> None:
        super().__init__(
            id="5.3",
            section=5,
            title="Typ souboru",
            instructions="""# Level 5.3: Typ souboru

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
`shellgame submit fileX`""",
            hints=[
                "Příkaz 'file' zkoumá obsah souboru, ne jeho název. Jak zjistíte typ všech souborů najednou?",
                "Zkuste 'file *' nebo 'file file1 file2 file3'. Hledejte 'JPEG' ve výstupu.",
                "Použijte 'file *' a najděte soubor označený jako 'JPEG image data'.",
            ],
            start_directory="level-5/types",
        )

    def setup(self, workspace: Path) -> None:
        """Create files with different types."""
        _setup_level5_common(workspace)
        level_dir = workspace / "level-5" / "types"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Text file
        (level_dir / "file1").write_text("This is a text file.")

        # Binary file (random data)
        (level_dir / "file2").write_bytes(b"\x00\x01\x02\x03")

        # Fake JPEG header
        (level_dir / "file3").write_bytes(b"\xff\xd8\xff\xe0\x00\x10\x4a\x46\x49\x46")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate filename."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        # We can use FileTypeValidator to verify the user picked the right one
        # But simpler is just checking the name since we know which one it is.
        # However, let's use FileTypeValidator to show off.

        # Actually, FileTypeValidator checks if the ANSWER (filename) has the type.
        # So if they answer "file3", we check if "file3" is JPEG.

        filetype_validator = FileTypeValidator("level-5/types/" + answer.strip(), "JPEG")
        # But wait, FileTypeValidator takes relative path.
        # The user provides just filename. We need to construct path.
        # But validate method of FileTypeValidator takes answer and workspace.
        # My implementation of FileTypeValidator uses answer as the path relative to workspace?
        # No, my implementation uses `workspace / answer`.
        # So if user is in `level-5/types`, and types `file3`, `workspace / "file3"` is wrong.
        # It should be `workspace / "level-5/types/file3"`.

        # The validator I wrote:
        # target_file = workspace / answer.strip()

        # This assumes answer is relative to workspace OR absolute.
        # But user thinks relative to current directory.
        # The validator doesn't know current directory of the user.
        # This is a limitation of the current Validator interface.
        # It only gets (answer, workspace).

        # So I should probably just use StringValidator("file3") for simplicity and robustness here.

        filename_validator = StringValidator("file3")
        return filename_validator.validate(answer, state.workspace)


class Level5_4(Level):
    """Level 5.4: Viewing Large Files with Less."""

    def __init__(self) -> None:
        super().__init__(
            id="5.4",
            section=5,
            title="Prohlížení velkých souborů (less)",
            instructions="""# Level 5.4: Příkaz less - prohlížeč souborů

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
/hledaný_text         → Hledat (n = další výskyt)
q                     → Ukončit
```

### Proč less a ne cat?
- `cat velky_soubor.txt` → zaplaví terminál tisíci řádky
- `less velky_soubor.txt` → klidné procházení po stránkách

## Úkol
Soubor `server.log` má 200 řádků. Najděte řádek, který obsahuje "CRITICAL".

1. Otevřete soubor: `less server.log`
2. Hledejte: stiskněte `/`, napište `CRITICAL`, Enter
3. Zjistěte, jaké číslo je na konci nalezeného řádku

## Odevzdání
Odevzdejte číslo z CRITICAL řádku.
`shellgame submit <číslo>`""",
            hints=[
                "V less použijte / pro vyhledávání. Napište /CRITICAL a stiskněte Enter.",
                "Nalezený řádek obsahuje číslo na konci. Přečtěte ho.",
                "Pozor: neodevzdáváte číslo řádku (v závorkách na začátku), ale kód na konci věty.",
            ],
            start_directory="level-5/logs",
        )

    def setup(self, workspace: Path) -> None:
        """Create large log file."""
        _setup_level5_common(workspace)
        level_dir = workspace / "level-5" / "logs"
        level_dir.mkdir(parents=True, exist_ok=True)

        lines = []
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

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate the number found."""
        if answer is None:
            return False, "Zadejte číslo z CRITICAL řádku."

        try:
            num = int(answer.strip())
        except ValueError:
            return False, "Odpověď musí být číslo."

        if num == 42:
            return True, "Správně! Less je nezbytný pro práci s velkými soubory."
        elif num == 137:
            return False, "137 je číslo řádku, ne kód na konci. Přečtěte celý CRITICAL řádek."
        else:
            return False, f"Číslo není {num}. Najděte řádek s 'CRITICAL' a přečtěte číslo na konci."


class Level5_5(Level):
    """Level 5.5: Disguised File."""

    def __init__(self) -> None:
        super().__init__(
            id="5.5",
            section=5,
            title="Zamaskovaný soubor",
            instructions="""# Level 5.5: Zamaskovaný soubor

Někdo se pokusil skrýt tajnou zprávu tím, že soubor pojmenoval jako obrázek.

## Úkol:
V adresáři `downloads` je několik souborů s příponou `.jpg`.
Jeden z nich je ale ve skutečnosti textový soubor (ASCII text). Najděte ho.

## Příkazy:
- `file *.jpg`: Zkontroluje všechny soubory s příponou .jpg

## Odevzdání:
Odevzdejte název falešného obrázku.
`shellgame submit fake.jpg`""",
            hints=["Použijte 'file *.jpg'.", "Hledejte ten, který je 'ASCII text'."],
            extension=True,
            start_directory="level-5/downloads",
        )

    def setup(self, workspace: Path) -> None:
        """Create disguised files."""
        _setup_level5_common(workspace)
        level_dir = workspace / "level-5" / "downloads"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Real JPEGs (fake content but header matches)
        (level_dir / "photo1.jpg").write_bytes(b"\xff\xd8\xff\xe0\x00\x10\x4a\x46\x49\x46")
        (level_dir / "photo2.jpg").write_bytes(b"\xff\xd8\xff\xe0\x00\x10\x4a\x46\x49\x46")

        # Fake JPEG (Text)
        (level_dir / "secret.jpg").write_text("This is actually a text file.")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate filename."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        validator = StringValidator("secret.jpg")
        return validator.validate(answer, state.workspace)


class Level5_6(Level):
    """Level 5.6: Executable Recognition."""

    def __init__(self) -> None:
        super().__init__(
            id="5.6",
            section=5,
            title="Spustitelný skript",
            instructions="""# Level 5.6: Spustitelný skript

Některé textové soubory jsou skripty, které lze spustit. Poznáte je podle toho, že příkaz `file` o nich řekne např. "Python script" nebo "Bourne-Again shell script".

## Úkol:
Najděte v adresáři `bin` soubor, který je Python skriptem.

## Odevzdání:
Odevzdejte název skriptu.
`shellgame submit script.py`""",
            hints=["Použijte 'file *' v adresáři bin.", "Hledejte 'Python script'."],
            optional=True,
            start_directory="level-5/bin",
        )

    def setup(self, workspace: Path) -> None:
        """Create scripts."""
        _setup_level5_common(workspace)
        level_dir = workspace / "level-5" / "bin"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Text file
        (level_dir / "readme.txt").write_text("Just text.")

        # Shell script
        (level_dir / "run.sh").write_text("#!/bin/bash\necho hello")

        # Python script
        (level_dir / "calc.py").write_text("#!/usr/bin/env python3\nprint(1+1)")

        # Binary
        (level_dir / "program").write_bytes(b"\x7f\x45\x4c\x46")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate filename."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        validator = StringValidator("calc.py")
        return validator.validate(answer, state.workspace)


class Level5_7(Level):
    """Level 5.7: Section 5 Summary."""

    def __init__(self) -> None:
        super().__init__(
            id="5.7",
            section=5,
            title="Souhrn Sekce 5",
            instructions="""### 🎯 Výzva: Detektiv souborů

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
`shellgame submit <text>,<img>,<script>`""",
            hints=[
                "Použijte 'file *' k zobrazení typů všech souborů najednou.",
                "Hledejte: 'ASCII text' pro textové, 'image' pro obrázky, 'Python' pro skripty.",
                "Když si nejste jistí, počítejte jen podle klíčových slov ve výstupu 'file'. Např. 'PNG image data' berte jako obrázek.",
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create mystery files."""
        _setup_level5_common(workspace)
        mystery_dir = workspace / "level-5" / "mystery"

        if mystery_dir.exists():
            import shutil

            shutil.rmtree(mystery_dir)

        mystery_dir.mkdir(parents=True, exist_ok=True)

        # Text files (2)
        (mystery_dir / "data.bin").write_text("This is just plain text.\n")
        (mystery_dir / "notes.xyz").write_text("More text content.\n")

        # Image file (1) - PNG header
        (mystery_dir / "config.txt").write_bytes(b"\x89PNG\r\n\x1a\n\x00\x00\x00")

        # Python script with misleading name (1)
        (mystery_dir / "analyzer.dat").write_text(
            "#!/usr/bin/env python3\nimport sys\nprint('Hello')\n"
        )

        # Binary file
        (mystery_dir / "readme.doc").write_bytes(b"\x7f\x45\x4c\x46\x02\x01\x01")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate answer."""
        if answer is None:
            return False, "Musíte zadat odpověď ve formátu: text_count,img_count,script_name"

        answer = answer.strip()
        parts = answer.split(",")

        if len(parts) != 3:
            return (
                False,
                "Formát: počet_textových,počet_obrázků,název_skriptu (např. 2,1,skript.py)",
            )

        try:
            text_count = int(parts[0].strip())
            img_count = int(parts[1].strip())
            script_name = parts[2].strip().lower()
        except ValueError:
            return False, "První dvě hodnoty musí být čísla."

        if text_count != 2:
            return (
                False,
                f"Počet textových souborů není {text_count}. Hledejte 'ASCII text' ve výstupu 'file *'.",
            )

        if img_count != 1:
            return False, f"Počet obrázků není {img_count}. Hledejte 'image' ve výstupu."

        if script_name != "analyzer.dat":
            return False, f"Python skript není {script_name}. Hledejte 'Python script' ve výstupu."

        return (
            True,
            "🎉 Skvělá detektivní práce! Dokončili jste Sekci 5. Příponám se už nedáte zmást!",
        )


def get_levels() -> list:
    """
    Return all Section 5 level classes.

    Returns:
        List of Level instances for Section 5
    """
    return [
        Level5_0(),
        Level5_1(),
        Level5_2(),
        Level5_3(),
        Level5_4(),
        Level5_5(),
        Level5_6(),
        Level5_7(),
    ]
