from __future__ import annotations

import stat

from shellgame.levels.base import Level
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    AtDirectory,
    ChoiceAnswer,
    Completion,
    DirectoryPermissionMode,
    ExactAnswer,
    PathsMatch,
    PermissionBits,
    PermissionMode,
)
from shellgame.levels.fixture import DirectoryFixture, FileFixture, WorkspaceFixture
from shellgame.levels.solution import Chdir, Chmod, RunShell, Solution

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
        První trojice práv po znaku typu patří vlastníkovi souboru.

        ## Úkol:
        V adresáři je několik skriptů. Najděte jediný, který má právo spuštění (`x`)
        nastavené pro vlastníka. `x` u skupiny nebo ostatních nestačí.

        ## Příkazy:
        - `ls -l`: Zobrazí oprávnění

        ## Odevzdání:
        Odevzdejte název spustitelného souboru.
        `shellgame submit <soubor>`
        """
    hints = [
        "Podrobný výpis souborů včetně sloupců s oprávněními získáte přepínačem '-l'.",
        "Spusťte 'ls -l' a rozdělte práva po znaku typu na trojice pro vlastníka, skupinu a ostatní.",
        "Hledejte 'x' v první trojici (pozice vlastníka), ne v trojici skupiny nebo ostatních.",
    ]
    start_directory = "executables"
    fixture = WorkspaceFixture(
        files=(
            FileFixture("executables/test.sh", "#!/bin/bash", mode=0o654),
            FileFixture("executables/data.sh", "#!/bin/bash", mode=0o645),
            FileFixture(
                "executables/script.sh",
                "#!/bin/bash\necho Hi",
                mode=0o744,
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
    solution = Solution(steps=(RunShell("chmod a-w config.readonly"),), answer=None)
    title = "Odebrání práva zápisu"
    instructions = """
        Někdy chcete zabránit nechtěnému přepsání souboru. Můžete mu odebrat právo pro zápis (`w`).

        ## Úkol:
        Odeberte souboru `config.readonly` právo zápisu pro všechny: vlastníka, skupinu i ostatní.

        ## Příkazy:
        - `chmod a-w <soubor>`: Odeberte write pro all (všechny)

        Písmeno `a` nevynechávejte: bez něj výsledek ovlivňuje výchozí maska práv (`umask`).

        ## Odevzdání:
        Po odebrání všech práv zápisu spusťte:
        `shellgame submit`
        """
    hints = [
        "'a' znamená all (všechny: vlastníka, skupinu i ostatní), '-w' odebírá právo zápisu.",
        "Spusťte 'chmod a-w config.readonly'.",
    ]
    start_directory = "permissions"
    fixture = WorkspaceFixture(files=(FileFixture("permissions/config.readonly", "Do not touch", mode=0o644),))
    completion = Completion(
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
    solution = Solution(steps=(RunShell("chmod 755 deploy.sh"),), answer=None)
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
        Nastavte spouštěcímu skriptu `deploy.sh` oprávnění `755` (`rwxr-xr-x`).

        ## Příkazy:
        - `chmod 755 <soubor>`

        ## Odevzdání:
        Po nastavení přesného režimu spusťte:
        `shellgame submit`
        """
    hints = [
        "Oktalový zápis: 7 = 4+2+1 (rwx) pro vlastníka, 5 = 4+1 (r-x) pro skupinu a ostatní.",
        "Spusťte 'chmod 755 deploy.sh' a výsledek zkontrolujte pomocí 'ls -l deploy.sh'.",
    ]
    start_directory = "permissions"
    fixture = WorkspaceFixture(
        files=(FileFixture("permissions/deploy.sh", "#!/bin/sh\necho 'Nasazuji aplikaci'\n", mode=0o600),)
    )
    completion = Completion(requirements=(PermissionMode("permissions/deploy.sh", 0o755),))


@section.level(5)
class PermissionsChallengeLevel(Level):
    solution = Solution(
        steps=(RunShell("chmod u+x script.sh && chmod a-w secret.txt && chmod 644 shared.txt && chmod g+r team.txt"),),
        answer=None,
    )
    title = "Výzva: Oprávnění souborů"
    instructions = """
        ### Výzva: Oprávnění souborů

        Ukažte, že rozumíte oprávněním!

        ### Úkol
        V `level-7/challenge` jsou 4 soubory:

        1. `script.sh` - potřebuje být **spustitelný** vlastníkem
        2. `secret.txt` - odeberte **právo zápisu všem**
        3. `shared.txt` - nastavte oprávnění **644** (rw-r--r--)
        4. `team.txt` - přidejte skupině právo čtení, ostatní práva neměňte

        ### Shrnutí příkazů Sekce 7
        ```
        ls -l            → Zobrazí oprávnění
        chmod u+x soubor → Přidá vlastníkovi právo spuštění
        chmod a-w soubor → Odebere všem právo zápisu
        chmod g+r soubor → Přidá skupině právo čtení
        chmod 755 soubor → Nastaví rwxr-xr-x
        chmod 644 soubor → Nastaví rw-r--r--
        ```

        ### Oktalové oprávnění
        ```
        4 = read    2 = write    1 = execute
        7 = rwx     6 = rw-      5 = r-x     4 = r--
        ```

        ### Odevzdání
        Po splnění všech bodů spusťte `shellgame submit`.
        """
    hints = [
        "U každého souboru určete skupinu uživatelů (u, g, o nebo a), operaci (+, - nebo =) a požadované právo.",
        "První, druhý a čtvrtý úkol lze vyřešit symbolicky; u shared.txt nastavte přesný číselný režim.",
        "Použijte 'chmod u+x script.sh', 'chmod a-w secret.txt', 'chmod 644 shared.txt' a 'chmod g+r team.txt'.",
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
            FileFixture("challenge/team.txt", "Team notes\n", mode=0o600),
        ),
        clean=("challenge",),
    )
    completion = Completion(
        requirements=(
            PermissionBits(
                "challenge/script.sh",
                required=stat.S_IXUSR,
                error_message="Vlastník stále nemůže spustit script.sh.",
            ),
            PermissionBits(
                "challenge/secret.txt",
                forbidden=stat.S_IWUSR | stat.S_IWGRP | stat.S_IWOTH,
                error_message="secret.txt má stále alespoň jedno právo zápisu.",
            ),
            PermissionMode(
                "challenge/shared.txt",
                0o644,
                error_message="shared.txt nemá přesně oprávnění rw-r--r--.",
            ),
            PermissionMode(
                "challenge/team.txt",
                0o640,
                error_message="team.txt nemá právo čtení pro skupinu při zachování ostatních práv.",
            ),
        ),
    )
    success_message = "Výborně! Oprávnění běžných souborů máte pod kontrolou."


@section.level(6)
class DirectoryWritePermissionLevel(Level):
    solution = Solution(
        steps=(
            RunShell("cp locked/report.txt recovered.txt"),
            Chmod("directory-write/locked", 0o750),
            RunShell("cp source.txt locked/"),
        ),
        answer="adresář",
    )
    title = "Kdo dovoluje zápis do adresáře?"
    instructions = """
        ### Cíl
        Prakticky ověřte, že vytvoření nové položky řídí oprávnění **adresáře**,
        ne oprávnění kopírovaného souboru.

        Adresář `locked` má režim `550`: lze ho číst a procházet, ale nelze do něj zapisovat.

        ### Předpověď a pozorování
        1. Nejdřív předpovězte, zda projde `cp source.txt locked/`.
        2. Příkaz spusťte a přečtěte chybovou hlášku.
        3. Ověřte, že čtení ven funguje: `cp locked/report.txt recovered.txt`.
        4. Přidejte vlastníkovi adresáře právo zápisu: `chmod u+w locked`.
        5. Zopakujte `cp source.txt locked/`; tentokrát musí uspět.

        ### Vysvětlení
        Co rozhodovalo o možnosti vytvořit `locked/source.txt` — oprávnění
        zdrojového **souboru**, nebo cílového **adresáře**?

        ### Odevzdání
        `shellgame submit adresář`
        """
    hints = [
        "Právo 'w' na soubor řídí změnu jeho obsahu. Vytvoření nového jména je operace nad adresářem.",
        "Režim 550 nedává vlastníkovi adresáře právo 'w'; příkaz 'chmod u+w locked' ho přidá.",
        "Zkopírujte report ven, přidejte 'w' adresáři locked, zopakujte kopii dovnitř a odevzdejte 'adresář'.",
    ]
    start_directory = "directory-write"
    fixture = WorkspaceFixture(
        clean=("directory-write",),
        directory_fixtures=(DirectoryFixture("directory-write/locked", mode=0o550),),
        files=(
            FileFixture("directory-write/source.txt", "nová data\n"),
            FileFixture("directory-write/locked/report.txt", "existující zpráva\n"),
        ),
    )
    completion = Completion(
        answer=ChoiceAnswer(
            ("adresář", "adresar", "directory"),
            case_sensitive=False,
            error_message="Zaměřte se na objekt, ve kterém vzniká nové jméno souboru.",
            required_message="Odevzdejte odpověď: shellgame submit <soubor|adresář>",
        ),
        requirements=(
            PathsMatch(
                "directory-write/locked/report.txt",
                "directory-write/recovered.txt",
                destination_error="Nejdřív zkopírujte existující report z locked ven.",
            ),
            PathsMatch(
                "directory-write/source.txt",
                "directory-write/locked/source.txt",
                destination_error="Po přidání práva zápisu zkopírujte source.txt do locked/.",
            ),
            DirectoryPermissionMode(
                "directory-write/locked",
                0o750,
                error_message="Adresář locked nemá očekávaná práva po příkazu 'chmod u+w locked'.",
            ),
        ),
    )
    success_message = "Správně! Vytváření a mazání jmen řídí zapisovatelnost nadřazeného adresáře."


@section.level(7)
class DirectoryTraversePermissionLevel(Level):
    solution = Solution(
        steps=(
            Chmod("directory-traverse/parent", 0o644),
            Chmod("directory-traverse/parent", 0o744),
            Chdir("directory-traverse/parent"),
        ),
        answer="x",
    )
    title = "Právo x u adresáře"
    instructions = """
        ### Cíl
        Zažijte rozdíl mezi čtením názvů a **průchodem** adresářem.

        Začínáte v `parent/child`. Nadřazený adresář má běžná práva `755`.

        ### Předpověď a pozorování
        1. Odeberte všem právo průchodu nadřazeným adresářem: `chmod a-x ..`.
        2. Předpovězte výsledek a zkuste `cd ..`.
        3. Vraťte vlastníkovi průchod: `chmod u+x ..`.
        4. Zopakujte `cd ..`; nyní musí uspět.

        ### Vysvětlení
        Které písmeno oprávnění umožňuje vstoupit do adresáře a procházet přes něj?

        ### Odevzdání
        Z adresáře `parent` spusťte `shellgame submit x`.
        """
    hints = [
        "U adresáře neznamená 'x' spuštění programu, ale možnost průchodu cestou.",
        "Po 'chmod a-x ..' nadřazený adresář stále může mít 'r', ale cesta přes něj nefunguje.",
        "Obnovte vlastníkovi průchod příkazem 'chmod u+x ..', přejděte do parent a odevzdejte 'x'.",
    ]
    start_directory = "directory-traverse/parent/child"
    fixture = WorkspaceFixture(
        clean=("directory-traverse",),
        directory_fixtures=(
            DirectoryFixture("directory-traverse/parent", mode=0o755),
            DirectoryFixture("directory-traverse/parent/child", mode=0o755),
        ),
    )
    completion = Completion(
        answer=ChoiceAnswer(
            ("x", "execute", "průchod", "pruchod"),
            case_sensitive=False,
            error_message="Hledejte právo, které u adresáře znamená průchod cestou.",
            required_message="Odevzdejte označení práva: shellgame submit <písmeno>",
        ),
        requirements=(
            AtDirectory("directory-traverse/parent"),
            DirectoryPermissionMode(
                "directory-traverse/parent",
                0o744,
                error_message="Obnovte právo průchodu pouze vlastníkovi nadřazeného adresáře.",
            ),
        ),
    )
    success_message = "Výborně! Dokončili jste Sekci 7 a rozumíte právům souborů i adresářů."
