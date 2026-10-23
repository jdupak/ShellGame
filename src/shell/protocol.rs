//! The directives the binary writes for the `shellgame` shell function (wire format: [`super`]).

use std::ffi::OsStr;
use std::io::{self, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

/// One directive line without its newline: `__SHELLGAME_EXEC__v1 <verb> <base64 argument>...`.
///
/// ```
/// use shellgame::shell::protocol::encode;
///
/// assert_eq!(encode("export", &["NAME", "value"]), "__SHELLGAME_EXEC__v1 export TkFNRQ== dmFsdWU=");
/// assert_eq!(encode::<&str>("exit", &[]), "__SHELLGAME_EXEC__v1 exit");
/// ```
pub fn encode<A: AsRef<[u8]>>(verb: &str, args: &[A]) -> String {
    let mut line = format!("__SHELLGAME_EXEC__v1 {verb}");
    for arg in args {
        line.push(' ');
        STANDARD.encode_string(arg, &mut line);
    }
    line
}

/// Writes directives to stderr, or nothing outside the game shell, where nobody would replay them.
#[derive(Debug, Clone, Copy, Default)]
pub struct Directives {
    enabled: bool,
}

impl Directives {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Change directory like a teleport: the shell's own `cd`, never the hook.
    pub fn cd(&self, path: impl AsRef<Path>) {
        self.emit("cd", &[path.as_ref().as_os_str().as_bytes()]);
    }

    /// Set and export `name=value`; the scripts drop a name that is not `[A-Za-z_][A-Za-z0-9_]*`.
    pub fn export(&self, name: &str, value: impl AsRef<OsStr>) {
        self.emit("export", &[name.as_bytes(), value.as_ref().as_bytes()]);
    }

    pub fn exit(&self) {
        self.emit::<&str>("exit", &[]);
    }

    /// One `write`, so concurrent writers cannot split a line. Errors are ignored: a closed stderr
    /// leaves the shell unchanged, which is the only sensible outcome.
    fn emit<A: AsRef<[u8]>>(&self, verb: &str, args: &[A]) {
        if self.enabled {
            let _ = io::stderr().write_all((encode(verb, args) + "\n").as_bytes());
        }
    }
}
