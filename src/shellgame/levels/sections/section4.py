"""Section 4: Creation and Cleanup."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Optional, Tuple

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    DirectoryExistsValidator,
    FileExistsValidator,
    MultiValidator,
    StringValidator,
)


class Level4_0(Level):
    """Level 4.0: Section 4 Introduction."""

    def __init__(self) -> None:
        super().__init__(
            id="4.0",
            section=4,
            title="Sekce 4: Vytváření a mazání",
            instructions_file="section4_intro.md",
            hints=["Přečtěte si úvod a pokračujte stisknutím Enter."],
        )

    @override
    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        return

    @override
    def validate(
        self, answer: str | None, state: GameStateProtocol
    ) -> tuple[bool, str]:
        """Always valid."""
        return True, "Jdeme na to!"


class Level4_1(Level):
    """Level 4.1: Creating a File."""

    def __init__(self) -> None:
        super().__init__(
            id="4.1",
            section=4,
            title="Vytvoření souboru",
            instructions="""# Level 4.1: Vytvoření souboru

Příkaz `touch` slouží k vytvoření prázdného souboru (nebo aktualizaci času přístupu, pokud soubor již existuje).

## Úkol:
Vytvořte prázdný soubor s názvem `novy_soubor.txt` v aktuálním adresáři.

## Příkazy:
- `touch <název_souboru>`: Vytvoří prázdný soubor

## Odevzdání:
Odevzdejte název vytvořeného souboru.
`shellgame submit -f novy_soubor.txt`""",
            hints=[
                "Příkaz 'touch' vytvoří prázdný soubor. Jaký název má mít?",
                "Syntaxe je jednoduchá: touch název_souboru",
                "Použijte 'touch novy_soubor.txt' a pak ověřte pomocí 'ls'.",
            ],
            start_directory="level-4/creation",
        )

    def setup(self, workspace: Path) -> None:
        """Ensure clean slate."""
        target = workspace / "level-4" / "creation" / "novy_soubor.txt"
        if target.exists():
            target.unlink()

        (workspace / "level-4" / "creation").mkdir(parents=True, exist_ok=True)

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate file creation."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        # Check if they typed the correct filename
        str_val = StringValidator("novy_soubor.txt")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        # Check if file actually exists
        file_val = FileExistsValidator("level-4/creation/novy_soubor.txt")
        return file_val.validate(answer, state.workspace)


class Level4_2(Level):
    """Level 4.2: Creating a Directory."""

    def __init__(self) -> None:
        super().__init__(
            id="4.2",
            section=4,
            title="Vytvoření adresáře",
            instructions="""# Level 4.2: Vytvoření adresáře

Příkaz `mkdir` (make directory) slouží k vytváření nových adresářů.

## Úkol:
Vytvořte adresář s názvem `data` v aktuálním adresáři.

## Příkazy:
- `mkdir <název_adresáře>`: Vytvoří nový adresář

## Odevzdání:
Odevzdejte název vytvořeného adresáře.
`shellgame submit -f data`""",
            hints=[
                "Použijte 'mkdir data'.",
                "Ověřte pomocí 'ls -F' (adresáře mají lomítko).",
            ],
            start_directory="level-4/creation",
        )

    def setup(self, workspace: Path) -> None:
        """Ensure clean slate."""
        target = workspace / "level-4" / "creation" / "data"
        if target.exists():
            import shutil

            shutil.rmtree(target)

        (workspace / "level-4" / "creation").mkdir(parents=True, exist_ok=True)

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate directory creation."""
        if answer is None:
            return False, "Musíte zadat název adresáře."

        str_val = StringValidator("data")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        dir_val = DirectoryExistsValidator("level-4/creation/data")
        return dir_val.validate(answer, state.workspace)


class Level4_3(Level):
    """Level 4.3: Nested Creation."""

    def __init__(self) -> None:
        super().__init__(
            id="4.3",
            section=4,
            title="Vytváření zanořených adresářů",
            instructions="""# Level 4.3: Vytváření zanořených adresářů

Pokud chcete vytvořit celou cestu adresářů najednou (např. `projekt/src/main`), příkaz `mkdir` by normálně selhal, pokud rodičovské adresáře neexistují.
Přepínač `-p` (parents) řekne příkazu `mkdir`, aby vytvořil i všechny chybějící rodičovské adresáře.

## Úkol:
Vytvořte strukturu adresářů `projekt/src/tests` jedním příkazem.

## Příkazy:
- `mkdir -p <cesta>`: Vytvoří adresář včetně rodičů

## Odevzdání:
Odevzdejte celou cestu, kterou jste vytvořili.
`shellgame submit -f projekt/src/tests`""",
            hints=[
                "Co se stane, když zkusíte 'mkdir projekt/src/tests' bez přepínače -p?",
                "Přepínač -p (parents) vytvoří i všechny nadřazené adresáře, které chybí.",
                "Použijte 'mkdir -p projekt/src/tests'.",
            ],
            start_directory="level-4/nested",
        )

    def setup(self, workspace: Path) -> None:
        """Ensure clean slate."""
        target = workspace / "level-4" / "nested" / "projekt"
        if target.exists():
            import shutil

            shutil.rmtree(target)

        (workspace / "level-4" / "nested").mkdir(parents=True, exist_ok=True)

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate nested directory creation."""
        if answer is None:
            return False, "Musíte zadat cestu."

        str_val = StringValidator("projekt/src/tests")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        dir_val = DirectoryExistsValidator("level-4/nested/projekt/src/tests")
        return dir_val.validate(answer, state.workspace)


class Level4_4(Level):
    """Level 4.4: Deleting Files."""

    def __init__(self) -> None:
        super().__init__(
            id="4.4",
            section=4,
            title="Mazání souborů",
            instructions="""# Level 4.4: Mazání souborů

Příkaz `rm` (remove) slouží k mazání souborů.

### ⚠️ Důležité varování
Smazané soubory v příkazové řádce **NEJDOU DO KOŠE**!
Jsou nenávratně pryč. Vždy si rozmyslete, co mažete.

### Běžné bezpečnostní praktiky
- `rm -i soubor` - zeptá se před smazáním (interactive)
- Před `rm *.log` si raději zkontrolujte `ls *.log`

## Úkol:
Smažte soubor `stary_log.txt`, který se nachází v aktuálním adresáři.

## Příkazy:
- `rm <soubor>`: Smaže soubor

## Odevzdání:
Odevzdejte název smazaného souboru.
`shellgame submit -f stary_log.txt`""",
            hints=[
                "Příkaz rm permanentně maže soubory. Jaký soubor máte smazat?",
                "Syntaxe je jednoduchá: rm název_souboru",
                "Použijte 'rm stary_log.txt'.",
            ],
            start_directory="level-4/cleanup",
        )

    def setup(self, workspace: Path) -> None:
        """Create file to delete."""
        cleanup_dir = workspace / "level-4" / "cleanup"
        cleanup_dir.mkdir(parents=True, exist_ok=True)
        (cleanup_dir / "stary_log.txt").write_text("old data")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate file deletion."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        str_val = StringValidator("stary_log.txt")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        file_val = FileExistsValidator(
            "level-4/cleanup/stary_log.txt", should_exist=False
        )
        return file_val.validate(answer, state.workspace)


class Level4_5(Level):
    """Level 4.5: Deleting Directories."""

    def __init__(self) -> None:
        super().__init__(
            id="4.5",
            section=4,
            title="Mazání adresářů",
            instructions="""# Level 4.5: Mazání adresářů

Pro mazání prázdných adresářů slouží příkaz `rmdir`.
Pokud adresář není prázdný, `rmdir` selže. Pro smazání adresáře i s jeho obsahem použijte `rm -r` (recursive).

## Úkol:
Smažte adresář `temp`, který obsahuje nějaké dočasné soubory.

## Příkazy:
- `rmdir <adresář>`: Smaže prázdný adresář
- `rm -r <adresář>`: Smaže adresář a všechen jeho obsah

## Odevzdání:
Odevzdejte název smazaného adresáře.
`shellgame submit -f temp`""",
            hints=[
                "Zkuste nejdřív 'rmdir temp'. Co se stane?",
                "Pokud adresář není prázdný, rmdir selže. Jaký přepínač potřebujete pro rekurzivní mazání?",
                "Použijte 'rm -r temp' pro smazání adresáře včetně obsahu.",
            ],
            start_directory="level-4/cleanup",
        )

    def setup(self, workspace: Path) -> None:
        """Create directory to delete."""
        temp_dir = workspace / "level-4" / "cleanup" / "temp"
        temp_dir.mkdir(parents=True, exist_ok=True)
        (temp_dir / "junk.txt").write_text("junk")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate directory deletion."""
        if answer is None:
            return False, "Musíte zadat název adresáře."

        str_val = StringValidator("temp")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        dir_val = DirectoryExistsValidator("level-4/cleanup/temp", should_exist=False)
        return dir_val.validate(answer, state.workspace)


class Level4_6(Level):
    """Level 4.6: Project Scaffold."""

    def __init__(self) -> None:
        super().__init__(
            id="4.6",
            section=4,
            title="Příprava projektu",
            instructions="""# Level 4.6: Příprava projektu

Vytvořte základní strukturu pro nový webový projekt.

## Úkol:
Vytvořte následující strukturu v adresáři `web`:
- `web/index.html` (soubor)
- `web/css/style.css` (soubor v podadresáři)
- `web/js` (prázdný adresář)

Použijte kombinaci `mkdir -p` a `touch`.

## Odevzdání:
Odevzdejte název kořenového adresáře projektu.
`shellgame submit -f web`""",
            hints=[
                "Nejdřív vytvořte adresáře: 'mkdir -p web/css web/js'",
                "Pak vytvořte soubory: 'touch web/index.html web/css/style.css'",
            ],
            extension=True,
            start_directory="level-4/project",
        )

    def setup(self, workspace: Path) -> None:
        """Ensure clean slate."""
        target = workspace / "level-4" / "project" / "web"
        if target.exists():
            import shutil

            shutil.rmtree(target)

        (workspace / "level-4" / "project").mkdir(parents=True, exist_ok=True)

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate project structure."""
        if answer is None:
            return False, "Musíte zadat název projektu."

        str_val = StringValidator("web")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        validator = MultiValidator(
            [
                FileExistsValidator("level-4/project/web/index.html"),
                FileExistsValidator("level-4/project/web/css/style.css"),
                DirectoryExistsValidator("level-4/project/web/js"),
            ]
        )
        return validator.validate(answer, state.workspace)


class Level4_7(Level):
    """Level 4.7: Pattern Preview Cleanup."""

    def __init__(self) -> None:
        super().__init__(
            id="4.7",
            section=4,
            title="Úklid nepořádku",
            instructions="""# Level 4.7: Úklid nepořádku

Někdy je potřeba smazat více souborů najednou. Příkaz `rm` přijímá více argumentů.

## Úkol:
V adresáři `mess` se nachází tři soubory, které je třeba smazat: `error.log`, `temp.dat` a `junk.tmp`.
Smažte je všechny.

## Příkazy:
- `rm soubor1 soubor2 soubor3`

## Odevzdání:
Odevzdejte název adresáře, který jste vyčistili.
`shellgame submit -f mess`""",
            hints=[
                "Použijte 'rm error.log temp.dat junk.tmp'",
                "Nebo je smažte po jednom.",
            ],
            optional=True,
            start_directory="level-4/mess",
        )

    def setup(self, workspace: Path) -> None:
        """Create mess."""
        mess_dir = workspace / "level-4" / "mess"
        mess_dir.mkdir(parents=True, exist_ok=True)

        (mess_dir / "error.log").write_text("")
        (mess_dir / "temp.dat").write_text("")
        (mess_dir / "junk.tmp").write_text("")
        (mess_dir / "keep_me.txt").write_text("important")

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate cleanup."""
        if answer is None:
            return False, "Musíte zadat název adresáře."

        str_val = StringValidator("mess")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        validator = MultiValidator(
            [
                FileExistsValidator("level-4/mess/error.log", should_exist=False),
                FileExistsValidator("level-4/mess/temp.dat", should_exist=False),
                FileExistsValidator("level-4/mess/junk.tmp", should_exist=False),
                FileExistsValidator("level-4/mess/keep_me.txt", should_exist=True),
            ]
        )
        return validator.validate(answer, state.workspace)


class Level4_8(Level):
    """Level 4.8: Section 4 Summary."""

    def __init__(self) -> None:
        super().__init__(
            id="4.8",
            section=4,
            title="Souhrn Sekce 4",
            instructions="""### 🎯 Výzva: Stavitel souborového systému

Ukažte, že umíte vytvářet i mazat!

### Úkol
V `level-4/challenge`:

1. Vytvořte adresářovou strukturu: `myproject/src` a `myproject/docs`
2. Vytvořte soubor `myproject/README.md`
3. Smažte existující soubor `delete_me.txt`
4. Smažte existující prázdný adresář `empty_dir`

Pak odevzdejte heslo: **builder**

### Shrnutí příkazů Sekce 4
```
mkdir adresar       → Nový adresář
mkdir -p a/b/c      → Celá cesta najednou
touch soubor        → Nový prázdný soubor
rm soubor           → Smazat soubor
rmdir adresar       → Smazat prázdný adresář
rm -r adresar       → Smazat adresář s obsahem
```

### Odevzdání
`shellgame submit builder`""",
            hints=[
                "Postupujte krok za krokem. Začněte s 'mkdir -p myproject/src myproject/docs'.",
                "Pro soubor: 'touch myproject/README.md'. Pro mazání: 'rm delete_me.txt' a 'rmdir empty_dir'.",
                "Zkontrolujte strukturu pomocí 'ls -R myproject' a pak odevzdejte 'builder'.",
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create challenge environment."""
        challenge_dir = workspace / "level-4" / "challenge"

        # Clean slate
        if challenge_dir.exists():
            import shutil

            shutil.rmtree(challenge_dir)

        challenge_dir.mkdir(parents=True, exist_ok=True)

        # Items to delete
        (challenge_dir / "delete_me.txt").write_text("Delete this!\n")
        (challenge_dir / "empty_dir").mkdir(exist_ok=True)

    def validate(self, answer: Optional[str], state: Any) -> Tuple[bool, str]:
        """Validate structure."""
        if answer is None:
            return False, "Musíte zadat heslo."

        answer = answer.strip().lower()

        if answer != "builder":
            return False, "Heslo není správné. Splňte úkol a odevzdejte: builder"

        # Check structure
        challenge = state.workspace / "level-4" / "challenge"

        # Must exist
        required = [
            challenge / "myproject" / "src",
            challenge / "myproject" / "docs",
            challenge / "myproject" / "README.md",
        ]

        for path in required:
            if not path.exists():
                if "src" in str(path):
                    return (
                        False,
                        "Chybí adresář myproject/src. Použijte 'mkdir -p myproject/src'.",
                    )
                elif "docs" in str(path):
                    return False, "Chybí adresář myproject/docs."
                else:
                    return (
                        False,
                        "Chybí soubor myproject/README.md. Použijte 'touch myproject/README.md'.",
                    )

        # Must not exist
        if (challenge / "delete_me.txt").exists():
            return (
                False,
                "Soubor delete_me.txt nebyl smazán. Použijte 'rm delete_me.txt'.",
            )

        if (challenge / "empty_dir").exists():
            return False, "Adresář empty_dir nebyl smazán. Použijte 'rmdir empty_dir'."

        return True, "🎉 Brilantní! Dokončili jste Sekci 4. Umíte vytvářet i bourat!"


def get_levels() -> list[Level]:
    """
    Return all Section 4 level classes.

    Returns:
        List of Level instances for Section 4
    """
    return [
        Level4_0(),
        Level4_1(),
        Level4_2(),
        Level4_3(),
        Level4_4(),
        Level4_5(),
        Level4_6(),
        Level4_7(),
        Level4_8(),
    ]
