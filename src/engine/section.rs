//! Sections: a numbered group of levels that share a directory and a base fixture.

use std::path::{Path, PathBuf};

use crate::engine::fixture::Fixture;
use crate::engine::level::Level;
use crate::engine::{Ctx, LevelId};
use crate::paths::{ContainmentError, resolve_within};

/// A section of the course: its number, the directory (relative to the workspace) every path
/// of its levels is relative to, a fixture shared by all of them, and the levels.
///
/// ```
/// use shellgame::engine::prelude::*;
///
/// let section = Section::new(2, "level-2")
///     .fixture(Fixture::new().dir("location-A"))
///     .intro(intro("Práce se soubory", "Úvod."))
///     .levels([level(1, "První").answer(exact("a")).solution("shellgame submit a")]);
/// assert_eq!(section.levels[0].id.to_string(), "2.0");
/// assert_eq!(section.levels[1].id.to_string(), "2.1");
/// ```
#[derive(Debug, Clone)]
pub struct Section {
    /// The section number.
    pub number: u16,
    /// The section directory relative to the workspace: `""` for section 0 (the workspace
    /// itself), `"level-N"` for the others.
    pub root: String,
    /// Applied before each level's own fixture.
    pub fixture: Fixture,
    /// The intro (`N.0`) and the levels, in the order they were added.
    pub levels: Vec<Level>,
}

impl Section {
    /// An empty section. `root` is relative to the workspace.
    pub fn new(number: u16, root: impl Into<String>) -> Self {
        Self {
            number,
            root: root.into(),
            fixture: Fixture::new(),
            levels: Vec::new(),
        }
    }

    /// Add a fixture shared by every level of the section, applied before the level's own.
    #[must_use]
    pub fn fixture(mut self, fixture: Fixture) -> Self {
        self.fixture = self.fixture.merge(fixture);
        self
    }

    /// Add the section intro; it becomes `N.0`.
    #[must_use]
    pub fn intro(mut self, mut intro: Level) -> Self {
        intro.id = LevelId::new(self.number, 0);
        self.levels.push(intro);
        self
    }

    /// Add levels; each gets this section's number.
    #[must_use]
    pub fn levels<I>(mut self, levels: I) -> Self
    where
        I: IntoIterator<Item = Level>,
    {
        for mut level in levels {
            level.id.section = self.number;
            self.levels.push(level);
        }
        self
    }

    /// The level with this number, if the section has it.
    pub fn get(&self, number: u16) -> Option<&Level> {
        self.levels.iter().find(|level| level.id.number == number)
    }

    /// The absolute directory every path of this section's levels is relative to, with the
    /// containment rules applied: the workspace and `level-N` must not be symlinks.
    pub fn section_root(&self, workspace: &Path) -> Result<PathBuf, ContainmentError> {
        resolve_within(workspace, &self.root, false)
    }

    /// The [`Ctx`] for checking one of this section's levels.
    pub fn ctx<'a>(
        &self,
        level: &Level,
        workspace: &'a Path,
        cwd: Option<PathBuf>,
        home: &'a Path,
    ) -> Result<Ctx<'a>, ContainmentError> {
        Ok(Ctx::new(
            workspace,
            self.section_root(workspace)?,
            cwd,
            home,
            level.id,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::level::{intro, level};

    #[test]
    fn levels_get_the_sections_number_and_the_intro_becomes_zero() {
        let section = Section::new(3, "level-3")
            .levels([level(2, "b"), level(1, "a")])
            .intro(intro("Úvod", "text"));
        let ids: Vec<String> = section.levels.iter().map(|l| l.id.to_string()).collect();
        assert_eq!(ids, ["3.2", "3.1", "3.0"]);
        assert!(section.get(0).unwrap().is_intro());
        assert_eq!(section.get(1).unwrap().title, "a");
        assert!(section.get(9).is_none());
    }

    #[test]
    fn the_root_is_resolved_against_the_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let section = Section::new(2, "level-2");
        assert_eq!(
            section.section_root(dir.path()).unwrap(),
            dir.path().join("level-2")
        );
        assert_eq!(
            Section::new(0, "").section_root(dir.path()).unwrap(),
            dir.path()
        );
        assert!(Section::new(2, "../x").section_root(dir.path()).is_err());
        assert!(Section::new(2, "/abs").section_root(dir.path()).is_err());
    }

    #[test]
    fn a_symlinked_section_root_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("level-2")).unwrap();
        assert!(matches!(
            Section::new(2, "level-2").section_root(dir.path()),
            Err(ContainmentError::Symlink { .. })
        ));
    }

    #[test]
    fn fixtures_added_twice_are_merged() {
        let section = Section::new(1, "level-1")
            .fixture(Fixture::new().dir("a"))
            .fixture(Fixture::new().dir("b"));
        assert_eq!(section.fixture.dir_paths(), ["a", "b"]);
    }

    #[test]
    fn ctx_carries_the_levels_section_root() {
        let dir = tempfile::tempdir().unwrap();
        let home = Path::new("/home/ada");
        let section = Section::new(1, "level-1").levels([level(4, "x")]);
        let ctx = section
            .ctx(&section.levels[0], dir.path(), None, home)
            .unwrap();
        assert_eq!(ctx.section_root, dir.path().join("level-1"));
        assert_eq!(ctx.level, LevelId::new(1, 4));
        assert_eq!(ctx.cwd, None);
        assert_eq!(ctx.home, home);
    }
}
