//! CommonMark for the terminal, built on [`pulldown_cmark`].
//!
//! The level texts are written for Rich: paragraphs are wrapped by hand in the source and are
//! expected to reflow, headings are left-aligned, and code blocks hold box-drawing trees that
//! must survive untouched. The renderer therefore works in two steps. The parser events are
//! first collected into a small block tree; the tree is then laid out for a given
//! width, which makes hanging indents trivial: a list item or block quote simply lays out its
//! children at a smaller width and prefixes the resulting lines.
//!
//! | Markdown | Rendering |
//! |---|---|
//! | paragraph | reflowed; soft breaks become spaces, hard breaks stay |
//! | `#`, `##` | the heading in bold underline followed by `────` to the full width |
//! | `###` and deeper | the heading text in bold (h4 dim, h5/h6 italic) |
//! | `*em*`, `**strong**`, `` `code` `` | italic, bold, bold cyan; a code span is not split across lines while it fits on one |
//! | `- item` | ` • ` marker in bold yellow, continuation lines indented under the text |
//! | `1. item` | right-aligned numbers, same hanging indent; nested lists indent further |
//! | fenced or indented code | verbatim, never wrapped, indented by two spaces |
//! | `> quote` | every line prefixed with `▌ `, text in magenta |
//!
//! Blocks are separated by exactly one blank line, items of a list by none. Raw HTML is shown as
//! text. Control characters in the input are replaced, so a document can never write escape
//! sequences of its own, and `⚠️` is shown as the bare `⚠` (see
//! [`normalize_emoji`](super::text::normalize_emoji)). Tables and the other CommonMark extensions
//! are not enabled.

use anstyle::Style;
use pulldown_cmark::{Event, HeadingLevel, LinkType, Parser, Tag, TagEnd};

use super::Term;
use super::style::{self, combine};
use super::text::{
    Frag, Inline, TAB_WIDTH, expand_tabs, frags_width, layout, push_styled, render_frags, sanitize,
    tokenize,
};

const BULLET: &str = " • ";
const BULLET_WIDTH: usize = 3;
const QUOTE_BAR: &str = "▌";
const RULE_CHAR: &str = "─";
const CODE_INDENT: &str = "  ";
/// Separator line of intro pages, as in the Python `raw.split("\n---\n")`.
const PAGE_SEPARATOR: &str = "\n---\n";

// ---------------------------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------------------------

/// Splits intro text into pages at lines that consist of exactly `---`.
///
/// This is the Python `raw.split("\n---\n")` followed by trimming each page and dropping empty
/// ones. Text without a separator is a single page, and blank text yields one empty page rather
/// than none, so a caller can always show at least one panel.
///
/// ```
/// use shellgame::ui::split_pages;
///
/// assert_eq!(split_pages("První\n\n---\n\nDruhá\n"), ["První", "Druhá"]);
/// assert_eq!(split_pages("Jedna"), ["Jedna"]);
/// assert_eq!(split_pages("  \n"), [""]);
/// ```
#[must_use]
pub fn split_pages(text: &str) -> Vec<String> {
    let normalized = text.replace("\r\n", "\n");
    let pages: Vec<String> = normalized
        .split(PAGE_SEPARATOR)
        .map(str::trim)
        .filter(|page| !page.is_empty())
        .map(str::to_owned)
        .collect();
    if pages.is_empty() {
        vec![normalized.trim().to_owned()]
    } else {
        pages
    }
}

/// Renders CommonMark to terminal lines, `term.width` columns wide.
///
/// Every line is at most `term.width` visible columns wide, with one exception: lines of code
/// blocks are never wrapped, so they can be wider (a [`Panel`](super::Panel) folds them). With
/// `term.color` off the lines contain no escape sequences; with it on each line is
/// self-contained.
///
/// ```
/// use shellgame::ui::{Term, render_markdown};
///
/// let md = "Přejděte\ndo adresáře `start`\n\n- první\n- druhá\n";
/// assert_eq!(
///     render_markdown(md, &Term::plain(40)),
///     ["Přejděte do adresáře start", "", " • první", " • druhá"],
/// );
/// ```
#[must_use]
pub fn render_markdown(md: &str, term: &Term) -> Vec<String> {
    let blocks = parse(md);
    Renderer { color: term.color }.blocks(&blocks, term.width.max(1), false)
}

// ---------------------------------------------------------------------------------------------
// Block tree
// ---------------------------------------------------------------------------------------------

/// A block-level element with its inline content already styled.
#[derive(Debug)]
enum Block {
    Paragraph(Vec<Inline>),
    Heading {
        level: HeadingLevel,
        inlines: Vec<Inline>,
    },
    Code(String),
    Html(String),
    List {
        start: Option<u64>,
        items: Vec<Item>,
    },
    Quote(Vec<Block>),
    Rule,
}

/// A list item. `loose` is set when the item contained explicit paragraphs, which means the
/// source separated its blocks with blank lines.
#[derive(Debug)]
struct Item {
    blocks: Vec<Block>,
    loose: bool,
}

fn parse(md: &str) -> Vec<Block> {
    let mut builder = Builder {
        events: Parser::new(md).collect(),
        pos: 0,
    };
    builder.blocks(Style::new()).0
}

fn heading_style(level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => style::H1,
        HeadingLevel::H2 => style::H2,
        HeadingLevel::H3 => style::H3,
        HeadingLevel::H4 => style::H4,
        HeadingLevel::H5 => style::H5,
        HeadingLevel::H6 => style::H6,
    }
}

/// Text of an inline run: control characters are replaced and tabs or newlines become spaces.
fn clean_inline(text: &str) -> String {
    sanitize(text).replace(['\t', '\n'], " ")
}

/// Breakable inline text.
fn inline_text(text: &str, style: Style) -> Inline {
    Inline::Text(Frag::new(clean_inline(text), style))
}

/// An inline code span: kept in one piece where possible so a command is not split in two.
fn inline_code(text: &str, style: Style) -> Inline {
    Inline::Unbreakable(Frag::new(clean_inline(text), style))
}

fn is_inline_tag(tag: &Tag<'_>) -> bool {
    matches!(
        tag,
        Tag::Emphasis
            | Tag::Strong
            | Tag::Strikethrough
            | Tag::Superscript
            | Tag::Subscript
            | Tag::Link { .. }
            | Tag::Image { .. }
    )
}

/// The target shown after a link, unless the link text already is the target.
fn link_target(link_type: LinkType, dest: &str) -> Option<String> {
    match link_type {
        LinkType::Autolink | LinkType::Email => None,
        _ if dest.is_empty() => None,
        _ => Some(dest.to_owned()),
    }
}

/// Builds the block tree from parser events.
///
/// The events are collected first so the builder can look ahead and step back freely.
struct Builder<'a> {
    events: Vec<Event<'a>>,
    pos: usize,
}

impl<'a> Builder<'a> {
    fn peek(&self) -> Option<Event<'a>> {
        self.events.get(self.pos).cloned()
    }

    /// Consumes the closing event of the container whose content was just parsed.
    fn finish(&mut self) {
        if matches!(self.events.get(self.pos), Some(Event::End(_))) {
            self.pos += 1;
        }
    }

    /// Collects the literal text of a code or HTML block up to its end tag.
    fn verbatim(&mut self, end: TagEnd) -> String {
        let mut text = String::new();
        while let Some(event) = self.events.get(self.pos) {
            self.pos += 1;
            match event {
                Event::End(tag_end) if *tag_end == end => break,
                Event::Text(chunk) | Event::Html(chunk) | Event::InlineHtml(chunk) => {
                    text.push_str(chunk);
                }
                _ => {}
            }
        }
        text
    }

    /// Parses blocks until the end of the enclosing container (not consumed) or of the input.
    ///
    /// The flag is true when the content had explicit paragraphs, which marks a loose list item.
    fn blocks(&mut self, base: Style) -> (Vec<Block>, bool) {
        let mut blocks = Vec::new();
        let mut loose = false;
        while let Some(event) = self.peek() {
            let before = self.pos;
            match event {
                Event::End(_) => break,
                Event::Rule => {
                    self.pos += 1;
                    blocks.push(Block::Rule);
                }
                Event::Start(Tag::Paragraph) => {
                    self.pos += 1;
                    let inlines = self.inlines(base, Some(TagEnd::Paragraph));
                    blocks.push(Block::Paragraph(inlines));
                    loose = true;
                }
                Event::Start(Tag::Heading { level, .. }) => {
                    self.pos += 1;
                    let style = combine(base, heading_style(level));
                    let inlines = self.inlines(style, Some(TagEnd::Heading(level)));
                    blocks.push(Block::Heading { level, inlines });
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    self.pos += 1;
                    let (inner, _) = self.blocks(combine(base, style::BLOCKQUOTE));
                    self.finish();
                    blocks.push(Block::Quote(inner));
                }
                Event::Start(Tag::CodeBlock(_)) => {
                    self.pos += 1;
                    blocks.push(Block::Code(self.verbatim(TagEnd::CodeBlock)));
                }
                Event::Start(Tag::HtmlBlock) => {
                    self.pos += 1;
                    blocks.push(Block::Html(self.verbatim(TagEnd::HtmlBlock)));
                }
                Event::Start(Tag::List(start)) => {
                    self.pos += 1;
                    let mut items = Vec::new();
                    while matches!(self.events.get(self.pos), Some(Event::Start(Tag::Item))) {
                        self.pos += 1;
                        let (inner, item_loose) = self.blocks(base);
                        self.finish();
                        items.push(Item {
                            blocks: inner,
                            loose: item_loose,
                        });
                    }
                    self.finish();
                    blocks.push(Block::List { start, items });
                }
                Event::Start(tag) if !is_inline_tag(&tag) => {
                    // Containers this renderer has no special look for (tables, footnotes,
                    // definition lists, … are not enabled, but stay harmless): keep the content.
                    self.pos += 1;
                    let (inner, inner_loose) = self.blocks(base);
                    self.finish();
                    blocks.extend(inner);
                    loose |= inner_loose;
                }
                _ => {
                    // Inline content outside any paragraph: tight list items.
                    let inlines = self.inlines(base, None);
                    if !inlines.is_empty() {
                        blocks.push(Block::Paragraph(inlines));
                    }
                }
            }
            if self.pos == before {
                self.pos += 1;
            }
        }
        (blocks, loose)
    }

    /// Parses inline events into styled text.
    ///
    /// With `end` the run stops after that end tag; without it, at the first block-level event.
    fn inlines(&mut self, base: Style, end: Option<TagEnd>) -> Vec<Inline> {
        let mut out = Vec::new();
        let mut styles = vec![base];
        let mut links: Vec<Option<String>> = Vec::new();

        while let Some(event) = self.peek() {
            let current = styles.last().copied().unwrap_or(base);
            match event {
                Event::End(tag_end) => {
                    if end == Some(tag_end) {
                        self.pos += 1;
                        break;
                    }
                    match tag_end {
                        TagEnd::Emphasis
                        | TagEnd::Strong
                        | TagEnd::Strikethrough
                        | TagEnd::Superscript
                        | TagEnd::Subscript => {
                            self.pos += 1;
                            pop_style(&mut styles);
                        }
                        TagEnd::Link | TagEnd::Image => {
                            self.pos += 1;
                            pop_style(&mut styles);
                            if let Some(Some(target)) = links.pop() {
                                let outer = styles.last().copied().unwrap_or(base);
                                out.push(inline_text(
                                    &format!(" ({target})"),
                                    combine(outer, style::MUTED),
                                ));
                            }
                        }
                        _ => break,
                    }
                }
                Event::Start(tag) => match tag {
                    Tag::Emphasis => {
                        self.pos += 1;
                        styles.push(combine(current, style::EMPHASIS));
                    }
                    Tag::Strong => {
                        self.pos += 1;
                        styles.push(combine(current, style::STRONG));
                    }
                    Tag::Strikethrough => {
                        self.pos += 1;
                        styles.push(combine(current, style::STRIKETHROUGH));
                    }
                    Tag::Superscript | Tag::Subscript => {
                        self.pos += 1;
                        styles.push(current);
                    }
                    Tag::Link {
                        link_type,
                        dest_url,
                        ..
                    } => {
                        self.pos += 1;
                        styles.push(combine(current, style::LINK));
                        links.push(link_target(link_type, &dest_url));
                    }
                    Tag::Image { dest_url, .. } => {
                        self.pos += 1;
                        styles.push(combine(current, style::EMPHASIS));
                        links.push((!dest_url.is_empty()).then(|| dest_url.to_string()));
                    }
                    _ => break,
                },
                Event::Text(text) => {
                    self.pos += 1;
                    out.push(inline_text(&text, current));
                }
                Event::Code(code) => {
                    self.pos += 1;
                    out.push(inline_code(&code, combine(current, style::INLINE_CODE)));
                }
                Event::SoftBreak => {
                    self.pos += 1;
                    out.push(inline_text(" ", current));
                }
                Event::HardBreak => {
                    self.pos += 1;
                    out.push(Inline::Hard);
                }
                Event::Html(text)
                | Event::InlineHtml(text)
                | Event::InlineMath(text)
                | Event::DisplayMath(text) => {
                    self.pos += 1;
                    out.push(inline_text(&text, current));
                }
                Event::FootnoteReference(label) => {
                    self.pos += 1;
                    out.push(inline_text(&format!("[{label}]"), current));
                }
                Event::TaskListMarker(done) => {
                    self.pos += 1;
                    out.push(inline_text(if done { "[x] " } else { "[ ] " }, current));
                }
                Event::Rule => break,
            }
        }
        out
    }
}

/// Leaves the base style in place when an end tag has no matching start (never expected).
fn pop_style(styles: &mut Vec<Style>) {
    if styles.len() > 1 {
        styles.pop();
    }
}

// ---------------------------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------------------------

/// Lays the block tree out as lines.
struct Renderer {
    color: bool,
}

impl Renderer {
    fn paint(&self, style: Style, text: &str) -> String {
        let mut out = String::new();
        push_styled(&mut out, style, text, self.color);
        out
    }

    /// Lays out sibling blocks, separated by a blank line unless `tight`.
    fn blocks(&self, blocks: &[Block], width: usize, tight: bool) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for block in blocks {
            let lines = self.block(block, width);
            if lines.is_empty() {
                continue;
            }
            if !out.is_empty() && !tight {
                out.push(String::new());
            }
            out.extend(lines);
        }
        out
    }

    fn block(&self, block: &Block, width: usize) -> Vec<String> {
        match block {
            Block::Paragraph(inlines) => self.paragraph(inlines, width),
            Block::Heading { level, inlines } => self.heading(*level, inlines, width),
            Block::Code(text) => self.code(text),
            Block::Html(text) => verbatim_lines(text),
            Block::List { start, items } => self.list(*start, items, width),
            Block::Quote(blocks) => self.quote(blocks, width),
            Block::Rule => vec![self.paint(style::MUTED, &RULE_CHAR.repeat(width))],
        }
    }

    fn paragraph(&self, inlines: &[Inline], width: usize) -> Vec<String> {
        layout(&tokenize(inlines), width)
            .iter()
            .map(|row| render_frags(row, self.color))
            .collect()
    }

    /// Level 1 and 2 headings are followed by a rule up to the full width, like Rich's
    /// `Rule(align="left")`; deeper levels are just styled text.
    fn heading(&self, level: HeadingLevel, inlines: &[Inline], width: usize) -> Vec<String> {
        let rows = layout(&tokenize(inlines), width);
        let with_rule = matches!(level, HeadingLevel::H1 | HeadingLevel::H2);
        let last = rows.len().saturating_sub(1);
        rows.iter()
            .enumerate()
            .map(|(index, row)| {
                let mut line = render_frags(row, self.color);
                let used = frags_width(row);
                if with_rule && index == last && used + 2 <= width {
                    line.push(' ');
                    push_styled(
                        &mut line,
                        style::HEADING_RULE,
                        &RULE_CHAR.repeat(width - used - 1),
                        self.color,
                    );
                }
                line
            })
            .collect()
    }

    /// Code is printed as written: not wrapped, only indented and styled per line.
    fn code(&self, text: &str) -> Vec<String> {
        verbatim_lines(text)
            .into_iter()
            .map(|line| {
                if line.is_empty() {
                    return line;
                }
                let mut out = String::from(CODE_INDENT);
                push_styled(&mut out, style::CODE_BLOCK, &line, self.color);
                out
            })
            .collect()
    }

    fn list(&self, start: Option<u64>, items: &[Item], width: usize) -> Vec<String> {
        // Like Rich: numbers are right-aligned to the width of the last one, plus a space on
        // each side.
        let last_number =
            start.map(|first| first.saturating_add(items.len().saturating_sub(1) as u64));
        let marker_width = last_number.map_or(BULLET_WIDTH, |last| last.to_string().len() + 2);
        let child_width = width.saturating_sub(marker_width).max(1);
        let hang = " ".repeat(marker_width);

        let mut out = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let marker = match start {
                None => BULLET.to_owned(),
                Some(first) => format!(
                    "{:>digits$} ",
                    first.saturating_add(index as u64),
                    digits = marker_width - 1
                ),
            };
            let body = self.blocks(&item.blocks, child_width, !item.loose);
            if body.is_empty() {
                out.push(self.paint(style::LIST_MARKER, marker.trim_end()));
                continue;
            }
            for (row, line) in body.into_iter().enumerate() {
                if row == 0 {
                    out.push(format!("{}{line}", self.paint(style::LIST_MARKER, &marker)));
                } else if line.is_empty() {
                    out.push(line);
                } else {
                    out.push(format!("{hang}{line}"));
                }
            }
        }
        out
    }

    fn quote(&self, blocks: &[Block], width: usize) -> Vec<String> {
        let inner = self.blocks(blocks, width.saturating_sub(2).max(1), false);
        let bar = self.paint(style::BLOCKQUOTE, &format!("{QUOTE_BAR} "));
        let bare = self.paint(style::BLOCKQUOTE, QUOTE_BAR);
        inner
            .into_iter()
            .map(|line| {
                if line.is_empty() {
                    bare.clone()
                } else {
                    format!("{bar}{line}")
                }
            })
            .collect()
    }
}

/// Lines of a raw block as they are: tabs expanded, control characters replaced, trailing spaces
/// and trailing blank lines removed.
fn verbatim_lines(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = text
        .lines()
        .map(|raw| {
            let expanded = expand_tabs(raw, TAB_WIDTH);
            sanitize(&expanded).trim_end().to_owned()
        })
        .collect();
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::text::{is_self_contained, visible_width};

    fn plain(md: &str, width: usize) -> Vec<String> {
        render_markdown(md, &Term::plain(width))
    }

    fn colored(md: &str, width: usize) -> Vec<String> {
        render_markdown(md, &Term::ansi(width))
    }

    // --- pages -----------------------------------------------------------------------------

    #[test]
    fn split_pages_splits_on_separator_lines() {
        assert_eq!(split_pages("a\n---\nb"), ["a", "b"]);
        assert_eq!(split_pages("a\n\n---\n\nb\n\n---\n\nc\n"), ["a", "b", "c"]);
    }

    #[test]
    fn split_pages_trims_and_drops_empty_pages() {
        assert_eq!(split_pages("  a  \n---\n \n---\n b\n\n"), ["a", "b"]);
        assert_eq!(split_pages("a\n---\n---\nb"), ["a", "---\nb"]);
    }

    #[test]
    fn split_pages_keeps_inline_dashes_and_edge_separators() {
        assert_eq!(split_pages("a --- b"), ["a --- b"]);
        assert_eq!(split_pages("---\na"), ["---\na"]);
        assert_eq!(split_pages("a\r\n---\r\nb"), ["a", "b"]);
    }

    #[test]
    fn split_pages_of_blank_text_is_one_empty_page() {
        assert_eq!(split_pages(""), [""]);
        assert_eq!(split_pages(" \n\n "), [""]);
    }

    // --- paragraphs ------------------------------------------------------------------------

    #[test]
    fn paragraph_lines_reflow() {
        let md = "Adresáře `start` a `finish` leží vedle sebe\nve stejném rodičovském adresáři.";
        assert_eq!(
            plain(md, 30),
            [
                "Adresáře start a finish leží",
                "vedle sebe ve stejném",
                "rodičovském adresáři."
            ]
        );
    }

    #[test]
    fn hard_breaks_stay() {
        assert_eq!(plain("jedna  \ndva\\\ntři", 40), ["jedna", "dva", "tři"]);
    }

    #[test]
    fn blocks_are_separated_by_one_blank_line() {
        assert_eq!(plain("a\n\n\n\nb\n\nc", 40), ["a", "", "b", "", "c"]);
        assert_eq!(plain("", 40), Vec::<String>::new());
        assert_eq!(plain("\n\n", 40), Vec::<String>::new());
    }

    #[test]
    fn punctuation_after_styled_text_never_wraps_away() {
        let md = "xxxxx **bold**, yyyy";
        let lines = plain(md, 12);
        assert_eq!(lines, ["xxxxx bold,", "yyyy"]);
    }

    #[test]
    fn long_words_are_split_between_characters() {
        assert_eq!(plain("abcdefghij", 4), ["abcd", "efgh", "ij"]);
    }

    #[test]
    fn inline_code_is_not_split_while_it_fits() {
        assert_eq!(
            plain("alpha beta `cd ../finish` gamma", 20),
            ["alpha beta", "cd ../finish gamma"]
        );
        // Wider than the line: it breaks at its spaces after all.
        assert_eq!(plain("`cd ../finish now`", 10), ["cd", "../finish", "now"]);
    }

    #[test]
    fn emphasis_and_code_keep_their_text() {
        assert_eq!(plain("a *b* **c** `d e` f", 40), ["a b c d e f"]);
    }

    #[test]
    fn raw_html_and_angle_brackets_are_text() {
        assert_eq!(
            plain("Použijte <příkaz> a <b>tučně</b>.", 60),
            ["Použijte <příkaz> a <b>tučně</b>."]
        );
        assert_eq!(
            plain("<div>\nahoj\n</div>", 60),
            ["<div>", "ahoj", "</div>"]
        );
    }

    #[test]
    fn escapes_and_entities_are_decoded() {
        assert_eq!(plain(r"1 \* 2 &amp; 3 &lt;4&gt;", 60), ["1 * 2 & 3 <4>"]);
    }

    #[test]
    fn control_characters_in_the_source_are_replaced() {
        let lines = plain("a \x1b[31mred\x1b[0m", 40);
        assert!(lines.iter().all(|line| !line.contains('\x1b')), "{lines:?}");
    }

    // --- headings --------------------------------------------------------------------------

    #[test]
    fn h1_and_h2_are_followed_by_a_rule_to_the_full_width() {
        assert_eq!(plain("# Cíl", 20), [format!("Cíl {}", "─".repeat(16))]);
        assert_eq!(
            plain("## Proč je to důležité?", 30),
            [format!("Proč je to důležité? {}", "─".repeat(9))]
        );
        for line in plain("# Nadpis\n\n## Druhý", 33)
            .iter()
            .filter(|line| !line.is_empty())
        {
            assert_eq!(visible_width(line), 33, "{line:?}");
        }
    }

    #[test]
    fn deeper_headings_have_no_rule() {
        assert_eq!(plain("### Cíl\n\ntext", 20), ["Cíl", "", "text"]);
        assert_eq!(plain("#### Bezpečnost", 20), ["Bezpečnost"]);
    }

    #[test]
    fn long_headings_wrap_and_the_rule_follows_the_last_row() {
        assert_eq!(
            plain("## Velmi dlouhý nadpis sekce", 14),
            ["Velmi dlouhý", "nadpis sekce ─"]
        );
        // No room for a rule at all: plain text.
        assert_eq!(plain("# Dvanáct znaků", 14), ["Dvanáct znaků"]);
    }

    #[test]
    fn setext_headings_work_too() {
        assert_eq!(
            plain("Název\n=====\n\ntext", 12),
            ["Název ──────", "", "text"]
        );
        assert_eq!(plain("Název\n-----", 12), ["Název ──────"]);
    }

    #[test]
    fn heading_styles_are_applied_per_word() {
        let lines = colored("### Dvě slova", 40);
        let open = style::H3.render().to_string();
        assert_eq!(lines, [format!("{open}Dvě slova\x1b[0m")]);
        let h1 = colored("# Dvě slova", 40);
        assert!(
            h1[0].starts_with(&style::H1.render().to_string()),
            "{}",
            h1[0]
        );
        assert!(h1[0].contains(&style::HEADING_RULE.render().to_string()));
    }

    // --- lists -----------------------------------------------------------------------------

    #[test]
    fn bullets_use_a_marker_and_a_hanging_indent() {
        let md = "- první položka je dost dlouhá na zalomení\n- druhá";
        assert_eq!(
            plain(md, 24),
            [
                " • první položka je dost",
                "   dlouhá na zalomení",
                " • druhá"
            ]
        );
    }

    #[test]
    fn numbered_lists_align_their_numbers() {
        let md = (1..=10)
            .map(|n| format!("{n}. položka {n}\n"))
            .collect::<String>();
        let lines = plain(&md, 40);
        assert_eq!(lines[0], "  1 položka 1");
        assert_eq!(lines[8], "  9 položka 9");
        assert_eq!(lines[9], " 10 položka 10");
    }

    #[test]
    fn short_numbered_lists_use_narrow_markers() {
        assert_eq!(plain("1. a\n2. b", 20), [" 1 a", " 2 b"]);
    }

    #[test]
    fn numbered_lists_can_start_anywhere() {
        assert_eq!(plain("5. pět\n6. šest", 20), [" 5 pět", " 6 šest"]);
    }

    #[test]
    fn numbered_items_wrap_under_the_text() {
        let md = "1. **Orientace**: Jak zjistit, kde se právě nacházíte. V grafickém rozhraní to vidíte.";
        assert_eq!(
            plain(md, 40),
            [
                " 1 Orientace: Jak zjistit, kde se právě",
                "   nacházíte. V grafickém rozhraní to",
                "   vidíte."
            ]
        );
    }

    #[test]
    fn lazy_continuation_lines_join_the_item() {
        let md =
            "5. Pokud si nevíte rady, můžete si vyžádat nápovědu příkazem:\n`shellgame hint`.\n";
        assert_eq!(
            plain(md, 80),
            [" 5 Pokud si nevíte rady, můžete si vyžádat nápovědu příkazem: shellgame hint."]
        );
    }

    #[test]
    fn nested_lists_indent_under_their_parent() {
        let md = "- rodič\n  - dítě jedna\n  - dítě dvě\n- druhý rodič";
        assert_eq!(
            plain(md, 40),
            [
                " • rodič",
                "    • dítě jedna",
                "    • dítě dvě",
                " • druhý rodič"
            ]
        );
    }

    #[test]
    fn nested_ordered_lists_inside_bullets() {
        let md = "- krok\n  1. jedna\n  2. dvě";
        assert_eq!(plain(md, 40), [" • krok", "    1 jedna", "    2 dvě"]);
    }

    #[test]
    fn items_may_contain_code_blocks_and_several_paragraphs() {
        let md = "1. Spusťte:\n   ```\n   ls -l\n   ```\n2. Hotovo";
        assert_eq!(plain(md, 40), [" 1 Spusťte:", "     ls -l", " 2 Hotovo"]);

        let loose = "- první odstavec\n\n  druhý odstavec\n- další";
        assert_eq!(
            plain(loose, 40),
            [" • první odstavec", "", "   druhý odstavec", " • další"]
        );
    }

    #[test]
    fn a_list_after_a_paragraph_is_a_separate_block() {
        assert_eq!(
            plain("Příkazy:\n- `pwd`\n- `ls`", 40),
            ["Příkazy:", "", " • pwd", " • ls"]
        );
    }

    #[test]
    fn empty_items_keep_their_marker() {
        assert_eq!(plain("-\n- x", 20), [" •", " • x"]);
    }

    #[test]
    fn list_markers_are_styled_bold_yellow() {
        let lines = colored("- x", 20);
        let marker = format!("{}{}\x1b[0m", style::LIST_MARKER.render(), BULLET);
        assert_eq!(lines, [format!("{marker}x")]);
    }

    // --- code ------------------------------------------------------------------------------

    #[test]
    fn code_blocks_are_verbatim_and_indented() {
        let md = "```\n/                          ← Kořen (root)\n├── home/                  ← Domovské adresáře\n│   └── student/\n```";
        assert_eq!(
            plain(md, 80),
            [
                "  /                          ← Kořen (root)",
                "  ├── home/                  ← Domovské adresáře",
                "  │   └── student/",
            ]
        );
    }

    #[test]
    fn code_blocks_are_never_wrapped() {
        let long = "x".repeat(100);
        let md = format!("```\n{long}\n```");
        assert_eq!(plain(&md, 20), [format!("  {long}")]);
    }

    #[test]
    fn code_blocks_keep_blank_lines_but_not_trailing_ones() {
        let md = "```\na\n\nb\n\n\n```\n\ntext";
        assert_eq!(plain(md, 20), ["  a", "", "  b", "", "text"]);
    }

    #[test]
    fn code_markup_is_not_interpreted() {
        let md = "```bash\n# komentář **ne tučně** `ne kód`\necho [ab]*.txt\n```";
        assert_eq!(
            plain(md, 60),
            ["  # komentář **ne tučně** `ne kód`", "  echo [ab]*.txt"]
        );
    }

    #[test]
    fn indented_code_blocks_work() {
        assert_eq!(
            plain("text\n\n    ls -l\n    pwd\n", 40),
            ["text", "", "  ls -l", "  pwd"]
        );
    }

    #[test]
    fn code_block_lines_expand_tabs_and_drop_trailing_spaces() {
        assert_eq!(plain("```\na\tb   \n```", 40), ["  a   b"]);
    }

    #[test]
    fn code_is_styled_line_by_line() {
        let lines = colored("```\na\n\nb\n```", 40);
        let open = style::CODE_BLOCK.render().to_string();
        assert_eq!(
            lines,
            [
                format!("  {open}a\x1b[0m"),
                String::new(),
                format!("  {open}b\x1b[0m")
            ]
        );
    }

    // --- quotes and rules ------------------------------------------------------------------

    #[test]
    fn block_quotes_get_a_bar_on_every_line() {
        let md = "> ⚠️ **Pozor na rozsahy:** `[a-z]` a `[A-Z]` vycházejí\n> z pořadí znaků.\n>\n> Druhý odstavec.";
        assert_eq!(
            plain(md, 30),
            [
                "▌ ⚠ Pozor na rozsahy: [a-z] a",
                "▌ [A-Z] vycházejí z pořadí",
                "▌ znaků.",
                "▌",
                "▌ Druhý odstavec.",
            ]
        );
    }

    #[test]
    fn quote_text_is_magenta_and_inline_styles_layer_on_top() {
        let lines = colored("> a `b`", 40);
        let magenta = style::BLOCKQUOTE.render().to_string();
        assert!(
            lines[0].starts_with(&format!("{magenta}▌ \x1b[0m")),
            "{lines:?}"
        );
        assert!(
            lines[0].contains(&format!("{magenta}a \x1b[0m")),
            "{lines:?}"
        );
        let code = combine(style::BLOCKQUOTE, style::INLINE_CODE)
            .render()
            .to_string();
        assert!(lines[0].contains(&format!("{code}b\x1b[0m")), "{lines:?}");
    }

    #[test]
    fn thematic_breaks_fill_the_width() {
        assert_eq!(plain("a\n\n***\n\nb", 6), ["a", "", "──────", "", "b"]);
    }

    // --- links -----------------------------------------------------------------------------

    #[test]
    fn links_show_their_target() {
        assert_eq!(
            plain("[Rust](https://rust-lang.org) je fajn", 60),
            ["Rust (https://rust-lang.org) je fajn"]
        );
        assert_eq!(
            plain("<https://rust-lang.org>", 60),
            ["https://rust-lang.org"]
        );
    }

    // --- width and colour invariants -------------------------------------------------------

    #[test]
    fn no_line_exceeds_the_width_except_code() {
        let md = "# Nadpis sekce s dlouhým názvem\n\nPříliš žluťoučký kůň úpěl ďábelské ódy, 💡 tip.\n\n1. První\n2. **Druhá** s `kódem` a delším textem\n   - vnořená položka s dlouhým textem\n\n> citace s dlouhým textem na více řádků\n";
        for width in [10, 20, 33, 40, 80] {
            for line in plain(md, width) {
                assert!(visible_width(&line) <= width, "{line:?} at {width}");
            }
        }
    }

    #[test]
    fn colour_off_emits_no_escape_sequences() {
        let md = "# H\n\n**b** *i* `c`\n\n- x\n\n> q\n\n```\ncode\n```";
        for line in plain(md, 30) {
            assert!(!line.contains('\x1b'), "{line:?}");
        }
    }

    #[test]
    fn every_coloured_line_is_self_contained() {
        let md = "# Nadpis\n\nSlova **tučně a *šikmo* dál** a `kód s mezerami` v odstavci, který se musí zalomit na několik řádků.\n\n- položka s `kódem` a **tučným** textem, který se také zalomí\n\n> citace s **tučným** textem na dva řádky, aspoň doufám\n\n```\nkód\n```";
        for width in [12, 20, 40] {
            for line in colored(md, width) {
                assert!(is_self_contained(&line), "{line:?} at {width}");
            }
        }
    }

    #[test]
    fn a_styled_span_that_wraps_is_restyled_on_each_row() {
        let lines = colored("**alfa beta gama**", 10);
        let open = style::STRONG.render().to_string();
        assert_eq!(
            lines,
            [
                format!("{open}alfa beta\x1b[0m"),
                format!("{open}gama\x1b[0m")
            ]
        );
    }

    #[test]
    fn nested_styles_combine() {
        let lines = colored("**a *b***", 40);
        let both = combine(style::STRONG, style::EMPHASIS).render().to_string();
        let strong = style::STRONG.render().to_string();
        assert_eq!(lines, [format!("{strong}a \x1b[0m{both}b\x1b[0m")]);
    }

    #[test]
    fn zero_and_tiny_widths_do_not_panic() {
        let md = "# H\n\n- a b c\n  - d\n\n> q\n\n```\ncode\n```\n\n1. x";
        for width in 0..6 {
            let _ = plain(md, width);
        }
    }
}
