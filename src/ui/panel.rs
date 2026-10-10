//! Rounded boxes with a centered title in the top border, like Rich's `Panel`.
//!
//! ```text
//! ╭────────────── 1.2: Výpis obsahu ──────────────╮
//! │                                                │
//! │  Body lines, padded by their visible width.    │
//! │                                                │
//! ╰────────────────────────────────────────────────╯
//! ```

use std::borrow::Cow;

use anstyle::Style;

use super::style;
use super::text::{
    center, fold_ansi, normalize_emoji, pad_to, strip_ansi, truncate, visible_width,
    with_base_style,
};
use super::{Term, markdown, text};

const TOP_LEFT: &str = "╭";
const TOP_RIGHT: &str = "╮";
const BOTTOM_LEFT: &str = "╰";
const BOTTOM_RIGHT: &str = "╯";
const HORIZONTAL: &str = "─";
const VERTICAL: &str = "│";

/// A rounded box around pre-rendered body lines.
///
/// The fields are public so a panel can be written as a struct literal; the builder methods are
/// a shorthand for the common cases.
///
/// ```
/// use shellgame::ui::{Panel, Term, style};
///
/// let panel = Panel::new(style::GREEN).padding(0, 1).centered();
/// let lines = panel.render(&["Hotovo".to_owned()], &Term::plain(20));
/// assert_eq!(lines, [
///     "╭──────────────────╮",
///     "│      Hotovo      │",
///     "╰──────────────────╯",
/// ]);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Panel {
    /// Text in the top border. It may contain ANSI escape sequences of its own, in which case
    /// [`Panel::title_style`] should usually be left plain. Newlines become spaces and a title
    /// that is too long is truncated with `…`.
    pub title: Option<String>,
    /// Style of the border characters.
    pub border: Style,
    /// Style applied to the whole title, re-applied after any reset inside it.
    pub title_style: Style,
    /// Blank rows above and below the body, and blank columns left and right of it, inside the
    /// border (the order matches Rich: `(vertical, horizontal)`).
    pub padding: (usize, usize),
    /// Center every body line instead of left-aligning it.
    pub center: bool,
}

impl Default for Panel {
    /// A plain, untitled, left-aligned panel with Rich's `(1, 2)` padding.
    fn default() -> Self {
        Self {
            title: None,
            border: style::PLAIN,
            title_style: style::PLAIN,
            padding: (1, 2),
            center: false,
        }
    }
}

impl Panel {
    /// A panel with the given border style and default padding.
    #[must_use]
    pub fn new(border: Style) -> Self {
        Self {
            border,
            ..Self::default()
        }
    }

    /// Sets the title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the title style.
    #[must_use]
    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    /// Sets vertical and horizontal padding.
    #[must_use]
    pub fn padding(mut self, vertical: usize, horizontal: usize) -> Self {
        self.padding = (vertical, horizontal);
        self
    }

    /// Centers every body line.
    #[must_use]
    pub fn centered(mut self) -> Self {
        self.center = true;
        self
    }

    /// Horizontal padding that still leaves at least one column for the body.
    fn horizontal_padding(&self, outer: usize) -> usize {
        self.padding.1.min(outer.saturating_sub(3) / 2)
    }

    /// Columns available to body lines in a panel as wide as `term`.
    ///
    /// Render markdown or wrap text to this width before calling [`Panel::render`].
    #[must_use]
    pub fn body_width(&self, term: &Term) -> usize {
        let outer = term.width.max(3);
        outer - 2 - 2 * self.horizontal_padding(outer)
    }

    /// Draws the panel around `body_lines`, `term.width` columns wide.
    ///
    /// Body lines may contain ANSI escape sequences and are padded by their visible width. A line
    /// wider than the body is folded onto continuation rows that keep its leading indentation
    /// (code blocks are the only content that is not wrapped before it gets here). When
    /// `term.color` is off, escape sequences in the body and the title are removed.
    #[must_use]
    pub fn render(&self, body_lines: &[String], term: &Term) -> Vec<String> {
        let outer = term.width.max(3);
        let pad_h = self.horizontal_padding(outer);
        let inner = outer - 2 - 2 * pad_h;
        let side = term.paint(self.border, VERTICAL);
        let margin = " ".repeat(pad_h);
        let blank = format!("{side}{}{side}", " ".repeat(outer - 2));

        let mut out = Vec::with_capacity(body_lines.len() + 2 * self.padding.0 + 2);
        out.push(self.top_border(term, outer));
        out.extend(std::iter::repeat_n(blank.clone(), self.padding.0));
        for line in body_lines {
            for logical in line.split('\n') {
                let logical: Cow<'_, str> = if term.color {
                    Cow::Borrowed(logical)
                } else {
                    strip_ansi(logical)
                };
                let logical = normalize_emoji(&logical);
                for row in self.rows(&logical, inner) {
                    let aligned = if self.center {
                        center(&row, inner)
                    } else {
                        pad_to(&row, inner)
                    };
                    out.push(format!("{side}{margin}{aligned}{margin}{side}"));
                }
            }
        }
        out.extend(std::iter::repeat_n(blank, self.padding.0));
        out.push(self.bottom_border(term, outer));
        out
    }

    /// Splits one logical body line into rows that fit `inner` columns.
    fn rows(&self, line: &str, inner: usize) -> Vec<String> {
        if visible_width(line) <= inner {
            return vec![line.to_owned()];
        }
        let indent = if self.center {
            0
        } else {
            leading_spaces(line).min(inner / 2)
        };
        fold_ansi(line, inner, indent)
    }

    /// Wraps markdown to the body width and draws the panel around it.
    #[must_use]
    pub fn render_markdown(&self, md: &str, term: &Term) -> Vec<String> {
        let body = markdown::render_markdown(md, &term.with_width(self.body_width(term)));
        self.render(&body, term)
    }

    /// Wraps plain text to the body width, styles it as a whole and draws the panel around it.
    ///
    /// The text is printed verbatim (see [`text::wrap_plain`]): brackets, asterisks and backticks
    /// are not interpreted. Pass [`style::PLAIN`] for unstyled text.
    #[must_use]
    pub fn render_text(&self, content: &str, text_style: Style, term: &Term) -> Vec<String> {
        let body = text::wrap_plain(content, self.body_width(term));
        self.render(&term.paint_lines(text_style, &body), term)
    }

    fn top_border(&self, term: &Term, outer: usize) -> String {
        let plain = || {
            term.paint(
                self.border,
                &format!("{TOP_LEFT}{}{TOP_RIGHT}", HORIZONTAL.repeat(outer - 2)),
            )
        };
        let Some(title) = self.title.as_deref().filter(|title| !title.is_empty()) else {
            return plain();
        };
        // "╭─" + fill + " " + title + " " + fill + "─╮": the title and its two spaces live in
        // the `outer - 4` columns between the corner pieces.
        let Some(room) = outer.checked_sub(6).filter(|room| *room > 0) else {
            return plain();
        };

        let title = title.replace('\n', " ");
        let title = if term.color {
            Cow::Borrowed(title.as_str())
        } else {
            strip_ansi(&title)
        };
        let title = truncate(&normalize_emoji(&title), room);
        let excess = outer - 4 - (visible_width(&title) + 2);
        let left = excess / 2;
        let right = excess - left;
        let title = if term.color {
            with_base_style(&title, self.title_style)
        } else {
            title
        };

        let mut line = term.paint(
            self.border,
            &format!("{TOP_LEFT}{HORIZONTAL}{}", HORIZONTAL.repeat(left)),
        );
        line.push(' ');
        line.push_str(&title);
        line.push(' ');
        line.push_str(&term.paint(
            self.border,
            &format!("{}{HORIZONTAL}{TOP_RIGHT}", HORIZONTAL.repeat(right)),
        ));
        line
    }

    fn bottom_border(&self, term: &Term, outer: usize) -> String {
        term.paint(
            self.border,
            &format!(
                "{BOTTOM_LEFT}{}{BOTTOM_RIGHT}",
                HORIZONTAL.repeat(outer - 2)
            ),
        )
    }
}

/// Number of spaces a line starts with, not counting escape sequences.
fn leading_spaces(line: &str) -> usize {
    let visible = strip_ansi(line);
    visible.len() - visible.trim_start_matches(' ').len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::style::{BOLD_CYAN, CYAN, DIM};
    use crate::ui::text::is_self_contained;

    fn widths(lines: &[String]) -> Vec<usize> {
        lines.iter().map(|line| visible_width(line)).collect()
    }

    #[test]
    fn draws_a_titled_panel_like_rich() {
        let panel = Panel::new(CYAN).title("Nápověda");
        let lines = panel.render(&["Ahoj".to_owned()], &Term::plain(24));
        assert_eq!(
            lines,
            [
                "╭────── Nápověda ──────╮",
                "│                      │",
                "│  Ahoj                │",
                "│                      │",
                "╰──────────────────────╯",
            ]
        );
    }

    #[test]
    fn untitled_panel_has_a_plain_top_border() {
        let lines = Panel::default().render(&[], &Term::plain(10));
        assert_eq!(
            lines,
            ["╭────────╮", "│        │", "│        │", "╰────────╯"]
        );
    }

    #[test]
    fn odd_margins_put_the_extra_column_on_the_right() {
        let lines = Panel::new(CYAN)
            .title("abc")
            .padding(0, 0)
            .render(&[], &Term::plain(14));
        // 10 columns between the corner pieces hold " abc " and 5 spare dashes: 2 left, 3 right.
        assert_eq!(lines[0], "╭─── abc ────╮");
    }

    #[test]
    fn every_line_has_the_same_visible_width_with_diacritics_and_emoji() {
        let body = vec![
            "Příliš žluťoučký kůň úpěl ďábelské ódy".to_owned(),
            "💡 Nápověda ✅ ❌ 🏆 🚪 ⚠\u{fe0f} ✓".to_owned(),
            "├── home/   ← Domovské adresáře".to_owned(),
            String::new(),
        ];
        for width in [40, 61, 80] {
            let panel = Panel::new(CYAN).title("💡 Nápověda 1 z 3");
            let lines = panel.render(&body, &Term::plain(width));
            assert!(
                widths(&lines).iter().all(|w| *w == width),
                "{:?}",
                widths(&lines)
            );
        }
    }

    #[test]
    fn ansi_body_lines_are_padded_by_visible_width() {
        let body = vec![
            format!(
                "{}červená{}",
                style::RED.render(),
                style::RED.render_reset()
            ),
            "plain".to_owned(),
        ];
        let lines = Panel::new(CYAN).render(&body, &Term::ansi(30));
        assert!(
            widths(&lines).iter().all(|w| *w == 30),
            "{:?}",
            widths(&lines)
        );
    }

    #[test]
    fn colour_off_strips_escape_sequences_from_body_and_title() {
        let body = vec!["\x1b[31mred\x1b[0m".to_owned()];
        let panel = Panel::new(CYAN).title("\x1b[1mTitle\x1b[0m");
        let lines = panel.render(&body, &Term::plain(30));
        assert!(lines.iter().all(|line| !line.contains('\x1b')), "{lines:?}");
        assert!(lines[0].contains(" Title "));
        assert!(lines[2].contains("red"));
    }

    #[test]
    fn colour_on_styles_borders_and_title_separately() {
        let panel = Panel::new(CYAN).title("T").title_style(BOLD_CYAN);
        let lines = panel.render(&["x".to_owned()], &Term::ansi(20));
        let cyan = CYAN.render().to_string();
        let bold_cyan = BOLD_CYAN.render().to_string();
        assert!(lines[0].starts_with(&cyan));
        assert!(lines[0].contains(&format!("{bold_cyan}T\x1b[0m")));
        assert!(lines[2].starts_with(&format!("{cyan}│\x1b[0m")));
        assert!(lines[2].ends_with(&format!("{cyan}│\x1b[0m")));
    }

    #[test]
    fn every_coloured_line_is_self_contained() {
        let panel = Panel::new(CYAN).title("Title").title_style(BOLD_CYAN);
        let body = vec![format!("{}dim{}", DIM.render(), DIM.render_reset())];
        for line in panel.render(&body, &Term::ansi(30)) {
            assert!(is_self_contained(&line), "{line:?}");
        }
    }

    #[test]
    fn centered_panel_centers_each_line() {
        let panel = Panel::new(CYAN).padding(0, 1).centered();
        let body = vec!["a".to_owned(), "abcd".to_owned()];
        let lines = panel.render(&body, &Term::plain(10));
        assert_eq!(
            lines,
            ["╭────────╮", "│   a    │", "│  abcd  │", "╰────────╯"]
        );
    }

    #[test]
    fn long_titles_are_truncated_with_an_ellipsis() {
        let panel = Panel::new(CYAN).title("Velmi dlouhý název panelu, který se nevejde");
        let lines = panel.render(&[], &Term::plain(24));
        assert!(lines[0].contains('…'), "{}", lines[0]);
        assert_eq!(visible_width(&lines[0]), 24);
    }

    #[test]
    fn tiny_widths_do_not_panic() {
        for width in 0..12 {
            let panel = Panel::new(CYAN).title("Titulek").padding(2, 5);
            let lines = panel.render(&["tělo panelu".to_owned()], &Term::plain(width));
            let expected = width.max(3);
            assert!(
                widths(&lines).iter().all(|w| *w == expected),
                "{width}: {lines:?}"
            );
        }
    }

    #[test]
    fn overlong_lines_are_folded_and_keep_their_indent() {
        let body = vec!["  0123456789abcdef".to_owned()];
        let panel = Panel::new(CYAN).padding(0, 1);
        let lines = panel.render(&body, &Term::plain(14));
        assert_eq!(
            lines,
            [
                "╭────────────╮",
                "│   01234567 │",
                "│   89abcdef │",
                "╰────────────╯",
            ]
        );
    }

    #[test]
    fn folding_keeps_styles_closed_on_every_row() {
        let long = format!("{}{}{}", CYAN.render(), "x".repeat(40), CYAN.render_reset());
        let lines = Panel::new(style::RED)
            .padding(0, 1)
            .render(&[long], &Term::ansi(20));
        assert!(lines.len() > 3);
        for line in &lines[1..lines.len() - 1] {
            assert_eq!(line.matches("\x1b[0m").count(), 3, "{line:?}");
        }
        assert!(widths(&lines).iter().all(|w| *w == 20));
    }

    #[test]
    fn embedded_newlines_become_separate_rows() {
        let lines = Panel::default()
            .padding(0, 0)
            .render(&["a\nb".to_owned()], &Term::plain(6));
        assert_eq!(lines, ["╭────╮", "│a   │", "│b   │", "╰────╯"]);
    }

    #[test]
    fn body_width_accounts_for_border_and_padding() {
        let panel = Panel::new(CYAN);
        assert_eq!(panel.body_width(&Term::plain(80)), 74);
        assert_eq!(
            Panel::new(CYAN).padding(1, 4).body_width(&Term::plain(80)),
            70
        );
        assert_eq!(
            Panel::new(CYAN).padding(0, 0).body_width(&Term::plain(80)),
            78
        );
        assert_eq!(panel.body_width(&Term::plain(3)), 1);
    }

    #[test]
    fn render_text_wraps_verbatim_text_to_the_body() {
        let panel = Panel::new(CYAN).padding(0, 1);
        let lines = panel.render_text(
            "použijte `ls [ab]*.txt` zde",
            style::PLAIN,
            &Term::plain(20),
        );
        assert_eq!(
            lines,
            [
                "╭──────────────────╮",
                "│ použijte `ls     │",
                "│ [ab]*.txt` zde   │",
                "╰──────────────────╯",
            ]
        );
    }

    #[test]
    fn render_markdown_fits_the_body() {
        let panel = Panel::new(CYAN).padding(0, 1);
        let lines = panel.render_markdown("Jedna dva tři čtyři pět šest sedm.", &Term::plain(20));
        assert!(widths(&lines).iter().all(|w| *w == 20));
        assert!(lines.len() > 3);
    }
}
