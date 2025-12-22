"""Section 4: Creation and Cleanup (declarative style)."""

from __future__ import annotations

import shutil
from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    DirectoryExistsValidator,
    FileExistsValidator,
    StringValidator,
    ValidationResult,
)


def _setup_section4_common(workspace: Path) -> None:
    """Ensure the section workspace root exists."""
    (workspace / "level-4").mkdir(parents=True, exist_ok=True)


class SectionIntroLevel(Level):
    title = "Sekce 4: Vytváření a mazání"
    instructions_file = "section4_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    start_directory = None
    success_message = "Jdeme na to!"

    @override
    def setup(self, workspace: Path) -> None:
        _setup_section4_common(workspace)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


class CreateFileLevel(Level):
    title = "Vytvoření souboru"
    instructions = """
        Příkaz `touch` slouží k vytvoření prázdného souboru (nebo aktualizaci času přístupu,
        pokud soubor již existuje).

        ## Úkol
        Vytvořte prázdný soubor s názvem `novy_soubor.txt` v aktuálním adresáři.

        ## Příkazy
        - `touch <název_souboru>`

        ## Odevzdání
        Odevzdejte název vytvořeného souboru.
        `shellgame submit -f novy_soubor.txt`
        """
    hints = [
        "Příkaz 'touch' vytvoří prázdný soubor. Jaký název má mít?",
        "Syntaxe je jednoduchá: touch název_souboru",
        "Použijte 'touch novy_soubor.txt' a pak ověřte pomocí 'ls'.",
    ]
    start_directory = "level-4/creation"
    require_answer = True
    validators = [
        StringValidator("novy_soubor.txt"),
        FileExistsValidator("level-4/creation/novy_soubor.txt"),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_section4_common(workspace)
        (workspace / "level-4" / "creation").mkdir(parents=True, exist_ok=True)

        target = workspace / "level-4" / "creation" / "novy_soubor.txt"
        if target.exists():
            target.unlink()


class CreateDirectoryLevel(Level):
    title = "Vytvoření adresáře"
    instructions = """
        Příkaz `mkdir` (make directory) slouží k vytváření nových adresářů.

        ## Úkol
        Vytvořte adresář s názvem `data` v aktuálním adresáři.

        ## Příkazy
        - `mkdir <název_adresáře>`

        ## Odevzdání
        Odevzdejte název vytvořeného adresáře.
        `shellgame submit -f data`
        """
    hints = [
        "Použijte 'mkdir data'.",
        "Ověřte pomocí 'ls -F' (adresáře mají lomítko).",
    ]
    start_directory = "level-4/creation"
    require_answer = True
    validators = [
        StringValidator("data"),
        DirectoryExistsValidator("level-4/creation/data"),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_section4_common(workspace)
        (workspace / "level-4" / "creation").mkdir(parents=True, exist_ok=True)

        target = workspace / "level-4" / "creation" / "data"
        if target.exists():
            shutil.rmtree(target)


class NestedDirectoryCreationLevel(Level):
    title = "Vytváření zanořených adresářů"
    instructions = """
        Pokud chcete vytvořit celou cestu adresářů najednou (např. `projekt/src/main`),
        příkaz `mkdir` by normálně selhal, pokud rodičovské adresáře neexistují.

        Přepínač `-p` (parents) řekne příkazu `mkdir`, aby vytvořil i všechny chybějící
        rodičovské adresáře.

        ## Úkol
        Vytvořte strukturu adresářů `projekt/src/tests` jedním příkazem.

        ## Příkazy
        - `mkdir -p <cesta>`

        ## Odevzdání
        Odevzdejte celou cestu, kterou jste vytvořili.
        `shellgame submit -f projekt/src/tests`
        """
    hints = [
        "Co se stane, když zkusíte 'mkdir projekt/src/tests' bez přepínače -p?",
        "Přepínač -p (parents) vytvoří i všechny nadřazené adresáře, které chybí.",
        "Použijte 'mkdir -p projekt/src/tests'.",
    ]
    start_directory = "level-4/nested"
    require_answer = True
    validators = [
        StringValidator("projekt/src/tests"),
        DirectoryExistsValidator("level-4/nested/projekt/src/tests"),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_section4_common(workspace)
        (workspace / "level-4" / "nested").mkdir(parents=True, exist_ok=True)

        target = workspace / "level-4" / "nested" / "projekt"
        if target.exists():
            shutil.rmtree(target)


class DeleteFileLevel(Level):
    title = "Mazání souborů"
    instructions = """
        Příkaz `rm` (remove) slouží k mazání souborů.

        ### Důležité varování
        Smazané soubory v příkazové řádce NEJDOU DO KOŠE! Jsou nenávratně pryč.

        ### Bezpečnostní tipy
        - `rm -i soubor` (zeptá se před smazáním)
        - Před `rm *.log` si zkontrolujte `ls *.log`

        ## Úkol
        Smažte soubor `stary_log.txt`, který se nachází v aktuálním adresáři.

        ## Příkazy
        - `rm <soubor>`

        ## Odevzdání
        Odevzdejte název smazaného souboru.
        `shellgame submit -f stary_log.txt`
        """
    hints = [
        "Příkaz rm permanentně maže soubory. Jaký soubor máte smazat?",
        "Syntaxe je jednoduchá: rm název_souboru",
        "Použijte 'rm stary_log.txt'.",
    ]
    start_directory = "level-4/cleanup"
    require_answer = True
    validators = [
        StringValidator("stary_log.txt"),
        FileExistsValidator("level-4/cleanup/stary_log.txt", should_exist=False),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_section4_common(workspace)
        cleanup_dir = workspace / "level-4" / "cleanup"
        cleanup_dir.mkdir(parents=True, exist_ok=True)
        (cleanup_dir / "stary_log.txt").write_text("old data")


class DeleteDirectoryLevel(Level):
    title = "Mazání adresářů"
    instructions = """
        Pro mazání prázdných adresářů slouží příkaz `rmdir`.
        Pokud adresář není prázdný, `rmdir` selže.

        Pro smazání adresáře i s jeho obsahem použijte `rm -r` (recursive).

        ## Úkol
        Smažte adresář `temp`, který obsahuje nějaké dočasné soubory.

        ## Příkazy
        - `rmdir <adresář>` (jen prázdný)
        - `rm -r <adresář>` (rekurzivně i s obsahem)

        ## Odevzdání
        Odevzdejte název smazaného adresáře.
        `shellgame submit -f temp`
        """
    hints = [
        "Zkuste nejdřív 'rmdir temp'. Co se stane?",
        "Pokud adresář není prázdný, rmdir selže. Jaký přepínač potřebujete pro rekurzivní mazání?",
        "Použijte 'rm -r temp' pro smazání adresáře včetně obsahu.",
    ]
    start_directory = "level-4/cleanup"
    require_answer = True
    validators = [
        StringValidator("temp"),
        DirectoryExistsValidator("level-4/cleanup/temp", should_exist=False),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_section4_common(workspace)
        temp_dir = workspace / "level-4" / "cleanup" / "temp"
        temp_dir.mkdir(parents=True, exist_ok=True)
        (temp_dir / "junk.txt").write_text("junk")


class ProjectScaffoldLevel(Level):
    title = "Příprava projektu"
    instructions = """
        ## Úkol
        Vytvořte následující strukturu v adresáři `web`:

        - `web/index.html` (soubor)
        - `web/css/style.css` (soubor v podadresáři)
        - `web/js` (prázdný adresář)

        Použijte kombinaci `mkdir -p` a `touch`.

        ## Odevzdání
        Odevzdejte název kořenového adresáře projektu.
        `shellgame submit -f web`
        """
    hints = [
        "Nejdřív vytvořte adresáře: 'mkdir -p web/css web/js'",
        "Pak vytvořte soubory: 'touch web/index.html web/css/style.css'",
    ]
    extension = True
    start_directory = "level-4/project"
    require_answer = True
    validators = [
        StringValidator("web"),
        FileExistsValidator("level-4/project/web/index.html"),
        FileExistsValidator("level-4/project/web/css/style.css"),
        DirectoryExistsValidator("level-4/project/web/js"),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_section4_common(workspace)
        (workspace / "level-4" / "project").mkdir(parents=True, exist_ok=True)

        target = workspace / "level-4" / "project" / "web"
        if target.exists():
            shutil.rmtree(target)


class CleanupMultipleFilesLevel(Level):
    title = "Úklid nepořádku"
    instructions = """
        Někdy je potřeba smazat více souborů najednou. Příkaz `rm` přijímá více argumentů.

        ## Úkol
        V adresáři `mess` se nachází tři soubory, které je třeba smazat:
        `error.log`, `temp.dat` a `junk.tmp`.

        Smažte je všechny.

        ## Příkazy
        - `rm soubor1 soubor2 soubor3`

        ## Odevzdání
        Odevzdejte název adresáře, který jste vyčistili.
        `shellgame submit -f mess`
        """
    hints = [
        "Použijte 'rm error.log temp.dat junk.tmp'",
        "Nebo je smažte po jednom.",
    ]
    optional = True
    start_directory = "level-4/mess"
    require_answer = True
    validators = [
        StringValidator("mess"),
        FileExistsValidator("level-4/mess/error.log", should_exist=False),
        FileExistsValidator("level-4/mess/temp.dat", should_exist=False),
        FileExistsValidator("level-4/mess/junk.tmp", should_exist=False),
        FileExistsValidator("level-4/mess/keep_me.txt", should_exist=True),
    ]

    @override
    def setup(self, workspace: Path) -> None:
        _setup_section4_common(workspace)
        mess_dir = workspace / "level-4" / "mess"
        mess_dir.mkdir(parents=True, exist_ok=True)

        (mess_dir / "error.log").write_text("")
        (mess_dir / "temp.dat").write_text("")
        (mess_dir / "junk.tmp").write_text("")
        (mess_dir / "keep_me.txt").write_text("important")


class SectionChallengeLevel(Level):
    title = "Souhrn Sekce 4"
    instructions = """
        ### Výzva: Stavitel souborového systému

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
        `shellgame submit builder`
        """
    hints = [
        "Postupujte krok za krokem. Začněte s 'mkdir -p myproject/src myproject/docs'.",
        "Pro soubor: 'touch myproject/README.md'. Pro mazání: 'rm delete_me.txt' a 'rmdir empty_dir'.",
        "Zkontrolujte strukturu pomocí 'ls -R myproject' a pak odevzdejte 'builder'.",
    ]
    start_directory = "level-4/challenge"
    require_answer = True

    @override
    def setup(self, workspace: Path) -> None:
        _setup_section4_common(workspace)
        challenge_dir = workspace / "level-4" / "challenge"

        if challenge_dir.exists():
            shutil.rmtree(challenge_dir)

        challenge_dir.mkdir(parents=True, exist_ok=True)
        (challenge_dir / "delete_me.txt").write_text("Delete this!\n")
        (challenge_dir / "empty_dir").mkdir(exist_ok=True)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:  # noqa: PLR0911
        if answer is None:
            return False, "Musíte zadat heslo."

        cleaned = answer.strip().lower()
        if cleaned != "builder":
            return False, "Heslo není správné. Splňte úkol a odevzdejte: builder"

        challenge = state.workspace / "level-4" / "challenge"

        required = [
            challenge / "myproject" / "src",
            challenge / "myproject" / "docs",
            challenge / "myproject" / "README.md",
        ]
        for path in required:
            if not path.exists():
                if "src" in str(path):
                    return False, "Chybí adresář myproject/src. Použijte 'mkdir -p myproject/src'."
                if "docs" in str(path):
                    return False, "Chybí adresář myproject/docs."
                return False, "Chybí soubor myproject/README.md. Použijte 'touch myproject/README.md'."

        if (challenge / "delete_me.txt").exists():
            return False, "Soubor delete_me.txt nebyl smazán. Použijte 'rm delete_me.txt'."

        if (challenge / "empty_dir").exists():
            return False, "Adresář empty_dir nebyl smazán. Použijte 'rmdir empty_dir'."

        return True, "Brilantní! Dokončili jste Sekci 4. Umíte vytvářet i bourat!"


# ---------------------------------------------------------------------------
# Backwards-compatible class names (if other code/tests import by old names)
# ---------------------------------------------------------------------------


class Level4_0(SectionIntroLevel):
    pass


class Level4_1(CreateFileLevel):
    pass


class Level4_2(CreateDirectoryLevel):
    pass


class Level4_3(NestedDirectoryCreationLevel):
    pass


class Level4_4(DeleteFileLevel):
    pass


class Level4_5(DeleteDirectoryLevel):
    pass


class Level4_6(ProjectScaffoldLevel):
    pass


class Level4_7(CleanupMultipleFilesLevel):
    pass


class Level4_8(SectionChallengeLevel):
    pass


def get_levels() -> list[Level]:
    """Return all Section 4 level instances with IDs assigned dynamically."""
    levels: list[Level] = [
        SectionIntroLevel(),
        CreateFileLevel(),
        CreateDirectoryLevel(),
        NestedDirectoryCreationLevel(),
        DeleteFileLevel(),
        DeleteDirectoryLevel(),
        ProjectScaffoldLevel(),
        CleanupMultipleFilesLevel(),
        SectionChallengeLevel(),
    ]

    section_num = 4
    for i, level in enumerate(levels):
        level.section = section_num
        level.id = f"{section_num}.{i}"

    return levels
