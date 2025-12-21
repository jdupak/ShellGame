"""Section 1: Navigation and path manipulation levels."""

from __future__ import annotations

import sys
from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level
from shellgame.markers import MarkerManager
from shellgame.messages import Messages
from shellgame.protocols import GameStateProtocol
from shellgame.validation.validators import (
    CommonMistakeValidator,
    HomeDirectoryValidator,
    OrderedListValidator,
    ValidationResult,
)


def _setup_level1_common(workspace: Path) -> None:
    """Common setup for Level 1 section."""
    level_dir = workspace / "level-1"
    level_dir.mkdir(exist_ok=True)

    # Create basic structure for future levels
    (level_dir / "alpha").mkdir(exist_ok=True)
    if not (level_dir / "alpha" / "inside.txt").exists():
        (level_dir / "alpha" / "inside.txt").write_text("first step")

    (level_dir / "delta").mkdir(exist_ok=True)
    if not (level_dir / "delta" / "single.dat").exists():
        (level_dir / "delta" / "single.dat").write_text("momentum gained")

    (level_dir / "gamma").mkdir(exist_ok=True)

    # Patterns directory for level 1.2
    patterns_dir = level_dir / "patterns"
    patterns_dir.mkdir(exist_ok=True)
    for filename in ["data.txt", "dog.md", "drama.log", "zebra.txt"]:
        if not (patterns_dir / filename).exists():
            (patterns_dir / filename).write_text("")
    (patterns_dir / "omega").mkdir(exist_ok=True)


class Level1_0(Level):
    """Level 1.0: Section 1 Introduction."""

    def __init__(self) -> None:
        super().__init__(
            id="1.0",
            section=1,
            title="Sekce 1: Navigace",
            instructions_file="section1_intro.md",
            hints=["Přečtěte si úvod a pokračujte příkazem 'shellgame submit'."],
            start_directory="",  # workspace root
            success_message="Jdeme na to!",
        )

    @override
    def setup(self, workspace: Path) -> None:
        """No setup needed."""
        return

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


class Level1_1(Level):
    """Level 1.1: Current Location - Learn to use pwd."""

    def __init__(self) -> None:
        super().__init__(
            id="1.1",
            section=1,
            title="Aktuální umístění",
            instructions="""
### Cíl
Zjistěte název aktuálního adresáře.

### Příkazy
- `pwd` - vypíše celou cestu k aktuálnímu adresáři

### Úkol
1. Spusťte `pwd`
2. Odevzdejte název posledního adresáře v cestě (basename)

### Příklad
Cesta: `/home/student/dokumenty` -> Odpověď: `dokumenty`
            """.strip(),
            hints=[
                "Příkaz 'pwd' (Print Working Directory) vám ukáže, kde jste. Zkuste ho!",
                "Výstup pwd bude něco jako /tmp/shellgame-.../level-1. Co je za posledním lomítkem?",
                "Basename je poslední část cesty. Pokud pwd vypíše '/tmp/.../level-1', odpověď je 'level-1'.",
            ],
            start_directory="level-1",
            marker_name=MarkerManager.PWD_USED,
            marker_error=Messages.L1_1_USE_PWD_FIRST,
            require_answer=True,
            expected_answer="level-1",
        )

    def setup(self, workspace: Path) -> None:
        """Create level-1 directory."""
        _setup_level1_common(workspace)


class Level1_2(Level):
    """Level 1.2: Listing & Pattern Recognition."""

    def __init__(self) -> None:
        super().__init__(
            id="1.2",
            section=1,
            title="Výpis a rozpoznávání vzorů",
            instructions="""
### Cíl
Najděte adresář odpovídající vzoru.

### Příkazy
- `ls` - vypíše obsah adresáře

### Úkol
1. Vypište obsah (`ls`)
2. Najděte **ADRESÁŘ** začínající na `d` a končící na `a`
3. Odevzdejte jeho název
            """.strip(),
            hints=[
                "Použijte 'ls' pro výpis položek v aktuálním adresáři.",
                "Hledejte adresář začínající na 'd' a končící na 'a'.",
                "Ujistěte se, že odevzdáváte název adresáře, ne souboru.",
                "Adresáře jsou ve výpisu často barevně odlišeny (např. modře).",
            ],
            start_directory="level-1",
            expected_answer="delta",
            allow_cwd_as_answer=True,
            success_message="Správně! Našli jste adresář odpovídající vzoru.",
            validators=[
                CommonMistakeValidator(
                    {
                        (
                            "data.txt",
                            "dog.md",
                            "drama.log",
                        ): "To je soubor, ne adresář. Hledejte adresář začínající na 'd' a končící na 'a'.",
                        "data": "'data' končí na 'a', ale není to adresář. Zkuste 'ls -F' pro rozlišení adresářů.",
                    }
                )
            ],
        )

    @override
    def setup(self, workspace: Path) -> None:
        """Structure already created in 1.1."""
        _setup_level1_common(workspace)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:  # noqa: PLR0911
        """Validate with detailed pedagogical feedback for common mistakes."""
        # Use parent validation first (checks expected_answer, allow_cwd_as_answer)
        success, msg = super().validate(answer, state)
        if success:
            return True, msg

        if answer is None:
            return (
                False,
                "Musíte zadat odpověď. Odevzdejte název adresáře: shellgame submit <název>",
            )

        answer = answer.strip()
        if answer in ["data.txt", "dog.md", "drama.log"]:
            return (
                False,
                f"'{answer}' je soubor, ne adresář. Hledejte adresář začínající na 'd' a končící na 'a'.",
            )
        if answer == "data":
            return (
                False,
                "'data' končí na 'a', ale není to adresář. Zkuste 'ls -F' pro rozlišení adresářů.",
            )

        if answer.startswith("d") and not answer.endswith("a"):
            return (
                False,
                f"'{answer}' začíná na 'd', ale nekončí na 'a'. Hledáme vzor d...a.",
            )
        if answer.endswith("a") and not answer.startswith("d"):
            return (
                False,
                f"'{answer}' končí na 'a', ale nezačíná na 'd'. Hledáme vzor d...a.",
            )

        return (
            False,
            f"'{answer}' neodpovídá vzoru. Hledejte adresář začínající na 'd' a končící na 'a'.",
        )


class Level1_3(Level):
    """Level 1.3: Enter & Report (Extension Concept)."""

    def __init__(self) -> None:
        super().__init__(
            id="1.3",
            section=1,
            title="Vstup a hlášení (Koncept přípony)",
            instructions="""
### Cíl
Identifikujte soubor bez přípony.

### Příkazy
- `cd <název>` - změnit adresář

### Úkol
1. Jděte do `alpha` (`cd alpha`)
2. Najděte soubor uvnitř (`ls`)
3. Odevzdejte název souboru **BEZ** přípony (část za tečkou)
            """.strip(),
            hints=[
                "Použijte 'cd alpha' pro vstup do adresáře alpha.",
                "Vypište obsah pomocí 'ls'. Uvidíte soubor s příponou '.txt'.",
                "Odevzdejte název tohoto souboru, ale vynechejte část '.txt'.",
                "Příklad: Pokud je soubor 'data.csv', odevzdejte 'data'.",
            ],
            start_directory="level-1",
            required_cwd="alpha",
            require_answer=True,
            expected_answer="inside",
            success_message="Správně! Správně jste odstranili příponu.",
            validators=[
                CommonMistakeValidator(
                    {
                        "inside.txt": Messages.L1_3_INCLUDED_EXTENSION,
                        "alpha": (
                            "'alpha' je název adresáře, ne souboru uvnitř. "
                            "Nejdřív vstupte do alpha a podívejte se, co je uvnitř."
                        ),
                    }
                )
            ],
        )

    @override
    def setup(self, workspace: Path) -> None:
        """Structure already created in 1.1."""
        _setup_level1_common(workspace)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        # Parent validation handles required_cwd, expected_answer, and CommonMistakeValidator
        success, msg = super().validate(answer, state)
        if success:
            return True, msg

        # Additional custom check
        if answer and "." in answer:
            return (
                False,
                f"'{answer}' stále obsahuje příponu (část za tečkou). Odevzdejte pouze název bez přípony.",
            )

        return False, msg


class Level1_4(Level):
    """Level 1.4: Return to Base - Learn cd .. to go up one level."""

    def __init__(self) -> None:
        super().__init__(
            id="1.4",
            section=1,
            title="Návrat na základnu",
            instructions="""
### Cíl
Vraťte se o úroveň výše.

### Příkazy
- `cd ..` - jít o úroveň výše

### Úkol
1. Jděte do nadřazeného adresáře (`cd ..`)
2. Odevzdejte název tohoto adresáře
            """.strip(),
            hints=[
                "Použijte 'cd ..' pro přesun o jednu úroveň adresáře výše.",
                "Po přesunu spusťte 'pwd' pro potvrzení, že jste v nadřazeném adresáři.",
                "Odevzdejte název adresáře, do kterého jste se právě přesunuli.",
                "Dvě tečky '..' vždy reprezentují nadřazený adresář.",
            ],
            start_directory="level-1/alpha",
            require_answer=True,
            expected_answer="level-1",
        )

    def setup(self, workspace: Path) -> None:
        """Structure already created."""
        _setup_level1_common(workspace)


class Level1_5(Level):
    """Level 1.5: Deep Dive - Navigate down."""

    def __init__(self) -> None:
        super().__init__(
            id="1.5",
            section=1,
            title="Hluboký ponor",
            instructions="""
### Cíl
Sestupte hluboko do adresářové struktury.

### Úkol
1. ShellGame vás na začátku levelu umístí do správné části workspace
   (nemusíte spoléhat na to, kde jste skončili minule).
2. Jděte do `gamma/deep/a/b/c/` (v adresáři `level-1`)
3. Odevzdejte název aktuálního adresáře
            """.strip(),
            hints=[
                "Použijte 'cd' pro vstup do adresářů.",
                "Můžete jít postupně: cd gamma, cd deep, cd a...",
                "Nebo najednou: cd gamma/deep/a/b/c",
                "Pokud nejste v 'level-1', nejprve se do něj přesuňte (ověřte si to příkazem 'pwd').",
            ],
            start_directory="level-1",
            require_answer=True,
            expected_answer="c",
        )

    def setup(self, workspace: Path) -> None:
        """Create deep directory structure."""
        _setup_level1_common(workspace)
        deep_path = workspace / "level-1" / "gamma" / "deep" / "a" / "b" / "c"
        deep_path.mkdir(parents=True, exist_ok=True)


class Level1_6(Level):
    """Level 1.6: Multi-Level Ascent - Learn cd ../../.."""

    def __init__(self) -> None:
        super().__init__(
            id="1.6",
            section=1,
            title="Víceúrovňový výstup",
            instructions="""
### Cíl
Vystoupejte o více úrovní najednou.

### Příkazy
- `cd ../../..` - jít o 3 úrovně výše

### Úkol
1. ShellGame vás na začátku levelu umístí do správného adresáře (nemusíte řešit, kde jste skončili minule).
2. Vraťte se o 3 úrovně výše **JEDNÍM** příkazem
3. Odevzdejte název adresáře, kde jste skončili
            """.strip(),
            hints=[
                "Použijte 'cd ../../..' pro přesun o 3 úrovně výše najednou.",
                "Odevzdejte název adresáře, ve kterém jste skončili (měl by to být 'deep').",
            ],
            start_directory="level-1/gamma/deep/a/b/c",
            expected_answer="deep",
            allow_cwd_as_answer=True,
            success_message="Správně! Úspěšně jste vystoupali o 3 úrovně.",
        )

    def setup(self, workspace: Path) -> None:
        """Structure already created in 1.5."""
        _setup_level1_common(workspace)
        (workspace / "level-1" / "gamma" / "deep" / "a" / "b" / "c").mkdir(parents=True, exist_ok=True)


class Level1_7(Level):
    """Level 1.7: Maze Navigation - Follow instruction files through a maze."""

    def __init__(self) -> None:
        super().__init__(
            id="1.7",
            section=1,
            title="Navigace v bludišti",
            instructions="""
### Cíl
Projděte bludištěm podle instrukcí.

### Pravidla
- Start: `level-1/maze/00/`
- Sledujte soubory začínající na `GO_`
- `GO_TO_DIR_x` -> `cd x`
- `GO_UP_N_...` -> `cd ..` (N-krát)

### Úkol
1. Jděte do startu
2. Sledujte instrukce až do cíle
3. Odevzdejte název cílového adresáře
            """.strip(),
            hints=[
                "Sledujte pouze názvy souborů začínající na 'GO_'. Vypište je pomocí 'ls'.",
                "Pro instrukce typu 'GO_UP_4_THEN_GO_TO_02' použijte 'cd ../../../..' a poté 'cd 02'.",
                "Pokračujte ve sledování instrukcí, dokud nenajdete soubor 'VICTORY.marker'.",
                "Ignorujte soubory, které nezačínají na 'GO_', jsou to pasti.",
            ],
            extension=True,
            start_directory="level-1/maze/00",
            required_cwd="final",
            success_message="Správně! Prošli jste bludištěm.",
        )

    def setup(self, workspace: Path) -> None:
        """Create the maze structure with instruction files."""
        _setup_level1_common(workspace)
        maze_base = workspace / "level-1" / "maze"

        # Create maze directories and instruction files.
        # The specification defines 00/ as the start, and 01/02/03/04 as *siblings*
        # of 00/ (i.e., all are direct children of maze/).
        maze_structure = {
            "00": ["GO_TO_DIR_01", "side", "trap"],
            # decoy branches from the start
            "00/side": ["GO_TO_DIR_loop"],
            "00/side/loop": ["GO_UP_2_THEN_GO_TO_00"],
            "00/trap": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "01": ["GO_TO_DIR_deep", "shallow", "surface", "deep"],
            "01/deep": ["GO_TO_DIR_a", "c", "d", "a", "offtrack"],
            "01/deep/offtrack": ["GO_TO_DIR_a"],
            "01/deep/a": ["GO_TO_DIR_b", "x", "z", "b", "alt"],
            "01/deep/a/alt": ["GO_UP_1_THEN_GO_TO_a"],
            "01/shallow": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "01/surface": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "01/deep/c": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "01/deep/d": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "01/deep/a/x": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "01/deep/a/z": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            # From .../01/deep/a/b, to reach 02 (a sibling of 01 under maze/), we must go up:
            # b -> a -> deep -> 01 -> maze  ==> 4 levels, then enter 02.
            "01/deep/a/b": ["GO_UP_4_THEN_GO_TO_02", "decoy.txt", "note.md", "wrong"],
            "01/deep/a/b/wrong": ["GO_UP_1_THEN_GO_TO_b"],
            "02": ["GO_UP_1_THEN_GO_TO_03", "stray.txt", "readme.md"],
            "02/side": ["GO_UP_1_THEN_GO_TO_02"],
            "03": ["GO_TO_DIR_x", "w", "z", "x"],
            "03/x": ["GO_TO_DIR_y", "q", "r", "y"],
            "03/x/y": ["GO_UP_1_THEN_GO_TO_04", "marker.txt"],
            "03/w": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "03/z": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "03/x/q": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "03/x/r": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "04": ["GO_TO_DIR_final", "finish", "end", "done"],
            "04/finish": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "04/end": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "04/done": ["YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"],
            "04/final": ["VICTORY.marker"],
        }

        for dir_path, contents in maze_structure.items():
            full_path = maze_base / dir_path
            full_path.mkdir(parents=True, exist_ok=True)

            # In every directory, we want either:
            # - an instruction marker (a file starting with `GO_`), OR
            # - a warning marker (`YOU_ARE_NOT_SUPPOSED_TO_BE_HERE`),
            # but never both.
            has_go_file = any(item.startswith("GO_") for item in contents)

            for item in contents:
                item_path = full_path / item
                if (
                    item.startswith("GO_")
                    or item == "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"
                    or item.endswith((".txt", ".md", ".marker"))
                ):
                    # Create as file
                    item_path.write_text("")
                else:
                    # Create as directory (decoy branch)
                    item_path.mkdir(exist_ok=True)

                    # Decoy directories should be clearly marked.
                    # (And because they never contain a `_GO_` instruction, this
                    # also satisfies the "GO xor WARNING" invariant.)
                    (item_path / "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE").write_text("")

            if not has_go_file:
                # Only directories without navigation instructions get the warning marker.
                (full_path / "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE").write_text("")
            else:
                # Enforce XOR: if a directory provides navigation instructions,
                # it must not also have the warning marker (could be left over
                # from an older version or a partial rebuild).
                (full_path / "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE").unlink(missing_ok=True)


class Level1_8(Level):
    """Level 1.8: Absolute Path Jump - Learn to use absolute paths."""

    def __init__(self) -> None:
        super().__init__(
            id="1.8",
            section=1,
            title="Skok absolutní cestou",
            instructions="""
### Cíl
Použijte absolutní cestu.

### Příkazy
- `cd /cesta` - absolutní cesta (od kořene)

### Úkol
1. Zjistěte svou aktuální polohu (`pwd`)
2. Použijte **JEDEN** příkaz `cd` s absolutní cestou do:
   `level-1/absolute-target/`
3. Odevzdejte název cílového adresáře
            """.strip(),
            hints=[
                "Absolutní cesty začínají na /. Použijte 'pwd' pro zobrazení vaší plné cesty.",
                "Sestavte plnou cestu kombinací výstupu pwd a cílového adresáře.",
                "Odevzdejte název cílového adresáře.",
                "Příklad: cd /tmp/shellgame-user/level-1/absolute-target",
            ],
            start_directory="level-1",
            required_cwd="absolute-target",
            marker_name=MarkerManager.LEVEL1_8_ABSOLUTE_CD,
            marker_error=Messages.ABSOLUTE_CD_NOT_USED,
            success_message="Správně! Dostali jste se sem absolutní cestou.",
        )

    def setup(self, workspace: Path) -> None:
        """Create absolute-target directory."""
        _setup_level1_common(workspace)
        target_dir = workspace / "level-1" / "absolute-target"
        target_dir.mkdir(parents=True, exist_ok=True)
        (target_dir / "PLACEHOLDER.answer").write_text("")

    @property
    @override
    def hooks(self) -> dict[str, callable]:
        return {"cd": self._handle_cd}

    def _handle_cd(self, *, target: str | None, pwd: str | None, post_move: bool, state: GameStateProtocol) -> None:
        if post_move:
            return

        # Level 1.8: Enforce absolute path
        if not target:
            sys.stderr.write("ShellGame (1.8): Musíte zadat cestu.\n")
            sys.exit(1)

        if not target.startswith("/"):
            sys.stderr.write("ShellGame (1.8): Musíte použít absolutní cestu (začínající na /).\n")
            sys.exit(1)

        # Valid absolute path used
        MarkerManager.from_state(state).create(MarkerManager.LEVEL1_8_ABSOLUTE_CD)


class Level1_9(Level):
    """Level 1.9: Root to Home Walk (Extension) - Manual path reconstruction."""

    def __init__(self) -> None:
        super().__init__(
            id="1.9",
            section=1,
            title="Cesta z kořene domů",
            instructions="""
### Cíl
Zrekonstruujte cestu domů.

### Příkazy
- `cd /` - jít do kořene
- `echo $HOME` - zobrazit cestu domů

### Úkol
1. Jděte do kořene (`cd /`)
2. Zjistěte cestu domů (`echo $HOME`)
3. Jděte domů krok za krokem (každý segment zvlášť)
4. Nakonec použijte jen `shellgame submit`
            """.strip(),
            hints=[
                "Zjistěte cestu k domovu pomocí 'echo $HOME'.",
                "Začněte v / a vstupujte do každého adresáře v cestě jeden po druhém (bez přeskakování).",
                "Tip: pokud je HOME třeba /home/ada, udělejte: cd / ; cd home ; cd ada.",
                "V tomhle levelu neodevzdáváte textovou odpověď – důležitá je správná sekvence `cd`.",
            ],
            extension=True,
            marker_name=MarkerManager.LEVEL1_9_CD_WALK_COMPLETED,
            marker_error=Messages.CD_WALK_NOT_COMPLETED,
            success_message="Správně! Došli jste domů krok za krokem.",
            validators=[HomeDirectoryValidator(check_basename=False)],
        )

    @override
    def setup(self, workspace: Path) -> None:
        """No specific setup needed."""
        return

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)

    @property
    @override
    def hooks(self) -> dict[str, callable]:
        return {"cd": self._handle_cd}

    def _handle_cd(self, *, target: str | None, pwd: str | None, post_move: bool, state: GameStateProtocol) -> None:
        markers = MarkerManager.from_state(state)

        if not post_move:
            # Level 1.9 Pre-move: Enforce step-by-step (no jumps)
            if not target:
                # cd without args -> jump home -> forbidden
                sys.stderr.write("ShellGame (1.9): Skoky nejsou povoleny. Jděte krok za krokem.\n")
                sys.exit(1)

            if target == "/":
                return  # Allowed to start

            # Check for multi-segment paths
            # We allow "dir" or "dir/" but not "dir/subdir" or "/dir"
            cleaned = target.rstrip("/")

            if target.startswith("/"):
                sys.stderr.write("ShellGame (1.9): Absolutní skoky nejsou povoleny (kromě cd /).\n")
                sys.exit(1)

            if "/" in cleaned:
                sys.stderr.write("ShellGame (1.9): Cestujte po jednom segmentu (adresáři).\n")
                sys.exit(1)

        else:
            # Level 1.9 Post-move: Track step-by-step walk from root to home
            if not pwd:
                return

            current_path = Path(pwd)

            # If at root, start tracking (reset)
            if str(current_path) == "/":
                markers.create(MarkerManager.LEVEL1_9_CD_WALK_PROGRESS, "/")
                return

            # If we have progress, check if this step is valid
            progress = markers.read(MarkerManager.LEVEL1_9_CD_WALK_PROGRESS)
            if not progress:
                return

            lines = progress.strip().splitlines()
            if not lines:
                return

            last_path = Path(lines[-1])

            # Valid step: moving from parent to child (one level down)
            if current_path.parent == last_path:
                # Append new path
                new_progress = progress + "\n" + str(current_path)
                markers.create(MarkerManager.LEVEL1_9_CD_WALK_PROGRESS, new_progress)

                # Check completion
                if current_path == Path.home():
                    markers.create(MarkerManager.LEVEL1_9_CD_WALK_COMPLETED)

            elif current_path == last_path:
                # No-op (stayed in same dir)
                pass
            else:
                # Invalid step (jumped or went up/sideways), reset progress
                markers.remove(MarkerManager.LEVEL1_9_CD_WALK_PROGRESS)


class Level1_10(Level):
    """Level 1.10: Home Confirmation (Extension)."""

    def __init__(self) -> None:
        super().__init__(
            id="1.10",
            section=1,
            title="Potvrzení domova",
            instructions="""
### Cíl
Ověřte, že jste doma.

### Příkazy
- `cd ~` - jít domů

### Úkol
1. Jděte domů (`cd ~`)
2. Odevzdejte název domovského adresáře
            """.strip(),
            hints=[
                "Můžete použít 'cd ~' nebo 'cd $HOME' pro rychlý návrat domů.",
                "Použijte 'pwd' pro kontrolu, kde jste.",
                "Odevzdejte název vašeho domovského adresáře (poslední část cesty).",
                "Vlnovka '~' je zkratka pro domovský adresář aktuálního uživatele.",
            ],
            extension=True,
            allow_cwd_as_answer=True,
            success_message="Správně! Jste doma.",
            validators=[HomeDirectoryValidator(check_basename=True)],
        )

    @override
    def setup(self, workspace: Path) -> None:
        """No specific setup needed."""
        return

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


class Level1_11(Level):
    """Level 1.11: Visualizing Structure (Optional)."""

    def __init__(self) -> None:
        super().__init__(
            id="1.11",
            section=1,
            title="Vizualizace struktury",
            instructions="""
### Cíl
Vizualizujte strukturu.

### Příkazy
- `ls -F` - výpis s typy

### Úkol
1. Jděte do `level-1`
2. Vypište adresáře
3. Odevzdejte seznam adresářů (abecedně, oddělené čárkami)
            """.strip(),
            hints=[
                "Jděte do level-1 a spusťte 'ls'.",
                "Vypište názvy adresářů abecedně, oddělené čárkami.",
                "Ujistěte se, že uvádíte pouze adresáře, ne soubory.",
                "Přepínač -F přidá za názvy adresářů lomítko /, což pomáhá v orientaci.",
            ],
            optional=True,
            start_directory="level-1",
            require_answer=True,
            validators=[OrderedListValidator(["absolute-target", "alpha", "delta", "gamma", "maze", "patterns"])],
        )

    @override
    def setup(self, workspace: Path) -> None:
        """Structure already exists."""
        _setup_level1_common(workspace)
        # Ensure all directories for this level exist
        (workspace / "level-1" / "absolute-target").mkdir(parents=True, exist_ok=True)
        (workspace / "level-1" / "maze").mkdir(parents=True, exist_ok=True)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        return super().validate(answer, state)


class Level1_12(Level):
    """Level 1.12: Section 1 Summary Challenge."""

    def __init__(self) -> None:
        super().__init__(
            id="1.12",
            section=1,
            title="Souhrn Sekce 1",
            instructions="""
### 🎯 Výzva: Otestujte své dovednosti!

Ukažte, co jste se naučili v této sekci. Proveďte následující kroky:

### Úkol
1. Zjistěte svou aktuální polohu (`pwd`)
2. Přejděte do adresáře `level-1/gamma/deep/a/b/c`
3. Vraťte se o 4 úrovně výše jedním příkazem
4. Odevzdejte název adresáře, kde jste skončili

### Shrnutí příkazů Sekce 1
```
pwd           → Kde jsem?
ls            → Co tu je?
cd adresář    → Vstup do adresáře
cd ..         → O úroveň výše
cd ../..      → O více úrovní výše
cd /cesta     → Absolutní cesta
cd ~          → Domů
```

### Odevzdání
`shellgame submit <název_adresáře>`
            """.strip(),
            hints=[
                "Nejdřív se dostaňte do c: cd level-1/gamma/deep/a/b/c",
                "Z 'c' o 4 úrovně výše: cd ../../../..",
                "Spočítejte: c → b → a → deep → gamma. Odpověď je 'gamma'.",
            ],
            start_directory="level-1",
            expected_answer="gamma",
            allow_cwd_as_answer=True,
            success_message="🎉 Výborně! Dokončili jste Sekci 1. Ovládáte základy navigace!",
            validators=[
                CommonMistakeValidator(
                    {
                        "deep": "Téměř! 'deep' je o 3 úrovně nad 'c'. Potřebujete jít o 4 úrovně.",
                        (
                            "a",
                            "b",
                            "c",
                        ): "To není dost vysoko. Spočítejte: c→b→a→deep→gamma = 4 kroky.",
                    }
                )
            ],
        )

    @override
    def setup(self, workspace: Path) -> None:
        """Structure already created in 1.5."""
        _setup_level1_common(workspace)
        # Ensure deep structure exists
        (workspace / "level-1" / "gamma" / "deep" / "a" / "b" / "c").mkdir(parents=True, exist_ok=True)

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        """Validate user completed the challenge."""
        # Parent validation handles expected_answer, allow_cwd_as_answer and CommonMistakeValidator
        success, msg = super().validate(answer, state)
        if success:
            return True, msg

        # If parent validation failed, it might be because of expected_answer mismatch or CommonMistakeValidator match.
        # If it was a CommonMistakeValidator match, msg is already the specific feedback.
        # If it was expected_answer mismatch, msg is generic.

        # We can just return the result from parent, as it covers most cases.
        # The only thing lost is the very specific "X není správně. Začněte v 'c'..." message for unknown wrong answers.
        # But the generic "Expected gamma, got X" is probably fine.

        return False, msg


def get_levels() -> list[Level]:
    """Return all Section 1 level instances."""
    return [
        Level1_0(),
        Level1_1(),
        Level1_2(),
        Level1_3(),
        Level1_4(),
        Level1_5(),
        Level1_6(),
        Level1_7(),
        Level1_8(),
        Level1_9(),
        Level1_10(),
        Level1_11(),
        Level1_12(),
    ]
