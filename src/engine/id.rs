//! Level identifiers.

use std::fmt;
use std::str::FromStr;

/// The stable identifier of a level: `section.number`, such as `1.4` or `10.4`.
///
/// Identifiers are explicit and never renumbered, so saved games keep working when levels
/// are reordered and the gap at 1.11 stays. Ordering is numeric on `(section, number)`:
/// `1.12` sorts after `1.10`, which string comparison would get wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LevelId {
    /// The section number; 0 is the game introduction.
    pub section: u16,
    /// The level number within the section; 0 is the section intro.
    pub number: u16,
}

impl LevelId {
    /// Build an identifier from its two parts.
    pub const fn new(section: u16, number: u16) -> Self {
        Self { section, number }
    }

    /// True for `N.0`, the intro page of section `N`.
    pub const fn is_intro(self) -> bool {
        self.number == 0
    }

    /// The intro level of this level's section.
    pub const fn section_intro(self) -> Self {
        Self::new(self.section, 0)
    }
}

impl fmt::Display for LevelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.section, self.number)
    }
}

/// A string is not a level identifier of the form `S.N`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLevelIdError {
    input: String,
}

impl fmt::Display for ParseLevelIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a level id (expected S.N): {:?}", self.input)
    }
}

impl std::error::Error for ParseLevelIdError {}

impl FromStr for LevelId {
    type Err = ParseLevelIdError;

    /// Parse `S.N` where both parts are non-empty ASCII digit strings. Like the Python
    /// `parse_level_id`, leading zeros are accepted (`"01.002"` is 1.2); anything else,
    /// including `"1.2.3"`, `"1."` and signs, is an error.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let error = || ParseLevelIdError {
            input: value.to_owned(),
        };
        let (section, number) = value.split_once('.').ok_or_else(error)?;
        let digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
        if !digits(section) || !digits(number) {
            return Err(error());
        }
        Ok(Self {
            section: section.parse().map_err(|_| error())?,
            number: number.parse().map_err(|_| error())?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_as_section_dot_number() {
        assert_eq!(LevelId::new(1, 4).to_string(), "1.4");
        assert_eq!(LevelId::new(10, 0).to_string(), "10.0");
    }

    #[test]
    fn parses_what_it_displays() {
        for id in [
            LevelId::new(0, 0),
            LevelId::new(1, 11),
            LevelId::new(10, 4),
            LevelId::new(65535, 65535),
        ] {
            assert_eq!(id.to_string().parse::<LevelId>(), Ok(id));
        }
    }

    #[test]
    fn rejects_malformed_ids() {
        for bad in [
            "", ".", "1", "1.", ".1", "1.2.3", "a.b", "1.b", "-1.2", "+1.2", "1.-2", " 1.2",
            "1.2 ", "1,2", "65536.1", "1.65536",
        ] {
            assert!(bad.parse::<LevelId>().is_err(), "{bad:?}");
        }
        assert!(
            "x".parse::<LevelId>()
                .unwrap_err()
                .to_string()
                .contains("\"x\"")
        );
    }

    #[test]
    fn accepts_leading_zeros_like_python() {
        assert_eq!("01.002".parse::<LevelId>(), Ok(LevelId::new(1, 2)));
    }

    #[test]
    fn orders_numerically_not_lexically() {
        let mut ids: Vec<LevelId> = ["1.12", "1.2", "1.10", "10.1", "2.0", "1.1"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        ids.sort();
        let sorted: Vec<String> = ids.iter().map(ToString::to_string).collect();
        assert_eq!(sorted, ["1.1", "1.2", "1.10", "1.12", "2.0", "10.1"]);
    }

    #[test]
    fn intro_helpers() {
        assert!(LevelId::new(3, 0).is_intro());
        assert!(!LevelId::new(3, 1).is_intro());
        assert_eq!(LevelId::new(3, 7).section_intro(), LevelId::new(3, 0));
    }
}
