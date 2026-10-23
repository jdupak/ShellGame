//! The command line: clap definitions with Czech help, the hooks of the game shell, booting the
//! game shell and dispatch.
//!
//! `cd-hook` and `guard` come from the game shell's functions and carry arbitrary text (a
//! `cd` target can be `-`, `--` or a directory named `pre`), so they are parsed by hand before clap.
//! Hooks fail open: anything malformed exits 0 and allows. Exit status: 0 for normal play (a wrong
//! answer included), 1 for errors and a declined `remove`, 2 for usage errors, 3 when a hook refuses.

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;

use clap::{Arg, ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};

use crate::env::Env;
use crate::game::Pos;
use crate::sandbox;
use crate::session::{BLOCKED, FAILURE, Flow, Halt, OK, Session};
use crate::shell::{self, Directives, ENV_PENDING, LaunchOptions, ShellKind};
use crate::ui::{BOLD_RED, RED, Term, screens};

const BUILD: &str = match option_env!("SHELLGAME_BUILD") {
    Some(build) => build,
    None => "dev",
};

/// `shellgame 0.2.0 (build-123)`
pub fn version_line() -> String {
    format!("shellgame {} ({BUILD})", env!("CARGO_PKG_VERSION"))
}

const ROOT_TEMPLATE: &str = r#"{about}

Použití: {usage}

Možnosti:
{options}

Příkazy:
{subcommands}
"#;
const COMMAND_TEMPLATE: &str = r#"{about}

Použití: {usage}

{all-args}
"#;

#[derive(Debug, Parser)]
#[command(
    name = "shellgame",
    about = "ShellGame - Interaktivní výuka navigace v terminálu.",
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = ROOT_TEMPLATE,
    override_usage = "shellgame [MOŽNOSTI] [PŘÍKAZ]",
    subcommand_help_heading = "Příkazy",
    next_help_heading = "Možnosti"
)]
struct Cli {
    #[arg(long, value_name = "bash|fish", value_parser = ["bash", "fish"], ignore_case = true, hide_possible_values = true, help = "Vynutit typ subshellu (bash/fish).")]
    shell: Option<String>,

    #[arg(
        long = "unsafe",
        help = "Spustit hru bez izolace, na vlastní riziko (když systém nemá Landlock)."
    )]
    unsafe_mode: bool,

    #[arg(short = 'V', long, help = "Zobrazit verzi a ukončit.")]
    version: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(
        about = "Zobrazit dosud odemčené nápovědy aktuálního levelu a odemknout další.",
        override_usage = "shellgame hint [MOŽNOSTI]"
    )]
    Hint {
        #[arg(short, long, help = "Jen zopakovat odemčené nápovědy, další neodemykat.")]
        repeat: bool,
    },
    #[command(
        about = "Odeslat odpověď pro aktuální level.",
        override_usage = "shellgame submit [ODPOVĚĎ]..."
    )]
    Submit {
        #[arg(allow_hyphen_values = true, num_args = 0.., value_name = "ODPOVĚĎ", help_heading = "Argumenty", help = "Odpověď (více slov se spojí mezerou).")]
        answer: Vec<String>,
    },
    #[command(about = "Zobrazit postup a statistiky.", override_usage = "shellgame status")]
    Status,
    #[command(about = "Obnovit strukturu aktuálního levelu.", override_usage = "shellgame reset")]
    Reset,
    #[command(
        about = "Znovu zobrazit zadání levelu nebo úvod sekce.",
        override_usage = "shellgame show [S.L | S]",
        alias = "repeat"
    )]
    Show {
        #[arg(
            value_name = "S.L | S",
            help_heading = "Argumenty",
            help = "Pozice levelu (např. 3.2), nebo číslo sekce (např. 3) pro její úvod. Bez argumentu se zobrazí aktuální level."
        )]
        target: Option<String>,
    },
    #[command(about = "Vypsat všechny levely.", override_usage = "shellgame levels")]
    Levels,
    #[command(
        about = "Pokračovat od zvoleného levelu, např. na jiném počítači.",
        override_usage = "shellgame resume <S.L | S>"
    )]
    Resume {
        #[arg(
            value_name = "S.L | S",
            help_heading = "Argumenty",
            help = "Pozice levelu (např. 3.2), nebo číslo sekce (např. 3) pro její úvod a první level."
        )]
        target: Option<String>,
    },
    #[command(
        about = "Smazat stav a pracovní prostor.",
        override_usage = "shellgame remove [MOŽNOSTI]"
    )]
    Remove {
        #[arg(long, help = "Potvrdit bez dotazu.")]
        yes: bool,
    },
    #[command(about = "Ukončit ShellGame.", override_usage = "shellgame exit")]
    Exit,
    #[command(
        about = "Aktualizovat ShellGame na nejnovější vydání.",
        override_usage = "shellgame update"
    )]
    Update,
}

/// `--help` in Czech, for every command.
fn help_arg() -> Arg {
    Arg::new("help")
        .short('h')
        .long("help")
        .action(ArgAction::Help)
        .help("Zobrazit tuto nápovědu a ukončit.")
        .help_heading("Možnosti")
}

/// Czech headings and help flag for a subcommand.
fn localize(command: clap::Command) -> clap::Command {
    command
        .disable_help_flag(true)
        .help_template(COMMAND_TEMPLATE)
        .mut_args(|arg| match arg.get_help_heading() {
            Some(_) => arg,
            None => arg.help_heading("Možnosti"),
        })
        .arg(help_arg())
}

fn parse(args: &[OsString]) -> Result<Cli, clap::Error> {
    let command = Cli::command().arg(help_arg()).mut_subcommands(localize);
    let matches = command.try_get_matches_from(
        std::iter::once(OsString::from("shellgame")).chain(args.iter().cloned()),
    )?;
    Cli::from_arg_matches(&matches)
}

/// Prints a command-line error and returns its status. Help and the version go out as clap writes
/// them; a mistyped command, argument or value is told in Czech (clap's words are English).
fn usage(error: clap::Error) -> i32 {
    use clap::error::{ContextKind as K, ContextValue, ErrorKind};
    let value = |kind| match error.get(kind)? {
        ContextValue::String(text) => Some(text.clone()),
        ContextValue::Strings(texts) => Some(texts.join(", ")),
        _ => None,
    };
    let similar = |kind| value(kind).map(|name| format!(" Měli jste na mysli '{name}'?"));
    let what = match error.kind() {
        ErrorKind::InvalidSubcommand => value(K::InvalidSubcommand).map(|name| {
            format!(
                "Neznámý příkaz '{name}'.{}",
                similar(K::SuggestedSubcommand).unwrap_or_default()
            )
        }),
        ErrorKind::UnknownArgument => value(K::InvalidArg).map(|arg| {
            format!("Nečekaný argument '{arg}'.{}", similar(K::SuggestedArg).unwrap_or_default())
        }),
        ErrorKind::InvalidValue => value(K::InvalidValue)
            .zip(value(K::ValidValue))
            .map(|(given, valid)| format!("Neplatná hodnota '{given}'. Možnosti: {valid}.")),
        _ => None,
    };
    let Some(what) = what else {
        let _ = error.print();
        return error.exit_code();
    };
    let term = Term::stderr();
    term.print(&screens::say(&term, RED, &format!("Chyba: {what}")));
    term.print(&screens::cmd_note(&term, "Přehled příkazů: ", "shellgame --help"));
    error.exit_code()
}

/// The hook commands of the game shell; `None` for everything else.
fn hook(args: &[OsString], env: &Env) -> Option<i32> {
    let (name, rest) = args.split_first()?;
    Some(match (name.to_str()?, rest) {
        ("guard", [separator, chmod_args @ ..]) if separator == "--" => {
            match sandbox::guard(env.cwd.as_deref().unwrap_or(Path::new(".")), chmod_args) {
                Ok(()) => OK,
                Err(refusal) => {
                    let _ = writeln!(io::stderr(), "{refusal}");
                    BLOCKED
                }
            }
        }
        ("cd-hook", [phase, first, second, separator, cd_args @ ..])
            if separator == "--" && (phase == "pre" || phase == "post") =>
        {
            Session::new(env).cd_hook(phase == "post", Path::new(first), Path::new(second), cd_args)
        }
        ("guard" | "cd-hook", _) => OK,
        _ => return None,
    })
}

/// Typed outside the game shell, these start it and run inside it: the bare command, and the ones
/// that need a game (`resume` only with a level to go to; without one it just lists the levels).
fn starts_game_shell(command: &Option<Command>) -> bool {
    match command {
        None | Some(Command::Reset | Command::Submit { .. }) => true,
        Some(Command::Resume { target }) => {
            target.as_deref().is_some_and(|t| t.parse::<Pos>().is_ok())
        }
        _ => false,
    }
}

/// What the new game shell runs once it is up: the subcommand and its arguments, joined by U+001F.
fn pending(command: &Option<Command>) -> Option<String> {
    let words = match command {
        Some(Command::Reset) => vec!["reset"],
        Some(Command::Submit { answer }) => {
            ["submit"].into_iter().chain(answer.iter().map(String::as_str)).collect()
        }
        Some(Command::Resume { target: Some(target) }) => vec!["resume", target],
        _ => return None,
    };
    Some(words.join("\u{1f}"))
}

/// `shellgame` outside the game shell starts it, with the command that was typed (if any) left for
/// the new shell in `SHELLGAME_PENDING`. Its own exit status is not a failure. The game
/// shell is confined where Landlock works, unconfined only when the player asked for it with
/// `--unsafe` (then the probe is not even run), otherwise it does not start at all.
fn boot(cli: &Cli, env: &Env) -> i32 {
    let term = Term::stdout();
    let Some(binary) = env.exe.clone() else {
        term.print(&screens::say(&term, BOLD_RED, "CHYBA: Nelze zjistit cestu k programu."));
        return FAILURE;
    };
    let confined = !cli.unsafe_mode;
    if confined && !sandbox::probe(&binary, &env.state_dir) {
        term.print(&screens::refusal(&term));
        return FAILURE;
    }
    let shell = shell::detect(cli.shell.as_deref(), env.shell.as_deref());
    // fish makes its own directories on its first start, which the sandbox forbids, and fish 3.3
    // then warns on every start: make them as fish would, before the shell is confined.
    if confined && shell == ShellKind::Fish {
        for home in [&env.config_home, &env.data_home] {
            let _ = std::fs::create_dir_all(home.join("fish"));
        }
    }
    let options =
        LaunchOptions { shell, binary, state_dir: env.state_dir.clone(), sandbox: confined };
    let pending = pending(&cli.command);
    let configure = |command: &mut std::process::Command| match pending {
        Some(pending) => drop(command.env(ENV_PENDING, pending)),
        None => drop(command.env_remove(ENV_PENDING)),
    };
    match shell::launch_with(&options, configure) {
        Ok(_) => OK,
        Err(error) => {
            term.print(&screens::say(
                &term,
                BOLD_RED,
                &format!("CHYBA: Nepodařilo se spustit herní shell ({error})"),
            ));
            FAILURE
        }
    }
}

fn dispatch(session: &mut Session, command: Option<Command>) -> Flow {
    match command {
        None => session.current(),
        Some(Command::Hint { repeat }) => session.hint(repeat),
        Some(Command::Submit { answer }) => {
            session.submit((!answer.is_empty()).then(|| answer.join(" ")))
        }
        Some(Command::Status) => session.status(),
        Some(Command::Reset) => session.reset(),
        Some(Command::Show { target }) => session.show(target.as_deref()),
        Some(Command::Levels) => session.levels(),
        Some(Command::Resume { target }) => session.resume(target.as_deref()),
        Some(Command::Remove { yes }) => session.remove(yes),
        Some(Command::Exit) => session.exit(),
        Some(Command::Update) => session.update(),
    }
}

/// Runs the command line of this process; returns the exit status.
pub fn main() -> i32 {
    let (args, env) = (std::env::args_os().skip(1).collect::<Vec<_>>(), Env::from_process());
    if let Some(status) = hook(&args, &env) {
        return status;
    }
    let cli = match parse(&args) {
        Ok(cli) => cli,
        Err(error) => return usage(error),
    };
    if cli.version {
        let _ = writeln!(io::stdout(), "{}", version_line());
        return OK;
    }
    let mut session = Session::new(&env);
    if let Err(why) = env.check() {
        return session.finish(Err(Halt::Fail(why)));
    }
    if !env.wrapped && starts_game_shell(&cli.command) {
        return boot(&cli, &env);
    }
    let mut command = cli.command;
    // The autostart of a game shell that was started for a command: run it once, as if typed here.
    if let (true, None, Some(pending)) = (env.wrapped, &command, &env.pending) {
        Directives::new(true).export(ENV_PENDING, "");
        let args: Vec<OsString> = pending.split('\u{1f}').map(OsString::from).collect();
        match parse(&args) {
            Ok(parsed) => command = parsed.command,
            Err(error) => return usage(error),
        }
    }
    let result = dispatch(&mut session, command);
    session.finish(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clap_definition_is_consistent() {
        Cli::command().debug_assert();
    }
}
