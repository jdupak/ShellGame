"""Navigation and path manipulation levels."""

from __future__ import annotations

import re
from pathlib import Path

from typing_extensions import override

from shellgame.levels.base import Level, block_cd
from shellgame.levels.cdpolicy import (
    CdEvidence,
    CdPolicy,
    FromDirectory,
    RequireAbsolutePath,
    RequireEvidence,
    RequireExactCommand,
)
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    AtDirectory,
    AtHome,
    Completion,
    Evidence,
    ExactAnswer,
    OrderedListAnswer,
)
from shellgame.levels.fixture import FileFixture, WorkspaceFixture
from shellgame.levels.solution import Chdir, GoHome, PerformCd, RecordEvidence, Solution, WalkHome
from shellgame.markers import MarkerManager
from shellgame.messages import Messages
from shellgame.paths import WORKSPACE_ROOT
from shellgame.protocols import CdHookCallback, GameStateProtocol, ValidationResult

section = Section(
    1,
    root="level-1",
    fixture=WorkspaceFixture(
        directories=("alpha", "delta", "gamma", "patterns", "patterns/omega"),
        files=(
            FileFixture("alpha/inside.txt", "first step", overwrite=False),
            FileFixture("delta/single.dat", "momentum gained", overwrite=False),
            FileFixture("patterns/data.txt", overwrite=False),
            FileFixture("patterns/dog.md", overwrite=False),
            FileFixture("patterns/drama.log", overwrite=False),
            FileFixture("patterns/zebra.txt", overwrite=False),
        ),
    ),
)


@section.level(0)
class Section1Intro(Level):
    is_intro = True
    title = "Navigace"
    instructions_file = "section1_intro.md"
    hints = ["Přečtěte si úvod a pokračujte stisknutím Enter."]
    start_directory = WORKSPACE_ROOT
    success_message = "Jdeme na to!"


@section.level(1)
class PwdLevel(Level):
    solution = Solution(steps=(RecordEvidence(MarkerManager.PWD_USED),), answer="level-1")
    title = "Aktuální umístění"
    instructions = """
        ### Cíl
        Zjistěte název aktuálního adresáře.

        ### Příkazy
        - `pwd` - vypíše celou cestu k aktuálnímu adresáři

        ### Úkol
        1. Spusťte `pwd`
        2. Odevzdejte název posledního adresáře v cestě (basename)

        ### Příklad
        Cesta: `/home/student/dokumenty` -> Odpověď: `dokumenty`

        ### Odevzdání
        `shellgame submit <název>`
        """
    hints = [
        "Příkaz 'pwd' (Print Working Directory) vám ukáže, kde jste. Zkuste ho!",
        "Podívejte se na výstup 'pwd'. Co je za posledním lomítkem?",
        "Basename je poslední část cesty. Z cesty '/home/student/dokumenty' byste odevzdali 'dokumenty'.",
    ]
    start_directory = ""
    completion = Completion(
        answer=ExactAnswer("level-1"),
        requirements=(Evidence(MarkerManager.PWD_USED, Messages.L1_1_USE_PWD_FIRST),),
    )


@section.level(2)
class LsLevel(Level):
    solution = Solution(answer="delta")
    title = "Výpis a rozpoznávání vzorů"
    instructions = """
        ### Cíl
        Najděte adresář odpovídající vzoru.

        ### Příkazy
        - `ls` - vypíše obsah adresáře

        ### Úkol
        1. Vypište obsah (`ls`)
        2. Najděte **ADRESÁŘ** začínající na `d` a končící na `a`
        3. Odevzdejte jeho název
        """
    hints = [
        "Použijte 'ls' pro výpis položek v aktuálním adresáři.",
        "Hledejte adresář začínající na 'd' a končící na 'a'.",
        "Ujistěte se, že odevzdáváte název adresáře, ne souboru.",
        "Adresáře jsou ve výpisu často barevně odlišeny (např. modře).",
    ]
    start_directory = ""
    success_message = "Správně! Našli jste adresář odpovídající vzoru."
    completion = Completion(
        answer=ExactAnswer(
            "delta",
            mistakes={
                "data.txt": "To je soubor, ne adresář. Hledejte adresář začínající na 'd' a končící na 'a'.",
                "dog.md": "To je soubor, ne adresář. Hledejte adresář začínající na 'd' a končící na 'a'.",
                "drama.log": "To je soubor, ne adresář. Hledejte adresář začínající na 'd' a končící na 'a'.",
                "data": "'data' končí na 'a', ale není to adresář. Zkuste 'ls -F' pro rozlišení adresářů.",
            },
            required_message="Musíte zadat odpověď. Odevzdejte název adresáře: shellgame submit <název>",
        ),
        allow_empty_when=AtDirectory("delta"),
    )

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        normalized = answer.strip().rstrip("/") if answer is not None else None
        success, msg = super().validate(normalized, state)
        if success or normalized is None:
            return success, msg

        # Pattern-shape diagnostics that cannot be expressed as a fixed mistake map.
        if normalized.startswith("d") and not normalized.endswith("a"):
            return (
                False,
                f"'{normalized}' začíná na 'd', ale nekončí na 'a'. Hledáme vzor d...a.",
            )
        if normalized.endswith("a") and not normalized.startswith("d"):
            return (
                False,
                f"'{normalized}' končí na 'a', ale nezačíná na 'd'. Hledáme vzor d...a.",
            )

        return False, msg


@section.level(3)
class ExtensionLevel(Level):
    solution = Solution(steps=(Chdir("alpha"),), answer="inside")
    title = "Vstup a hlášení (Koncept přípony)"
    instructions = """
        ### Cíl
        Identifikujte soubor bez přípony.

        ### Příkazy
        - `cd <název>` - změnit adresář

        ### Úkol
        1. Jděte do `alpha` (`cd alpha`)
        2. Najděte soubor uvnitř (`ls`)
        3. Odevzdejte název souboru **BEZ** přípony (vynechte poslední tečku a část za ní)
        """
    hints = [
        "Použijte 'cd alpha' pro vstup do adresáře alpha.",
        "Vypište obsah pomocí 'ls'. Uvidíte soubor s příponou '.txt'.",
        "Odevzdejte název tohoto souboru, ale vynechejte část '.txt'.",
        "Příklad: Pokud je soubor 'data.csv', odevzdejte 'data'.",
    ]
    start_directory = ""
    success_message = "Správně! Správně jste odstranili příponu."
    completion = Completion(
        answer=ExactAnswer(
            "inside",
            mistakes={
                "inside.txt": Messages.L1_3_INCLUDED_EXTENSION,
                "alpha": (
                    "'alpha' je název adresáře, ne souboru uvnitř. "
                    "Nejdřív vstupte do alpha a podívejte se, co je uvnitř."
                ),
            },
        ),
        requirements=(AtDirectory("alpha"),),
    )

    @override
    def validate(self, answer: str | None, state: GameStateProtocol) -> ValidationResult:
        success, msg = super().validate(answer, state)
        if success:
            return True, msg

        if answer and "." in answer and msg == Messages.INCORRECT:
            return False, Messages.L1_3_INCLUDED_EXTENSION

        return False, msg


@section.level(4)
class CdUpLevel(Level):
    solution = Solution(steps=(PerformCd(".."),), answer="level-1")
    title = "Návrat na základnu"
    instructions = """
        ### Cíl
        Vraťte se o úroveň výše.

        ### Příkazy
        - `cd ..` - jít o úroveň výše

        ### Úkol
        1. Jděte do nadřazeného adresáře (`cd ..`)
        2. Odevzdejte název tohoto adresáře
        """
    hints = [
        "Dvě tečky '..' označují nadřazený (rodičovský) adresář.",
        "Použijte příkaz 'cd ..' pro přesun o jednu úroveň výše.",
        "Spusťte 'pwd' a odevzdejte jen název posledního adresáře (část za posledním '/').",
    ]
    start_directory = "alpha"
    completion = Completion(
        answer=ExactAnswer("level-1"),
        requirements=(
            AtDirectory(""),
            CdEvidence("Použijte pro návrat přesně příkaz `cd ..`."),
        ),
    )
    cd_policy = CdPolicy(rules=(RequireExactCommand("..", "Použijte přesně příkaz 'cd ..'."),))


@section.level(5)
class DeepDiveLevel(Level):
    solution = Solution(steps=(Chdir("gamma/deep/a/b/c"),), answer="c")
    title = "Hluboký ponor"
    instructions = """
        ### Cíl
        Sestupte hluboko do adresářové struktury.

        ### Úkol
        1. Začínáte v adresáři `level-1`
        2. Jděte do `gamma/deep/a/b/c/` (v adresáři `level-1`)
        3. Odevzdejte název aktuálního adresáře
        """
    hints = [
        "Použijte 'cd' pro vstup do adresářů.",
        "Můžete jít postupně: cd gamma, cd deep, cd a...",
        "Nebo najednou: cd gamma/deep/a/b/c",
        "Po přesunu ověřte polohu příkazem 'pwd' a odevzdejte poslední část cesty.",
    ]
    start_directory = ""
    completion = Completion(
        answer=ExactAnswer("c"),
        requirements=(AtDirectory("gamma/deep/a/b/c"),),
    )
    fixture = WorkspaceFixture(directories=("gamma/deep/a/b/c",))


@section.level(6)
class MultiLevelAscentLevel(Level):
    solution = Solution(steps=(PerformCd("../../.."),), answer="deep")
    title = "Víceúrovňový výstup"
    instructions = """
        ### Cíl
        Vystoupejte o více úrovní najednou.

        ### Příkazy
        - `cd ../../..` - jít o 3 úrovně výše

        ### Úkol
        1. ShellGame vás na začátku umístí do správného adresáře (nemusíte řešit, kde jste skončili minule).
        2. Vraťte se o 3 úrovně výše **JEDNÍM** příkazem
        3. Odevzdejte název adresáře, kde jste skončili
        """
    hints = [
        "Cesty lze řetězit: každé '..' představuje posun o jednu úroveň nahoru.",
        "Pro posun o tři úrovně najednou spojte tři segmenty: 'cd ../../..'.",
        "Odevzdejte název adresáře, ve kterém jste skončili. Ověřte si ho příkazem 'pwd'.",
    ]
    start_directory = "gamma/deep/a/b/c"
    success_message = "Správně! Úspěšně jste vystoupali o 3 úrovně."
    completion = Completion(
        answer=ExactAnswer("deep"),
        requirements=(
            AtDirectory("gamma/deep"),
            CdEvidence("Vraťte se o tři úrovně jedním příkazem `cd ../../..`."),
        ),
        allow_empty=True,
    )
    fixture = WorkspaceFixture(directories=("gamma/deep/a/b/c",))
    cd_policy = CdPolicy(rules=(RequireExactCommand("../../..", "Použijte jeden příkaz 'cd ../../..'."),))


_GO_TO_DIR = re.compile(r"^GO_TO_DIR_(.+)$")
_GO_UP_THEN = re.compile(r"^GO_UP_(\d+)_THEN_GO_TO_(.+)$")
_GO_UP_ONLY = re.compile(r"^GO_UP_(\d+)$")


def resolve_maze_instruction(name: str, cwd: Path) -> Path | None:
    """Where a `GO_*` filename would take the player from ``cwd``."""
    if match := _GO_TO_DIR.fullmatch(name):
        return cwd / match.group(1)
    if match := _GO_UP_THEN.fullmatch(name):
        target = cwd
        for _ in range(int(match.group(1))):
            target = target.parent
        return target / match.group(2)
    if match := _GO_UP_ONLY.fullmatch(name):
        target = cwd
        for _ in range(int(match.group(1))):
            target = target.parent
        return target
    return None


def maze_marker_text(name: str) -> str:
    """Short note so `cat` is not a blank page. The filename remains the instruction."""
    if _GO_TO_DIR.fullmatch(name) or _GO_UP_THEN.fullmatch(name) or _GO_UP_ONLY.fullmatch(name):
        return "Instrukce je v názvu tohoto souboru. Řiďte se jménem, ne obsahem.\n"
    if name == "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE":
        return "Tady nemáte být. Vraťte se a sledujte soubory začínající na GO_.\n"
    if name == "VICTORY.marker":
        return "Cíl! Odevzdejte název tohoto adresáře: shellgame submit final\n"
    return ""


_MAZE_TRAP = "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"
# 01/02/03/04 are *siblings* of 00/, so hops between them must go up first.
_MAZE_STRUCTURE: dict[str, tuple[str, ...]] = {
    "00": ("GO_UP_1_THEN_GO_TO_01", "side", "trap"),
    "00/side": ("GO_TO_DIR_loop",),
    "00/side/loop": ("GO_UP_2",),
    "00/trap": (_MAZE_TRAP,),
    "01": ("GO_TO_DIR_deep", "shallow", "surface", "deep"),
    "01/deep": ("GO_TO_DIR_a", "c", "d", "a", "offtrack"),
    "01/deep/offtrack": ("GO_UP_1_THEN_GO_TO_a",),
    "01/deep/a": ("GO_TO_DIR_b", "x", "z", "b", "alt"),
    "01/deep/a/alt": ("GO_UP_1",),
    "01/shallow": (_MAZE_TRAP,),
    "01/surface": (_MAZE_TRAP,),
    "01/deep/c": (_MAZE_TRAP,),
    "01/deep/d": (_MAZE_TRAP,),
    "01/deep/a/x": (_MAZE_TRAP,),
    "01/deep/a/z": (_MAZE_TRAP,),
    "01/deep/a/b": ("GO_UP_4_THEN_GO_TO_02", "decoy.txt", "note.md", "wrong"),
    "01/deep/a/b/wrong": ("GO_UP_1",),
    "02": ("GO_UP_1_THEN_GO_TO_03", "stray.txt", "readme.md"),
    "02/side": ("GO_UP_1",),
    "03": ("GO_TO_DIR_x", "w", "z", "x"),
    "03/x": ("GO_TO_DIR_y", "q", "r", "y"),
    "03/x/y": ("GO_UP_3_THEN_GO_TO_04", "marker.txt"),
    "03/w": (_MAZE_TRAP,),
    "03/z": (_MAZE_TRAP,),
    "03/x/q": (_MAZE_TRAP,),
    "03/x/r": (_MAZE_TRAP,),
    "04": ("GO_TO_DIR_final", "finish", "end", "done"),
    "04/finish": (_MAZE_TRAP,),
    "04/end": (_MAZE_TRAP,),
    "04/done": (_MAZE_TRAP,),
    "04/final": ("VICTORY.marker",),
}


def _is_maze_file(item: str) -> bool:
    return item.startswith("GO_") or item == _MAZE_TRAP or item.endswith((".txt", ".md", ".marker"))


def _maze_fixture() -> WorkspaceFixture:
    directories: list[str] = []
    files: list[FileFixture] = []
    seen_files: set[str] = set()

    def add_file(relative: str, name: str) -> None:
        if relative in seen_files:
            return
        seen_files.add(relative)
        files.append(FileFixture(relative, maze_marker_text(name)))

    for dir_path, contents in _MAZE_STRUCTURE.items():
        directories.append(f"maze/{dir_path}")
        has_go = any(item.startswith("GO_") for item in contents)
        for item in contents:
            relative = f"maze/{dir_path}/{item}"
            if _is_maze_file(item):
                add_file(relative, item)
            else:
                directories.append(relative)
        if not has_go:
            add_file(f"maze/{dir_path}/{_MAZE_TRAP}", _MAZE_TRAP)

    return WorkspaceFixture(
        directories=tuple(dict.fromkeys(directories)),
        files=tuple(files),
        clean=("maze",),
    )


@section.level(7)
class MazeLevel(Level):
    solution = Solution(
        steps=tuple(
            Chdir(step)
            for step in (
                "maze/00",
                "maze/01",
                "maze/01/deep",
                "maze/01/deep/a",
                "maze/01/deep/a/b",
                "maze/02",
                "maze/03",
                "maze/03/x",
                "maze/03/x/y",
                "maze/04",
                "maze/04/final",
            )
        ),
    )
    title = "Navigace v bludišti"
    instructions = """
        ### Cíl
        Projděte bludištěm podle instrukcí.

        ### Pravidla
        - Start: `level-1/maze/00/`
        - Instrukce je v **názvu** souboru `GO_…` (příkaz `ls`)
        - `GO_TO_DIR_x` → `cd x`
        - `GO_UP_N` → `cd ..` (N-krát)
        - `GO_UP_N_THEN_GO_TO_x` → `cd ..` (N-krát), potom `cd x`

        ### Úkol
        1. Jděte do startu
        2. Sledujte instrukce až do cíle
        3. Odevzdejte název cílového adresáře
           `shellgame submit <název>`
        """
    hints = [
        "Sledujte pouze názvy souborů začínající na 'GO_'. Vypište je pomocí 'ls'.",
        "Pro instrukce typu 'GO_UP_4_THEN_GO_TO_02' použijte 'cd ../../../..' a poté 'cd 02'.",
        "Pokračujte ve sledování instrukcí, dokud nenajdete soubor 'VICTORY.marker'.",
        "Ignorujte soubory, které nezačínají na 'GO_', jsou to pasti.",
    ]
    extension = True
    start_directory = "maze/00"
    fixture = _maze_fixture()
    success_message = "Správně! Prošli jste bludištěm."
    completion = Completion(requirements=(AtDirectory("maze/04/final"),))


@section.level(8)
class AbsoluteCdLevel(Level):
    solution = Solution(steps=(PerformCd("absolute-target", absolute=True),))
    title = "Skok absolutní cestou"
    instructions = """
        ### Cíl
        Použijte absolutní cestu.

        ### Příkazy
        - `cd /cesta` - absolutní cesta (od kořene)

        ### Úkol
        1. Začínáte v `level-1`. Zjistěte celou cestu příkazem `pwd`
        2. Použijte **JEDEN** příkaz `cd` s absolutní cestou do podadresáře `absolute-target`
        3. Odevzdejte název cílového adresáře
        """
    hints = [
        "Absolutní cesty začínají na /. Použijte 'pwd' pro zobrazení vaší plné cesty.",
        "K celé cestě vypsané příkazem 'pwd' na startu připojte '/absolute-target'.",
        "Odevzdejte název cílového adresáře.",
        "Za 'cd' napište celou sestavenou cestu od /. Pokud obsahuje mezery, uzavřete ji do uvozovek.",
    ]
    start_directory = ""
    success_message = "Správně! Dostali jste se sem absolutní cestou."
    completion = Completion(
        requirements=(
            AtDirectory("absolute-target"),
            CdEvidence(Messages.ABSOLUTE_CD_NOT_USED),
        )
    )
    fixture = WorkspaceFixture(files=(FileFixture("absolute-target/PLACEHOLDER.answer"),))
    cd_policy = CdPolicy(
        rules=(
            RequireAbsolutePath(
                "Musíte použít absolutní cestu (začínající na /).",
                missing_message="Musíte zadat cestu.",
            ),
        )
    )


@section.level(9)
class HomeWalkLevel(Level):
    solution = Solution(steps=(WalkHome(),))
    title = "Cesta z kořene domů"
    instructions = """
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
        """
    hints = [
        "Zjistěte cestu k domovu pomocí 'echo $HOME'.",
        "Začněte v / a vstupujte do každého adresáře v cestě jeden po druhém (bez přeskakování).",
        "Tip: pokud je HOME třeba /home/ada, udělejte: cd / ; cd home ; cd ada.",
        "V tomhle levelu neodevzdáváte textovou odpověď – důležitá je správná sekvence `cd`.",
    ]
    extension = True
    start_directory = WORKSPACE_ROOT
    reset_markers = (MarkerManager.LEVEL1_9_CD_WALK_PROGRESS,)
    success_message = "Správně! Došli jste domů krok za krokem."
    completion = Completion(
        requirements=(
            AtHome(),
            Evidence(MarkerManager.LEVEL1_9_CD_WALK_COMPLETED, Messages.CD_WALK_NOT_COMPLETED),
        )
    )

    @property
    @override
    def hooks(self) -> dict[str, CdHookCallback]:
        return {"cd": self._handle_cd}

    def _handle_cd(  # noqa: PLR0912
        self, *, target: str | None, pwd: str | None, post_move: bool, state: GameStateProtocol
    ) -> None:
        markers = MarkerManager.from_state(state)

        if not post_move:
            # Level 1.9 Pre-move: Enforce step-by-step (no jumps)
            if self.cd_enforcement_lifted(
                MarkerManager.LEVEL1_9_CD_WALK_COMPLETED, target=target, pwd=pwd, state=state
            ):
                return

            if not target:
                # cd without args -> jump home -> forbidden
                block_cd(self.id, "Skoky nejsou povoleny. Jděte krok za krokem.")

            if target == "/":
                return  # Allowed to start

            # Check for multi-segment paths
            # We allow "dir" or "dir/" but not "dir/subdir" or "/dir"
            cleaned = target.rstrip("/")

            if target.startswith("/"):
                block_cd(self.id, "Absolutní skoky nejsou povoleny (kromě cd /).")

            if "/" in cleaned:
                block_cd(self.id, "Cestujte po jednom segmentu (adresáři).")

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


@section.level(10)
class HomeCheckLevel(Level):
    solution = Solution(steps=(GoHome(),), answer=Path.home().name)
    title = "Potvrzení domova"
    instructions = """
        ### Cíl
        Ověřte, že jste doma.

        ### Příkazy
        - `cd ~` - jít domů

        ### Úkol
        1. Jděte domů (`cd ~`)
        2. Odevzdejte název domovského adresáře
        """
    hints = [
        "Můžete použít 'cd ~' nebo 'cd $HOME' pro rychlý návrat domů.",
        "Použijte 'pwd' pro kontrolu, kde jste.",
        "Odevzdejte název vašeho domovského adresáře (poslední část cesty).",
        "Vlnovka '~' je zkratka pro domovský adresář aktuálního uživatele.",
    ]
    extension = True
    start_directory = WORKSPACE_ROOT
    completion = Completion(
        answer=ExactAnswer(Path.home().name),
        requirements=(AtHome(),),
    )


@section.level(11)
class StructureLevel(Level):
    title = "Vizualizace struktury"
    instructions = """
        ### Cíl
        Vizualizujte strukturu.

        ### Příkazy
        - `ls -F` - výpis s typy

        ### Úkol
        1. Začínáte v `level-1`. Vypište jeho obsah příkazem `ls -F`
        2. Vyberte pouze adresáře (mají na konci lomítko)
        3. Odevzdejte jejich názvy bez lomítek, abecedně a oddělené čárkami
        """
    hints = [
        "Začínáte v level-1; spusťte 'ls -F'.",
        "Vypište názvy adresářů abecedně, oddělené čárkami.",
        "Ujistěte se, že uvádíte pouze adresáře, ne soubory.",
        "Lomítko / ve výpisu označuje adresář. Do odpovědi ho nepište.",
    ]
    optional = True
    start_directory = ""
    fixture = WorkspaceFixture(directories=("absolute-target", "maze"))
    completion = Completion(
        answer=OrderedListAnswer(("absolute-target", "alpha", "delta", "gamma", "maze", "patterns"))
    )


@section.level(12)
class SummaryLevel(Level):
    solution = Solution(
        steps=(
            RecordEvidence(MarkerManager.PWD_USED),
            Chdir("gamma/deep/a/b/c"),
            PerformCd("../../../.."),
        ),
        answer="gamma",
    )
    title = "Souhrn"
    instructions = """
        ### Výzva: Otestujte své dovednosti!

        Ukažte, co jste se naučili v této sekci. Proveďte následující kroky:

        ### Úkol
        1. Začínáte v `level-1`. Zjistěte svou aktuální polohu (`pwd`)
        2. Přejděte do adresáře `gamma/deep/a/b/c`
        3. Vraťte se o 4 úrovně výše jedním příkazem
        4. Odevzdejte název adresáře, kde jste skončili

        ### Shrnutí příkazů
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
        """
    hints = [
        "Ze startu přejděte do c: `cd gamma/deep/a/b/c`.",
        "Z 'c' o 4 úrovně výše: cd ../../../..",
        "Spočítejte úrovně: c → b → a → deep → ? Kde jste skončili, ověří 'pwd'.",
    ]
    start_directory = ""
    reset_markers = (MarkerManager.PWD_USED,)
    success_message = "Výborně! Ovládáte základy navigace!"
    completion = Completion(
        answer=ExactAnswer(
            "gamma",
            mistakes={
                "deep": "Téměř! 'deep' je o 3 úrovně nad 'c'. Potřebujete jít o 4 úrovně.",
                "a": "To není dost vysoko. Vraťte se od startovního 'c' o čtyři úrovně a ověřte polohu pomocí 'pwd'.",
                "b": "To není dost vysoko. Vraťte se od startovního 'c' o čtyři úrovně a ověřte polohu pomocí 'pwd'.",
                "c": "To není dost vysoko. Vraťte se od startovního 'c' o čtyři úrovně a ověřte polohu pomocí 'pwd'.",
            },
        ),
        requirements=(
            AtDirectory("gamma"),
            CdEvidence(
                "Nejdřív použijte `pwd`, přejděte do `c` a vraťte se jedním příkazem o čtyři úrovně.",
            ),
        ),
        allow_empty=True,
    )
    fixture = WorkspaceFixture(directories=("gamma/deep/a/b/c",))

    cd_policy = CdPolicy(
        scope=(FromDirectory("gamma/deep/a/b/c"),),
        rules=(
            RequireEvidence(MarkerManager.PWD_USED, "Nejdřív použijte příkaz 'pwd'."),
            RequireExactCommand("../../../..", "Z adresáře 'c' použijte jeden příkaz 'cd ../../../..'."),
        ),
    )
