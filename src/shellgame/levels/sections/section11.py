"""Section 11: Searching."""

import shutil
from pathlib import Path
from typing import Any, Optional

from shellgame.levels.base import Level
from shellgame.validation.validators import (
    StringValidator,
)


class Level11_0(Level):
    """Level 11.0: Section 11 Introduction."""

    def __init__(self) -> None:
        super().__init__(
            id="11.0",
            section=11,
            title="Sekce 11: Vyhledávání",
            instructions_file="section11_intro.md",
            hints=["Přečtěte si úvod a pokračujte stisknutím Enter."],
        )

    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        pass

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Always valid."""
        return True, "Jdeme na to!"


class Level11_1(Level):
    """Level 11.1: Grep."""

    def __init__(self) -> None:
        super().__init__(
            id="11.1",
            section=11,
            title="Hledání v souboru (grep)",
            instructions="""# Level 11.1: Hledání v souboru (grep)

Příkaz `grep` hledá řádky obsahující zadaný vzor.
Je to jeden z nejužitečnějších nástrojů pro práci s textem.

### Proč je to užitečné
Představte si konfigurační soubor s tisíci řádky.
Místo ručního hledání použijete grep a okamžitě najdete, co potřebujete.

### Úkol
Najděte řádek obsahující "PASSWORD" v souboru `config.txt` a uložte ho do `pass.txt`.

### Příkazy
- `grep "vzor" soubor` - hledá vzor v souboru
- `grep "vzor" soubor > výstup` - uloží nalezené řády

### Odevzdání
Odevzdejte název vytvořeného souboru.
`shellgame submit -f pass.txt`""",
            hints=[
                "Příkaz 'grep' hledá text v souborech.",
                "Syntaxe: grep 'hledaný_text' soubor.",
                "Zkuste 'grep \"heslo\" secret.txt'.",
            ],
            start_directory="level-11/searching",
        )

    def setup(self, workspace: Path) -> None:
        """Create config file."""
        level_dir = workspace / "level-11" / "grep"
        level_dir.mkdir(parents=True, exist_ok=True)

        content = "user=admin\nhost=localhost\nport=8080\nPASSWORD=Secret123\ndebug=true\n"
        (level_dir / "config.txt").write_text(content)

        if (level_dir / "pass.txt").exists():
            (level_dir / "pass.txt").unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate grep output."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        str_val = StringValidator("pass.txt")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        target = state.workspace / "level-11/grep/pass.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text().strip()
        if "PASSWORD=Secret123" in content:
            return True, "Správně!"
        else:
            return False, "Soubor neobsahuje hledaný řádek."


class Level11_2(Level):
    """Level 11.2: Recursive Grep."""

    def __init__(self) -> None:
        super().__init__(
            id="11.2",
            section=11,
            title="Rekurzivní hledání",
            instructions="""# Level 11.2: Rekurzivní hledání v souborech

Přepínač `-r` (recursive) umožňuje hledat ve všech souborech v adresáři a jeho podadresářích.

### Proč je to užitečné
Při práci s projektem často nevíte, ve kterém souboru je hledaný text.
S `grep -r` prohledáte celý projekt najednou.

### Úkol
Najděte, který soubor v adresáři `project` obsahuje text "SECRET_KEY".
Odevzdejte relativní cestu k nalezenému souboru (např. `project/složka/soubor`).

### Příkazy
- `grep -r "vzor" adresář/` - rekurzivní hledání

### Odevzdání
Odevzdejte cestu k souboru.
`shellgame submit -f project/config/settings.py`""",
            hints=[
                "Přepínač '-r' hledá rekurzivně v adresářích.",
                "Prohledá všechny soubory ve všech podadresářích.",
                "Zkuste 'grep -r \"TODO\" project/'.",
            ],
            start_directory="level-11/searching",
        )

    def setup(self, workspace: Path) -> None:
        """Create project structure."""
        level_dir = workspace / "level-11" / "recursive"
        level_dir.mkdir(parents=True, exist_ok=True)

        project = level_dir / "project"
        project.mkdir(exist_ok=True)
        (project / "src").mkdir(exist_ok=True)
        (project / "config").mkdir(exist_ok=True)

        (project / "README.md").write_text("# Project\n")
        (project / "src" / "main.py").write_text("print('Hello')\n")
        (project / "config" / "settings.py").write_text("SECRET_KEY = 'xyz'\nDEBUG = True\n")

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate found file."""
        if answer is None:
            return False, "Zadejte cestu k nalezenému souboru."

        # Allow both full relative path or just from project root if they cd'd
        # But instructions say "relative path to the file"
        expected = "project/config/settings.py"

        if answer.strip().endswith(expected):
            return True, "Správně!"
        else:
            return False, "To není správný soubor. Hledáme ten s 'SECRET_KEY'."


class Level11_3(Level):
    """Level 11.3: Case-Insensitive Grep."""

    def __init__(self) -> None:
        super().__init__(
            id="11.3",
            section=11,
            title="Hledání bez ohledu na velikost písmen",
            instructions="""# Level 11.3: Grep -i (case-insensitive)

Přepínač `-i` ignoruje velikost písmen při hledání.

### Proč je to důležité?
```bash
# Bez -i:
grep "error" log.txt      # Najde: "error occurred"
                          # Nenajde: "ERROR: failed", "Error: timeout"

# S -i:
grep -i "error" log.txt   # Najde VŠECHNY: error, ERROR, Error, eRrOr...
```

### Běžná chyba začátečníků
Hledáte "TODO" v kódu, ale autor napsal "todo" nebo "Todo"?
Přepínač `-i` zachrání situaci!

## Úkol
V souboru `messages.log` jsou zprávy s různou velikostí písmen.
Najděte VŠECHNY řádky obsahující slovo "warning" (bez ohledu na velikost).
Spočítejte je.

## Příkazy
- `grep -i "vzor" soubor` - hledá bez ohledu na velikost písmen
- `grep -i "vzor" soubor | wc -l` - spočítá nalezené řády

## Odevzdání
Odevzdejte počet nalezených řádků.
`shellgame submit -f <číslo>`""",
            hints=[
                "Bez -i byste našli jen některé. S -i najdete warning, WARNING, Warning...",
                'Použijte: grep -i "warning" messages.log | wc -l',
                "V logu jsou 4 řádky s různými variantami slova 'warning'.",
            ],
            start_directory="level-11/searching",
        )

    def setup(self, workspace: Path) -> None:
        """Create log file with mixed case."""
        level_dir = workspace / "level-11" / "case"
        level_dir.mkdir(parents=True, exist_ok=True)

        log_content = """2024-01-01 10:00:00 INFO: Server started
2024-01-01 10:05:00 WARNING: Memory usage high
2024-01-01 10:10:00 ERROR: Connection lost
2024-01-01 10:15:00 warning: Disk space low
2024-01-01 10:20:00 INFO: User logged in
2024-01-01 10:25:00 Warning: CPU temperature elevated
2024-01-01 10:30:00 DEBUG: Cache cleared
2024-01-01 10:35:00 WARNING: Network latency detected
2024-01-01 10:40:00 INFO: Backup completed
"""
        (level_dir / "messages.log").write_text(log_content)

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate warning count."""
        if answer is None:
            return False, "Zadejte počet nalezených řádků."

        try:
            count = int(answer.strip())
        except ValueError:
            return False, "Odpověď musí být číslo."

        messages = {
            4: (True, "Správně! Přepínač -i je nepostradatelný pro robustní hledání."),
            2: (
                False,
                "Našli jste jen 'WARNING'. Použijte -i pro nalezení všech variant (warning, Warning...).",
            ),
            1: (False, "Našli jste jen jednu variantu. Přepínač -i ignoruje velikost písmen."),
        }

        if count in messages:
            return messages[count]

        return False, f'Počet není {count}. Zkuste: grep -i "warning" messages.log | wc -l'


class Level11_4(Level):
    """Level 11.4: Find by Name."""

    def __init__(self) -> None:
        super().__init__(
            id="11.4",
            section=11,
            title="Hledání souborů (find)",
            instructions="""# Level 11.4: Hledání souborů podle názvu (find)

Příkaz `find` hledá soubory v adresářové hierarchii.
Na rozdíl od `grep` (který hledá OBSAH), `find` hledá podle NÁZVU nebo vlastností souboru.

### Proč je to užitečné
Ztratili jste soubor někde v hlubokém adresářovém stromě?
Příkaz `find` ho najde za vás.

### Úkol
Najděte soubor pojmenovaný `lost_file.txt` někde uvnitř adresáře `messy_dir`.
Odevzdejte celou relativní cestu, kterou jste našli.

### Příkazy
- `find adresář -name "název"` - hledá soubory podle názvu

### Odevzdání
Odevzdejte nalezenou cestu.
`shellgame submit -f messy_dir/a/b/c/d/lost_file.txt`""",
            hints=[
                "Příkaz find prohledá všechny podadresáře automaticky.",
                'Syntaxe je: find kde_hledat -name "co_hledat"',
                "Použijte 'find messy_dir -name \"lost_file.txt\"'.",
            ],
            start_directory="level-11/searching",
        )

    def setup(self, workspace: Path) -> None:
        """Create messy directory."""
        level_dir = workspace / "level-11" / "find"
        level_dir.mkdir(parents=True, exist_ok=True)

        messy = level_dir / "messy_dir"
        messy.mkdir(exist_ok=True)

        # Create deep structure
        d = messy / "a" / "b" / "c" / "d"
        d.mkdir(parents=True, exist_ok=True)

        (d / "lost_file.txt").touch()
        (messy / "other.txt").touch()
        (messy / "a" / "junk.txt").touch()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate found path."""
        if answer is None:
            return False, "Zadejte cestu k nalezenému souboru."

        expected = "messy_dir/a/b/c/d/lost_file.txt"

        if answer.strip().endswith(expected):
            return True, "Správně!"
        else:
            return False, "To není správná cesta."


class Level11_5(Level):
    """Level 11.5: Find by Extension."""

    def __init__(self) -> None:
        super().__init__(
            id="11.5",
            section=11,
            title="Hledání podle přípony",
            instructions="""# Level 11.4: Hledání souborů podle přípony

S příkazem `find` můžete použít žolíky pro hledání souborů určitého typu.
**Důležité:** Vzor musíte dát do uvozovek, aby ho shell nerozbalil předčasně!

### Proč uvozovky?
Bez uvozovek by shell nahradil `*.py` za seznam souborů v AKTUÁLNÍM adresáři,
místo aby hledal v cílovém adresáři.

### Úkol
Najděte všechny soubory s příponou `.py` v adresáři `src_code` a uložte seznam do `python_files.txt`.

### Příkazy
- `find adresář -name "*.py"` - hledá soubory s příponou .py
- `find ... > soubor` - uloží výsledky

### Odevzdání
Odevzdejte název vytvořeného souboru.
`shellgame submit -f python_files.txt`""",
            hints=[
                'Nezapomeňte dát vzor do uvozovek: "*.py", ne *.py',
                "Kombinujte find s přesměrováním > pro uložení výsledků.",
                "Použijte 'find src_code -name \"*.py\" > python_files.txt'.",
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create source code dir."""
        level_dir = workspace / "level-11" / "extension"
        level_dir.mkdir(parents=True, exist_ok=True)

        src = level_dir / "src_code"
        src.mkdir(exist_ok=True)

        (src / "main.py").touch()
        (src / "utils.py").touch()
        (src / "README.txt").touch()
        (src / "data.csv").touch()

        if (level_dir / "python_files.txt").exists():
            (level_dir / "python_files.txt").unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate output file."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        str_val = StringValidator("python_files.txt")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        target = state.workspace / "level-11/extension/python_files.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "main.py" in content and "utils.py" in content and "README.txt" not in content:
            return True, "Správně!"
        else:
            return False, "Soubor neobsahuje správný seznam souborů."


class Level11_6(Level):
    """Level 11.6: Section 11 Summary - Final Challenge."""

    def __init__(self) -> None:
        super().__init__(
            id="11.6",
            section=11,
            title="Finální výzva",
            instructions="""### 🏆 Finální výzva: Terminálový ninja

Gratulujeme! Dostali jste se na konec ShellGame.
Tato výzva kombinuje vše, co jste se naučili.

### Úkol
V `level-11/final`:

1. Najděte **všechny** `.sh` skripty (rekurzivně) pomocí `find`
2. Jeden z nich obsahuje tajný kód - najděte ho pomocí `grep`
3. Skript s kódem **zkopírujte** do složky `found/`

### Struktura odpovědi
`<počet_skriptů>,<tajný_kód>`

### Shrnutí všech sekcí
```
Navigace:     cd, pwd, ls
Soubory:      cat, head, tail, less
Skryté:       ls -a, .soubory
Vytváření:    mkdir, touch, rm, rmdir
Typy:         file
Přesun:       cp, mv
Oprávnění:    chmod, ls -l
Přesměrování: >, >>, 2>, &>
Streamy:      stdout, stderr, /dev/null
Wildcards:    *, ?, [abc]
Find:         find -name -type -size
```

### Odevzdání
`shellgame submit <počet>,<kód>`""",
            hints=[
                "Najděte skripty: 'find . -name \"*.sh\"'. Hledejte kód: 'grep -r \"SECRET\" .'",
                "Až najdete skript s kódem, zkopírujte ho: 'cp cesta/skript.sh found/'.",
                "Jsou 4 skripty, kód je 'NINJA2024', a skript je 'hidden/secret.sh'.",
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create final challenge."""
        final_dir = workspace / "level-11" / "final"

        if final_dir.exists():
            shutil.rmtree(final_dir)

        final_dir.mkdir(parents=True, exist_ok=True)
        (final_dir / "found").mkdir()

        # Scripts in various locations
        (final_dir / "run.sh").write_text("#!/bin/bash\necho 'Running...'\n")

        (final_dir / "scripts").mkdir()
        (final_dir / "scripts" / "build.sh").write_text("#!/bin/bash\nmake all\n")
        (final_dir / "scripts" / "test.sh").write_text("#!/bin/bash\npytest\n")

        (final_dir / "hidden").mkdir()
        (final_dir / "hidden" / "secret.sh").write_text("#!/bin/bash\n# SECRET_CODE=NINJA2024\necho 'You found me!'\n")

        # Distractors
        (final_dir / "readme.txt").write_text("Look for shell scripts!\n")
        (final_dir / "data.csv").write_text("col1,col2\n")

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:  # noqa: PLR0911
        """Validate final challenge."""
        if answer is None:
            return False, "Zadejte odpověď ve formátu: počet_skriptů,tajný_kód"

        answer = answer.strip()
        parts = answer.split(",")

        if len(parts) != 2:
            return False, "Formát: počet,kód (např. 5,SECRET123)"

        try:
            script_count = int(parts[0].strip())
            secret_code = parts[1].strip().upper()
        except ValueError:
            return False, "První hodnota musí být číslo."

        if script_count != 4:
            return (
                False,
                f"Počet .sh skriptů není {script_count}. Použijte 'find . -name \"*.sh\"'.",
            )

        if secret_code != "NINJA2024":
            return (
                False,
                f"Tajný kód není {secret_code}. Hledejte 'SECRET' v obsahu skriptů pomocí 'grep'.",
            )

        # Bonus check: did they copy the file?
        found_dir = state.workspace / "level-11" / "final" / "found"
        if (found_dir / "secret.sh").exists():
            return (
                True,
                """🎊 GRATULUJEME! 🎊

Úspěšně jste dokončili ShellGame!

Nyní ovládáte základy práce s terminálem:
✓ Navigace v souborovém systému
✓ Práce se soubory a adresáři
✓ Oprávnění a typy souborů
✓ Přesměrování a streamy
✓ Wildcards a vyhledávání

Jste připraveni na další dobrodružství v Linuxu! 🐧""",
            )
        else:
            return (
                True,
                """🎊 GRATULUJEME! 🎊

Úspěšně jste dokončili ShellGame!

(Tip: Pro plný zážitek zkopírujte secret.sh do found/ složky 😉)

Nyní ovládáte základy práce s terminálem:
✓ Navigace v souborovém systému
✓ Práce se soubory a adresáři
✓ Oprávnění a typy souborů
✓ Přesměrování a streamy
✓ Wildcards a vyhledávání

Jste připraveni na další dobrodružství v Linuxu! 🐧""",
            )


def get_levels() -> list:
    """
    Return all Section 11 level classes.

    Returns:
        List of Level instances for Section 11
    """
    return [
        Level11_0(),
        Level11_1(),
        Level11_2(),
        Level11_3(),
        Level11_4(),
        Level11_5(),
        Level11_6(),
    ]
