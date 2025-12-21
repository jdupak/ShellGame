"""Section 3: Hidden Files."""

from __future__ import annotations

from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import IntegerValidator, StringValidator


def _setup_level3_common(workspace: Path) -> None:
    """Common setup for Level 3 section."""
    level_dir = workspace / "level-3"
    level_dir.mkdir(parents=True, exist_ok=True)


class Level3_0(Level):
    """Level 3.0: Section 3 Introduction."""

    def __init__(self) -> None:
        super().__init__(
            id="3.0",
            section=3,
            title="Sekce 3: Skryté soubory",
            instructions_file="section3_intro.md",
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


class Level3_1(Level):
    """Level 3.1: Hidden Directory Count."""

    def __init__(self) -> None:
        super().__init__(
            id="3.1",
            section=3,
            title="Počítání skrytých adresářů",
            instructions="""### Cíl
Najděte a spočítejte skryté adresáře.

### Příkazy k naučení
- `ls -a` (zobrazí všechny soubory včetně skrytých)

### Úkol
Nacházíte se v adresáři `level-3/hub`.
1. Použijte `ls -a` pro zobrazení všech položek.
2. Spočítejte, kolik je zde **skrytých adresářů** (začínají tečkou).
3. **Důležité:** Do počtu NEZAHRNUJTE speciální adresáře `.` (aktuální) a `..` (nadřazený).

Odevzdejte počet nalezených skrytých adresářů (číslo).

Odevzdejte pomocí: `shellgame submit [číslo]`
Potřebujete pomoc? Napište: `shellgame hint`""",
            hints=[
                "Použijte 'ls -a' pro zobrazení skrytých položek.",
                "Hledejte adresáře začínající tečkou (např. .beta).",
                "Nepočítejte '.' a '..'.",
            ],
            start_directory="level-3/hub",
        )

    @override
    def setup(self, workspace: Path) -> None:
        """Create hidden directories."""
        _setup_level3_common(workspace)
        hub = workspace / "level-3" / "hub"
        hub.mkdir(parents=True, exist_ok=True)

        # Hidden directories
        (hub / ".beta").mkdir(exist_ok=True)
        (hub / ".gamma").mkdir(exist_ok=True)

        # Hidden files (distraction)
        (hub / ".config").write_text("")

        # Visible items
        (hub / "visible_dir").mkdir(exist_ok=True)
        (hub / "visible_file.txt").write_text("")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate count (2)."""
        if answer is None:
            return False, "Musíte zadat číslo."

        validator = IntegerValidator(2)
        return validator.validate(answer, state.workspace)


class Level3_2(Level):
    """Level 3.2: Reading a Hidden File."""

    def __init__(self) -> None:
        super().__init__(
            id="3.2",
            section=3,
            title="Čtení skrytého souboru",
            instructions="""### Cíl
Přečtěte obsah skrytého souboru.

### Úkol
V aktuálním adresáři je skrytý soubor `.secret_config`.
1. Ověřte jeho existenci pomocí `ls -a`.
2. Přečtěte jeho obsah pomocí `cat`.
3. Odevzdejte obsah souboru.

Odevzdejte pomocí: `shellgame submit [obsah]`
Potřebujete pomoc? Napište: `shellgame hint`""",
            hints=[
                "Skryté soubory začínají tečkou. Jak je zobrazíte pomocí ls?",
                "Příkaz 'cat' funguje i na skryté soubory - stačí zadat správný název včetně tečky.",
                "Zkuste: cat .secret_config",
            ],
            start_directory="level-3",
        )

    @override
    def setup(self, workspace: Path) -> None:
        """Create hidden file."""
        _setup_level3_common(workspace)
        level_dir = workspace / "level-3"
        (level_dir / ".secret_config").write_text("mode=stealth\n")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate content."""
        if answer is None:
            return False, "Musíte zadat obsah souboru."

        validator = StringValidator("mode=stealth", case_sensitive=True)
        return validator.validate(answer, state.workspace)


class Level3_3(Level):
    """Level 3.3: Inside a Hidden Directory."""

    def __init__(self) -> None:
        super().__init__(
            id="3.3",
            section=3,
            title="Uvnitř skrytého adresáře",
            instructions="""### Cíl
Vstupte do skrytého adresáře.

### Úkol
1. Najděte skrytý adresář `.vault`.
2. Vstupte do něj.
3. Uvnitř najděte soubor `key.txt` a přečtěte ho.
4. Odevzdejte nalezený klíč.

Odevzdejte pomocí: `shellgame submit [klíč]`
Potřebujete pomoc? Napište: `shellgame hint`""",
            hints=[
                "Jak byste vstoupili do běžného adresáře? Stejně to funguje i se skrytými.",
                "Skrytý adresář .vault - jak se do něj dostanete pomocí cd?",
                "Jděte do .vault pomocí 'cd .vault', pak přečtěte key.txt.",
            ],
            extension=True,
            start_directory="level-3/hub",
        )

    @override
    def setup(self, workspace: Path) -> None:
        """Create hidden vault."""
        _setup_level3_common(workspace)
        vault = workspace / "level-3" / "hub" / ".vault"
        vault.mkdir(parents=True, exist_ok=True)
        (vault / "key.txt").write_text("platinum\n")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate key."""
        if answer is None:
            return False, "Musíte zadat nalezený klíč."

        validator = StringValidator("platinum", case_sensitive=False)
        return validator.validate(answer, state.workspace)


class Level3_4(Level):
    """Level 3.4: Hidden Filename Suffix."""

    def __init__(self) -> None:
        super().__init__(
            id="3.4",
            section=3,
            title="Skrytá záloha",
            instructions="""### Cíl
Identifikujte skrytý soubor podle přípony.

### Úkol
V adresáři `level-3/backup` je několik skrytých souborů.
Najděte ten, který má příponu `.bak` (záloha).
Odevzdejte jeho celý název.

Odevzdejte pomocí: `shellgame submit [název-souboru]`
Potřebujete pomoc? Napište: `shellgame hint`""",
            hints=[
                "Použijte 'ls -a' v adresáři backup.",
                "Hledejte soubor začínající tečkou a končící .bak.",
                "Odevzdejte celý název včetně tečky na začátku.",
            ],
            optional=True,
            start_directory="level-3/backup",
        )

    @override
    def setup(self, workspace: Path) -> None:
        """Create backup directory."""
        _setup_level3_common(workspace)
        backup = workspace / "level-3" / "backup"
        backup.mkdir(parents=True, exist_ok=True)

        (backup / ".config").write_text("")
        (backup / ".data.bak").write_text("")
        (backup / "normal.txt").write_text("")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Validate filename."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        validator = StringValidator(".data.bak", case_sensitive=True)
        return validator.validate(answer, state.workspace)


class Level3_5(Level):
    """Level 3.5: Section 3 Summary."""

    def __init__(self) -> None:
        super().__init__(
            id="3.5",
            section=3,
            title="Souhrn Sekce 3",
            instructions="""### 🎯 Výzva: Mistři skrytých souborů

Ukažte, že ovládáte práci se skrytými soubory!

### Úkol
V adresáři `level-3/final_test` jsou normální i skryté položky.

1. Spočítejte celkový počet **skrytých adresářů** (bez . a ..)
2. Najděte skrytý soubor `.secret_code`
3. Přečtěte jeho obsah
4. Odevzdejte: `<počet>,<obsah>` (např. `3,tajne123`)

### Shrnutí příkazů Sekce 3
```
ls -a         → Zobrazí vše včetně skrytých
ls -la        → Detailní výpis všeho
cat .soubor   → Přečíst skrytý soubor
cd .adresar   → Vstoupit do skrytého adresáře
```

### Odevzdání
`shellgame submit <počet>,<obsah>`""",
            hints=[
                "Skryté položky začínají tečkou. Použijte 'ls -la' pro zobrazení všeho včetně typů.",
                "Adresáře poznáte podle 'd' na začátku řádku v ls -l, nebo podle / na konci v ls -F.",
                "Jsou tam 2 skryté adresáře. Kód v .secret_code je 'hidden_master'.",
            ],
        )

    @override
    def setup(self, workspace: Path) -> None:
        """Create test structure."""
        _setup_level3_common(workspace)
        test_dir = workspace / "level-3" / "final_test"
        test_dir.mkdir(parents=True, exist_ok=True)

        # Hidden directories (2)
        (test_dir / ".hidden_dir1").mkdir(exist_ok=True)
        (test_dir / ".hidden_dir2").mkdir(exist_ok=True)

        # Hidden files
        (test_dir / ".secret_code").write_text("hidden_master\n")
        (test_dir / ".config").write_text("not this one\n")

        # Normal items (distractions)
        (test_dir / "visible_dir").mkdir(exist_ok=True)
        (test_dir / "readme.txt").write_text("Look for hidden items!\n")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:  # noqa: PLR0911
        """Validate count and code."""
        if answer is None:
            return False, "Musíte zadat odpověď ve formátu: počet,kód"

        answer = answer.strip()

        # Parse answer
        if "," not in answer:
            return False, "Formát odpovědi je: počet,kód (např. 3,tajne123)"

        parts = answer.split(",", 1)
        try:
            count = int(parts[0].strip())
            code = parts[1].strip().lower()
        except ValueError:
            return False, "První část musí být číslo (počet skrytých adresářů)."

        # Validate count
        if count != 2:
            if count == 4:
                return False, "Počítáte i . a .. - ty nepočítejte, jsou speciální."
            return (
                False,
                f"Počet skrytých adresářů není {count}. Zkontrolujte pomocí 'ls -la'.",
            )

        # Validate code
        if code == "not this one":
            return False, "To je obsah .config, ne .secret_code."
        if code != "hidden_master":
            return False, f"Kód '{code}' není správný. Přečtěte .secret_code."

        return (
            True,
            "🎉 Výborně! Dokončili jste Sekci 3. Skryté soubory před vámi nic neskryjí!",
        )


class Level3_6(Level):
    """Level 3.6: Self-reflection checkpoint."""

    def __init__(self) -> None:
        super().__init__(
            id="3.6",
            section=3,
            title="🪞 Kontrolní bod: Sebehodnocení",
            instructions="""### 🪞 Čas na zamyšlení!

Právě jste se naučili základy práce v terminálu:
- **Sekce 1**: Navigace (`pwd`, `cd`, `ls`)
- **Sekce 2**: Čtení souborů (`cat`, `ls -l`, `man`/`--help`)
- **Sekce 3**: Skryté soubory (`ls -a`, soubory začínající `.`)

### Úkol: Sebehodnocení

Na stupnici **1-5** ohodnoťte svou jistotu:
- **1** = Potřebuji víc procvičování
- **3** = Rozumím základům, ale občas váhám
- **5** = Cítím se jistě, mohu pokračovat

### Otázky k zamyšlení
1. Umím se pohybovat mezi adresáři pomocí `cd`?
2. Dokážu zobrazit skryté soubory?
3. Vím, jak přečíst obsah souboru?
4. Umím najít nápovědu k příkazu?

### Odevzdání
Odevzdejte číslo 1-5 podle vaší jistoty.
- Pokud je vaše hodnocení **1-2**, doporučujeme vrátit se k předchozím sekcím (`shellgame jump 1.0`)
- Pokud je **3-5**, pokračujte dál!

`shellgame submit <1-5>`""",
            hints=[
                "Toto je sebehodnocení - neexistuje špatná odpověď!",
                "Buďte k sobě upřímní. Pokud váháte, vraťte se k předchozím levelům.",
                "Odevzdejte jakékoliv číslo od 1 do 5.",
            ],
            optional=True,
        )

    @override
    def setup(self, workspace: Path) -> None:
        """No setup needed for reflection."""
        return

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> tuple[bool, str]:
        """Accept any rating 1-5 with personalized feedback."""
        if answer is None:
            return False, "Odevzdejte číslo 1-5 podle vaší jistoty."

        try:
            rating = int(answer.strip())
        except ValueError:
            return False, "Odevzdejte číslo od 1 do 5."

        if rating < 1 or rating > 5:
            return False, "Hodnocení musí být od 1 do 5."

        if rating <= 2:
            return True, (
                "👍 Děkujeme za upřímnost! Doporučujeme vrátit se k "
                "předchozím sekcím pomocí 'shellgame jump 1.0' nebo "
                "'shellgame jump 2.0'. Opakování je matka moudrosti!"
            )
        elif rating == 3:
            return True, (
                "✨ Dobrý základ! Pokud si nejste jisti konkrétním příkazem, "
                "můžete se kdykoliv vrátit. Pokračujte na Sekci 4!"
            )
        else:
            return True, (
                "🚀 Skvělé! Máte solidní základy. Pokračujte na Sekci 4, kde se naučíte vytvářet a organizovat soubory!"
            )


def get_levels() -> list[Level]:
    """Return all Section 3 level instances."""
    return [
        Level3_0(),
        Level3_1(),
        Level3_2(),
        Level3_3(),
        Level3_4(),
        Level3_5(),
        Level3_6(),
    ]
