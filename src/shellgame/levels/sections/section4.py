"""Section 4: Creation and Cleanup (declarative style)."""

from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    Completion,
    DirectoryExists,
    ExactAnswer,
    FileExists,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import RunShell, Solution

section = Section(4, root="level-4")


@section.level(0)
class SectionIntroLevel(Level):
    is_intro = True
    title = "Sekce 4: Vytváření a mazání"
    instructions_file = "section4_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    start_directory = None
    success_message = "Jdeme na to!"


@section.level(1)
class CreateFileLevel(Level):
    solution = Solution(steps=(RunShell("touch novy_soubor.txt"),), answer="novy_soubor.txt")
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
        `shellgame submit novy_soubor.txt`
        """
    hints = [
        "Příkaz 'touch' vytvoří prázdný soubor. Jaký název má mít?",
        "Syntaxe je jednoduchá: touch název_souboru",
        "Použijte 'touch novy_soubor.txt' a pak ověřte pomocí 'ls'.",
    ]
    start_directory = "creation"
    fixture = WorkspaceFixture(
        directories=("creation",),
        clean=("creation/novy_soubor.txt",),
    )
    completion = Completion(
        answer=ExactAnswer("novy_soubor.txt"),
        requirements=(FileExists("creation/novy_soubor.txt"),),
    )


@section.level(2)
class CreateDirectoryLevel(Level):
    solution = Solution(steps=(RunShell("mkdir data"),), answer="data")
    title = "Vytvoření adresáře"
    instructions = """
        Příkaz `mkdir` (make directory) slouží k vytváření nových adresářů.

        ## Úkol
        Vytvořte adresář s názvem `data` v aktuálním adresáři.

        ## Příkazy
        - `mkdir <název_adresáře>`

        ## Odevzdání
        Odevzdejte název vytvořeného adresáře.
        `shellgame submit data`
        """
    hints = [
        "Příkaz 'mkdir' slouží k vytvoření nového adresáře.",
        "Spusťte 'mkdir data' pro vytvoření adresáře data.",
        "Ověřte pomocí 'ls -F' (adresáře mají lomítko) a odevzdejte 'data'.",
    ]
    start_directory = "creation"
    fixture = WorkspaceFixture(
        directories=("creation",),
        clean=("creation/data",),
    )
    completion = Completion(
        answer=ExactAnswer("data"),
        requirements=(DirectoryExists("creation/data"),),
    )


@section.level(3)
class NestedDirectoryCreationLevel(Level):
    solution = Solution(steps=(RunShell("mkdir -p projekt/src/tests"),), answer="projekt/src/tests")
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
        `shellgame submit projekt/src/tests`
        """
    hints = [
        "Co se stane, když zkusíte 'mkdir projekt/src/tests' bez přepínače -p?",
        "Přepínač -p (parents) vytvoří i všechny nadřazené adresáře, které chybí.",
        "Použijte 'mkdir -p projekt/src/tests'.",
    ]
    start_directory = "nested"
    fixture = WorkspaceFixture(
        directories=("nested",),
        clean=("nested/projekt",),
    )
    completion = Completion(
        answer=ExactAnswer("projekt/src/tests"),
        requirements=(DirectoryExists("nested/projekt/src/tests"),),
    )


@section.level(4)
class DeleteFileLevel(Level):
    solution = Solution(steps=(RunShell("rm stary_log.txt"),), answer="stary_log.txt")
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
        `shellgame submit stary_log.txt`
        """
    hints = [
        "Příkaz rm permanentně maže soubory. Jaký soubor máte smazat?",
        "Syntaxe je jednoduchá: rm název_souboru",
        "Použijte 'rm stary_log.txt'.",
    ]
    start_directory = "cleanup"
    fixture = WorkspaceFixture(
        files=(FileFixture("cleanup/stary_log.txt", "old data"),),
    )
    completion = Completion(
        answer=ExactAnswer("stary_log.txt"),
        requirements=(FileExists("cleanup/stary_log.txt", should_exist=False),),
    )


@section.level(5)
class DeleteDirectoryLevel(Level):
    solution = Solution(steps=(RunShell("rm -r temp"),), answer="temp")
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
        `shellgame submit temp`
        """
    hints = [
        "Zkuste nejdřív 'rmdir temp'. Co se stane?",
        "Pokud adresář není prázdný, rmdir selže. Jaký přepínač potřebujete pro rekurzivní mazání?",
        "Použijte 'rm -r temp' pro smazání adresáře včetně obsahu.",
    ]
    start_directory = "cleanup"
    fixture = WorkspaceFixture(files=(FileFixture("cleanup/temp/junk.txt", "junk"),))
    completion = Completion(
        answer=ExactAnswer("temp"),
        requirements=(DirectoryExists("cleanup/temp", should_exist=False),),
    )


@section.level(6)
class ProjectScaffoldLevel(Level):
    solution = Solution(
        steps=(RunShell("mkdir -p web/css web/js && touch web/index.html web/css/style.css"),),
        answer="web",
    )
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
        `shellgame submit web`
        """
    hints = [
        "Nejdřív vytvořte adresáře: 'mkdir -p web/css web/js'",
        "Pak vytvořte soubory: 'touch web/index.html web/css/style.css'",
    ]
    extension = True
    start_directory = "project"
    fixture = WorkspaceFixture(
        directories=("project",),
        clean=("project/web",),
    )
    completion = Completion(
        answer=ExactAnswer("web"),
        requirements=(
            FileExists("project/web/index.html"),
            FileExists("project/web/css/style.css"),
            DirectoryExists("project/web/js"),
        ),
    )


@section.level(7)
class CleanupMultipleFilesLevel(Level):
    solution = Solution(steps=(RunShell("rm error.log temp.dat junk.tmp"),), answer="mess")
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
        `shellgame submit mess`
        """
    hints = [
        "Příkaz 'rm' dokáže smazat více souborů najednou, stačí je uvést oddělené mezerami.",
        "Spusťte 'rm error.log temp.dat junk.tmp' (nebo je smažte postupně po jednom).",
        "Ověřte pomocí 'ls', že zbyly jen potřebné soubory, a odevzdejte 'mess'.",
    ]
    optional = True
    start_directory = "mess"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("mess/error.log"),
            FileFixture("mess/temp.dat"),
            FileFixture("mess/junk.tmp"),
            FileFixture("mess/keep_me.txt", "important"),
        )
    )
    completion = Completion(
        answer=ExactAnswer("mess"),
        requirements=(
            FileExists("mess/error.log", should_exist=False),
            FileExists("mess/temp.dat", should_exist=False),
            FileExists("mess/junk.tmp", should_exist=False),
            FileExists("mess/keep_me.txt"),
        ),
    )


@section.level(8)
class SectionChallengeLevel(Level):
    solution = Solution(
        steps=(
            RunShell("mkdir -p myproject/src myproject/docs"),
            RunShell("touch myproject/README.md"),
            RunShell("rm delete_me.txt && rmdir empty_dir"),
        ),
        answer="builder",
    )
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
        "Zkontrolujte strukturu pomocí 'ls -R myproject' a pak odevzdejte heslo ze zadání.",
    ]
    start_directory = "challenge"
    fixture = WorkspaceFixture(
        directories=("challenge/empty_dir",),
        files=(FileFixture("challenge/delete_me.txt", "Delete this!\n"),),
        clean=("challenge",),
    )
    completion = Completion(
        answer=ExactAnswer(
            "builder",
            case_sensitive=False,
            error_message="Heslo není správné. Splňte nejdřív všechny body úkolu.",
            required_message="Musíte zadat heslo.",
        ),
        requirements=(
            DirectoryExists(
                "challenge/myproject/src",
                error_message="Chybí adresář myproject/src. Použijte 'mkdir -p myproject/src'.",
            ),
            DirectoryExists(
                "challenge/myproject/docs",
                error_message="Chybí adresář myproject/docs.",
            ),
            FileExists(
                "challenge/myproject/README.md",
                error_message="Chybí soubor myproject/README.md. Použijte 'touch myproject/README.md'.",
            ),
            FileExists(
                "challenge/delete_me.txt",
                should_exist=False,
                error_message="Soubor delete_me.txt nebyl smazán. Použijte 'rm delete_me.txt'.",
            ),
            DirectoryExists(
                "challenge/empty_dir",
                should_exist=False,
                error_message="Adresář empty_dir nebyl smazán. Použijte 'rmdir empty_dir'.",
            ),
        ),
    )
    success_message = "Brilantní! Dokončili jste Sekci 4. Umíte vytvářet i bourat!"
