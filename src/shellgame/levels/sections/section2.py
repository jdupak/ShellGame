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

        Odevzdejte název cílového adresáře:
        `shellgame submit finish`
        (nebo přímo v cíli: `shellgame submit`)
        """
    hints = [
        "Do sourozeneckého adresáře se dostanete přes rodičovský adresář ('..').",
        "Můžete použít 'cd ..' a pak 'cd finish', nebo to spojit do jednoho příkazu 'cd ../finish'.",
        "Po přesunu ověřte polohu příkazem 'pwd'. Odevzdejte poslední část cesty nebo prázdný 'shellgame submit'.",
    ]
    start_directory = "start"
    fixture = WorkspaceFixture(directories=("start", "finish"))
    completion = Completion(
        answer=ExactAnswer("finish"),
        requirements=(AtDirectory("finish"),),
        allow_empty=True,
    )
    success_message = "Správně! K sourozenci se chodí přes společného rodiče — nahoru a hned dolů jiným směrem."


@section.level(2)
class PreviousDirectoryToggleLevel(Level):
    solution = Solution(
        steps=(Chdir("location-B"), PerformCd("-", move_to="location-A")),
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
        4. Po návratu spusťte pouze `shellgame submit`.
        """
    hints = [
        "Příkaz 'cd -' vás vrátí do předchozího pracovního adresáře (jako tlačítko Zpět).",
        "Nejprve přejděte do 'location-B' ('cd ../location-B') a odtud zadejte 'cd -'.",
        "Po návratu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
    ]
    start_directory = "location-A"
    fixture = WorkspaceFixture(directories=("location-A", "location-B"))
    completion = Completion(
        requirements=(
            AtDirectory("location-A"),
            CdEvidence(
                "Nejdřív přejděte do `location-B` a vraťte se příkazem `cd -`.",
            ),
        ),
    )

    cd_policy = CdPolicy(
        scope=(WithTarget("-"),),
        rules=(RequireSourceDirectory("location-B", "Příkaz 'cd -' použijte až z adresáře location-B."),),
    )
    success_message = "Správně! Shell si pamatuje předchozí adresář, takže `cd -` přepíná mezi dvěma místy bez cesty."


@section.level(3)
class DeepRelativeNavigationLevel(Level):
    solution = Solution(steps=(PerformCd("../../other/target"),))
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

        V cíli spusťte pouze `shellgame submit`.
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Pro přechod do jiné větve stromu musíte nejprve vystoupat nahoru přes '..' a pak sestoupit dolů.",
        "Ze 'start' vystoupejte o dvě úrovně ('../..') a zadejte 'cd ../../other/target'.",
        "Po přesunu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
    ]
    start_directory = "deep/structure/start"
    fixture = WorkspaceFixture(
        directories=("deep/structure/start", "deep/other/target"),
    )
    completion = Completion(
        requirements=(
            AtDirectory("deep/other/target"),
            CdEvidence(
                "Použijte ze startu jeden relativní příkaz `cd ../../other/target`.",
            ),
        ),
    )

    cd_policy = CdPolicy(
        rules=(
            RequireSourceDirectory("deep/structure/start", "Použijte ze startu jeden příkaz 'cd ../../other/target'."),
            RequireExactCommand("../../other/target", "Použijte ze startu jeden příkaz 'cd ../../other/target'."),
        )
    )
    success_message = "Správně! Jedna relativní cesta umí obsahovat výstup i sestup — nejdřív `..`, potom jména větve."


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

        Odevzdejte pomocí: `shellgame submit <slovo>`
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
    success_message = "Správně! `cat` vysype celý obsah souboru do terminálu — ideální na krátké textové soubory."


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

        Odevzdejte pomocí: `shellgame submit <heslo>`
        Potřebujete pomoc? Napište: `shellgame hint`
        """
    hints = [
        "Začněte čtením start.txt - co vám soubor říká?",
        "Instrukce vás posílají někam dál. Jaký příkaz použijete pro přesun do adresáře?",
        "Přečtěte start.txt, přejděte do adresáře 'next', přečtěte clue.txt.",
    ]
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
    success_message = "Správně! Střídat `ls`, `cd` a `cat` stačí k prozkoumání libovolné neznámé struktury."


@section.level(6)
class CreateFileWithTouchLevel(Level):
    solution = Solution(steps=(RunShell("touch my_file.txt"),), answer=None)
    title = "Vytvoření souboru"
    instructions = """
        ### Cíl
        Vytvořte nový prázdný soubor.

        ### Příkazy k naučení
        - `touch <název>` (vytvoří prázdný soubor nebo aktualizuje časové značky existujícího souboru)

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
    start_directory = ""
    fixture = WorkspaceFixture(clean=("my_file.txt",))
    completion = Completion(requirements=(FileExists("my_file.txt"),))
    success_message = "Správně! `touch` založí prázdný soubor; u existujícího jen posune časové značky."


@section.level(7)
class HelpDiscoveryLevel(Level):
    title = "Jak najít pomoc"
    instructions = """
        # Jak najít pomoc

        Nikdo si nepamatuje všechny přepínače všech příkazů. Proto existuje nápověda!

        ### Dva způsoby, jak získat pomoc
        1. `ls --help` → stručný přehled přepínačů
        2. `man ls` → podrobný manuál; ukončíte ho klávesou `q`

        ### Hledání v manuálu
        V `man ls` stiskněte `/`, napište `-h` a potvrďte Enterem.
        Klávesa `n` přejde na další výskyt.

        ## Úkol
        Pomocí `ls --help` nebo `man ls` zjistěte, v jakém formátu přepínač `-h` zobrazuje velikosti.

        ## Odevzdání
        `shellgame submit <hodnota>`
        """
    hints = [
        "Spusťte 'ls --help' a projděte seznam přepínačů.",
        "Alternativně otevřete 'man ls' a vyhledejte '-h' pomocí klávesy '/'.",
        "Odevzdejte termín, který dokumentace používá pro tento formát velikostí.",
    ]
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
            error_message="Odpověď není správně. Podívejte se na popis '-h' v 'ls --help' nebo 'man ls'.",
            required_message="Musíte zadat odpověď: shellgame submit <hodnota>",
        )
    )
    success_message = "Správně! Teď víte, jak najít pomoc. Příkaz --help a man jsou vaši nejlepší přátelé!"


@section.level(8)
class SectionChallengeLevel(Level):
    solution = Solution(
        steps=(Chdir("challenge/room1"), Chdir("challenge/room2")),
        answer="navigator",
    )
    title = "Integrační výzva"
    instructions = """
        ### Výzva: Propojte navigaci a čtení souborů

        Závěrečný úkol sekce: použijete v něm několik dovedností najednou.

        ### Úkol
        1. Začínáte v `level-2`. Přejděte do `challenge/room1`.
        2. Přečtěte soubor `hint.txt`.
        3. Podle stopy přejděte do `room2`.
        4. Přečtěte `password.txt` a odevzdejte nalezené heslo.

        ### Odevzdání
        `shellgame submit <hodnota>`
        """
    hints = [
        "Ze startu přejděte do 'challenge/room1' a přečtěte 'hint.txt'.",
        "Do sousedního 'room2' se dostanete například příkazem 'cd ../room2'.",
        "V 'room2' přečtěte soubor 'password.txt' pomocí 'cat'.",
    ]
    start_directory = ""
    fixture = WorkspaceFixture(
        files=(
            FileFixture(
                "challenge/room1/hint.txt",
                "Heslo je v sousedním adresáři room2.\n",
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
            required_message="Musíte zadat heslo: shellgame submit <hodnota>",
        ),
        requirements=(AtDirectory("challenge/room2"),),
    )
    success_message = "Výborně! Dokončili jste Sekci 2 a umíte propojit navigaci se čtením souborů."
