//! Terminal rendering primitives.
//!
//! This layer turns data into lines of text and knows nothing about game state. The concrete
//! screens (level panel, hint panel, success panel, status table, …) are built on top of it.
//!
//! | Module | Contents |
//! |---|---|
//! | [`style`] | the colour palette and markdown element styles as [`anstyle::Style`] constants |
//! | [`text`] | visible width, padding, centering, verbatim word wrapping, ANSI-aware cutting |
//! | [`panel`] | [`Panel`], a rounded box with a title in the top border |
//! | [`markdown`] | [`render_markdown`] (CommonMark with paragraph reflow) and [`split_pages`] |
//! | [`prompt`] | [`wait_for_enter`] and [`confirm`] |
//!
//! Everything that produces lines takes a [`Term`]: the layout width plus a flag saying whether
//! ANSI colour should be emitted. Renderers return `Vec<String>` with one entry per output line;
//! every entry is self-contained (styles opened on a line are closed on it), so lines can be
//! printed, indented, boxed or reordered freely. [`print_lines`] writes them through
//! [`anstream`], which strips colour again when the destination does not want it.
//!
//! ```
//! use shellgame::ui::{Panel, Term, render_markdown, style};
//!
//! let term = Term::plain(40);
//! let panel = Panel::new(style::CYAN).title("Level 1.1");
//! let body = render_markdown("Přejděte do `start`.", &term.with_width(panel.body_width(&term)));
//! let lines = panel.render(&body, &term);
//! assert_eq!(lines[0], "╭───────────── Level 1.1 ──────────────╮");
//! assert_eq!(lines[2], "│  Přejděte do start.                  │");
//! ```

pub mod markdown;
pub mod panel;
pub mod prompt;
pub mod screens;
pub mod style;
pub mod text;

use std::io::{self, IsTerminal, Write};

use anstream::stream::{AsLockedWrite, RawStream};
use anstream::{AutoStream, ColorChoice};
use anstyle::Style;
use terminal_size::{Width, terminal_size};

pub use markdown::{render_markdown, split_pages};
pub use panel::Panel;
pub use prompt::{PromptOutcome, confirm, enter_prompt, wait_for_enter};

/// Escape sequences that clear the screen and move the cursor to the top left corner.
const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";

/// The standard stream a [`Term`] writes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// What the output looks like: layout width, colour and destination.
///
/// Construct it once per run with [`Term::stdout`], [`Term::stderr`] or [`Term::stderr_forced`]
/// and pass it to every renderer. Tests use [`Term::plain`] and [`Term::ansi`], which never touch
/// the real terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Term {
    /// Columns available for layout; panels span exactly this many.
    pub width: usize,
    /// Whether renderers should emit ANSI colour and text attributes.
    pub color: bool,
    /// Where [`print_lines`] writes.
    pub stream: Stream,
    /// Whether standard output is a terminal, so screen control such as [`clear_screen`] is
    /// allowed.
    pub interactive: bool,
}

impl Term {
    /// Narrowest layout width ever used, however small the terminal reports itself.
    pub const MIN_WIDTH: usize = 40;
    /// Layout width when no terminal size is available (output is piped, for example).
    pub const FALLBACK_WIDTH: usize = 80;

    /// A terminal for standard output.
    ///
    /// The width is that of the terminal (falling back to [`Self::FALLBACK_WIDTH`], never below
    /// [`Self::MIN_WIDTH`]). Colour follows [`anstream`]'s auto-detection: on for a terminal,
    /// off when `NO_COLOR` is set, `CLICOLOR=0` or `TERM=dumb`, and off when output is redirected
    /// (unless `CLICOLOR_FORCE` is set).
    #[must_use]
    pub fn stdout() -> Self {
        let out = io::stdout();
        Self {
            width: detect_width(),
            color: wants_color(&out),
            stream: Stream::Stdout,
            interactive: out.is_terminal(),
        }
    }

    /// A terminal for standard error with colour auto-detected from stderr itself.
    #[must_use]
    pub fn stderr() -> Self {
        let err = io::stderr();
        Self {
            width: detect_width(),
            color: wants_color(&err),
            stream: Stream::Stderr,
            interactive: io::stdout().is_terminal(),
        }
    }

    /// A terminal for standard error whose output is replayed to the terminal later.
    ///
    /// The shell wrapper captures stderr into a file and prints it when the command ends, so
    /// stderr itself is not a terminal and auto-detection would turn colour off. Colour is
    /// forced on here unless a terminal that is attached says no: when stdout (or else stderr)
    /// is a terminal, its auto-detection decides, which honours `NO_COLOR`, `CLICOLOR=0` and
    /// `TERM=dumb`.
    #[must_use]
    pub fn stderr_forced() -> Self {
        let out = io::stdout();
        let err = io::stderr();
        let color = if out.is_terminal() {
            wants_color(&out)
        } else if err.is_terminal() {
            wants_color(&err)
        } else {
            true
        };
        Self {
            width: detect_width(),
            color,
            stream: Stream::Stderr,
            interactive: out.is_terminal(),
        }
    }

    /// A colourless terminal of the given width that is never interactive.
    ///
    /// Rendered output contains no escape sequences at all. Meant for tests and for output that
    /// is known to be consumed by a program.
    #[must_use]
    pub fn plain(width: usize) -> Self {
        Self {
            width,
            color: false,
            stream: Stream::Stdout,
            interactive: false,
        }
    }

    /// Like [`Term::plain`] but with colour on, for testing ANSI output.
    #[must_use]
    pub fn ansi(width: usize) -> Self {
        Self {
            color: true,
            ..Self::plain(width)
        }
    }

    /// The same terminal with a different layout width, for content that is rendered inside
    /// a container (see [`Panel::body_width`]).
    #[must_use]
    pub fn with_width(self, width: usize) -> Self {
        Self { width, ..self }
    }

    /// `text` in `style` when colour is on, otherwise unchanged. The result is self-contained.
    ///
    /// ```
    /// use shellgame::ui::{Term, style};
    ///
    /// assert_eq!(Term::plain(80).paint(style::BOLD, "x"), "x");
    /// assert_eq!(Term::ansi(80).paint(style::BOLD, "x"), "\x1b[1mx\x1b[0m");
    /// ```
    #[must_use]
    pub fn paint(&self, style: Style, text: &str) -> String {
        let mut out = String::with_capacity(text.len() + 16);
        text::push_styled(&mut out, style, text, self.color);
        out
    }

    /// Applies [`Term::paint`] to every line.
    #[must_use]
    pub fn paint_lines(&self, style: Style, lines: &[String]) -> Vec<String> {
        lines.iter().map(|line| self.paint(style, line)).collect()
    }
}

/// Terminal width in columns, bounded below by [`Term::MIN_WIDTH`].
fn detect_width() -> usize {
    terminal_size()
        .map_or(Term::FALLBACK_WIDTH, |(Width(columns), _)| {
            usize::from(columns)
        })
        .max(Term::MIN_WIDTH)
}

/// Whether [`anstream`]'s auto-detection wants colour on `raw`.
fn wants_color<S: RawStream>(raw: &S) -> bool {
    AutoStream::choice(raw) != ColorChoice::Never
}

/// Writes `lines` to the stream of `term`, one per line.
///
/// The text goes through [`anstream`]: when `term.color` is off every ANSI escape sequence is
/// stripped (a safety net for strings that were styled by hand), when it is on they are passed
/// through unchanged.
///
/// # Errors
///
/// Returns the I/O error of the underlying stream, e.g. a broken pipe.
pub fn print_lines(term: &Term, lines: &[String]) -> io::Result<()> {
    let mut text = String::with_capacity(lines.iter().map(|line| line.len() + 1).sum());
    for line in lines {
        text.push_str(line);
        text.push('\n');
    }
    let choice = if term.color {
        ColorChoice::Always
    } else {
        ColorChoice::Never
    };
    match term.stream {
        Stream::Stdout => write_through(io::stdout().lock(), choice, &text).map(drop),
        Stream::Stderr => write_through(io::stderr().lock(), choice, &text).map(drop),
    }
}

/// Writes one line, see [`print_lines`].
///
/// # Errors
///
/// Returns the I/O error of the underlying stream.
pub fn print_line(term: &Term, line: &str) -> io::Result<()> {
    print_lines(term, &[line.to_owned()])
}

fn write_through<W>(raw: W, choice: ColorChoice, text: &str) -> io::Result<W>
where
    W: RawStream + AsLockedWrite,
{
    let mut stream = AutoStream::new(raw, choice);
    stream.write_all(text.as_bytes())?;
    stream.flush()?;
    Ok(stream.into_inner())
}

/// Clears the screen and moves the cursor to the top left corner.
///
/// Does nothing unless standard output is a terminal ([`Term::interactive`]), so piped output and
/// tests never receive the escape sequences.
pub fn clear_screen(term: &Term) {
    if term.interactive {
        let mut out = io::stdout().lock();
        // A failing terminal write is not worth reporting; the screen simply stays as it is.
        let _ = out.write_all(CLEAR_SCREEN.as_bytes());
        let _ = out.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn through(choice: ColorChoice, text: &str) -> String {
        let bytes = write_through(Vec::new(), choice, text).expect("writing to a Vec cannot fail");
        String::from_utf8(bytes).expect("valid UTF-8")
    }

    #[test]
    fn plain_term_has_no_colour_and_is_not_interactive() {
        let term = Term::plain(60);
        assert_eq!(term.width, 60);
        assert!(!term.color);
        assert!(!term.interactive);
        assert_eq!(term.stream, Stream::Stdout);
    }

    #[test]
    fn ansi_term_differs_only_in_colour() {
        let term = Term::ansi(60);
        assert!(term.color);
        assert_eq!(
            term.with_width(60),
            Term {
                color: true,
                ..Term::plain(60)
            }
        );
    }

    #[test]
    fn with_width_changes_only_the_width() {
        let term = Term::ansi(100).with_width(30);
        assert_eq!(term.width, 30);
        assert!(term.color);
    }

    #[test]
    fn paint_respects_the_colour_flag() {
        assert_eq!(Term::plain(40).paint(style::RED, "chyba"), "chyba");
        let painted = Term::ansi(40).paint(style::RED, "chyba");
        assert!(painted.starts_with("\x1b[31m"));
        assert!(painted.ends_with("\x1b[0m"));
        assert_eq!(Term::ansi(40).paint(style::PLAIN, "x"), "x");
        assert_eq!(Term::ansi(40).paint(style::RED, ""), "");
    }

    #[test]
    fn paint_lines_paints_each_line_separately() {
        let lines = vec!["a".to_owned(), String::new(), "b".to_owned()];
        let painted = Term::ansi(40).paint_lines(style::BOLD, &lines);
        assert_eq!(painted, ["\x1b[1ma\x1b[0m", "", "\x1b[1mb\x1b[0m"]);
    }

    #[test]
    fn detected_width_is_at_least_the_minimum() {
        assert!(detect_width() >= Term::MIN_WIDTH);
        assert!(Term::stdout().width >= Term::MIN_WIDTH);
        assert!(Term::stderr_forced().width >= Term::MIN_WIDTH);
    }

    #[test]
    fn writing_without_colour_strips_escape_sequences() {
        let styled = format!("{}red{}\n", style::RED.render(), style::RED.render_reset());
        assert_eq!(through(ColorChoice::Never, &styled), "red\n");
    }

    #[test]
    fn writing_with_colour_passes_escape_sequences_through() {
        let styled = format!("{}red{}\n", style::RED.render(), style::RED.render_reset());
        assert_eq!(through(ColorChoice::Always, &styled), styled);
    }

    #[test]
    fn clear_screen_is_silent_when_not_interactive() {
        // The test harness captures stdout; a non-interactive term must not emit anything.
        clear_screen(&Term::plain(80));
    }
}
