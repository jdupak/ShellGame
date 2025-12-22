from __future__ import annotations

import shutil
import stat
from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    ExecutableValidator,
    PermissionValidator,
    StringValidator,
    ValidationResult,
)


class SectionIntroLevel(Level):
    title = "Sekce 7: Oprávnění"
    instructions_file = "section7_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"

    @override
    def setup(self, workspace: Path) -> None:
        pass

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


class FindExecutableLevel(Level):
    title = "Hledání spustitelného souboru"
    instructions = """
        Příkaz `ls -l` zobrazuje oprávnění v prvním sloupci (např. `-rw-r--r--`).
        Pokud je soubor spustitelný, má nastaven příznak `x` (např. `-rwxr-xr-x`).

        ## Úkol:
        V adresáři je několik skriptů, ale jen jeden je nastaven jako spustitelný. Najděte ho.

        ## Příkazy:
        - `ls -l`: Zobrazí oprávnění

        ## Odevzdání:
        Odevzdejte název spustitelného souboru.
        `shellgame submit -f script.sh`
        """
    hints = ["Použijte 'ls -l'.", "Hledejte 'x' v oprávněních (např. -rwxr-xr-x)."]
    start_directory = "level-7/executables"
    require_answer = True
    validators = [StringValidator("script.sh")]

    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-7" / "executables"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Non-executable scripts
        (level_dir / "test.sh").write_text("#!/bin/bash")
        (level_dir / "data.sh").write_text("#!/bin/bash")

        # Executable script
        target = level_dir / "script.sh"
        target.write_text("#!/bin/bash\necho Hi")
        target.chmod(target.stat().st_mode | stat.S_IEXEC)


class MakeExecutableLevel(Level):
    title = "Nastavení spustitelnosti"
    instructions = """
        Aby šel skript spustit (např. `./script.sh`), musí mít nastavené právo `x`.
        Příkaz `chmod` (change mode) mění oprávnění.

        ### Proč je to důležité?
        Když napíšete skript nebo stáhnete program, často není automaticky spustitelný.
        Musíte mu explicitně dát právo ke spuštění - je to bezpečnostní opatření.

        ### Běžný pracovní postup
        1. Napíšete skript: `nano muj_skript.sh`
        2. Pokusíte se spustit: `./muj_skript.sh` → "Permission denied"
        3. Přidáte právo: `chmod u+x muj_skript.sh`
        4. Nyní funguje: `./muj_skript.sh` ✓

        ## Úkol:
        Soubor `run_me.sh` nejde spustit. Přidejte mu právo pro spuštění pro vlastníka (`u`).

        ## Příkazy:
        - `chmod u+x <soubor>`: Přidá právo execute pro usera

        ## Odevzdání:
        Odevzdejte název souboru.
        `shellgame submit -f run_me.sh`
        """
    hints = [
        "Právo 'x' (execute) je potřeba pro spuštění. Jak ho přidáte pro vlastníka (user)?",
        "Syntaxe chmod: chmod kdo+co soubor. 'u' = user, 'x' = execute.",
        "Použijte 'chmod u+x run_me.sh'.",
    ]
    start_directory = "level-7/permissions"
    require_answer = True
    validators = [StringValidator("run_me.sh"), ExecutableValidator("level-7/permissions/run_me.sh")]

    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-7" / "permissions"
        level_dir.mkdir(parents=True, exist_ok=True)

        target = level_dir / "run_me.sh"
        target.write_text("#!/bin/bash\necho Run me")
        target.chmod(0o644)  # ensure not executable


class MakeReadOnlyLevel(Level):
    title = "Ochrana souboru"
    instructions = """
        Někdy chcete zabránit nechtěnému přepsání souboru. Můžete mu odebrat právo pro zápis (`w`).

        ## Úkol:
        Soubor `config.readonly` by neměl být měněn. Odeberte právo zápisu pro všechny (user, group, other).

        ## Příkazy:
        - `chmod a-w <soubor>`: Odeberte write pro all (všechny)
        - Nebo `chmod -w <soubor>` (zkratka pro a-w)

        ## Odevzdání:
        Odevzdejte název souboru.
        `shellgame submit -f config.readonly`
        """
    hints = ["Použijte 'chmod -w config.readonly'.", "Tím odeberete právo zápisu."]
    start_directory = "level-7/permissions"
    require_answer = True
    validators = [StringValidator("config.readonly")]

    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-7" / "permissions"
        level_dir.mkdir(parents=True, exist_ok=True)

        target = level_dir / "config.readonly"
        target.write_text("Do not touch")
        target.chmod(0o644)  # rw-r--r--

    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        ok, msg = super().validate(answer, state)
        if not ok:
            return ok, msg

        target = state.workspace / "level-7" / "permissions" / "config.readonly"
        if not target.exists():
            return False, "Soubor neexistuje."

        mode = target.stat().st_mode
        if mode & stat.S_IWUSR:
            return False, "Soubor má stále právo zápisu pro vlastníka."

        if mode & (stat.S_IWGRP | stat.S_IWOTH):
            return False, "Soubor má stále právo zápisu pro skupinu nebo ostatní."

        return True, "Správně!"


class NumericPermissionsLevel(Level):
    title = "Číselný zápis"
    instructions = """
        Oprávnění lze nastavit i číselně (oktalově).
        - 4 = read (r)
        - 2 = write (w)
        - 1 = execute (x)

        Součet dává kombinaci (např. 7 = 4+2+1 = rwx, 5 = 4+1 = r-x).
        Zadávají se tři čísla: pro vlastníka, skupinu a ostatní.
        Např. `755` znamená `rwx` pro vlastníka, `r-x` pro skupinu, `r-x` pro ostatní.

        ## Úkol:
        Nastavte souboru `public_html` oprávnění `755` (rwxr-xr-x).

        ## Příkazy:
        - `chmod 755 <soubor>`

        ## Odevzdání:
        Odevzdejte název souboru.
        `shellgame submit -f public_html`
        """
    hints = ["Použijte 'chmod 755 public_html'.", "7 = rwx, 5 = r-x."]
    start_directory = "level-7/permissions"
    require_answer = True
    validators = [StringValidator("public_html"), PermissionValidator("level-7/permissions/public_html", "755")]

    def setup(self, workspace: Path) -> None:
        level_dir = workspace / "level-7" / "permissions"
        level_dir.mkdir(parents=True, exist_ok=True)

        target = level_dir / "public_html"
        target.write_text("<html></html>")
        target.chmod(0o600)  # rw-------


class PermissionsChallengeLevel(Level):
    title = "Souhrn Sekce 7"
    instructions = """
        ### Výzva: Strážce oprávnění

        Ukažte, že rozumíte oprávněním!

        ### Úkol
        V `level-7/challenge` jsou 3 soubory:

        1. `script.sh` - potřebuje být **spustitelný** vlastníkem
        2. `secret.txt` - má být **pouze pro čtení** (žádný zápis pro nikoho)
        3. `shared.txt` - nastavte oprávnění **644** (rw-r--r--)

        Pak odevzdejte heslo: **guardian**

        ### Shrnutí příkazů Sekce 7
        ```
        ls -l            → Zobrazí oprávnění
        chmod +x soubor  → Přidá právo spuštění
        chmod -w soubor  → Odebere právo zápisu
        chmod 755 soubor → Nastaví rwxr-xr-x
        chmod 644 soubor → Nastaví rw-r--r--
        ```

        ### Oktalové oprávnění
        ```
        4 = read    2 = write    1 = execute
        7 = rwx     6 = rw-      5 = r-x     4 = r--
        ```

        ### Odevzdání
        `shellgame submit guardian`
        """
    hints = [
        "Pro spustitelnost: 'chmod +x script.sh'. Pro pouze čtení: 'chmod 444 secret.txt' nebo 'chmod a-w secret.txt'.",
        "Pro 644: 'chmod 644 shared.txt'. Zkontrolujte pomocí 'ls -l'.",
        "script.sh musí mít 'x' pro vlastníka, secret.txt nesmí mít žádné 'w', shared.txt musí být rw-r--r--.",
    ]
    require_answer = True
    expected_answer = "guardian"

    def setup(self, workspace: Path) -> None:
        challenge_dir = workspace / "level-7" / "challenge"

        if challenge_dir.exists():
            shutil.rmtree(challenge_dir)

        challenge_dir.mkdir(parents=True, exist_ok=True)

        script = challenge_dir / "script.sh"
        script.write_text("#!/bin/bash\necho 'Hello'\n")
        script.chmod(0o600)  # rw------- (no execute)

        secret = challenge_dir / "secret.txt"
        secret.write_text("Top secret!\n")
        secret.chmod(0o666)  # rw-rw-rw- (writable)

        shared = challenge_dir / "shared.txt"
        shared.write_text("Shared content\n")
        shared.chmod(0o777)  # rwxrwxrwx (wrong)

    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        ok, msg = super().validate(answer, state)
        if not ok:
            return ok, msg

        challenge_dir = state.workspace / "level-7" / "challenge"

        error: str | None = None

        script = challenge_dir / "script.sh"
        if not script.exists():
            error = "Chybí soubor script.sh."
        elif not (script.stat().st_mode & stat.S_IXUSR):
            error = "script.sh není spustitelný. Použijte 'chmod +x script.sh'."

        if error is None:
            secret = challenge_dir / "secret.txt"
            if not secret.exists():
                error = "Chybí soubor secret.txt."
            else:
                mode = secret.stat().st_mode
                if mode & (stat.S_IWUSR | stat.S_IWGRP | stat.S_IWOTH):
                    error = (
                        "secret.txt má stále právo zápisu. Použijte 'chmod a-w secret.txt' nebo 'chmod 444 secret.txt'."
                    )

        if error is None:
            shared = challenge_dir / "shared.txt"
            if not shared.exists():
                error = "Chybí soubor shared.txt."
            else:
                actual = shared.stat().st_mode & 0o777
                if actual != 0o644:
                    error = f"shared.txt nemá oprávnění 644. Aktuální: {oct(actual)}. Použijte 'chmod 644 shared.txt'."

        if error is not None:
            return False, error

        return True, "Výborně! Dokončili jste Sekci 7. Oprávnění vám jsou jasná jako den!"


def get_levels() -> list[Level]:
    return [
        SectionIntroLevel(),
        FindExecutableLevel(),
        MakeExecutableLevel(),
        MakeReadOnlyLevel(),
        NumericPermissionsLevel(),
        PermissionsChallengeLevel(),
    ]
