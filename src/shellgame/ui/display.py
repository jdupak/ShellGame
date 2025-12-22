"""UI display and formatting using Rich."""

from __future__ import annotations

from typing import Any, cast
from pathlib import Path

from rich.align import Align
from rich.console import Console, ConsoleOptions, RenderResult
from rich.markdown import Markdown
from rich.panel import Panel
from rich.rule import Rule
from rich.table import Table

Heading = cast(Any, Markdown.elements["heading_open"])  # type: ignore[valid-type]


class LeftHeading(Heading):  # type: ignore[misc, valid-type]
    def __rich_console__(self, console: Console, options: ConsoleOptions) -> RenderResult:
        text = self.text
        text.justify = "left"

        level = 1
        try:
            level = int(self.tag[1:])
        except Exception:
            level = 1

        if level == 1:
            yield Rule(text, style="markdown.h1", align="left")
        elif level == 2:
            yield Rule(text, style="markdown.h2", align="left")
        else:
            text.style = f"markdown.h{level}"
            yield text


class LeftMarkdown(Markdown):
    elements = Markdown.elements.copy()
    elements["heading_open"] = LeftHeading


class Display:
    def __init__(self, console: Console):
        self.console = console

    def wait_for_continue(self) -> None:
        self.console.input(" [dim]Stiskněte Enter pro pokračování...[/dim]")

    def show_instructions(self, level: Any) -> None:
        if level.id.endswith(".0"):
            title_str = f"[bold cyan]{level.title}[/bold cyan]"

            raw = level.instructions or ""
            pages = [p.strip() for p in raw.split("\n---\n") if p.strip()]

            if not pages:
                pages = [raw.strip()]

            total_pages = len(pages)
            for idx, page in enumerate(pages, start=1):
                self.console.clear()

                md = LeftMarkdown(page, justify="left")
                page_title = title_str
                if total_pages > 1:
                    page_title = f"{title_str} [dim](strana {idx}/{total_pages})[/dim]"

                panel = Panel(md, title=page_title, border_style="cyan", padding=(1, 2))
                self.console.print(panel)

                if idx < total_pages:
                    self.wait_for_continue()
            return

        self.console.clear()
        md = LeftMarkdown(level.instructions, justify="left")
        title_str = f"[bold cyan]{level.id}: {level.title}[/bold cyan]"
        panel = Panel(md, title=title_str, border_style="cyan", padding=(1, 2))
        self.console.print(panel)

    def note(self, text: str, indent: str = " ") -> None:
        for line in text.splitlines():
            self.console.print(indent + line if line else indent.rstrip())

    _HINT_REPEAT_TIP = (
        "[dim]Tip: Již zobrazené nápovědy si můžete vypsat znovu příkazem: "
        "[violet]shellgame hint --repeat[/violet][/dim]"
    )
    _HINT_NEXT_HELP = "[dim]Potřebujete další pomoc? Napište: shellgame hint[/dim]"
    _HINT_ALL_SHOWN = ""

    def _hint_footer(
        self,
        hint_num: int,
        total_hints: int,
        *,
        show_repeat_tip: bool = True,
        show_next_help: bool = True,
    ) -> None:
        if show_repeat_tip:
            self.note(self._HINT_REPEAT_TIP)

        if hint_num < total_hints - 1:
            if show_next_help:
                self.note(self._HINT_NEXT_HELP)
                self.console.print()
            else:
                self.console.print()
        else:
            self.console.print()

    def show_hint(
        self,
        hint_text: str,
        hint_num: int,
        total_hints: int,
        *,
        show_repeat_tip: bool = True,
        show_next_help: bool = True,
    ) -> None:
        panel = Panel(
            hint_text,
            title=f"[yellow]💡 Nápověda {hint_num + 1} z {total_hints}[/yellow]",
            border_style="yellow",
            padding=(1, 2),
        )
        self.console.print(panel)
        self._hint_footer(
            hint_num,
            total_hints,
            show_repeat_tip=show_repeat_tip,
            show_next_help=show_next_help,
        )

    def show_no_more_hints(self) -> None:
        panel = Panel(
            "[yellow]Pro tento level již nejsou k dispozici žádné další nápovědy.[/yellow]",
            title="[yellow]💡 Nápověda[/yellow]",
            border_style="yellow",
            padding=(1, 2),
        )
        self.console.print(panel)
        self.note(self._HINT_REPEAT_TIP)
        self.console.print()

    def show_success(self, message: str, time_sec: int = 0, hints_used: int = 0) -> None:
        panel = Panel(
            Align.center(f"[bold green]{message}[/bold green]", vertical="middle"),
            border_style="green",
            padding=(1, 4),
        )
        self.console.print(panel)

    def show_failure(self, message: str, suggestion: str = "") -> None:
        body = f"[bold red]{message}[/bold red]"
        if suggestion:
            body += f"\n\n[dim]{suggestion}[/dim]"

        panel = Panel(
            Align.center(body, vertical="middle"),
            border_style="red",
            padding=(1, 4),
        )
        self.console.print(panel)

        self.note("[dim]Zkuste to znovu nebo použijte: shellgame hint[/dim]")
        self.console.print()

    def show_status(self, state: Any) -> None:
        self.console.print("\n[bold cyan]═══ ShellGame Postup ═══[/bold cyan]\n")

        self.console.print(f"[bold]Aktuální Level:[/bold] {state.current_level}")
        self.console.print(f"[bold]Zahájeno:[/bold] {state.start_time.strftime('%Y-%m-%d %H:%M')}\n")

        if state.levels_complete:
            table = Table(title="Dokončené Levely")
            table.add_column("Level", style="cyan")
            table.add_column("Čas", justify="right")
            table.add_column("Nápovědy", justify="right")
            table.add_column("Pokusy", justify="right")

            for level_id, completion in sorted(state.levels_complete.items()):
                table.add_row(
                    level_id,
                    f"{completion.time_sec}s",
                    str(completion.hints),
                    str(completion.attempts),
                )

            self.console.print(table)

            total_time = sum(c.time_sec for c in state.levels_complete.values())
            total_hints = sum(c.hints for c in state.levels_complete.values())
            total_attempts = sum(c.attempts for c in state.levels_complete.values())
            avg_hints = total_hints / len(state.levels_complete) if state.levels_complete else 0

            self.console.print("\n[bold]Statistiky:[/bold]")
            self.console.print(f"  Celkový čas: {self._format_duration(total_time)}")
            self.console.print(f"  Dokončené levely: {len(state.levels_complete)}")
            self.console.print(f"  Průměrně nápověd: {avg_hints:.1f}")
            self.console.print(f"  Celkem pokusů: {total_attempts}\n")
        else:
            self.console.print("[dim]Zatím žádné dokončené levely.[/dim]\n")

    def show_init_success(self, username: str, workspace: str) -> None:
        self.console.print(f"\n[green]✓ ShellGame inicializována pro {username}[/green]")
        self.console.print(f"[dim]Pracovní prostor: {workspace}[/dim]\n")

        self.console.print("[bold]Automaticky vás přesměrovávám do pracovního prostoru...[/bold]\n")

    def show_already_initialized(self) -> None:
        self.console.print("[yellow]Již inicializováno! Spusťte 'shellgame' pro pokračování.[/yellow]\n")

    def show_not_initialized(self) -> None:
        self.console.print("[yellow]Neinicializováno. Spusťte: shellgame init[/yellow]\n")

    def show_removed(self) -> None:
        self.console.print("[green]Stav ShellGame a pracovní prostor odstraněny.[/green]\n")

    def show_reset(self, level_id: str) -> None:
        self.console.print(f"[green]✓ Level {level_id} byl resetován.[/green]\n")

    def show_current_directory(self, path: str | Path) -> None:
        self.console.print(f"[dim]Aktuální adresář: {path}[/dim]")

    def _format_duration(self, seconds: int) -> str:
        if seconds < 60:
            return f"{seconds}s"

        minutes = seconds // 60
        remaining_seconds = seconds % 60

        if minutes < 60:
            return f"{minutes}m {remaining_seconds}s"

        hours = minutes // 60
        remaining_minutes = minutes % 60
        return f"{hours}h {remaining_minutes}m {remaining_seconds}s"

    def show_repeated_hints(self, hints: list[str], revealed_count: int) -> None:
        total = len(hints)
        count = min(revealed_count, total)

        if count == 0:
            self.show_hint(
                hints[0],
                0,
                total,
                show_repeat_tip=False,
                show_next_help=False,
            )
            self.note(self._HINT_REPEAT_TIP)
            if total > 1:
                self.note(self._HINT_NEXT_HELP)
            self.console.print()
            return

        for idx in range(count):
            self.show_hint(
                hints[idx],
                idx,
                total,
                show_repeat_tip=False,
                show_next_help=False,
            )

        self.note(self._HINT_REPEAT_TIP)

        if count < total:
            self.note(self._HINT_NEXT_HELP)
        self.console.print()

    def show_workspace_restored(self, workspace: str | Path) -> None:
        self.console.print(
            "[yellow]⚠ Pracovní prostor byl smazán (např. restart systému). Obnovuji...[/yellow]"
        )
        self.console.print(f"[green]✓ Pracovní prostor obnoven: {workspace}[/green]\n")
