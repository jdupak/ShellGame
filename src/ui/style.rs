//! Named styles: the palette of the Python UI plus the markdown element styles.
//!
//! Everything here is a plain [`anstyle::Style`] constant, so the values are `Copy`, can be used
//! in `const` contexts and are only turned into escape sequences when a renderer decides that
//! colour is wanted (see [`crate::ui::Term::color`]).
//!
//! The palette follows `display.py`: cyan for instructions, yellow for hints, green for success,
//! red for failures and blocked commands, dim for secondary notes and violet for commands that
//! are mentioned inside such notes.

use anstyle::{Ansi256Color, AnsiColor, Style};

// ---------------------------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------------------------

/// No styling at all.
pub const PLAIN: Style = Style::new();
/// Bold text.
pub const BOLD: Style = Style::new().bold();
/// Dimmed (faint) text, used for secondary notes.
pub const DIM: Style = Style::new().dimmed();
/// Cyan: instruction panels.
pub const CYAN: Style = AnsiColor::Cyan.on_default();
/// Bold cyan: panel titles of instruction panels.
pub const BOLD_CYAN: Style = CYAN.bold();
/// Yellow: hints and warnings.
pub const YELLOW: Style = AnsiColor::Yellow.on_default();
/// Bold yellow.
pub const BOLD_YELLOW: Style = YELLOW.bold();
/// Green: success.
pub const GREEN: Style = AnsiColor::Green.on_default();
/// Bold green: success messages.
pub const BOLD_GREEN: Style = GREEN.bold();
/// Red: failures and blocked commands.
pub const RED: Style = AnsiColor::Red.on_default();
/// Bold red: failure messages.
pub const BOLD_RED: Style = RED.bold();
/// Magenta.
pub const MAGENTA: Style = AnsiColor::Magenta.on_default();
/// Violet (256-colour index 177, the colour Rich calls `violet`): commands inside notes.
pub const VIOLET: Style = Ansi256Color(177).on_default();
/// Dim and bold, for emphasis inside a dim note.
pub const DIM_BOLD: Style = combine(DIM, BOLD);
/// Dim and violet, for a command inside a dim note (`[dim]… [violet]cmd[/violet][/dim]`).
pub const DIM_VIOLET: Style = combine(DIM, VIOLET);

// ---------------------------------------------------------------------------------------------
// Markdown elements
// ---------------------------------------------------------------------------------------------

/// Level 1 heading text.
pub const H1: Style = Style::new().bold().underline();
/// Level 2 heading text.
pub const H2: Style = Style::new().bold().underline();
/// Level 3 heading text.
pub const H3: Style = Style::new().bold();
/// Level 4 heading text.
pub const H4: Style = Style::new().bold().dimmed();
/// Level 5 heading text.
pub const H5: Style = Style::new().italic();
/// Level 6 heading text.
pub const H6: Style = Style::new().italic().dimmed();
/// The `────` line that follows a level 1 or 2 heading (the text itself is underlined already).
pub const HEADING_RULE: Style = Style::new().bold();
/// `*emphasis*`.
pub const EMPHASIS: Style = Style::new().italic();
/// `**strong**`.
pub const STRONG: Style = Style::new().bold();
/// `~~strikethrough~~` (only produced when the extension is enabled).
pub const STRIKETHROUGH: Style = Style::new().strikethrough();
/// `` `inline code` ``.
pub const INLINE_CODE: Style = BOLD_CYAN;
/// Fenced and indented code blocks: plain cyan, calmer than the bold cyan of inline code and
/// readable on dark and light terminals alike.
pub const CODE_BLOCK: Style = CYAN;
/// Bullets and list numbers.
pub const LIST_MARKER: Style = BOLD_YELLOW;
/// Block quotes (the `▌` bar and the quoted text).
pub const BLOCKQUOTE: Style = MAGENTA;
/// Link text.
pub const LINK: Style = Style::new().underline();
/// Thematic breaks (`***`) and link targets.
pub const MUTED: Style = DIM;

// ---------------------------------------------------------------------------------------------
// Combinators
// ---------------------------------------------------------------------------------------------

/// Lays `top` over `base`, like Rich's `Style.__add__`: colours of `top` win where it sets one,
/// text effects are the union of both.
///
/// ```
/// use shellgame::ui::style::{combine, BOLD, CYAN};
///
/// let style = combine(BOLD, CYAN);
/// assert_eq!(style, CYAN.bold());
/// ```
#[must_use]
pub const fn combine(base: Style, top: Style) -> Style {
    let fg = match top.get_fg_color() {
        Some(color) => Some(color),
        None => base.get_fg_color(),
    };
    let bg = match top.get_bg_color() {
        Some(color) => Some(color),
        None => base.get_bg_color(),
    };
    let underline = match top.get_underline_color() {
        Some(color) => Some(color),
        None => base.get_underline_color(),
    };
    Style::new()
        .fg_color(fg)
        .bg_color(bg)
        .underline_color(underline)
        .effects(base.get_effects().insert(top.get_effects()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anstyle::Color;

    #[test]
    fn combine_unions_effects_and_prefers_top_colours() {
        let base = Style::new()
            .bold()
            .fg_color(Some(Color::Ansi(AnsiColor::Red)));
        let top = Style::new()
            .underline()
            .fg_color(Some(Color::Ansi(AnsiColor::Blue)));
        let merged = combine(base, top);
        assert_eq!(merged.get_fg_color(), Some(Color::Ansi(AnsiColor::Blue)));
        assert!(merged.get_effects().contains(anstyle::Effects::BOLD));
        assert!(merged.get_effects().contains(anstyle::Effects::UNDERLINE));
    }

    #[test]
    fn combine_keeps_base_colour_when_top_has_none() {
        assert_eq!(combine(CYAN, BOLD), BOLD_CYAN);
        assert_eq!(combine(PLAIN, PLAIN), PLAIN);
    }

    #[test]
    fn dim_violet_is_dim_and_coloured() {
        assert!(DIM_VIOLET.get_effects().contains(anstyle::Effects::DIMMED));
        assert_eq!(DIM_VIOLET.get_fg_color(), VIOLET.get_fg_color());
    }

    #[test]
    fn code_block_differs_from_inline_code() {
        assert_ne!(CODE_BLOCK, INLINE_CODE);
        assert!(!CODE_BLOCK.is_plain());
    }

    #[test]
    fn heading_styles_follow_the_spec() {
        assert!(H1.get_effects().contains(anstyle::Effects::UNDERLINE));
        assert!(H2.get_effects().contains(anstyle::Effects::BOLD));
        assert_eq!(H3, BOLD);
        assert!(H4.get_effects().contains(anstyle::Effects::DIMMED));
    }
}
