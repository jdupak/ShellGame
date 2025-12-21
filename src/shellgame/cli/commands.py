"""CLI commands for ShellGame.

This module should be a thin Click parsing layer.
Core gameplay orchestration lives in `shellgame.core.session.GameSession`.

IMPORTANT:
A few UX policies are still expressed at the Click-layer (e.g. confirmation prompts)
so that we don't accidentally remove safety rails while keeping the business logic
out of Click.

Hard UX rule: Teleport notice helper must remain in this module:
`_teleport_notice(destination: Path)`.

Shell override:
- You can force which subshell ShellGame launches with `--shell bash|fish`.
- Useful when auto-detection is wrong (nested shells / wrappers like `uv` / `make`).
"""

import os
from pathlib import Path
from typing import Optional

import click
from rich.console import Console

from shellgame.core.session import GameSession
from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import get_registry
from shellgame.state.manager import StateManager
from shellgame.ui.display import Display

# Initialize singletons
console = Console()
state_manager = StateManager()
level_registry = get_registry()
display = Display(console)

# Load all levels into registry
initialize_levels()


def _teleport_notice(destination: Path) -> None:
    """Print the standard teleport notice.

    UX rules (must not change):
    - Show only if cwd actually changed (caller responsibility)
    - Message in yellow
    - Destination path in violet
    - No “from -> to”
    - No “reason” text
    """
    console.print(
        f"[yellow] Byli jste [bold]teleportováni[/bold] do adresáře [violet]{destination}[/violet][/yellow]\n"
    )


def _get_session() -> GameSession:
    """Create a session using module singletons.

    Tests monkeypatch `display`, `level_registry`, and `state_manager` in this module.
    Creating the session lazily ensures those patches are respected.
    """
    return GameSession(
        console=console,
        display=display,
        state_manager=state_manager,
        level_registry=level_registry,
        teleport_notice=_teleport_notice,
    )


class CzechGroup(click.Group):
    """Custom Click Group to translate help headers to Czech."""

    def get_help(self, ctx: click.Context) -> str:
        """Override get_help to translate headers."""
        help_text = super().get_help(ctx)
        replacements = {
            "Usage:": "Použití:",
            "Options:": "Možnosti:",
            "Commands:": "Příkazy:",
            "Show this message and exit.": "Zobrazit tuto nápovědu a ukončit.",
        }
        for eng, cze in replacements.items():
            help_text = help_text.replace(eng, cze)
        return help_text


# NOTE:
# Shell integration helpers were consolidated into `shellgame.cli.subshell`.
# Core boot behavior (including subshell launch decision) is orchestrated by GameSession.


def get_level_start_directory(level_id: str, workspace: Path) -> Optional[Path]:
    """
    Get the starting directory for a level.

    Args:
        level_id: Level identifier (e.g., "1.3")
        workspace: Workspace root path

    Returns:
        Starting directory path or None if level starts in current location
    """
    level = level_registry.get(level_id)
    if level:
        return level.get_start_directory(workspace)
    return None


@click.group(cls=CzechGroup, invoke_without_command=True)
@click.option("--devmode", is_flag=True, hidden=True, help="Enable developer mode")
@click.option(
    "--shell",
    "forced_shell",
    type=click.Choice(["bash", "fish"], case_sensitive=False),
    default=None,
    help="Vynutit typ subshellu (bash/fish).",
)
@click.pass_context
def cli(ctx: click.Context, devmode: bool, forced_shell: Optional[str]) -> None:
    """ShellGame - Interaktivní výuka navigace v terminálu."""
    # Store devmode in context object
    ctx.ensure_object(dict)
    ctx.obj["devmode"] = devmode

    wrapped = bool(os.environ.get("SHELLGAME_WRAPPER"))

    # Optional override for subshell selection (useful under wrappers / nested shells).
    if forced_shell:
        os.environ["SHELLGAME_FORCE_SHELL"] = forced_shell.lower()

    try:
        boot = _get_session().boot_if_needed(wrapped=wrapped, devmode=devmode, parent_shell="unknown")
    except Exception as e:
        console.print(f"[bold red]CHYBA: Nepodařilo se spustit herní shell ({e})[/bold red]")
        ctx.exit(1)

    if boot.should_exit:
        ctx.exit(boot.exit_code)

    if ctx.invoked_subcommand is None:
        _get_session().show_current_level()


@cli.command()
def init() -> None:
    """Inicializace relace ShellGame (již není potřeba, děje se automaticky)."""
    _get_session().init()


@cli.command()
@click.option(
    "-r",
    "--repeat",
    is_flag=True,
    help="Zobrazit znovu všechny již zobrazené nápovědy pro aktuální level.",
)
def hint(repeat: bool) -> None:
    """Zobrazit nápovědu pro aktuální level."""
    _get_session().hint(repeat=repeat)


@cli.command()
@click.argument("answer", required=False)
@click.pass_context
def submit(ctx: click.Context, answer: Optional[str] = None) -> None:
    """Odeslat odpověď pro aktuální level."""
    # Keep Click parsing here; core submit logic is in GameSession (including auto-advance for .0)
    _get_session().submit(answer)


@cli.command()
def status() -> None:
    """Zobrazit postup a statistiky."""
    _get_session().status()


@cli.command()
def reset() -> None:
    """Obnovit strukturu aktuálního levelu."""
    _get_session().reset()


@cli.command()
@click.option(
    "--section",
    "section_num",
    type=int,
    default=None,
    help="Číslo sekce, kterou chcete zopakovat (např. 1).",
)
@click.option(
    "--level",
    "level_id",
    type=str,
    default=None,
    help="ID levelu ke zopakování (např. 1.7). Pokud neuvedete, zopakuje se aktuální level.",
)
def repeat(section_num: Optional[int], level_id: Optional[str]) -> None:
    """Znovu zobrazit zadání (aktuálního nebo zvoleného) levelu."""
    _get_session().repeat(section_num=section_num, level_id=level_id)


@cli.command(name="show")
@click.option(
    "--section",
    is_flag=True,
    help="Zobrazit znovu úvod aktuální sekce (X.0).",
)
@click.option(
    "--level",
    is_flag=True,
    help="Zobrazit znovu zadání aktuálního levelu.",
)
def show(section: bool, level: bool) -> None:
    """Zobrazit znovu zadání aktuálního levelu nebo úvod aktuální sekce."""
    state = state_manager.load()
    if not state:
        display.show_not_initialized()
        return

    # Exactly one mode must be selected.
    if (1 if section else 0) + (1 if level else 0) != 1:
        console.print("[yellow]Použití: shellgame show --level  nebo  shellgame show --section[/yellow]\n")
        return

    if level:
        target_id = state.current_level
    else:
        # Section intro is always X.0 based on the current level.
        try:
            section_prefix = state.current_level.split(".", 1)[0]
            int(section_prefix)  # sanity check
            target_id = f"{section_prefix}.0"
        except Exception:
            console.print(f"[red]Chyba: Neplatný formát aktuálního levelu: {state.current_level}[/red]\n")
            return

    target_level = level_registry.get(target_id)
    if target_level is None:
        console.print(f"[red]Chyba: Level {target_id} nenalezen[/red]\n")
        return

    display.show_instructions(target_level)


@cli.command()
@click.confirmation_option(prompt="Opravdu chcete odstranit všechna data ShellGame?")
def remove() -> None:
    """Smazat stav a pracovní prostor."""
    _get_session().remove()


@cli.command()
def exit() -> None:
    """Ukončit ShellGame."""
    console.print("[yellow]Ukončuji ShellGame...[/yellow]")
    raise SystemExit(0)


@cli.group(hidden=True)
def dev() -> None:
    """Developer tools."""
    pass


@dev.command(name="jump")
@click.argument("level_id")
def dev_jump(level_id: str) -> None:
    """Přejít na konkrétní level (Dev only)."""
    _get_session().dev_jump_to(level_id)


@dev.command(name="next")
def dev_next() -> None:
    """Přejít na další level (Dev only)."""
    state = state_manager.load()
    if not state:
        display.show_not_initialized()
        return

    next_id = level_registry.next_level(state.current_level)
    if not next_id:
        console.print("[yellow]Žádný další level.[/yellow]")
        return

    _get_session().dev_jump_to(next_id)


@dev.command(name="prev")
def dev_prev() -> None:
    """Přejít na předchozí level (Dev only)."""
    state = state_manager.load()
    if not state:
        display.show_not_initialized()
        return

    # This is inefficient but simple: iterate to find prev
    levels = level_registry.list_levels()
    prev_id = None
    for lvl in levels:
        if lvl.id == state.current_level:
            break
        prev_id = lvl.id

    if not prev_id:
        console.print("[yellow]Žádný předchozí level.[/yellow]")
        return

    _get_session().dev_jump_to(prev_id)


@dev.command(name="reload")
def dev_reload() -> None:
    """Znovu načíst aktuální level (Dev only)."""
    state = state_manager.load()
    if not state:
        display.show_not_initialized()
        return

    level = level_registry.get(state.current_level)
    if not level:
        console.print(f"[red]Chyba: Level {state.current_level} nenalezen[/red]\n")
        return

    # Reset and show instructions again (no direct shell integration here).
    level.reset(state.workspace)
    display.wait_for_continue()
    display.show_instructions(level)
    console.print(f"[green]✓ Level {state.current_level} znovu načten.[/green]")


@dev.command(name="start")
def dev_start() -> None:
    """Jump to the first level (Dev only)."""
    levels = level_registry.list_levels()
    if not levels:
        console.print("[red]No levels registered[/red]")
        return

    first_id = levels[0].id
    ctx = click.get_current_context()
    ctx.invoke(dev_jump, level_id=first_id)
