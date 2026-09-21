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
        Přejděte z aktuálního podadresáře do sousedního (sourozeneckého) adresáře.

        ### Struktura adresářů
        ```
        level-2/
        ├── start/       ← Zde se nacházíte (start)
        └── finish/      ← Váš cíl
        ```

        ### Navigace mezi sourozenci
        Adresáře `start` a `finish` leží vedle sebe ve stejném rodičovském adresáři (`level-2`).
        Protože `finish` neleží uvnitř `start`, nelze použít přímý příkaz `cd finish`.
        Musíte se nejprve vrátit k rodiči (`..`) a odtud vstoupit do cíle:
        - Ve dvou krocích: nejprve o úroveň výše (`cd ..`) a potom do cíle.
        - Nebo v jednom kroku spojenou relativní cestou: `cd ../<cílový_adresář>`.

        ### Úkol
        1. Začínáte v adresáři `level-2/start`
        2. Přejděte do sousedního adresáře `finish`
        3. V cíli ověřte polohu příkazem `pwd` a odešlete řešení

        ### Odevzdání
        V cílovém adresáři spusťte:
        `shellgame submit`
        """
    hints = [
        "Do sourozeneckého adresáře se dostanete přes společného rodiče ('..').",
        "Zkuste nejprve vystoupat o úroveň výše nebo použít spojenou relativní cestu začínající '../'.",
        "Po přesunu ověřte polohu příkazem 'pwd' a spusťte 'shellgame submit'.",
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

        ### ⚠️ Bezpečnostní pojistka: Ctrl+C
        Kdybyste omylem spustili `cat` bez názvu souboru, příkaz neví, co číst,
        a začne čekat na vstup z klávesnice (terminál se zdánlivě „zasekne“).
        Kdykoliv se vám to stane, stiskněte **Ctrl+C** — to běžící příkaz okamžitě přeruší.

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
    solution = Solution(answer="human-readable")
    title = "Jak najít pomoc"
    instructions = """
        ### Cíl
        Naučte se vyhledávat v nápovědě k příkazům.

        ### Dva způsoby, jak získat pomoc
        Nikdo si nepamatuje všechny přepínače všech příkazů:
        1. `<příkaz> --help` → stručný přehled přepínačů přímo v terminálu
        2. `man <příkaz>` → podrobný manuál (stránkovaný; ukončíte ho klávesou `q`)

        ### 💡 Hledání v manuálu (`man`)
        V manuálu můžete vyhledávat:
        - Stiskněte klávesu `/`, napište hledaný text (např. `-h`) a stiskněte **Enter**.
        - Klávesou `n` přejdete na další výskyt.
        - Klávesou `q` manuál ukončíte.

        ### Úkol
        1. Spusťte `ls --help` nebo otevřete manuál `man ls`.
        2. Vyhledejte přepínač `-h`.
        3. Krátké přepínače mívají svůj dlouhý ekvivalent začínající na `--` (např. `-a` má `--all`).
           Jaký dlouhý název má přepínač `-h`?
        4. Odevzdejte tento dlouhý název (např. `human-readable` nebo `--human-readable`).

        ### Odevzdání
        `shellgame submit <název>`
        """
    hints = [
        "Spusťte 'ls --help' nebo 'man ls' a vyhledejte řádek s přepínačem '-h'.",
        "V nápovědě uvidíte zápis ve tvaru '-h, --název'. Hledejte slovo za dvěma pomlčkami.",
        "Přepínač '-h' je zkratka pro 'human-readable'. Odevzdejte: shellgame submit human-readable",
    ]
    start_directory = ""
    completion = Completion(
        answer=ChoiceAnswer(
            (
                "human-readable",
                "--human-readable",
                "human readable",
                "human",
                "čitelné",
                "citelne",
                "čitelné formátování",
            ),
            case_sensitive=False,
            error_message=(
                "Odpověď není správně. V nápovědě vyhledejte řádek s přepínačem '-h' "
                "a najděte jeho dlouhý název (--...)."
            ),
            required_message="Musíte zadat odpověď: shellgame submit <název>",
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
