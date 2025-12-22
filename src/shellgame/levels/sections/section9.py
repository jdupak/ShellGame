"""Section 9: Error Streams."""

from __future__ import annotations

import shutil
from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import StringValidator, ValidationResult


class SectionIntro(Level):
    title = "Sekce 9: Chybové výstupy"
    instructions_file = "section9_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"

    @override
    def setup(self, workspace: Path) -> None:
        return

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


class StderrToFileLevel(Level):
    title = "Přesměrování chyb"
    instructions = """
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
        `shellgame submit -f errors.log`
        """
    hints = [
        "Běžný výstup jde na stdout (1), chyby na stderr (2). Jak přesměrujete jen dvojku?",
        "Syntaxe je: příkaz 2> soubor. Zkuste to se skriptem buggy.sh.",
        "Použijte './buggy.sh 2> errors.log'.",
    ]
    start_directory = "level-9/errors"
    require_answer = True
    validators = [StringValidator("errors.log")]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-9"
        level_dir.mkdir(parents=True, exist_ok=True)

        script_path = level_dir / "buggy.sh"
        script_path.write_text("#!/bin/bash\necho 'This is normal output'\necho 'This is an error message' >&2\n")
        script_path.chmod(0o755)

        target = level_dir / "errors.log"
        if target.exists():
            target.unlink()

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        target = state.workspace / "level-9/errors.log"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "This is an error message" in content and "This is normal output" not in content:
            return True, "Správně! Soubor obsahuje pouze chyby."
        if "This is normal output" in content:
            return False, "Soubor obsahuje i normální výstup (použili jste &> nebo chybí 2?)."
        return False, "Soubor neobsahuje očekávanou chybu."


class AppendStderrToFileLevel(Level):
    title = "Přidávání chyb"
    instructions = """
        Stejně jako u normálního výstupu můžete chyby přidávat na konec souboru pomocí `2>>`.

        ### Úkol
        Spusťte `buggy.sh` znovu, ale tentokrát PŘIDEJTE chybové zprávy na konec `errors.log`.
        Nepřepisujte existující chyby!

        ### Příkazy
        - `./script 2>> soubor` - přidá stderr na konec souboru

        ### Odevzdání
        Odevzdejte název souboru.
        `shellgame submit -f errors.log`
        """
    hints = [
        "Jaký je rozdíl mezi > a >>? Jeden přepisuje, druhý přidává.",
        "Pro přidání chyb na konec použijte dvě šipky: 2>>",
        "Použijte './buggy.sh 2>> errors.log'.",
    ]
    start_directory = "level-9/errors"
    require_answer = True
    validators = [StringValidator("errors.log")]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-9"
        level_dir.mkdir(parents=True, exist_ok=True)

        script_path = level_dir / "buggy.sh"
        if not script_path.exists():
            script_path.write_text("#!/bin/bash\necho 'This is normal output'\necho 'This is an error message' >&2\n")
            script_path.chmod(0o755)

        (level_dir / "errors.log").write_text("Old error 1\n")

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        target = state.workspace / "level-9/errors.log"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "Old error 1" in content and "This is an error message" in content:
            return True, "Správně!"
        if "Old error 1" not in content:
            return False, "Původní obsah zmizel (použili jste 2> místo 2>>?)."
        return False, "Soubor neobsahuje novou chybu."


class AllOutputToFileLevel(Level):
    title = "Všechny výstupy"
    instructions = """
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
        `shellgame submit -f all_output.log`
        """
    hints = [
        "Ampersand (&) v tomto kontextu znamená 'obojí' - stdout i stderr.",
        "Kombinace &> je zkratka pro přesměrování obou výstupů.",
        "Použijte './buggy.sh &> all_output.log'.",
    ]
    start_directory = "level-9/errors"
    require_answer = True
    validators = [StringValidator("all_output.log")]

    @override
    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-9"
        level_dir.mkdir(parents=True, exist_ok=True)

        target = level_dir / "all_output.log"
        if target.exists():
            target.unlink()

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        target = state.workspace / "level-9/all_output.log"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "This is normal output" in content and "This is an error message" in content:
            return True, "Správně! Máme všechno."
        return False, "Soubor neobsahuje oba typy výstupů."


class DevNullLevel(Level):
    title = "Černá díra"
    instructions = """
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
        `shellgame submit -f /dev/null`
        """
    hints = [
        "Kam v Linuxu 'vyhodíte' data, která nechcete? Existuje speciální soubor...",
        "Soubor /dev/null je jako černá díra - vše pohltí a nic nevrátí.",
        "Použijte './buggy.sh &> /dev/null'.",
    ]
    start_directory = "level-9/errors"
    require_answer = True
    validators = [StringValidator("/dev/null")]

    @override
    def setup(self, workspace: Path) -> None:
        return

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


class StreamsChallengeLevel(Level):
    title = "Souhrn Sekce 9"
    instructions = """
        ### Výzva: Mistr streamů

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
        `shellgame submit <chyby>,<výstup>`
        """
    hints = [
        "Pro zachycení chyb: './mixed.sh 2> errors.log'. Pro normální výstup: './mixed.sh > output.log'.",
        "Počet řádků zjistíte pomocí 'wc -l errors.log output.log' nebo 'cat errors.log | wc -l'.",
        "errors.log má 2 řádky, output.log má 3 řádky. Odpověď je '2,3'.",
    ]
    require_answer = True

    @override
    def setup(self, workspace: Path) -> None:
        challenge_dir = workspace / "level-9" / "challenge"
        if challenge_dir.exists():
            shutil.rmtree(challenge_dir)

        challenge_dir.mkdir(parents=True, exist_ok=True)

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

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if not success:
            return False, msg

        assert answer is not None
        text = answer.strip()

        ok = False
        error_msg = "Formát odpovědi: chyby,výstup (např. 3,5)"

        if "," in text:
            parts = text.split(",")
            if len(parts) == 2:
                try:
                    errors = int(parts[0].strip())
                    output = int(parts[1].strip())
                except ValueError:
                    error_msg = "Obě hodnoty musí být čísla."
                else:
                    if errors != 2:
                        error_msg = (
                            f"Počet chyb není {errors}. Spusťte './mixed.sh 2> errors.log' a pak 'wc -l errors.log'."
                        )
                    elif output != 3:
                        error_msg = (
                            f"Počet normálních řádků není {output}. "
                            "Spusťte './mixed.sh > output.log' a pak 'wc -l output.log'."
                        )
                    else:
                        ok = True

        if ok:
            return True, "Perfektní! Dokončili jste Sekci 9. Stdout a stderr jsou pro vás jako otevřená kniha!"
        return False, error_msg


def get_levels() -> list[Level]:
    levels: list[Level] = [
        SectionIntro(),
        StderrToFileLevel(),
        AppendStderrToFileLevel(),
        AllOutputToFileLevel(),
        DevNullLevel(),
        StreamsChallengeLevel(),
    ]

    section_num = 9
    for i, level in enumerate(levels):
        level.section = section_num
        level.id = f"{section_num}.{i}"

    return levels
