"""CLI commands for ShellGame.

This module should be a thin Click parsing layer.
Core gameplay orchestration lives in `shellgame.core.session.GameSession`.

Shell override:
- You can force which subshell ShellGame launches with `--shell bash|fish`.
- Useful when auto-detection is wrong (nested shells / wrappers like `uv` / `make`).
"""

import os
from pathlib import Path
from typing import Optional

import click
from rich.console import Console

from shellgame.cli.boot import boot_if_needed
from shellgame.core.services import GameServices
from shellgame.core.session import GameSession
from shellgame.levels.loader import initialize_levels
from shellgame.levels.registry import get_registry
from shellgame.shell.client import ShellClient
from shellgame.state.manager import StateManager
from shellgame.ui.display import Display

services = GameServices()
console = services.console
state_manager = services.state_manager
level_registry = services.level_registry
display = services.display
shell_client = services.shell_client


def _teleport_notice(destination: Path) -> None:
    console.print(
        f"[yellow] Byli jste [bold]teleportováni[/bold] do adresáře [violet]{destination}[/violet][/yellow]\n"
    )


def _get_session() -> GameSession:
    return GameSession(
        console=console,
        display=display,
        state_manager=state_manager,
        level_registry=level_registry,
        teleport_notice=_teleport_notice,
        shell_client=shell_client,
        workspace_factory=services.get_workspace_manager_factory(),
    )


class CzechGroup(click.Group):
    def get_help(self, ctx: click.Context) -> str:
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


def get_level_start_directory(level_id: str, workspace: Path) -> Optional[Path]:
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
    ctx.ensure_object(dict)
    ctx.obj["devmode"] = devmode

    wrapped = bool(os.environ.get("SHELLGAME_WRAPPER"))

    if forced_shell:
        os.environ["SHELLGAME_FORCE_SHELL"] = forced_shell.lower()

    try:
        boot = boot_if_needed(wrapped=wrapped, devmode=devmode)
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

    if (1 if section else 0) + (1 if level else 0) != 1:
        console.print("[yellow]Použití: shellgame show --level  nebo  shellgame show --section[/yellow]\n")
        return

    if level:
        target_id = state.current_level
    else:
        try:
            section_prefix = state.current_level.split(".", 1)[0]
            int(section_prefix)
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
    pass


@dev.command(name="jump")
@click.argument("level_id")
def dev_jump(level_id: str) -> None:
    _get_session().dev_jump_to(level_id)


@dev.command(name="next")
def dev_next() -> None:
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
    state = state_manager.load()
    if not state:
        display.show_not_initialized()
        return

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
    state = state_manager.load()
    if not state:
        display.show_not_initialized()
        return

    level = level_registry.get(state.current_level)
    if not level:
        console.print(f"[red]Chyba: Level {state.current_level} nenalezen[/red]\n")
        return

    level.reset(state.workspace)
    display.wait_for_continue()
    display.show_instructions(level)
    console.print(f"[green]✓ Level {state.current_level} znovu načten.[/green]")


@dev.command(name="start")
def dev_start() -> None:
    levels = level_registry.list_levels()
    if not levels:
        console.print("[red]No levels registered[/red]")
        return

    first_id = levels[0].id
    ctx = click.get_current_context()
    ctx.invoke(dev_jump, level_id=first_id)


@cli.command(hidden=True, name="cd-hook")
@click.argument("arg1", required=False)
@click.argument("arg2", required=False)
@click.option("--post-move", is_flag=True)
def cd_hook(arg1: Optional[str], arg2: Optional[str], post_move: bool) -> None:
    if post_move:
        target = None
        pwd = arg1
    else:
        target = arg1
        pwd = arg2

    try:
        _get_session().handle_cd_hook(target=target, pwd=pwd, post_move=post_move)
    except SystemExit:
        raise
    except Exception:
        pass
