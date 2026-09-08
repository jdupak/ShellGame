from __future__ import annotations

import stat

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    Completion,
    ExactAnswer,
    PermissionBits,
    PermissionMode,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import RunShell, Solution

section = Section(7, root="level-7")


@section.level(0)
class SectionIntroLevel(Level):
    is_intro = True
    title = "Sekce 7: Oprávnění"
    instructions_file = "section7_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    success_message = "Jdeme na to!"


@section.level(1)
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
        `shellgame submit <soubor>`
        """
    hints = [
        "Podrobný výpis souborů včetně sloupců s oprávněními získáte přepínačem '-l'.",
        "Spusťte 'ls -l' a hledejte písmeno 'x' (execute) v prvním sloupci oprávnění.",
        "Hledejte skript s právy např. '-rwxr-xr-x' a jeho název zadejte do 'shellgame submit <soubor>'.",
    ]
    start_directory = "executables"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("executables/test.sh", "#!/bin/bash", mode=0o644),
            FileFixture("executables/data.sh", "#!/bin/bash", mode=0o644),
            FileFixture(
                "executables/script.sh",
                "#!/bin/bash\necho Hi",
                mode=0o755,
            ),
        )
    )
    completion = Completion(answer=ExactAnswer("script.sh"))


@section.level(2)
class MakeExecutableLevel(Level):
    solution = Solution(steps=(RunShell("chmod u+x run_me.sh"),), answer="run_me.sh")
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
        `shellgame submit run_me.sh`
        """
    hints = [
        "Právo 'x' (execute) je potřeba pro spuštění. Jak ho přidáte pro vlastníka (user)?",
        "Syntaxe chmod: chmod kdo+co soubor. 'u' = user, 'x' = execute.",
        "Použijte 'chmod u+x run_me.sh'.",
    ]
    start_directory = "permissions"
    fixture = WorkspaceFixture(files=(FileFixture("permissions/run_me.sh", "#!/bin/bash\necho Run me", mode=0o644),))
    completion = Completion(
        answer=ExactAnswer("run_me.sh"),
        requirements=(
            PermissionBits(
                "permissions/run_me.sh",
                required=stat.S_IXUSR,
                error_message="Soubor stále není spustitelný pro vlastníka.",
            ),
        ),
    )


@section.level(3)
class MakeReadOnlyLevel(Level):
    solution = Solution(steps=(RunShell("chmod a-w config.readonly"),), answer="config.readonly")
    title = "Ochrana souboru"
    instructions = """
        Někdy chcete zabránit nechtěnému přepsání souboru. Můžete mu odebrat právo pro zápis (`w`).

        ## Úkol:
        Soubor `config.readonly` by neměl být měněn. Odeberte právo zápisu pro všechny (user, group, other).

        ## Příkazy:
        - `chmod a-w <soubor>`: Odeberte write pro all (všechny)

        Písmeno `a` nevynechávejte: bez něj výsledek ovlivňuje výchozí maska práv (`umask`).

        ## Odevzdání:
        Odevzdejte název souboru.
        `shellgame submit config.readonly`
        """
    hints = [
        "'a' znamená all (všechny: vlastníka, skupinu i ostatní), '-w' odebírá právo zápisu.",
        "Spusťte 'chmod a-w config.readonly'.",
    ]
    start_directory = "permissions"
    fixture = WorkspaceFixture(files=(FileFixture("permissions/config.readonly", "Do not touch", mode=0o644),))
    completion = Completion(
        answer=ExactAnswer("config.readonly"),
        requirements=(
            PermissionBits(
                "permissions/config.readonly",
                forbidden=stat.S_IWUSR | stat.S_IWGRP | stat.S_IWOTH,
                error_message="Soubor má stále právo zápisu.",
            ),
        ),
    )


@section.level(4)
class NumericPermissionsLevel(Level):
    solution = Solution(steps=(RunShell("chmod 755 public_html"),), answer="public_html")
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
        `shellgame submit public_html`
        """
    hints = [
        "Oktalový zápis: 7 = 4+2+1 (rwx) pro vlastníka, 5 = 4+1 (r-x) pro skupinu a ostatní.",
        "Spusťte 'chmod 755 public_html'.",
    ]
    start_directory = "permissions"
    fixture = WorkspaceFixture(files=(FileFixture("permissions/public_html", "<html></html>", mode=0o600),))
    completion = Completion(
        answer=ExactAnswer("public_html"),
        requirements=(PermissionMode("permissions/public_html", 0o755),),
    )


@section.level(5)
class PermissionsChallengeLevel(Level):
    solution = Solution(
        steps=(RunShell("chmod u+x script.sh && chmod a-w secret.txt && chmod 644 shared.txt"),),
        answer="guardian",
    )
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
        chmod u+x soubor → Přidá vlastníkovi právo spuštění
        chmod a-w soubor → Odebere všem právo zápisu
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
        "Pro vlastníka: 'chmod u+x script.sh'. Pro pouze čtení: 'chmod 444 secret.txt' nebo 'chmod a-w secret.txt'.",
        "Pro 644: 'chmod 644 shared.txt'. Zkontrolujte pomocí 'ls -l'.",
        "script.sh musí mít 'x' pro vlastníka, secret.txt nesmí mít žádné 'w', shared.txt musí být rw-r--r--.",
    ]
    start_directory = "challenge"
    fixture = WorkspaceFixture(
        files=(
            FileFixture(
                "challenge/script.sh",
                "#!/bin/bash\necho 'Hello'\n",
                mode=0o600,
            ),
            FileFixture("challenge/secret.txt", "Top secret!\n", mode=0o666),
            FileFixture("challenge/shared.txt", "Shared content\n", mode=0o777),
        ),
        clean=("challenge",),
    )
    completion = Completion(
        answer=ExactAnswer("guardian"),
        requirements=(
            PermissionBits(
                "challenge/script.sh",
                required=stat.S_IXUSR,
                error_message="script.sh není spustitelný. Použijte 'chmod u+x script.sh'.",
            ),
            PermissionBits(
                "challenge/secret.txt",
                forbidden=stat.S_IWUSR | stat.S_IWGRP | stat.S_IWOTH,
                error_message=(
                    "secret.txt má stále právo zápisu. Použijte 'chmod a-w secret.txt' nebo 'chmod 444 secret.txt'."
                ),
            ),
            PermissionMode(
                "challenge/shared.txt",
                0o644,
                error_message="shared.txt nemá oprávnění 644. Použijte 'chmod 644 shared.txt'.",
            ),
        ),
    )
    success_message = "Výborně! Dokončili jste Sekci 7. Oprávnění vám jsou jasná jako den!"
