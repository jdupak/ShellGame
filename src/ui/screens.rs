//! The screens of the game: every message the player sees, as lines of text.
//!
//! Each function builds one screen from plain data and a [`Term`] and returns it as `Vec<String>`
//! (one entry per output line, trailing blank lines included, exactly as the Python `Display`
//! printed them). Nothing is written here: the session prints the lines with
//! [`print_lines`](super::print_lines), and tests compare them with [`Term::plain`].
//!
//! The Czech texts are exact ports of `display.py`, `session.py` and `commands.py`, minus the Rich
//! markup. Hints, answers' feedback, success messages, user names and paths are printed
//! **verbatim**: they are wrapped as plain text, never interpreted as Markdown or markup, so
//! `[ab]*.txt` and `` `ls` `` come out exactly as written. Only the level assignment itself
//! (`Level::text`) goes through the Markdown renderer.
//!
//! The palette follows the Python UI: cyan level panels, yellow hints, green success, red
//! failures, dim notes with violet commands.

use anstyle::Style;

use super::panel::Panel;
use super::text::{
    Frag, Inline, center, layout, pad_start, pad_to, render_frags, sanitize, tokenize,
    visible_width, wrap_plain,
};
use super::{Term, style};
use crate::engine::LevelId;
use crate::engine::level::Level;
use crate::messages;
use crate::state::{LevelCompletion, State};

/// The `shellgame` commands mentioned in notes, shown in violet.
const CMD_HINT: &str = "shellgame hint";
const CMD_REPEAT_HINTS: &str = "shellgame hint --repeat";

// ---------------------------------------------------------------------------------------------
// Building blocks
// ---------------------------------------------------------------------------------------------

/// A run of styled text inside a flowing line.
enum Run<'a> {
    /// Text that may break at its spaces.
    Words(Style, &'a str),
    /// Text kept in one piece when it fits on a line of its own (a path, a command).
    Whole(Style, &'a str),
}

/// Lays runs out on lines of at most `width` columns. Every line is self-contained. A newline in
/// a run starts a new line, so error details that span several lines keep their shape.
fn flow(term: &Term, runs: &[Run<'_>], width: usize) -> Vec<String> {
    let mut inlines: Vec<Inline> = Vec::new();
    for run in runs {
        let (style, text, whole) = match run {
            Run::Words(style, text) => (*style, *text, false),
            Run::Whole(style, text) => (*style, *text, true),
        };
        for (index, piece) in text.split('\n').enumerate() {
            if index > 0 {
                inlines.push(Inline::Hard);
            }
            let frag = Frag::new(piece, style);
            inlines.push(if whole {
                Inline::Unbreakable(frag)
            } else {
                Inline::Text(frag)
            });
        }
    }
    layout(&tokenize(&inlines), width)
        .iter()
        .map(|row| render_frags(row, term.color))
        .collect()
}

/// `console.print(text)`: one paragraph, wrapped to the terminal.
fn paragraph(term: &Term, runs: &[Run<'_>]) -> Vec<String> {
    flow(term, runs, term.width)
}

/// `Display.note`: a paragraph indented by one space.
fn note(term: &Term, runs: &[Run<'_>]) -> Vec<String> {
    flow(term, runs, term.width.saturating_sub(1))
        .into_iter()
        .map(|row| format!(" {row}"))
        .collect()
}

/// `paragraph` followed by the blank line a trailing `\n` in the Python text produced.
fn paragraph_then_blank(term: &Term, runs: &[Run<'_>]) -> Vec<String> {
    let mut lines = paragraph(term, runs);
    lines.push(String::new());
    lines
}

/// A panel of centred, wrapped paragraphs, one style per paragraph, separated by blank lines:
/// the shape of the success, failure, error and "game complete" panels (`Align.center` with
/// padding `(1, 4)`).
fn message_panel(
    border: Style,
    title: Option<(&str, Style)>,
    parts: &[(Style, &str)],
    term: &Term,
) -> Vec<String> {
    let mut panel = Panel::new(border).padding(1, 4).centered();
    if let Some((text, style)) = title {
        panel = panel.title(text).title_style(style);
    }
    let width = panel.body_width(term);
    let mut body = Vec::new();
    for (index, (style, text)) in parts.iter().enumerate() {
        if index > 0 {
            body.push(String::new());
        }
        body.extend(term.paint_lines(*style, &wrap_plain(text, width)));
    }
    panel.render(&body, term)
}

/// The yellow hint panel: the hint text verbatim, titled `💡 <title>`.
fn yellow_panel(title: &str, text: &str, term: &Term) -> Vec<String> {
    Panel::new(style::YELLOW)
        .title(format!("💡 {title}"))
        .title_style(style::YELLOW)
        .render_text(text, style::PLAIN, term)
}

// ---------------------------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------------------------

/// `37s`, `2m 5s`, `1h 2m 3s`: a duration the way the Python UI wrote it.
///
/// ```
/// use shellgame::ui::screens::format_duration;
///
/// assert_eq!(format_duration(59), "59s");
/// assert_eq!(format_duration(125), "2m 5s");
/// assert_eq!(format_duration(3723), "1h 2m 3s");
/// ```
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        return format!("{seconds}s");
    }
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;
    if minutes < 60 {
        return format!("{minutes}m {remaining_seconds}s");
    }
    format!("{}h {}m {remaining_seconds}s", minutes / 60, minutes % 60)
}

/// The statistics line of the success panel: `Čas: 12s  |  Nápovědy: 1  |  Chybné pokusy: 2`.
/// Failed attempts are mentioned only when there were any.
///
/// ```
/// use chrono::NaiveDate;
/// use shellgame::state::LevelCompletion;
/// use shellgame::ui::screens::format_stats;
///
/// let completed_at = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap().and_hms_opt(3, 4, 5).unwrap();
/// let clean = LevelCompletion { time_sec: 12, hints: 0, attempts: 0, completed_at };
/// assert_eq!(format_stats(&clean), "Čas: 12s  |  Nápovědy: 0");
/// let messy = LevelCompletion { attempts: 2, ..clean };
/// assert_eq!(format_stats(&messy), "Čas: 12s  |  Nápovědy: 0  |  Chybné pokusy: 2");
/// ```
pub fn format_stats(completion: &LevelCompletion) -> String {
    let mut parts = vec![
        format!("Čas: {}", format_duration(completion.time_sec)),
        format!("Nápovědy: {}", completion.hints),
    ];
    if completion.attempts > 0 {
        parts.push(format!("Chybné pokusy: {}", completion.attempts));
    }
    parts.join("  |  ")
}

/// Sort key that orders level ids numerically (`1.2` before `1.10`) and puts anything that is not
/// an id last, by text.
fn level_sort_key(id: &str) -> (bool, u16, u16, String) {
    match id.parse::<LevelId>() {
        Ok(parsed) => (false, parsed.section, parsed.number, String::new()),
        Err(_) => (true, 0, 0, id.to_owned()),
    }
}

// ---------------------------------------------------------------------------------------------
// The level
// ---------------------------------------------------------------------------------------------

/// The assignment of a regular level: a cyan panel titled `<id>: <title>` holding the rendered
/// Markdown, followed (for a bonus level) by the note that it may be skipped.
pub fn level_panel(level: &Level, term: &Term) -> Vec<String> {
    let panel = Panel::new(style::CYAN)
        .title(level.display_title())
        .title_style(style::BOLD_CYAN);
    let mut lines = panel.render_markdown(level.text.trim(), term);
    if level.is_bonus() {
        lines.extend(bonus_note(term));
        lines.push(String::new());
    }
    lines
}

/// One page of a section intro: a cyan panel titled with the intro's title, and `(strana i/N)`
/// when the intro has several pages. `number` counts from 1.
pub fn intro_page(
    level: &Level,
    page: &str,
    number: usize,
    total: usize,
    term: &Term,
) -> Vec<String> {
    let mut title = term.paint(style::BOLD_CYAN, &level.title);
    if total > 1 {
        title.push(' ');
        title.push_str(&term.paint(style::DIM, &format!("(strana {number}/{total})")));
    }
    Panel::new(style::CYAN)
        .title(title)
        .render_markdown(page.trim(), term)
}

/// The note under the panel of a bonus level.
pub fn bonus_note(term: &Term) -> Vec<String> {
    note(
        term,
        &[
            Run::Words(style::DIM, "Tento level je "),
            Run::Words(style::DIM_BOLD, "nepovinný"),
            Run::Words(style::DIM, ". Můžete ho přeskočit příkazem: "),
            Run::Whole(style::DIM_VIOLET, "shellgame skip"),
        ],
    )
}

// ---------------------------------------------------------------------------------------------
// Hints
// ---------------------------------------------------------------------------------------------

/// `Tip: Již zobrazené nápovědy si můžete vypsat znovu příkazem: shellgame hint --repeat`
fn repeat_tip(term: &Term) -> Vec<String> {
    note(
        term,
        &[
            Run::Words(
                style::DIM,
                "Tip: Již zobrazené nápovědy si můžete vypsat znovu příkazem: ",
            ),
            Run::Whole(style::DIM_VIOLET, CMD_REPEAT_HINTS),
        ],
    )
}

/// `Potřebujete další pomoc? Napište: shellgame hint`
fn next_help(term: &Term) -> Vec<String> {
    note(
        term,
        &[Run::Words(
            style::DIM,
            "Potřebujete další pomoc? Napište: shellgame hint",
        )],
    )
}

/// The yellow panel of one hint, without anything below it.
fn hint_panel(text: &str, index: usize, total: usize, term: &Term) -> Vec<String> {
    yellow_panel(&format!("Nápověda {} z {total}", index + 1), text, term)
}

/// A newly revealed hint: its panel, the tip about `--repeat`, the pointer to the next hint (when
/// there is one) and a blank line. `index` counts from 0.
pub fn hint(text: &str, index: usize, total: usize, term: &Term) -> Vec<String> {
    let mut lines = hint_panel(text, index, total, term);
    lines.extend(repeat_tip(term));
    if index + 1 < total {
        lines.extend(next_help(term));
    }
    lines.push(String::new());
    lines
}

/// Every hint revealed so far (`hint --repeat`): the first `revealed` of `hints`, then the tips.
/// Nothing revealed yet has its own message.
pub fn repeated_hints(hints: &[String], revealed: usize, term: &Term) -> Vec<String> {
    let total = hints.len();
    let count = revealed.min(total);
    if count == 0 {
        return no_hints_revealed(term);
    }
    let mut lines = Vec::new();
    for (index, text) in hints.iter().take(count).enumerate() {
        lines.extend(hint_panel(text, index, total, term));
        lines.push(String::new());
    }
    lines.extend(repeat_tip(term));
    if count < total {
        lines.extend(next_help(term));
    }
    lines.push(String::new());
    lines
}

/// All hints have been shown already.
pub fn no_more_hints(term: &Term) -> Vec<String> {
    let mut lines = yellow_panel(
        "Nápověda",
        "Pro tento level již nejsou k dispozici žádné další nápovědy.",
        term,
    );
    lines.extend(repeat_tip(term));
    lines.push(String::new());
    lines
}

/// The level has no hints at all.
pub fn no_hints_available(term: &Term) -> Vec<String> {
    let mut lines = yellow_panel(
        "Nápověda",
        "Pro tento level nejsou k dispozici žádné nápovědy.",
        term,
    );
    lines.push(String::new());
    lines
}

/// `hint --repeat` before any hint was shown.
pub fn no_hints_revealed(term: &Term) -> Vec<String> {
    let mut lines = yellow_panel(
        "Nápověda",
        "Zatím jste si nezobrazili žádnou nápovědu.",
        term,
    );
    lines.extend(note(
        term,
        &[
            Run::Words(style::DIM, "První nápovědu zobrazíte příkazem: "),
            Run::Whole(style::DIM_VIOLET, CMD_HINT),
        ],
    ));
    lines.push(String::new());
    lines
}

// ---------------------------------------------------------------------------------------------
// Success and failure
// ---------------------------------------------------------------------------------------------

/// The green panel shown when a level is completed: the level's own message and, with a
/// completion record, the line of statistics.
pub fn success(message: &str, completion: Option<&LevelCompletion>, term: &Term) -> Vec<String> {
    let stats = completion.map(format_stats);
    let mut parts = vec![(style::BOLD_GREEN, message)];
    if let Some(stats) = &stats {
        parts.push((style::DIM, stats));
    }
    message_panel(style::GREEN, None, &parts, term)
}

/// The red panel for a wrong answer or an unmet requirement, and the pointer to `hint`.
pub fn failure(message: &str, term: &Term) -> Vec<String> {
    let mut lines = message_panel(style::RED, None, &[(style::BOLD_RED, message)], term);
    lines.extend(note(
        term,
        &[Run::Words(
            style::DIM,
            "Zkuste to znovu nebo použijte: shellgame hint",
        )],
    ));
    lines.push(String::new());
    lines
}

/// The red panel for a `cd` the level forbids, printed on stderr by the `cd` hook: titled
/// `ShellGame (<level id>)`, with the recovery tip below it.
pub fn cd_block(level_id: LevelId, message: &str, term: &Term) -> Vec<String> {
    let title = format!("ShellGame ({level_id})");
    let mut lines = message_panel(
        style::RED,
        Some((&title, style::BOLD_RED)),
        &[(style::BOLD_RED, message)],
        term,
    );
    lines.extend(note(
        term,
        &[Run::Words(style::DIM, messages::CD_RECOVERY_TIP)],
    ));
    lines.push(String::new());
    lines
}

/// The red panel for a failure of the game itself (unreadable state, a filesystem error).
pub fn state_error(message: &str, term: &Term) -> Vec<String> {
    message_panel(style::RED, None, &[(style::BOLD_RED, message)], term)
}

/// The next level could not be prepared; the player's progress is kept.
pub fn level_setup_error(level_id: &str, error: &str, term: &Term) -> Vec<String> {
    let headline = format!("Level {level_id} se nepodařilo připravit.");
    let mut lines = message_panel(
        style::RED,
        None,
        &[(style::BOLD_RED, &headline), (style::DIM, error)],
        term,
    );
    lines.extend(note(
        term,
        &[
            Run::Words(style::DIM, "Zkuste: "),
            Run::Whole(style::DIM_VIOLET, "shellgame reset"),
        ],
    ));
    lines.push(String::new());
    lines
}

/// The saved level no longer exists in this version of the game.
pub fn level_missing(stale_id: &str, resolved_id: &str, term: &Term) -> Vec<String> {
    let headline = format!("Level {stale_id} už v této verzi hry neexistuje.");
    let detail = format!("Pokračujeme nejbližším dostupným levelem: {resolved_id}");
    let mut lines = message_panel(
        style::YELLOW,
        None,
        &[(style::BOLD_YELLOW, &headline), (style::DIM, &detail)],
        term,
    );
    lines.extend(note(
        term,
        &[
            Run::Words(style::DIM, "Váš dosavadní postup zůstává zachován: "),
            Run::Whole(style::DIM_VIOLET, "shellgame status"),
        ],
    ));
    lines.push(String::new());
    lines
}

/// The game has no levels at all.
pub fn no_levels_available(term: &Term) -> Vec<String> {
    let mut lines = message_panel(
        style::RED,
        None,
        &[(style::BOLD_RED, "Nejsou registrovány žádné levely.")],
        term,
    );
    lines.push(String::new());
    lines
}

/// The player finished the last level.
pub fn game_complete(state: &State, term: &Term) -> Vec<String> {
    let total_time: u64 = state.levels_complete.values().map(|c| c.time_sec).sum();
    let summary = format!(
        "Dokončené levely: {}  |  Celkový čas: {}",
        state.levels_complete.len(),
        format_duration(total_time)
    );
    let mut lines = message_panel(
        style::GREEN,
        None,
        &[
            (style::BOLD_GREEN, "Dokončili jste ShellGame!"),
            (style::DIM, &summary),
        ],
        term,
    );
    lines.extend(note(
        term,
        &[
            Run::Words(style::DIM, "Statistiky: "),
            Run::Whole(style::DIM_VIOLET, "shellgame status"),
        ],
    ));
    lines.extend(note(
        term,
        &[
            Run::Words(style::DIM, "Ukončit hru: "),
            Run::Whole(style::DIM_VIOLET, "shellgame exit"),
        ],
    ));
    lines.extend(note(
        term,
        &[
            Run::Words(style::DIM, "Zahrát znovu od začátku: "),
            Run::Whole(style::DIM_VIOLET, "shellgame remove"),
            Run::Words(style::DIM, " a pak "),
            Run::Whole(style::DIM_VIOLET, "shellgame"),
        ],
    ));
    lines.push(String::new());
    lines
}

// ---------------------------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------------------------

/// One column of the status table.
struct Column {
    header: &'static str,
    right: bool,
    style: Style,
}

/// A table in the style of Rich's default `HEAVY_HEAD` box, as wide as its content: a centred
/// italic title, a heavy header, light rows.
fn table(title: &str, columns: &[Column], rows: &[Vec<String>], term: &Term) -> Vec<String> {
    let widths: Vec<usize> = columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            rows.iter()
                .map(|row| visible_width(&row[index]))
                .fold(visible_width(column.header), usize::max)
        })
        .collect();
    let rule = |left: &str, fill: &str, middle: &str, right: &str| {
        let cells: Vec<String> = widths.iter().map(|w| fill.repeat(w + 2)).collect();
        format!("{left}{}{right}", cells.join(middle))
    };
    let row_line = |edge: &str, cells: Vec<String>| {
        let aligned: Vec<String> = cells
            .iter()
            .zip(columns.iter().zip(&widths))
            .map(|(cell, (column, width))| {
                let padded = if column.right {
                    pad_start(cell, *width)
                } else {
                    pad_to(cell, *width)
                };
                format!(" {padded} ")
            })
            .collect();
        format!("{edge}{}{edge}", aligned.join(edge))
    };

    let total_width: usize = widths.iter().map(|w| w + 3).sum::<usize>() + 1;
    let mut lines = vec![
        center(&term.paint(style::EMPHASIS, title), total_width)
            .trim_end()
            .to_owned(),
        rule("┏", "━", "┳", "┓"),
    ];
    let header: Vec<String> = columns
        .iter()
        .map(|column| term.paint(style::BOLD, column.header))
        .collect();
    lines.push(row_line("┃", header));
    lines.push(rule("┡", "━", "╇", "┩"));
    for row in rows {
        let cells = row
            .iter()
            .zip(columns)
            .map(|(cell, column)| term.paint(column.style, cell))
            .collect();
        lines.push(row_line("│", cells));
    }
    lines.push(rule("└", "─", "┴", "┘"));
    lines
}

/// The progress screen: the current level, when the game started, a table of finished levels
/// (numerically sorted, so `1.10` follows `1.2`) and totals.
pub fn status(state: &State, term: &Term) -> Vec<String> {
    let mut lines = vec![
        String::new(),
        term.paint(style::BOLD_CYAN, "═══ ShellGame Postup ═══"),
        String::new(),
        format!(
            "{} {}",
            term.paint(style::BOLD, "Aktuální Level:"),
            sanitize(&state.current_level)
        ),
        format!(
            "{} {}",
            term.paint(style::BOLD, "Zahájeno:"),
            state.start_time.format("%Y-%m-%d %H:%M")
        ),
        String::new(),
    ];
    let completed = &state.levels_complete;
    if completed.is_empty() {
        lines.push(term.paint(style::DIM, "Zatím žádné dokončené levely."));
        lines.push(String::new());
        return lines;
    }

    let mut entries: Vec<(&String, &LevelCompletion)> = completed.iter().collect();
    entries.sort_by_key(|(id, _)| level_sort_key(id));
    let rows: Vec<Vec<String>> = entries
        .iter()
        .map(|(id, completion)| {
            vec![
                sanitize(id).into_owned(),
                format_duration(completion.time_sec),
                completion.hints.to_string(),
                completion.attempts.to_string(),
            ]
        })
        .collect();
    let columns = [
        Column {
            header: "Level",
            right: false,
            style: style::CYAN,
        },
        Column {
            header: "Čas",
            right: true,
            style: style::PLAIN,
        },
        Column {
            header: "Nápovědy",
            right: true,
            style: style::PLAIN,
        },
        Column {
            header: "Chybné pokusy",
            right: true,
            style: style::PLAIN,
        },
    ];
    lines.extend(table("Dokončené Levely", &columns, &rows, term));

    let total_time: u64 = completed.values().map(|c| c.time_sec).sum();
    let total_hints: u64 = completed.values().map(|c| u64::from(c.hints)).sum();
    let total_attempts: u64 = completed.values().map(|c| u64::from(c.attempts)).sum();
    // The counts are tiny, so the conversions to `f64` are exact.
    #[allow(clippy::cast_precision_loss)]
    let average_hints = total_hints as f64 / completed.len() as f64;

    lines.push(String::new());
    lines.push(term.paint(style::BOLD, "Statistiky:"));
    lines.push(format!("  Celkový čas: {}", format_duration(total_time)));
    lines.push(format!("  Dokončené levely: {}", completed.len()));
    lines.push(format!("  Průměrně nápověd: {average_hints:.1}"));
    lines.push(format!("  Celkem chybných pokusů: {total_attempts}"));
    lines.push(String::new());
    lines
}

// ---------------------------------------------------------------------------------------------
// Starting, resetting, removing
// ---------------------------------------------------------------------------------------------

/// A new game was created.
pub fn init_success(username: &str, workspace: &str, term: &Term) -> Vec<String> {
    let username = sanitize(username);
    let workspace = sanitize(workspace);
    let mut lines = vec![String::new()];
    lines.extend(paragraph(
        term,
        &[
            Run::Words(style::GREEN, "✓ ShellGame inicializována pro "),
            Run::Whole(style::GREEN, &username),
        ],
    ));
    lines.extend(paragraph_then_blank(
        term,
        &[
            Run::Words(style::DIM, "Pracovní prostor: "),
            Run::Whole(style::DIM, &workspace),
        ],
    ));
    lines.extend(paragraph_then_blank(
        term,
        &[Run::Words(
            style::BOLD,
            "Automaticky vás přesměrovávám do pracovního prostoru...",
        )],
    ));
    lines
}

/// `init` on a game that exists already.
pub fn already_initialized(term: &Term) -> Vec<String> {
    paragraph_then_blank(
        term,
        &[Run::Words(
            style::YELLOW,
            "Již inicializováno! Spusťte 'shellgame' pro pokračování.",
        )],
    )
}

/// There is no saved game.
pub fn not_initialized(term: &Term) -> Vec<String> {
    paragraph_then_blank(
        term,
        &[Run::Words(
            style::YELLOW,
            "Neinicializováno. Spusťte: shellgame init",
        )],
    )
}

/// The state and the workspace were deleted.
pub fn removed(term: &Term) -> Vec<String> {
    paragraph(
        term,
        &[Run::Words(
            style::GREEN,
            "Stav ShellGame a pracovní prostor odstraněny.",
        )],
    )
}

/// The player declined the confirmation of `remove`.
pub fn removal_declined(term: &Term) -> Vec<String> {
    paragraph(
        term,
        &[Run::Words(
            style::YELLOW,
            "Odstranění zrušeno. Žádná data nebyla smazána.",
        )],
    )
}

/// `remove` refuses to delete a workspace outside the temporary directories.
pub fn removal_refused(workspace: &str, term: &Term) -> Vec<String> {
    let workspace = sanitize(workspace);
    paragraph_then_blank(
        term,
        &[
            Run::Words(
                style::RED,
                "Chyba: Odmítám odstranit pracovní prostor mimo dočasný adresář: ",
            ),
            Run::Whole(style::RED, &workspace),
        ],
    )
}

/// The level was rebuilt by `reset`.
pub fn reset_notice(level_id: &str, term: &Term) -> Vec<String> {
    let level_id = sanitize(level_id);
    paragraph_then_blank(
        term,
        &[
            Run::Words(style::GREEN, "✓ Level "),
            Run::Whole(style::GREEN, &level_id),
            Run::Words(style::GREEN, " byl resetován."),
        ],
    )
}

/// The directory the player starts in.
pub fn current_directory(path: &str, term: &Term) -> Vec<String> {
    let path = sanitize(path);
    paragraph(
        term,
        &[
            Run::Words(style::DIM, "Aktuální adresář: "),
            Run::Whole(style::DIM, &path),
        ],
    )
}

/// The workspace was gone (a cleared `/tmp`) and has been rebuilt.
pub fn workspace_restored(workspace: &str, term: &Term) -> Vec<String> {
    let workspace = sanitize(workspace);
    let mut lines = paragraph(
        term,
        &[Run::Words(
            style::YELLOW,
            "⚠ Pracovní prostor byl smazán (např. restart systému). Obnovuji...",
        )],
    );
    lines.extend(paragraph_then_blank(
        term,
        &[
            Run::Words(style::GREEN, "✓ Pracovní prostor obnoven: "),
            Run::Whole(style::GREEN, &workspace),
        ],
    ));
    lines
}

/// The player was moved to the start directory of the level.
pub fn teleport_notice(destination: &str, term: &Term) -> Vec<String> {
    let destination = sanitize(destination);
    paragraph_then_blank(
        term,
        &[
            Run::Words(style::YELLOW, "Byli jste "),
            Run::Words(style::BOLD_YELLOW, "teleportováni"),
            Run::Words(style::YELLOW, " do adresáře "),
            Run::Whole(style::VIOLET, &destination),
        ],
    )
}

/// A bonus level was skipped.
pub fn skipped(level_id: &str, term: &Term) -> Vec<String> {
    let level_id = sanitize(level_id);
    paragraph_then_blank(
        term,
        &[
            Run::Words(style::YELLOW, "Level "),
            Run::Whole(style::YELLOW, &level_id),
            Run::Words(style::YELLOW, " přeskočen (byl nepovinný)."),
        ],
    )
}

/// `skip` on a required level.
pub fn not_skippable(level_id: &str, term: &Term) -> Vec<String> {
    let level_id = sanitize(level_id);
    let mut lines = paragraph(
        term,
        &[
            Run::Words(style::YELLOW, "Level "),
            Run::Whole(style::YELLOW, &level_id),
            Run::Words(style::YELLOW, " je povinný a nelze ho přeskočit."),
        ],
    );
    lines.extend(note(
        term,
        &[
            Run::Words(style::DIM, "Zkuste nápovědu: "),
            Run::Whole(style::DIM_VIOLET, CMD_HINT),
        ],
    ));
    lines.push(String::new());
    lines
}

// ---------------------------------------------------------------------------------------------
// One-line messages of the session
// ---------------------------------------------------------------------------------------------

/// The catalog has no level with this id.
pub fn level_not_found(id: &str, term: &Term) -> Vec<String> {
    let id = sanitize(id);
    paragraph_then_blank(
        term,
        &[
            Run::Words(style::RED, "Chyba: Level "),
            Run::Whole(style::RED, &id),
            Run::Words(style::RED, " nenalezen"),
        ],
    )
}

/// `show` without exactly one of `--level` and `--section`.
pub fn show_usage(term: &Term) -> Vec<String> {
    paragraph_then_blank(
        term,
        &[Run::Words(
            style::YELLOW,
            "Použití: shellgame show --level  nebo  shellgame show --section",
        )],
    )
}

/// The saved level id is not of the form `S.N`, so its section cannot be found.
pub fn invalid_level_format(id: &str, term: &Term) -> Vec<String> {
    let id = sanitize(id);
    paragraph_then_blank(
        term,
        &[
            Run::Words(style::RED, "Chyba: Neplatný formát aktuálního levelu: "),
            Run::Whole(style::RED, &id),
        ],
    )
}

/// `reset` could not rebuild the level.
pub fn reset_failed(error: &str, term: &Term) -> Vec<String> {
    let error = sanitize(error);
    paragraph_then_blank(
        term,
        &[Run::Words(
            style::RED,
            &format!("Chyba při obnově levelu: {error}"),
        )],
    )
}

/// `dev jump` to an id that does not exist.
pub fn unknown_level(id: &str, term: &Term) -> Vec<String> {
    let id = sanitize(id);
    paragraph(
        term,
        &[
            Run::Words(style::RED, "Neznámý level: "),
            Run::Whole(style::RED, &id),
        ],
    )
}

/// `dev next` on the last level.
pub fn no_next_level(term: &Term) -> Vec<String> {
    paragraph(term, &[Run::Words(style::YELLOW, "Žádný další level.")])
}

/// `dev prev` on the first level.
pub fn no_previous_level(term: &Term) -> Vec<String> {
    paragraph(term, &[Run::Words(style::YELLOW, "Žádný předchozí level.")])
}

/// `dev reload` finished.
pub fn level_reloaded(level_id: &str, term: &Term) -> Vec<String> {
    let level_id = sanitize(level_id);
    paragraph(
        term,
        &[
            Run::Words(style::GREEN, "✓ Level "),
            Run::Whole(style::GREEN, &level_id),
            Run::Words(style::GREEN, " znovu načten."),
        ],
    )
}

/// `dev start` on a game without levels.
pub fn no_levels_registered(term: &Term) -> Vec<String> {
    paragraph(
        term,
        &[Run::Words(style::RED, "Nejsou registrovány žádné levely.")],
    )
}

/// `shellgame exit`.
pub fn exiting(term: &Term) -> Vec<String> {
    paragraph(term, &[Run::Words(style::YELLOW, "Ukončuji ShellGame...")])
}

/// The game shell could not be started.
pub fn launch_failed(error: &str, term: &Term) -> Vec<String> {
    let error = sanitize(error);
    paragraph(
        term,
        &[Run::Words(
            style::BOLD_RED,
            &format!("CHYBA: Nepodařilo se spustit herní shell ({error})"),
        )],
    )
}

/// The command that reinstalls ShellGame, for when `update` cannot do it itself.
pub const INSTALL_COMMAND: &str =
    "curl -fsSL https://raw.githubusercontent.com/jdupak/ShellGame/main/scripts/install.sh | bash";

/// `update` succeeded: the version before and after.
pub fn update_done(old: &str, new: &str, term: &Term) -> Vec<String> {
    let old = sanitize(old);
    let new = sanitize(new);
    paragraph_then_blank(
        term,
        &[
            Run::Words(style::GREEN, "✓ ShellGame byl aktualizován: "),
            Run::Whole(style::GREEN, &old),
            Run::Words(style::GREEN, " -> "),
            Run::Whole(style::GREEN, &new),
        ],
    )
}

/// `update` found nothing newer than the running version.
pub fn update_current(version: &str, term: &Term) -> Vec<String> {
    let version = sanitize(version);
    paragraph_then_blank(
        term,
        &[
            Run::Words(style::GREEN, "Již používáte nejnovější verzi: "),
            Run::Whole(style::GREEN, &version),
        ],
    )
}

/// `update` failed. With `reinstall`, the installer command is shown on a line of its own (so it
/// can be copied), as the way to update by hand.
pub fn update_failed(reason: &str, reinstall: bool, term: &Term) -> Vec<String> {
    let reason = sanitize(reason);
    let mut lines = paragraph(term, &[Run::Words(style::RED, &reason)]);
    if reinstall {
        lines.extend(note(
            term,
            &[Run::Words(
                style::DIM,
                "Aktualizaci můžete provést znovu instalačním skriptem:",
            )],
        ));
        lines.push(format!(
            "   {}",
            term.paint(style::DIM_VIOLET, INSTALL_COMMAND)
        ));
    }
    lines.push(String::new());
    lines
}

/// An ordinary line in `style`, for the few messages that have no screen of their own (the
/// progress and the result of `update`).
pub fn plain_line(text: &str, style: Style, term: &Term) -> Vec<String> {
    let text = sanitize(text);
    paragraph(term, &[Run::Words(style, &text)])
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};

    use super::*;
    use crate::engine::level::{intro, level};
    use crate::ui::text::{is_self_contained, strip_ansi};

    fn plain(width: usize) -> Term {
        Term::plain(width)
    }

    fn at() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 3, 4)
            .unwrap()
            .and_hms_opt(14, 5, 9)
            .unwrap()
    }

    fn completion(time_sec: u64, hints: u32, attempts: u32) -> LevelCompletion {
        LevelCompletion {
            time_sec,
            hints,
            attempts,
            completed_at: at(),
        }
    }

    fn state_with(completed: &[(&str, LevelCompletion)]) -> State {
        let mut state = State::new("ada", "/tmp/shellgame-ada", at());
        for (id, record) in completed {
            state
                .levels_complete
                .insert((*id).to_owned(), record.clone());
        }
        state
    }

    fn text(lines: &[String]) -> String {
        lines.join("\n")
    }

    fn widths(lines: &[String]) -> Vec<usize> {
        lines.iter().map(|line| visible_width(line)).collect()
    }

    // --- formatting -------------------------------------------------------------------------

    #[test]
    fn durations_use_seconds_minutes_and_hours() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(59), "59s");
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(61), "1m 1s");
        assert_eq!(format_duration(3599), "59m 59s");
        assert_eq!(format_duration(3600), "1h 0m 0s");
        assert_eq!(format_duration(3723), "1h 2m 3s");
        assert_eq!(format_duration(90_061), "25h 1m 1s");
    }

    #[test]
    fn stats_mention_attempts_only_when_there_were_some() {
        assert_eq!(
            format_stats(&completion(42, 2, 0)),
            "Čas: 42s  |  Nápovědy: 2"
        );
        assert_eq!(
            format_stats(&completion(125, 0, 3)),
            "Čas: 2m 5s  |  Nápovědy: 0  |  Chybné pokusy: 3"
        );
    }

    #[test]
    fn level_ids_sort_numerically_and_odd_ids_come_last() {
        let mut ids = [
            "1.12", "10.1", "1.2", "1.10", "weird", "2.0", "1.1", "alpha",
        ];
        ids.sort_by_key(|id| level_sort_key(id));
        assert_eq!(
            ids,
            [
                "1.1", "1.2", "1.10", "1.12", "2.0", "10.1", "alpha", "weird"
            ]
        );
    }

    // --- levels -----------------------------------------------------------------------------

    #[test]
    fn a_level_panel_is_cyan_titled_with_the_id_and_title() {
        let level = Level {
            id: LevelId::new(1, 4),
            ..level(4, "Návrat na základnu").text("### Cíl\nVraťte se o úroveň výše.")
        };
        let lines = level_panel(&level, &plain(50));
        assert!(
            lines[0].contains(" 1.4: Návrat na základnu "),
            "{}",
            lines[0]
        );
        assert!(text(&lines).contains("Vraťte se o úroveň výše."));
        assert!(
            widths(&lines).iter().all(|w| *w == 50),
            "{:?}",
            widths(&lines)
        );
        assert!(!text(&lines).contains("nepovinný"));
    }

    #[test]
    fn a_bonus_level_adds_the_skip_note() {
        let level = Level {
            id: LevelId::new(5, 2),
            ..level(2, "Hledání podle velikosti").text("Text").optional()
        };
        let lines = level_panel(&level, &plain(80));
        let tail = &lines[lines.len() - 2..];
        assert_eq!(
            tail[0],
            " Tento level je nepovinný. Můžete ho přeskočit příkazem: shellgame skip"
        );
        assert_eq!(tail[1], "");
    }

    #[test]
    fn level_panels_with_colour_are_self_contained() {
        let level = Level {
            id: LevelId::new(5, 2),
            ..level(2, "Bonus").text("Použijte `ls -l`.").optional()
        };
        for line in level_panel(&level, &Term::ansi(60)) {
            assert!(is_self_contained(&line), "{line:?}");
        }
    }

    #[test]
    fn intro_pages_are_numbered_only_when_there_are_several() {
        let level = intro("Vítejte v ShellGame", "x");
        let single = intro_page(&level, "Jedna strana", 1, 1, &plain(60));
        assert!(single[0].contains(" Vítejte v ShellGame "), "{}", single[0]);
        assert!(!single[0].contains("strana"));

        let several = intro_page(&level, "Druhá", 2, 3, &plain(60));
        assert!(
            several[0].contains(" Vítejte v ShellGame (strana 2/3) "),
            "{}",
            several[0]
        );
        assert!(text(&several).contains("Druhá"));
    }

    #[test]
    fn intro_titles_keep_their_colours_in_a_coloured_terminal() {
        let level = intro("Navigace", "x");
        let lines = intro_page(&level, "Text", 1, 2, &Term::ansi(60));
        assert!(lines[0].contains(&style::BOLD_CYAN.render().to_string()));
        assert!(lines[0].contains(&style::DIM.render().to_string()));
        assert_eq!(strip_ansi(&lines[0]).matches("strana 1/2").count(), 1);
    }

    // --- hints ------------------------------------------------------------------------------

    #[test]
    fn a_new_hint_has_a_panel_the_repeat_tip_the_next_pointer_and_a_blank_line() {
        let lines = hint("Zkuste příkaz pwd.", 0, 3, &plain(100));
        let rendered = text(&lines);
        assert!(lines[0].contains("💡 Nápověda 1 z 3"), "{}", lines[0]);
        assert!(rendered.contains("Zkuste příkaz pwd."));
        assert_eq!(
            lines[lines.len() - 3],
            " Tip: Již zobrazené nápovědy si můžete vypsat znovu příkazem: shellgame hint --repeat"
        );
        assert_eq!(
            lines[lines.len() - 2],
            " Potřebujete další pomoc? Napište: shellgame hint"
        );
        assert_eq!(lines[lines.len() - 1], "");
    }

    #[test]
    fn the_last_hint_does_not_point_to_another() {
        let lines = hint("Poslední", 2, 3, &plain(80));
        assert!(!text(&lines).contains("Potřebujete další pomoc"));
        assert!(text(&lines).contains("shellgame hint --repeat"));
        assert_eq!(lines.last().unwrap(), "");
    }

    #[test]
    fn hints_are_printed_verbatim() {
        let lines = hint(
            "Použijte `ls [ab]*.txt` a [bold]nic[/bold].",
            0,
            1,
            &plain(80),
        );
        assert!(
            text(&lines).contains("Použijte `ls [ab]*.txt` a [bold]nic[/bold]."),
            "{lines:#?}"
        );
    }

    #[test]
    fn repeated_hints_list_every_revealed_hint_without_a_tip_per_hint() {
        let hints: Vec<String> = ["první", "druhá", "třetí"].map(String::from).to_vec();
        let lines = repeated_hints(&hints, 2, &plain(80));
        let rendered = text(&lines);
        assert!(rendered.contains("💡 Nápověda 1 z 3"));
        assert!(rendered.contains("💡 Nápověda 2 z 3"));
        assert!(!rendered.contains("Nápověda 3 z 3"));
        assert!(rendered.contains("první") && rendered.contains("druhá"));
        assert_eq!(rendered.matches("shellgame hint --repeat").count(), 1);
        assert_eq!(rendered.matches("Potřebujete další pomoc").count(), 1);
        assert_eq!(lines.last().unwrap(), "");
    }

    #[test]
    fn repeating_all_hints_does_not_offer_more() {
        let hints: Vec<String> = ["a", "b"].map(String::from).to_vec();
        let rendered = text(&repeated_hints(&hints, 5, &plain(80)));
        assert!(rendered.contains("Nápověda 2 z 2"));
        assert!(!rendered.contains("Potřebujete další pomoc"));
    }

    #[test]
    fn repeating_before_any_hint_explains_how_to_get_one() {
        let hints: Vec<String> = ["a", "b"].map(String::from).to_vec();
        let lines = repeated_hints(&hints, 0, &plain(80));
        let rendered = text(&lines);
        assert!(rendered.contains("Zatím jste si nezobrazili žádnou nápovědu."));
        assert!(rendered.contains("První nápovědu zobrazíte příkazem: shellgame hint"));
        assert_eq!(lines.last().unwrap(), "");
    }

    #[test]
    fn the_other_hint_messages_say_what_is_missing() {
        let none = text(&no_hints_available(&plain(80)));
        assert!(none.contains("Pro tento level nejsou k dispozici žádné nápovědy."));
        assert!(!none.contains("shellgame hint --repeat"));

        let exhausted = text(&no_more_hints(&plain(80)));
        assert!(exhausted.contains("Pro tento level již nejsou k dispozici žádné další nápovědy."));
        assert!(exhausted.contains("shellgame hint --repeat"));
    }

    // --- success and failure ----------------------------------------------------------------

    #[test]
    fn success_shows_the_message_and_the_statistics_centred() {
        let lines = success("Správně!", Some(&completion(75, 1, 2)), &plain(60));
        let rendered = text(&lines);
        assert!(rendered.contains("Správně!"));
        assert!(rendered.contains("Čas: 1m 15s  |  Nápovědy: 1  |  Chybné pokusy: 2"));
        assert!(widths(&lines).iter().all(|w| *w == 60));
        // Centred: the short message is indented more than the statistics line.
        let indent = |needle: &str| {
            let line = lines.iter().find(|l| l.contains(needle)).unwrap();
            line.chars().skip(1).take_while(|c| *c == ' ').count()
        };
        assert!(indent("Správně!") > indent("Čas:"));
    }

    #[test]
    fn success_without_a_record_has_no_statistics() {
        let rendered = text(&success("Hotovo.", None, &plain(60)));
        assert!(rendered.contains("Hotovo."));
        assert!(!rendered.contains("Čas:"));
    }

    #[test]
    fn success_messages_are_not_markup() {
        let rendered = text(&success("Použijte [bold]x[/bold] a `y`.", None, &plain(80)));
        assert!(rendered.contains("Použijte [bold]x[/bold] a `y`."));
    }

    #[test]
    fn a_failure_is_red_with_a_pointer_to_hint() {
        let lines = failure("Nesprávně.", &plain(60));
        let rendered = text(&lines);
        assert!(rendered.contains("Nesprávně."));
        assert!(
            lines[lines.len() - 2].ends_with("Zkuste to znovu nebo použijte: shellgame hint"),
            "{lines:#?}"
        );
        assert_eq!(lines.last().unwrap(), "");

        let coloured = failure("Nesprávně.", &Term::ansi(60));
        assert!(coloured[2].contains(&style::BOLD_RED.render().to_string()));
        assert!(coloured[0].contains(&style::RED.render().to_string()));
    }

    #[test]
    fn a_long_failure_message_wraps_inside_the_panel() {
        let message =
            "Tohle je velmi dlouhá zpráva o chybě, která se musí zalomit na více řádků v panelu.";
        let lines = failure(message, &plain(40));
        let panel: Vec<String> = lines
            .iter()
            .filter(|line| "╭│╰".contains(line.chars().next().unwrap_or(' ')))
            .cloned()
            .collect();
        assert!(widths(&panel).iter().all(|w| *w == 40), "{panel:#?}");
        assert!(panel.len() > 6, "the message wraps onto several rows");
        assert!(lines.iter().all(|line| visible_width(line) <= 40));
    }

    #[test]
    fn the_cd_block_panel_carries_the_level_id_and_the_recovery_tip() {
        let lines = cd_block(
            LevelId::new(1, 4),
            "Použijte přesně příkaz 'cd ..'.",
            &plain(80),
        );
        assert!(lines[0].contains(" ShellGame (1.4) "), "{}", lines[0]);
        let rendered = text(&lines);
        assert!(rendered.contains("Použijte přesně příkaz 'cd ..'."));
        assert_eq!(
            lines[lines.len() - 2],
            format!(" {}", messages::CD_RECOVERY_TIP)
        );
        assert_eq!(lines.last().unwrap(), "");
    }

    #[test]
    fn the_cd_block_panel_is_red_in_a_coloured_terminal() {
        let lines = cd_block(LevelId::new(1, 4), "Ne.", &Term::ansi(60));
        assert!(lines[0].contains(&style::RED.render().to_string()));
        assert!(
            lines.iter().all(|line| is_self_contained(line)),
            "{lines:?}"
        );
    }

    #[test]
    fn error_panels_carry_their_text() {
        assert!(
            text(&state_error("Stav nelze načíst.", &plain(60))).contains("Stav nelze načíst.")
        );

        let setup = text(&level_setup_error(
            "1.5",
            "cannot create /ws/level-1",
            &plain(80),
        ));
        assert!(setup.contains("Level 1.5 se nepodařilo připravit."));
        assert!(setup.contains("cannot create /ws/level-1"));
        assert!(setup.contains("Zkuste: shellgame reset"));

        let missing = text(&level_missing("1.11", "1.12", &plain(80)));
        assert!(missing.contains("Level 1.11 už v této verzi hry neexistuje."));
        assert!(missing.contains("Pokračujeme nejbližším dostupným levelem: 1.12"));
        assert!(missing.contains("Váš dosavadní postup zůstává zachován: shellgame status"));

        assert!(
            text(&no_levels_available(&plain(60))).contains("Nejsou registrovány žádné levely.")
        );
    }

    #[test]
    fn the_completion_screen_adds_up_the_game() {
        let state = state_with(&[
            ("0.1", completion(100, 0, 0)),
            ("1.1", completion(3600, 1, 0)),
        ]);
        let lines = game_complete(&state, &plain(80));
        let rendered = text(&lines);
        assert!(rendered.contains("Dokončili jste ShellGame!"));
        assert!(rendered.contains("Dokončené levely: 2  |  Celkový čas: 1h 1m 40s"));
        assert!(rendered.contains(" Statistiky: shellgame status"));
        assert!(rendered.contains(" Ukončit hru: shellgame exit"));
        assert!(rendered.contains(" Zahrát znovu od začátku: shellgame remove a pak shellgame"));
        assert_eq!(lines.last().unwrap(), "");
    }

    // --- status -----------------------------------------------------------------------------

    #[test]
    fn status_without_progress_says_so() {
        let lines = status(&state_with(&[]), &plain(80));
        assert_eq!(lines[0], "");
        assert_eq!(lines[1], "═══ ShellGame Postup ═══");
        assert_eq!(lines[2], "");
        assert_eq!(lines[3], "Aktuální Level: 0.0");
        assert_eq!(lines[4], "Zahájeno: 2026-03-04 14:05");
        assert_eq!(lines[5], "");
        assert_eq!(lines[6], "Zatím žádné dokončené levely.");
        assert_eq!(lines[7], "");
        assert_eq!(lines.len(), 8);
    }

    #[test]
    fn status_tables_sort_levels_numerically() {
        let state = state_with(&[
            ("1.12", completion(5, 0, 0)),
            ("1.2", completion(9, 1, 0)),
            ("1.10", completion(7, 0, 2)),
            ("0.1", completion(1, 0, 0)),
        ]);
        let lines = status(&state, &plain(80));
        let rows: Vec<&String> = lines.iter().filter(|l| l.starts_with('│')).collect();
        let order: Vec<&str> = rows
            .iter()
            .map(|row| {
                row.trim_start_matches('│')
                    .split_whitespace()
                    .next()
                    .unwrap()
            })
            .collect();
        assert_eq!(order, ["0.1", "1.2", "1.10", "1.12"]);
    }

    #[test]
    fn the_status_table_is_aligned_like_a_heavy_head_rich_table() {
        let state = state_with(&[
            ("0.1", completion(5, 0, 0)),
            ("1.10", completion(125, 2, 1)),
        ]);
        let lines = status(&state, &plain(80));
        let start = lines
            .iter()
            .position(|l| l.contains("Dokončené Levely"))
            .unwrap();
        let table = &lines[start..start + 7];
        assert_eq!(
            table,
            [
                "              Dokončené Levely",
                "┏━━━━━━━┳━━━━━━━┳━━━━━━━━━━┳━━━━━━━━━━━━━━━┓",
                "┃ Level ┃   Čas ┃ Nápovědy ┃ Chybné pokusy ┃",
                "┡━━━━━━━╇━━━━━━━╇━━━━━━━━━━╇━━━━━━━━━━━━━━━┩",
                "│ 0.1   │    5s │        0 │             0 │",
                "│ 1.10  │ 2m 5s │        2 │             1 │",
                "└───────┴───────┴──────────┴───────────────┘",
            ]
        );
        assert!(
            widths(&table[1..])
                .windows(2)
                .all(|pair| pair[0] == pair[1])
        );
    }

    #[test]
    fn status_statistics_add_up() {
        let state = state_with(&[
            ("0.1", completion(30, 1, 0)),
            ("1.1", completion(40, 2, 3)),
            ("1.2", completion(50, 0, 1)),
        ]);
        let lines = status(&state, &plain(80));
        assert_eq!(
            lines[lines.len() - 7..],
            [
                "",
                "Statistiky:",
                "  Celkový čas: 2m 0s",
                "  Dokončené levely: 3",
                "  Průměrně nápověd: 1.0",
                "  Celkem chybných pokusů: 4",
                "",
            ]
        );
    }

    #[test]
    fn the_average_of_hints_has_one_decimal() {
        let state = state_with(&[("0.1", completion(1, 1, 0)), ("0.2", completion(1, 0, 0))]);
        assert!(text(&status(&state, &plain(80))).contains("  Průměrně nápověd: 0.5"));
        let three = state_with(&[
            ("0.1", completion(1, 1, 0)),
            ("0.2", completion(1, 0, 0)),
            ("0.3", completion(1, 0, 0)),
        ]);
        assert!(text(&status(&three, &plain(80))).contains("  Průměrně nápověd: 0.3"));
    }

    // --- the small screens ------------------------------------------------------------------

    #[test]
    fn the_one_line_messages_are_exact() {
        let t = plain(100);
        assert_eq!(
            already_initialized(&t),
            [
                "Již inicializováno! Spusťte 'shellgame' pro pokračování.",
                ""
            ]
        );
        assert_eq!(
            not_initialized(&t),
            ["Neinicializováno. Spusťte: shellgame init", ""]
        );
        assert_eq!(
            removed(&t),
            ["Stav ShellGame a pracovní prostor odstraněny."]
        );
        assert_eq!(reset_notice("1.4", &t), ["✓ Level 1.4 byl resetován.", ""]);
        assert_eq!(
            current_directory("/tmp/ws/level-1", &t),
            ["Aktuální adresář: /tmp/ws/level-1"]
        );
        assert_eq!(
            teleport_notice("/tmp/ws", &t),
            ["Byli jste teleportováni do adresáře /tmp/ws", ""]
        );
        assert_eq!(
            skipped("5.2", &t),
            ["Level 5.2 přeskočen (byl nepovinný).", ""]
        );
        assert_eq!(
            level_not_found("9.9", &t),
            ["Chyba: Level 9.9 nenalezen", ""]
        );
        assert_eq!(
            show_usage(&t),
            [
                "Použití: shellgame show --level  nebo  shellgame show --section",
                ""
            ]
        );
        assert_eq!(
            invalid_level_format("x", &t),
            ["Chyba: Neplatný formát aktuálního levelu: x", ""]
        );
        assert_eq!(
            reset_failed("path contains a symlink", &t),
            ["Chyba při obnově levelu: path contains a symlink", ""]
        );
        assert_eq!(unknown_level("7.7", &t), ["Neznámý level: 7.7"]);
        assert_eq!(no_next_level(&t), ["Žádný další level."]);
        assert_eq!(no_previous_level(&t), ["Žádný předchozí level."]);
        assert_eq!(level_reloaded("1.4", &t), ["✓ Level 1.4 znovu načten."]);
        assert_eq!(
            no_levels_registered(&t),
            ["Nejsou registrovány žádné levely."]
        );
        assert_eq!(exiting(&t), ["Ukončuji ShellGame..."]);
        assert_eq!(
            launch_failed("No such file or directory", &t),
            ["CHYBA: Nepodařilo se spustit herní shell (No such file or directory)"]
        );
        assert_eq!(
            removal_declined(&t),
            ["Odstranění zrušeno. Žádná data nebyla smazána."]
        );
    }

    #[test]
    fn the_init_screen_greets_the_player() {
        let lines = init_success("ada", "/tmp/shellgame-ada", &plain(100));
        assert_eq!(
            lines,
            [
                "",
                "✓ ShellGame inicializována pro ada",
                "Pracovní prostor: /tmp/shellgame-ada",
                "",
                "Automaticky vás přesměrovávám do pracovního prostoru...",
                "",
            ]
        );
    }

    #[test]
    fn the_restored_workspace_notice_has_two_lines_and_a_blank() {
        assert_eq!(
            workspace_restored("/tmp/ws", &plain(100)),
            [
                "⚠ Pracovní prostor byl smazán (např. restart systému). Obnovuji...",
                "✓ Pracovní prostor obnoven: /tmp/ws",
                "",
            ]
        );
    }

    #[test]
    fn skipping_a_required_level_points_to_hint() {
        assert_eq!(
            not_skippable("1.1", &plain(100)),
            [
                "Level 1.1 je povinný a nelze ho přeskočit.",
                " Zkuste nápovědu: shellgame hint",
                "",
            ]
        );
    }

    #[test]
    fn dynamic_text_cannot_inject_escape_sequences() {
        let lines = init_success("ev\u{1b}[31mil", "/tmp/\u{1b}x", &Term::ansi(100));
        for line in &lines {
            assert!(!line.contains("\u{1b}[31mil"), "{line:?}");
        }
        assert!(text(&lines).contains('\u{fffd}'));
    }

    #[test]
    fn notes_wrap_to_a_narrow_terminal_and_stay_indented() {
        let lines = bonus_note(&plain(40));
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.starts_with(' '));
            assert!(visible_width(line) <= 40, "{line:?}");
        }
    }

    #[test]
    fn a_newline_in_dynamic_text_starts_a_new_line() {
        let lines = reset_failed("first\nsecond", &plain(80));
        assert_eq!(lines, ["Chyba při obnově levelu: first", "second", ""]);
        assert!(lines.iter().all(|line| !line.contains('\n')));
    }

    #[test]
    fn paths_stay_in_one_piece_when_they_fit() {
        let lines = teleport_notice("/tmp/my workspace/level-1", &plain(40));
        assert!(
            lines
                .iter()
                .any(|l| l.contains("/tmp/my workspace/level-1")),
            "{lines:?}"
        );
    }

    #[test]
    fn update_messages_show_both_versions_and_the_installer() {
        let t = plain(120);
        assert_eq!(
            update_done("shellgame 0.2.0 (dev)", "shellgame 0.3.0 (build-7)", &t),
            [
                "✓ ShellGame byl aktualizován: shellgame 0.2.0 (dev) -> shellgame 0.3.0 (build-7)",
                ""
            ]
        );
        assert_eq!(
            update_current("shellgame 0.2.0 (dev)", &t),
            ["Již používáte nejnovější verzi: shellgame 0.2.0 (dev)", ""]
        );
        let failed = update_failed("Do adresáře /usr/bin nelze zapisovat.", true, &t);
        assert_eq!(failed[0], "Do adresáře /usr/bin nelze zapisovat.");
        assert_eq!(
            failed[1],
            " Aktualizaci můžete provést znovu instalačním skriptem:"
        );
        assert_eq!(failed[2], format!("   {INSTALL_COMMAND}"));
        assert_eq!(failed.last().unwrap(), "");
        assert_eq!(update_failed("Stažení se nepodařilo.", false, &t).len(), 2);
    }

    #[test]
    fn plain_lines_carry_their_style() {
        assert_eq!(plain_line("hotovo", style::GREEN, &plain(40)), ["hotovo"]);
        let coloured = plain_line("hotovo", style::GREEN, &Term::ansi(40));
        assert_eq!(
            coloured[0],
            format!(
                "{}hotovo{}",
                style::GREEN.render(),
                style::GREEN.render_reset()
            )
        );
    }
}
