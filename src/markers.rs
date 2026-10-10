//! Evidence markers: dotfiles in the workspace root that prove the player did something.
//!
//! The game cannot observe a shell, so anything it must know about a *command* leaves a
//! trace: the `pwd` wrapper, the `cd` hook and the `fd` hook each write a small file.
//! Markers are evidence, not saved state; they are cleared when a level is prepared
//! (see [`Level::prepare`](crate::engine::level::Level::prepare)) and live directly in the
//! workspace.
//!
//! A marker is named relative to a level, so two levels can never share one by accident:
//! `Marker::Cd` of level 1.4 is `.cd-1.4`, and `Marker::Own("walk")` of level 1.9 is
//! `.1.9-walk`. The one shared marker is `Marker::Pwd`, written by the shell wrapper
//! whenever the player runs `pwd`.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use rustix::fs::OFlags;

use crate::engine::LevelId;

/// A level-relative reference to a marker file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Marker {
    /// `.pwd_used`, written by the shell wrapper when `pwd` runs. Shared by all levels.
    Pwd,
    /// `.cd-<id>`: this level's `cd` drill was performed.
    Cd,
    /// `.fd-<id>`: this level's file-descriptor drill was performed.
    Fd,
    /// `.<id>-<name>`: a private marker of one level, such as the progress of a walk.
    Own(&'static str),
}

impl Marker {
    /// The marker's file name for the given level.
    pub fn file_name(&self, level: LevelId) -> String {
        match self {
            Self::Pwd => ".pwd_used".to_owned(),
            Self::Cd => format!(".cd-{level}"),
            Self::Fd => format!(".fd-{level}"),
            Self::Own(name) => format!(".{level}-{name}"),
        }
    }
}

/// The markers of one level, in one workspace.
#[derive(Debug, Clone, Copy)]
pub struct Markers<'a> {
    workspace: &'a Path,
    level: LevelId,
}

impl<'a> Markers<'a> {
    /// Markers of `level` in `workspace`.
    pub fn new(workspace: &'a Path, level: LevelId) -> Self {
        Self { workspace, level }
    }

    /// The level these markers belong to.
    pub fn level(&self) -> LevelId {
        self.level
    }

    /// Where the marker lives.
    pub fn path(&self, marker: &Marker) -> PathBuf {
        self.workspace.join(marker.file_name(self.level))
    }

    /// True when the marker file exists.
    pub fn exists(&self, marker: &Marker) -> bool {
        self.path(marker).exists()
    }

    /// Create the marker (empty), creating the workspace if it is missing.
    pub fn create(&self, marker: &Marker) -> io::Result<()> {
        self.create_with(marker, "")
    }

    /// Create the marker with the given content, replacing any previous content. A marker
    /// path that is a symlink is refused rather than written through (`O_NOFOLLOW`), so a
    /// link in the workspace can never make the game overwrite another file;
    /// preparing the level removes the link itself.
    pub fn create_with(&self, marker: &Marker, content: &str) -> io::Result<()> {
        fs::create_dir_all(self.workspace)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .custom_flags(OFlags::NOFOLLOW.bits() as i32)
            .open(self.path(marker))?;
        file.write_all(content.as_bytes())
    }

    /// The marker's content, or `None` when it does not exist or cannot be read.
    pub fn read(&self, marker: &Marker) -> Option<String> {
        fs::read_to_string(self.path(marker)).ok()
    }

    /// Delete the marker. A marker that is already gone is not an error.
    pub fn remove(&self, marker: &Marker) -> io::Result<()> {
        match fs::remove_file(self.path(marker)) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEVEL: LevelId = LevelId::new(1, 9);

    #[test]
    fn file_names_are_derived_from_the_level() {
        assert_eq!(Marker::Pwd.file_name(LEVEL), ".pwd_used");
        assert_eq!(Marker::Cd.file_name(LevelId::new(1, 4)), ".cd-1.4");
        assert_eq!(Marker::Fd.file_name(LevelId::new(10, 4)), ".fd-10.4");
        assert_eq!(Marker::Own("walk").file_name(LEVEL), ".1.9-walk");
    }

    #[test]
    fn pwd_is_shared_but_the_others_are_per_level() {
        let a = LevelId::new(1, 4);
        let b = LevelId::new(1, 6);
        assert_eq!(Marker::Pwd.file_name(a), Marker::Pwd.file_name(b));
        assert_ne!(Marker::Cd.file_name(a), Marker::Cd.file_name(b));
        assert_ne!(Marker::Fd.file_name(a), Marker::Cd.file_name(a));
        assert_ne!(Marker::Own("x").file_name(a), Marker::Own("y").file_name(a));
    }

    #[test]
    fn create_exists_read_and_remove() {
        let dir = tempfile::tempdir().unwrap();
        let markers = Markers::new(dir.path(), LEVEL);
        assert!(!markers.exists(&Marker::Cd));
        assert_eq!(markers.read(&Marker::Cd), None);

        markers.create(&Marker::Cd).unwrap();
        assert!(markers.exists(&Marker::Cd));
        assert!(dir.path().join(".cd-1.9").is_file());
        assert_eq!(markers.read(&Marker::Cd).as_deref(), Some(""));

        markers.remove(&Marker::Cd).unwrap();
        assert!(!markers.exists(&Marker::Cd));
        assert!(!dir.path().join(".cd-1.9").exists());
    }

    #[test]
    fn create_with_replaces_content() {
        let dir = tempfile::tempdir().unwrap();
        let markers = Markers::new(dir.path(), LEVEL);
        markers
            .create_with(&Marker::Own("walk"), "/\n/home")
            .unwrap();
        assert_eq!(
            markers.read(&Marker::Own("walk")).as_deref(),
            Some("/\n/home")
        );
        markers.create_with(&Marker::Own("walk"), "/").unwrap();
        assert_eq!(markers.read(&Marker::Own("walk")).as_deref(), Some("/"));
        assert!(dir.path().join(".1.9-walk").is_file());
    }

    #[test]
    fn removing_a_missing_marker_is_fine() {
        let dir = tempfile::tempdir().unwrap();
        let markers = Markers::new(dir.path(), LEVEL);
        markers.remove(&Marker::Fd).unwrap();
        markers.remove(&Marker::Own("never")).unwrap();
    }

    #[test]
    fn create_builds_a_missing_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("not/yet/there");
        let markers = Markers::new(&workspace, LEVEL);
        markers.create(&Marker::Pwd).unwrap();
        assert!(workspace.join(".pwd_used").is_file());
    }

    #[test]
    fn markers_of_different_levels_do_not_collide() {
        let dir = tempfile::tempdir().unwrap();
        let a = Markers::new(dir.path(), LevelId::new(1, 4));
        let b = Markers::new(dir.path(), LevelId::new(1, 6));
        a.create(&Marker::Cd).unwrap();
        assert!(a.exists(&Marker::Cd));
        assert!(!b.exists(&Marker::Cd));
        // `pwd` is the one shared marker.
        a.create(&Marker::Pwd).unwrap();
        assert!(b.exists(&Marker::Pwd));
        assert_eq!(a.level(), LevelId::new(1, 4));
    }

    #[test]
    fn creating_a_marker_never_writes_through_a_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let markers = Markers::new(dir.path(), LEVEL);
        let precious = dir.path().join("precious.txt");
        fs::write(&precious, "keep me").unwrap();
        std::os::unix::fs::symlink(&precious, markers.path(&Marker::Cd)).unwrap();
        assert!(markers.create(&Marker::Cd).is_err());
        assert!(markers.create_with(&Marker::Cd, "overwritten").is_err());
        assert_eq!(fs::read_to_string(&precious).unwrap(), "keep me");

        // Even a dangling link does not get its target created.
        let ghost = dir.path().join("ghost.txt");
        std::os::unix::fs::symlink(&ghost, markers.path(&Marker::Fd)).unwrap();
        assert!(markers.create(&Marker::Fd).is_err());
        assert!(!ghost.exists());

        // Removing the link (what preparing a level does) clears the way.
        markers.remove(&Marker::Cd).unwrap();
        markers.create(&Marker::Cd).unwrap();
        assert_eq!(fs::read_to_string(&precious).unwrap(), "keep me");
        assert!(markers.exists(&Marker::Cd));
    }

    #[test]
    fn a_dangling_symlink_is_not_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let markers = Markers::new(dir.path(), LEVEL);
        std::os::unix::fs::symlink("nowhere", markers.path(&Marker::Cd)).unwrap();
        assert!(!markers.exists(&Marker::Cd));
        // ...but it can still be cleared.
        markers.remove(&Marker::Cd).unwrap();
        assert!(fs::symlink_metadata(markers.path(&Marker::Cd)).is_err());
    }
}
