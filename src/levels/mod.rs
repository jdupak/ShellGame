//! Level content: the whole course, one module per section.
//!
//! A section module (`s04_vytvareni`, ...) has a `section()` that lists its levels, one small
//! function per level, and next to it the assignment of every level as a Markdown file
//! (`NN-slug.md`, the intro is `00-uvod.md`) that the level includes with `include_str!`.
//! Level ids are explicit, so the 1.11 gap stays a gap and reordering never renumbers.
//!
//! [`catalog()`] collects the sections; `Catalog::validate` lints the result.

use crate::engine::catalog::Catalog;

mod s00_uvod;
mod s01_navigace;
mod s02_soubory;
mod s03_skryte;
mod s04_vytvareni;
mod s05_zkoumani;
mod s06_kopirovani;
mod s07_zoliky;
mod s08_opravneni;
mod s09_vstup_vystup;
mod s10_chyby;
mod s11_vyhledavani;

/// The whole course.
pub fn catalog() -> Catalog {
    Catalog::new(vec![
        s00_uvod::section(),
        s01_navigace::section(),
        s02_soubory::section(),
        s03_skryte::section(),
        s04_vytvareni::section(),
        s05_zkoumani::section(),
        s06_kopirovani::section(),
        s07_zoliky::section(),
        s08_opravneni::section(),
        s09_vstup_vystup::section(),
        s10_chyby::section(),
        s11_vyhledavani::section(),
    ])
}

/// A solution script from its lines: what a student would type, one command per line, run in
/// the level's start directory. It must be valid in bash and fish.
fn script<const N: usize>(lines: [&str; N]) -> String {
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::prelude::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn the_catalog_is_sound() {
        let catalog = catalog();
        assert!(catalog.validate().is_empty(), "{:#?}", catalog.validate());
    }

    /// A workspace whose path contains a space (as in the solution replay) and a home.
    struct World {
        _tmp: tempfile::TempDir,
        workspace: PathBuf,
        home: PathBuf,
    }

    impl World {
        fn new() -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let base = fs::canonicalize(tmp.path()).unwrap();
            let home = base.join("home/ada");
            fs::create_dir_all(&home).unwrap();
            Self {
                workspace: base.join("shellgame workspace"),
                home,
                _tmp: tmp,
            }
        }

        /// Prepare a level; its context has the player in the start directory (if it has one).
        fn start<'a>(&'a self, section: &Section, level: &Level) -> Ctx<'a> {
            level.prepare(section, &self.workspace).unwrap();
            let root = section.section_root(&self.workspace).unwrap();
            let cwd = level.start_dir(&self.workspace, &root);
            if let Some(dir) = &cwd {
                assert!(dir.is_dir(), "{}: the start {dir:?} must exist", level.id);
            }
            section
                .ctx(level, &self.workspace, cwd, &self.home)
                .unwrap()
        }
    }

    #[test]
    fn every_section_opens_with_its_intro_and_lists_its_levels_in_order() {
        let catalog = catalog();
        for (index, section) in catalog.sections().iter().enumerate() {
            assert_eq!(usize::from(section.number), index, "sections 0, 1, 2, ...");
            assert!(
                section.levels[0].is_intro(),
                "section {} opens with an intro",
                section.number
            );
            let numbers: Vec<u16> = section.levels.iter().map(|level| level.id.number).collect();
            assert!(
                numbers.windows(2).all(|pair| pair[0] < pair[1]),
                "section {} lists its levels in order: {numbers:?}",
                section.number
            );
        }
    }

    #[test]
    fn every_level_has_text_and_every_task_has_hints() {
        for (_, level) in catalog().iter() {
            assert!(!level.text.trim().is_empty(), "{}: no text", level.id);
            if !level.is_intro() {
                assert!(!level.hints.is_empty(), "{}: no hints", level.id);
            }
        }
    }

    #[test]
    fn preparing_every_level_in_one_workspace_works_and_starts_in_an_existing_directory() {
        // In course order and in the same workspace, as a real game does: fixtures of one
        // level must not break another's.
        let world = World::new();
        let catalog = catalog();
        for (section, level) in catalog.iter() {
            world.start(section, level);
        }
    }

    #[test]
    fn nothing_is_completed_by_submitting_at_the_start() {
        // Requirements are unconditional: an empty submission never completes a task, and a
        // known answer does not either while the work is not done.
        let world = World::new();
        let catalog = catalog();
        for (section, level) in catalog.iter().filter(|(_, level)| !level.is_intro()) {
            let ctx = world.start(section, level);
            assert!(
                level.validate(&ctx, None).is_fail(),
                "{}: an empty submission completes the level right after it starts",
                level.id
            );
            if level.requirements.is_empty() {
                continue;
            }
            if let Some(answer) = level.answer.as_ref().and_then(Answer::static_answer) {
                assert!(
                    level.validate(&ctx, Some(&answer)).is_fail(),
                    "{}: the answer {answer:?} completes the level without the work",
                    level.id
                );
            }
        }
    }

    #[test]
    fn every_cd_policy_blocks_something_and_never_traps_the_player() {
        use crate::paths::RelPath;
        use std::path::Path;

        let world = World::new();
        let catalog = catalog();
        let strict: Vec<_> = catalog
            .iter()
            .filter(|(_, level)| level.cd.is_some())
            .collect();
        assert!(
            !strict.is_empty(),
            "the course teaches `cd` with strict levels"
        );
        for (section, level) in strict {
            let ctx = world.start(section, level);
            let policy = level.cd.as_ref().unwrap();
            let start = ctx.cwd.clone().unwrap();
            // Where the policy cares about the player: the start and the directories it names.
            let places: Vec<PathBuf> = std::iter::once(start.clone())
                .chain(
                    policy
                        .paths()
                        .into_iter()
                        .filter_map(|path| RelPath::parse(path).ok())
                        .map(|path| path.under(&ctx.section_root)),
                )
                .collect();
            let decide = |place: &Path, target: &str| {
                let args: Vec<String> = if target.is_empty() {
                    Vec::new()
                } else {
                    vec![target.to_owned()]
                };
                level.cd_pre_move(&CdMove::new(&ctx, target, &args, place))
            };

            let attempts = ["/etc", "..", "../..", "/", "", "elsewhere", "-"];
            let blocked: Vec<(&PathBuf, &str)> = places
                .iter()
                .flat_map(|place| attempts.iter().map(move |target| (place, *target)))
                .filter(|(place, target)| decide(place, target).is_block())
                .collect();
            assert!(
                !blocked.is_empty(),
                "{}: the policy blocks nothing",
                level.id
            );

            // The recovery move: back to the start directory is allowed from anywhere.
            let start_text = start.to_string_lossy();
            for place in &places {
                assert_eq!(
                    decide(place, &start_text),
                    CdDecision::Allow,
                    "{}: the move to the start directory is blocked from {place:?}",
                    level.id
                );
            }

            // Once the drill is done, nothing is blocked any more.
            ctx.markers().create(&Marker::Cd).unwrap();
            for (place, target) in blocked {
                assert_eq!(
                    decide(place, target),
                    CdDecision::Allow,
                    "{}: `cd {target}` from {place:?} is still blocked after the drill",
                    level.id
                );
            }
        }
    }

    #[test]
    fn every_solution_ends_by_submitting() {
        let world = World::new();
        let catalog = catalog();
        for (section, level) in catalog.iter().filter(|(_, level)| !level.is_intro()) {
            let ctx = world.start(section, level);
            let solution = level.solution.as_ref().expect("a solution");
            let script = solution.script(&ctx);
            let last = script.lines().last().unwrap_or_default();
            assert!(
                last.contains("submit"),
                "{}: the solution should end with the submission, not {last:?}",
                level.id
            );
        }
    }
}
