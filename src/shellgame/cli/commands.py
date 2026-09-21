"""CLI commands for ShellGame.

This module should be a thin Click parsing layer.
Core gameplay orchestration lives in `shellgame.core.session.GameSession`.

Shell override:
- You can force which subshell ShellGame launches with `--shell bash|fish`.
- Useful when auto-detection is wrong (nested shells / wrappers like `uv` / `make`).
"""

import os
import sys
from pathlib import Path
from typing import cast

import click
from rich.console import Console

from shellgame.cli.boot import boot_if_needed
from shellgame.core.services import GameServices
from shellgame.core.session import GameSession
from shellgame.levels.registry import LevelRegistry
from shellgame.messages import Messages
from shellgame.paths import current_directory
from shellgame.shell.client import ShellClient
from shellgame.state.manager import StateManager, StatePersistenceError
from shellgame.ui.display import Display


class _Services:
    instance: GameServices | None = None


def _game_services() -> GameServices:
    """Create process-wide services on first use, not at import.

    `StateManager` reads XDG paths in its constructor. Binding it at import
    time would freeze those paths before tests or playtesting can isolate HOME.
    """
    if _Services.instance is None:
        _Services.instance = GameServices()
    return _Services.instance


def __getattr__(name: str) -> object:
    if name in {"console", "display", "state_manager", "level_registry", "shell_client", "services"}:
        default = _game_services()
        if name == "services":
            return default
        return getattr(default, name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


def _module_attr(name: str) -> object:
    return getattr(sys.modules[__name__], name)


def _teleport_notice(destination: Path) -> None:
    cast(Console, _module_attr("console")).print(
        f"[yellow]Byli jste [bold]teleportováni[/bold] do adresáře [violet]{destination}[/violet][/yellow]\n"
    )


def _get_session() -> GameSession:
    return GameSession(
        console=cast(Console, _module_attr("console")),
        display=cast(Display, _module_attr("display")),
        state_manager=cast(StateManager, _module_attr("state_manager")),
        level_registry=cast(LevelRegistry, _module_attr("level_registry")),
        teleport_notice=_teleport_notice,
        shell_client=cast(ShellClient, _module_attr("shell_client")),
        workspace_factory=_game_services().get_workspace_manager_factory(),
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

    def invoke(self, ctx: click.Context) -> object:
        try:
            return super().invoke(ctx)
        except StatePersistenceError as exc:
            cast(Display, _module_attr("display")).show_state_error(str(exc))
            ctx.exit(1)
        except OSError as exc:
            message = Messages.FILESYSTEM_ERROR.format(error=exc)
            if current_directory() is None:
                message = f"{Messages.CWD_MISSING}\n\n{exc}"
            cast(Display, _module_attr("display")).show_state_error(message)
            ctx.exit(1)


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
def cli(ctx: click.Context, devmode: bool, forced_shell: str | None) -> None:
    """ShellGame - Interaktivní výuka navigace v terminálu."""
    ctx.ensure_object(dict)
    ctx.obj["devmode"] = devmode

    wrapped = bool(os.environ.get("SHELLGAME_WRAPPER"))

    if forced_shell:
        os.environ["SHELLGAME_FORCE_SHELL"] = forced_shell.lower()

    try:
        boot = boot_if_needed(wrapped=wrapped, devmode=devmode)
    except Exception as e:
        cast(Console, _module_attr("console")).print(
            f"[bold red]CHYBA: Nepodařilo se spustit herní shell ({e})[/bold red]"
        )
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


@cli.command(context_settings={"ignore_unknown_options": True})
@click.argument("answer", required=False)
@click.pass_context
def submit(ctx: click.Context, answer: str | None = None) -> None:
    """Odeslat odpověď pro aktuální level."""
    if answer is None and ctx.args:
        answer = " ".join(ctx.args)
    _get_session().submit(answer)


@cli.command()
def status() -> None:
    """Zobrazit postup a statistiky."""
    _get_session().status()


@cli.command()
def skip() -> None:
    """Přeskočit aktuální level (pouze nepovinné levely)."""
    _get_session().skip()


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
def repeat(section_num: int | None, level_id: str | None) -> None:
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
    _get_session().show(section=section, level=level)


@cli.command()
@click.confirmation_option(prompt="Opravdu chcete odstranit všechna data ShellGame?")
def remove() -> None:
    """Smazat stav a pracovní prostor."""
    _get_session().remove()


@cli.command()
def exit() -> None:
    """Ukončit ShellGame."""
    _get_session().exit()


@cli.group(hidden=True)
def dev() -> None:
    pass


@dev.command(name="jump")
@click.argument("level_id")
def dev_jump(level_id: str) -> None:
    _get_session().dev_jump_to(level_id)


@dev.command(name="next")
def dev_next() -> None:
    _get_session().dev_next()


@dev.command(name="prev")
def dev_prev() -> None:
    _get_session().dev_previous()


@dev.command(name="reload")
def dev_reload() -> None:
    _get_session().dev_reload()


@dev.command(name="start")
def dev_start() -> None:
    _get_session().dev_start()


@cli.command(hidden=True, name="cd-hook")
@click.argument("arg1", required=False)
@click.argument("arg2", required=False)
@click.option("--post-move", is_flag=True)
def cd_hook(arg1: str | None, arg2: str | None, post_move: bool) -> None:
    if post_move:
        target = None
        pwd = arg1
    else:
        target = arg1
        pwd = arg2

    _get_session().handle_cd_hook(target=target, pwd=pwd, post_move=post_move)


@cli.command(hidden=True, name="fd-hook")
def fd_hook() -> None:
    _get_session().handle_fd_hook()
