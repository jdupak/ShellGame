//! The command line: argument parsing, the shell hooks, booting the game shell and dispatch.
//!
//! The player's commands are described with `clap` ([`Cli`], [`Command`]), with Czech help. The
//! commands that the game shell's own functions send are not: `cd-hook` and `fd-hook` are parsed
//! by hand, *before* clap ([`parse_hook`]), because their arguments are arbitrary text. A `cd`
//! target can be `""`, `-`, `-P`, `--` or a directory literally named `pre`, and none of those may
//! be taken for an option or a subcommand.
//!
//! # Commands
//!
//! ```text
//! shellgame [--shell bash|fish]            show the current level (outside the game shell: start it)
//! shellgame hint [-r|--repeat]             the next hint, or all revealed hints again
//! shellgame submit [ANSWER...]             answer the level ("-5" is an answer, words are joined)
//! shellgame status | skip | reset | exit | update
//! shellgame repeat [--section N] [--level ID]
//! shellgame show (--section | --level)
//! shellgame remove [--yes]
//! shellgame --version                      `shellgame <version> (<build>)`
//! ```
//!
//! Hidden: `init` (legacy), `dev jump ID | next | prev | reload | start`, `--devmode`, and the
//! hook commands.
//!
//! # The hook commands
//!
//! ```text
//! cd-hook pre  <pwd> <prev> -- <cd args...>      may the player cd? exit 3 = no
//! cd-hook post <old pwd> <new pwd> -- <cd args...>
//! cd-hook [TARGET] [PWD]                         legacy: shells started by the Python version
//! cd-hook --post-move [PWD]                      legacy
//! fd-hook
//! ```
//!
//! A leading `--devmode` is accepted before any command, because the Python wrapper prefixed it.
//!
//! # Exit status
//!
//! `0` for everything that is normal gameplay (a wrong answer included), `1` for errors and a
//! declined `remove`, `2` for usage errors (also those of clap), `3` when a `cd` is blocked by
//! the new hook. The legacy `cd-hook` blocks with `1`, because the old wrapper treats any
//! non-zero status as "blocked".

use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;

use clap::{Arg, ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand};

use crate::engine::prelude::Catalog;
use crate::env::Env;
use crate::session::{FAILURE, OK, Session};
use crate::shell::{self, LaunchOptions};
use crate::ui::{Term, print_lines, screens};

/// The build that is reported by `--version`: `$SHELLGAME_BUILD` at compile time, or `dev`.
pub const BUILD: &str = match option_env!("SHELLGAME_BUILD") {
    Some(build) => build,
    None => "dev",
};

/// What `shellgame --version` prints: `shellgame 0.2.0 (build-123)`.
///
/// ```
/// let line = shellgame::cli::version_line();
/// assert!(line.starts_with("shellgame "));
/// assert!(line.ends_with(')'));
/// ```
pub fn version_line() -> String {
    let build = if BUILD.is_empty() { "dev" } else { BUILD };
    format!("shellgame {} ({build})", env!("CARGO_PKG_VERSION"))
}

const ABOUT: &str = "ShellGame - Interaktivní výuka navigace v terminálu.";

/// Help of the root command: Czech headings, options before commands (as click printed them).
const ROOT_TEMPLATE: &str = "\
{about}

Použití: {usage}

Možnosti:
{options}

Příkazy:
{subcommands}
";

/// Help of every subcommand.
const COMMAND_TEMPLATE: &str = "\
{about}

Použití: {usage}

{all-args}
";

/// The command line of the player.
#[derive(Debug, Parser)]
#[command(
    name = "shellgame",
    bin_name = "shellgame",
    about = ABOUT,
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    help_template = ROOT_TEMPLATE,
    override_usage = "shellgame [MOŽNOSTI] [PŘÍKAZ]",
    subcommand_help_heading = "Příkazy",
    next_help_heading = "Možnosti"
)]
pub struct Cli {
    /// Which shell to start when `shellgame` is run outside the game shell.
    #[arg(
        long,
        value_name = "bash|fish",
        value_parser = ["bash", "fish"],
        ignore_case = true,
        hide_possible_values = true,
        help = "Vynutit typ subshellu (bash/fish)."
    )]
    pub shell: Option<String>,

    /// Define the developer shortcuts (`r n p j s e g`) in the game shell.
    #[arg(long, hide = true)]
    pub devmode: bool,

    /// Print the version and exit.
    #[arg(short = 'V', long, help = "Zobrazit verzi a ukončit.")]
    pub version: bool,

    /// What to do; the current level is shown without a command.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// A command of the player.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Legacy explicit initialization.
    #[command(
        hide = true,
        about = "Inicializace relace ShellGame (již není potřeba, děje se automaticky).",
        override_usage = "shellgame init"
    )]
    Init,

    /// Reveal the next hint.
    #[command(
        about = "Zobrazit nápovědu pro aktuální level.",
        override_usage = "shellgame hint [MOŽNOSTI]"
    )]
    Hint {
        /// Show the hints revealed so far again.
        #[arg(
            short,
            long,
            help = "Zobrazit znovu všechny již zobrazené nápovědy pro aktuální level."
        )]
        repeat: bool,
    },

    /// Answer the current level.
    #[command(
        about = "Odeslat odpověď pro aktuální level.",
        override_usage = "shellgame submit [ODPOVĚĎ]..."
    )]
    Submit {
        /// The answer; several words are joined with a space, and a leading `-` is allowed.
        #[arg(
            allow_hyphen_values = true,
            num_args = 0..,
            value_name = "ODPOVĚĎ",
            help_heading = "Argumenty",
            help = "Odpověď (více slov se spojí mezerou)."
        )]
        answer: Vec<String>,
    },

    /// Show progress and statistics.
    #[command(
        about = "Zobrazit postup a statistiky.",
        override_usage = "shellgame status"
    )]
    Status,

    /// Skip a bonus level.
    #[command(
        about = "Přeskočit aktuální level (pouze nepovinné levely).",
        override_usage = "shellgame skip"
    )]
    Skip,

    /// Rebuild the current level.
    #[command(
        about = "Obnovit strukturu aktuálního levelu.",
        override_usage = "shellgame reset"
    )]
    Reset,

    /// Show an assignment again.
    #[command(
        about = "Znovu zobrazit zadání (aktuálního nebo zvoleného) levelu.",
        override_usage = "shellgame repeat [MOŽNOSTI]"
    )]
    Repeat {
        /// The section whose intro to show.
        #[arg(
            long,
            value_name = "ČÍSLO",
            help = "Číslo sekce, kterou chcete zopakovat (např. 1)."
        )]
        section: Option<u32>,
        /// The level to show.
        #[arg(
            long,
            value_name = "ID",
            help = "ID levelu ke zopakování (např. 1.7). Pokud neuvedete, zopakuje se aktuální level."
        )]
        level: Option<String>,
    },

    /// Show the current level or the current section intro again.
    #[command(
        about = "Zobrazit znovu zadání aktuálního levelu nebo úvod aktuální sekce.",
        override_usage = "shellgame show (--section | --level)"
    )]
    Show {
        /// Show the intro of the current section.
        #[arg(long, help = "Zobrazit znovu úvod aktuální sekce (X.0).")]
        section: bool,
        /// Show the current level.
        #[arg(long, help = "Zobrazit znovu zadání aktuálního levelu.")]
        level: bool,
    },

    /// Delete the state and the workspace.
    #[command(
        about = "Smazat stav a pracovní prostor.",
        override_usage = "shellgame remove [MOŽNOSTI]"
    )]
    Remove {
        /// Do not ask for confirmation.
        #[arg(long, help = "Potvrdit bez dotazu.")]
        yes: bool,
    },

    /// Close the game shell.
    #[command(about = "Ukončit ShellGame.", override_usage = "shellgame exit")]
    Exit,

    /// Replace this binary with the latest release.
    #[command(
        about = "Aktualizovat ShellGame na nejnovější vydání.",
        override_usage = "shellgame update"
    )]
    Update,

    /// Developer commands.
    #[command(
        hide = true,
        subcommand,
        about = "Vývojářské příkazy.",
        override_usage = "shellgame dev <PŘÍKAZ>"
    )]
    Dev(DevCommand),
}

/// A developer command (`shellgame dev ...`).
#[derive(Debug, Subcommand)]
pub enum DevCommand {
    /// Prepare and show any level; its progress is cleared.
    #[command(
        about = "Připravit a zobrazit libovolný level (jeho postup se smaže).",
        override_usage = "shellgame dev jump <ID>"
    )]
    Jump {
        /// The level id, such as `1.4`.
        #[arg(value_name = "ID", help = "ID levelu (např. 1.4).")]
        id: String,
    },
    /// Jump to the next level.
    #[command(
        about = "Přejít na další level.",
        override_usage = "shellgame dev next"
    )]
    Next,
    /// Jump to the previous level.
    #[command(
        about = "Přejít na předchozí level.",
        override_usage = "shellgame dev prev"
    )]
    Prev,
    /// Rebuild and show the current level.
    #[command(
        about = "Obnovit a znovu zobrazit aktuální level.",
        override_usage = "shellgame dev reload"
    )]
    Reload,
    /// Jump to the first level.
    #[command(
        about = "Přejít na první level.",
        override_usage = "shellgame dev start"
    )]
    Start,
}

/// The `--help` flag in Czech, shared by every command.
fn help_arg() -> Arg {
    Arg::new("help")
        .short('h')
        .long("help")
        .action(ArgAction::Help)
        .help("Zobrazit tuto nápovědu a ukončit.")
        .help_heading("Možnosti")
}

/// Gives a subcommand (and, recursively, its subcommands) Czech help: headings for its arguments
/// and options, and a Czech `--help` flag.
fn localize(command: clap::Command) -> clap::Command {
    command
        .disable_help_flag(true)
        .help_template(COMMAND_TEMPLATE)
        .subcommand_help_heading("Příkazy")
        .mut_args(|arg| match arg.get_help_heading() {
            Some(_) => arg,
            None if arg.is_positional() => arg.help_heading("Argumenty"),
            None => arg.help_heading("Možnosti"),
        })
        .arg(help_arg())
        .mut_subcommands(localize)
}

/// The `clap` command of the player, with Czech help everywhere.
pub fn command() -> clap::Command {
    Cli::command().arg(help_arg()).mut_subcommands(localize)
}

/// Parses the arguments (without the program name) of the player's command line.
///
/// # Errors
///
/// The `clap` error for a usage error, `--help` or an unknown command; [`clap::Error::print`]
/// shows it and [`clap::Error::exit_code`] says how to exit.
pub fn parse(args: &[OsString]) -> Result<Cli, clap::Error> {
    let argv = std::iter::once(OsString::from("shellgame")).chain(args.iter().cloned());
    let matches = command().try_get_matches_from(argv)?;
    Cli::from_arg_matches(&matches)
}

// ---------------------------------------------------------------------------------------------
// Hooks
// ---------------------------------------------------------------------------------------------

/// A command that the game shell's functions send, parsed by hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hook {
    /// `cd-hook pre <pwd> <prev> -- <cd args...>`: may the player `cd`?
    CdPre {
        /// The shell's directory before the move.
        pwd: OsString,
        /// The directory `cd -` would go to (empty when unknown).
        prev: OsString,
        /// The arguments of the `cd` call.
        args: Vec<OsString>,
    },
    /// `cd-hook post <old pwd> <new pwd> -- <cd args...>`: the move happened.
    CdPost {
        /// The directory before the move.
        old: OsString,
        /// The directory after it.
        new: OsString,
        /// The arguments of the `cd` call.
        args: Vec<OsString>,
    },
    /// Legacy `cd-hook [TARGET] [PWD]` of a shell started by the Python version.
    LegacyCdPre {
        /// The first argument of `cd`.
        target: Option<OsString>,
        /// The shell's directory.
        pwd: Option<OsString>,
    },
    /// Legacy `cd-hook --post-move [PWD]`.
    LegacyCdPost {
        /// The shell's directory after the move.
        pwd: Option<OsString>,
    },
    /// `fd-hook`: where do this process's stdout and stderr point?
    Fd,
}

/// Recognizes the hook commands. Returns `None` for everything else, which then goes to clap.
///
/// The new `cd-hook` forms are `pre` and `post` with exactly the two directories and a `--` before
/// the `cd` arguments (`cd-hook pre <pwd> <prev> -- <cd args...>`). Anything else after `cd-hook`
/// is the legacy form: at most `[TARGET] [PWD]`, or `--post-move [PWD]`. So a legacy hook for
/// `cd pre` (`cd-hook pre /some/dir`) is not mistaken for the new `pre` form.
///
/// ```
/// use std::ffi::OsString;
/// use shellgame::cli::{Hook, parse_hook};
///
/// let args: Vec<OsString> = ["cd-hook", "pre", "/a", "", "--", "-P", "--", "x"]
///     .iter().map(OsString::from).collect();
/// let Some(Hook::CdPre { pwd, prev, args }) = parse_hook(&args) else { unreachable!() };
/// assert_eq!(pwd, "/a");
/// assert_eq!(prev, "");
/// assert_eq!(args, ["-P", "--", "x"]);
/// ```
pub fn parse_hook(args: &[OsString]) -> Option<Hook> {
    // The Python wrapper prefixed `--devmode` to every command.
    let args = match args.first() {
        Some(first) if first == "--devmode" => &args[1..],
        _ => args,
    };
    let (command, rest) = args.split_first()?;
    if command == "fd-hook" {
        return Some(Hook::Fd);
    }
    if command != "cd-hook" {
        return None;
    }

    if let [phase, first, second, separator, cd_args @ ..] = rest
        && separator == "--"
    {
        if phase == "pre" {
            return Some(Hook::CdPre {
                pwd: first.clone(),
                prev: second.clone(),
                args: cd_args.to_vec(),
            });
        }
        if phase == "post" {
            return Some(Hook::CdPost {
                old: first.clone(),
                new: second.clone(),
                args: cd_args.to_vec(),
            });
        }
    }
    if rest.first().is_some_and(|first| first == "--post-move") {
        return Some(Hook::LegacyCdPost {
            pwd: rest.get(1).cloned(),
        });
    }
    Some(Hook::LegacyCdPre {
        target: rest.first().cloned(),
        pwd: rest.get(1).cloned(),
    })
}

/// Runs a hook and returns its exit status.
fn run_hook(hook: &Hook, env: &Env, catalog: &Catalog) -> i32 {
    let session = Session::new(env, catalog);
    match hook {
        Hook::CdPre { pwd, prev, args } => session.cd_pre(Path::new(pwd), Path::new(prev), args),
        Hook::CdPost { old, new, args } => session.cd_post(Path::new(old), Path::new(new), args),
        Hook::LegacyCdPre { target, pwd } => {
            session.legacy_cd_pre(target.as_deref(), pwd.as_deref())
        }
        Hook::LegacyCdPost { pwd } => session.legacy_cd_post(pwd.as_deref()),
        Hook::Fd => session.fd_hook(),
    }
}

// ---------------------------------------------------------------------------------------------
// Running
// ---------------------------------------------------------------------------------------------

/// Runs the program and returns its exit status. The environment is read once, here.
pub fn main() -> i32 {
    let env = Env::from_process();
    let catalog = crate::levels::catalog();
    run(
        &std::env::args_os().skip(1).collect::<Vec<_>>(),
        &env,
        &catalog,
    )
}

/// Runs one command line (without the program name) in this environment and returns the exit
/// status.
pub fn run(args: &[OsString], env: &Env, catalog: &Catalog) -> i32 {
    if cfg!(debug_assertions)
        && let Some(status) = reject_malformed_catalog(catalog)
    {
        return status;
    }
    if let Some(hook) = parse_hook(args) {
        return run_hook(&hook, env, catalog);
    }
    let cli = match parse(args) {
        Ok(cli) => cli,
        Err(error) => {
            // Printing can fail on a closed stream; the exit status still says what happened.
            let _ = error.print();
            return error.exit_code();
        }
    };
    if cli.version {
        let _ = writeln!(io::stdout(), "{}", version_line());
        return OK;
    }
    match cli.command {
        // Outside the game shell, the bare command starts it.
        None if !env.wrapped => boot(&cli, env),
        None => Session::new(env, catalog).show_current_level(),
        Some(command) => dispatch(&mut Session::new(env, catalog), command),
    }
}

/// Debug builds refuse to run with a malformed level catalog, so a broken level fails on the first
/// `cargo run` instead of in front of a player. Returns the exit status when it refuses.
fn reject_malformed_catalog(catalog: &Catalog) -> Option<i32> {
    let problems = catalog.validate();
    if problems.is_empty() {
        return None;
    }
    let mut err = io::stderr().lock();
    let _ = writeln!(
        err,
        "The level catalog is malformed (checked in debug builds):"
    );
    for problem in &problems {
        let _ = writeln!(err, "  - {problem}");
    }
    Some(FAILURE)
}

/// Starts the game shell and waits for it. Its own exit status is not a failure of the game;
/// only a shell that cannot be started is.
fn boot(cli: &Cli, env: &Env) -> i32 {
    let forced = cli.shell.as_deref().or(env.force_shell.as_deref());
    let shell = shell::detect(forced, env.shell.as_deref());
    let term = Term::stdout();
    let launched = match &env.exe {
        Some(binary) => shell::launch(&LaunchOptions {
            shell,
            binary: binary.clone(),
            devmode: cli.devmode,
            state_dir: env.state_dir.clone(),
        })
        .map(drop)
        .map_err(|error| error.to_string()),
        None => Err("nelze zjistit cestu k programu".to_owned()),
    };
    match launched {
        Ok(()) => OK,
        Err(reason) => {
            let _ = print_lines(&term, &screens::launch_failed(&reason, &term));
            FAILURE
        }
    }
}

/// Calls the session method of a command.
fn dispatch(session: &mut Session<'_>, command: Command) -> i32 {
    match command {
        Command::Init => session.init(),
        Command::Hint { repeat } => session.hint(repeat),
        Command::Submit { answer } => {
            let answer = (!answer.is_empty()).then(|| answer.join(" "));
            session.submit(answer.as_deref())
        }
        Command::Status => session.status(),
        Command::Skip => session.skip(),
        Command::Reset => session.reset(),
        Command::Repeat { section, level } => session.repeat(section, level.as_deref()),
        Command::Show { section, level } => session.show(section, level),
        Command::Remove { yes } => session.remove(yes),
        Command::Exit => session.exit(),
        Command::Update => session.update(),
        Command::Dev(dev) => match dev {
            DevCommand::Jump { id } => session.dev_jump(&id),
            DevCommand::Next => session.dev_next(),
            DevCommand::Prev => session.dev_prev(),
            DevCommand::Reload => session.dev_reload(),
            DevCommand::Start => session.dev_start(),
        },
    }
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use super::*;

    fn os(args: &[&str]) -> Vec<OsString> {
        args.iter().map(OsString::from).collect()
    }

    fn cli(args: &[&str]) -> Cli {
        parse(&os(args)).unwrap_or_else(|error| panic!("{args:?}: {error}"))
    }

    fn error(args: &[&str]) -> clap::Error {
        parse(&os(args)).expect_err("expected a usage error")
    }

    fn help_of(args: &[&str]) -> String {
        error(args).to_string()
    }

    #[test]
    fn the_clap_definition_is_consistent() {
        command().debug_assert();
    }

    // --- the surface ------------------------------------------------------------------------

    #[test]
    fn no_arguments_show_the_current_level() {
        let parsed = cli(&[]);
        assert!(parsed.command.is_none());
        assert!(!parsed.devmode && !parsed.version);
        assert_eq!(parsed.shell, None);
    }

    #[test]
    fn root_flags() {
        assert_eq!(cli(&["--shell", "fish"]).shell.as_deref(), Some("fish"));
        assert_eq!(cli(&["--shell", "BASH"]).shell.as_deref(), Some("BASH"));
        assert!(cli(&["--devmode"]).devmode);
        assert!(cli(&["--version"]).version);
        assert!(cli(&["-V"]).version);
        assert_eq!(error(&["--shell", "zsh"]).kind(), ErrorKind::InvalidValue);
        assert!(cli(&["--shell", "fish", "status"]).command.is_some());
    }

    #[test]
    fn hint_takes_a_repeat_flag() {
        assert!(matches!(
            cli(&["hint"]).command,
            Some(Command::Hint { repeat: false })
        ));
        for flag in ["-r", "--repeat"] {
            assert!(matches!(
                cli(&["hint", flag]).command,
                Some(Command::Hint { repeat: true })
            ));
        }
    }

    fn answer_of(args: &[&str]) -> Vec<String> {
        match cli(args).command {
            Some(Command::Submit { answer }) => answer,
            other => panic!("expected submit, got {other:?}"),
        }
    }

    #[test]
    fn submit_takes_any_answer_including_negative_numbers() {
        assert_eq!(answer_of(&["submit"]), Vec::<String>::new());
        assert_eq!(answer_of(&["submit", "start"]), ["start"]);
        assert_eq!(answer_of(&["submit", "-5"]), ["-5"]);
        assert_eq!(answer_of(&["submit", "--", "-5"]), ["-5"]);
        assert_eq!(answer_of(&["submit", "--not-a-flag"]), ["--not-a-flag"]);
        assert_eq!(answer_of(&["submit", "a", "b c", "-d"]), ["a", "b c", "-d"]);
        assert_eq!(answer_of(&["submit", ""]), [""]);
    }

    #[test]
    fn submit_still_answers_to_help() {
        assert_eq!(error(&["submit", "--help"]).kind(), ErrorKind::DisplayHelp);
    }

    #[test]
    fn repeat_and_show_take_their_options() {
        match cli(&["repeat", "--section", "3", "--level", "1.7"]).command {
            Some(Command::Repeat { section, level }) => {
                assert_eq!(section, Some(3));
                assert_eq!(level.as_deref(), Some("1.7"));
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            cli(&["repeat"]).command,
            Some(Command::Repeat {
                section: None,
                level: None
            })
        ));
        assert_eq!(
            error(&["repeat", "--section", "abc"]).kind(),
            ErrorKind::ValueValidation
        );
        // `show` validates its flags in the session (it prints the Czech usage line), not here.
        assert!(matches!(
            cli(&["show"]).command,
            Some(Command::Show {
                section: false,
                level: false
            })
        ));
        assert!(matches!(
            cli(&["show", "--section", "--level"]).command,
            Some(Command::Show {
                section: true,
                level: true
            })
        ));
    }

    #[test]
    fn remove_has_a_yes_flag() {
        assert!(matches!(
            cli(&["remove"]).command,
            Some(Command::Remove { yes: false })
        ));
        assert!(matches!(
            cli(&["remove", "--yes"]).command,
            Some(Command::Remove { yes: true })
        ));
    }

    #[test]
    fn the_plain_commands_parse() {
        for (name, expected) in [
            ("status", "Status"),
            ("skip", "Skip"),
            ("reset", "Reset"),
            ("exit", "Exit"),
            ("update", "Update"),
            ("init", "Init"),
        ] {
            let command = cli(&[name]).command.unwrap();
            assert_eq!(format!("{command:?}"), expected);
        }
    }

    #[test]
    fn dev_commands_parse() {
        match cli(&["dev", "jump", "1.4"]).command {
            Some(Command::Dev(DevCommand::Jump { id })) => assert_eq!(id, "1.4"),
            other => panic!("{other:?}"),
        }
        for (name, expected) in [
            ("next", "Next"),
            ("prev", "Prev"),
            ("reload", "Reload"),
            ("start", "Start"),
        ] {
            match cli(&["dev", name]).command {
                Some(Command::Dev(dev)) => assert_eq!(format!("{dev:?}"), expected),
                other => panic!("{other:?}"),
            }
        }
        assert_eq!(
            error(&["dev", "jump"]).kind(),
            ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn unknown_commands_and_options_are_usage_errors() {
        assert_eq!(error(&["bogus"]).exit_code(), 2);
        assert_eq!(error(&["hint", "--nope"]).exit_code(), 2);
        assert_eq!(error(&["status", "extra"]).exit_code(), 2);
    }

    #[test]
    fn the_hooks_are_not_clap_commands() {
        // They are parsed by hand, so clap would reject them.
        assert_eq!(error(&["cd-hook", "pre"]).exit_code(), 2);
        assert_eq!(error(&["fd-hook"]).exit_code(), 2);
    }

    // --- help -------------------------------------------------------------------------------

    #[test]
    fn root_help_is_czech_and_hides_the_hidden_commands() {
        let help = help_of(&["--help"]);
        assert!(
            help.contains("ShellGame - Interaktivní výuka navigace v terminálu."),
            "{help}"
        );
        assert!(help.contains("Použití: shellgame [MOŽNOSTI] [PŘÍKAZ]"));
        assert!(help.contains("Možnosti:"));
        assert!(help.contains("Příkazy:"));
        assert!(help.find("Možnosti:") < help.find("Příkazy:"));
        assert!(help.contains("Vynutit typ subshellu (bash/fish)."));
        assert!(help.contains("Zobrazit tuto nápovědu a ukončit."));
        for command in [
            "hint", "submit", "status", "skip", "reset", "repeat", "show", "remove", "exit",
            "update",
        ] {
            assert!(
                help.lines()
                    .any(|line| line.trim_start().starts_with(command)),
                "{command} missing from\n{help}"
            );
        }
        for hidden in ["init", "dev", "devmode", "cd-hook", "fd-hook"] {
            assert!(
                !help
                    .lines()
                    .any(|line| line.trim_start().starts_with(hidden)
                        || line.contains(&format!("--{hidden}"))),
                "{hidden} must stay hidden\n{help}"
            );
        }
        assert!(
            !help.contains("Usage:") && !help.contains("Options:") && !help.contains("Commands:")
        );
    }

    #[test]
    fn command_texts_come_from_the_python_version() {
        let expected = [
            ("hint", "Zobrazit nápovědu pro aktuální level."),
            ("submit", "Odeslat odpověď pro aktuální level."),
            ("status", "Zobrazit postup a statistiky."),
            ("skip", "Přeskočit aktuální level (pouze nepovinné levely)."),
            ("reset", "Obnovit strukturu aktuálního levelu."),
            (
                "repeat",
                "Znovu zobrazit zadání (aktuálního nebo zvoleného) levelu.",
            ),
            (
                "show",
                "Zobrazit znovu zadání aktuálního levelu nebo úvod aktuální sekce.",
            ),
            ("remove", "Smazat stav a pracovní prostor."),
            ("exit", "Ukončit ShellGame."),
        ];
        let root = help_of(&["--help"]);
        for (name, text) in expected {
            assert!(root.contains(text), "{name}: {text}\n{root}");
            let own = help_of(&[name, "--help"]);
            assert!(own.starts_with(text), "{name}\n{own}");
            assert!(own.contains(&format!("Použití: shellgame {name}")), "{own}");
        }
        assert!(
            help_of(&["hint", "--help"])
                .contains("Zobrazit znovu všechny již zobrazené nápovědy pro aktuální level.")
        );
        assert!(
            help_of(&["repeat", "--help"]).contains("Pokud neuvedete, zopakuje se aktuální level.")
        );
        assert!(help_of(&["show", "--help"]).contains("Zobrazit znovu úvod aktuální sekce (X.0)."));
    }

    #[test]
    fn subcommand_help_is_czech_too() {
        let help = help_of(&["hint", "--help"]);
        assert!(help.contains("Možnosti:"));
        assert!(!help.contains("Options:") && !help.contains("Usage:"));
        // The options and the help flag share one section.
        assert_eq!(help.matches("Možnosti:").count(), 1, "{help}");
        assert!(help.contains("-r, --repeat"));
        assert!(help.contains("-h, --help"));

        let submit = help_of(&["submit", "--help"]);
        assert!(submit.contains("Argumenty:"));
        assert!(submit.contains("[ODPOVĚĎ]"));

        let dev = help_of(&["dev", "--help"]);
        assert!(
            dev.contains("Příkazy:") && !dev.contains("Commands:"),
            "{dev}"
        );
        let jump = help_of(&["dev", "jump", "--help"]);
        assert!(
            jump.contains("Argumenty:") && !jump.contains("Arguments:"),
            "{jump}"
        );
    }

    #[test]
    fn the_version_line_reports_package_and_build() {
        let line = version_line();
        assert!(line.starts_with(&format!("shellgame {} (", env!("CARGO_PKG_VERSION"))));
        assert!(line.contains(BUILD) || BUILD.is_empty());
    }

    // --- hooks ------------------------------------------------------------------------------

    fn hook(args: &[&str]) -> Option<Hook> {
        parse_hook(&os(args))
    }

    #[test]
    fn other_commands_are_not_hooks() {
        assert_eq!(hook(&[]), None);
        assert_eq!(hook(&["status"]), None);
        assert_eq!(hook(&["--devmode", "hint"]), None);
        assert_eq!(hook(&["submit", "cd-hook"]), None);
    }

    #[test]
    fn the_fd_hook() {
        assert_eq!(hook(&["fd-hook"]), Some(Hook::Fd));
        assert_eq!(hook(&["--devmode", "fd-hook"]), Some(Hook::Fd));
    }

    #[test]
    fn the_new_pre_form_carries_every_cd_argument_verbatim() {
        let expected = |pwd: &str, prev: &str, args: &[&str]| {
            Some(Hook::CdPre {
                pwd: pwd.into(),
                prev: prev.into(),
                args: os(args),
            })
        };
        assert_eq!(
            hook(&["cd-hook", "pre", "/w", "/p", "--", ".."]),
            expected("/w", "/p", &[".."])
        );
        // A bare cd, `cd -`, options, `--` and the empty string are all just data.
        assert_eq!(
            hook(&["cd-hook", "pre", "/w", "", "--"]),
            expected("/w", "", &[])
        );
        assert_eq!(
            hook(&["cd-hook", "pre", "/w", "/p", "--", "-"]),
            expected("/w", "/p", &["-"])
        );
        assert_eq!(
            hook(&["cd-hook", "pre", "/w", "", "--", "-P", "--", "pre"]),
            expected("/w", "", &["-P", "--", "pre"])
        );
        assert_eq!(
            hook(&["cd-hook", "pre", "/w", "", "--", ""]),
            expected("/w", "", &[""])
        );
        assert_eq!(
            hook(&["--devmode", "cd-hook", "pre", "/w", "", "--", "x"]),
            expected("/w", "", &["x"])
        );
    }

    #[test]
    fn the_new_post_form() {
        assert_eq!(
            hook(&["cd-hook", "post", "/old", "/new", "--", "-P", "dir"]),
            Some(Hook::CdPost {
                old: "/old".into(),
                new: "/new".into(),
                args: os(&["-P", "dir"]),
            })
        );
        assert_eq!(
            hook(&["cd-hook", "post", "/old", "/new", "--"]),
            Some(Hook::CdPost {
                old: "/old".into(),
                new: "/new".into(),
                args: Vec::new(),
            })
        );
    }

    #[test]
    fn the_legacy_pre_form_is_target_and_pwd() {
        let legacy = |target: Option<&str>, pwd: Option<&str>| {
            Some(Hook::LegacyCdPre {
                target: target.map(OsString::from),
                pwd: pwd.map(OsString::from),
            })
        };
        assert_eq!(
            hook(&["cd-hook", "..", "/w"]),
            legacy(Some(".."), Some("/w"))
        );
        assert_eq!(hook(&["cd-hook", "", "/w"]), legacy(Some(""), Some("/w")));
        assert_eq!(hook(&["cd-hook", "-", "/w"]), legacy(Some("-"), Some("/w")));
        assert_eq!(
            hook(&["cd-hook", "-P", "/w"]),
            legacy(Some("-P"), Some("/w"))
        );
        assert_eq!(
            hook(&["cd-hook", "--", "/w"]),
            legacy(Some("--"), Some("/w"))
        );
        assert_eq!(hook(&["cd-hook"]), legacy(None, None));
        assert_eq!(hook(&["cd-hook", "x"]), legacy(Some("x"), None));
        assert_eq!(
            hook(&["--devmode", "cd-hook", "..", "/w"]),
            legacy(Some(".."), Some("/w"))
        );
    }

    #[test]
    fn a_directory_named_pre_or_post_is_a_legacy_target() {
        let legacy = |target: &str, pwd: &str| {
            Some(Hook::LegacyCdPre {
                target: Some(target.into()),
                pwd: Some(pwd.into()),
            })
        };
        assert_eq!(hook(&["cd-hook", "pre", "/w"]), legacy("pre", "/w"));
        assert_eq!(hook(&["cd-hook", "post", "/w"]), legacy("post", "/w"));
        // Without the `--` separator at the fourth place it is not the new form either.
        assert_eq!(
            hook(&["cd-hook", "pre", "/w", "x", "y"]),
            Some(Hook::LegacyCdPre {
                target: Some("pre".into()),
                pwd: Some("/w".into()),
            })
        );
    }

    #[test]
    fn the_legacy_post_form() {
        assert_eq!(
            hook(&["cd-hook", "--post-move", "/w"]),
            Some(Hook::LegacyCdPost {
                pwd: Some("/w".into())
            })
        );
        assert_eq!(
            hook(&["cd-hook", "--post-move"]),
            Some(Hook::LegacyCdPost { pwd: None })
        );
        assert_eq!(
            hook(&["--devmode", "cd-hook", "--post-move", "/w"]),
            Some(Hook::LegacyCdPost {
                pwd: Some("/w".into())
            })
        );
    }

    #[test]
    fn hook_arguments_may_be_invalid_utf8() {
        use std::os::unix::ffi::OsStringExt;
        let bad = OsString::from_vec(b"/tmp/\xff".to_vec());
        let args = vec![
            OsString::from("cd-hook"),
            OsString::from("pre"),
            bad.clone(),
            OsString::new(),
            OsString::from("--"),
        ];
        assert_eq!(
            parse_hook(&args),
            Some(Hook::CdPre {
                pwd: bad,
                prev: OsString::new(),
                args: Vec::new(),
            })
        );
    }
}
