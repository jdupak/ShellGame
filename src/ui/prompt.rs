//! Interactive prompts: "press Enter to continue" and yes/no confirmation.
//!
//! Prompts go to **stdout**, never stderr: the shell wrapper buffers stderr until the command
//! exits, so a prompt there would not be visible while the program waits for the answer.
//!
//! # The Enter prompt
//!
//! [`wait_for_enter`] reads a key with the terminal in non-canonical mode and with `ISIG` off.
//! Ctrl-C is therefore an ordinary key (byte `0x03`) instead of a `SIGINT`: the process is never
//! killed in the middle of the prompt, [`wait_for_enter`] returns [`PromptOutcome::Interrupted`]
//! and the caller carries on and exits normally. That is what lets the shell wrapper still apply
//! the directives (`cd`, `export`) the process emitted before it asked for Enter. The terminal
//! settings are restored by a drop guard, also when the read fails.
//!
//! # Confirmation
//!
//! [`confirm`] is the replacement of `click.confirmation_option` for `shellgame remove`.
//! Click prints `<prompt> [y/N]: ` and accepts `y`/`yes` (any case) as yes, `n`/`no` or an empty
//! line as no, and for anything else prints "Error: invalid input" and asks again; end of input
//! aborts. This version accepts `y`, `yes` and, since the game is Czech, `a` and `ano` as yes.
//! Everything else, including empty input, end of input and unrecognised words, means no: it never
//! loops, so it cannot hang on a pipe, and declining is the safe answer for a destructive
//! action. Input is read as a line even when stdin is not a terminal, so `yes | shellgame remove`
//! works.

use std::io::{self, BufRead, IsTerminal, Read, Write};

use rustix::termios::{self, LocalModes, OptionalActions, SpecialCodeIndex, Termios};

use super::Term;
use super::style;

/// What the Enter prompt of a page is called in Czech: shown dimmed after a space.
pub const ENTER_PROMPT_TEXT: &str = "Stiskněte Enter pro pokračování...";

/// Control-C as read in non-canonical mode.
const CTRL_C: u8 = 0x03;
/// Control-D, end of transmission.
const CTRL_D: u8 = 0x04;

/// How [`wait_for_enter`] ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PromptOutcome {
    /// Enter (or Return) was pressed.
    Continued,
    /// Ctrl-C or Ctrl-D was pressed, or the input ended. The caller should carry on and exit
    /// normally, as if the remaining pages had been skipped.
    Interrupted,
    /// Nothing was asked because stdin or stdout is not a terminal.
    NotATty,
}

/// The standard Enter prompt: a space and [`ENTER_PROMPT_TEXT`], dimmed when colour is on.
///
/// ```
/// use shellgame::ui::{Term, enter_prompt};
///
/// assert_eq!(enter_prompt(&Term::plain(80)), " Stiskněte Enter pro pokračování...");
/// ```
#[must_use]
pub fn enter_prompt(term: &Term) -> String {
    format!(" {}", term.paint(style::DIM, ENTER_PROMPT_TEXT))
}

/// Prints `prompt` to stdout and waits for Enter.
///
/// Does nothing and returns [`PromptOutcome::NotATty`] unless both stdin and stdout are
/// terminals, so piped input is never swallowed by a prompt and nothing waits on a prompt that
/// nobody can see. The prompt may contain ANSI styling; it is stripped when stdout does not want
/// colour. See the [module documentation](self) for how Ctrl-C is handled.
pub fn wait_for_enter(prompt: &str) -> PromptOutcome {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return PromptOutcome::NotATty;
    }

    let mut out = anstream::stdout();
    // A prompt that cannot be written is not worth failing the command for.
    let _ = write!(out, "{prompt}");
    let _ = out.flush();

    let outcome = match RawMode::enable() {
        Ok(_guard) => read_until_enter(&mut RawStdin),
        // The terminal refused the mode change: fall back to a line read in canonical mode.
        Err(_) => {
            let mut line = String::new();
            match io::stdin().lock().read_line(&mut line) {
                Ok(0) | Err(_) => PromptOutcome::Interrupted,
                Ok(_) => PromptOutcome::Continued,
            }
        }
    };

    // Echo is off, so the terminal did not move to the next line by itself.
    let _ = writeln!(out);
    let _ = out.flush();
    outcome
}

/// Asks a yes/no question on stdout and reads the answer from stdin.
///
/// Prints `"<prompt> [y/N]: "` (the default is no) and reads one line, also when stdin is not a
/// terminal. `y`, `yes`, `a` and `ano` in any case are yes; everything else is no. See the
/// [module documentation](self) for the comparison with click.
pub fn confirm(prompt: &str) -> bool {
    let stdin = io::stdin();
    let echoes = stdin.is_terminal();
    let mut out = anstream::stdout();
    confirm_with(&mut stdin.lock(), &mut out, prompt, echoes)
}

/// The logic of [`confirm`] over explicit streams. `echoes` says whether the user's Enter moved
/// the cursor to a new line; when it did not (piped input) a newline is printed.
fn confirm_with<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    prompt: &str,
    echoes: bool,
) -> bool {
    let _ = write!(out, "{prompt} [y/N]: ");
    let _ = out.flush();
    let mut answer = String::new();
    let read = input.read_line(&mut answer);
    if !echoes {
        let _ = writeln!(out);
        let _ = out.flush();
    }
    matches!(read, Ok(bytes) if bytes > 0) && is_yes(&answer)
}

/// Whether an answer to a `[y/N]` question means yes.
fn is_yes(answer: &str) -> bool {
    matches!(
        answer.trim().to_lowercase().as_str(),
        "y" | "yes" | "a" | "ano"
    )
}

/// Reads bytes until Enter, Ctrl-C, Ctrl-D or the end of the input; other keys are ignored.
fn read_until_enter(input: &mut impl Read) -> PromptOutcome {
    let mut byte = [0_u8; 1];
    loop {
        match input.read(&mut byte) {
            Ok(0) => return PromptOutcome::Interrupted,
            Ok(_) => match byte[0] {
                b'\n' | b'\r' => return PromptOutcome::Continued,
                CTRL_C | CTRL_D => return PromptOutcome::Interrupted,
                _ => {}
            },
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return PromptOutcome::Interrupted,
        }
    }
}

/// Unbuffered reads from file descriptor 0.
///
/// `std::io::Stdin` reads ahead into a buffer, which would swallow keys typed after Enter; they
/// belong to the shell.
struct RawStdin;

impl Read for RawStdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        rustix::io::read(rustix::stdio::stdin(), buf).map_err(io::Error::from)
    }
}

/// Puts stdin into non-canonical mode without echo and without signal characters, and restores
/// the previous settings when dropped.
struct RawMode {
    saved: Termios,
}

impl RawMode {
    fn enable() -> rustix::io::Result<Self> {
        let stdin = rustix::stdio::stdin();
        let saved = termios::tcgetattr(stdin)?;
        let mut raw = saved.clone();
        raw.local_modes
            .remove(LocalModes::ICANON | LocalModes::ECHO | LocalModes::ISIG | LocalModes::IEXTEN);
        raw.special_codes[SpecialCodeIndex::VMIN] = 1;
        raw.special_codes[SpecialCodeIndex::VTIME] = 0;
        termios::tcsetattr(stdin, OptionalActions::Drain, &raw)?;
        Ok(Self { saved })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // Nothing sensible can be done if the terminal refuses the old settings.
        let _ = termios::tcsetattr(rustix::stdio::stdin(), OptionalActions::Now, &self.saved);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(bytes: &[u8]) -> PromptOutcome {
        read_until_enter(&mut &*bytes)
    }

    #[test]
    fn enter_continues() {
        assert_eq!(keys(b"\n"), PromptOutcome::Continued);
        assert_eq!(keys(b"\r"), PromptOutcome::Continued);
        assert_eq!(keys(b"\r\n"), PromptOutcome::Continued);
    }

    #[test]
    fn other_keys_are_ignored_until_enter() {
        assert_eq!(keys(b"abc \x1b[A\x1b[B\x7f\n"), PromptOutcome::Continued);
    }

    #[test]
    fn ctrl_c_and_ctrl_d_interrupt() {
        assert_eq!(keys(b"\x03"), PromptOutcome::Interrupted);
        assert_eq!(keys(b"\x04"), PromptOutcome::Interrupted);
        assert_eq!(keys(b"ab\x03\n"), PromptOutcome::Interrupted);
    }

    #[test]
    fn end_of_input_interrupts() {
        assert_eq!(keys(b""), PromptOutcome::Interrupted);
        assert_eq!(keys(b"abc"), PromptOutcome::Interrupted);
    }

    #[test]
    fn only_the_bytes_up_to_enter_are_consumed() {
        let mut input: &[u8] = b"\nrest";
        assert_eq!(read_until_enter(&mut input), PromptOutcome::Continued);
        assert_eq!(input, b"rest");
    }

    #[test]
    fn an_interrupted_read_is_retried() {
        struct Flaky(u8);
        impl Read for Flaky {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                self.0 += 1;
                if self.0 == 1 {
                    Err(io::ErrorKind::Interrupted.into())
                } else {
                    buf[0] = b'\n';
                    Ok(1)
                }
            }
        }
        assert_eq!(read_until_enter(&mut Flaky(0)), PromptOutcome::Continued);
    }

    #[test]
    fn a_failing_read_interrupts() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
        }
        assert_eq!(read_until_enter(&mut Broken), PromptOutcome::Interrupted);
    }

    #[test]
    fn enter_prompt_is_a_dim_czech_sentence() {
        assert_eq!(
            enter_prompt(&Term::plain(80)),
            " Stiskněte Enter pro pokračování..."
        );
        let dim = enter_prompt(&Term::ansi(80));
        assert!(dim.starts_with(' '));
        assert!(dim.contains(&style::DIM.render().to_string()));
        assert!(dim.ends_with("\x1b[0m"));
    }

    #[test]
    fn yes_answers() {
        for answer in [
            "y", "Y", "yes", "YES", "Yes\n", " y \r\n", "a", "A", "ano", "Ano", "ANO",
        ] {
            assert!(is_yes(answer), "{answer:?}");
        }
    }

    #[test]
    fn everything_else_is_no() {
        for answer in [
            "",
            "\n",
            "n",
            "no",
            "N",
            "ne",
            "nope",
            "yy",
            "yes please",
            "1",
            "true",
            "y n",
        ] {
            assert!(!is_yes(answer), "{answer:?}");
        }
    }

    fn ask(input: &str, echoes: bool) -> (bool, String) {
        let mut reader = input.as_bytes();
        let mut out = Vec::new();
        let answer = confirm_with(&mut reader, &mut out, "Opravdu?", echoes);
        (answer, String::from_utf8(out).expect("utf-8 prompt"))
    }

    #[test]
    fn confirm_prints_the_prompt_with_the_default_marker() {
        let (_, shown) = ask("y\n", true);
        assert_eq!(shown, "Opravdu? [y/N]: ");
    }

    #[test]
    fn confirm_reads_a_line_from_a_pipe_like_yes_does() {
        let (answer, shown) = ask("y\ny\ny\n", false);
        assert!(answer);
        // The cursor did not move on its own: a newline closes the prompt line.
        assert_eq!(shown, "Opravdu? [y/N]: \n");
    }

    #[test]
    fn confirm_declines_on_no_empty_and_eof() {
        assert!(!ask("n\n", true).0);
        assert!(!ask("\n", true).0);
        assert!(!ask("", true).0);
        assert!(!ask("maybe\n", true).0);
    }

    #[test]
    fn confirm_accepts_the_last_line_without_a_newline() {
        assert!(ask("yes", false).0);
    }
}
