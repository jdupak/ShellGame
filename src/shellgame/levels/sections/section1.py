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
    cd_marker,
)
from shellgame.levels.collector import Section
from shellgame.levels.completion import (
    AtDirectory,
    AtHome,
    Completion,
    Evidence,
    ExactAnswer,
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
    success_message = "Správně! `pwd` vypisuje celou cestu; poslední část za lomítkem je adresář, ve kterém stojíte."


@section.level(2)
class LsLevel(Level):
    solution = Solution(answer="delta")
    title = "Soubory, nebo adresáře?"
    instructions = """
        ### Cíl
        Nejprve si prohlédněte běžný výpis. Potom v nápovědě zjistěte, jak zobrazit
        podrobnosti, a rozlište soubor od adresáře bez spoléhání na barvy terminálu.

        ### Příkazy
        - `ls` - vypíše názvy položek v aktuálním adresáři
        - `ls --help` - zobrazí stručnou nápovědu k příkazu `ls`
        - `man ls` - otevře podrobný manuál; ukončíte ho klávesou `q`

        ### Úkol
        1. Spusťte nejprve obyčejné `ls`
        2. V nápovědě najděte přepínač pro dlouhý (podrobný) výpis a použijte ho
        3. V podrobném výpisu značí první znak `d` adresář a `-` běžný soubor
        4. Najděte **adresář**, jehož název začíná na `d` a končí na `a`
        5. Odevzdejte jeho název

        ### Odevzdání
        `shellgame submit <hodnota>`
        """
    hints = [
        "Běžný výpis ukáže názvy, ale bez barev z něj typ položky spolehlivě nepoznáte.",
        "Zjistěte v nápovědě k příkazu `ls`, který přepínač zapíná dlouhý výpis.",
        "Otevřete nápovědu pomocí `ls --help` nebo `man ls` a hledejte výraz 'long listing'.",
        "Spusťte `ls -l`: řádek adresáře začíná `d`, řádek běžného souboru `-`.",
    ]
    start_directory = ""
    fixture = WorkspaceFixture(
        clean=("data", "dog", "dome"),
        directories=("delta", "dome"),
        files=(FileFixture("data"), FileFixture("dog")),
    )
    success_message = "Správně! Podle prvního znaku výpisu `ls -l` jste rozlišili adresář od souboru."
    completion = Completion(
        answer=ExactAnswer(
            "delta",
            mistakes={
                "data": "'data' odpovídá názvem, ale není to adresář: jeho řádek v 'ls -l' začíná '-'.",
                "dome": "'dome' je adresář, ale jeho název nekončí na 'a'.",
            },
            required_message="Musíte zadat název adresáře: shellgame submit <hodnota>",
        ),
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
    success_message = "Správně! `..` neodkazuje na konkrétní jméno, ale vždy na rodiče toho adresáře, kde právě jste."


@section.level(5)
class DeepDiveLevel(Level):
    solution = Solution(steps=(Chdir("gamma/deep/a/b/c"),), answer="c")
    title = "Hluboký ponor"
    instructions = """
        ### Cíl
        Sestupte hluboko do adresářové struktury.

        ### 💡 Tip: Klávesa Tab (doplňování názvů)
        Dlouhé cesty nemusíte vypisovat celé písmeno po písmenu!
        Napište `cd g` a stiskněte **Tab** — shell název `gamma/` doplní za vás.
        Pak napište `d` a znovu stiskněte **Tab**. Klávesa Tab je v terminálu váš největší pomocník proti překlepům.

        ### Úkol
        1. Začínáte v adresáři `level-1`
        2. Jděte do `gamma/deep/a/b/c/` (v adresáři `level-1`) — vyzkoušejte klávesu Tab!
        3. Odevzdejte název aktuálního adresáře
        """
    hints = [
        "Použijte 'cd' pro vstup do adresářů. Vyzkoušejte klávesu Tab pro automatické doplňování!",
        "Můžete jít postupně: cd gamma, cd deep, cd a...",
        "Nebo najednou: cd gamma/deep/a/b/c (napište 'cd g' a stiskněte Tab)",
        "Po přesunu ověřte polohu příkazem 'pwd' a odevzdejte poslední část cesty.",
    ]
    start_directory = ""
    completion = Completion(
        answer=ExactAnswer("c"),
        requirements=(AtDirectory("gamma/deep/a/b/c"),),
    )
    fixture = WorkspaceFixture(directories=("gamma/deep/a/b/c",))
    success_message = "Správně! Lomítka spojují kroky do jedné relativní cesty — jeden `cd` zvládne celou větev."


@section.level(6)
class MultiLevelAscentLevel(Level):
    solution = Solution(steps=(PerformCd("../../.."),))
    title = "Víceúrovňový výstup"
    instructions = """
        ### Cíl
        Vystoupejte o více úrovní najednou.

        ### Příkazy
        - `cd ../../..` - jít o 3 úrovně výše

        ### Úkol
        1. ShellGame vás na začátku umístí do správného adresáře (nemusíte řešit, kde jste skončili minule).
        2. Vraťte se o 3 úrovně výše **JEDNÍM** příkazem
        3. V cíli spusťte pouze `shellgame submit`
        """
    hints = [
        "Cesty lze řetězit: každé '..' představuje posun o jednu úroveň nahoru.",
        "Pro posun o tři úrovně najednou spojte tři segmenty: 'cd ../../..'.",
        "Polohu si ověřte příkazem 'pwd'. Pokud jste správně, spusťte jen 'shellgame submit'.",
    ]
    start_directory = "gamma/deep/a/b/c"
    success_message = "Správně! Úspěšně jste vystoupali o 3 úrovně."
    completion = Completion(
        requirements=(
            AtDirectory("gamma/deep"),
            CdEvidence("Vraťte se o tři úrovně jedním příkazem `cd ../../..`."),
        ),
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
        return (
            "Tady nemáte být. Vraťte se a sledujte soubory začínající na GO_.\n"
            "Ztratili jste se? Příkaz 'shellgame reset' vás vrátí na začátek bludiště (maze/entry).\n"
        )
    if name == "VICTORY.marker":
        return (
            "Konečně skutečný cíl (opravdu_final_v2_FINAL)! Level je automaticky splněn.\n"
            "Případně můžete odevzdat ručně příkazem: shellgame submit\n"
        )
    return ""


_MAZE_TRAP = "YOU_ARE_NOT_SUPPOSED_TO_BE_HERE"
# The long route is deliberate practice: repeated inspection and navigation,
# several local backtracks, compound upward moves, branching, and decoys.
_MAZE_STRUCTURE: dict[str, tuple[str, ...]] = {
    # Entrance area
    "entry": ("GO_TO_DIR_hall", "dungeon", "courtyard"),
    "entry/courtyard": (_MAZE_TRAP,),
    "entry/dungeon": (_MAZE_TRAP,),
    "entry/hall": ("GO_TO_DIR_nexus", "alcove", "side_door"),
    "entry/hall/alcove": (_MAZE_TRAP,),
    "entry/hall/side_door": ("GO_UP_1",),
    # Nexus hub
    "entry/hall/nexus": ("GO_TO_DIR_passages", "sanctum", "rotunda", "archives"),
    "entry/hall/nexus/sanctum": (_MAZE_TRAP,),
    "entry/hall/nexus/archives": (_MAZE_TRAP,),
    "entry/hall/nexus/rotunda": ("GO_UP_1",),
    # Passages branch
    "entry/hall/nexus/passages": ("GO_TO_DIR_tunnel", "gallery", "shaft"),
    "entry/hall/nexus/passages/gallery": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/shaft": ("GO_UP_1",),
    "entry/hall/nexus/passages/tunnel": ("GO_TO_DIR_cavern", "crevice"),
    "entry/hall/nexus/passages/tunnel/crevice": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/cavern": ("GO_TO_DIR_depths", "grotto", "abyss"),
    "entry/hall/nexus/passages/tunnel/cavern/grotto": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/cavern/abyss": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/cavern/depths": (
        "GO_UP_2_THEN_GO_TO_catacombs",
        "echoes.txt",
    ),
    # Catacombs branch
    "entry/hall/nexus/passages/tunnel/catacombs": ("GO_TO_DIR_vault", "ossuary", "crypts"),
    "entry/hall/nexus/passages/tunnel/catacombs/ossuary": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/catacombs/crypts": ("GO_UP_1",),
    "entry/hall/nexus/passages/tunnel/catacombs/vault": (
        "GO_TO_DIR_chamber",
        "iron_cell",
        "sepulcher",
    ),
    "entry/hall/nexus/passages/tunnel/catacombs/vault/iron_cell": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/catacombs/vault/sepulcher": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/catacombs/vault/chamber": (
        "GO_UP_2_THEN_GO_TO_labyrinth",
        "relic.txt",
    ),
    # Labyrinth and final sequence
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth": (
        "GO_TO_DIR_corridor",
        "dead_end",
        "ruins",
    ),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/dead_end": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/ruins": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor": (
        "GO_TO_DIR_shrine",
        "false_exit",
        "mist",
    ),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/false_exit": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/mist": ("GO_UP_1",),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine": (
        "GO_TO_DIR_final",
        "mirage",
        "shadow",
    ),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/mirage": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/shadow": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final": (
        "GO_TO_DIR_final_v2",
        "decoy_exit",
        "draft.txt",
    ),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/decoy_exit": (_MAZE_TRAP,),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2": (
        "GO_TO_DIR_final_final",
        "abandoned_branch",
        "old.bak",
    ),
    ("entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2/abandoned_branch"): (
        _MAZE_TRAP,
    ),
    "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2/final_final": (
        "GO_TO_DIR_opravdu_final_v2_FINAL",
        "fake_end",
    ),
    ("entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2/final_final/fake_end"): (
        _MAZE_TRAP,
    ),
    (
        "entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/"
        "final/final_v2/final_final/opravdu_final_v2_FINAL"
    ): ("VICTORY.marker",),
}


def _is_maze_file(item: str) -> bool:
    return item.startswith("GO_") or item == _MAZE_TRAP or item.endswith((".txt", ".md", ".marker", ".bak"))


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
        if not has_go and not any(item == "VICTORY.marker" for item in contents):
            add_file(f"maze/{dir_path}/{_MAZE_TRAP}", _MAZE_TRAP)

    return WorkspaceFixture(
        directories=tuple(dict.fromkeys(directories)),
        files=tuple(files),
        clean=("maze",),
    )


@section.level(7)
class MazeLevel(Level):
    _SANCTUARY = (
        "maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine"
        "/final/final_v2/final_final/opravdu_final_v2_FINAL"
    )
    solution = Solution(
        steps=tuple(
            Chdir(step)
            for step in (
                "maze/entry",
                "maze/entry/hall",
                "maze/entry/hall/nexus",
                "maze/entry/hall/nexus/passages",
                "maze/entry/hall/nexus/passages/tunnel",
                "maze/entry/hall/nexus/passages/tunnel/cavern",
                "maze/entry/hall/nexus/passages/tunnel/cavern/depths",
                "maze/entry/hall/nexus/passages/tunnel/catacombs",
                "maze/entry/hall/nexus/passages/tunnel/catacombs/vault",
                "maze/entry/hall/nexus/passages/tunnel/catacombs/vault/chamber",
                "maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth",
                "maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor",
                "maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine",
                "maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final",
                "maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/final/final_v2",
                (
                    "maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/"
                    "final/final_v2/final_final"
                ),
                (
                    "maze/entry/hall/nexus/passages/tunnel/catacombs/labyrinth/corridor/shrine/"
                    "final/final_v2/final_final/opravdu_final_v2_FINAL"
                ),
            )
        ),
    )
    title = "Navigace v bludišti"
    instructions = """
        ### Cíl
        Projděte bludištěm podle instrukcí.

        ### Pravidla
        - Start: `level-1/maze/entry/`
        - Instrukce je v **názvu** souboru `GO_…` (příkaz `ls`)
        - `GO_TO_DIR_x` → `cd x`
        - `GO_UP_N` → jděte pomocí `cd ..` o N úrovní výše
        - `GO_UP_N_THEN_GO_TO_x` → jděte o N úrovní výše a potom do `x`
        - Během bludiště nemůžete odejít mimo `maze/`

        ### Úkol
        1. Ze startu sledujte názvy souborů `GO_…`
        2. Bludiště je záměrně delší: procvičíte opakované `ls`, sestup, návraty i přechod k sourozencům
        3. Jakmile vstoupíte do skutečného cíle, level se automaticky splní
           (případně můžete odevzdat `shellgame submit`)

        Pokud se v bludišti ztratíte, příkaz `shellgame reset` vás vrátí zpět na začátek (`maze/entry`).
        """
    hints = [
        "Sledujte pouze názvy souborů začínající na 'GO_'. Vypište je pomocí 'ls'.",
        "U instrukce 'GO_UP_2_THEN_GO_TO_catacombs' se vraťte o dvě úrovně a vstupte do catacombs.",
        "Spojený návrat lze zapsat jako 'cd ../..'; potom pokračujte příkazem 'cd catacombs'.",
        "Ignorujte položky, které nezačínají na 'GO_'; jsou to odbočky nebo pasti.",
        "Pokud se ztratíte, příkaz 'shellgame reset' vás vrátí na start.",
    ]
    start_directory = "maze/entry"
    fixture = _maze_fixture()
    success_message = "Správně! Prošli jste bludištěm až do skutečného cíle!"
    completion = Completion(requirements=(AtDirectory(_SANCTUARY),))

    @property
    @override
    def hooks(self) -> dict[str, CdHookCallback]:
        return {"cd": self._handle_cd}

    def _handle_cd(self, *, target: str | None, pwd: str | None, post_move: bool, state: GameStateProtocol) -> None:
        maze_root = (self.section_path(state.workspace) / "maze").resolve()

        if not post_move:
            # Pre-move: Guard player from leaving level-1/maze before completion
            if self.cd_enforcement_lifted(cd_marker(self.id), target=target, pwd=pwd, state=state):
                return

            if not target:
                # `cd` without arguments attempts to go home (outside the maze)
                block_cd(self.id, "Nemůžete opustit bludiště. Pokračujte v navigaci uvnitř maze/.")

            dest = Path(target).expanduser()
            if not dest.is_absolute():
                base = Path(pwd) if pwd else Path.cwd()
                dest = base / dest

            try:
                dest_resolved = dest.resolve()
            except (OSError, RuntimeError):
                return

            if not dest_resolved.is_relative_to(maze_root):
                block_cd(self.id, "Nemůžete opustit bludiště před jeho dokončením.")
        else:
            # Post-move: Check if user reached true final room for autowin
            if not pwd:
                return

            dest_resolved = Path(pwd).resolve()
            sanctuary_resolved = (self.section_path(state.workspace) / self._SANCTUARY).resolve()
            if dest_resolved == sanctuary_resolved:
                MarkerManager.from_state(state).create(cd_marker(self.id))


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
        3. V cíli spusťte pouze `shellgame submit`
        """
    hints = [
        "Absolutní cesty začínají na /. Použijte 'pwd' pro zobrazení vaší plné cesty.",
        "K celé cestě vypsané příkazem 'pwd' na startu připojte '/absolute-target'.",
        "Po přesunu ověřte polohu příkazem 'pwd' a spusťte jen 'shellgame submit'.",
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

        Poznámka: Váš domovský adresář (`$HOME`) leží mimo herní pracovní prostor.
        """
    hints = [
        "Zjistěte cestu k domovu pomocí 'echo $HOME'.",
        "Začněte v / a vstupujte do každého adresáře v cestě jeden po druhém (bez přeskakování).",
        "Tip: pokud je HOME třeba /home/ada, udělejte: cd / ; cd home ; cd ada.",
        "V tomhle levelu neodevzdáváte textovou odpověď – důležitá je správná sekvence `cd`.",
    ]
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
    start_directory = WORKSPACE_ROOT
    completion = Completion(
        answer=ExactAnswer(Path.home().name),
        requirements=(AtHome(),),
    )
    success_message = "Správně! `~` se rozbalí na váš domovský adresář, ať stojíte kdekoliv v systému."


#: The `1.11` suffix is intentionally unused: a level was retired after IDs had
#: already been persisted in player saves, and renumbering `1.12` would
#: invalidate them. Suffixes are stable identifiers, not positions.
@section.level(12)
class SummaryLevel(Level):
    solution = Solution(
        steps=(
            RecordEvidence(MarkerManager.PWD_USED),
            Chdir("gamma/deep/a/b/c"),
            PerformCd("../../../.."),
        ),
    )
    title = "Souhrn"
    instructions = """
        ### Výzva: Otestujte své dovednosti!

        Ukažte, co jste se naučili v této sekci. Proveďte následující kroky:

        ### Úkol
        1. Začínáte v `level-1`. Zjistěte svou aktuální polohu (`pwd`)
        2. Přejděte do adresáře `gamma/deep/a/b/c`
        3. Vraťte se o 4 úrovně výše jedním příkazem
        4. V cíli spusťte pouze `shellgame submit`

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
        `shellgame submit`
        """
    hints = [
        "Nejdřív si pomocí 'pwd' potvrďte výchozí polohu a cestu rozdělte na jednotlivé adresáře.",
        (
            "Při návratu spočítejte, kolik segmentů musíte z adresáře 'c' odstranit. "
            "Každý segment '..' znamená jednu úroveň."
        ),
        "Použijte `cd gamma/deep/a/b/c` a potom `cd ../../../..`. Výsledek ověřte pomocí 'pwd'.",
    ]
    start_directory = ""
    reset_markers = (MarkerManager.PWD_USED,)
    success_message = "Výborně! Ovládáte základy navigace!"
    completion = Completion(
        requirements=(
            AtDirectory("gamma"),
            CdEvidence(
                "Nejdřív použijte `pwd`, přejděte do `c` a vraťte se jedním příkazem o čtyři úrovně.",
            ),
        ),
    )
    fixture = WorkspaceFixture(directories=("gamma/deep/a/b/c",))

    cd_policy = CdPolicy(
        scope=(FromDirectory("gamma/deep/a/b/c"),),
        rules=(
            RequireEvidence(MarkerManager.PWD_USED, "Nejdřív použijte příkaz 'pwd'."),
            RequireExactCommand("../../../..", "Z adresáře 'c' použijte jeden příkaz 'cd ../../../..'."),
        ),
    )
