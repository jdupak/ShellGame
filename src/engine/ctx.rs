//! Everything a check needs to know about the world, passed explicitly.

use std::path::{Path, PathBuf};

use crate::engine::LevelId;
use crate::markers::Markers;
use crate::paths::{ContainmentError, resolve_within};

/// The facts about the player's situation that requirements, answers and `cd` rules
/// consult.
///
/// The engine never reads the process working directory, `HOME` or any environment
/// variable: the caller captures them once and hands them in here. That keeps every rule a
/// plain function of its inputs, so tests run in parallel without `chdir` or `set_var`.
#[derive(Debug, Clone)]
pub struct Ctx<'a> {
    /// The workspace root (holds `level-N` directories and the marker files).
    pub workspace: &'a Path,
    /// The absolute directory every path of the current level is relative to.
    pub section_root: PathBuf,
    /// The player's current directory, already canonical, or `None` when it no longer
    /// exists (the player deleted the directory they were standing in).
    pub cwd: Option<PathBuf>,
    /// The logical `$HOME`.
    pub home: &'a Path,
    /// The level being checked.
    pub level: LevelId,
}

impl<'a> Ctx<'a> {
    /// Gather the facts. See the field docs for what each one must be.
    pub fn new(
        workspace: &'a Path,
        section_root: PathBuf,
        cwd: Option<PathBuf>,
        home: &'a Path,
        level: LevelId,
    ) -> Self {
        Self {
            workspace,
            section_root,
            cwd,
            home,
            level,
        }
    }

    /// The marker files of the current level.
    pub fn markers(&self) -> Markers<'a> {
        Markers::new(self.workspace, self.level)
    }

    /// The last component of `$HOME`, such as `ada` for `/home/ada` (empty for `/`).
    pub fn home_name(&self) -> String {
        self.home
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    /// Resolve a section-relative path with the full containment rules: no absolute
    /// paths, no `..`, no symlinked component.
    pub fn path(&self, rel: &str) -> Result<PathBuf, ContainmentError> {
        resolve_within(&self.section_root, rel, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markers::Marker;

    fn ctx<'a>(workspace: &'a Path, home: &'a Path) -> Ctx<'a> {
        Ctx::new(
            workspace,
            workspace.join("level-1"),
            None,
            home,
            LevelId::new(1, 4),
        )
    }

    #[test]
    fn home_name_is_the_basename_of_home() {
        let workspace = Path::new("/ws");
        assert_eq!(ctx(workspace, Path::new("/home/ada")).home_name(), "ada");
        assert_eq!(ctx(workspace, Path::new("/home/ada/")).home_name(), "ada");
        assert_eq!(ctx(workspace, Path::new("/")).home_name(), "");
    }

    #[test]
    fn markers_belong_to_the_level_in_the_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let home = Path::new("/home/ada");
        let ctx = ctx(dir.path(), home);
        ctx.markers().create(&Marker::Cd).unwrap();
        assert!(dir.path().join(".cd-1.4").exists());
        assert!(ctx.markers().exists(&Marker::Cd));
    }

    #[test]
    fn path_resolves_inside_the_section_root_only() {
        let dir = tempfile::tempdir().unwrap();
        let home = Path::new("/home/ada");
        let ctx = ctx(dir.path(), home);
        assert_eq!(
            ctx.path("alpha/inside.txt").unwrap(),
            dir.path().join("level-1/alpha/inside.txt")
        );
        assert_eq!(ctx.path("").unwrap(), dir.path().join("level-1"));
        assert!(ctx.path("../level-2").is_err());
        assert!(ctx.path("/etc").is_err());
    }

    #[test]
    fn path_refuses_symlinked_components() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let root = dir.path().join("level-1");
        std::fs::create_dir_all(&root).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("link")).unwrap();
        let home = Path::new("/home/ada");
        let ctx = ctx(dir.path(), home);
        assert!(matches!(
            ctx.path("link/file"),
            Err(ContainmentError::Symlink { .. })
        ));
    }
}
