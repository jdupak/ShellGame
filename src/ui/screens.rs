//! What the player sees: panels and notes built from plain data. Czech text for the player; hints,
//! messages, paths and names are printed verbatim (never as markdown), only a level's own text
//! goes through the markdown renderer.

use anstyle::Style;

use super::{
    BOLD, BOLD_CYAN, BOLD_GREEN, BOLD_RED, BOLD_YELLOW, CYAN, DIM, DIM_VIOLET, GREEN, Line, PLAIN,
    Panel, Piece, RED, Term, VIOLET, YELLOW, markdown, reflow, span, whole, words, wrap_plain,
};
use crate::game::{Pos, dedent, messages};
use crate::levels::COURSE;
use crate::state::{Progress, State};

/// A paragraph in one style, which may span several lines, and a blank line.
pub fn say(term: &Term, style: Style, text: &str) -> Vec<Line> {
    let mut lines = wrap_plain(text, term.width, style);
    lines.push(Line::new());
    lines
}

/// A paragraph indented by one space, for what a screen adds below its panel.
pub fn note(term: &Term, pieces: &[Piece]) -> Vec<Line> {
    let mut rows = reflow(pieces, term.width.saturating_sub(1));
    rows.iter_mut().for_each(|row| row.insert(0, span(" ", PLAIN)));
    rows
}

/// A dim note that ends in a command shown in violet.
pub fn cmd_note(term: &Term, text: &str, command: &str) -> Vec<Line> {
    note(term, &[words(DIM, text), whole(DIM_VIOLET, command)])
}

/// A centred panel of paragraphs (one style each), then notes with a command, then a blank line.
fn alert(
    term: &Term,
    border: Style,
    heading: &str,
    parts: &[(Style, &str)],
    tips: &[(&str, &str)],
) -> Vec<Line> {
    let title = if heading.is_empty() { vec![] } else { vec![span(heading, BOLD_RED)] };
    let panel = Panel { border, title, pad: (1, 4), center: true };
    let mut body = Vec::new();
    for (style, text) in parts {
        if !body.is_empty() {
            body.push(Line::new());
        }
        body.extend(wrap_plain(text, panel.inner(term.width), *style));
    }
    let mut lines = panel.render(body, term.width);
    tips.iter().for_each(|(text, command)| lines.extend(cmd_note(term, text, command)));
    lines.push(Line::new());
    lines
}

/// `37s`, `2m 5s`, `1h 2m 3s`.
pub fn duration(seconds: u64) -> String {
    match (seconds / 3600, seconds / 60 % 60, seconds % 60) {
        (0, 0, s) => format!("{s}s"),
        (0, m, s) => format!("{m}m {s}s"),
        (h, m, s) => format!("{h}h {m}m {s}s"),
    }
}

pub fn stats(done: &Progress) -> String {
    let (time, hints) = (duration(done.seconds()), done.hints);
    let attempts = format!("  |  Chybné pokusy: {}", done.attempts);
    format!("Čas: {time}  |  Nápovědy: {hints}{}", if done.attempts > 0 { &attempts } else { "" })
}

fn markdown_panel(term: &Term, heading: Line, md: &str) -> Vec<Line> {
    let panel = Panel { border: CYAN, title: heading, pad: (1, 2), ..Panel::default() };
    panel.render(markdown::render(md, panel.inner(term.width)), term.width)
}

/// The cyan panel with the assignment.
pub fn level(term: &Term, pos: Pos) -> Vec<Line> {
    let level = pos.level();
    let heading = vec![span(format!("{pos}: {}", level.title), BOLD_CYAN)];
    markdown_panel(term, heading, &dedent(level.text))
}

/// Page `number` of `total` of a section's intro.
pub fn intro_page(
    term: &Term,
    section: usize,
    page: &str,
    number: usize,
    total: usize,
) -> Vec<Line> {
    let mut heading = format!("Sekce {section}: {}", COURSE[section].title);
    if total > 1 {
        heading += &format!(" (strana {number}/{total})");
    }
    markdown_panel(term, vec![span(heading, BOLD_CYAN)], page)
}

/// A yellow panel for every one of the first `shown` hints (numbered from 1 of `texts.len()`), then
/// where to ask for the next one, or that there is none.
pub fn hints(term: &Term, texts: &[&str], shown: usize) -> Vec<Line> {
    let mut lines = Vec::new();
    for (index, text) in texts[..shown].iter().enumerate() {
        let title = vec![span(format!("💡 Nápověda {} z {}", index + 1, texts.len()), YELLOW)];
        let panel = Panel { border: YELLOW, title, pad: (1, 2), ..Panel::default() };
        lines.extend(panel.render(wrap_plain(text, panel.inner(term.width), PLAIN), term.width));
        lines.push(Line::new());
    }
    lines.extend(if shown < texts.len() {
        let mut tips = cmd_note(term, "Potřebujete další pomoc? Napište: ", "shellgame hint");
        tips.extend(cmd_note(term, "Jen zopakovat odemčené: ", "shellgame hint -r"));
        tips
    } else {
        note(term, &[words(DIM, "Pro tento level již nejsou k dispozici žádné další nápovědy.")])
    });
    lines.push(Line::new());
    lines
}

pub fn success(term: &Term, message: &str, done: &Progress) -> Vec<Line> {
    alert(term, GREEN, "", &[(BOLD_GREEN, message), (DIM, &stats(done))], &[])
}

pub fn failure(term: &Term, message: &str) -> Vec<Line> {
    let tip = [("Zkuste to znovu nebo použijte: ", "shellgame hint")];
    alert(term, RED, "", &[(BOLD_RED, message)], &tip)
}

/// The red panel of a blocked `cd`, with the way out below it.
pub fn cd_block(term: &Term, pos: Pos, message: &str) -> Vec<Line> {
    let heading = format!("ShellGame ({pos})");
    alert(term, RED, &heading, &[(BOLD_RED, message)], &[(messages::CD_RECOVERY_TIP, "")])
}

pub fn error(term: &Term, message: &str) -> Vec<Line> {
    alert(term, RED, "", &[(BOLD_RED, message)], &[])
}

pub fn setup_error(term: &Term, pos: Pos, why: &str) -> Vec<Line> {
    let headline = format!("Level {pos} se nepodařilo připravit.");
    let parts = [(BOLD_RED, headline.as_str()), (DIM, why)];
    alert(term, RED, "", &parts, &[("Zkuste: ", "shellgame reset")])
}

/// An older saved game was set aside as `file`; `resume` is where it was, when it said.
pub fn retired(term: &Term, file: &str, resume: Option<&str>) -> Vec<Line> {
    let headline = "Uložená hra pochází ze starší verze ShellGame a byla odložena.";
    let detail = format!("Původní záznam zůstává v souboru {file}.");
    let command = format!("shellgame resume {}", resume.unwrap_or("S.L"));
    let mut tips = vec![];
    if resume.is_none() {
        tips.push(("Najděte svůj level: ", "shellgame levels"));
    }
    tips.push(("Pokračujte tam, kde jste skončili: ", command.as_str()));
    alert(term, YELLOW, "", &[(BOLD_YELLOW, headline), (DIM, &detail)], &tips)
}

pub fn stale(term: &Term, saved: &str, resolved: Pos) -> Vec<Line> {
    let headline = format!("Level {saved} už v této verzi hry neexistuje.");
    let detail = format!("Pokračujeme nejbližším dostupným levelem: {resolved}");
    let tip = [("Váš dosavadní postup zůstává zachován: ", "shellgame status")];
    alert(term, YELLOW, "", &[(BOLD_YELLOW, &headline), (DIM, &detail)], &tip)
}

fn done_levels(state: &State) -> Vec<(&String, &Progress)> {
    state.levels.iter().filter(|(_, progress)| progress.done.is_some()).collect()
}

/// "Dokončené levely: 5", "Celkový čas: 3m 20s", "Nápovědy: 2", "Chybné pokusy: 1"
fn totals(done: &[(&String, &Progress)]) -> [String; 4] {
    let sum = |value: fn(&Progress) -> u64| done.iter().map(|(_, p)| value(p)).sum::<u64>();
    let (time, hints, attempts) =
        (sum(Progress::seconds), sum(|p| p.hints.into()), sum(|p| p.attempts.into()));
    [
        format!("Dokončené levely: {}", done.len()),
        format!("Celkový čas: {}", duration(time)),
        format!("Nápovědy: {hints}"),
        format!("Chybné pokusy: {attempts}"),
    ]
}

pub fn complete(term: &Term, state: &State) -> Vec<Line> {
    let summary = totals(&done_levels(state)).join("  |  ");
    let parts = [(BOLD_GREEN, "Dokončili jste ShellGame!"), (DIM, summary.as_str())];
    let tips = [
        ("Statistiky: ", "shellgame status"),
        ("Ukončit hru: ", "shellgame exit"),
        ("Zahrát znovu od začátku: ", "shellgame remove"),
    ];
    alert(term, GREEN, "", &parts, &tips)
}

/// "Na jiném počítači pokračujte příkazem: shellgame resume 3.2"
pub fn resume_hint(term: &Term, position: &str) -> Vec<Line> {
    let command = format!("shellgame resume {position}");
    cmd_note(term, "Na jiném počítači pokračujte příkazem: ", &command)
}

/// Every level by section: "▸ zde" marks the saved position, "✓" the completed ones.
pub fn levels(term: &Term, state: Option<&State>) -> Vec<Line> {
    let panel = Panel {
        border: CYAN,
        title: vec![span("Levely", BOLD_CYAN)],
        pad: (1, 2),
        ..Panel::default()
    };
    let current = state.filter(|state| !state.finished).map(|state| state.level.as_str());
    let done =
        |pos: &str| state.and_then(|state| state.levels.get(pos)).is_some_and(|p| p.done.is_some());
    let width = |texts: &mut dyn Iterator<Item = usize>| texts.max().unwrap_or(0);
    let id_width = width(&mut Pos::all().map(|pos| pos.to_string().len()));
    let title_width = width(&mut Pos::all().map(|pos| pos.level().title.chars().count()));
    let mut body = Vec::new();
    for (n, section) in COURSE.iter().enumerate() {
        if n > 0 {
            body.push(Line::new());
        }
        body.push(vec![span(format!("Sekce {n}: {}", section.title), BOLD_CYAN)]);
        for level in 1..=section.levels.len() {
            let (pos, title) = (Pos { section: n, level }, section.levels[level - 1].title);
            let id = pos.to_string();
            let mut row = vec![span(format!("  {id:<id_width$}  {title:<title_width$}"), PLAIN)];
            if done(&id) {
                row.push(span("  ✓", GREEN));
            }
            if current == Some(id.as_str()) {
                row.push(span("  ▸ zde", BOLD_YELLOW));
            }
            body.push(row);
        }
    }
    let mut lines = panel.render(body, term.width);
    lines.extend(cmd_note(term, "Pokračovat od zvoleného levelu: ", "shellgame resume 1.4"));
    lines.push(Line::new());
    lines
}

/// A new game: who it is for and where it lives.
pub fn init(term: &Term, user: &str, workspace: &str) -> Vec<Line> {
    let who = [words(GREEN, "✓ ShellGame inicializována pro "), whole(GREEN, user)];
    let mut lines = reflow(&who, term.width);
    lines.extend(reflow(&[words(DIM, "Pracovní prostor: "), whole(DIM, workspace)], term.width));
    lines.push(Line::new());
    lines
}

/// One line about the isolation of the game shell, from `SHELLGAME_SANDBOX`; nothing outside it.
pub fn banner(term: &Term, sandbox: Option<&str>) -> Vec<Line> {
    let (style, text) = match sandbox {
        Some("landlock") => (DIM, "Izolace zapnuta: mimo /tmp nic nesmažete ani nezměníte."),
        Some("off") => (
            RED,
            "POZOR: herní shell běží bez izolace (--unsafe). Příkazy mohou změnit vaše soubory.",
        ),
        _ => return Vec::new(),
    };
    let mut lines = note(term, &[words(style, text)]);
    lines.push(Line::new());
    lines
}

pub fn teleport(term: &Term, dir: &str) -> Vec<Line> {
    let moved = [
        words(YELLOW, "Byli jste "),
        words(BOLD_YELLOW, "teleportováni"),
        words(YELLOW, " do adresáře "),
        whole(VIOLET, dir),
    ];
    let mut lines = reflow(&moved, term.width);
    lines.push(Line::new());
    lines
}

/// Rows of cells in aligned columns two spaces apart: the first column left-aligned, the others right.
fn columns(rows: &[Vec<String>]) -> Vec<String> {
    let width = |c: usize| rows.iter().map(|row| row[c].chars().count()).max().unwrap_or(0);
    let cell = |(c, text): (usize, &String)| match c {
        0 => format!("{text:<w$}", w = width(c)),
        _ => format!("{text:>w$}", w = width(c)),
    };
    rows.iter().map(|row| row.iter().enumerate().map(cell).collect::<Vec<_>>().join("  ")).collect()
}

/// The finished levels with their statistics, and where to continue on another computer.
pub fn status(term: &Term, state: &State) -> Vec<Line> {
    let title = vec![span("Postup", BOLD_CYAN)];
    let panel = Panel { border: CYAN, title, pad: (1, 2), ..Panel::default() };
    let line = |text: String, style: Style| vec![span(text, style)];
    let mut body = vec![line(format!("Aktuální level: {}", state.level), BOLD), Line::new()];
    let mut done = done_levels(state);
    done.sort_by_key(|(key, _)| key.parse::<Pos>().ok());
    if done.is_empty() {
        body.push(line("Zatím žádné dokončené levely.".into(), DIM));
    } else {
        let cells = |(key, p): &(&String, &Progress)| {
            vec![(*key).clone(), duration(p.seconds()), p.hints.to_string(), p.attempts.to_string()]
        };
        let mut rows =
            vec![["Level", "Čas", "Nápovědy", "Chybné pokusy"].map(String::from).to_vec()];
        rows.extend(done.iter().map(cells));
        let table = columns(&rows).into_iter().enumerate();
        body.extend(table.map(|(n, row)| line(row, if n == 0 { BOLD } else { PLAIN })));
        body.push(Line::new());
        body.extend(totals(&done).map(|total| line(total, PLAIN)));
    }
    let mut lines = panel.render(body, term.width);
    if !state.finished {
        lines.extend(resume_hint(term, &state.level));
    }
    lines.push(Line::new());
    lines
}

/// Why the game does not start without Landlock, and what to do instead.
pub fn refusal(term: &Term) -> Vec<Line> {
    let headline = "Tento systém nemá Landlock, kterým hra chrání vaše soubory.";
    let mut lines =
        alert(term, RED, "ShellGame nelze bezpečně spustit", &[(BOLD_RED, headline)], &[]);
    let paragraphs = [
        "Hra spouští příkazový řádek, který smí měnit soubory jen v /tmp. K tomu potřebuje Landlock, funkci jádra Linuxu, kterou tento systém nemá nebo nepovoluje.",
        "Podporované systémy: Ubuntu 22.04 a novější, Debian 12 a novější, Fedora, Arch Linux, RHEL 9.6 a novější, WSL2.",
        "Jinde (například na macOS) můžete hru spustit na vlastní riziko bez izolace: shellgame --unsafe. Příkazy ve hře pak mohou změnit nebo smazat vaše soubory.",
    ];
    paragraphs.iter().for_each(|text| lines.extend(say(term, PLAIN, text)));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{line_width, plain};

    #[test]
    fn every_panel_is_as_wide_as_the_terminal() {
        let first = Pos { section: 0, level: 1 };
        for width in [40, 80] {
            let term = Term { width, color: false, stderr: false };
            let panels = [
                level(&term, first),
                failure(&term, "Chyba, která se musí zalomit na více řádků"),
                refusal(&term),
            ];
            let framed =
                panels.iter().flatten().filter(|line| plain(line).starts_with(['╭', '│', '╰']));
            assert!(framed.map(|line| line_width(line)).all(|w| w == width));
            let body = plain(&panels[0][2]);
            assert!(body.starts_with("│  ") && !body.starts_with("│   "), "{body}");
        }
    }
}
