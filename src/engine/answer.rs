//! Answer rules: how a submitted answer is judged.
//!
//! An [`Answer`] is built from a constructor ([`exact`], [`one_of`], [`int`], [`suffix`],
//! [`parts`], [`exact_with`]) and refined with modifiers. The order in which a *text* rule
//! judges an answer is part of the contract, because authors rely on it:
//!
//! 1. **normalize**: strip surrounding whitespace (Python's `str.strip`, so `\x1c`..`\x1f`
//!    count as whitespace too) and, with [`Answer::trim_trailing_slash`], trailing `/`;
//! 2. **literal mistakes** ([`Answer::mistake`]): an exact match gives its tailored message;
//! 3. **correct?** If so, pass;
//! 4. **computed feedback** ([`Answer::feedback`]) for answers that cannot be listed
//!    literally, such as "starts with `d` but does not end with `a`";
//! 5. **the error text** ([`Answer::error`]), or the terse [`messages::INCORRECT`].
//!
//! The default wrong-answer text is exactly `Nesprávně.` and never contains the answer:
//! a hint about the answer belongs in a level's hints, where the player asks for it.
//!
//! Integer rules parse like Python's `int(text.strip())` (`"+5"`, `"1_000"` and `"007"` are
//! all numbers) and fail with [`Answer::invalid`] text otherwise.

use std::borrow::Cow;

use crate::engine::Check;
use crate::engine::Ctx;
use crate::engine::pytext::{PyInt, parse_py_int, py_strip};
use crate::messages;

/// What a literal mistake matches: text for text rules, an integer for [`int`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MistakeKey {
    /// An answer text, compared after the rule's normalization.
    Text(String),
    /// An integer, compared after parsing.
    Int(i64),
}

impl From<&str> for MistakeKey {
    fn from(text: &str) -> Self {
        Self::Text(text.to_owned())
    }
}

impl From<String> for MistakeKey {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<i64> for MistakeKey {
    fn from(number: i64) -> Self {
        Self::Int(number)
    }
}

#[derive(Debug, Clone)]
enum Kind {
    Exact(String),
    ExactWith(fn(&Ctx) -> String),
    OneOf(Vec<String>),
    Int(i64),
    Suffix(String),
    Parts {
        separator: String,
        parts: Vec<Answer>,
    },
}

/// A rule that judges a submitted answer. See the [module documentation](self).
///
/// ```
/// use shellgame::engine::answer::{exact, int, parts};
///
/// let rule = exact("navigator")
///     .ignore_case()
///     .mistake("toto není heslo", "To je obsah decoy.txt, ne password.txt.")
///     .error("Heslo není správné.");
/// let both = parts(",", [int(3), exact("log.txt")]).format_message("Formát: počet,soubor");
/// # let _ = (rule, both);
/// ```
#[derive(Debug, Clone)]
pub struct Answer {
    kind: Kind,
    ignore_case: bool,
    trim_slash: bool,
    mistakes: Vec<(MistakeKey, String)>,
    feedback: Option<fn(&str) -> Option<String>>,
    error: Option<String>,
    invalid: Option<String>,
    required: Option<String>,
    format: Option<String>,
    secret: bool,
    accepts: Vec<String>,
    rejects: Vec<String>,
    /// Modifiers that were applied to a kind they do nothing for; reported by `lint`.
    misuse: Vec<String>,
}

/// The answer must equal `expected` (after normalization).
pub fn exact(expected: impl Into<String>) -> Answer {
    Answer::new(Kind::Exact(expected.into()))
}

/// The answer must equal one of the accepted values.
pub fn one_of<I, S>(accepted: I) -> Answer
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    Answer::new(Kind::OneOf(accepted.into_iter().map(Into::into).collect()))
}

/// The answer must be this integer, parsed like Python's `int()`.
pub fn int(expected: i64) -> Answer {
    Answer::new(Kind::Int(expected))
}

/// The answer must equal `expected` or end with `/expected`, so a bare name and a path
/// ending in it are both accepted.
pub fn suffix(expected: impl Into<String>) -> Answer {
    Answer::new(Kind::Suffix(expected.into()))
}

/// The answer is several values joined by `separator`, each judged by its own rule (Python's
/// `TupleAnswer`). The text is split keeping empty items, each item is stripped, a wrong
/// number of items fails with [`Answer::format_message`], and otherwise the first failing
/// part's message wins.
pub fn parts<I>(separator: impl Into<String>, parts: I) -> Answer
where
    I: IntoIterator<Item = Answer>,
{
    Answer::new(Kind::Parts {
        separator: separator.into(),
        parts: parts.into_iter().collect(),
    })
}

/// The answer must equal a value computed from the situation, such as the basename of the
/// player's `$HOME`, which is not known until the game runs.
pub fn exact_with(expected: fn(&Ctx) -> String) -> Answer {
    Answer::new(Kind::ExactWith(expected))
}

impl Answer {
    fn new(kind: Kind) -> Self {
        Self {
            kind,
            ignore_case: false,
            trim_slash: false,
            mistakes: Vec::new(),
            feedback: None,
            error: None,
            invalid: None,
            required: None,
            format: None,
            secret: false,
            accepts: Vec::new(),
            rejects: Vec::new(),
            misuse: Vec::new(),
        }
    }

    fn misused(&mut self, modifier: &str, what: &str) {
        self.misuse
            .push(format!("{modifier} has no effect on {what}"));
    }

    fn is_parts(&self) -> bool {
        matches!(self.kind, Kind::Parts { .. })
    }

    fn is_int_kind(&self) -> bool {
        matches!(self.kind, Kind::Int(_))
    }

    /// Compare case-insensitively (Unicode lowercase on both sides).
    #[must_use]
    pub fn ignore_case(mut self) -> Self {
        if self.is_int_kind() || self.is_parts() {
            self.misused(".ignore_case()", "int() and parts() answers");
        }
        self.ignore_case = true;
        self
    }

    /// After stripping whitespace, also drop trailing `/`, so `delta/` counts as `delta`.
    #[must_use]
    pub fn trim_trailing_slash(mut self) -> Self {
        if self.is_int_kind() || self.is_parts() {
            self.misused(".trim_trailing_slash()", "int() and parts() answers");
        }
        self.trim_slash = true;
        self
    }

    /// A tailored message for one specific wrong answer. Text rules match it exactly after
    /// the rule's normalization (strip, trailing slash, case folding); for [`int`] the key
    /// is an integer. Mistakes are checked before the correct answer and in the order
    /// declared.
    #[must_use]
    pub fn mistake(mut self, key: impl Into<MistakeKey>, message: impl Into<String>) -> Self {
        let key = key.into();
        let problem = match (&self.kind, &key) {
            (Kind::Parts { .. }, _) => Some(".mistake() has no effect on parts() answers"),
            (Kind::Int(_), MistakeKey::Text(_)) => Some(".mistake() on int() needs an integer key"),
            (Kind::Int(_), MistakeKey::Int(_)) => None,
            (_, MistakeKey::Int(_)) => Some(".mistake() with an integer key needs an int() answer"),
            _ => None,
        };
        if let Some(problem) = problem {
            self.misuse.push(problem.to_owned());
        }
        self.mistakes.push((key, message.into()));
        self
    }

    /// Computed feedback for a wrong answer. It runs after literal mistakes and after the
    /// correct-answer check, receives the normalized answer (stripped, trailing slash
    /// trimmed if enabled, case as typed) and returns `Some(message)` to override the
    /// error text, or `None` to fall through to it.
    #[must_use]
    pub fn feedback(mut self, feedback: fn(&str) -> Option<String>) -> Self {
        if self.is_parts() {
            self.misused(".feedback()", "parts() answers");
        }
        self.feedback = Some(feedback);
        self
    }

    /// The message for a wrong answer, instead of [`messages::INCORRECT`]. It must not
    /// reveal the answer.
    #[must_use]
    pub fn error(mut self, message: impl Into<String>) -> Self {
        if self.is_parts() {
            self.misused(
                ".error()",
                "parts() answers (use .format_message() or the parts' own rules)",
            );
        }
        self.error = Some(message.into());
        self
    }

    /// The message when an [`int`] answer is not a number, instead of
    /// [`messages::EXPECTED_INTEGER`].
    #[must_use]
    pub fn invalid(mut self, message: impl Into<String>) -> Self {
        if !self.is_int_kind() {
            self.misused(".invalid()", "answers other than int()");
        }
        self.invalid = Some(message.into());
        self
    }

    /// The message when the player submits no answer at all, instead of
    /// [`messages::ANSWER_REQUIRED`] ([`messages::ANSWER_REQUIRED_NUMBER`] for [`int`]).
    #[must_use]
    pub fn required(mut self, message: impl Into<String>) -> Self {
        self.required = Some(message.into());
        self
    }

    /// The message of a [`parts`] answer with the wrong number of parts, instead of
    /// [`messages::ANSWER_FORMAT`]. Not the same as [`Answer::error`], which parts answers
    /// do not support: a wrong value is judged by the parts themselves.
    #[must_use]
    pub fn format_message(mut self, message: impl Into<String>) -> Self {
        if !self.is_parts() {
            self.misused(".format_message()", "answers other than parts()");
        }
        self.format = Some(message.into());
        self
    }

    /// Mark the answer as a secret: the catalog lint then checks that it never appears in
    /// hints or in a `shellgame submit <answer>` example in the text.
    #[must_use]
    pub fn secret(mut self) -> Self {
        self.secret = true;
        self
    }

    /// Example answers that must be accepted. Checked by the catalog lint, never at run
    /// time; they document (and pin) what the rule means.
    #[must_use]
    pub fn accepts<I, S>(mut self, examples: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.accepts.extend(examples.into_iter().map(Into::into));
        self
    }

    /// Example answers that must be rejected. Checked by the catalog lint.
    #[must_use]
    pub fn rejects<I, S>(mut self, examples: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.rejects.extend(examples.into_iter().map(Into::into));
        self
    }

    /// Judge an answer. Feedback is data: a wrong answer is `Check::Fail(text)`.
    pub fn check(&self, ctx: &Ctx, answer: &str) -> Check {
        match &self.kind {
            Kind::Parts { separator, parts } => self.check_parts(ctx, separator, parts, answer),
            Kind::Int(expected) => self.check_int(*expected, answer),
            _ => self.check_text(ctx, answer),
        }
    }

    /// What to tell a player who submits nothing for a level that needs an answer.
    pub fn required_message(&self) -> &str {
        match (&self.required, &self.kind) {
            (Some(message), _) => message,
            (None, Kind::Int(_)) => messages::ANSWER_REQUIRED_NUMBER,
            (None, _) => messages::ANSWER_REQUIRED,
        }
    }

    fn fold<'s>(&self, text: &'s str) -> Cow<'s, str> {
        if self.ignore_case {
            Cow::Owned(text.to_lowercase())
        } else {
            Cow::Borrowed(text)
        }
    }

    /// The wrong-answer outcome: computed feedback if it has something to say, otherwise
    /// the error text.
    fn wrong(&self, normalized: &str) -> Check {
        if let Some(feedback) = self.feedback
            && let Some(message) = feedback(normalized)
        {
            return Check::Fail(message);
        }
        Check::Fail(
            self.error
                .clone()
                .unwrap_or_else(|| messages::INCORRECT.to_owned()),
        )
    }

    fn check_text(&self, ctx: &Ctx, answer: &str) -> Check {
        let mut normalized = py_strip(answer);
        if self.trim_slash {
            normalized = normalized.trim_end_matches('/');
        }
        let folded = self.fold(normalized);
        for (key, message) in &self.mistakes {
            if let MistakeKey::Text(text) = key
                && self.fold(text) == folded
            {
                return Check::Fail(message.clone());
            }
        }
        if self.matches_expected(ctx, &folded) {
            return Check::Pass;
        }
        self.wrong(normalized)
    }

    fn matches_expected(&self, ctx: &Ctx, folded: &str) -> bool {
        match &self.kind {
            Kind::Exact(expected) => self.fold(expected) == folded,
            Kind::ExactWith(expected) => self.fold(&expected(ctx)) == folded,
            Kind::OneOf(accepted) => accepted.iter().any(|value| self.fold(value) == folded),
            Kind::Suffix(expected) => {
                let expected = self.fold(expected);
                folded == expected || folded.ends_with(&format!("/{expected}"))
            }
            Kind::Int(_) | Kind::Parts { .. } => false,
        }
    }

    fn check_int(&self, expected: i64, answer: &str) -> Check {
        let normalized = py_strip(answer);
        let value = match parse_py_int(normalized) {
            PyInt::Invalid => {
                return Check::Fail(
                    self.invalid
                        .clone()
                        .unwrap_or_else(|| messages::EXPECTED_INTEGER.to_owned()),
                );
            }
            PyInt::TooBig => None,
            PyInt::Value(number) => Some(number),
        };
        if let Some(number) = value {
            for (key, message) in &self.mistakes {
                if *key == MistakeKey::Int(number) {
                    return Check::Fail(message.clone());
                }
            }
            if number == expected {
                return Check::Pass;
            }
        }
        self.wrong(normalized)
    }

    fn check_parts(&self, ctx: &Ctx, separator: &str, parts: &[Answer], answer: &str) -> Check {
        let values: Vec<&str> = if separator.is_empty() {
            vec![py_strip(answer)]
        } else {
            answer.split(separator).map(py_strip).collect()
        };
        if values.len() != parts.len() {
            return Check::Fail(
                self.format
                    .clone()
                    .unwrap_or_else(|| messages::ANSWER_FORMAT.to_owned()),
            );
        }
        for (value, rule) in values.into_iter().zip(parts) {
            if let failure @ Check::Fail(_) = rule.check(ctx, value) {
                return failure;
            }
        }
        Check::Pass
    }

    /// True when the answer (or one of its parts) depends on the situation, so it cannot be
    /// known without a [`Ctx`].
    pub fn is_dynamic(&self) -> bool {
        match &self.kind {
            Kind::ExactWith(_) => true,
            Kind::Parts { parts, .. } => parts.iter().any(Answer::is_dynamic),
            _ => false,
        }
    }

    /// True when marked [`Answer::secret`].
    pub fn is_secret(&self) -> bool {
        self.secret
    }

    /// One correct answer, as a player would type it, when it is known statically: the
    /// expected text, the first of [`one_of`], the number, the suffix itself, the parts
    /// joined by the separator. `None` for [`exact_with`] (and anything containing it).
    pub fn static_answer(&self) -> Option<String> {
        match &self.kind {
            Kind::Exact(expected) | Kind::Suffix(expected) => Some(expected.clone()),
            Kind::OneOf(accepted) => accepted.first().cloned(),
            Kind::Int(expected) => Some(expected.to_string()),
            Kind::ExactWith(_) => None,
            Kind::Parts { separator, parts } => parts
                .iter()
                .map(Answer::static_answer)
                .collect::<Option<Vec<_>>>()
                .map(|values| values.join(separator)),
        }
    }

    /// The `accepts` examples of this rule (not of its parts).
    pub fn accepted_examples(&self) -> &[String] {
        &self.accepts
    }

    /// The `rejects` examples of this rule (not of its parts).
    pub fn rejected_examples(&self) -> &[String] {
        &self.rejects
    }

    /// This answer and, recursively, every part of it.
    pub fn nodes(&self) -> Vec<&Answer> {
        let mut nodes = vec![self];
        if let Kind::Parts { parts, .. } = &self.kind {
            for part in parts {
                nodes.extend(part.nodes());
            }
        }
        nodes
    }

    /// Every statically known correct answer in this tree (all accepted values, numbers,
    /// suffixes, each part's, and the joined text of every parts rule).
    pub fn expected_texts(&self) -> Vec<String> {
        let mut texts = Vec::new();
        for node in self.nodes() {
            match &node.kind {
                Kind::Exact(expected) | Kind::Suffix(expected) => texts.push(expected.clone()),
                Kind::OneOf(accepted) => texts.extend(accepted.iter().cloned()),
                Kind::Int(expected) => texts.push(expected.to_string()),
                Kind::ExactWith(_) => {}
                Kind::Parts { .. } => texts.extend(node.static_answer()),
            }
        }
        texts
    }

    /// Every message this tree can show a player, apart from computed feedback.
    pub fn message_texts(&self) -> Vec<&str> {
        let mut texts = Vec::new();
        for node in self.nodes() {
            texts.extend(
                [&node.error, &node.invalid, &node.required, &node.format]
                    .into_iter()
                    .flatten()
                    .map(String::as_str),
            );
            texts.extend(node.mistakes.iter().map(|(_, message)| message.as_str()));
        }
        texts
    }

    /// Problems with how the rule was built: modifiers that do nothing, impossible rules.
    pub fn lint(&self) -> Vec<String> {
        let mut problems = Vec::new();
        for node in self.nodes() {
            problems.extend(node.misuse.iter().cloned());
            match &node.kind {
                Kind::OneOf(accepted) if accepted.is_empty() => {
                    problems.push("one_of() needs at least one accepted value".to_owned());
                }
                Kind::Parts { separator, parts } => {
                    if separator.is_empty() {
                        problems.push("parts() needs a non-empty separator".to_owned());
                    }
                    if parts.is_empty() {
                        problems.push("parts() needs at least one part".to_owned());
                    }
                }
                _ => {}
            }
        }
        problems
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::LevelId;
    use std::path::{Path, PathBuf};

    fn check(answer: &Answer, text: &str) -> Check {
        let home = Path::new("/home/ada");
        let ctx = Ctx::new(
            Path::new("/ws"),
            PathBuf::from("/ws/level-1"),
            None,
            home,
            LevelId::new(1, 1),
        );
        answer.check(&ctx, text)
    }

    fn passes(answer: &Answer, text: &str) -> bool {
        check(answer, text) == Check::Pass
    }

    fn fails_with(answer: &Answer, text: &str, message: &str) {
        assert_eq!(check(answer, text), Check::fail(message), "{text:?}");
    }

    #[test]
    fn exact_matches_after_stripping_whitespace() {
        let rule = exact("level-1");
        assert!(passes(&rule, "level-1"));
        assert!(passes(&rule, "  level-1\n"));
        assert!(passes(&rule, "\t\u{a0}level-1\u{3000}"));
        assert!(
            passes(&rule, "\u{1c}level-1\u{1f}"),
            "Python strips \\x1c..\\x1f"
        );
        assert!(!passes(&rule, "level 1"));
        assert!(!passes(&rule, "level-1/"), "no slash trimming unless asked");
        assert!(!passes(&rule, ""));
    }

    #[test]
    fn the_default_wrong_answer_text_is_terse_and_never_reveals_the_answer() {
        let rule = exact("hunter2");
        fails_with(&rule, "wrong", messages::INCORRECT);
        let Check::Fail(text) = check(&rule, "wrong") else {
            panic!("expected failure")
        };
        assert_eq!(text, "Nesprávně.");
        assert!(!text.contains("hunter2"));
    }

    #[test]
    fn exact_is_case_sensitive_by_default() {
        let rule = exact("Delta");
        assert!(passes(&rule, "Delta"));
        assert!(!passes(&rule, "delta"));
        assert!(!passes(&rule, "DELTA"));
    }

    #[test]
    fn ignore_case_folds_both_sides_including_non_ascii() {
        let rule = exact("Český").ignore_case();
        assert!(passes(&rule, "český"));
        assert!(passes(&rule, "ČESKÝ"));
        assert!(passes(&rule, "  čESKý "));
        assert!(!passes(&rule, "cesky"));
        // The expected side is folded too.
        assert!(passes(&exact("NAVIGATOR").ignore_case(), "navigator"));
    }

    #[test]
    fn trim_trailing_slash_applies_after_strip() {
        let rule = exact("delta").trim_trailing_slash();
        assert!(passes(&rule, "delta"));
        assert!(passes(&rule, "delta/"));
        assert!(passes(&rule, "delta//"));
        assert!(passes(&rule, "  delta/  "));
        assert!(!passes(&rule, "delta/x"));
        assert!(!passes(&rule, "/delta"));
        // Only when enabled; and the bare root becomes empty, as Python's rstrip("/").
        assert!(passes(&exact("").trim_trailing_slash(), "/"));
        assert!(!passes(&exact("delta"), "delta/"));
    }

    #[test]
    fn literal_mistakes_give_tailored_messages() {
        let rule = exact("delta")
            .mistake("data", "'data' je soubor.")
            .mistake("dome", "'dome' nekončí na 'a'.");
        fails_with(&rule, "data", "'data' je soubor.");
        fails_with(&rule, " dome\n", "'dome' nekončí na 'a'.");
        fails_with(&rule, "other", messages::INCORRECT);
        assert!(passes(&rule, "delta"));
    }

    #[test]
    fn mistakes_match_after_the_same_normalization_as_the_answer() {
        let rule = exact("delta")
            .ignore_case()
            .trim_trailing_slash()
            .mistake("Dome", "dome");
        fails_with(&rule, "DOME/", "dome");
        fails_with(&rule, "dome", "dome");
    }

    #[test]
    fn mistakes_come_before_the_correct_check_in_declaration_order() {
        // A mistake that equals the answer shadows it: the order is the contract.
        let rule = exact("x").mistake("x", "first").mistake("x", "second");
        fails_with(&rule, "x", "first");
    }

    #[test]
    fn feedback_runs_after_mistakes_and_after_the_correct_check() {
        fn shape(answer: &str) -> Option<String> {
            if answer.starts_with('d') && !answer.ends_with('a') {
                Some(format!("'{answer}' začíná na 'd', ale nekončí na 'a'."))
            } else {
                None
            }
        }
        let rule = exact("delta")
            .mistake("dome", "literal mistake")
            .feedback(shape)
            .error("custom error");

        assert!(
            passes(&rule, "delta"),
            "correct answers never reach feedback"
        );
        fails_with(&rule, "dome", "literal mistake");
        fails_with(&rule, "dax", "'dax' začíná na 'd', ale nekončí na 'a'.");
        fails_with(&rule, "xyz", "custom error");
    }

    #[test]
    fn feedback_receives_the_normalized_answer_with_case_preserved() {
        fn echo(answer: &str) -> Option<String> {
            Some(format!("<{answer}>"))
        }
        let rule = exact("a")
            .trim_trailing_slash()
            .ignore_case()
            .feedback(echo);
        fails_with(&rule, "  B//  ", "<B>");
        let plain = exact("a").feedback(echo);
        fails_with(&plain, " B/ ", "<B/>");
    }

    #[test]
    fn feedback_returning_none_falls_back_to_the_error_text() {
        fn nothing(_: &str) -> Option<String> {
            None
        }
        fails_with(&exact("a").feedback(nothing), "b", messages::INCORRECT);
        fails_with(&exact("a").feedback(nothing).error("e"), "b", "e");
    }

    #[test]
    fn error_replaces_the_default_text() {
        fails_with(
            &exact("a").error("Heslo není správné."),
            "b",
            "Heslo není správné.",
        );
    }

    #[test]
    fn one_of_accepts_any_listed_value() {
        let rule = one_of(["seznam.txt", "seznam"]);
        assert!(passes(&rule, "seznam.txt"));
        assert!(passes(&rule, " seznam "));
        assert!(!passes(&rule, "SEZNAM"));
        assert!(passes(&one_of(["A", "b"]).ignore_case(), "a"));
        assert!(passes(&one_of(["A", "b"]).ignore_case(), "B"));
        fails_with(&rule, "other", messages::INCORRECT);
        assert!(!passes(&one_of(Vec::<String>::new()), ""));
    }

    #[test]
    fn one_of_supports_mistakes_and_errors() {
        let rule = one_of(["a", "b"]).mistake("c", "c je blízko").error("ne");
        fails_with(&rule, "c", "c je blízko");
        fails_with(&rule, "d", "ne");
    }

    #[test]
    fn suffix_accepts_the_name_or_a_path_ending_in_it() {
        let rule = suffix("c");
        assert!(passes(&rule, "c"));
        assert!(passes(&rule, "/tmp/shellgame/level-1/gamma/deep/a/b/c"));
        assert!(passes(&rule, "b/c"));
        assert!(
            !passes(&rule, "abc"),
            "a mere string suffix is not a path suffix"
        );
        assert!(!passes(&rule, "c/"));
        assert!(!passes(&rule, "b/cc"));
        assert!(passes(&suffix("File.TXT").ignore_case(), "/x/file.txt"));
        assert!(passes(&suffix("c").trim_trailing_slash(), "/x/c/"));
    }

    #[test]
    fn int_parses_like_python() {
        let rule = int(5);
        for text in ["5", "+5", " 5 ", "0005", "00_5", "5\n", "\u{1c}5\u{1c}"] {
            assert!(passes(&rule, text), "{text:?}");
        }
        assert!(passes(&int(1000), "1_000"));
        assert!(passes(&int(7), "007"));
        assert!(passes(&int(-3), "-3"));
        assert!(passes(&int(42), " 42 "));
        assert!(passes(&int(0), "-0"));
        for text in [
            "", "abc", "5.0", "5a", "1__0", "_5", "5_", "+ 5", "--5", "0x5", "1e1",
        ] {
            fails_with(&rule, text, messages::EXPECTED_INTEGER);
        }
        fails_with(&rule, "6", messages::INCORRECT);
        fails_with(&int(10), "1__0", messages::EXPECTED_INTEGER);
    }

    #[test]
    fn int_accepts_any_unicode_decimal_digit_like_python() {
        // A student with a CJK input method types fullwidth digits; Python's int() took them.
        assert!(passes(&int(42), "\u{ff14}\u{ff12}"));
        assert!(passes(&int(42), " \u{ff14}\u{ff12} "));
        assert!(passes(&int(3), "\u{0663}"));
        assert!(passes(&int(-5), "-\u{ff15}"));
        // Numerals that are not decimal digits stay invalid, as in Python.
        fails_with(&int(2), "\u{b2}", messages::EXPECTED_INTEGER);
        fails_with(&int(3), "\u{4e09}", messages::EXPECTED_INTEGER);
    }

    #[test]
    fn int_mistakes_are_integers_and_run_before_the_correct_check() {
        let rule = int(12345)
            .mistake(1234, "to je o řád méně")
            .mistake(-1, "záporné číslo")
            .error("jiné číslo");
        fails_with(&rule, "1234", "to je o řád méně");
        fails_with(&rule, "+1_234", "to je o řád méně");
        fails_with(&rule, "-1", "záporné číslo");
        fails_with(&rule, "99", "jiné číslo");
        assert!(passes(&rule, "12345"));
    }

    #[test]
    fn int_invalid_message_is_customizable() {
        let rule = int(3).invalid("Zadejte číslo, ne text.");
        fails_with(&rule, "tři", "Zadejte číslo, ne text.");
        fails_with(&rule, "4", messages::INCORRECT);
    }

    #[test]
    fn int_beyond_i64_is_a_wrong_number_not_a_parse_error() {
        fails_with(&int(1), "99999999999999999999", messages::INCORRECT);
        fails_with(&int(1), &"9".repeat(4301), messages::EXPECTED_INTEGER);
    }

    #[test]
    fn int_feedback_receives_the_stripped_text() {
        fn hint(answer: &str) -> Option<String> {
            Some(format!("got {answer}"))
        }
        fails_with(&int(1).feedback(hint), "  +7 ", "got +7");
    }

    #[test]
    fn required_messages_have_the_documented_defaults() {
        assert_eq!(exact("a").required_message(), messages::ANSWER_REQUIRED);
        assert_eq!(one_of(["a"]).required_message(), messages::ANSWER_REQUIRED);
        assert_eq!(suffix("a").required_message(), messages::ANSWER_REQUIRED);
        assert_eq!(int(1).required_message(), messages::ANSWER_REQUIRED_NUMBER);
        assert_eq!(
            parts(",", [int(1), int(2)]).required_message(),
            messages::ANSWER_REQUIRED
        );
        assert_eq!(
            exact("a").required("Zadejte název.").required_message(),
            "Zadejte název."
        );
        assert_eq!(int(1).required("Číslo!").required_message(), "Číslo!");
    }

    #[test]
    fn parts_judges_each_part_with_its_own_rule() {
        let rule = parts(",", [int(3), exact("SECRET").ignore_case()]);
        assert!(passes(&rule, "3,secret"));
        assert!(passes(&rule, " 3 , Secret "));
        fails_with(&rule, "4,secret", messages::INCORRECT);
        fails_with(&rule, "3,nope", messages::INCORRECT);
        fails_with(&rule, "x,secret", messages::EXPECTED_INTEGER);
    }

    #[test]
    fn parts_reports_the_first_failing_part() {
        let rule = parts(
            ",",
            [
                int(1).error("první špatně"),
                int(2).error("druhá špatně"),
                int(3).error("třetí špatně"),
            ],
        );
        fails_with(&rule, "1,9,9", "druhá špatně");
        fails_with(&rule, "9,9,9", "první špatně");
        assert!(passes(&rule, "1,2,3"));
    }

    #[test]
    fn parts_with_a_wrong_count_fails_with_the_format_message() {
        let rule = parts(",", [int(1), int(2)]);
        for text in ["", "1", "1,2,3", "1,2,", ",1,2", "1;2"] {
            fails_with(&rule, text, messages::ANSWER_FORMAT);
        }
        let custom = parts(",", [int(1), int(2)]).format_message("Formát: počet,kód (např. 5,X)");
        fails_with(&custom, "1", "Formát: počet,kód (např. 5,X)");
        fails_with(&custom, "1,2,3", "Formát: počet,kód (např. 5,X)");
        // The custom format message does not leak into wrong values.
        fails_with(&custom, "1,9", messages::INCORRECT);
    }

    #[test]
    fn parts_keeps_empty_items_and_strips_each() {
        let rule = parts(",", [exact(""), exact("x")]);
        assert!(passes(&rule, ",x"));
        assert!(passes(&rule, "  , x  "));
        fails_with(&rule, "x", messages::ANSWER_FORMAT);
        assert!(passes(&parts(",", [exact("")]), ""));
    }

    #[test]
    fn parts_supports_other_separators_and_nesting() {
        let rule = parts(";", [exact("a"), parts("|", [int(1), int(2)])]);
        assert!(passes(&rule, "a;1|2"));
        fails_with(&rule, "a;1|3", messages::INCORRECT);
        fails_with(&rule, "a;1", messages::ANSWER_FORMAT);
        let long = parts(" -> ", [exact("a"), exact("b")]);
        assert!(passes(&long, "a -> b"));
    }

    #[test]
    fn parts_uses_the_parts_own_mistakes_and_feedback() {
        fn fb(_: &str) -> Option<String> {
            Some("feedback".to_owned())
        }
        let rule = parts(
            ",",
            [exact("a").mistake("b", "b není a"), exact("c").feedback(fb)],
        );
        fails_with(&rule, "b,c", "b není a");
        fails_with(&rule, "a,z", "feedback");
    }

    #[test]
    fn exact_with_computes_the_expected_value_from_the_context() {
        let rule = exact_with(|ctx| ctx.home_name());
        assert!(passes(&rule, "ada"));
        assert!(passes(&rule, " ada "));
        assert!(!passes(&rule, "bob"));
        assert!(passes(
            &exact_with(|ctx| ctx.home_name().to_uppercase()).ignore_case(),
            "ADA"
        ));
        let in_parts = parts(",", [exact_with(|ctx| ctx.home_name()), int(1)]);
        assert!(passes(&in_parts, "ada,1"));
    }

    #[test]
    fn static_answers_cover_every_kind() {
        assert_eq!(exact("a").static_answer().as_deref(), Some("a"));
        assert_eq!(one_of(["x", "y"]).static_answer().as_deref(), Some("x"));
        assert_eq!(int(-4).static_answer().as_deref(), Some("-4"));
        assert_eq!(suffix("c").static_answer().as_deref(), Some("c"));
        assert_eq!(
            parts(",", [int(2), exact("log")])
                .static_answer()
                .as_deref(),
            Some("2,log")
        );
        assert_eq!(exact_with(|_| String::new()).static_answer(), None);
        assert_eq!(
            parts(",", [int(2), exact_with(|_| String::new())]).static_answer(),
            None
        );
        assert!(exact_with(|_| String::new()).is_dynamic());
        assert!(parts(",", [int(2), exact_with(|_| String::new())]).is_dynamic());
        assert!(!parts(",", [int(2)]).is_dynamic());
    }

    #[test]
    fn a_static_answer_is_always_accepted_by_its_own_rule() {
        for rule in [
            exact("x y"),
            exact("Mixed").ignore_case(),
            one_of(["a", "b"]),
            int(12),
            suffix("tail"),
            parts(",", [int(3), exact("ab")]),
            parts(";", [one_of(["p", "q"]), suffix("s")]),
        ] {
            let answer = rule.static_answer().unwrap();
            assert!(passes(&rule, &answer), "{answer:?}");
        }
    }

    #[test]
    fn expected_texts_and_messages_cover_nested_parts() {
        let rule = parts(
            ",",
            [
                int(12345).error("číslo je jiné"),
                exact("SECRET").mistake("x", "x ne"),
            ],
        )
        .format_message("Formát: n,heslo");
        let mut expected = rule.expected_texts();
        expected.sort();
        assert_eq!(expected, ["12345", "12345,SECRET", "SECRET"]);
        let mut texts = rule.message_texts();
        texts.sort_unstable();
        assert_eq!(texts, ["Formát: n,heslo", "x ne", "číslo je jiné"]);
    }

    #[test]
    fn nodes_lists_the_rule_and_its_parts() {
        let rule = parts(",", [int(1), parts("|", [int(2), int(3)])]);
        assert_eq!(rule.nodes().len(), 5);
        assert_eq!(exact("a").nodes().len(), 1);
    }

    #[test]
    fn examples_and_secret_are_recorded() {
        let rule = exact("a").accepts(["a", " A "]).rejects(["b"]).secret();
        assert_eq!(rule.accepted_examples(), ["a", " A "]);
        assert_eq!(rule.rejected_examples(), ["b"]);
        assert!(rule.is_secret());
        assert!(!exact("a").is_secret());
    }

    #[test]
    fn lint_flags_modifiers_that_do_nothing() {
        let clean = exact("a").ignore_case().trim_trailing_slash().error("x");
        assert!(clean.lint().is_empty());

        let on_parts = parts(",", [int(1)])
            .error("e")
            .mistake("a", "m")
            .ignore_case()
            .trim_trailing_slash()
            .invalid("i")
            .feedback(|_| None);
        assert_eq!(on_parts.lint().len(), 6, "{:?}", on_parts.lint());

        assert_eq!(int(1).ignore_case().trim_trailing_slash().lint().len(), 2);
        assert_eq!(int(1).mistake("one", "m").lint().len(), 1);
        assert_eq!(exact("a").mistake(1, "m").lint().len(), 1);
        assert_eq!(exact("a").invalid("i").lint().len(), 1);
        assert_eq!(exact("a").format_message("f").lint().len(), 1);
        assert!(int(1).mistake(2, "m").invalid("i").lint().is_empty());
    }

    #[test]
    fn lint_flags_impossible_rules() {
        assert_eq!(one_of(Vec::<String>::new()).lint().len(), 1);
        assert_eq!(parts("", [int(1)]).lint().len(), 1);
        assert_eq!(parts(",", Vec::<Answer>::new()).lint().len(), 1);
        // Problems inside a part are found too.
        assert_eq!(parts(",", [exact("a").invalid("i")]).lint().len(), 1);
    }

    #[test]
    fn mistake_keys_convert_from_text_and_integers() {
        assert_eq!(MistakeKey::from("a"), MistakeKey::Text("a".to_owned()));
        assert_eq!(
            MistakeKey::from(String::from("a")),
            MistakeKey::Text("a".to_owned())
        );
        assert_eq!(MistakeKey::from(5), MistakeKey::Int(5));
    }
}
