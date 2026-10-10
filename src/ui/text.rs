//! Text measurement, padding and word wrapping.
//!
//! Everything here works on *visible* columns: ANSI escape sequences count as zero width and
//! characters are measured with [`unicode_width`], so Czech diacritics, box-drawing characters
//! and emoji line up. `⚠️` (an emoji presentation sequence of two code points) is the one case
//! where terminals disagree with the standard, so text that goes into a box is normalised to
//! the bare `⚠` first, see [`normalize_emoji`].
//!
//! Two layers live in this module:
//!
//! * the public helpers ([`visible_width`], [`pad_to`], [`center`], [`wrap_plain`], …) that the
//!   screens use directly, and
//! * a small styled-text layout engine (crate-private) shared by [`wrap_plain`] and the markdown
//!   renderer. Words are laid out *before* any escape sequence is written and every styled run
//!   of a line is closed on that same line, so a line never carries a style over to the next
//!   one, wherever it is wrapped.

use std::borrow::Cow;
use std::fmt::Write as _;
use std::iter;

use anstyle::Style;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Tab stop distance used when tabs are expanded.
pub const TAB_WIDTH: usize = 4;

const ESC: char = '\x1b';
const RESET: &str = "\x1b[0m";
const ZERO_WIDTH_JOINER: char = '\u{200d}';
const VARIATION_SELECTOR_16: char = '\u{fe0f}';

// ---------------------------------------------------------------------------------------------
// ANSI scanning
// ---------------------------------------------------------------------------------------------

/// A piece of a string that may contain terminal escape sequences.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Piece<'a> {
    /// A complete escape sequence (zero width).
    Escape(&'a str),
    /// Printable text without any escape sequence.
    Text(&'a str),
}

/// Splits `s` into escape sequences and the text between them.
fn scan(s: &str) -> impl Iterator<Item = Piece<'_>> {
    let mut rest = s;
    iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        let len = if rest.starts_with(ESC) {
            escape_len(rest)
        } else {
            rest.find(ESC).unwrap_or(rest.len())
        };
        let (head, tail) = rest.split_at(len);
        rest = tail;
        Some(if head.starts_with(ESC) {
            Piece::Escape(head)
        } else {
            Piece::Text(head)
        })
    })
}

/// Byte length of the escape sequence at the start of `s`, which must begin with ESC.
///
/// Understands CSI (`ESC [ … final`), string sequences such as OSC (`ESC ] … BEL` or `… ESC \`)
/// and two-byte escapes. Truncated sequences swallow the rest of the string.
fn escape_len(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut len = match bytes.get(1) {
        None => 1,
        Some(b'[') => bytes[2..]
            .iter()
            .position(|b| (0x40..=0x7e).contains(b))
            .map_or(bytes.len(), |pos| pos + 3),
        Some(b']' | b'P' | b'X' | b'^' | b'_') => {
            let mut i = 2;
            loop {
                match bytes.get(i) {
                    None => break bytes.len(),
                    Some(0x07) => break i + 1,
                    Some(0x1b) if bytes.get(i + 1) == Some(&b'\\') => break i + 2,
                    Some(_) => i += 1,
                }
            }
        }
        Some(_) => {
            // ESC, optional intermediate bytes, one final byte.
            let mut i = 1;
            while bytes.get(i).is_some_and(|b| (0x20..=0x2f).contains(b)) {
                i += 1;
            }
            (i + 1).min(bytes.len())
        }
    };
    while !s.is_char_boundary(len) {
        len += 1;
    }
    len
}

/// True for the SGR sequences that switch every attribute off (`ESC[0m`, `ESC[m`).
fn is_sgr_reset(escape: &str) -> bool {
    escape
        .strip_prefix("\x1b[")
        .and_then(|rest| rest.strip_suffix('m'))
        .is_some_and(|params| params.is_empty() || params.chars().all(|c| c == '0'))
}

/// True for any SGR (colour or attribute) sequence.
fn is_sgr(escape: &str) -> bool {
    escape.starts_with("\x1b[") && escape.ends_with('m')
}

/// Tracks which SGR sequences are active while a styled line is cut into several rows.
#[derive(Debug, Default)]
struct SgrState<'a> {
    active: Vec<&'a str>,
}

impl<'a> SgrState<'a> {
    fn apply(&mut self, escape: &'a str) {
        if is_sgr_reset(escape) {
            self.active.clear();
        } else if is_sgr(escape) {
            self.active.push(escape);
        }
    }

    /// Ends the current row: closes whatever is open so the row is self-contained.
    fn close(&self, row: &mut String) {
        if !self.active.is_empty() {
            row.push_str(RESET);
        }
    }

    /// Starts a continuation row with the styles that were open when the previous one ended.
    fn reopen(&self, row: &mut String) {
        for escape in &self.active {
            row.push_str(escape);
        }
    }
}

/// Removes ANSI escape sequences from `s`.
///
/// Borrows when there is nothing to remove.
///
/// ```
/// use shellgame::ui::text::strip_ansi;
///
/// assert_eq!(strip_ansi("\x1b[1;36mahoj\x1b[0m"), "ahoj");
/// ```
#[must_use]
pub fn strip_ansi(s: &str) -> Cow<'_, str> {
    if !s.contains(ESC) {
        return Cow::Borrowed(s);
    }
    Cow::Owned(
        scan(s)
            .filter_map(|piece| match piece {
                Piece::Text(text) => Some(text),
                Piece::Escape(_) => None,
            })
            .collect(),
    )
}

/// True when `line` leaves no colour or attribute switched on at its end.
///
/// Every line returned by the renderers satisfies this, which is what lets lines be printed,
/// indented or boxed independently of each other.
///
/// ```
/// use shellgame::ui::text::is_self_contained;
///
/// assert!(is_self_contained("\x1b[1mbold\x1b[0m and plain"));
/// assert!(!is_self_contained("\x1b[1mbold"));
/// ```
#[must_use]
pub fn is_self_contained(line: &str) -> bool {
    let mut open = false;
    for piece in scan(line) {
        if let Piece::Escape(escape) = piece {
            if is_sgr_reset(escape) {
                open = false;
            } else if is_sgr(escape) {
                open = true;
            }
        }
    }
    !open
}

// ---------------------------------------------------------------------------------------------
// Measuring
// ---------------------------------------------------------------------------------------------

/// Number of terminal columns `s` occupies, ignoring ANSI escape sequences.
///
/// Wide characters (CJK, most emoji) count as two columns, combining marks as zero, and emoji
/// presentation sequences such as `⚠️` (U+26A0 U+FE0F) as two.
///
/// ```
/// use shellgame::ui::text::visible_width;
///
/// assert_eq!(visible_width("Příliš žluťoučký"), 16);
/// assert_eq!(visible_width("\x1b[31mred\x1b[0m"), 3);
/// assert_eq!(visible_width("💡 tip"), 6);
/// assert_eq!(visible_width("⚠️"), 2);
/// ```
#[must_use]
pub fn visible_width(s: &str) -> usize {
    UnicodeWidthStr::width(&*strip_ansi(s))
}

/// Splits text into display clusters: a base character together with the zero-width characters
/// that follow it (combining marks, variation selectors, joiners) and emoji modifiers.
///
/// This is not full grapheme segmentation, but it never separates an emoji from its variation
/// selector or a letter from its accent, which is what hard-wrapping needs.
fn clusters(text: &str) -> impl Iterator<Item = &str> {
    let mut rest = text;
    iter::from_fn(move || {
        let mut chars = rest.char_indices();
        let (_, first) = chars.next()?;
        let mut end = first.len_utf8();
        let mut previous = first;
        for (index, c) in chars {
            if !extends_cluster(previous, c) {
                break;
            }
            end = index + c.len_utf8();
            previous = c;
        }
        let (head, tail) = rest.split_at(end);
        rest = tail;
        Some(head)
    })
}

fn extends_cluster(previous: char, c: char) -> bool {
    previous == ZERO_WIDTH_JOINER
        || c == ZERO_WIDTH_JOINER
        || c.width() == Some(0)
        || ('\u{1f3fb}'..='\u{1f3ff}').contains(&c)
}

// ---------------------------------------------------------------------------------------------
// Padding and alignment
// ---------------------------------------------------------------------------------------------

/// Pads `line` on the right with spaces up to `width` visible columns.
///
/// A line that is already `width` columns or wider is returned unchanged.
///
/// ```
/// use shellgame::ui::text::{pad_to, visible_width};
///
/// let padded = pad_to("\x1b[1mčau\x1b[0m", 8);
/// assert_eq!(visible_width(&padded), 8);
/// ```
#[must_use]
pub fn pad_to(line: &str, width: usize) -> String {
    let missing = width.saturating_sub(visible_width(line));
    format!("{line}{}", " ".repeat(missing))
}

/// Pads `line` on the left so it ends at column `width` (right alignment).
///
/// ```
/// use shellgame::ui::text::pad_start;
///
/// assert_eq!(pad_start("42", 5), "   42");
/// ```
#[must_use]
pub fn pad_start(line: &str, width: usize) -> String {
    let missing = width.saturating_sub(visible_width(line));
    format!("{}{line}", " ".repeat(missing))
}

/// Centers `line` in `width` columns.
///
/// Like Rich, the extra column of an odd margin goes to the right. The result is padded on both
/// sides, so it is exactly `width` columns wide whenever `line` is narrower.
///
/// ```
/// use shellgame::ui::text::center;
///
/// assert_eq!(center("ab", 7), "  ab   ");
/// ```
#[must_use]
pub fn center(line: &str, width: usize) -> String {
    let missing = width.saturating_sub(visible_width(line));
    let left = missing / 2;
    format!("{}{line}{}", " ".repeat(left), " ".repeat(missing - left))
}

// ---------------------------------------------------------------------------------------------
// Sanitising
// ---------------------------------------------------------------------------------------------

/// Prepares text for display inside a box: control characters (except `\n` and `\t`) become
/// U+FFFD and emoji are normalised with [`normalize_emoji`].
///
/// Text that is printed verbatim may contain file names or user names; this keeps a stray ESC
/// from being interpreted by the terminal.
#[must_use]
pub fn sanitize(text: &str) -> Cow<'_, str> {
    let is_bad = |c: char| c.is_control() && c != '\n' && c != '\t';
    let cleaned = if text.chars().any(is_bad) {
        Cow::Owned(
            text.chars()
                .map(|c| if is_bad(c) { '\u{fffd}' } else { c })
                .collect(),
        )
    } else {
        Cow::Borrowed(text)
    };
    if cleaned.contains(VARIATION_SELECTOR_16) {
        Cow::Owned(normalize_emoji(&cleaned).into_owned())
    } else {
        cleaned
    }
}

/// Removes the emoji variation selector U+FE0F after a one-column character.
///
/// Terminals disagree about `⚠️` (U+26A0 U+FE0F): the Unicode standard makes it a two-column
/// emoji, but several widely used terminals (VTE, the base of GNOME Terminal, among them) keep
/// the single column of the bare `⚠`. No width calculation is right for both, and a wrong one
/// pushes the right border of a box out of line. Text that goes into a box is therefore
/// normalised to the bare symbol, whose width is one everywhere. Emoji that are wide anyway,
/// such as `💡`, are left alone.
///
/// ```
/// use shellgame::ui::text::{normalize_emoji, visible_width};
///
/// assert_eq!(normalize_emoji("⚠\u{fe0f} pozor"), "⚠ pozor");
/// assert_eq!(visible_width(&normalize_emoji("⚠\u{fe0f}")), 1);
/// assert_eq!(normalize_emoji("💡\u{fe0f}"), "💡\u{fe0f}");
/// ```
#[must_use]
pub fn normalize_emoji(text: &str) -> Cow<'_, str> {
    if !text.contains(VARIATION_SELECTOR_16) {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut previous: Option<char> = None;
    for c in text.chars() {
        if c == VARIATION_SELECTOR_16 && previous.is_some_and(|p| p.width() == Some(1)) {
            continue;
        }
        out.push(c);
        previous = Some(c);
    }
    Cow::Owned(out)
}

/// Expands tab characters to spaces, with tab stops every `tab_width` columns.
#[must_use]
pub fn expand_tabs(line: &str, tab_width: usize) -> Cow<'_, str> {
    if !line.contains('\t') {
        return Cow::Borrowed(line);
    }
    let tab_width = tab_width.max(1);
    let mut out = String::with_capacity(line.len() + 8);
    let mut column = 0;
    for c in line.chars() {
        if c == '\t' {
            let spaces = tab_width - column % tab_width;
            out.extend(iter::repeat_n(' ', spaces));
            column += spaces;
        } else {
            out.push(c);
            column += c.width().unwrap_or(0);
        }
    }
    Cow::Owned(out)
}

// ---------------------------------------------------------------------------------------------
// Cutting styled lines
// ---------------------------------------------------------------------------------------------

/// Cuts a possibly styled line into rows of at most `width` visible columns.
///
/// Styles that are open at a cut are closed at the end of the row and re-opened at the start of
/// the next one, so every row is self-contained. Continuation rows are indented by `indent`
/// spaces (clamped so that every row can hold at least one column of content). A line that fits
/// is returned as a single, unchanged row; an empty line yields one empty row.
///
/// ```
/// use shellgame::ui::text::fold_ansi;
///
/// let rows = fold_ansi("\x1b[36mabcdef\x1b[0m", 4, 0);
/// assert_eq!(rows, ["\x1b[36mabcd\x1b[0m", "\x1b[36mef\x1b[0m"]);
/// ```
#[must_use]
pub fn fold_ansi(line: &str, width: usize, indent: usize) -> Vec<String> {
    let width = width.max(1);
    let indent = indent.min(width - 1);
    let mut rows = Vec::new();
    let mut state = SgrState::default();
    let mut row = String::new();
    let mut row_width = 0;
    let mut content_start = 0;

    for piece in scan(line) {
        match piece {
            Piece::Escape(escape) => {
                state.apply(escape);
                row.push_str(escape);
            }
            Piece::Text(text) => {
                for cluster in clusters(text) {
                    let cluster_width = UnicodeWidthStr::width(cluster);
                    if row_width + cluster_width > width && row_width > content_start {
                        state.close(&mut row);
                        rows.push(std::mem::take(&mut row));
                        row.push_str(&" ".repeat(indent));
                        state.reopen(&mut row);
                        row_width = indent;
                        content_start = indent;
                    }
                    row.push_str(cluster);
                    row_width += cluster_width;
                }
            }
        }
    }
    rows.push(row);
    rows
}

/// Shortens `line` to at most `width` visible columns, ending it with `…` when something was cut.
///
/// Styles that are open at the cut are closed. A line that fits is returned unchanged.
///
/// ```
/// use shellgame::ui::text::truncate;
///
/// assert_eq!(truncate("abcdefgh", 5), "abcd…");
/// assert_eq!(truncate("abc", 5), "abc");
/// ```
#[must_use]
pub fn truncate(line: &str, width: usize) -> String {
    if visible_width(line) <= width {
        return line.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    let budget = width - 1;
    let mut out = String::new();
    let mut used = 0;
    let mut state = SgrState::default();
    'scan: for piece in scan(line) {
        match piece {
            Piece::Escape(escape) => {
                state.apply(escape);
                out.push_str(escape);
            }
            Piece::Text(text) => {
                for cluster in clusters(text) {
                    let cluster_width = UnicodeWidthStr::width(cluster);
                    if used + cluster_width > budget {
                        break 'scan;
                    }
                    out.push_str(cluster);
                    used += cluster_width;
                }
            }
        }
    }
    out.push('…');
    state.close(&mut out);
    out
}

/// Wraps `text` in `base`, keeping `base` active after any style reset inside the text.
///
/// This lets a caller put a single style around a string that already contains styled spans.
#[must_use]
pub fn with_base_style(text: &str, base: Style) -> String {
    if base == Style::new() || text.is_empty() {
        return text.to_owned();
    }
    let open = base.render().to_string();
    let mut out = String::with_capacity(text.len() + 2 * open.len() + RESET.len());
    out.push_str(&open);
    for piece in scan(text) {
        match piece {
            Piece::Escape(escape) => {
                out.push_str(escape);
                if is_sgr_reset(escape) {
                    out.push_str(&open);
                }
            }
            Piece::Text(text) => out.push_str(text),
        }
    }
    out.push_str(RESET);
    out
}

// ---------------------------------------------------------------------------------------------
// Styled layout engine
// ---------------------------------------------------------------------------------------------

/// A run of text in a single style.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Frag {
    pub(crate) text: String,
    pub(crate) style: Style,
}

impl Frag {
    pub(crate) fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    fn width(&self) -> usize {
        UnicodeWidthStr::width(self.text.as_str())
    }
}

/// Inline content before layout.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Inline {
    /// Text; spaces inside it are break opportunities.
    Text(Frag),
    /// Text that is kept in one piece (spaces included) as long as it fits on a line of its own,
    /// such as an inline command; only when it is wider than a whole line does it break at its
    /// spaces.
    Unbreakable(Frag),
    /// A forced line break.
    Hard,
}

/// A word: fragments with no break opportunity between them (`**bold**,` is one word).
#[derive(Clone, Debug)]
pub(crate) struct Word {
    frags: Vec<Frag>,
    width: usize,
    /// Contains spaces that are only kept because the word is [`Inline::Unbreakable`].
    inner_space: bool,
}

/// A word under construction.
#[derive(Default)]
struct WordBuilder {
    frags: Vec<Frag>,
    inner_space: bool,
}

/// An item of laid-out text.
#[derive(Clone, Debug)]
pub(crate) enum Token {
    Word(Word),
    /// A run of spaces: dropped where a line is wrapped, kept inside a line.
    Space(Frag),
    Hard,
}

/// Appends text to a list of fragments, merging runs of the same style.
fn push_frag(frags: &mut Vec<Frag>, text: &str, style: Style) {
    match frags.last_mut() {
        Some(last) if last.style == style => last.text.push_str(text),
        _ => frags.push(Frag::new(text, style)),
    }
}

fn finish_word(word: &mut WordBuilder, tokens: &mut Vec<Token>) {
    if word.frags.is_empty() {
        return;
    }
    let WordBuilder { frags, inner_space } = std::mem::take(word);
    let joined: String = frags.iter().map(|frag| frag.text.as_str()).collect();
    tokens.push(Token::Word(Word {
        width: UnicodeWidthStr::width(joined.as_str()),
        frags,
        inner_space,
    }));
}

/// Maximal runs of spaces and of non-spaces, tagged with `true` for spaces.
fn runs(text: &str) -> impl Iterator<Item = (bool, &str)> {
    let mut rest = text;
    iter::from_fn(move || {
        let first = rest.chars().next()?;
        let is_space = first == ' ';
        let end = rest
            .find(|c: char| (c == ' ') != is_space)
            .unwrap_or(rest.len());
        let (run, tail) = rest.split_at(end);
        rest = tail;
        Some((is_space, run))
    })
}

/// Splits inline content into words, spaces and hard breaks.
pub(crate) fn tokenize(inlines: &[Inline]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut word = WordBuilder::default();
    for inline in inlines {
        match inline {
            Inline::Hard => {
                finish_word(&mut word, &mut tokens);
                tokens.push(Token::Hard);
            }
            Inline::Text(frag) => {
                for (is_space, run) in runs(&frag.text) {
                    if is_space {
                        finish_word(&mut word, &mut tokens);
                        tokens.push(Token::Space(Frag::new(run, frag.style)));
                    } else {
                        push_frag(&mut word.frags, run, frag.style);
                    }
                }
            }
            Inline::Unbreakable(frag) => {
                word.inner_space |= frag.text.contains(' ');
                push_frag(&mut word.frags, &frag.text, frag.style);
            }
        }
    }
    finish_word(&mut word, &mut tokens);
    tokens
}

/// Greedy first-fit line breaking.
///
/// Every returned row is at most `width` columns wide, except when a single character is wider
/// than `width`. Spaces at a wrap point and at the ends of a row are dropped; a word that is
/// longer than a whole row is split between characters. An [`Inline::Unbreakable`] word moves to
/// the next row as a whole and only breaks at its spaces when it is wider than a row.
pub(crate) fn layout(tokens: &[Token], width: usize) -> Vec<Vec<Frag>> {
    let width = width.max(1);
    let mut rows: Vec<Vec<Frag>> = Vec::new();
    let mut row: Vec<Frag> = Vec::new();
    let mut row_width = 0;
    let mut pending: Vec<Frag> = Vec::new();
    let mut pending_width = 0;
    // Tokens still to be placed; the next one is last.
    let mut work: Vec<Token> = tokens.iter().rev().cloned().collect();

    while let Some(token) = work.pop() {
        match token {
            Token::Space(frag) => {
                if row_width > 0 {
                    pending_width += frag.width();
                    pending.push(frag);
                }
            }
            Token::Hard => {
                rows.push(std::mem::take(&mut row));
                row_width = 0;
                pending.clear();
                pending_width = 0;
            }
            Token::Word(word) if word.inner_space && word.width > width => {
                // Too wide to stay in one piece: fall back to breaking at its spaces.
                let inlines: Vec<Inline> = word.frags.into_iter().map(Inline::Text).collect();
                work.extend(tokenize(&inlines).into_iter().rev());
            }
            Token::Word(word) => {
                if row_width > 0 && row_width + pending_width + word.width > width {
                    rows.push(std::mem::take(&mut row));
                    row_width = 0;
                    pending.clear();
                    pending_width = 0;
                }
                for space in pending.drain(..) {
                    push_frag(&mut row, &space.text, space.style);
                }
                row_width += pending_width;
                pending_width = 0;

                if row_width + word.width <= width {
                    for frag in &word.frags {
                        push_frag(&mut row, &frag.text, frag.style);
                    }
                    row_width += word.width;
                } else {
                    // Wider than a whole row: split between clusters.
                    for frag in &word.frags {
                        for cluster in clusters(&frag.text) {
                            let cluster_width = UnicodeWidthStr::width(cluster);
                            if row_width > 0 && row_width + cluster_width > width {
                                rows.push(std::mem::take(&mut row));
                                row_width = 0;
                            }
                            push_frag(&mut row, cluster, frag.style);
                            row_width += cluster_width;
                        }
                    }
                }
            }
        }
    }
    if !row.is_empty() {
        rows.push(row);
    }
    rows
}

/// Visible width of a laid-out row.
pub(crate) fn frags_width(frags: &[Frag]) -> usize {
    frags.iter().map(Frag::width).sum()
}

/// Appends `text` to `out`, wrapped in `style` when `color` is on and the style is not plain.
///
/// The style is opened and reset around the text itself, so the result is self-contained.
pub(crate) fn push_styled(out: &mut String, style: Style, text: &str, color: bool) {
    if color && style != Style::new() && !text.is_empty() {
        // Writing into a String cannot fail.
        let _ = write!(out, "{}{text}{}", style.render(), style.render_reset());
    } else {
        out.push_str(text);
    }
}

/// Renders a row of fragments as one self-contained string.
pub(crate) fn render_frags(frags: &[Frag], color: bool) -> String {
    let mut out = String::new();
    let mut index = 0;
    while index < frags.len() {
        let style = frags[index].style;
        let mut run = String::new();
        while index < frags.len() && frags[index].style == style {
            run.push_str(&frags[index].text);
            index += 1;
        }
        push_styled(&mut out, style, &run, color);
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Plain text
// ---------------------------------------------------------------------------------------------

/// Wraps plain text to `width` columns without interpreting it in any way.
///
/// Meant for hints and messages, which must be printed exactly as written: `[ab]*.txt`,
/// backticks and asterisks are ordinary characters here. Explicit newlines start a new line,
/// blank lines are kept, runs of spaces inside a line are kept, and lines break only at spaces
/// (a word longer than `width` is split between characters). The leading indentation of a line
/// is repeated on its continuation rows. Trailing spaces are removed, control characters are
/// replaced (see [`sanitize`]) and tabs expanded.
///
/// ```
/// use shellgame::ui::text::wrap_plain;
///
/// assert_eq!(
///     wrap_plain("použijte `ls [ab]*.txt` zde", 14),
///     ["použijte `ls", "[ab]*.txt` zde"],
/// );
/// ```
#[must_use]
pub fn wrap_plain(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out = Vec::new();
    for raw in text.lines() {
        let expanded = expand_tabs(raw, TAB_WIDTH);
        let clean = sanitize(&expanded);
        wrap_plain_line(&clean, width, &mut out);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

fn wrap_plain_line(line: &str, width: usize, out: &mut Vec<String>) {
    let content = line.trim_start_matches(' ');
    let indent = (line.len() - content.len()).min(width / 2);
    let content = content.trim_end();
    if content.is_empty() {
        out.push(String::new());
        return;
    }
    let inlines = [Inline::Text(Frag::new(content, Style::new()))];
    let tokens = tokenize(&inlines);
    let prefix = " ".repeat(indent);
    for row in layout(&tokens, width - indent) {
        out.push(format!("{prefix}{}", render_frags(&row, false)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain_rows(text: &str, width: usize) -> Vec<String> {
        let inlines = [Inline::Text(Frag::new(text, Style::new()))];
        layout(&tokenize(&inlines), width)
            .iter()
            .map(|row| render_frags(row, false))
            .collect()
    }

    // --- scanning and widths ---------------------------------------------------------------

    #[test]
    fn strip_ansi_removes_csi_osc_and_two_byte_escapes() {
        assert_eq!(strip_ansi("plain"), "plain");
        assert!(matches!(strip_ansi("plain"), Cow::Borrowed(_)));
        assert_eq!(strip_ansi("\x1b[1;38;5;177mfoo\x1b[0m bar"), "foo bar");
        assert_eq!(
            strip_ansi("a\x1b]8;;https://x.y\x07link\x1b]8;;\x07b"),
            "alinkb"
        );
        assert_eq!(strip_ansi("a\x1b]0;title\x1b\\b"), "ab");
        assert_eq!(strip_ansi("a\x1b(Bb"), "ab");
    }

    #[test]
    fn strip_ansi_survives_truncated_and_odd_sequences() {
        assert_eq!(strip_ansi("abc\x1b"), "abc");
        assert_eq!(strip_ansi("abc\x1b[31"), "abc");
        assert_eq!(strip_ansi("abc\x1bé"), "abc");
        assert_eq!(strip_ansi("\x1b[mx"), "x");
    }

    #[test]
    fn visible_width_counts_columns() {
        assert_eq!(visible_width(""), 0);
        assert_eq!(visible_width("abc"), 3);
        assert_eq!(visible_width("Příliš žluťoučký kůň"), 20);
        assert_eq!(visible_width("├── home/"), 9);
        assert_eq!(visible_width("\x1b[1m\x1b[36mabc\x1b[0m"), 3);
    }

    #[test]
    fn visible_width_handles_emoji() {
        assert_eq!(visible_width("💡"), 2);
        assert_eq!(visible_width("🏆"), 2);
        assert_eq!(visible_width("✅"), 2);
        assert_eq!(visible_width("❌"), 2);
        assert_eq!(visible_width("🚪"), 2);
        // Emoji presentation sequence: base + VS16 is a wide emoji.
        assert_eq!(visible_width("⚠\u{fe0f}"), 2);
        // The bare text-presentation symbol stays narrow.
        assert_eq!(visible_width("⚠"), 1);
        assert_eq!(visible_width("✓"), 1);
    }

    #[test]
    fn clusters_keep_marks_and_variation_selectors_attached() {
        let parts: Vec<&str> = clusters("a\u{301}⚠\u{fe0f}b").collect();
        assert_eq!(parts, ["a\u{301}", "⚠\u{fe0f}", "b"]);
        let family: Vec<&str> = clusters("👨\u{200d}👩x").collect();
        assert_eq!(family, ["👨\u{200d}👩", "x"]);
    }

    // --- padding ---------------------------------------------------------------------------

    #[test]
    fn pad_to_pads_by_visible_width() {
        assert_eq!(pad_to("ab", 5), "ab   ");
        assert_eq!(pad_to("abcdef", 3), "abcdef");
        let colored = pad_to("\x1b[31mčau\x1b[0m", 6);
        assert_eq!(visible_width(&colored), 6);
        assert!(colored.starts_with("\x1b[31m"));
        assert_eq!(visible_width(&pad_to("💡 tip", 10)), 10);
    }

    #[test]
    fn pad_start_and_center_align() {
        assert_eq!(pad_start("7", 3), "  7");
        assert_eq!(center("ab", 6), "  ab  ");
        assert_eq!(center("abc", 6), " abc  ");
        assert_eq!(center("abcdefg", 3), "abcdefg");
        assert_eq!(visible_width(&center("\x1b[1mžluť\x1b[0m", 11)), 11);
    }

    // --- sanitising ------------------------------------------------------------------------

    #[test]
    fn sanitize_replaces_control_characters_only() {
        assert_eq!(sanitize("ok\tline\nnext"), "ok\tline\nnext");
        assert_eq!(sanitize("a\x1b[31mb"), "a\u{fffd}[31mb");
        assert_eq!(sanitize("a\u{7f}b\u{85}c"), "a\u{fffd}b\u{fffd}c");
        assert!(matches!(sanitize("clean"), Cow::Borrowed(_)));
    }

    #[test]
    fn normalize_emoji_drops_vs16_after_narrow_characters_only() {
        assert_eq!(
            normalize_emoji("⚠\u{fe0f} a ✔\u{fe0f} ❤\u{fe0f}"),
            "⚠ a ✔ ❤"
        );
        assert_eq!(normalize_emoji("💡\u{fe0f}"), "💡\u{fe0f}");
        assert_eq!(normalize_emoji("\u{fe0f}"), "\u{fe0f}");
        assert!(matches!(normalize_emoji("bez emoji ✅"), Cow::Borrowed(_)));
        assert_eq!(visible_width(&normalize_emoji("⚠\u{fe0f}")), 1);
    }

    #[test]
    fn sanitize_also_normalises_emoji() {
        assert_eq!(sanitize("⚠\u{fe0f}\x1b!"), "⚠\u{fffd}!");
    }

    #[test]
    fn expand_tabs_uses_tab_stops() {
        assert_eq!(expand_tabs("a\tb", 4), "a   b");
        assert_eq!(expand_tabs("\tb", 4), "    b");
        assert_eq!(expand_tabs("abcd\tb", 4), "abcd    b");
        assert_eq!(expand_tabs("žž\tb", 4), "žž  b");
        assert!(matches!(expand_tabs("none", 4), Cow::Borrowed(_)));
    }

    // --- folding and truncating -------------------------------------------------------------

    #[test]
    fn fold_ansi_splits_plain_text() {
        assert_eq!(fold_ansi("abcdef", 4, 0), ["abcd", "ef"]);
        assert_eq!(fold_ansi("abcd", 4, 0), ["abcd"]);
        assert_eq!(fold_ansi("", 4, 0), [""]);
        assert_eq!(fold_ansi("abcdefgh", 4, 2), ["abcd", "  ef", "  gh"]);
    }

    #[test]
    fn fold_ansi_carries_styles_across_rows() {
        let rows = fold_ansi("\x1b[1m\x1b[36mabcdef\x1b[0m!", 4, 0);
        assert_eq!(
            rows,
            ["\x1b[1m\x1b[36mabcd\x1b[0m", "\x1b[1m\x1b[36mef\x1b[0m!"]
        );
        for row in &rows {
            assert!(visible_width(row) <= 4);
        }
    }

    #[test]
    fn fold_ansi_never_splits_a_wide_character_or_emoji_sequence() {
        let rows = fold_ansi("ab⚠\u{fe0f}cd", 3, 0);
        assert_eq!(rows, ["ab", "⚠\u{fe0f}c", "d"]);
        let cjk = fold_ansi("字字字", 3, 0);
        assert_eq!(cjk, ["字", "字", "字"]);
    }

    #[test]
    fn fold_ansi_survives_degenerate_widths() {
        assert_eq!(fold_ansi("abc", 0, 5), ["a", "b", "c"]);
        assert_eq!(fold_ansi("abc", 1, 1), ["a", "b", "c"]);
    }

    #[test]
    fn truncate_shortens_with_ellipsis_and_closes_styles() {
        assert_eq!(truncate("abcdefgh", 5), "abcd…");
        assert_eq!(truncate("abc", 3), "abc");
        assert_eq!(truncate("abc", 0), "");
        assert_eq!(truncate("abcdef", 1), "…");
        assert_eq!(truncate("\x1b[1mabcdefgh\x1b[0m", 4), "\x1b[1mabc…\x1b[0m");
        assert_eq!(visible_width(&truncate("💡💡💡💡", 5)), 5);
    }

    #[test]
    fn with_base_style_survives_inner_resets() {
        let base = Style::new().bold();
        let open = base.render().to_string();
        let out = with_base_style("a\x1b[2mb\x1b[0mc", base);
        assert_eq!(out, format!("{open}a\x1b[2mb\x1b[0m{open}c\x1b[0m"));
        assert_eq!(with_base_style("x", Style::new()), "x");
        assert_eq!(with_base_style("", base), "");
    }

    // --- layout ----------------------------------------------------------------------------

    #[test]
    fn layout_breaks_at_spaces_only() {
        assert_eq!(
            plain_rows("jedna dva tři čtyři pět", 10),
            ["jedna dva", "tři čtyři", "pět"]
        );
        assert_eq!(plain_rows("a/b/c-d/e", 4), ["a/b/", "c-d/", "e"]);
    }

    #[test]
    fn layout_drops_spaces_at_wrap_points_but_keeps_inner_runs() {
        assert_eq!(plain_rows("aa   bb", 10), ["aa   bb"]);
        assert_eq!(plain_rows("aaaa    bbbb", 5), ["aaaa", "bbbb"]);
        assert_eq!(plain_rows("  lead", 10), ["lead"]);
        assert_eq!(plain_rows("trail   ", 10), ["trail"]);
    }

    #[test]
    fn layout_hard_splits_overlong_words() {
        assert_eq!(plain_rows("abcdefghij", 4), ["abcd", "efgh", "ij"]);
        assert_eq!(
            plain_rows("ab abcdefghij cd", 4),
            ["ab", "abcd", "efgh", "ij", "cd"]
        );
    }

    #[test]
    fn layout_keeps_adjacent_styled_fragments_in_one_word() {
        let bold = Style::new().bold();
        let inlines = [
            Inline::Text(Frag::new("xx ", Style::new())),
            Inline::Text(Frag::new("code", bold)),
            Inline::Text(Frag::new(",", Style::new())),
            Inline::Text(Frag::new(" end", Style::new())),
        ];
        let rows = layout(&tokenize(&inlines), 7);
        let texts: Vec<String> = rows.iter().map(|row| render_frags(row, false)).collect();
        // "code," must stay together even though the style changes inside the word.
        assert_eq!(texts, ["xx", "code,", "end"]);
    }

    #[test]
    fn layout_keeps_unbreakable_spans_together_while_they_fit() {
        let bold = Style::new().bold();
        let inlines = [
            Inline::Text(Frag::new("xx ", Style::new())),
            Inline::Unbreakable(Frag::new("cd ../a", bold)),
            Inline::Text(Frag::new(" end", Style::new())),
        ];
        let tokens = tokenize(&inlines);
        let rows = |width| -> Vec<String> {
            layout(&tokens, width)
                .iter()
                .map(|row| render_frags(row, false))
                .collect()
        };
        assert_eq!(rows(12), ["xx cd ../a", "end"]);
        assert_eq!(rows(8), ["xx", "cd ../a", "end"]);
        // Wider than a whole row: the span falls back to breaking at its spaces.
        assert_eq!(rows(5), ["xx cd", "../a", "end"]);
    }

    #[test]
    fn layout_honours_hard_breaks() {
        let inlines = [
            Inline::Text(Frag::new("one", Style::new())),
            Inline::Hard,
            Inline::Text(Frag::new("two three", Style::new())),
        ];
        let rows: Vec<String> = layout(&tokenize(&inlines), 20)
            .iter()
            .map(|row| render_frags(row, false))
            .collect();
        assert_eq!(rows, ["one", "two three"]);
    }

    #[test]
    fn rendered_rows_are_self_contained() {
        let bold = Style::new().bold();
        let inlines = [Inline::Text(Frag::new("alpha beta gamma delta", bold))];
        let rows = layout(&tokenize(&inlines), 11);
        assert_eq!(rows.len(), 2);
        for row in &rows {
            let line = render_frags(row, true);
            assert!(line.starts_with(&bold.render().to_string()), "{line:?}");
            assert!(line.ends_with("\x1b[0m"), "{line:?}");
        }
    }

    #[test]
    fn render_frags_merges_equal_styles_and_skips_plain() {
        let bold = Style::new().bold();
        let frags = [
            Frag::new("a", bold),
            Frag::new(" ", bold),
            Frag::new("b", bold),
            Frag::new(" c", Style::new()),
        ];
        let open = bold.render().to_string();
        assert_eq!(render_frags(&frags, true), format!("{open}a b\x1b[0m c"));
        assert_eq!(render_frags(&frags, false), "a b c");
    }

    // --- wrap_plain ------------------------------------------------------------------------

    #[test]
    fn wrap_plain_keeps_markup_like_text_verbatim() {
        let text = "Zkuste `ls [ab]*.txt` a **nic** _nemění_ [x](y)";
        let lines = wrap_plain(text, 200);
        assert_eq!(lines, [text]);
        let narrow = wrap_plain("ls [ab]*.txt `x`", 10);
        assert_eq!(narrow, ["ls", "[ab]*.txt", "`x`"]);
    }

    #[test]
    fn wrap_plain_respects_newlines_and_blank_lines() {
        assert_eq!(wrap_plain("a\n\nb", 10), ["a", "", "b"]);
        assert_eq!(wrap_plain("a\r\nb\n", 10), ["a", "b"]);
        assert_eq!(wrap_plain("", 10), [""]);
    }

    #[test]
    fn wrap_plain_indents_continuation_rows_like_the_first() {
        assert_eq!(
            wrap_plain("  cd ../finish && ls", 12),
            ["  cd", "  ../finish", "  && ls"]
        );
    }

    #[test]
    fn wrap_plain_replaces_control_characters_and_expands_tabs() {
        assert_eq!(wrap_plain("a\x1b[31mb", 20), ["a\u{fffd}[31mb"]);
        assert_eq!(wrap_plain("a\tb", 20), ["a   b"]);
    }

    #[test]
    fn wrap_plain_lines_never_exceed_the_width() {
        let text =
            "Příliš žluťoučký kůň úpěl ďábelské ódy 💡 a ještě jedno velmi_dlouhé_slovo_bez_mezer";
        for width in [5, 10, 17, 40] {
            for line in wrap_plain(text, width) {
                assert!(visible_width(&line) <= width, "{line:?} at {width}");
            }
        }
    }
}
