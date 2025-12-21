"""Section 2: File Interaction and Advanced Navigation."""

from __future__ import annotations

from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    BasenameValidator,
    FileExistsValidator,
    StringValidator,
)


def _setup_level2_common(workspace: Path) -> None:
    """Common setup for Level 2 section."""
    level_dir = workspace / "level-2"
    level_dir.mkdir(parents=True, exist_ok=True)


class Level2_0(Level):
    """Level 2.0: Section 2 Introduction."""

    def __init__(self) -> None:
        super().__init__(
            id="2.0",
            section=2,
            title="Sekce 2: Práce se soubory",
            instructions_file="section2_intro.md",
            hints=["Přečtěte si úvod a pokračujte stisknutím Enter."],
        )

    @override
    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        return

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Always valid."""
        return True, "Jdeme na to!"


class Level2_1(Level):
    """Level 2.1: Sibling Navigation Drill."""

    def __init__(self) -> None:
        super().__init__(
            id="2.1",
            section=2,
            title="Navigace mezi sourozenci",
            instructions="""### Cíl
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
Potřebujete pomoc? Napište: `shellgame hint`""",
            hints=[
                "Použijte 'cd ../finish' pro přechod do cílového adresáře.",
                "Odevzdejte název adresáře 'finish'.",
            ],
            start_directory="level-2/start",
        )

    @override
    def setup(self, workspace: Path) -> None:
        """Create sibling directories."""
        _setup_level2_common(workspace)
        level_dir = workspace / "level-2"
        (level_dir / "start").mkdir(exist_ok=True)
        (level_dir / "finish").mkdir(exist_ok=True)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate user is in 'finish' directory."""
        if answer is None:
            if Path.cwd().name == "finish":
                return True, "Správně! Jste v cíli."
            else:
                return (
                    False,
                    f"Jste v '{Path.cwd().name}', ale měli byste být v 'finish'.",
                )

        validator = BasenameValidator("finish")
        return validator.validate(answer, state.workspace)


class Level2_2(Level):
    """Level 2.2: Previous Directory Toggle."""

    def __init__(self) -> None:
        super().__init__(
            id="2.2",
            section=2,
            title="Rychlý návrat",
            instructions="""### Cíl
Naučte se rychle přepínat mezi dvěma adresáři.

### Příkazy k naučení
- `cd -` (návrat do předchozího adresáře)

### Úkol
1. Začínáte v `level-2/location-A`.
2. Přejděte do `level-2/location-B` (použijte `cd ../location-B`).
3. Použijte příkaz `cd -` pro okamžitý návrat zpět do `location-A`.
4. Odevzdejte název adresáře, kde jste skončili.

Odevzdejte pomocí: `shellgame submit [název-adresáře]`
Potřebujete pomoc? Napište: `shellgame hint`""",
            hints=[
                "Přejděte do location-B, pak použijte 'cd -' pro návrat.",
                "Odevzdejte název 'location-A'.",
            ],
            start_directory="level-2/location-A",
        )

    def setup(self, workspace: Path) -> None:
        """Create A and B directories."""
        _setup_level2_common(workspace)
        level_dir = workspace / "level-2"
        (level_dir / "location-A").mkdir(exist_ok=True)
        (level_dir / "location-B").mkdir(exist_ok=True)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate user is back in location-A."""
        if answer is None:
            if Path.cwd().name == "location-A":
                return True, "Správně! Vrátili jste se zpět."
            return (
                False,
                f"Jste v '{Path.cwd().name}', ale měli byste být v 'location-A'.",
            )

        validator = BasenameValidator("location-A")
        return validator.validate(answer, state.workspace)


class Level2_3(Level):
    """Level 2.3: Deep Relative Ascent & Descent."""

    def __init__(self) -> None:
        super().__init__(
            id="2.3",
            section=2,
            title="Hluboká navigace",
            instructions="""### Cíl
Navigace složitější strukturou pomocí relativních cest.

### Úkol
Nacházíte se hluboko ve struktuře adresářů.
Vaším cílem je přejít do jiné větve stromu pomocí jediného příkazu `cd`.

Start: `.../level-2/deep/structure/start`
Cíl: `.../level-2/deep/other/target`

Musíte jít o dvě úrovně výše a pak dolů do `other/target`.

Odevzdejte název cílového adresáře.

Odevzdejte pomocí: `shellgame submit [název-adresáře]`
Potřebujete pomoc? Napište: `shellgame hint`""",
            hints=[
                "Použijte 'cd ../../other/target' pro přesun.",
                "Odevzdejte název 'target'.",
            ],
            start_directory="level-2/deep/structure/start",
        )

    def setup(self, workspace: Path) -> None:
        """Create deep structure."""
        _setup_level2_common(workspace)
        base = workspace / "level-2" / "deep"
        (base / "structure" / "start").mkdir(parents=True, exist_ok=True)
        (base / "other" / "target").mkdir(parents=True, exist_ok=True)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate user is in target."""
        if answer is None:
            if Path.cwd().name == "target":
                return True, "Správně! Našli jste cestu."
            return (
                False,
                f"Jste v '{Path.cwd().name}', ale měli byste být v 'target'.",
            )

        validator = BasenameValidator("target")
        return validator.validate(answer, state.workspace)


class Level2_4(Level):
    """Level 2.4: Reading a File's First Word."""

    def __init__(self) -> None:
        super().__init__(
            id="2.4",
            section=2,
            title="Čtení souboru",
            instructions="""### Cíl
Přečtěte si obsah souboru.

### Příkazy k naučení
- `cat <soubor>` (vypíše obsah souboru do terminálu)

### Úkol
1. V aktuálním adresáři je soubor `message.txt`.
2. Přečtěte si jeho obsah pomocí `cat message.txt`.
3. Odevzdejte **PRVNÍ SLOVO**, které v souboru najdete.

Odevzdejte pomocí: `shellgame submit [slovo]`
Potřebujete pomoc? Napište: `shellgame hint`""",
            hints=[
                "Použijte 'cat message.txt' pro zobrazení obsahu.",
                "První slovo je 'Secret'.",
            ],
            start_directory="level-2",
        )

    def setup(self, workspace: Path) -> None:
        """Create message file."""
        _setup_level2_common(workspace)
        level_dir = workspace / "level-2"
        (level_dir / "message.txt").write_text("Secret is the key.\n")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate the secret word."""
        if answer is None:
            return False, "Musíte zadat první slovo ze zprávy: shellgame submit <slovo>"

        validator = StringValidator("secret", case_sensitive=False)
        return validator.validate(answer, state.workspace)


class Level2_5(Level):
    """Level 2.5: Chained Clue Traversal."""

    def __init__(self) -> None:
        super().__init__(
            id="2.5",
            section=2,
            title="Sledování stop",
            instructions="""### Cíl
Sledujte stopy v souborech k nalezení hesla.

### Úkol
1. Přečtěte si soubor `start.txt` v aktuálním adresáři.
2. Postupujte podle instrukcí v souboru.
3. Najděte finální heslo a odevzdejte ho.

Odevzdejte pomocí: `shellgame submit [heslo]`
Potřebujete pomoc? Napište: `shellgame hint`""",
            hints=[
                "Začněte čtením start.txt - co vám soubor říká?",
                "Instrukce vás posílají někam dál. Jaký příkaz použijete pro přesun do adresáře?",
                "Přečtěte start.txt, přejděte do adresáře 'next', přečtěte clue.txt.",
            ],
            extension=True,
            start_directory="level-2",
        )

    def setup(self, workspace: Path) -> None:
        """Create clue chain."""
        _setup_level2_common(workspace)
        level_dir = workspace / "level-2"

        # Start file
        (level_dir / "start.txt").write_text("Jděte do adresáře 'next' a přečtěte si clue.txt\n")

        # Next directory and clue
        next_dir = level_dir / "next"
        next_dir.mkdir(exist_ok=True)
        (next_dir / "clue.txt").write_text("Heslo je 'sunshine'\n")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate the password."""
        if answer is None:
            return False, "Musíte zadat heslo: shellgame submit <heslo>"

        validator = StringValidator("sunshine", case_sensitive=False)
        return validator.validate(answer, state.workspace)


class Level2_6(Level):
    """Level 2.6: Create & Confirm."""

    def __init__(self) -> None:
        super().__init__(
            id="2.6",
            section=2,
            title="Vytvoření souboru",
            instructions="""### Cíl
Vytvořte nový prázdný soubor.

### Příkazy k naučení
- `touch <název>` (vytvoří prázdný soubor nebo aktualizuje čas razítka)

### Úkol
1. Vytvořte soubor s názvem `my_file.txt` v aktuálním adresáři.
2. Ověřte jeho existenci pomocí `ls`.

Odevzdejte pomocí: `shellgame submit`
Potřebujete pomoc? Napište: `shellgame hint`""",
            hints=[
                "Použijte 'touch my_file.txt' pro vytvoření souboru.",
                "Pak spusťte 'shellgame submit'.",
            ],
            optional=True,
            start_directory="level-2",
        )

    def setup(self, workspace: Path) -> None:
        """Ensure clean slate."""
        _setup_level2_common(workspace)
        file_path = workspace / "level-2" / "my_file.txt"
        if file_path.exists():
            file_path.unlink()

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate file existence."""
        validator = FileExistsValidator("level-2/my_file.txt")
        return validator.validate(answer, state.workspace)


class Level2_7(Level):
    """Level 2.7: Section 2 Summary."""

    def __init__(self) -> None:
        super().__init__(
            id="2.7",
            section=2,
            title="Souhrn Sekce 2",
            instructions="""### 🎯 Výzva: Test dovedností Sekce 2

Kombinujte navigaci a čtení souborů!

### Úkol
1. Začněte v `level-2`
2. Přejděte do `level-2/challenge/room1`
3. Přečtěte `hint.txt` - řekne vám kam dál
4. Použijte `cd -` pro návrat a pak pokračujte
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
`shellgame submit <heslo>`""",
            hints=[
                "V room1 najdete hint.txt. Co říká?",
                "Hint vás pošle do room2. Použijte cd ../room2 nebo cd - a pak cd room2.",
                "V room2 je password.txt s heslem 'navigator'.",
            ],
            start_directory="level-2",
        )

    def setup(self, workspace: Path) -> None:
        """Create challenge structure."""
        _setup_level2_common(workspace)
        challenge = workspace / "level-2" / "challenge"
        room1 = challenge / "room1"
        room2 = challenge / "room2"

        room1.mkdir(parents=True, exist_ok=True)
        room2.mkdir(parents=True, exist_ok=True)

        (room1 / "hint.txt").write_text("Heslo je v room2. Vraťte se zpět (cd -) a pak jděte do room2.\n")
        (room2 / "password.txt").write_text("navigator\n")
        (room2 / "decoy.txt").write_text("Toto není heslo.\n")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate password."""
        if answer is None:
            return False, "Musíte zadat heslo: shellgame submit <heslo>"

        cleaned = answer.strip().lower()

        if cleaned == "navigator":
            return (
                True,
                "🎉 Výborně! Dokončili jste Sekci 2. Umíte navigovat a číst soubory!",
            )
        if cleaned in {"toto není heslo", "toto neni heslo"}:
            return (
                False,
                "To je obsah decoy.txt, ne password.txt. Přečtěte správný soubor.",
            )
        return (
            False,
            f"'{cleaned}' není správné heslo. Hledejte password.txt v room2.",
        )


class Level2_8(Level):
    """Level 2.8: Getting Help."""

    def __init__(self) -> None:
        super().__init__(
            id="2.8",
            section=2,
            title="Jak najít pomoc",
            instructions="""# Level 2.8: Jak najít pomoc

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
`shellgame submit <slovo>`""",
            hints=[
                "Příkaz 'ls --help' vypíše všechny dostupné přepínače.",
                "Hledejte řádek s '-h' - říká něco o 'human readable' velikostech.",
                "Odpověď je 'human' (human-readable = čitelné pro člověka, např. 1K, 2M, 3G).",
            ],
            extension=True,
            start_directory="level-2",
        )

    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        pass

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate help understanding."""
        if answer is None:
            return False, "Musíte zadat odpověď: shellgame submit <odpověď>"

        cleaned = answer.strip().lower()
        if cleaned in [
            "human-readable",
            "human",
            "čitelné",
            "citelne",
            "čitelné formátování",
        ]:
            return (
                True,
                "Správně! Teď víte, jak najít pomoc. Příkaz --help a man jsou vaši nejlepší přátelé!",
            )
        if cleaned in ["readable", "čitelné", "citelne"]:
            return (
                False,
                "Blízko! Hledáme celý termín - 'human-readable'. Zkráceně stačí 'human'.",
            )
        return (
            False,
            f"'{cleaned}' není správně. Podívejte se na 'ls --help | grep -- \"-h\"'.",
        )


def get_levels() -> list[Level]:
    """
    Return all Section 2 level classes.

    Returns:
        List of Level instances for Section 2
    """
    return [
        Level2_0(),
        Level2_1(),
        Level2_2(),
        Level2_3(),
        Level2_4(),
        Level2_5(),
        Level2_6(),
        Level2_7(),
        Level2_8(),
    ]
