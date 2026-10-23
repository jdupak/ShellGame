//! A rounded box with the title centred in the top border.
//!
//! ```text
//! ╭──────────────────── Title ─────────────────────╮
//! │                                                │
//! │  Body lines, padded to the inner width.        │
//! │                                                │
//! ╰────────────────────────────────────────────────╯
//! ```

use anstyle::Style;

use super::{Line, PLAIN, line_width, plain, span, split_at_width, truncate};

#[derive(Default)]
pub struct Panel {
    pub border: Style,
    pub title: Line,
    /// Blank rows above and below the body, blank columns left and right of it.
    pub pad: (usize, usize),
    pub center: bool,
}

impl Panel {
    /// Columns for the body in a panel `outer` columns wide.
    pub fn inner(&self, outer: usize) -> usize {
        outer.saturating_sub(2 + 2 * self.pad.1).max(1)
    }

    /// A body line wider than the panel (code is never wrapped) continues on further rows that
    /// keep its indentation.
    fn fold(mut line: Line, cols: usize) -> Vec<Line> {
        let text = plain(&line);
        let indent = (text.len() - text.trim_start_matches(' ').len()).min(cols / 2);
        let mut rows = Vec::new();
        while line_width(&line) > cols {
            let (head, tail) = split_at_width(line, cols);
            rows.push(head);
            line = [span(" ".repeat(indent), PLAIN)].into_iter().chain(tail).collect();
        }
        rows.push(line);
        rows
    }

    pub fn render(&self, body: Vec<Line>, outer: usize) -> Vec<Line> {
        let (outer, gap) = (outer.max(3), " ".repeat(self.pad.1));
        let inner = self.inner(outer);
        let border = |text: String| span(text, self.border);
        let line = |left: &str, fill: &str, right: &str| {
            vec![border(format!("{left}{}{right}", fill.repeat(outer - 2)))]
        };
        let blank = || vec![border(format!("│{}│", " ".repeat(outer - 2)))];
        let title = truncate(self.title.clone(), outer.saturating_sub(6));
        let mut top = line("╭", "─", "╮");
        if !title.is_empty() {
            let fill = outer - 4 - (line_width(&title) + 2);
            top = vec![border(format!("╭─{} ", "─".repeat(fill / 2)))];
            top.extend(title);
            top.push(border(format!(" {}─╮", "─".repeat(fill - fill / 2))));
        }
        let mut out = vec![top];
        out.extend((0..self.pad.0).map(|_| blank()));
        for row in body.into_iter().flat_map(|line| Panel::fold(line, inner)) {
            let missing = inner - line_width(&row);
            let left = if self.center { missing / 2 } else { 0 };
            let mut framed = vec![border(format!("│{gap}{}", " ".repeat(left)))];
            framed.extend(row);
            framed.push(border(format!("{}{gap}│", " ".repeat(missing - left))));
            out.push(framed);
        }
        out.extend((0..self.pad.0).map(|_| blank()));
        out.push(line("╰", "─", "╯"));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{BOLD, CYAN};

    fn text(lines: Vec<Line>) -> Vec<String> {
        lines.iter().map(|line| plain(line)).collect()
    }

    #[test]
    fn every_row_is_as_wide_as_the_panel_and_a_long_line_keeps_its_indent() {
        let long = vec![span("  0123456789abcdef0123456789abcdef", PLAIN)];
        let body = vec![vec![span("Příliš žluťoučký kůň 💡 ✓", PLAIN)], Line::new(), long];
        let title = vec![span("1.2: Velmi dlouhý název, který se nevejde", BOLD)];
        let panel = Panel { border: CYAN, title, pad: (1, 2), ..Panel::default() };
        for outer in [24, 40, 61] {
            assert!(panel.render(body.clone(), outer).iter().all(|row| line_width(row) == outer));
        }
        let panel = Panel { pad: (0, 1), ..Panel::default() };
        let rows = text(panel.render(vec![vec![span("  0123456789abcdef", PLAIN)]], 18));
        assert_eq!(rows[1..3], ["│   0123456789ab │", "│   cdef         │"]);
    }
}
