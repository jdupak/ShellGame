"""File interaction and advanced navigation levels (Section 2)."""

from __future__ import annotations

from shellgame.levels.base import Level
from shellgame.levels.cdpolicy import (
    CdEvidence,
    CdPolicy,
    RequireExactCommand,
    RequireSourceDirectory,
    WithTarget,
)
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    AtDirectory,
    ChoiceAnswer,
    Completion,
    ExactAnswer,
    FileExists,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import Chdir, PerformCd, RunShell, Solution

section = Section(2, root="level-2")


@section.level(0)
class SectionIntroLevel(Level):
    is_intro = True
    title = "Práce se soubory"
    instructions_file = "section2_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    # Keep consistent behavior with existing tests/UX: do not force a start dir for intro.
    start_directory = None
    success_message = "Jdeme na to!"


@section.level(1)
class SiblingNavigationLevel(Level):
    solution = Solution(steps=(Chdir("finish"),), answer="finish")
    title = "Navigace mezi sourozenci"
    instructions = """
        ### Cíl
        Přejděte z jednoho podadresáře do druhého (sourozeneckého) adresáře.

        ### Příkazy k naučení
        - `cd ../<název>` (jít nahoru a hned dolů do jiného adresáře)

        ### Úkol
        Nacházíte se v adresáři `level-2/start`.
        Vaším úkolem je přejít do adresáře `level-2/finish`.

        Můžete to udělat ve dvou krocích:
        1. `cd ..` (zpět do level-2)
        2. `cd finish` (do cíle)

        Nebo v jednom kroku:
        `cd ../finish`

        Odevzdejte název adresáře, ve kterém se nacházíte.

        Odevzdejte pomocí: `shellgame submit [název-adresáře]`
        (nebo jen `shellgame submit` pokud jste v cíli)
        """
    hints = [
        "Do sourozeneckého adresáře se dostanete přes rodičovský adresář ('..').",
        "Můžete použít 'cd ..' a pak 'cd finish', nebo to spojit do jednoho příkazu 'cd ../finish'.",
        "Po přesunu ověřte polohu příkazem 'pwd'. Odevzdejte poslední část cesty nebo zadejte prázdný 'shellgame submit'.",
    ]
    start_directory = "start"
    fixture = WorkspaceFixture(directories=("start", "finish"))
    completion = Completion(
        answer=ExactAnswer("finish"),
        requirements=(AtDirectory("finish"),),
        allow_empty=True,
    )


@section.level(2)
class PreviousDirectoryToggleLevel(Level):
    solution = Solution(
        steps=(Chdir("location-B"), PerformCd("-", move_to="location-A")),
        answer="location-A",
    )
    title = "Rychlý návrat"
    instructions = """
        ### Cíl
        Naučte se rychle přepínat mezi dvěma adresáři.

        ### Příkazy k naučení
        - `cd -` (návrat do předchozího adresáře)

        ### Úkol
        1. Začínáte v `level-2/location-A`.
        2. Přejděte do `level-2/location-B` (použijte `cd ../location-B`).
        3. Použijte příkaz `cd -` pro okamžitý návrat zpět do `location-A`.
        4. Odevzdejte název adresáře, kde jste skončili.

        Odevzdejte pomocí: `shellgame submit [název-adresáře]`
        """
    hints = [
        "Příkaz 'cd -' vás vrátí do předchozího pracovního adresáře (jako tlačítko Zpět).",
        "Nejprve přejděte do 'location-B' ('cd ../location-B') a odtud zadejte 'cd -'.",
        "Po návratu ověřte polohu příkazem 'pwd' a odevzdejte poslední část cesty.",
    ]
    start_directory = "location-A"
    fixture = WorkspaceFixture(directories=("location-A", "location-B"))
    completion = Completion(
        answer=ExactAnswer("location-A"),
        requirements=(
            AtDirectory("location-A"),
            CdEvidence(
                "Nejdřív přejděte do `location-B` a vraťte se příkazem `cd -`.",
            ),
        ),
        allow_empty=True,
    )

    cd_policy = CdPolicy(
        scope=(WithTarget("-"),),
        rules=(RequireSourceDirectory("location-B", "Příkaz 'cd -' použijte až z adresáře location-B."),),
    )


@section.level(3)
class DeepRelativeNavigationLevel(Level):
    solution = Solution(steps=(PerformCd("../../other/target"),), answer="target")
    title = "Hluboká navigace"
    instructions = """
        ### Cíl
        Navigace složitější strukturou pomocí relativních cest.

        ### Úkol
        Nacházíte se hluboko ve struktuře adresářů.
        Vaším cílem je přejít do jiné větve stromu pomocí jediného příkazu `cd`.

        Start: `.../level-2/deep/structure/start`
        Cíl: `.../level-2/deep/other/target`

        Musíte jít o dvě úrovně výše a pak dolů do `other/target`.

        Odevzdejte název cílového adresáře.

        Odevzdejte pomocí: `shellgame submit [název-adresáře]`
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Pro přechod do jiné větve stromu musíte nejprve vystoupat nahoru přes '..' a pak sestoupit dolů.",
        "Ze 'start' vystoupejte o dvě úrovně ('../..') a zadejte 'cd ../../other/target'.",
        "Po přesunu ověřte polohu příkazem 'pwd' a odevzdejte poslední část cesty.",
    ]
    start_directory = "deep/structure/start"
    fixture = WorkspaceFixture(
        directories=("deep/structure/start", "deep/other/target"),
    )
    completion = Completion(
        answer=ExactAnswer("target"),
        requirements=(
            AtDirectory("deep/other/target"),
            CdEvidence(
                "Použijte ze startu jeden relativní příkaz `cd ../../other/target`.",
            ),
        ),
        allow_empty=True,
    )

    cd_policy = CdPolicy(
        rules=(
            RequireSourceDirectory("deep/structure/start", "Použijte ze startu jeden příkaz 'cd ../../other/target'."),
            RequireExactCommand("../../other/target", "Použijte ze startu jeden příkaz 'cd ../../other/target'."),
        )
    )


@section.level(4)
class ReadFirstWordLevel(Level):
    title = "Čtení souboru"
    instructions = """
        ### Cíl
        Přečtěte si obsah souboru.

        ### Příkazy k naučení
        - `cat <soubor>` (vypíše obsah souboru do terminálu)

        ### Úkol
        1. V aktuálním adresáři je soubor `message.txt`.
        2. Přečtěte si jeho obsah pomocí `cat message.txt`.
        3. Odevzdejte **PRVNÍ SLOVO**, které v souboru najdete.

        Odevzdejte pomocí: `shellgame submit [slovo]`
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Příkaz 'cat' vypíše obsah textového souboru na obrazovku.",
        "Spusťte 'cat message.txt' pro zobrazení obsahu souboru.",
        "Z výstupu vezměte jen první slovo (před první mezerou) a zadejte: 'shellgame submit <slovo>'.",
    ]
    start_directory = ""
    fixture = WorkspaceFixture(files=(FileFixture("message.txt", "Secret is the key.\n"),))
    completion = Completion(
        answer=ExactAnswer(
            "secret",
            case_sensitive=False,
            required_message="Musíte zadat první slovo ze zprávy: shellgame submit <slovo>",
        )
    )


@section.level(5)
class ChainedClueTraversalLevel(Level):
    title = "Sledování stop"
    instructions = """
        ### Cíl
        Sledujte stopy v souborech k nalezení hesla.

        ### Úkol
        1. Přečtěte si soubor `start.txt` v aktuálním adresáři.
        2. Postupujte podle instrukcí v souboru.
        3. Najděte finální heslo a odevzdejte ho.

        Odevzdejte pomocí: `shellgame submit [heslo]`
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Začněte čtením start.txt - co vám soubor říká?",
        "Instrukce vás posílají někam dál. Jaký příkaz použijete pro přesun do adresáře?",
        "Přečtěte start.txt, přejděte do adresáře 'next', přečtěte clue.txt.",
    ]
    extension = True
    start_directory = ""
    fixture = WorkspaceFixture(
        files=(
            FileFixture("start.txt", "Jděte do adresáře 'next' a přečtěte si clue.txt\n"),
            FileFixture("next/clue.txt", "Heslo je 'sunshine'\n"),
        )
    )
    completion = Completion(
        answer=ExactAnswer(
            "sunshine",
            case_sensitive=False,
            required_message="Musíte zadat heslo: shellgame submit <heslo>",
        )
    )


@section.level(6)
class CreateFileWithTouchLevel(Level):
    solution = Solution(steps=(RunShell("touch my_file.txt"),), answer=None)
    title = "Vytvoření souboru"
    instructions = """
        ### Cíl
        Vytvořte nový prázdný soubor.

        ### Příkazy k naučení
        - `touch <název>` (vytvoří prázdný soubor nebo aktualizuje čas razítka)

        ### Úkol
        1. Vytvořte soubor s názvem `my_file.txt` v aktuálním adresáři.
        2. Ověřte jeho existenci pomocí `ls`.

        Odevzdejte pomocí: `shellgame submit`
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Příkaz 'touch' vytvoří prázdný soubor se zadaným názvem.",
        "Spusťte 'touch my_file.txt' v aktuálním adresáři.",
        "Ověřte vytvoření souboru příkazem 'ls' a odešlete: 'shellgame submit'.",
    ]
    optional = True
    start_directory = ""
    fixture = WorkspaceFixture(clean=("my_file.txt",))
    completion = Completion(requirements=(FileExists("my_file.txt"),))


@section.level(7)
class SectionChallengeLevel(Level):
    title = "Souhrn Sekce 2"
    instructions = """
        ### Výzva: Test dovedností Sekce 2

        Kombinujte navigaci a čtení souborů!

        ### Úkol
        1. Začínáte v `level-2`. Přejděte do `challenge` (`cd challenge`)
        2. Odtud vstupte do `room1` (`cd room1`)
        3. Přečtěte `hint.txt` - řekne vám kam dál
        4. Použijte `cd -` pro návrat do `challenge` a pak pokračujte podle stopy
        5. Najděte soubor `password.txt` a přečtěte ho
        6. Odevzdejte heslo

        ### Shrnutí příkazů Sekce 2
        ```
        cd ../jiný    → Přechod na sourozence
        cd -          → Zpět kde jsem byl
        cat soubor    → Přečíst soubor
        touch soubor  → Vytvořit prázdný soubor
        ```

        ### Odevzdání
        `shellgame submit <heslo>`
        """
    hints = [
        "Ze startu použijte 'cd challenge', pak 'cd room1'. Přečtěte hint.txt.",
        "Hint vás pošle do room2. Použijte cd ../room2 nebo cd - a pak cd room2.",
        "V room2 je soubor password.txt. Přečtěte ho pomocí 'cat password.txt'.",
    ]
    start_directory = ""
    fixture = WorkspaceFixture(
        files=(
            FileFixture(
                "challenge/room1/hint.txt",
                "Heslo je v room2. Vraťte se zpět (cd -) a pak jděte do room2.\n",
            ),
            FileFixture("challenge/room2/password.txt", "navigator\n"),
            FileFixture("challenge/room2/decoy.txt", "Toto není heslo.\n"),
        )
    )
    completion = Completion(
        answer=ExactAnswer(
            "navigator",
            case_sensitive=False,
            mistakes={
                "toto není heslo": "To je obsah decoy.txt, ne password.txt. Přečtěte správný soubor.",
                "toto neni heslo": "To je obsah decoy.txt, ne password.txt. Přečtěte správný soubor.",
            },
            error_message="Heslo není správné. Hledejte password.txt v room2.",
            required_message="Musíte zadat heslo: shellgame submit <heslo>",
        )
    )
    success_message = "Výborně! Dokončili jste Sekci 2. Umíte navigovat a číst soubory!"


@section.level(8)
class HelpDiscoveryLevel(Level):
    title = "Jak najít pomoc"
    instructions = """
        # Jak najít pomoc

        Nikdo si nepamatuje všechny přepínače všech příkazů. Proto existuje nápověda!

        ### Dva způsoby, jak získat pomoc
        1. `příkaz --help` → Stručná nápověda (většina příkazů)
        2. `man příkaz` → Podrobný manuál (klávesa `q` pro ukončení)

        ### Příklady
        ```bash
        ls --help       # Rychlý přehled přepínačů
        man ls          # Kompletní dokumentace
        ```

        ### Tip: Hledání v manuálu
        V `man` můžete hledat: stiskněte `/`, napište hledaný text, Enter.
        Klávesa `n` = další výskyt, `q` = konec.

        ## Úkol
        Zjistěte, co dělá přepínač `-h` u příkazu `ls`.

        Použijte: `ls --help | grep -- "-h"` nebo si přečtěte `man ls`.

        Odpovězte: Přepínač -h zobrazuje velikosti v jakém formátu?
        (Odpověď je jedno anglické slovo)

        ## Odevzdání
        `shellgame submit <slovo>`
        """
    hints = [
        "Příkaz 'ls --help' vypíše všechny dostupné přepínače.",
        "Hledejte řádek s '-h' - říká něco o 'human readable' velikostech.",
        "V nápovědě vyhledejte popis přepínače -h; odevzdejte první slovo z výrazu 'human-readable'.",
    ]
    extension = True
    start_directory = ""
    completion = Completion(
        answer=ChoiceAnswer(
            (
                "human-readable",
                "human",
                "čitelné",
                "citelne",
                "čitelné formátování",
            ),
            case_sensitive=False,
            error_message="Odpověď není správně. Podívejte se na 'ls --help | grep -- \"-h\"'.",
            required_message="Musíte zadat odpověď: shellgame submit <odpověď>",
        )
    )
    success_message = "Správně! Teď víte, jak najít pomoc. Příkaz --help a man jsou vaši nejlepší přátelé!"
