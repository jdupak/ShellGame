//! The walk home of level 1.9: from `/` to `$HOME`, one directory at a time.
//!
//! A `cd` that would skip steps is blocked: a bare `cd`, any absolute path except `/` (which
//! includes `cd ~` and `cd $HOME`, since the shell has expanded those before the hook sees
//! them), and any path with more than one segment. The walk itself is tracked after each
//! successful move, in the level's private marker, as the list of directories visited since
//! `/`. Reaching `$HOME` at the end of an unbroken walk records the level's `cd` evidence;
//! stepping aside, or up, restarts it.
//!
//! Everything here compares the *logical* directories the shell reports, never canonical paths.
//! A symlink to `$HOME` is therefore no shortcut (its logical path is not `$HOME`), and a
//! `$HOME` below a symlinked directory (macOS `/var` is `/private/var`) still works.

use std::path::{Component, Path};

use crate::engine::prelude::*;

/// The directories visited since `/`, one per line.
const PROGRESS: Marker = Marker::Own("walk");

const NO_JUMPS: &str = "Skoky nejsou povoleny. Jděte krok za krokem.";
const NO_ABSOLUTE: &str = "Absolutní skoky nejsou povoleny (kromě cd /).";
const ONE_SEGMENT: &str = "Cestujte po jednom segmentu (adresáři).";

/// Blocks every `cd` that is not a single step. `cd /` is allowed: it starts the walk.
pub(super) fn rule(mv: &CdMove) -> Result<(), String> {
    if mv.is_bare() {
        return Err(NO_JUMPS.to_owned());
    }
    if mv.target == "/" {
        return Ok(());
    }
    if mv.target.starts_with('/') {
        return Err(NO_ABSOLUTE.to_owned());
    }
    if mv.target.trim_end_matches('/').contains('/') {
        return Err(ONE_SEGMENT.to_owned());
    }
    Ok(())
}

/// Tracks the walk after a successful move. Evidence is best effort, like the engine's own: a
/// marker that cannot be written must not break the player's `cd`, and the level then simply
/// stays incomplete.
pub(super) fn progress(mv: &CdMove, markers: &Markers) {
    let Some(dest) = mv.dest else { return };
    if dest == Path::new("/") {
        let _ = markers.create_with(&PROGRESS, "/");
        return;
    }
    let Some(walked) = markers.read(&PROGRESS) else {
        return;
    };
    let Some(last) = walked.trim().lines().last().map(Path::new) else {
        return;
    };
    if dest.parent() == Some(last) {
        let _ = markers.create_with(&PROGRESS, &format!("{walked}\n{}", dest.display()));
        if dest == mv.ctx.home {
            let _ = markers.create(&Marker::Cd);
        }
    } else if dest != last {
        let _ = markers.remove(&PROGRESS);
    }
}

/// The solution: `cd /`, then one `cd` per component of `$HOME`, then submit. Valid in bash
/// and fish.
pub(super) fn solution(ctx: &Ctx) -> String {
    let steps = ctx.home.components().filter_map(|part| match part {
        Component::Normal(name) => Some(format!("cd {}", quote(&name.to_string_lossy()))),
        _ => None,
    });
    let mut lines = vec!["cd /".to_owned()];
    lines.extend(steps);
    lines.push("shellgame submit".to_owned());
    lines.join("\n")
}

/// A directory name as one shell word that bash and fish read the same way: bare when it is
/// plain, single-quoted when that is safe in both, double-quoted with escapes otherwise.
fn quote(name: &str) -> String {
    let plain = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_.+-=:@%,".contains(c));
    if plain {
        name.to_owned()
    } else if !name.contains(['\'', '\\']) {
        format!("'{name}'")
    } else {
        let escaped = name
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$");
        format!("\"{escaped}\"")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Level 1.9 in a temporary workspace, played the way the shell reports moves.
    struct Walk {
        _tmp: tempfile::TempDir,
        workspace: PathBuf,
        section: Section,
        home: PathBuf,
    }

    impl Walk {
        /// A walk for a player whose `$HOME` is `home`. The directories need not exist: the walk
        /// is about the logical paths the shell reports.
        fn new(home: &str) -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let workspace = tmp.path().join("workspace");
            let section = super::super::section();
            let level = section.get(9).unwrap();
            level.prepare(&section, &workspace).unwrap();
            Self {
                _tmp: tmp,
                workspace,
                section,
                home: PathBuf::from(home),
            }
        }

        fn level(&self) -> &Level {
            self.section.get(9).unwrap()
        }

        fn ctx(&self) -> Ctx<'_> {
            let root = self.workspace.join("level-1");
            Ctx::new(&self.workspace, root, None, &self.home, self.level().id)
        }

        /// One `cd`: the hook is asked first and, when it allows the move, told where the shell
        /// ended up.
        fn cd(&self, target: &str, pwd: &str, dest: &str) -> CdDecision {
            let ctx = self.ctx();
            let args: Vec<String> = if target.is_empty() {
                Vec::new()
            } else {
                vec![target.to_owned()]
            };
            let pwd = Path::new(pwd);
            let mv = CdMove::new(&ctx, target, &args, pwd);
            let decision = self.level().cd_pre_move(&mv);
            if decision == CdDecision::Allow {
                self.level().cd_post_move(&mv.with_dest(Path::new(dest)));
            }
            decision
        }

        fn walked(&self) -> Option<String> {
            self.ctx().markers().read(&PROGRESS)
        }

        fn done(&self) -> bool {
            self.ctx().markers().exists(&Marker::Cd)
        }
    }

    #[test]
    fn every_jump_is_blocked_with_its_own_message() {
        let walk = Walk::new("/home/ada");
        let blocked = |target, expected: &str| {
            assert_eq!(
                walk.cd(target, "/", "/home/ada"),
                CdDecision::Block(expected.to_owned()),
                "cd {target:?}"
            );
        };
        // A bare `cd`.
        blocked("", NO_JUMPS);
        // `cd ~` and `cd $HOME` reach the hook as the absolute path the shell expanded them to.
        blocked("/home/ada", NO_ABSOLUTE);
        blocked("//", NO_ABSOLUTE);
        // More than one segment, with or without a trailing slash.
        blocked("home/ada", ONE_SEGMENT);
        blocked("home/ada/", ONE_SEGMENT);
        blocked("./home", ONE_SEGMENT);
        blocked("../home", ONE_SEGMENT);
        assert!(!walk.done());
        assert_eq!(walk.walked(), None);
    }

    #[test]
    fn a_single_segment_is_a_step_with_or_without_a_trailing_slash() {
        let walk = Walk::new("/home/ada");
        assert_eq!(walk.cd("/", "/tmp", "/"), CdDecision::Allow);
        assert_eq!(walk.cd("home/", "/", "/home"), CdDecision::Allow);
        assert_eq!(walk.walked().as_deref(), Some("/\n/home"));
    }

    #[test]
    fn walking_from_the_root_to_home_completes_the_level() {
        let walk = Walk::new("/home/ada");
        assert_eq!(walk.cd("/", "/tmp/workspace", "/"), CdDecision::Allow);
        assert_eq!(walk.cd("home", "/", "/home"), CdDecision::Allow);
        assert!(!walk.done(), "one directory short");
        assert_eq!(walk.cd("ada", "/home", "/home/ada"), CdDecision::Allow);
        assert!(walk.done());
        assert_eq!(walk.walked().as_deref(), Some("/\n/home\n/home/ada"));
    }

    #[test]
    fn once_the_walk_is_done_the_player_moves_freely_again() {
        let walk = Walk::new("/home/ada");
        walk.cd("/", "/tmp", "/");
        walk.cd("home", "/", "/home");
        walk.cd("ada", "/home", "/home/ada");
        assert_eq!(walk.cd("", "/home/ada", "/home/ada"), CdDecision::Allow);
        assert_eq!(walk.cd("/etc", "/home/ada", "/etc"), CdDecision::Allow);
    }

    #[test]
    fn a_symlink_that_points_home_is_not_a_shortcut() {
        let walk = Walk::new("/home/ada");
        walk.cd("/", "/tmp", "/");
        walk.cd("shortcut", "/", "/shortcut");
        // The shell's logical path is /shortcut/ada, whatever the directory really is.
        walk.cd("ada", "/shortcut", "/shortcut/ada");
        assert!(!walk.done());
    }

    #[test]
    fn a_home_below_a_symlinked_directory_still_completes() {
        // macOS: /var is a link to /private/var, but the shell reports the path as it was typed.
        let walk = Walk::new("/var/folders/ab/home");
        walk.cd("/", "/tmp", "/");
        for (step, from, to) in [
            ("var", "/", "/var"),
            ("folders", "/var", "/var/folders"),
            ("ab", "/var/folders", "/var/folders/ab"),
            ("home", "/var/folders/ab", "/var/folders/ab/home"),
        ] {
            assert_eq!(walk.cd(step, from, to), CdDecision::Allow);
        }
        assert!(walk.done());
    }

    #[test]
    fn stepping_aside_or_up_breaks_the_walk() {
        let walk = Walk::new("/home/ada");
        walk.cd("/", "/tmp", "/");
        walk.cd("home", "/", "/home");
        // A move that is allowed (a single segment) but lands somewhere unrelated, as `cd -`
        // can: the progress is dropped, and nothing is tracked until the walk restarts at `/`.
        walk.cd("-", "/home", "/tmp");
        assert_eq!(walk.walked(), None);
        walk.cd("ada", "/tmp", "/tmp/ada");
        assert!(!walk.done());

        // Going up lands on `/`, which is the start of a fresh walk.
        walk.cd("/", "/tmp/ada", "/");
        walk.cd("home", "/", "/home");
        walk.cd("..", "/home", "/");
        assert_eq!(walk.walked().as_deref(), Some("/"));
        assert!(!walk.done());

        // Staying where one is keeps the progress.
        walk.cd("home", "/", "/home");
        walk.cd(".", "/home", "/home");
        assert_eq!(walk.walked().as_deref(), Some("/\n/home"));
        walk.cd("ada", "/home", "/home/ada");
        assert!(walk.done());
    }

    #[test]
    fn a_walk_that_reaches_home_without_starting_at_the_root_does_not_count() {
        let walk = Walk::new("/home/ada");
        // The player starts from /home without ever having been at /.
        walk.cd("ada", "/home", "/home/ada");
        assert!(!walk.done());
        assert_eq!(walk.walked(), None);
    }

    #[test]
    fn preparing_the_level_forgets_the_walk() {
        let walk = Walk::new("/home/ada");
        walk.cd("/", "/tmp", "/");
        walk.cd("home", "/", "/home");
        walk.cd("ada", "/home", "/home/ada");
        assert!(walk.done() && walk.walked().is_some());
        walk.level()
            .prepare(&walk.section, &walk.workspace)
            .unwrap();
        assert!(!walk.done());
        assert_eq!(walk.walked(), None);
    }

    /// The `cd` lines of a script, as (target) words.
    fn cd_targets(script: &str) -> Vec<String> {
        script
            .lines()
            .filter_map(|line| line.strip_prefix("cd "))
            .map(|word| word.trim_matches(['\'', '"']).to_owned())
            .collect()
    }

    #[test]
    fn the_solution_is_the_walk_from_the_root() {
        let walk = Walk::new("/home/ada");
        let script = solution(&walk.ctx());
        assert_eq!(script, "cd /\ncd home\ncd ada\nshellgame submit");

        // Replayed through the level, it completes the walk.
        let mut here = "/tmp".to_owned();
        for target in cd_targets(&script) {
            let dest = match target.as_str() {
                "/" => "/".to_owned(),
                step => format!("{}/{step}", here.trim_end_matches('/')),
            };
            assert_eq!(walk.cd(&target, &here, &dest), CdDecision::Allow);
            here = dest;
        }
        assert!(walk.done());
    }

    #[test]
    fn the_solution_quotes_unusual_directory_names_for_both_shells() {
        let walk = Walk::new("/home/Jan Novák");
        assert_eq!(
            solution(&walk.ctx()),
            "cd /\ncd home\ncd 'Jan Novák'\nshellgame submit"
        );
        assert_eq!(quote("plain-name_1.2"), "plain-name_1.2");
        assert_eq!(quote("a b"), "'a b'");
        assert_eq!(quote(""), "''");
        assert_eq!(quote("it's"), "\"it's\"");
        assert_eq!(quote("a\\b$c\"d"), "\"a\\\\b\\$c\\\"d\"");
    }
}
