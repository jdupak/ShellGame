//! How a submitted answer is judged. Order: trim, and drop trailing slashes (Tab completion adds
//! one to a directory name) -> literal mistakes -> correct? -> `error` or "Nesprávně." The default
//! wrong text never contains the answer; a hint about it belongs in the level's hints.

use super::messages::{self, or};

#[derive(Clone, Copy)]
enum Kind {
    None,
    Exact(&'static str),
    OneOf(&'static [&'static str]),
    Int(i64),
    Suffix(&'static str),
    Parts(&'static str, &'static [Answer]),
}

/// The texts are empty when the default applies.
#[derive(Clone, Copy)]
pub struct Answer {
    kind: Kind,
    nocase: bool,
    wrong: &'static [(&'static str, &'static str)],
    error: &'static str,
    format: &'static str,
}

/// The level takes no answer.
pub const NO_ANSWER: Answer =
    Answer { kind: Kind::None, nocase: false, wrong: &[], error: "", format: "" };

#[rustfmt::skip]
mod vocabulary {
    use super::*;

    const fn new(kind: Kind) -> Answer { Answer { kind, ..NO_ANSWER } }

    pub const fn exact(expected: &'static str) -> Answer { new(Kind::Exact(expected)) }
    pub const fn one_of(accepted: &'static [&'static str]) -> Answer { new(Kind::OneOf(accepted)) }
    /// A text that is not a number is a wrong answer like any other.
    pub const fn int(expected: i64) -> Answer { new(Kind::Int(expected)) }
    /// The expected text, or a path ending in `/expected`.
    pub const fn suffix(expected: &'static str) -> Answer { new(Kind::Suffix(expected)) }
    /// Several values joined by `separator`, each judged by its own rule; the first failure wins.
    pub const fn parts(separator: &'static str, parts: &'static [Answer]) -> Answer { new(Kind::Parts(separator, parts)) }

    impl Answer {
        pub const fn nocase(self) -> Self { Self { nocase: true, ..self } }
        /// Tailored messages for specific wrong answers (compared like the answer, integers by
        /// value), checked before the correct answer in the order given.
        pub const fn mistakes(self, wrong: &'static [(&'static str, &'static str)]) -> Self { Self { wrong, ..self } }
        /// The wrong-answer text instead of "Nesprávně."
        pub const fn error(self, error: &'static str) -> Self { Self { error, ..self } }
        /// How to submit: the text when no answer is given, and for a `parts` answer with the
        /// wrong number of parts.
        pub const fn format(self, format: &'static str) -> Self { Self { format, ..self } }
    }
}
pub use vocabulary::*;

impl Answer {
    /// `None` is a submission without an answer.
    pub fn judge(&self, answer: Option<&str>) -> Result<(), String> {
        match (self.kind, answer) {
            (Kind::None, _) => Ok(()),
            (_, Some(text)) => self.verdict(text),
            (_, None) => Err(or(self.format, messages::ANSWER_REQUIRED)),
        }
    }

    fn verdict(&self, text: &str) -> Result<(), String> {
        let text = text.trim().trim_end_matches('/');
        let fold = |s: &str| {
            if self.nocase { s.to_lowercase() } else { s.to_owned() }
        };
        let got = fold(text);
        let number = text.parse::<i64>().ok();
        let known = self.wrong.iter().find(|(key, _)| match self.kind {
            Kind::Int(_) => number.is_some_and(|n| key.parse() == Ok(n)),
            _ => fold(key) == got,
        });
        let correct = match self.kind {
            _ if known.is_some() => false,
            Kind::Parts(separator, parts) => {
                let values: Vec<&str> = text.split(separator).collect();
                if values.len() != parts.len() {
                    return Err(or(self.format, messages::ANSWER_FORMAT));
                }
                return parts.iter().zip(values).try_for_each(|(rule, value)| rule.verdict(value));
            }
            Kind::Int(expected) => number == Some(expected),
            Kind::Exact(expected) => fold(expected) == got,
            Kind::OneOf(accepted) => accepted.iter().any(|value| fold(value) == got),
            Kind::Suffix(expected) => {
                let expected = fold(expected);
                got == expected || got.ends_with(&format!("/{expected}"))
            }
            Kind::None => true,
        };
        if correct {
            return Ok(());
        }
        let message = known.map(|(_, message)| (*message).to_owned());
        Err(message.unwrap_or_else(|| or(self.error, messages::INCORRECT)))
    }

    #[cfg(test)]
    pub fn is_set(&self) -> bool {
        !matches!(self.kind, Kind::None)
    }

    /// One correct answer as a player would type it.
    #[cfg(test)]
    pub fn sample(&self) -> Option<String> {
        match self.kind {
            Kind::Exact(expected) | Kind::Suffix(expected) => Some(expected.to_owned()),
            Kind::OneOf(accepted) => accepted.first().map(|value| (*value).to_owned()),
            Kind::Int(expected) => Some(expected.to_string()),
            Kind::Parts(separator, parts) => {
                let values: Option<Vec<String>> = parts.iter().map(Answer::sample).collect();
                values.map(|values| values.join(separator))
            }
            Kind::None => None,
        }
    }

    /// Every static correct answer and every text a wrong answer can show, this rule's and its parts'.
    #[cfg(test)]
    pub fn secrets_and_texts(&self) -> (Vec<String>, Vec<&'static str>) {
        let (mut secrets, mut texts) = (self.sample().into_iter().collect::<Vec<_>>(), vec![]);
        match self.kind {
            Kind::OneOf(accepted) => {
                secrets.extend(accepted.iter().map(|value| (*value).to_owned()))
            }
            Kind::Parts(_, parts) => {
                for (inner_secrets, inner_texts) in parts.iter().map(Answer::secrets_and_texts) {
                    secrets.extend(inner_secrets);
                    texts.extend(inner_texts);
                }
            }
            _ => {}
        }
        texts.extend([self.error, self.format]);
        texts.extend(self.wrong.iter().map(|(_, message)| *message));
        texts.retain(|text| !text.is_empty());
        (secrets, texts)
    }
}

#[cfg(test)]
mod tests {
    use super::messages::*;
    use super::*;
    use crate::game::testing::err;

    fn judge(answer: Answer, text: &str) -> Result<(), String> {
        answer.judge(Some(text))
    }

    #[test]
    fn text_answers_are_trimmed_folded_and_the_default_text_is_terse() {
        const A: Answer = exact("Delta").nocase();
        assert!(["Delta", "  delta/\n", "DELTA//"].iter().all(|ok| judge(A, ok).is_ok()));
        assert_eq!(judge(exact("Delta"), "delta"), err("Nesprávně."));
        const SUFFIX: Answer = suffix("config/settings.py");
        assert!(
            ["x/config/settings.py", "./config/settings.py"]
                .iter()
                .all(|ok| judge(SUFFIX, ok).is_ok())
        );
        assert!(
            ["settings.py", "config/settings.pyc"].iter().all(|bad| judge(SUFFIX, bad).is_err())
        );
        const ONE: Answer = one_of(&["a", "b"]);
        assert!(judge(ONE, "b").is_ok() && judge(ONE, "c").is_err());
    }

    #[test]
    fn the_order_is_mistakes_then_correct_then_error() {
        const A: Answer = exact("delta")
            .mistakes(&[("delta", "a mistake wins over the answer"), ("dome", "the dome")])
            .error("plain");
        assert_eq!(judge(A, "delta"), err("a mistake wins over the answer"));
        assert_eq!([judge(A, "dome"), judge(A, "x")], [err("the dome"), err("plain")]);
        const B: Answer = exact("Delta").nocase().mistakes(&[("DOME", "dome")]);
        assert_eq!(judge(B, "dome/"), err("dome"));
    }

    #[test]
    fn integers_and_parts_report_what_is_wrong_with_them() {
        const N: Answer = int(5).mistakes(&[("3", "three")]).error("e");
        assert!(["5", " 5 ", "+5"].iter().all(|ok| judge(N, ok).is_ok()));
        assert_eq!([judge(N, "+3"), judge(N, "6")], [err("three"), err("e")]);
        assert!(["x", "5.0", "", "five"].iter().all(|bad| judge(N, bad) == err("e")));
        const P: Answer = parts(",", &[int(2).error("count"), exact("Key").nocase().error("key")])
            .format("shape");
        assert!(judge(P, "2,key").is_ok() && judge(P, " 2 , KEY ").is_ok());
        assert_eq!([judge(P, "2"), judge(P, "2,key,x")], [err("shape"), err("shape")]);
        let wrong = [judge(P, "3,nope"), judge(P, "2,nope"), judge(P, "x,key")];
        assert_eq!(wrong, [err("count"), err("key"), err("count")]);
        const DEFAULT: Answer = parts(",", &[exact("a"), exact("b")]);
        assert_eq!(judge(DEFAULT, "a"), err(ANSWER_FORMAT));
    }

    #[test]
    fn a_missing_answer_shows_the_format_or_asks_for_one() {
        const PARTS: Answer = parts(",", &[exact("a")]);
        for answer in [exact("a"), int(1), PARTS] {
            assert_eq!(answer.judge(None), err(ANSWER_REQUIRED));
        }
        assert_eq!(exact("a").format("say it").judge(None), err("say it"));
        assert!(NO_ANSWER.judge(None).is_ok() && judge(NO_ANSWER, "anything").is_ok());
    }
}
