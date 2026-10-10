//! Text helpers that reproduce Python's behaviour exactly.
//!
//! Answers and file checks were written against Python's `str.strip()`, `int()` and
//! `str.splitlines()`. Each of them differs from the obvious Rust counterpart in a way a
//! player can hit, so the differences are pinned here, in one place, and tested against
//! the values Python itself produces.

use std::borrow::Cow;

/// Python's `str.isspace()`: Rust's Unicode `White_Space` plus the four ASCII
/// information separators `\x1c`..`\x1f`.
pub(crate) fn is_py_space(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}')
}

/// Python's `str.strip()`.
pub(crate) fn py_strip(s: &str) -> &str {
    s.trim_matches(is_py_space)
}

/// `len(s.splitlines())`.
///
/// A line ends at `\n`, `\r`, `\r\n` (one break), `\v`, `\f`, `\x1c`, `\x1d`, `\x1e`,
/// `\x85`, `U+2028` and `U+2029`. A final line needs no terminator, and an empty final
/// segment is not a line, so `"a\n"` has one line and `"\n"` has one empty line.
pub(crate) fn py_splitlines_count(s: &str) -> usize {
    let mut count = 0;
    let mut open_line = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                count += 1;
                open_line = false;
            }
            '\n' | '\u{0b}' | '\u{0c}' | '\u{1c}' | '\u{1d}' | '\u{1e}' | '\u{85}' | '\u{2028}'
            | '\u{2029}' => {
                count += 1;
                open_line = false;
            }
            _ => open_line = true,
        }
    }
    count + usize::from(open_line)
}

/// What Python's text-mode `read_text()` does to line endings: `\r\n` and a lone `\r`
/// both become `\n`.
pub(crate) fn universal_newlines(s: &str) -> Cow<'_, str> {
    if s.contains('\r') {
        Cow::Owned(s.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        Cow::Borrowed(s)
    }
}

/// The result of parsing text the way `int(text.strip())` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PyInt {
    /// A valid integer that fits an `i64`.
    Value(i64),
    /// A valid integer too large for an `i64`. Python's integers are unbounded, so this
    /// is a number like any other, just never one a level expects.
    TooBig,
    /// Not an integer; Python raises `ValueError`.
    Invalid,
}

/// Python allows at most this many decimal digits in `int(str)` (since 3.11); longer
/// input raises `ValueError`.
const MAX_INT_STR_DIGITS: usize = 4300;

/// The digit zero of every Unicode decimal-digit (`Nd`) block, as of Unicode 16.0. Each
/// block is the ten consecutive code points `zero..zero + 10`, valued 0 to 9.
///
/// Python's `int()` accepts every one of them (`int("４２")` is 42, and a student with a
/// CJK input method can easily type fullwidth digits), so answers must too. Generated from
/// `unicodedata` and cross-checked against `int()` for every code point.
const DECIMAL_ZEROS: [u32; 76] = [
    0x30, 0x660, 0x6f0, 0x7c0, 0x966, 0x9e6, 0xa66, 0xae6, 0xb66, 0xbe6, 0xc66, 0xce6, 0xd66,
    0xde6, 0xe50, 0xed0, 0xf20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946, 0x19d0, 0x1a80, 0x1a90,
    0x1b50, 0x1bb0, 0x1c40, 0x1c50, 0xa620, 0xa8d0, 0xa900, 0xa9d0, 0xa9f0, 0xaa50, 0xabf0, 0xff10,
    0x104a0, 0x10d30, 0x10d40, 0x11066, 0x110f0, 0x11136, 0x111d0, 0x112f0, 0x11450, 0x114d0,
    0x11650, 0x116c0, 0x116d0, 0x116da, 0x11730, 0x118e0, 0x11950, 0x11bf0, 0x11c50, 0x11d50,
    0x11da0, 0x11f50, 0x16130, 0x16a60, 0x16ac0, 0x16b50, 0x16d70, 0x1ccf0, 0x1d7ce, 0x1d7d8,
    0x1d7e2, 0x1d7ec, 0x1d7f6, 0x1e140, 0x1e2f0, 0x1e4f0, 0x1e5f1, 0x1e950, 0x1fbf0,
];

/// The value of a Unicode decimal digit, or `None` for any other character.
fn decimal_value(c: char) -> Option<u32> {
    let code = u32::from(c);
    DECIMAL_ZEROS
        .iter()
        .find(|&&zero| (zero..zero + 10).contains(&code))
        .map(|zero| code - zero)
}

/// Parse a string like Python's `int()`: an optional ASCII sign, then decimal digits (any
/// Unicode `Nd` digit, not just `0`-`9`) with single underscores *between* digits
/// (`"1_000"`), leading zeros allowed (`"007"`). The caller strips surrounding whitespace.
pub(crate) fn parse_py_int(s: &str) -> PyInt {
    let (negative, body) = match s.chars().next() {
        Some('+') => (false, &s[1..]),
        Some('-') => (true, &s[1..]),
        _ => (false, s),
    };
    if body.is_empty() || body.starts_with('_') || body.ends_with('_') {
        return PyInt::Invalid;
    }
    let mut digits = String::with_capacity(body.len());
    let mut previous_underscore = false;
    for c in body.chars() {
        if c == '_' {
            if previous_underscore {
                return PyInt::Invalid;
            }
            previous_underscore = true;
        } else if let Some(value) = decimal_value(c).and_then(|value| char::from_digit(value, 10)) {
            previous_underscore = false;
            digits.push(value);
        } else {
            return PyInt::Invalid;
        }
    }
    if digits.len() > MAX_INT_STR_DIGITS {
        return PyInt::Invalid;
    }
    let signed = if negative {
        format!("-{digits}")
    } else {
        digits
    };
    signed.parse::<i64>().map_or(PyInt::TooBig, PyInt::Value)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every code point for which Python's `chr(c).isspace()` is true (Python 3.14).
    const PYTHON_WHITESPACE: [u32; 29] = [
        0x9, 0xa, 0xb, 0xc, 0xd, 0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x85, 0xa0, 0x1680, 0x2000, 0x2001,
        0x2002, 0x2003, 0x2004, 0x2005, 0x2006, 0x2007, 0x2008, 0x2009, 0x200a, 0x2028, 0x2029,
        0x202f, 0x205f, 0x3000,
    ];

    #[test]
    fn whitespace_matches_python_for_every_code_point() {
        for code in 0..=0x10_FFFFu32 {
            let Some(c) = char::from_u32(code) else {
                continue;
            };
            assert_eq!(
                is_py_space(c),
                PYTHON_WHITESPACE.contains(&code),
                "U+{code:04X}"
            );
        }
    }

    #[test]
    fn strip_removes_python_whitespace_only() {
        assert_eq!(py_strip("  a b \t\n"), "a b");
        assert_eq!(py_strip("\u{1c}\u{1f}x\u{a0}\u{3000}"), "x");
        assert_eq!(
            py_strip("\u{200b}x\u{200b}"),
            "\u{200b}x\u{200b}",
            "zero width space is not space"
        );
        assert_eq!(py_strip(""), "");
        assert_eq!(py_strip("   "), "");
    }

    #[test]
    fn splitlines_counts_like_python() {
        // Values produced by `len(s.splitlines())` in Python 3.14.
        let cases: [(&str, usize); 18] = [
            ("", 0),
            ("a", 1),
            ("a\n", 1),
            ("\n", 1),
            ("a\nb", 2),
            ("a\nb\n", 2),
            ("a\n\nb", 3),
            ("a\r\nb", 2),
            ("a\rb", 2),
            ("a\r\n\r\n", 2),
            ("\r\r\n", 2),
            ("a\u{0b}b", 2),
            ("a\u{0c}b", 2),
            ("a\u{1c}b\u{1d}c\u{1e}d", 4),
            ("a\u{1f}b", 1),
            ("a\u{85}b", 2),
            ("a\u{2028}b\u{2029}c", 3),
            ("one\ntwo\r\nthree\rfour", 4),
        ];
        for (text, expected) in cases {
            assert_eq!(py_splitlines_count(text), expected, "{text:?}");
        }
    }

    #[test]
    fn universal_newlines_translates_cr_and_crlf() {
        assert_eq!(universal_newlines("a\r\nb\rc\nd"), "a\nb\nc\nd");
        assert!(matches!(
            universal_newlines("no returns\n"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn py_int_accepts_what_python_accepts() {
        let valid: [(&str, i64); 14] = [
            ("5", 5),
            ("+5", 5),
            ("-5", -5),
            ("007", 7),
            ("-0", 0),
            ("1_000", 1000),
            ("1_0_0", 100),
            ("0_7", 7),
            ("0_0_7", 7),
            ("00_1", 1),
            ("+1_2", 12),
            ("9223372036854775807", i64::MAX),
            ("-9223372036854775808", i64::MIN),
            ("12345", 12345),
        ];
        for (text, expected) in valid {
            assert_eq!(parse_py_int(text), PyInt::Value(expected), "{text:?}");
        }
    }

    #[test]
    fn py_int_rejects_what_python_rejects() {
        for text in [
            "", "+", "-", "_1", "1_", "1__0", "+_1", "- 5", "--5", "+-5", "1e3", "0x10", "1.5",
            "5a", "a5", "1 000", " 5", "5 ",
        ] {
            assert_eq!(parse_py_int(text), PyInt::Invalid, "{text:?}");
        }
    }

    #[test]
    fn py_int_accepts_unicode_decimal_digits_like_python() {
        let valid: [(&str, i64); 9] = [
            ("\u{0663}", 3),
            ("\u{ff14}\u{ff12}", 42),
            ("\u{0967}\u{0968}\u{0969}", 123),
            ("\u{1d7d0}\u{1d7ce}", 20),
            ("-\u{ff15}", -5),
            ("+\u{0663}", 3),
            ("1_\u{0663}", 13),
            ("\u{0663}\u{0664}_5", 345),
            ("\u{1fbf9}", 9),
        ];
        for (text, expected) in valid {
            assert_eq!(parse_py_int(text), PyInt::Value(expected), "{text:?}");
        }
    }

    #[test]
    fn py_int_rejects_numerals_that_are_not_decimal_digits() {
        // Superscripts, fractions, roman numerals and CJK numerals are numbers to a human
        // but not to Python's int().
        for text in [
            "\u{b2}",
            "\u{bd}",
            "\u{2167}",
            "\u{4e09}",
            "\u{2460}",
            "\u{ff0b}5",
            "\u{2212}5",
        ] {
            assert_eq!(parse_py_int(text), PyInt::Invalid, "{text:?}");
        }
    }

    #[test]
    fn the_decimal_digit_table_is_well_formed() {
        for pair in DECIMAL_ZEROS.windows(2) {
            assert!(
                pair[0] + 10 <= pair[1],
                "blocks are sorted and disjoint: {pair:x?}"
            );
        }
        for zero in DECIMAL_ZEROS {
            for value in 0..10 {
                let c = char::from_u32(zero + value).unwrap();
                assert_eq!(decimal_value(c), Some(value), "U+{:04X}", zero + value);
                assert!(c.is_numeric(), "U+{:04X}", zero + value);
            }
        }
        // Neighbours of the ASCII block.
        assert_eq!(decimal_value('/'), None);
        assert_eq!(decimal_value(':'), None);
        assert_eq!(decimal_value('a'), None);
    }

    #[test]
    fn py_int_beyond_i64_is_a_valid_but_huge_number() {
        assert_eq!(parse_py_int("9223372036854775808"), PyInt::TooBig);
        assert_eq!(parse_py_int("-9223372036854775809"), PyInt::TooBig);
        assert_eq!(parse_py_int(&"9".repeat(4300)), PyInt::TooBig);
    }

    #[test]
    fn py_int_enforces_pythons_digit_limit() {
        // 4301 digits: ValueError in Python, so "invalid", with or without underscores or
        // leading zeros.
        assert_eq!(parse_py_int(&"9".repeat(4301)), PyInt::Invalid);
        assert_eq!(
            parse_py_int(&format!("{}1", "0".repeat(5000))),
            PyInt::Invalid
        );
        assert_eq!(
            parse_py_int(&format!("1{}", "_0".repeat(4301))),
            PyInt::Invalid
        );
        assert_eq!(
            parse_py_int(&format!("1{}", "_0".repeat(4299))),
            PyInt::TooBig
        );
        assert_eq!(
            parse_py_int(&format!("{}1", "0_".repeat(3000))),
            PyInt::Value(1)
        );
    }
}
