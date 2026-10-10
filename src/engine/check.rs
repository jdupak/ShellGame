//! The result of checking a player's work.

/// The outcome of a requirement or an answer.
///
/// Feedback for the player is *data*, never an error: a wrong answer is the normal case
/// of this program, not an exceptional one. `Fail` carries the exact text to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    /// The work is correct.
    Pass,
    /// The work is not correct; the text is what the player sees.
    Fail(String),
}

impl Check {
    /// A failure with the given player-facing message.
    pub fn fail(message: impl Into<String>) -> Self {
        Self::Fail(message.into())
    }

    /// True for [`Check::Pass`].
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }

    /// True for [`Check::Fail`].
    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail(_))
    }

    /// The failure message, if this is a failure.
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Pass => None,
            Self::Fail(message) => Some(message),
        }
    }

    /// Run `next` only when this check passed; otherwise keep this failure.
    #[must_use]
    pub fn and_then(self, next: impl FnOnce() -> Check) -> Check {
        match self {
            Self::Pass => next(),
            failure => failure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_and_fail_report_themselves() {
        assert!(Check::Pass.is_pass());
        assert!(!Check::Pass.is_fail());
        assert_eq!(Check::Pass.message(), None);
        let failure = Check::fail("nope");
        assert!(failure.is_fail());
        assert_eq!(failure.message(), Some("nope"));
        assert_eq!(failure, Check::Fail("nope".to_owned()));
    }

    #[test]
    fn and_then_keeps_the_first_failure() {
        assert_eq!(Check::Pass.and_then(|| Check::Pass), Check::Pass);
        assert_eq!(Check::Pass.and_then(|| Check::fail("b")), Check::fail("b"));
        assert_eq!(
            Check::fail("a").and_then(|| panic!("must not run")),
            Check::fail("a")
        );
    }
}
