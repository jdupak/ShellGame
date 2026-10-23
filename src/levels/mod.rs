//! The whole course, in order: twelve sections from the welcome ([`s00_welcome`]) to searching
//! ([`s11_search`]), listed once in [`COURSE`]. Each section's page below states its goals.
//!
//! **Audience.** First-semester students new to the shell. Players read Czech, the rest is English.
//!
//! **Structure.** A [`Section`] is a title, an intro (pages split by `---`) and its levels. A
//! [`Level`](crate::game::Level) is one struct literal: the assignment, escalating hints, the
//! files it sets up, how it is judged (an answer, checks on files and location, or both) and a
//! solution script that the tests replay.
//!
//! **Numbers are positions.** "S.L" is the L-th level of the S-th section of [`COURSE`]; there are
//! no ids, so adding, moving or removing a level or section renumbers what follows and shifts saved
//! games. Section S works in `level-S` of the workspace.
//!
//! **Progression.** Linear and cumulative: a section may use what earlier ones taught and asks for
//! nothing that a later one teaches, except a few tools its text hands over ready-made (`file *` in
//! [`s05_inspect`], `mv *.log logs/` in [`s06_copy_move`], `grep` in [`s09_redirection`]). Each
//! section file starts with its goals, and every level has to fit them. Solution scripts may use
//! plumbing (`cut`, `grep`, `wc`) that the student is not asked to type.

pub mod s00_welcome;
pub mod s01_navigation;
pub mod s02_files;
pub mod s03_hidden;
pub mod s04_create;
pub mod s05_inspect;
pub mod s06_copy_move;
pub mod s07_wildcards;
pub mod s08_permissions;
pub mod s09_redirection;
pub mod s10_streams;
pub mod s11_search;

use crate::game::Section;

pub static COURSE: &[Section] = &[
    s00_welcome::SECTION,
    s01_navigation::SECTION,
    s02_files::SECTION,
    s03_hidden::SECTION,
    s04_create::SECTION,
    s05_inspect::SECTION,
    s06_copy_move::SECTION,
    s07_wildcards::SECTION,
    s08_permissions::SECTION,
    s09_redirection::SECTION,
    s10_streams::SECTION,
    s11_search::SECTION,
];

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::COURSE;
    use crate::game::paths::split;
    use crate::game::testing::World;
    use crate::game::{CD_DONE, CdMove, Level, Pos, cd, dedent};

    fn levels() -> impl Iterator<Item = (Pos, &'static Level)> {
        Pos::all().map(|pos| (pos, pos.level()))
    }

    #[test]
    fn everything_a_player_reads_is_there_and_every_level_can_be_completed() {
        for section in COURSE {
            assert!(!section.title.is_empty() && !section.intro.trim().is_empty());
        }
        for (pos, level) in levels() {
            assert!(!level.title.is_empty() && !level.text.trim().is_empty(), "{pos}");
            assert!(
                !level.hints.is_empty() && level.hints.iter().all(|h| !h.trim().is_empty()),
                "{pos}: hints"
            );
            assert!(level.answer.is_set() || !level.check.is_empty(), "{pos}: nothing to complete");
            // Judging drops trailing slashes, so an answer that ends in one could not be given.
            let (secrets, _) = level.answer.secrets_and_texts();
            assert!(secrets.iter().all(|secret| !secret.ends_with('/')), "{pos}: {secrets:?}");
            let solution = dedent(level.solution);
            let last = solution.lines().last().unwrap_or_default();
            // The last line submits, through `shellgame` or an alias of it.
            assert_eq!(last.split_whitespace().nth(1), Some("submit"), "{pos}: {last:?}");
        }
    }

    #[test]
    fn declared_paths_are_relative_without_dotdot() {
        for (pos, level) in levels() {
            let setup = level.setup.iter().filter_map(|setup| setup.path());
            let checks = level.check.iter().flat_map(|check| check.paths());
            let clauses = level.cd.iter().flat_map(|clause| clause.paths());
            for path in setup.chain(checks).chain(clauses).chain([level.start]) {
                assert!(split(path).is_ok(), "{pos}: {path:?}");
            }
        }
    }

    /// A reply to a wrong answer or to a failed check never contains the answer, and a failed `at`
    /// does not name where to go.
    #[test]
    fn a_failure_never_reveals_the_answer() {
        let contains =
            |text: &str, secret: &str| text.to_lowercase().contains(&secret.to_lowercase());
        let mut leaks = vec![];
        for (pos, level) in levels() {
            let (secrets, mut texts) = level.answer.secrets_and_texts();
            texts.extend(level.check.iter().flat_map(|check| check.texts()));
            for secret in secrets.iter().filter(|secret| secret.chars().count() >= 4) {
                if let Some(text) = texts.iter().find(|text| contains(text, secret)) {
                    leaks.push(format!("{pos}: {secret:?} appears in {text:.60?}"));
                }
            }
            for check in level.check {
                let Some(name) = check.target_dir().and_then(|dir| split(dir).ok()?.pop()) else {
                    continue;
                };
                if let Some(text) = check.texts().into_iter().find(|text| contains(text, name)) {
                    leaks.push(format!("{pos}: a failed `at` names {name:?}: {text:.60?}"));
                }
            }
        }
        assert!(leaks.is_empty(), "{}", leaks.join("\n"));
    }

    #[test]
    fn raw_strings_are_indented_blocks() {
        let intros = COURSE.iter().map(|section| ("intro", section.intro));
        let blocks = levels().flat_map(|(_, level)| {
            [("text", level.text), ("solution", level.solution), ("success", level.success)]
        });
        for (what, block) in intros.chain(blocks) {
            assert!(!block.contains('\t'), "{what}: tabs ({block:.40})");
            // A one-line solution or success may be a plain string; every other block starts on
            // the line after its opener and ends with the closing delimiter on a line of its own.
            if !matches!(what, "solution" | "success") || block.contains('\n') {
                assert!(
                    block.starts_with('\n'),
                    "{what} must start on the line after the opener: {block:.40}"
                );
                assert!(
                    block.rsplit('\n').next().unwrap().chars().all(|c| c == ' '),
                    "{what}: {block:.40}"
                );
                let text = dedent(block);
                assert!(!text.starts_with('\n') && !text.ends_with('\n'), "{what}: {text:.40}");
            }
        }
    }

    #[test]
    fn mentions_of_levels_and_sections_match_their_position() {
        let mentions = |text: &str| -> Vec<usize> {
            let found = ["level-", "Sekce ", "Sekci "]
                .iter()
                .flat_map(|prefix| text.match_indices(prefix).map(|(at, _)| at + prefix.len()));
            found
                .filter_map(|at| {
                    text[at..].split(|c: char| !c.is_ascii_digit()).next()?.parse().ok()
                })
                .collect()
        };
        for (number, section) in COURSE.iter().enumerate() {
            assert!(
                mentions(section.intro).iter().all(|found| *found == number),
                "intro of section {number}"
            );
            // The intro panel's title already shows "Sekce N: <title>"; don't repeat it in the text.
            assert!(
                !dedent(section.intro).starts_with('#'),
                "intro of section {number} starts with a heading"
            );
        }
        for (pos, level) in levels() {
            let text = format!("{}\n{}\n{}", level.text, level.hints.join("\n"), level.success);
            for found in mentions(&text) {
                assert_eq!(
                    found, pos.section,
                    "{pos} mentions {found}: another section is named by its title"
                );
            }
        }
    }

    #[test]
    fn every_level_prepares_in_course_order_and_nothing_completes_at_its_start() {
        let world = World::new();
        for (pos, level) in levels() {
            let ctx = world.start(pos);
            assert!(level.judge(&ctx, None).is_err(), "{pos}: an empty submission completes it");
            if let (false, Some(answer)) = (level.check.is_empty(), level.answer.sample()) {
                assert!(
                    level.judge(&ctx, Some(&answer)).is_err(),
                    "{pos}: {answer:?} completes it without the work"
                );
            }
        }
    }

    #[test]
    fn every_cd_policy_blocks_something_and_never_traps_the_player() {
        let world = World::new();
        for (pos, level) in levels().filter(|(_, level)| !level.cd.is_empty()) {
            let ctx = world.start(pos);
            let start = ctx.cwd.clone().unwrap();
            let named = level.cd.iter().flat_map(|clause| clause.paths());
            let places: Vec<PathBuf> = [start.clone()]
                .into_iter()
                .chain(named.map(|p| ctx.section_root.join(p)))
                .collect();
            let decide = |place: &PathBuf, target: &str| {
                cd::before(pos, &CdMove { target, pwd: place, prev: None, dest: None, ctx: &ctx })
            };
            let attempts = ["/etc", "..", "../..", "/", "", "elsewhere", "-"];
            let blocked: Vec<_> = places
                .iter()
                .flat_map(|place| attempts.map(move |target| (place, target)))
                .filter(|(place, target)| decide(place, target).is_err())
                .collect();
            assert!(!blocked.is_empty(), "{pos}: the policy blocks nothing");
            for place in &places {
                let back = decide(place, &start.to_string_lossy());
                assert_eq!(
                    back,
                    Ok(()),
                    "{pos}: the way back to the start is blocked from {place:?}"
                );
            }
            // Once the drill is done, nothing is blocked any more.
            fs::write(ctx.marker(CD_DONE), "").unwrap();
            for (place, target) in blocked {
                let after = decide(place, target);
                assert_eq!(after, Ok(()), "{pos}: `cd {target}` from {place:?} after the drill");
            }
        }
    }
}
