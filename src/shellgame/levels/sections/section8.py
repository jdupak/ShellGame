"""Section 8: Redirection."""

import shutil
from pathlib import Path
from typing import Any, Optional

from shellgame.levels.base import Level
from shellgame.validation.validators import (
    StringValidator,
)


class Level8_0(Level):
    """Level 8.0: Section 8 Introduction."""

    def __init__(self) -> None:
        super().__init__(
            id="8.0",
            section=8,
            title="Sekce 8: Přesměrování výstupu",
            instructions_file="section8_intro.md",
            hints=["Přečtěte si úvod a pokračujte stisknutím Enter."],
        )

    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        pass

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Always valid."""
        return True, "Jdeme na to!"


class Level8_1(Level):
    """Level 8.1: Redirect to File."""

    def __init__(self) -> None:
        super().__init__(
            id="8.1",
            section=8,
            title="Uložení výstupu",
            instructions="""# Level 8.1: Uložení výstupu

Operátor `>` přesměruje výstup příkazu do souboru. Pokud soubor neexistuje, vytvoří se. Pokud existuje, **přepíše se**.

## Úkol:
Uložte seznam souborů v aktuálním adresáři (výstup `ls`) do souboru `seznam.txt`.

## Příkazy:
- `ls > seznam.txt`

## Odevzdání:
Odevzdejte název vytvořeného souboru.
`shellgame submit -f seznam.txt`""",
            hints=[
                "Použijte operátor '>' pro přesměrování výstupu.",
                "Příkaz 'echo' vypíše text.",
                "Zkuste 'echo \"Hello World\" > hello.txt'.",
            ],
            start_directory="level-8/redirect",
        )

    def setup(self, workspace: Path) -> None:
        """Create some files to list."""
        level_dir = workspace / "level-8" / "redirection"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "file1").touch()
        (level_dir / "file2").touch()

        # Clean up
        if (level_dir / "seznam.txt").exists():
            (level_dir / "seznam.txt").unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate file creation and content."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        str_val = StringValidator("seznam.txt")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        target = state.workspace / "level-8/redirection/seznam.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "file1" in content and "file2" in content:
            return True, "Správně!"
        else:
            return False, "Soubor neobsahuje očekávaný výstup příkazu ls."


class Level8_2(Level):
    """Level 8.2: Append to File."""

    def __init__(self) -> None:
        super().__init__(
            id="8.2",
            section=8,
            title="Přidání na konec",
            instructions="""# Level 8.2: Přidání na konec

Operátor `>>` (append) přidá výstup na konec souboru, aniž by smazal původní obsah.

## Úkol:
Máte soubor `log.txt` s nějakým obsahem. Přidejte na jeho konec text "Konec logu" pomocí příkazu `echo`.

## Příkazy:
- `echo "Text" >> soubor`

## Odevzdání:
Odevzdejte název souboru.
`shellgame submit -f log.txt`""",
            hints=["Použijte 'echo \"Konec logu\" >> log.txt'.", "Dvě šipky >> znamenají append."],
            start_directory="level-8/redirect",
        )

    def setup(self, workspace: Path) -> None:
        """Create initial log file."""
        level_dir = workspace / "level-8" / "redirection"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "log.txt").write_text("Start logu\nZaznam 1\n")

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate append."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        str_val = StringValidator("log.txt")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        target = state.workspace / "level-8/redirection/log.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "Start logu" in content and "Konec logu" in content:
            return True, "Správně!"
        elif "Konec logu" in content:
            return False, "Zdá se, že jste přepsali původní obsah (použili jste > místo >>?)."
        else:
            return False, "Soubor neobsahuje nový text."


class Level8_3(Level):
    """Level 8.3: Concatenate Files."""

    def __init__(self) -> None:
        super().__init__(
            id="8.3",
            section=8,
            title="Spojování souborů",
            instructions="""# Level 8.3: Spojování souborů

Příkaz `cat` (concatenate) umí vypsat obsah více souborů za sebou.
Když to zkombinujete s přesměrováním, můžete spojit více souborů do jednoho.

## Úkol:
Spojte obsah souborů `part1.txt` a `part2.txt` do nového souboru `full.txt`.

## Příkazy:
- `cat soubor1 soubor2 > novy_soubor`

## Odevzdání:
Odevzdejte název nového souboru.
`shellgame submit -f full.txt`""",
            hints=[
                "Použijte 'cat part1.txt part2.txt > full.txt'.",
                "Pořadí argumentů určuje pořadí v cílovém souboru.",
            ],
            start_directory="level-8/redirect",
        )

    def setup(self, workspace: Path) -> None:
        """Create parts."""
        level_dir = workspace / "level-8" / "concat"
        level_dir.mkdir(parents=True, exist_ok=True)

        (level_dir / "part1.txt").write_text("First part.\n")
        (level_dir / "part2.txt").write_text("Second part.\n")

        if (level_dir / "full.txt").exists():
            (level_dir / "full.txt").unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate concatenation."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        str_val = StringValidator("full.txt")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        target = state.workspace / "level-8/concat/full.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text()
        if "First part." in content and "Second part." in content:
            return True, "Správně!"
        else:
            return False, "Soubor neobsahuje text z obou částí."


class Level8_4(Level):
    """Level 8.4: Create with Echo."""

    def __init__(self) -> None:
        super().__init__(
            id="8.4",
            section=8,
            title="Vytvoření souboru s obsahem",
            instructions="""# Level 8.4: Vytvoření souboru s obsahem

Místo editoru můžete pro vytvoření krátkého souboru použít `echo` a přesměrování.

## Úkol:
Vytvořte soubor `pozdrav.txt`, který bude obsahovat text "Ahoj svete".

## Příkazy:
- `echo "Ahoj svete" > pozdrav.txt`

## Odevzdání:
Odevzdejte název souboru.
`shellgame submit -f pozdrav.txt`""",
            hints=[
                "Použijte 'echo \"Ahoj svete\" > pozdrav.txt'.",
                "Uvozovky jsou důležité, pokud text obsahuje mezery.",
            ],
            start_directory="level-8/redirect",
        )

    def setup(self, workspace: Path) -> None:
        """Clean up."""
        level_dir = workspace / "level-8" / "echo"
        level_dir.mkdir(parents=True, exist_ok=True)

        if (level_dir / "pozdrav.txt").exists():
            (level_dir / "pozdrav.txt").unlink()

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate content."""
        if answer is None:
            return False, "Musíte zadat název souboru."

        str_val = StringValidator("pozdrav.txt")
        success, msg = str_val.validate(answer, state.workspace)
        if not success:
            return False, msg

        target = state.workspace / "level-8/echo/pozdrav.txt"
        if not target.exists():
            return False, "Soubor neexistuje."

        content = target.read_text().strip()
        if content == "Ahoj svete":
            return True, "Správně!"
        else:
            return False, f"Očekáváno 'Ahoj svete', nalezeno '{content}'."


class Level8_5(Level):
    """Level 8.5: Piping Commands."""

    def __init__(self) -> None:
        super().__init__(
            id="8.5",
            section=8,
            title="Propojení příkazů (Pipes)",
            instructions="""# Level 8.5: Propojení příkazů pomocí rour (pipes)

Znak `|` (pipe/roura) pošle výstup jednoho příkazu jako vstup druhému.
Je to jako propojení trubek - data "tečou" z jednoho příkazu do druhého.

### Proč je to revolučně užitečné?
Místo ukládání mezivýsledků do souborů můžete příkazy řetězit:
```bash
# Bez pipe (3 kroky):
ls -l > temp.txt
grep ".py" temp.txt > python_files.txt
rm temp.txt

# S pipe (1 krok):
ls -l | grep ".py" > python_files.txt
```

### Běžné kombinace
- `cat soubor | wc -l` → spočítá řádky v souboru
- `ls | head -5` → zobrazí prvních 5 položek
- `cat log.txt | grep "ERROR"` → najde chybové zprávy

## Úkol
V aktuálním adresáři je soubor `access.log` s mnoha řádky.
Spočítejte, kolik řádků obsahuje slovo "ERROR".

Použijte: `grep "ERROR" access.log | wc -l`

## Odevzdání
Odevzdejte nalezený počet (číslo).
`shellgame submit -f <číslo>`""",
            hints=[
                "Pipe (|) propojuje výstup prvního příkazu se vstupem druhého.",
                "grep najde řádky s 'ERROR', wc -l je spočítá. Spojte je pomocí |.",
                'Použijte: grep "ERROR" access.log | wc -l',
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create log file with errors."""
        level_dir = workspace / "level-8" / "pipes"
        level_dir.mkdir(parents=True, exist_ok=True)

        # Create access log with 7 ERROR lines
        log_content = """2024-01-01 10:00:00 INFO Server started
2024-01-01 10:05:23 ERROR Connection refused
2024-01-01 10:10:45 INFO User logged in
2024-01-01 10:15:00 WARNING Low memory
2024-01-01 10:20:12 ERROR Database timeout
2024-01-01 10:25:00 INFO Request processed
2024-01-01 10:30:33 ERROR File not found
2024-01-01 10:35:00 INFO Cache cleared
2024-01-01 10:40:55 ERROR Permission denied
2024-01-01 10:45:00 DEBUG Verbose output
2024-01-01 10:50:18 ERROR Network unreachable
2024-01-01 10:55:00 INFO Backup completed
2024-01-01 11:00:00 ERROR Disk full
2024-01-01 11:05:00 INFO Server shutdown
2024-01-01 11:10:42 ERROR Service unavailable
"""
        (level_dir / "access.log").write_text(log_content)

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate error count."""
        if answer is None:
            return False, "Musíte zadat počet ERROR řádků."

        try:
            count = int(answer.strip())
        except ValueError:
            return False, "Odpověď musí být číslo."

        if count == 7:
            return True, "Správně! Pipe je mocný nástroj pro kombinování příkazů."
        elif count == 15:
            return (
                False,
                "Spočítali jste všechny řádky. Potřebujete jen ty s 'ERROR'. Použijte grep před wc.",
            )
        else:
            return False, f'Počet ERROR řádků není {count}. Zkuste: grep "ERROR" access.log | wc -l'


class Level8_6(Level):
    """Level 8.6: Head and Tail."""

    def __init__(self) -> None:
        super().__init__(
            id="8.6",
            section=8,
            title="Začátek a konec souboru",
            instructions="""# Level 8.6: Head a Tail - prohlížení částí souboru

Při práci s velkými soubory (logy, datasety) nechcete vidět vše najednou.

### Příkazy
- `head soubor` → prvních 10 řádků (výchozí)
- `head -n 5 soubor` → prvních 5 řádků
- `tail soubor` → posledních 10 řádků
- `tail -n 3 soubor` → poslední 3 řádky

### Praktické použití
- `tail -f /var/log/syslog` → sleduje nové záznamy v reálném čase
- `head -n 1 data.csv` → zobrazí hlavičku CSV souboru

## Úkol
V souboru `long_file.txt` je 50 řádků.
1. Zjistěte první slovo na **1. řádku** (pomocí `head -n 1`)
2. Zjistěte první slovo na **posledním řádku** (pomocí `tail -n 1`)

## Odevzdání
Odevzdejte obě slova oddělená čárkou: `první,poslední`
`shellgame submit -f START,END`""",
            hints=[
                "head -n 1 zobrazí první řádek, tail -n 1 zobrazí poslední.",
                "První řádek začíná slovem 'START', poslední slovem 'END'.",
                "Odpověď je: START,END",
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create long file."""
        level_dir = workspace / "level-8" / "headtail"
        level_dir.mkdir(parents=True, exist_ok=True)

        lines = ["START of the file - this is line 1"]
        for i in range(2, 50):
            lines.append(f"Line number {i} with some content")
        lines.append("END of the file - this is line 50")

        (level_dir / "long_file.txt").write_text("\n".join(lines) + "\n")

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate head/tail answer."""
        if answer is None:
            return False, "Zadejte odpověď ve formátu: první_slovo,poslední_slovo"

        answer = answer.strip().upper()

        if "," not in answer:
            return False, "Formát: první_slovo,poslední_slovo (např. AHOJ,SVET)"

        parts = answer.split(",")
        first = parts[0].strip()
        last = parts[1].strip()

        if first == "START" and last == "END":
            return True, "Správně! Head a tail jsou skvělé pro rychlý náhled do souborů."
        elif first != "START":
            return False, f"První slovo není '{first}'. Použijte 'head -n 1 long_file.txt'."
        else:
            return False, f"Poslední slovo není '{last}'. Použijte 'tail -n 1 long_file.txt'."


class Level8_7(Level):
    """Level 8.7: Word Count."""

    def __init__(self) -> None:
        super().__init__(
            id="8.7",
            section=8,
            title="Počítání (wc)",
            instructions="""# Level 8.7: Příkaz wc (word count)

Příkaz `wc` (word count) počítá řádky, slova a znaky v souboru.

### Přepínače
- `wc soubor` → řádky, slova, znaky (vše)
- `wc -l soubor` → pouze řádky (lines)
- `wc -w soubor` → pouze slova (words)
- `wc -c soubor` → pouze bajty/znaky (characters)

### Kombinace s pipe
- `ls | wc -l` → počet souborů v adresáři
- `cat soubor | wc -w` → počet slov

## Úkol
Zjistěte o souboru `article.txt`:
1. Kolik má **řádků**? (`wc -l`)
2. Kolik má **slov**? (`wc -w`)

## Odevzdání
Odevzdejte: `řádky,slova` (např. `10,50`)
`shellgame submit -f <řádky>,<slova>`""",
            hints=[
                "wc -l počítá řádky, wc -w počítá slova.",
                "Článek má 5 řádků a 25 slov.",
                "Odpověď je: 5,25",
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create article file."""
        level_dir = workspace / "level-8" / "wc"
        level_dir.mkdir(parents=True, exist_ok=True)

        # 5 lines, 25 words
        article = """Linux je svobodný operační systém.
Byl vytvořen Linusem Torvaldsem v roce 1991.
Dnes pohání většinu serverů na internetu.
Je základem systému Android a mnoha dalších.
Open source komunita ho neustále vylepšuje.
"""
        (level_dir / "article.txt").write_text(article)

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate wc answer."""
        if answer is None:
            return False, "Zadejte odpověď ve formátu: řádky,slova"

        answer = answer.strip()

        if "," not in answer:
            return False, "Formát: řádky,slova (např. 10,50)"

        parts = answer.split(",")
        try:
            lines = int(parts[0].strip())
            words = int(parts[1].strip())
        except ValueError:
            return False, "Obě hodnoty musí být čísla."

        if lines == 5 and words == 25:
            return True, "Správně! Příkaz wc je nepostradatelný pro rychlou analýzu souborů."
        elif lines != 5:
            return False, f"Počet řádků není {lines}. Použijte 'wc -l article.txt'."
        else:
            return False, f"Počet slov není {words}. Použijte 'wc -w article.txt'."


class Level8_8(Level):
    """Level 8.8: Sorting and Deduplication."""

    def __init__(self) -> None:
        super().__init__(
            id="8.8",
            section=8,
            title="Řazení a odstranění duplikátů",
            instructions="""# Level 8.8: Sort a Uniq - řazení a deduplikace

Příkazy `sort` a `uniq` jsou mocné nástroje pro zpracování textových dat.

### Příkazy
- `sort soubor` → Seřadí řádky abecedně
- `sort -n soubor` → Seřadí číselně
- `sort -r soubor` → Seřadí obráceně (sestupně)
- `uniq` → Odstraní **po sobě jdoucí** duplicitní řádky

### Klíčový vzor: sort | uniq
```bash
# uniq funguje jen na sousedních řádcích!
# Proto nejdřív seřadíme, pak odstraníme duplikáty:
sort names.txt | uniq
```

### Praktické použití
- Unikátní IP adresy z logu
- Seznam všech uživatelů bez opakování
- Seřazený seznam souborů podle velikosti

## Úkol
V souboru `visitors.txt` jsou jména návštěvníků (někteří přišli vícekrát).
Zjistěte, kolik je **UNIKÁTNÍCH** návštěvníků.

Použijte: `sort visitors.txt | uniq | wc -l`

## Odevzdání
Odevzdejte počet unikátních návštěvníků.
`shellgame submit -f <číslo>`""",
            hints=[
                "Příkaz uniq odstraní duplikáty, ale jen sousedící! Proto nejdřív sort.",
                "Řetězec: sort → uniq → wc -l spočítá unikátní řádky.",
                "V souboru je 5 unikátních jmen.",
            ],
            extension=True,
        )

    def setup(self, workspace: Path) -> None:
        """Create visitors file with duplicates."""
        level_dir = workspace / "level-8" / "sort"
        level_dir.mkdir(parents=True, exist_ok=True)

        # 8 entries, 5 unique
        visitors = """Alice
Bob
Charlie
Alice
David
Bob
Eve
Alice
"""
        (level_dir / "visitors.txt").write_text(visitors)

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:
        """Validate unique count."""
        if answer is None:
            return False, "Zadejte počet unikátních návštěvníků."

        try:
            count = int(answer.strip())
        except ValueError:
            return False, "Odpověď musí být číslo."

        messages = {
            5: (True, "Správně! Sort | uniq je klasická kombinace pro práci s daty."),
            8: (
                False,
                "Spočítali jste všechny řádky, ne unikátní. Použijte: sort visitors.txt | uniq | wc -l",
            ),
            3: (False, "Možná jste spočítali jen duplikáty. Hledáme počet unikátních jmen."),
        }

        if count in messages:
            return messages[count]

        return (
            False,
            f"Počet unikátních návštěvníků není {count}. Zkuste: sort visitors.txt | uniq | wc -l",
        )


class Level8_9(Level):
    """Level 8.9: Section 8 Summary."""

    def __init__(self) -> None:
        super().__init__(
            id="8.9",
            section=8,
            title="Souhrn Sekce 8",
            instructions="""### 🎯 Výzva: Mistr přesměrování a pipes

Ukažte, že ovládáte přesměrování i roury!

### Úkol
V `level-8/challenge`:

1. Vytvořte `message.txt` s textem "Hello World" pomocí echo
2. Přidejte na konec souboru další řádek "Goodbye" (append)
3. Spočítejte, kolik `.txt` souborů je v adresáři pomocí `ls *.txt | wc -l`

Odevzdejte: **<počet_txt_souborů>**

### Shrnutí příkazů Sekce 8
```
echo "text" > soubor    → Vytvoří/přepíše soubor
echo "text" >> soubor   → Připojí na konec
příkaz > soubor         → Výstup do souboru
příkaz | příkaz2        → Propojení rourou
head -n 5 soubor        → Prvních 5 řádků
tail -n 5 soubor        → Posledních 5 řádků
wc -l soubor            → Počet řádků
```

### Odevzdání
`shellgame submit <počet>`""",
            hints=[
                "První řádek: 'echo \"Hello World\" > message.txt'. Druhý: 'echo \"Goodbye\" >> message.txt' (dva >>).",
                "Pro počítání: 'ls *.txt | wc -l'. Nezapomeňte vytvořit message.txt!",
                "Po vytvoření message.txt budou v adresáři 3 .txt soubory (sample1.txt, sample2.txt, message.txt).",
            ],
        )

    def setup(self, workspace: Path) -> None:
        """Create challenge environment."""
        challenge_dir = workspace / "level-8" / "challenge"

        if challenge_dir.exists():
            shutil.rmtree(challenge_dir)

        challenge_dir.mkdir(parents=True, exist_ok=True)

        # Some files for ls to list
        (challenge_dir / "sample1.txt").write_text("sample")
        (challenge_dir / "sample2.txt").write_text("sample")

    def validate(self, answer: Optional[str], state: Any) -> tuple[bool, str]:  # noqa: PLR0911
        """Validate redirections and pipes."""
        if answer is None:
            return False, "Musíte zadat počet .txt souborů."

        try:
            count = int(answer.strip())
        except ValueError:
            return False, "Odpověď musí být číslo."

        challenge_dir = state.workspace / "level-8" / "challenge"

        # Check message.txt exists with correct content
        message_file = challenge_dir / "message.txt"
        if not message_file.exists():
            return False, "Chybí message.txt. Vytvořte pomocí 'echo \"Hello World\" > message.txt'."

        content = message_file.read_text()
        lines = content.strip().split("\n")

        if len(lines) < 2:
            return (
                False,
                "message.txt má jen jeden řádek. Přidejte druhý pomocí 'echo \"Goodbye\" >> message.txt' (dva >>).",
            )

        if "Hello World" not in lines[0]:
            return False, "První řádek message.txt nemá 'Hello World'."

        if "Goodbye" not in lines[1]:
            return False, "Druhý řádek message.txt nemá 'Goodbye'."

        # Check count
        if count == 3:
            return True, "🎉 Skvělé! Dokončili jste Sekci 8. Přesměrování i pipes máte v malíku!"
        elif count == 2:
            return (
                False,
                "Spočítali jste jen sample1.txt a sample2.txt. Vytvořili jste message.txt?",
            )
        else:
            return (
                False,
                f"Počet .txt souborů není {count}. Po vytvoření message.txt jich tam budou 3.",
            )


def get_levels() -> list:
    """
    Return all Section 8 level classes.

    Returns:
        List of Level instances for Section 8
    """
    return [
        Level8_0(),
        Level8_1(),
        Level8_2(),
        Level8_3(),
        Level8_4(),
        Level8_5(),
        Level8_6(),
        Level8_7(),
        Level8_8(),
        Level8_9(),
    ]
