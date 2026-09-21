"""Section 4: Creation and Cleanup (declarative style)."""

from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    Completion,
    DirectoryExists,
    ExactAnswer,
    FileExists,
    TextFileContent,
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
    solution = Solution(steps=(RunShell("touch novy_soubor.txt"),), answer=None)
    title = "Vytvoření souboru"
    instructions = """
        Příkaz `touch` slouží k vytvoření prázdného souboru. U existujícího souboru
        aktualizuje jeho časové značky.

        ## Úkol
        Vytvořte prázdný soubor s názvem `novy_soubor.txt` v aktuálním adresáři.

        ## Příkazy
        - `touch <název_souboru>`

        ## Odevzdání
        Až bude soubor existovat a bude zcela prázdný, spusťte:
        `shellgame submit`
        """
    hints = [
        "Příkaz 'touch' vytvoří prázdný soubor. Jaký název má mít?",
        "Syntaxe je jednoduchá: touch název_souboru",
        "Použijte 'touch novy_soubor.txt', ověřte výsledek pomocí 'ls -l' a spusťte 'shellgame submit'.",
    ]
    start_directory = "creation"
    fixture = WorkspaceFixture(
        directories=("creation",),
        clean=("creation/novy_soubor.txt",),
    )
    completion = Completion(
        requirements=(
            FileExists("creation/novy_soubor.txt"),
            TextFileContent(
                "creation/novy_soubor.txt",
                exact="",
                error_message="Soubor novy_soubor.txt existuje, ale není prázdný.",
            ),
        ),
    )
    success_message = "Správně! Soubor může vzniknout úplně prázdný a obsah do něj doplníte až později."


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
        `shellgame submit <název>`
        """
    hints = [
        "Příkaz 'mkdir' slouží k vytvoření nového adresáře.",
        "Spusťte 'mkdir data' pro vytvoření adresáře data.",
        "Ověřte pomocí 'ls -l' (řádek adresáře začíná 'd') a odevzdejte 'data'.",
    ]
    start_directory = "creation"
    fixture = WorkspaceFixture(
        directories=("creation",),
        clean=("creation/data",),
    )
    completion = Completion(
        answer=ExactAnswer("data"),
        requirements=(
            DirectoryExists(
                "creation/data",
                error_message=(
                    "Požadovaný adresář zatím neexistuje. Ověřte pomocí 'ls -l', "
                    "že jste ho založili v aktuálním adresáři a bez překlepu v názvu."
                ),
            ),
        ),
    )
    success_message = "Správně! Nový adresář vznikne jediným příkazem a hned do něj můžete vstoupit."


@section.level(3)
class NestedDirectoryCreationLevel(Level):
    solution = Solution(steps=(RunShell("mkdir -p projekt/src/tests"),), answer=None)
    title = "Vytváření zanořených adresářů"
    instructions = """
        Pokud chcete vytvořit celou cestu adresářů najednou (např. `projekt/src/main`),
        příkaz `mkdir` by normálně selhal, pokud rodičovské adresáře neexistují.

        Přepínač `-p` (parents) řekne příkazu `mkdir`, aby vytvořil i všechny chybějící
        rodičovské adresáře.

        ## Úkol
        Vytvořte strukturu adresářů `projekt/src/tests`.

        ## Příkazy
        - `mkdir -p <cesta>`

        ## Odevzdání
        Až bude celá struktura vytvořená, spusťte:
        `shellgame submit`
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
        requirements=(
            DirectoryExists(
                "nested/projekt/src/tests",
                error_message=(
                    "Struktura projekt/src/tests zatím není celá. Zkontrolujte ji pomocí 'ls -R projekt': "
                    "bez přepínače -p vytvoří mkdir jen poslední článek cesty, a to jen když jeho rodič už existuje. "
                    "S -p vzniknou chybějící rodičovské adresáře zároveň s ním."
                ),
            ),
        )
    )
    success_message = "Správně! Přepínač -p vytvoří celou cestu naráz, takže rodiče nemusíte zakládat po jednom."


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
        `shellgame submit <název>`
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
        requirements=(
            FileExists(
                "cleanup/stary_log.txt",
                should_exist=False,
                error_message=(
                    "Soubor stary_log.txt tu pořád je. Zkontrolujte pomocí 'ls', že mažete ve správném "
                    "adresáři a že jste název napsali bez překlepu."
                ),
            ),
        ),
    )
    success_message = "Správně! Mazání v shellu je okamžité a nevratné, proto se vyplatí nejdřív ověřit, co mažete."


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
        `shellgame submit <název>`
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
        requirements=(
            DirectoryExists(
                "cleanup/temp",
                should_exist=False,
                error_message=(
                    "Zadaný adresář tu pořád je. Pokud rmdir skončil hláškou 'Directory not empty', "
                    "pracoval přesně podle očekávání: umí odstranit jen prázdný adresář a nesmí se dotknout obsahu. "
                    "Adresář s obsahem proto smažte až rekurzivní variantou příkazu rm."
                ),
            ),
        ),
    )
    success_message = "Správně! Prázdný adresář zvládne rmdir, na adresář s obsahem je potřeba rekurzivní mazání."


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
        `shellgame submit <název>`
        """
    hints = [
        "Nejdřív si rozdělte úkol na adresáře a soubory. Všechny potřebné adresáře lze vytvořit jedním příkazem.",
        "Pro adresáře použijte 'mkdir -p' a pro prázdné soubory 'touch'; oba příkazy přijímají více cest.",
        "Spusťte 'mkdir -p web/css web/js' a potom 'touch web/index.html web/css/style.css'.",
    ]
    start_directory = "project"
    fixture = WorkspaceFixture(
        directories=("project",),
        clean=("project/web",),
    )
    completion = Completion(
        answer=ExactAnswer("web"),
        requirements=(
            FileExists(
                "project/web/index.html",
                error_message="Chybí soubor web/index.html. Prázdný soubor založí 'touch'.",
            ),
            FileExists(
                "project/web/css/style.css",
                error_message=(
                    "Chybí soubor web/css/style.css. Nejdřív musí existovat podadresář css, "
                    "teprve potom do něj lze soubor založit."
                ),
            ),
            DirectoryExists(
                "project/web/js",
                error_message="Chybí adresář web/js. Má zůstat prázdný, ale existovat musí.",
            ),
        ),
    )
    success_message = "Správně! Kostru projektu připravíte předem: adresáře jedním příkazem, prázdné soubory druhým."


@section.level(7)
class CleanupMultipleFilesLevel(Level):
    solution = Solution(steps=(RunShell("rm error.log temp.dat junk.tmp"),), answer=None)
    title = "Úklid nepořádku"
    instructions = """
        Příkaz `rm` přijímá více argumentů, ale soubory lze mazat také postupně.

        ## Úkol
        V adresáři `mess` smažte soubory `error.log`, `temp.dat` a `junk.tmp`.
        Soubor `keep_me.txt` musí zůstat zachovaný. Hodnotí se výsledný stav, ne počet příkazů.

        ## Příkazy
        - `rm soubor1 soubor2 soubor3`

        ## Odevzdání
        Až budou nepotřebné soubory pryč, spusťte:
        `shellgame submit`
        """
    hints = [
        "Příkaz 'rm' dokáže smazat více souborů najednou, stačí je uvést oddělené mezerami.",
        "Spusťte 'rm error.log temp.dat junk.tmp' (nebo je smažte postupně po jednom).",
        "Ověřte pomocí 'ls', že zůstal jen soubor keep_me.txt, a spusťte 'shellgame submit'.",
    ]
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
        requirements=(
            FileExists("mess/error.log", should_exist=False),
            FileExists("mess/temp.dat", should_exist=False),
            FileExists("mess/junk.tmp", should_exist=False),
            FileExists(
                "mess/keep_me.txt",
                error_message=(
                    "Soubor keep_me.txt zmizel, a ten měl zůstat. Typická příčina je příliš široký "
                    "argument jako 'rm *', který smaže všechno v adresáři. Příště si nejdřív vypište, "
                    "co by takový příkaz zasáhl. Level obnovíte příkazem 'shellgame reset'."
                ),
            ),
        ),
    )
    success_message = "Správně! Jeden příkaz zvládne víc cest, ale odpovídáte za to, které soubory zasáhne."


@section.level(8)
class SectionChallengeLevel(Level):
    solution = Solution(
        steps=(
            RunShell("mkdir -p myproject/src myproject/docs"),
            RunShell("touch myproject/README.md"),
            RunShell("rm delete_me.txt && rmdir empty_dir"),
        ),
        answer=None,
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
        Po splnění všech bodů spusťte `shellgame submit`.
        """
    hints = [
        (
            "Rozdělte úkol na výsledky: dva adresáře a jeden soubor vytvořit, "
            "dvě existující položky odstranit."
        ),
        (
            "Vnořené adresáře vytvoří 'mkdir -p'; prázdný soubor 'touch'. "
            "Soubor a prázdný adresář se mažou různými příkazy."
        ),
        (
            "Použijte 'mkdir -p myproject/src myproject/docs', 'touch myproject/README.md', "
            "'rm delete_me.txt' a 'rmdir empty_dir'."
        ),
    ]
    start_directory = "challenge"
    fixture = WorkspaceFixture(
        directories=("challenge/empty_dir",),
        files=(FileFixture("challenge/delete_me.txt", "Delete this!\n"),),
        clean=("challenge",),
    )
    completion = Completion(
        requirements=(
            DirectoryExists(
                "challenge/myproject/src",
                error_message="Chybí adresář myproject/src. Zkontrolujte vytvořenou strukturu.",
            ),
            DirectoryExists(
                "challenge/myproject/docs",
                error_message="Chybí adresář myproject/docs.",
            ),
            FileExists(
                "challenge/myproject/README.md",
                error_message="Chybí soubor myproject/README.md. Zkontrolujte jeho název a umístění.",
            ),
            FileExists(
                "challenge/delete_me.txt",
                should_exist=False,
                error_message="Soubor delete_me.txt stále existuje.",
            ),
            DirectoryExists(
                "challenge/empty_dir",
                should_exist=False,
                error_message="Prázdný adresář empty_dir stále existuje.",
            ),
        ),
    )
    success_message = "Brilantní! Dokončili jste Sekci 4. Umíte vytvářet i bourat!"
