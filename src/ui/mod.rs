//! Terminal output. Text stays structured (styled [`Span`]s in [`Line`]s) until [`Term::print`]
//! turns it into escape sequences, so layout is plain arithmetic on column widths.

pub mod markdown;
pub mod panel;
pub mod prompt;
pub mod screens;

use std::io::{self, IsTerminal, Write};

use anstream::{AutoStream, ColorChoice};
use anstyle::{Ansi256Color, AnsiColor, Style};
use rustix::{stdio, termios};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub use panel::Panel;

pub const PLAIN: Style = Style::new();
pub const BOLD: Style = Style::new().bold();
pub const DIM: Style = Style::new().dimmed();
pub const DIM_BOLD: Style = DIM.bold();
pub const CYAN: Style = AnsiColor::Cyan.on_default();
pub const BOLD_CYAN: Style = CYAN.bold();
pub const YELLOW: Style = AnsiColor::Yellow.on_default();
pub const BOLD_YELLOW: Style = YELLOW.bold();
pub const GREEN: Style = AnsiColor::Green.on_default();
pub const BOLD_GREEN: Style = GREEN.bold();
pub const RED: Style = AnsiColor::Red.on_default();
pub const BOLD_RED: Style = RED.bold();
pub const VIOLET: Style = Ansi256Color(177).on_default();
pub const DIM_VIOLET: Style = VIOLET.dimmed();

#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub text: String,
    pub style: Style,
}

pub type Line = Vec<Span>;

/// A span of `text`, made safe to print: control characters (a file name must not drive the
/// terminal) become U+FFFD, a tab four spaces, and the emoji variation selector is dropped because
/// terminals disagree about the width of `⚠️` (the text form is one column everywhere).
pub fn span(text: impl AsRef<str>, style: Style) -> Span {
    let text = text.as_ref().replace('\u{fe0f}', "").replace('\t', "    ");
    let clean = text.chars().map(|c| if c.is_control() { '\u{fffd}' } else { c });
    Span { text: clean.collect(), style }
}

pub fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

pub fn line_width(line: &[Span]) -> usize {
    line.iter().map(|span| width(&span.text)).sum()
}

pub fn plain(line: &[Span]) -> String {
    line.iter().map(|span| span.text.as_str()).collect()
}

fn push_char(line: &mut Line, c: char, style: Style) {
    match line.last_mut() {
        Some(last) if last.style == style => last.text.push(c),
        _ => line.push(span(c.to_string(), style)),
    }
}

/// Splits a line after at most `cols` columns (always taking one character, so progress is certain).
pub fn split_at_width(line: Line, cols: usize) -> (Line, Line) {
    let (mut head, mut tail, mut used) = (Line::new(), Line::new(), 0);
    for Span { text, style } in line {
        for c in text.chars() {
            let w = c.width().unwrap_or(0);
            if tail.is_empty() && (used == 0 || used + w <= cols) {
                used += w;
                push_char(&mut head, c, style);
            } else {
                push_char(&mut tail, c, style);
            }
        }
    }
    (head, tail)
}

/// At most `cols` columns, ending in `…` when something was cut.
pub fn truncate(line: Line, cols: usize) -> Line {
    if line_width(&line) <= cols {
        return line;
    }
    let style = line.last().map_or(PLAIN, |last| last.style);
    let mut head = split_at_width(line, cols.saturating_sub(1)).0;
    if cols > 0 {
        head.push(span("…", style));
    }
    head
}

/// Inline content to lay out.
pub enum Piece {
    /// Breaks at its spaces.
    Words(Span),
    /// A command or path: kept in one piece as long as it fits a row.
    Whole(Span),
}

pub fn words(style: Style, text: &str) -> Piece {
    Piece::Words(span(text, style))
}

pub fn whole(style: Style, text: &str) -> Piece {
    Piece::Whole(span(text, style))
}

#[derive(Default)]
struct Rows {
    done: Vec<Line>,
    row: Line,
    used: usize,
    cols: usize,
}

impl Rows {
    fn end_row(&mut self) {
        self.done.push(std::mem::take(&mut self.row));
        self.used = 0;
    }

    /// Puts a word after its `gap` on the row, or on the next one; a word wider than a whole row is
    /// split between characters.
    fn put(&mut self, gap: Option<Span>, word: Line) {
        if word.is_empty() {
            return;
        }
        let (gap_width, word_width) =
            (gap.as_ref().map_or(0, |gap| width(&gap.text)), line_width(&word));
        if self.used > 0 && self.used + gap_width + word_width > self.cols {
            self.end_row();
        }
        if let (true, Some(gap)) = (self.used > 0, gap) {
            self.row.push(gap);
            self.used += gap_width;
        }
        if self.used + word_width <= self.cols {
            self.used += word_width;
            self.row.extend(word);
            return;
        }
        for Span { text, style } in word {
            for c in text.chars() {
                let w = c.width().unwrap_or(0);
                if self.used > 0 && self.used + w > self.cols {
                    self.end_row();
                }
                push_char(&mut self.row, c, style);
                self.used += w;
            }
        }
    }
}

/// Greedy line breaking at spaces. A run of spaces is kept inside a row and dropped at a break.
pub fn reflow(pieces: &[Piece], cols: usize) -> Vec<Line> {
    let cols = cols.max(1);
    let mut rows = Rows { cols, ..Rows::default() };
    let (mut word, mut gap) = (Line::new(), None);
    for piece in pieces {
        match piece {
            Piece::Whole(span) if width(&span.text) <= rows.cols => word.push(span.clone()),
            Piece::Words(span) | Piece::Whole(span) => {
                let mut rest = span.text.as_str();
                while let Some(first) = rest.chars().next() {
                    let end =
                        rest.find(|c: char| (c == ' ') != (first == ' ')).unwrap_or(rest.len());
                    let (run, tail) = rest.split_at(end);
                    rest = tail;
                    if first == ' ' {
                        rows.put(gap.take(), std::mem::take(&mut word));
                        gap = Some(self::span(run, span.style));
                    } else {
                        word.push(self::span(run, span.style));
                    }
                }
            }
        }
    }
    rows.put(gap.take(), word);
    if !rows.row.is_empty() {
        rows.end_row();
    }
    rows.done
}

/// Words, with every span quoted by `'` or `` ` `` (a command in a hint or message) kept whole.
fn quoted(text: &str, style: Style) -> Vec<Piece> {
    let (mut pieces, mut rest) = (vec![], text);
    while let Some(open) = rest.find(['\'', '`']) {
        let quote = &rest[open..open + 1];
        let Some(close) = rest[open + 1..].find(quote) else { break };
        let end = open + 1 + close + 1;
        pieces.extend([words(style, &rest[..open]), whole(style, &rest[open..end])]);
        rest = &rest[end..];
    }
    pieces.push(words(style, rest));
    pieces
}

/// Text exactly as written, wrapped at spaces (hints and messages are never markdown): newlines
/// and blank lines stay, a continuation row repeats its line's indentation, and a quoted command
/// stays on one row when it fits.
pub fn wrap_plain(text: &str, cols: usize, style: Style) -> Vec<Line> {
    let mut out = Vec::new();
    for raw in text.lines() {
        let body = raw.trim_start_matches(' ');
        let indent = (raw.len() - body.len()).min(cols / 2);
        if body.trim().is_empty() {
            out.push(Line::new());
        }
        for mut row in reflow(&quoted(body.trim_end(), style), cols - indent) {
            row.insert(0, span(" ".repeat(indent), PLAIN));
            out.push(row);
        }
    }
    out
}

/// Where output goes: the width of the layout, whether to emit colour, which stream.
#[derive(Clone, Copy, Debug)]
pub struct Term {
    pub width: usize,
    pub color: bool,
    pub stderr: bool,
}

fn wants_color(stream: &impl anstream::stream::RawStream) -> bool {
    AutoStream::choice(stream) != ColorChoice::Never
}

pub fn write_now(mut out: impl Write, bytes: &[u8]) {
    let _ = out.write_all(bytes).and_then(|()| out.flush());
}

/// The columns of the first of stdout, stderr and stdin that is a terminal; 80 when none is.
fn terminal_width() -> usize {
    let fds = [stdio::stdout(), stdio::stderr(), stdio::stdin()];
    let size =
        fds.into_iter().find_map(|fd| termios::tcgetwinsize(fd).ok().filter(|s| s.ws_col > 0));
    size.map_or(80, |size| usize::from(size.ws_col)).max(40)
}

impl Term {
    fn new(color: bool, stderr: bool) -> Term {
        Term { width: terminal_width(), color, stderr }
    }

    /// Colour follows the usual rules (`NO_COLOR`, `CLICOLOR`, a terminal).
    pub fn stdout() -> Term {
        Term::new(wants_color(&io::stdout()), false)
    }

    /// The same rules, for the live stderr of a hook.
    pub fn stderr() -> Term {
        Term::new(wants_color(&io::stderr()), true)
    }

    /// One string; adjacent spans of one style share their escape sequences.
    pub fn render(&self, line: &[Span]) -> String {
        let mut runs: Vec<(Style, String)> = Vec::new();
        for span in line {
            match runs.last_mut() {
                Some((style, text)) if *style == span.style => text.push_str(&span.text),
                _ => runs.push((span.style, span.text.clone())),
            }
        }
        let paint =
            |(style, text): (Style, String)| match self.color && style != PLAIN && !text.is_empty()
            {
                true => format!("{}{text}{}", style.render(), style.render_reset()),
                false => text,
            };
        runs.into_iter().map(paint).collect()
    }

    /// A closed or broken stream is not worth failing a command for.
    pub fn print(&self, lines: &[Line]) {
        let text: String = lines.iter().map(|line| self.render(line) + "\n").collect();
        if self.stderr {
            write_now(io::stderr(), text.as_bytes());
        } else {
            write_now(io::stdout(), text.as_bytes());
        }
    }

    /// Clears a terminal screen before a panel; piped output never gets the escape sequences.
    pub fn clear(&self) {
        if io::stdout().is_terminal() {
            write_now(io::stdout(), b"\x1b[2J\x1b[H");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(lines: Vec<Line>) -> Vec<String> {
        lines.iter().map(|row| plain(row)).collect()
    }

    #[test]
    fn text_wraps_at_spaces_keeps_commands_whole_and_splits_long_words() {
        let prose = [words(PLAIN, "jedna dva tři čtyři")];
        assert_eq!(text(reflow(&prose, 9)), ["jedna dva", "tři čtyři"]);
        assert_eq!(text(reflow(&[words(PLAIN, "abcdefghij")], 4)), ["abcd", "efgh", "ij"]);
        let command =
            [words(PLAIN, "alpha beta "), whole(PLAIN, "cd ../finish"), words(PLAIN, " gamma")];
        assert_eq!(text(reflow(&command, 20)), ["alpha beta", "cd ../finish gamma"]);
        assert_eq!(
            text(reflow(&[whole(PLAIN, "cd ../finish now")], 10)),
            ["cd", "../finish", "now"]
        );
    }

    #[test]
    fn plain_text_is_verbatim_and_spans_are_safe() {
        let hint = "použijte `ls [ab]*.txt` zde\n\n    odsazený řádek je dlouhý";
        let wrapped =
            ["použijte", "`ls [ab]*.txt`", "zde", "", "    odsazený", "    řádek je", "    dlouhý"];
        assert_eq!(text(wrap_plain(hint, 14, PLAIN)), wrapped);
        assert_eq!(span("a\x1b[31mb\tc⚠\u{fe0f}", PLAIN).text, "a\u{fffd}[31mb    c⚠");
        let line = vec![span("příliš ", BOLD), span("žluťoučký", PLAIN)];
        assert_eq!(plain(&truncate(line.clone(), 8)), "příliš …");
        assert_eq!(plain(&split_at_width(line, 7).0), "příliš ");
    }
}
