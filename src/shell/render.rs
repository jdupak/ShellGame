//! Rendering of the shell integration scripts.
//!
//! The scripts are `bash.sh` and `fish.fish` next to this file, compiled in with `include_str!`.
//! They are static: everything that varies per machine (the binary path, the script path, the
//! fd-hook launcher) reaches the shell through environment variables, so nothing is ever quoted
//! into script text. The only thing rendering does is to put the developer shortcuts in place of
//! a `@DEV_SHORTCUTS@` placeholder, using plain `str::replace`.

use super::detect::ShellKind;

const BASH_TEMPLATE: &str = include_str!("bash.sh");
const FISH_TEMPLATE: &str = include_str!("fish.fish");

/// The placeholder line that [`DEV_SHORTCUTS`] replace in both templates.
const DEV_PLACEHOLDER: &str = "@DEV_SHORTCUTS@";

/// Developer shortcuts for `--devmode`: bash aliases and fish abbreviations.
pub const DEV_SHORTCUTS: [(&str, &str); 7] = [
    ("r", "shellgame dev reload"),
    ("n", "shellgame dev next"),
    ("p", "shellgame dev prev"),
    ("j", "shellgame dev jump"),
    ("s", "shellgame dev start"),
    ("e", "shellgame exit"),
    ("g", "shellgame"),
];

/// What can differ between two renderings of the same script.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScriptOptions {
    /// Define the developer shortcuts (`r n p j s e g`).
    pub devmode: bool,
}

/// Renders the integration script for `shell`.
#[must_use]
pub fn render(shell: ShellKind, options: &ScriptOptions) -> String {
    match shell {
        ShellKind::Bash => render_bash(options),
        ShellKind::Fish => render_fish(options),
    }
}

/// Renders the bash integration script (`bash --rcfile <script> -i`).
#[must_use]
pub fn render_bash(options: &ScriptOptions) -> String {
    substitute(
        BASH_TEMPLATE,
        &dev_shortcuts(options, "alias {name}=\"{command}\""),
    )
}

/// Renders the fish integration script (`source $SHELLGAME_INIT`).
#[must_use]
pub fn render_fish(options: &ScriptOptions) -> String {
    substitute(
        FISH_TEMPLATE,
        &dev_shortcuts(options, "abbr -a {name} \"{command}\""),
    )
}

fn substitute(template: &str, shortcuts: &str) -> String {
    template.replace(DEV_PLACEHOLDER, shortcuts)
}

/// The shortcut definitions in `format` (with `{name}` and `{command}` slots), or an empty string
/// when developer mode is off.
fn dev_shortcuts(options: &ScriptOptions, format: &str) -> String {
    if !options.devmode {
        return String::new();
    }
    let definitions = DEV_SHORTCUTS
        .map(|(name, command)| format.replace("{name}", name).replace("{command}", command));
    format!("# Dev shortcuts\n{}", definitions.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEV: ScriptOptions = ScriptOptions { devmode: true };
    const PLAIN: ScriptOptions = ScriptOptions { devmode: false };

    #[test]
    fn each_template_has_exactly_one_placeholder_and_none_is_left_after_rendering() {
        for template in [BASH_TEMPLATE, FISH_TEMPLATE] {
            assert_eq!(template.matches(DEV_PLACEHOLDER).count(), 1);
        }
        for shell in [ShellKind::Bash, ShellKind::Fish] {
            for options in [PLAIN, DEV] {
                let script = render(shell, &options);
                assert!(
                    !has_placeholder(&script),
                    "{shell} {options:?} still has a placeholder"
                );
            }
        }
    }

    /// Whether the text contains an `@NAME@` placeholder. (A bare `@` is fine: `"$@"`, `${a[@]}`.)
    fn has_placeholder(text: &str) -> bool {
        let pieces: Vec<&str> = text.split('@').collect();
        pieces.len() > 2
            && pieces[1..pieces.len() - 1].iter().any(|piece| {
                !piece.is_empty() && piece.chars().all(|c| c.is_ascii_uppercase() || c == '_')
            })
    }

    #[test]
    fn placeholder_detection_works() {
        assert!(has_placeholder("a\n@DEV_SHORTCUTS@\nb"));
        assert!(!has_placeholder("\"$@\" and ${words[@]} and ${#words[@]}"));
        assert!(!has_placeholder("no at sign"));
    }

    #[test]
    fn bash_dev_shortcuts_are_aliases() {
        let script = render_bash(&DEV);
        for (name, command) in DEV_SHORTCUTS {
            assert!(
                script.contains(&format!("alias {name}=\"{command}\"\n")),
                "{name}"
            );
        }
        assert!(!render_bash(&PLAIN).contains("alias "));
        assert!(!render_bash(&PLAIN).contains("Dev shortcuts"));
    }

    #[test]
    fn fish_dev_shortcuts_are_abbreviations() {
        let script = render_fish(&DEV);
        for (name, command) in DEV_SHORTCUTS {
            assert!(
                script.contains(&format!("abbr -a {name} \"{command}\"\n")),
                "{name}"
            );
        }
        assert!(!render_fish(&PLAIN).contains("abbr "));
        assert!(!render_fish(&PLAIN).contains("Dev shortcuts"));
    }

    #[test]
    fn rendering_is_deterministic_and_dispatches_by_shell() {
        assert_eq!(render_bash(&DEV), render(ShellKind::Bash, &DEV));
        assert_eq!(render_fish(&PLAIN), render(ShellKind::Fish, &PLAIN));
        assert_eq!(render_bash(&DEV), render_bash(&DEV));
        assert_ne!(render_bash(&DEV), render_bash(&PLAIN));
        assert_ne!(render_bash(&PLAIN), render_fish(&PLAIN));
        assert_eq!(ScriptOptions::default(), PLAIN);
    }

    #[test]
    fn scripts_never_guess_the_workspace() {
        // The workspace comes from SHELLGAME_WORKSPACE only. A $USER-derived fallback would let
        // a stale or foreign directory collect evidence markers.
        for script in [render_bash(&DEV), render_fish(&DEV)] {
            assert!(!script.contains("$USER"));
            assert!(!script.contains("/tmp/shellgame-"));
        }
    }

    #[test]
    fn bash_script_avoids_bash_4_features() {
        // macOS ships bash 3.2 and the CI that would notice runs elsewhere, so the features that
        // arrived after 3.2 are listed here. (`globasciiranges` is bash 5 and is guarded.)
        let code: String = render_bash(&DEV)
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");
        for forbidden in [
            "declare -A",
            "local -A",
            "typeset -A",
            "local -n",
            "declare -n",
            "mapfile",
            "readarray",
            "coproc",
            "&>>",
            "|&",
            ";&",
            ";;&",
            ",,}",
            "^^}",
            "[[ -v",
            "[[ -R",
            "read -N",
            "read -i",
            "printf '%(",
            "globstar",
            "[-1]",
            "EPOCHSECONDS",
            "EPOCHREALTIME",
            "BASH_ARGV0",
            "SRANDOM",
            "wait -n",
            "$'\\u",
        ] {
            assert!(!code.contains(forbidden), "bash 4+ syntax: {forbidden}");
        }
        assert!(
            code.contains("shopt -s globasciiranges 2>/dev/null || true"),
            "newer shopt options must stay guarded"
        );
    }
}
