"""Section 11: Searching."""

from __future__ import annotations

import shutil
from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    StringValidator,
    ValidationResult,
)


class SectionIntro(Level):
    title = "Sekce 11: Vyhledávání"
    instructions_file = "section11_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"

    @override
    def setup(self, workspace: Path) -> None:
        return

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


class GrepPasswordLineToFileLevel(Level):
    title = "Hledání v souboru (grep)"
    instructions = """
        ### Cíl
        Najděte řádek obsahující `PASSWORD` v souboru `config.txt` a uložte ho do `pass.txt`.

        ### Příkazy
        - `grep "vzor" soubor` - hledá vzor v souboru
        - `grep "vzor" soubor > výstup` - uloží nalezené řádky do souboru

        ### Úkol
        1. Najděte řádek s `PASSWORD`
        2. Přesměrujte výsledek do `pass.txt`

        ### Odevzdání
        Odevzdejte název vytvořeného souboru:
        `shellgame submit -f pass.txt`
        """
    hints = [
        "Příkaz `grep` hledá text v souborech.",
        'Použijte `grep "PASSWORD" config.txt > pass.txt`.',
        "V souboru je řádek `PASSWORD=Secret123`.",
    ]
    start_directory = "level-11/searching"
    require_answer = True
    validators = [StringValidator("pass.txt")]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-11" / "grep"
        level_dir.mkdir(parents=True, exist_ok=True)

        content = "user=admin\nhost=localhost\nport=8080\nPASSWORD=Secret123\ndebug=true\n"
        (level_dir / "config.txt").write_text(content, encoding="utf-8")

        target = level_dir / "pass.txt"
        if target.exists():
            target.unlink()

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        target = state.workspace / "level-11" / "grep" / "pass.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text(encoding="utf-8").strip()
        if "PASSWORD=Secret123" in content:
            return True, "Správně!"
        return False, "Soubor neobsahuje hledaný řádek."


class RecursiveGrepFindFileLevel(Level):
    title = "Rekurzivní hledání"
    instructions = """
        ### Cíl
        Pomocí `grep -r` zjistěte, který soubor v adresáři `project` obsahuje text `SECRET_KEY`.

        ### Příkazy
        - `grep -r "vzor" adresář/` - rekurzivní hledání

        ### Úkol
        Najděte soubor obsahující `SECRET_KEY` a odevzdejte jeho relativní cestu.

        ### Odevzdání
        `shellgame submit -f project/config/settings.py`
        """
    hints = [
        "Přepínač `-r` hledá rekurzivně ve všech podadresářích.",
        'Zkuste `grep -r "SECRET_KEY" project/`.',
        "Správný soubor je `project/config/settings.py`.",
    ]
    start_directory = "level-11/searching"
    require_answer = True

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-11" / "recursive"
        level_dir.mkdir(parents=True, exist_ok=True)

        project = level_dir / "project"
        project.mkdir(exist_ok=True)
        (project / "src").mkdir(exist_ok=True)
        (project / "config").mkdir(exist_ok=True)

        (project / "README.md").write_text("# Project\n", encoding="utf-8")
        (project / "src" / "main.py").write_text("print('Hello')\n", encoding="utf-8")
        (project / "config" / "settings.py").write_text("SECRET_KEY = 'xyz'\nDEBUG = True\n", encoding="utf-8")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        # Keep parent checks (require_answer)
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        expected = "project/config/settings.py"
        if answer is not None and answer.strip().endswith(expected):
            return True, "Správně!"
        return False, "To není správný soubor. Hledáme ten s 'SECRET_KEY'."


class CaseInsensitiveWarningCountLevel(Level):
    title = "Hledání bez ohledu na velikost písmen"
    instructions = """
        ### Cíl
        Najděte všechny řádky obsahující slovo `warning` bez ohledu na velikost písmen a spočítejte je.

        ### Příkazy
        - `grep -i "vzor" soubor` - ignoruje velikost písmen
        - `grep -i "vzor" soubor | wc -l` - spočítá nalezené řádky

        ### Úkol
        Spočítejte, kolik řádků v `messages.log` odpovídá `warning` (case-insensitive).

        ### Odevzdání
        `shellgame submit -f <číslo>`
        """
    hints = [
        "Bez `-i` najdete jen některé varianty. S `-i` najdete `warning`, `WARNING`, `Warning`…",
        'Použijte: `grep -i "warning" messages.log | wc -l`',
        "Správná odpověď je 4.",
    ]
    start_directory = "level-11/searching"
    require_answer = True

    @override
    def setup(self, workspace: Path) -> None:
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
        (level_dir / "messages.log").write_text(log_content, encoding="utf-8")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        assert answer is not None
        try:
            count = int(answer.strip())
        except ValueError:
            return False, "Odpověď musí být číslo."

        messages: dict[int, ValidationResult] = {
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


class FindLostFilePathLevel(Level):
    title = "Hledání souborů (find)"
    instructions = """
        ### Cíl
        Pomocí `find` najděte soubor `lost_file.txt` někde uvnitř `messy_dir`.

        ### Příkazy
        - `find adresář -name "název"` - hledá soubory podle názvu

        ### Úkol
        Najděte `lost_file.txt` a odevzdejte celou relativní cestu.

        ### Odevzdání
        `shellgame submit -f messy_dir/a/b/c/d/lost_file.txt`
        """
    hints = [
        "Příkaz `find` prohledá všechny podadresáře automaticky.",
        'Syntaxe je: `find kde_hledat -name "co_hledat"`',
        'Použijte: `find messy_dir -name "lost_file.txt"`.',
    ]
    start_directory = "level-11/searching"
    require_answer = True

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-11" / "find"
        level_dir.mkdir(parents=True, exist_ok=True)

        messy = level_dir / "messy_dir"
        messy.mkdir(exist_ok=True)

        d = messy / "a" / "b" / "c" / "d"
        d.mkdir(parents=True, exist_ok=True)

        (d / "lost_file.txt").touch()
        (messy / "other.txt").touch()
        (messy / "a" / "junk.txt").touch()

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        expected = "messy_dir/a/b/c/d/lost_file.txt"
        if answer is not None and answer.strip().endswith(expected):
            return True, "Správně!"
        return False, "To není správná cesta."


class FindPythonFilesToListLevel(Level):
    title = "Hledání podle přípony"
    instructions = """
        ### Cíl
        Najděte všechny `.py` soubory v `src_code` a uložte seznam do `python_files.txt`.

        **Důležité:** Vzor musí být v uvozovkách: `"*.py"`.

        ### Příkazy
        - `find adresář -name "*.py"` - hledá soubory s příponou .py
        - `find ... > soubor` - uloží výsledky

        ### Úkol
        Vytvořte `python_files.txt` se seznamem nalezených `.py` souborů.

        ### Odevzdání
        `shellgame submit -f python_files.txt`
        """
    hints = [
        'Nezapomeňte dát vzor do uvozovek: "*.py", ne *.py',
        "Kombinujte find s přesměrováním `>` pro uložení výsledků.",
        'Použijte: `find src_code -name "*.py" > python_files.txt`.',
    ]
    require_answer = True
    validators = [StringValidator("python_files.txt")]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-11" / "extension"
        level_dir.mkdir(parents=True, exist_ok=True)

        src = level_dir / "src_code"
        src.mkdir(exist_ok=True)

        (src / "main.py").touch()
        (src / "utils.py").touch()
        (src / "README.txt").touch()
        (src / "data.csv").touch()

        target = level_dir / "python_files.txt"
        if target.exists():
            target.unlink()

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        target = state.workspace / "level-11" / "extension" / "python_files.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text(encoding="utf-8")
        if "main.py" in content and "utils.py" in content and "README.txt" not in content:
            return True, "Správně!"
        return False, "Soubor neobsahuje správný seznam souborů."


class FinalChallengeLevel(Level):
    title = "Finální výzva"
    instructions = """
        ### 🏆 Finální výzva: Terminálový ninja

        Gratulujeme! Dostali jste se na konec ShellGame.
        Tato výzva kombinuje vše, co jste se naučili.

        ### Úkol
        V `level-11/final`:

        1. Najděte **všechny** `.sh` skripty (rekurzivně) pomocí `find`
        2. Jeden z nich obsahuje tajný kód - najděte ho pomocí `grep`
        3. Skript s kódem **zkopírujte** do složky `found/`

        ### Struktura odpovědi
        `<počet_skriptů>,<tajný_kód>`

        ### Odevzdání
        `shellgame submit <počet>,<kód>`
        """
    hints = [
        'Najděte skripty: `find . -name "*.sh"`. Hledejte kód: `grep -r "SECRET" .`',
        "Až najdete skript s kódem, zkopírujte ho: `cp cesta/skript.sh found/`.",
        "Jsou 4 skripty, kód je `NINJA2024`, a skript je `hidden/secret.sh`.",
    ]
    require_answer = True

    @override
    def setup(self, workspace: Path) -> None:
        final_dir = workspace / "level-11" / "final"
        if final_dir.exists():
            shutil.rmtree(final_dir)

        final_dir.mkdir(parents=True, exist_ok=True)
        (final_dir / "found").mkdir()

        (final_dir / "run.sh").write_text("#!/bin/bash\necho 'Running...'\n", encoding="utf-8")

        (final_dir / "scripts").mkdir()
        (final_dir / "scripts" / "build.sh").write_text("#!/bin/bash\nmake all\n", encoding="utf-8")
        (final_dir / "scripts" / "test.sh").write_text("#!/bin/bash\npytest\n", encoding="utf-8")

        (final_dir / "hidden").mkdir()
        (final_dir / "hidden" / "secret.sh").write_text(
            "#!/bin/bash\n# SECRET_CODE=NINJA2024\necho 'You found me!'\n",
            encoding="utf-8",
        )

        (final_dir / "readme.txt").write_text("Look for shell scripts!\n", encoding="utf-8")
        (final_dir / "data.csv").write_text("col1,col2\n", encoding="utf-8")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:  # noqa: PLR0911
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        assert answer is not None
        answer = answer.strip()
        parts = answer.split(",")

        if len(parts) != 2:
            return False, "Formát: počet,kód (např. 5,SECRET123)"

        try:
            script_count = int(parts[0].strip())
        except ValueError:
            return False, "První hodnota musí být číslo."

        secret_code = parts[1].strip().upper()

        if script_count != 4:
            return False, f"Počet .sh skriptů není {script_count}. Použijte 'find . -name \"*.sh\"'."

        if secret_code != "NINJA2024":
            return (
                False,
                f"Tajný kód není {secret_code}. Hledejte 'SECRET' v obsahu skriptů pomocí 'grep'.",
            )

        found_dir = state.workspace / "level-11" / "final" / "found"
        if (found_dir / "secret.sh").exists():
            return (
                True,
                """
🎊 GRATULUJEME! 🎊

Úspěšně jste dokončili ShellGame!

Nyní ovládáte základy práce s terminálem:
✓ Navigace v souborovém systému
✓ Práce se soubory a adresáři
✓ Oprávnění a typy souborů
✓ Přesměrování a streamy
✓ Wildcards a vyhledávání

Jste připraveni na další dobrodružství v Linuxu!
""".strip(),
            )

        return (
            True,
            """
🎊 GRATULUJEME! 🎊

Úspěšně jste dokončili ShellGame!

(Tip: Pro plný zážitek zkopírujte secret.sh do found/ složky.)

Jste připraveni na další dobrodružství v Linuxu!
""".strip(),
        )


def get_levels() -> list[Level]:
    levels: list[Level] = [
        SectionIntro(),
        GrepPasswordLineToFileLevel(),
        RecursiveGrepFindFileLevel(),
        CaseInsensitiveWarningCountLevel(),
        FindLostFilePathLevel(),
        FindPythonFilesToListLevel(),
        FinalChallengeLevel(),
    ]

    section_num = 11
    for i, level in enumerate(levels):
        level.section = section_num
        level.id = f"{section_num}.{i}"
    return levels
