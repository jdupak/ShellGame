//! CommonMark for the terminal, laid out for a width in one pass over the parser's events.
//!
//! Paragraphs reflow (the sources break lines by hand), headings are left-aligned (levels 1 and 2
//! are followed by a rule), list items hang under their text, code blocks are verbatim and never
//! wrapped (the panel folds them), quotes get a `▌` bar. Blocks are separated by one blank line,
//! items of a list by none. Raw HTML is shown as text; tables and other extensions are not enabled.
//! A thematic break (`---`, which splits an intro into [`pages`]) draws nothing, a hard line break
//! is a space.

use anstyle::Style;
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

use super::{
    BOLD, BOLD_CYAN, BOLD_YELLOW, CYAN, DIM_BOLD, Line, Piece, Span, line_width, reflow, span,
    whole, width, words,
};

const MAGENTA: Style = anstyle::AnsiColor::Magenta.on_default();

/// Pages of an intro: split at lines of `---`, trimmed, empty ones dropped.
pub fn pages(text: &str) -> Vec<&str> {
    text.split("\n---\n").map(str::trim).filter(|page| !page.is_empty()).collect()
}

/// `top` over `base`: its colour wins where it has one, effects add up.
fn layer(base: Style, top: Style) -> Style {
    Style::new()
        .fg_color(top.get_fg_color().or(base.get_fg_color()))
        .effects(base.get_effects() | top.get_effects())
}

/// An enclosing list item or quote: what starts its first line, and what every later line.
struct Container {
    marker: Option<Span>,
    hang: Span,
    quote: bool,
}

#[derive(Default)]
struct Renderer {
    cols: usize,
    out: Vec<Line>,
    stack: Vec<Container>,
    styles: Vec<Style>,
    pieces: Vec<Piece>,
    /// The next number of each open list (`None` for bullets).
    lists: Vec<Option<u64>>,
    code: Option<String>,
    heading: Option<HeadingLevel>,
    /// A blank line is due before the next block.
    gap: bool,
}

pub fn render(md: &str, cols: usize) -> Vec<Line> {
    let mut r = Renderer { cols, styles: vec![Style::new()], ..Renderer::default() };
    Parser::new(md).for_each(|event| r.event(event));
    r.out
}

impl Renderer {
    fn style(&self) -> Style {
        self.styles.last().copied().unwrap_or_default()
    }

    fn room(&self) -> usize {
        let indent: usize = self.stack.iter().map(|c| width(&c.hang.text)).sum();
        self.cols.saturating_sub(indent).max(1)
    }

    fn blank(&mut self) {
        let bars = self.stack.iter().filter(|c| c.quote);
        self.out.push(bars.map(|c| span(c.hang.text.trim_end(), c.hang.style)).collect());
    }

    fn separate(&mut self) {
        if std::mem::take(&mut self.gap) {
            self.blank();
        }
    }

    fn emit(&mut self, lines: Vec<Line>) {
        for line in lines {
            let mut full: Line = self
                .stack
                .iter_mut()
                .map(|c| c.marker.take().unwrap_or_else(|| c.hang.clone()))
                .collect();
            full.extend(line);
            self.out.push(full);
        }
    }

    /// Lays the collected inline text out as a paragraph or heading. Text of a tight list item is
    /// flushed without a gap after it, so a nested list follows directly.
    fn flush(&mut self, gap_after: bool) {
        if self.pieces.is_empty() {
            return;
        }
        self.separate();
        let cols = self.room();
        let mut lines = reflow(&std::mem::take(&mut self.pieces), cols);
        if let (Some(HeadingLevel::H1 | HeadingLevel::H2), Some(last)) =
            (self.heading.take(), lines.last_mut())
        {
            let used = line_width(last);
            if used + 2 <= cols {
                last.extend([span(" ", Style::new()), span("─".repeat(cols - used - 1), BOLD)]);
            }
        }
        self.emit(lines);
        self.gap = gap_after;
    }

    fn event(&mut self, event: Event) {
        let style = self.style();
        match event {
            Event::End(TagEnd::Paragraph) => self.flush(true),
            Event::Start(Tag::Heading { level, .. }) => {
                self.heading = Some(level);
                self.styles.push(layer(style, heading_style(level)));
            }
            Event::End(TagEnd::Heading(_)) => {
                self.styles.pop();
                self.flush(true);
            }
            Event::Start(Tag::BlockQuote(_)) => {
                self.flush(false);
                self.separate();
                let bar = span("▌ ", MAGENTA);
                self.stack.push(Container { marker: None, hang: bar, quote: true });
                self.styles.push(layer(style, MAGENTA));
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                self.flush(false);
                self.stack.pop();
                self.styles.pop();
                self.gap = true;
            }
            Event::Start(Tag::List(first)) => {
                self.flush(false);
                self.separate();
                self.lists.push(first);
            }
            Event::End(TagEnd::List(_)) => {
                self.lists.pop();
                self.gap = true;
            }
            Event::Start(Tag::Item) => {
                let next = self.lists.last_mut().and_then(Option::as_mut);
                let marker =
                    next.as_deref().map_or(" • ".to_owned(), |number| format!(" {number} "));
                next.into_iter().for_each(|number| *number += 1);
                self.gap = false;
                let hang = span(" ".repeat(marker.chars().count()), Style::new());
                let marker = Some(span(marker, BOLD_YELLOW));
                self.stack.push(Container { marker, hang, quote: false });
            }
            Event::End(TagEnd::Item) => {
                self.flush(false);
                if let Some(Container { marker: Some(marker), .. }) = self.stack.pop() {
                    self.out.push(vec![span(marker.text.trim_end(), marker.style)]);
                }
            }
            Event::Start(Tag::CodeBlock(_) | Tag::HtmlBlock) => {
                self.flush(false);
                self.separate();
                self.code = Some(String::new());
            }
            Event::End(TagEnd::CodeBlock | TagEnd::HtmlBlock) => {
                let text = self.code.take().unwrap_or_default();
                let mut lines: Vec<&str> = text.lines().map(str::trim_end).collect();
                while lines.last() == Some(&"") {
                    lines.pop();
                }
                let rows = lines.iter().map(|line| {
                    if line.is_empty() {
                        Line::new()
                    } else {
                        vec![span(format!("  {line}"), CYAN)]
                    }
                });
                self.emit(rows.collect());
                self.gap = true;
            }
            Event::Start(Tag::Emphasis) => self.styles.push(layer(style, Style::new().italic())),
            Event::Start(Tag::Strong) => self.styles.push(layer(style, BOLD)),
            Event::End(TagEnd::Emphasis | TagEnd::Strong) => {
                self.styles.pop();
            }
            Event::Text(text) | Event::Html(text) | Event::InlineHtml(text) => match &mut self.code
            {
                Some(code) => code.push_str(&text),
                None => self.pieces.push(words(style, &text)),
            },
            Event::Code(code) => self.pieces.push(whole(layer(style, BOLD_CYAN), &code)),
            Event::SoftBreak | Event::HardBreak => self.pieces.push(words(style, " ")),
            _ => {}
        }
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 | HeadingLevel::H2 => BOLD.underline(),
        HeadingLevel::H3 => BOLD,
        _ => DIM_BOLD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::dedent;
    use crate::levels::COURSE;
    use crate::ui::plain;

    fn doc(md: &str, cols: usize) -> String {
        render(md, cols).iter().map(|line| plain(line)).collect::<Vec<_>>().join("|")
    }

    #[test]
    fn pages_are_split_at_rules_and_lists_wrap_under_their_marker() {
        assert_eq!(pages("a\n\n---\n\nb\n---\n \n---\nc"), ["a", "b", "c"]);
        let loose = "- jedna dlouhá položka\n\n  druhý odstavec\n- další";
        assert_eq!(doc(loose, 14), " • jedna|   dlouhá|   položka||   druhý|   odstavec| • další");
    }

    #[test]
    fn nothing_but_code_is_wider_than_the_width() {
        let levels =
            COURSE.iter().flat_map(|section| section.levels.iter().map(|level| level.text));
        let texts = COURSE.iter().map(|section| section.intro).chain(levels);
        for page in texts
            .flat_map(|text| pages(&dedent(text)).into_iter().map(String::from).collect::<Vec<_>>())
        {
            for cols in [40, 80] {
                for line in render(&page, cols)
                    .iter()
                    .filter(|line| !line.iter().any(|span| span.style == CYAN))
                {
                    assert!(line_width(line) <= cols, "{:?} at {cols}", plain(line));
                }
            }
        }
    }
}
