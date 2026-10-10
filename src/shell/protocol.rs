//! The directive protocol: how the binary changes the state of the wrapped shell.
//!
//! A process cannot change the working directory or the environment of its parent. Instead the
//! binary writes *directives* to its own stderr, and the `shellgame` shell function
//! (`bash.sh`, `fish.fish`) replays them in the player's shell after the binary has exited.
//!
//! ```text
//! __SHELLGAME_EXEC__v1 <verb>[ <base64 argument>...]
//! ```
//!
//! | verb     | arguments            | effect in the shell                                   |
//! |----------|----------------------|-------------------------------------------------------|
//! | `cd`     | destination          | change directory, bypassing the player-facing `cd`    |
//! | `export` | name, value          | set and export one variable (`^[A-Za-z_][A-Za-z0-9_]*$`) |
//! | `echo`   | message              | print one line                                        |
//! | `pwd`    | none                 | print the working directory                           |
//! | `exit`   | none                 | close the game shell                                  |
//!
//! Every argument is standard, padded base64 of the raw bytes, so spaces and shell
//! metacharacters never reach a shell parser. Lines end in LF. Unknown versions, verbs and
//! argument counts are ignored by the shell, and there is no way to run arbitrary commands.
//!
//! [`Directives`] is the typed emitter used by the game; [`encode`] is the pure encoder behind it.

use std::ffi::OsStr;
use std::fmt;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

/// Marker that starts every directive line. Anything else on stderr is plain output.
pub const PREFIX: &str = "__SHELLGAME_EXEC__";

/// Protocol version, written right after [`PREFIX`].
pub const VERSION: &str = "v1";

/// An environment variable name that the shell would refuse to export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidName(pub String);

impl fmt::Display for InvalidName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid environment variable name: {:?}", self.0)
    }
}

impl std::error::Error for InvalidName {}

/// Whether `name` matches `^[A-Za-z_][A-Za-z0-9_]*$`, the only names the shell accepts.
fn is_valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b'A'..=b'Z' | b'a'..=b'z' | b'_'))
        && bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Encodes one directive line, without the trailing newline.
///
/// Arguments are raw bytes (`&str`, `String`, `&[u8]`, ...) and are written as standard padded
/// base64. An empty argument becomes an empty word, so `encode("export", &["X", ""])` ends in a
/// space.
///
/// ```
/// use shellgame::shell::protocol::encode;
///
/// assert_eq!(encode("echo", &["hello"]), "__SHELLGAME_EXEC__v1 echo aGVsbG8=");
/// assert_eq!(encode::<&str>("exit", &[]), "__SHELLGAME_EXEC__v1 exit");
/// ```
pub fn encode<A: AsRef<[u8]>>(verb: &str, args: &[A]) -> String {
    let mut line = format!("{PREFIX}{VERSION} {verb}");
    for arg in args {
        line.push(' ');
        STANDARD.encode_string(arg.as_ref(), &mut line);
    }
    line
}

/// Writes `line` plus a newline in a single `write_all`, so concurrent writers to the same stderr
/// cannot split a directive, then flushes.
fn write_line<W: Write>(out: &mut W, mut line: String) -> io::Result<()> {
    line.push('\n');
    out.write_all(line.as_bytes())?;
    out.flush()
}

/// Emits directives for the wrapper shell, or does nothing when the binary runs outside of it.
///
/// Directives go to stderr and are flushed immediately, so they are in place before any prompt
/// that follows. Write errors are ignored: a closed stderr leaves the player's shell unchanged,
/// which is the only sensible outcome.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Directives {
    /// `true` when `SHELLGAME_WRAPPER` is set, i.e. a shell function will replay the output.
    pub enabled: bool,
}

impl Directives {
    /// Creates an emitter. Pass `false` outside the wrapper: nobody would replay the lines and
    /// they would be printed to the player as noise.
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Asks the shell to change its working directory to `path`.
    ///
    /// The shell moves with the plain `cd` implementation, never through the player-facing hook,
    /// so teleports are exempt from level restrictions.
    pub fn cd(&self, path: impl AsRef<Path>) {
        self.emit("cd", &[path.as_ref().as_os_str().as_bytes()]);
    }

    /// Asks the shell to set and export `name=value`. The value may be any string or path; its raw
    /// bytes are sent.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidName`] (and emits nothing) unless `name` matches
    /// `^[A-Za-z_][A-Za-z0-9_]*$`. The name is checked even when directives are disabled, so a
    /// mistake surfaces in tests that run outside the wrapper.
    pub fn export(&self, name: &str, value: impl AsRef<OsStr>) -> Result<(), InvalidName> {
        if !is_valid_name(name) {
            return Err(InvalidName(name.to_owned()));
        }
        self.emit("export", &[name.as_bytes(), value.as_ref().as_bytes()]);
        Ok(())
    }

    /// Asks the shell to print `message` followed by a newline.
    pub fn echo(&self, message: impl AsRef<str>) {
        self.emit("echo", &[message.as_ref().as_bytes()]);
    }

    /// Asks the shell to print its working directory.
    pub fn pwd(&self) {
        self.emit::<&[u8]>("pwd", &[]);
    }

    /// Asks the shell to close the game shell, for example after the game data was removed.
    pub fn exit(&self) {
        self.emit::<&[u8]>("exit", &[]);
    }

    fn emit<A: AsRef<[u8]>>(&self, verb: &str, args: &[A]) {
        // Ignored on purpose, see the type documentation.
        let _ = self.emit_to(&mut io::stderr().lock(), verb, args);
    }

    /// Writes one directive to `out`, or nothing when the emitter is disabled.
    fn emit_to<W: Write, A: AsRef<[u8]>>(
        &self,
        out: &mut W,
        verb: &str,
        args: &[A],
    ) -> io::Result<()> {
        if self.enabled {
            write_line(out, encode(verb, args))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(word: &str) -> Vec<u8> {
        STANDARD.decode(word).expect("valid standard base64")
    }

    #[test]
    fn encodes_marker_version_verb_and_base64_arguments() {
        assert_eq!(
            encode("cd", &["/tmp/shellgame"]),
            "__SHELLGAME_EXEC__v1 cd L3RtcC9zaGVsbGdhbWU="
        );
        assert_eq!(
            encode("export", &["SHELLGAME_LEVEL", "1.4"]),
            "__SHELLGAME_EXEC__v1 export U0hFTExHQU1FX0xFVkVM MS40"
        );
    }

    #[test]
    fn zero_argument_verbs_have_no_trailing_space() {
        assert_eq!(encode::<&str>("pwd", &[]), "__SHELLGAME_EXEC__v1 pwd");
        assert_eq!(encode::<&str>("exit", &[]), "__SHELLGAME_EXEC__v1 exit");
    }

    #[test]
    fn padding_is_standard_and_arguments_round_trip() {
        // One, two and three bytes: two, one and no padding characters.
        for (input, encoded) in [("a", "YQ=="), ("ab", "YWI="), ("abc", "YWJj")] {
            let line = encode("echo", &[input]);
            assert!(line.ends_with(encoded), "{line}");
            assert_eq!(decode(encoded), input.as_bytes());
        }
    }

    #[test]
    fn metacharacters_survive_as_data() {
        let nasty = "dir with spaces; $(touch PWNED) `id` 'q' \"dq\" \\ * ? \n tab\t";
        let line = encode("cd", &[nasty]);
        let words: Vec<&str> = line.split(' ').collect();

        assert_eq!(words.len(), 3, "spaces inside arguments must not add words");
        assert!(!line.contains('\n'), "a directive is exactly one line");
        assert_eq!(decode(words[2]), nasty.as_bytes());
    }

    #[test]
    fn empty_argument_is_an_empty_word() {
        assert_eq!(
            encode("export", &["EMPTY", ""]),
            "__SHELLGAME_EXEC__v1 export RU1QVFk= "
        );
    }

    #[test]
    fn non_utf8_bytes_are_encoded_losslessly() {
        let line = encode("cd", &[&b"/tmp/\xff\xfe"[..]]);
        let word = line.rsplit(' ').next().unwrap();

        assert_eq!(decode(word), b"/tmp/\xff\xfe");
    }

    #[test]
    fn variable_names_follow_the_shell_pattern() {
        for valid in ["A", "_", "_x1", "SHELLGAME_LEVEL", "a_B_9"] {
            assert!(is_valid_name(valid), "{valid}");
        }
        for invalid in [
            "", "1A", "BAD-NAME", "A B", "A=B", "A\n", "a.b", "é", "$X", "A;B",
        ] {
            assert!(!is_valid_name(invalid), "{invalid:?}");
        }
    }

    #[test]
    fn export_rejects_invalid_names_even_when_disabled() {
        for enabled in [true, false] {
            let error = Directives::new(enabled)
                .export("BAD-NAME", "value")
                .unwrap_err();

            assert_eq!(error, InvalidName("BAD-NAME".into()));
            assert!(error.to_string().contains("BAD-NAME"));
        }
    }

    #[test]
    fn enabled_directives_write_one_flushed_line_and_disabled_ones_nothing() {
        let mut out = Vec::new();
        Directives::new(true)
            .emit_to(&mut out, "echo", &["hi"])
            .unwrap();
        assert_eq!(out, b"__SHELLGAME_EXEC__v1 echo aGk=\n");

        let mut out = Vec::new();
        Directives::new(false)
            .emit_to(&mut out, "echo", &["hi"])
            .unwrap();
        assert!(out.is_empty());

        assert!(
            !Directives::default().enabled,
            "outside the wrapper by default"
        );
    }
}
