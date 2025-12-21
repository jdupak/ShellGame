"""Section 9: Error Streams."""

import shutil
from pathlib import Path
from typing import Any, Optional

from shellgame.levels.base import Level
from shellgame.validation.validators import (
    StringValidator,
)


class Level9_0(Level):
    """Level 9.0: Section 9 Introduction."""

    def __init__(self) -> None:
        super().__init__(
            id="9.0",
            section=9,
            title="Sekce 9: Chybové výstupy",
            instructions_file="section9_intro.md",
            hints=["Přečtěte si úvod a pokračujte stisknutím Enter."],
        )

    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        pass

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Always valid."""
        return True, "Jdeme na to!"


class Level9_1(Level):
    """Level 9.1: Redirecting Errors."""

    def __init__(self) -> None:
        super().__init__(
            id="9.1",
            section=9,
            title="Přesměrování chyb",
            instructions="""# Level 9.1: Přesměrování chyb

Standardní chybový výstup (stderr) používá deskriptor souboru 2.
Pro přesměrování pouze chyb použijte `2>`.

### Proč je to důležité
Při běhu programů často chcete zachytit chybové hlášky do logu,
zatímco normální výstup zobrazíte uživateli. Oddělení stdout a stderr
je klíčové pro diagnostiku problémů.

### Úkol
V adresáři je skript `buggy.sh`, který vypisuje normální text i chybové zprávy.
Spusťte ho a přesměrujte POUZE chybové zprávy do souboru `errors.log`.

### Příkazy
- `./script 2> soubor` - přesměruje stderr do souboru

### Odevzdání
Odevzdejte název vytvořeného souboru.
`shellgame submit -f errors.log`""",
            hints=[
                "Běžný výstup jde na stdout (1), chyby na stderr (2). Jak přesměrujete jen dvojku?",
                "Syntaxe je: příkaz 2> soubor. Zkuste to se skriptem buggy.sh.",
                "Použijte './buggy.sh 2> errors.log'.",
            ],
            start_directory="level-9/errors",
        )

    def setup(self, workspace: Path) -> None:
        """Create buggy script."""
        level_dir = workspace / "level-9"
        level_dir.mkdir(parents=True, exist_ok=True)

        script_content = "#!/bin/bash\necho 'This is normal output'\necho 'This is an error message' >&2\n"

        script_path = level_dir / "buggy.sh"
        script_path.write_text(script_content)
        script_path.chmod(0o755)

        # Clean up
        if (level_dir / "errors.log").exists():
            (level_dir / "errors.log").unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate error log creation."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        str_val = StringValidator("errors.log")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        target = state.workspace / "level-9/errors.log"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "This is an error message" in content and "This is normal output" not in content:
            return True, "Správně! Soubor obsahuje pouze chyby."
        elif "This is normal output" in content:
            return False, "Soubor obsahuje i normální výstup (použili jste &> nebo chybí 2?)."
        else:
            return False, "Soubor neobsahuje očekávanou chybu."


class Level9_2(Level):
    """Level 9.2: Appending Errors."""

    def __init__(self) -> None:
        super().__init__(
            id="9.2",
            section=9,
            title="Přidávání chyb",
            instructions="""# Level 9.2: Přidávání chyb na konec souboru

Stejně jako u normálního výstupu můžete chyby přidávat na konec souboru pomocí `2>>`.

### Úkol
Spusťte `buggy.sh` znovu, ale tentokrát PŘIDEJTE chybové zprávy na konec `errors.log`.
Nepřepisujte existující chyby!

### Příkazy
- `./script 2>> soubor` - přidá stderr na konec souboru

### Odevzdání
Odevzdejte název souboru.
`shellgame submit -f errors.log`""",
            hints=[
                "Jaký je rozdíl mezi > a >>? Jeden přepisuje, druhý přidává.",
                "Pro přidání chyb na konec použijte dvě šipky: 2>>",
                "Použijte './buggy.sh 2>> errors.log'.",
            ],
            start_directory="level-9/errors",
        )

    def setup(self, workspace: Path) -> None:
        """Create initial error log."""
        level_dir = workspace / "level-9"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Ensure script exists
        script_path = level_dir / "buggy.sh"
        if not script_path.exists():
            script_content = "#!/bin/bash\necho 'This is normal output'\necho 'This is an error message' >&2\n"
            script_path.write_text(script_content)
            script_path.chmod(0o755)

        (level_dir / "errors.log").write_text("Old error 1\n")

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate append."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        str_val = StringValidator("errors.log")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        target = state.workspace / "level-9/errors.log"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "Old error 1" in content and "This is an error message" in content:
            return True, "Správně!"
        elif "Old error 1" not in content:
            return False, "Původní obsah zmizel (použili jste 2> místo 2>>?)."
        else:
            return False, "Soubor neobsahuje novou chybu."


class Level9_3(Level):
    """Level 9.3: All Output."""

    def __init__(self) -> None:
        super().__init__(
            id="9.3",
            section=9,
            title="Všechny výstupy",
            instructions="""# Level 9.3: Zachycení všech výstupů

Někdy chcete zachytit VŠECHNO - normální výstup i chyby do jednoho souboru.
K tomu slouží `&>`.

### Proč je to užitečné
Při ladění skriptů nebo automatizaci často potřebujete kompletní log
všeho, co program vypsal - ať už to byla informace nebo chyba.

### Úkol
Spusťte `buggy.sh` a přesměrujte OBOJÍ (stdout i stderr) do `all_output.log`.

### Příkazy
- `./script &> soubor` - přesměruje stdout i stderr

### Odevzdání
Odevzdejte název souboru.
`shellgame submit -f all_output.log`""",
            hints=[
                "Ampersand (&) v tomto kontextu znamená 'obojí' - stdout i stderr.",
                "Kombinace &> je zkratka pro přesměrování obou výstupů.",
                "Použijte './buggy.sh &> all_output.log'.",
            ],
            start_directory="level-9/errors",
        )

    def setup(self, workspace: Path) -> None:
        """Clean up."""
        level_dir = workspace / "level-9"
        level_dir.mkdir(parents=True, exist_ok=True)

        if (level_dir / "all_output.log").exists():
            (level_dir / "all_output.log").unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate all output."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        str_val = StringValidator("all_output.log")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        target = state.workspace / "level-9/all_output.log"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "This is normal output" in content and "This is an error message" in content:
            return True, "Správně! Máme všechno."
        else:
            return False, "Soubor neobsahuje oba typy výstupů."


class Level9_4(Level):
    """Level 9.4: The Black Hole."""

    def __init__(self) -> None:
        super().__init__(
            id="9.4",
            section=9,
            title="Černá díra",
            instructions="""# Level 9.4: Černá díra /dev/null

`/dev/null` je speciální soubor, který zahodí všechno, co do něj pošlete.
Je užitečný pro umlčení hlučných příkazů.

### Proč je to užitečné
Některé příkazy vypisují spoustu informací, které nepotřebujete.
Místo zahlcení obrazovky je můžete "poslat do černé díry".

### Úkol
Spusťte `buggy.sh` a umlčte VŠECHNY výstupy (stdout i stderr) přesměrováním do `/dev/null`.

### Příkazy
- `./script &> /dev/null` - zahodí veškerý výstup

### Odevzdání
Odevzdejte název speciálního souboru, který jste použili.
`shellgame submit -f /dev/null`""",
            hints=[
                "Kam v Linuxu 'vyhodíte' data, která nechcete? Existuje speciální soubor...",
                "Soubor /dev/null je jako černá díra - vše pohltí a nic nevrátí.",
                "Použijte './buggy.sh &> /dev/null'.",
            ],
            start_directory="level-9/errors",
        )

    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        pass

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate /dev/null usage."""
        if answer is None:
            return False, "Zadejte speciální soubor, který jste použili."

        str_val = StringValidator("/dev/null")
        return str_val.validate(answer, state.workspace)


class Level9_5(Level):
    """Level 9.5: Section 9 Summary."""

    def __init__(self) -> None:
        super().__init__(
            id="9.5",
            section=9,
            title="Souhrn Sekce 9",
            instructions="""### 🎯 Výzva: Mistr streamů

Ukažte, že rozumíte stdout, stderr a /dev/null!

### Úkol
V `level-9/challenge` je skript `mixed.sh` který vypisuje:
- normální výstup na stdout
- chyby na stderr

1. Spusťte skript a uložte **pouze chyby** do `errors.log`
2. Spusťte znovu a uložte **pouze normální výstup** do `output.log`

Odpovězte: kolik řádků má errors.log a kolik output.log?
Formát: `chyby,výstup` (např. `3,5`)

### Shrnutí příkazů Sekce 9
```
./skript > out.txt       → stdout do souboru
./skript 2> err.txt      → stderr do souboru
./skript &> all.txt      → vše do souboru
./skript 2>&1            → stderr do stdout
./skript > /dev/null     → zahodit stdout
```

### Odevzdání
`shellgame submit <chyby>,<výstup>`""",
            hints=[
                "Pro zachycení chyb: './mixed.sh 2> errors.log'. Pro normální výstup: './mixed.sh > output.log'.",
                "Počet řádků zjistíte pomocí 'wc -l errors.log output.log' nebo 'cat errors.log | wc -l'.",
                "errors.log má 2 řádky, output.log má 3 řádky. Odpověď je '2,3'.",
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create mixed output script."""
        challenge_dir = workspace / "level-9" / "challenge"

        if challenge_dir.exists():
            shutil.rmtree(challenge_dir)

        challenge_dir.mkdir(parents=True, exist_ok=True)

        # Script that outputs to both stdout and stderr
        script = challenge_dir / "mixed.sh"
        script.write_text(
            """#!/bin/bash
echo "Line 1 - normal output"
echo "ERROR: Something went wrong" >&2
echo "Line 2 - more output"
echo "ERROR: Another problem" >&2
echo "Line 3 - final output"
"""
        )
        script.chmod(0o755)

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate line counts."""
        if answer is None:
            return False, "Zadejte odpověď ve formátu: počet_chyb,počet_výstupů"

        answer = answer.strip()

        if "," not in answer:
            return False, "Formát odpovědi: chyby,výstup (např. 3,5)"

        parts = answer.split(",")
        try:
            errors = int(parts[0].strip())
            output = int(parts[1].strip())
        except ValueError:
            return False, "Obě hodnoty musí být čísla."

        if errors != 2:
            return (
                False,
                f"Počet chyb není {errors}. Spusťte './mixed.sh 2> errors.log' a pak 'wc -l errors.log'.",
            )

        if output != 3:
            return (
                False,
                f"Počet normálních řádků není {output}. Spusťte './mixed.sh > output.log' a pak 'wc -l output.log'.",
            )

        return (
            True,
            "🎉 Perfektní! Dokončili jste Sekci 9. Stdout a stderr jsou pro vás jako otevřená kniha!",
        )


def get_levels() -> list:
    """
    Return all Section 9 level classes.

    Returns:
        List of Level instances for Section 9
    """
    return [
        Level9_0(),
        Level9_1(),
        Level9_2(),
        Level9_3(),
        Level9_4(),
        Level9_5(),
    ]
