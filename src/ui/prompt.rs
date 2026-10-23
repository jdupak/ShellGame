//! Prompts. They go to stdout: the shell function holds stderr back until the command ends.

use std::io::{self, BufRead, IsTerminal, Read};

use rustix::termios::{self, LocalModes, OptionalActions, SpecialCodeIndex, Termios};

use super::{DIM, Term, span, write_now};

/// Waits for Enter. The terminal is put in non-canonical mode with `ISIG` off, so Ctrl-C is an
/// ordinary key: the process is not killed mid-prompt, the command ends normally and the shell
/// function still replays the directives emitted before. False when the player pressed Ctrl-C or
/// Ctrl-D (or the input ended); true on Enter, and without asking when stdin or stdout is no
/// terminal, so a pipe is never swallowed by a prompt nobody can see.
pub fn pause(term: &Term) -> bool {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return true;
    }
    let prompt = term.render(&[span(" Stiskněte Enter pro pokračování...", DIM)]);
    write_now(io::stdout(), prompt.as_bytes());
    // A terminal that refuses the mode change is no terminal for prompts.
    let Ok(_raw) = RawMode::enable() else {
        return true;
    };
    let enter = read_until_enter(&mut RawStdin);
    // Echo is off, so the cursor did not move to the next line by itself.
    write_now(io::stdout(), b"\n");
    enter
}

/// `<question> [y/N]: `, one line from stdin even when it is a pipe. Only yes (`y`, `yes`, `a`,
/// `ano`, any case) is yes: empty input, end of input and anything else decline.
pub fn confirm(question: &str) -> bool {
    write_now(io::stdout(), format!("{question} [y/N]: ").as_bytes());
    let mut answer = String::new();
    let read = io::stdin().lock().read_line(&mut answer);
    if !io::stdin().is_terminal() {
        write_now(io::stdout(), b"\n");
    }
    read.is_ok_and(|bytes| bytes > 0)
        && matches!(answer.trim().to_lowercase().as_str(), "y" | "yes" | "a" | "ano")
}

fn read_until_enter(input: &mut impl Read) -> bool {
    let mut key = [0_u8; 1];
    loop {
        match input.read(&mut key) {
            Ok(1) => match key[0] {
                b'\n' | b'\r' => return true,
                0x03 | 0x04 => return false,
                _ => {}
            },
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            _ => return false,
        }
    }
}

/// Unbuffered reads from stdin: `io::Stdin` would swallow keys typed after Enter, which are the shell's.
struct RawStdin;

impl Read for RawStdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        rustix::io::read(rustix::stdio::stdin(), buf).map_err(io::Error::from)
    }
}

struct RawMode {
    saved: Termios,
}

impl RawMode {
    fn enable() -> rustix::io::Result<RawMode> {
        let stdin = rustix::stdio::stdin();
        let saved = termios::tcgetattr(stdin)?;
        let mut raw = saved.clone();
        raw.local_modes
            .remove(LocalModes::ICANON | LocalModes::ECHO | LocalModes::ISIG | LocalModes::IEXTEN);
        raw.special_codes[SpecialCodeIndex::VMIN] = 1;
        raw.special_codes[SpecialCodeIndex::VTIME] = 0;
        termios::tcsetattr(stdin, OptionalActions::Drain, &raw)?;
        Ok(RawMode { saved })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = termios::tcsetattr(rustix::stdio::stdin(), OptionalActions::Now, &self.saved);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_enter_continues_and_ctrl_c_or_the_end_of_input_stop() {
        let enter = |keys: &[u8]| read_until_enter(&mut &*keys);
        assert!(enter(b"\n") && enter(b"\r") && enter(b"abc \x1b[A\x7f\n"));
        assert!(!enter(b"\x03") && !enter(b"ab\x03\n") && !enter(b"\x04") && !enter(b"abc"));
        let mut keys: &[u8] = b"\nrest";
        assert!(
            read_until_enter(&mut keys) && keys == b"rest",
            "what follows Enter is the shell's"
        );
    }
}
