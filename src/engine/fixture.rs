//! Declarative workspace fixtures: what a level's directory must look like.
//!
//! A [`Fixture`] lists directories, files, modes and paths to clean. [`Fixture::apply`]
//! makes a root match it *whatever state the previous attempt left behind*, because
//! `shellgame reset` is the one command that has to work after a player has done
//! something drastic. The repair rules, ported from the Python version:
//!
//! * **Chmod before delete.** Section 8 teaches `chmod`, so a player can legitimately
//!   leave a directory at `000` or a fixture file read-only. Anything that must be
//!   replaced or removed is made accessible first.
//! * **Never follow a symlink.** A link at a declared path is removed and replaced; what
//!   it pointed to is never read, written, chmod'ed or deleted, and files are created with
//!   `O_EXCL` so not even a race can write through one.
//! * **File/directory mix-ups are replaced.** A directory where a file belongs (or the
//!   reverse) is removed and recreated as declared.
//! * **A fixture cannot target its own root.** Cleaning or replacing the root would
//!   delete the very directory the level lives in.
//! * **Directory modes come last, deepest first**, so restricting a parent never blocks
//!   the chmod of a child.
//!
//! One deliberate change from the Python version: every directory and file the fixture
//! creates or repairs ends with an *explicit* mode, `0o755` for directories and `0o644`
//! for files, unless the fixture declares one. Results therefore no longer depend on the
//! player's umask, and no directory keeps the temporary `0o700` used while repairing.
//! (A kept file, see [`Fixture::keep_file`], is neither created nor repaired, so it keeps
//! whatever mode it has unless one is declared.)

use std::cmp::Reverse;
use std::fmt;
use std::fs::{self, OpenOptions, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::paths::{ContainmentError, PathError, RelPath};

/// Mode of every directory a fixture creates or repairs, unless it declares another.
pub const DEFAULT_DIR_MODE: u32 = 0o755;
/// Mode of every file a fixture writes, unless it declares another.
pub const DEFAULT_FILE_MODE: u32 = 0o644;
/// The largest permission value `chmod` accepts.
const MAX_MODE: u32 = 0o7777;

/// The content of a fixture file: text or raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    /// UTF-8 text.
    Text(String),
    /// Arbitrary bytes, for binary fixtures (images, archives, magic headers).
    Bytes(Vec<u8>),
}

impl Content {
    /// The exact bytes written to disk.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Text(text) => text.as_bytes(),
            Self::Bytes(bytes) => bytes,
        }
    }
}

impl From<&str> for Content {
    fn from(text: &str) -> Self {
        Self::Text(text.to_owned())
    }
}

impl From<String> for Content {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

impl From<&String> for Content {
    fn from(text: &String) -> Self {
        Self::Text(text.clone())
    }
}

impl From<Vec<u8>> for Content {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }
}

impl From<&[u8]> for Content {
    fn from(bytes: &[u8]) -> Self {
        Self::Bytes(bytes.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for Content {
    fn from(bytes: &[u8; N]) -> Self {
        Self::Bytes(bytes.to_vec())
    }
}

/// Why a fixture is invalid or could not be applied.
#[derive(Debug)]
pub enum FixtureError {
    /// A declared path is not a plain relative path.
    InvalidPath(PathError),
    /// The root is a symlink, or a declared path passes through one.
    Containment(ContainmentError),
    /// A declaration names the fixture's own root.
    TargetsRoot {
        /// Which kind of declaration (`"file"`, `"cleanup"`, `"directory mode"`).
        what: &'static str,
        /// The declared path.
        path: String,
    },
    /// A declared mode is outside `0o0000..=0o7777`.
    InvalidMode {
        /// The declared path.
        path: String,
        /// The offending mode.
        mode: u32,
    },
    /// A filesystem operation failed.
    Io {
        /// What was being attempted, such as `"remove"` or `"chmod"`.
        action: &'static str,
        /// The path it was attempted on.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },
}

impl FixtureError {
    fn io(action: &'static str, path: &Path, source: io::Error) -> Self {
        Self::Io {
            action,
            path: path.to_path_buf(),
            source,
        }
    }
}

impl fmt::Display for FixtureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(error) => write!(f, "invalid fixture path: {error}"),
            Self::Containment(error) => write!(f, "fixture path rejected: {error}"),
            Self::TargetsRoot { what, path } => {
                write!(f, "a {what} fixture cannot target its root: {path:?}")
            }
            Self::InvalidMode { path, mode } => {
                write!(f, "mode {mode:#o} for {path:?} is outside 0o0000..=0o7777")
            }
            Self::Io {
                action,
                path,
                source,
            } => write!(f, "cannot {action} {}: {source}", path.display()),
        }
    }
}

impl std::error::Error for FixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPath(error) => Some(error),
            Self::Containment(error) => Some(error),
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<PathError> for FixtureError {
    fn from(error: PathError) -> Self {
        Self::InvalidPath(error)
    }
}

impl From<ContainmentError> for FixtureError {
    fn from(error: ContainmentError) -> Self {
        Self::Containment(error)
    }
}

#[derive(Debug, Clone)]
struct FileSpec {
    path: String,
    content: Content,
    mode: Option<u32>,
    overwrite: bool,
}

#[derive(Debug, Clone)]
struct DirModeSpec {
    path: String,
    mode: u32,
}

/// A declaration of what a directory must contain, built with chained calls.
///
/// ```
/// use shellgame::engine::fixture::Fixture;
///
/// let fixture = Fixture::new()
///     .dirs(["alpha", "gamma/deep/a/b/c"])
///     .file("alpha/inside.txt", "first step")
///     .file_mode("run.sh", "#!/bin/sh\n", 0o755)
///     .dir_mode("locked", 0o550)
///     .clean(["scratch"]);
/// assert!(!fixture.is_empty());
/// ```
#[derive(Debug, Clone, Default)]
pub struct Fixture {
    dirs: Vec<String>,
    files: Vec<FileSpec>,
    clean: Vec<String>,
    dir_modes: Vec<DirModeSpec>,
}

impl Fixture {
    /// An empty fixture.
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare a directory (and its parents). Its mode becomes `0o755`.
    #[must_use]
    pub fn dir(mut self, path: impl Into<String>) -> Self {
        self.dirs.push(path.into());
        self
    }

    /// Declare several directories.
    #[must_use]
    pub fn dirs<I, S>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.dirs.extend(paths.into_iter().map(Into::into));
        self
    }

    /// Declare a file that is rewritten on every apply. Its mode becomes `0o644`.
    #[must_use]
    pub fn file(self, path: impl Into<String>, content: impl Into<Content>) -> Self {
        self.push_file(path.into(), content.into(), None, true)
    }

    /// Declare a file with an explicit mode (`0..=0o7777`).
    #[must_use]
    pub fn file_mode(
        self,
        path: impl Into<String>,
        content: impl Into<Content>,
        mode: u32,
    ) -> Self {
        self.push_file(path.into(), content.into(), Some(mode), true)
    }

    /// Declare a file that is only written when missing: a regular file already there is
    /// kept, content, mode and all (unless a mode is declared). Section fixtures use this
    /// for files the player is expected to have edited.
    #[must_use]
    pub fn keep_file(self, path: impl Into<String>, content: impl Into<Content>) -> Self {
        self.push_file(path.into(), content.into(), None, false)
    }

    fn push_file(
        mut self,
        path: String,
        content: Content,
        mode: Option<u32>,
        overwrite: bool,
    ) -> Self {
        self.files.push(FileSpec {
            path,
            content,
            mode,
            overwrite,
        });
        self
    }

    /// Declare a directory with an explicit mode (`0..=0o7777`). Modes are applied after
    /// everything else, deepest directory first.
    #[must_use]
    pub fn dir_mode(mut self, path: impl Into<String>, mode: u32) -> Self {
        self.dir_modes.push(DirModeSpec {
            path: path.into(),
            mode,
        });
        self
    }

    /// Remove these paths (files, directory trees or symlinks) before anything is created.
    #[must_use]
    pub fn clean<I, S>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.clean.extend(paths.into_iter().map(Into::into));
        self
    }

    /// Add everything `other` declares after what this fixture declares.
    #[must_use]
    pub fn merge(mut self, other: Fixture) -> Self {
        self.dirs.extend(other.dirs);
        self.files.extend(other.files);
        self.clean.extend(other.clean);
        self.dir_modes.extend(other.dir_modes);
        self
    }

    /// True when nothing is declared.
    pub fn is_empty(&self) -> bool {
        self.dirs.is_empty()
            && self.files.is_empty()
            && self.clean.is_empty()
            && self.dir_modes.is_empty()
    }

    /// The declared directories, as written.
    pub fn dir_paths(&self) -> &[String] {
        &self.dirs
    }

    /// The declared files with their content, as written.
    pub fn file_entries(&self) -> impl Iterator<Item = (&str, &Content)> {
        self.files
            .iter()
            .map(|file| (file.path.as_str(), &file.content))
    }

    /// The declared cleanup paths, as written.
    pub fn clean_paths(&self) -> &[String] {
        &self.clean
    }

    /// The declared directory modes, as written.
    pub fn dir_mode_entries(&self) -> impl Iterator<Item = (&str, u32)> {
        self.dir_modes
            .iter()
            .map(|entry| (entry.path.as_str(), entry.mode))
    }

    /// Every path this fixture declares, as written.
    pub fn declared_paths(&self) -> impl Iterator<Item = &str> {
        self.dirs
            .iter()
            .map(String::as_str)
            .chain(self.files.iter().map(|file| file.path.as_str()))
            .chain(self.clean.iter().map(String::as_str))
            .chain(self.dir_modes.iter().map(|entry| entry.path.as_str()))
    }

    /// Everything wrong with the declarations themselves (bad paths, modes out of range,
    /// declarations that target the root), without touching the filesystem.
    pub fn problems(&self) -> Vec<FixtureError> {
        let mut found = Vec::new();
        self.plan(&mut found);
        found
    }

    /// Make `root` match this fixture; see the [module documentation](self) for the repair
    /// rules and the explicit-mode guarantee.
    ///
    /// The root is created when missing and repaired when its owner permissions were taken
    /// away; either way it ends `0o755`. A root that is a symlink is refused. Nothing is
    /// applied when the declarations are invalid.
    pub fn apply(&self, root: &Path) -> Result<(), FixtureError> {
        let mut found = Vec::new();
        let plan = self.plan(&mut found);
        if let Some(first) = found.into_iter().next() {
            return Err(first);
        }

        ensure_root(root)?;
        for path in &plan.clean {
            clean_path(root, path)?;
        }
        for path in &plan.dirs {
            ensure_dir(root, path)?;
        }
        for (path, _) in &plan.dir_modes {
            ensure_dir(root, path)?;
        }
        for (spec, path) in &plan.files {
            write_file(root, spec, path)?;
        }

        // Restricting a directory can lock out chmod of everything beneath it, so the
        // deepest ones go first. The sort is stable: equal depths keep declaration order.
        let mut ordered: Vec<&(RelPath, u32)> = plan.dir_modes.iter().collect();
        ordered.sort_by_key(|(path, _)| Reverse(path.depth()));
        for (path, mode) in ordered {
            apply_dir_mode(root, path, *mode)?;
        }
        Ok(())
    }

    /// Parse and validate every declaration, pushing problems into `found`.
    fn plan(&self, found: &mut Vec<FixtureError>) -> Plan<'_> {
        let mut plan = Plan::default();
        for path in &self.dirs {
            match RelPath::parse(path) {
                Ok(parsed) => plan.dirs.push(parsed),
                Err(error) => found.push(error.into()),
            }
        }
        for spec in &self.files {
            match RelPath::parse(&spec.path) {
                Ok(parsed) if parsed.is_root() => found.push(FixtureError::TargetsRoot {
                    what: "file",
                    path: spec.path.clone(),
                }),
                Ok(parsed) => {
                    check_mode(&spec.path, spec.mode, found);
                    plan.files.push((spec, parsed));
                }
                Err(error) => found.push(error.into()),
            }
        }
        for path in &self.clean {
            match RelPath::parse(path) {
                Ok(parsed) if parsed.is_root() => found.push(FixtureError::TargetsRoot {
                    what: "cleanup",
                    path: path.clone(),
                }),
                Ok(parsed) => plan.clean.push(parsed),
                Err(error) => found.push(error.into()),
            }
        }
        for spec in &self.dir_modes {
            match RelPath::parse(&spec.path) {
                Ok(parsed) if parsed.is_root() => found.push(FixtureError::TargetsRoot {
                    what: "directory mode",
                    path: spec.path.clone(),
                }),
                Ok(parsed) => {
                    check_mode(&spec.path, Some(spec.mode), found);
                    plan.dir_modes.push((parsed, spec.mode));
                }
                Err(error) => found.push(error.into()),
            }
        }
        plan
    }
}

fn check_mode(path: &str, mode: Option<u32>, found: &mut Vec<FixtureError>) {
    if let Some(mode) = mode
        && mode > MAX_MODE
    {
        found.push(FixtureError::InvalidMode {
            path: path.to_owned(),
            mode,
        });
    }
}

/// A validated fixture: every path parsed, nothing left to reject while applying.
#[derive(Default)]
struct Plan<'f> {
    clean: Vec<RelPath>,
    dirs: Vec<RelPath>,
    dir_modes: Vec<(RelPath, u32)>,
    files: Vec<(&'f FileSpec, RelPath)>,
}

/// Prepare the *section root* the way the Python `Level.prepare` does before applying any
/// fixture: a non-directory in its place (a file the player created) is made removable
/// and deleted, and a directory without owner permissions is made usable. A symlink is
/// left for [`Fixture::apply`] to refuse. Nothing is created here.
pub(crate) fn repair_section_root(root: &Path) -> Result<(), FixtureError> {
    let meta = match fs::symlink_metadata(root) {
        Ok(meta) => meta,
        Err(error) if is_absent(&error) => return Ok(()),
        Err(error) => return Err(FixtureError::io("inspect", root, error)),
    };
    let kind = meta.file_type();
    if kind.is_dir() {
        make_usable(root, &meta);
    } else if !kind.is_symlink() {
        remove_entry(root, &meta)?;
    }
    Ok(())
}

/// True for "there is nothing at this path": the path or one of its parents is missing,
/// or a parent is not a directory.
fn is_absent(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
    )
}

fn set_mode(path: &Path, mode: u32) -> Result<(), FixtureError> {
    fs::set_permissions(path, Permissions::from_mode(mode))
        .map_err(|error| FixtureError::io("chmod", path, error))
}

/// Give a directory owner `rwx` if it lacks it, ignoring failure: the operation that
/// needed the access reports the real error.
fn make_usable(path: &Path, meta: &fs::Metadata) {
    if meta.mode() & 0o700 != 0o700 {
        let _ = fs::set_permissions(path, Permissions::from_mode(DEFAULT_DIR_MODE));
    }
}

fn ensure_root(root: &Path) -> Result<(), FixtureError> {
    match fs::symlink_metadata(root) {
        Ok(meta) if meta.file_type().is_symlink() => {
            Err(ContainmentError::RootIsSymlink(root.to_path_buf()).into())
        }
        Ok(meta) if meta.is_dir() => {
            if meta.mode() & 0o700 != 0o700 {
                set_mode(root, DEFAULT_DIR_MODE)?;
            }
            Ok(())
        }
        Ok(_) => Err(FixtureError::io(
            "use as a directory",
            root,
            io::Error::from(io::ErrorKind::NotADirectory),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all(root).map_err(|error| FixtureError::io("create", root, error))?;
            set_mode(root, DEFAULT_DIR_MODE)
        }
        Err(error) => Err(FixtureError::io("inspect", root, error)),
    }
}

/// Remove whatever is at `rel`: a file, a directory tree, or a symlink (never its
/// target). A symlink anywhere along the way is removed in place of what lies beyond it,
/// and directories along the way are made usable, so a parent the player locked cannot
/// block the cleanup.
fn clean_path(root: &Path, rel: &RelPath) -> Result<(), FixtureError> {
    let count = rel.depth();
    let mut current = root.to_path_buf();
    for (index, part) in rel.components().enumerate() {
        current.push(part);
        let meta = match fs::symlink_metadata(&current) {
            Ok(meta) => meta,
            Err(error) if is_absent(&error) => return Ok(()),
            Err(error) => return Err(FixtureError::io("inspect", &current, error)),
        };
        let kind = meta.file_type();
        if kind.is_symlink() || index + 1 == count {
            return if kind.is_dir() {
                remove_tree(&current)
            } else {
                remove_entry(&current, &meta)
            };
        }
        if !kind.is_dir() {
            // A file in the way: nothing can exist below it.
            return Ok(());
        }
        make_usable(&current, &meta);
    }
    Ok(())
}

/// Make `rel` a real directory, component by component, repairing what is in the way:
/// a symlink or file is removed and replaced, an existing directory is reset to `0o755`.
fn ensure_dir(root: &Path, rel: &RelPath) -> Result<(), FixtureError> {
    let mut current = root.to_path_buf();
    for part in rel.components() {
        current.push(part);
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.is_dir() => {
                if meta.mode() & MAX_MODE != DEFAULT_DIR_MODE {
                    let _ = fs::set_permissions(&current, Permissions::from_mode(DEFAULT_DIR_MODE));
                }
            }
            Ok(meta) => {
                remove_entry(&current, &meta)?;
                create_dir(&current)?;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => create_dir(&current)?,
            Err(error) => return Err(FixtureError::io("inspect", &current, error)),
        }
    }
    Ok(())
}

fn create_dir(path: &Path) -> Result<(), FixtureError> {
    fs::create_dir(path).map_err(|error| FixtureError::io("create directory", path, error))?;
    set_mode(path, DEFAULT_DIR_MODE)
}

fn write_file(root: &Path, spec: &FileSpec, rel: &RelPath) -> Result<(), FixtureError> {
    if let Some(parent) = rel.parent() {
        ensure_dir(root, &parent)?;
    }
    let path = rel.under(root);
    match fs::symlink_metadata(&path) {
        Ok(meta) if meta.is_file() && !spec.overwrite => {
            if let Some(mode) = spec.mode {
                set_mode(&path, mode)?;
            }
            return Ok(());
        }
        Ok(meta) if meta.is_dir() => remove_tree(&path)?,
        Ok(meta) => remove_entry(&path, &meta)?,
        Err(error) if is_absent(&error) => {}
        Err(error) => return Err(FixtureError::io("inspect", &path, error)),
    }
    create_file(
        &path,
        spec.content.as_bytes(),
        spec.mode.unwrap_or(DEFAULT_FILE_MODE),
    )
}

/// Create a new file without ever writing through a symlink (`O_EXCL`), restrictive at
/// first and chmod'ed to its final mode once the content is in.
fn create_file(path: &Path, bytes: &[u8], mode: u32) -> Result<(), FixtureError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| FixtureError::io("create file", path, error))?;
    file.write_all(bytes)
        .map_err(|error| FixtureError::io("write", path, error))?;
    file.set_permissions(Permissions::from_mode(mode))
        .map_err(|error| FixtureError::io("chmod", path, error))
}

fn apply_dir_mode(root: &Path, rel: &RelPath, mode: u32) -> Result<(), FixtureError> {
    let path = rel.under(root);
    let meta =
        fs::symlink_metadata(&path).map_err(|error| FixtureError::io("inspect", &path, error))?;
    if !meta.is_dir() {
        return Err(FixtureError::io(
            "set the mode of",
            &path,
            io::Error::from(io::ErrorKind::NotADirectory),
        ));
    }
    set_mode(&path, mode)
}

/// Remove a file, symlink or other non-directory, after making a regular file writable.
/// A symlink is removed itself, never chmod'ed (chmod would follow it).
fn remove_entry(path: &Path, meta: &fs::Metadata) -> Result<(), FixtureError> {
    if !meta.file_type().is_symlink() {
        let _ = fs::set_permissions(path, Permissions::from_mode(0o600));
    }
    match fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            Err(FixtureError::io("remove", path, error))
        }
        _ => Ok(()),
    }
}

/// Recursively remove a directory tree (or a single entry), recovering from restricted
/// permissions and never following a symlink. Used by fixtures and by `shellgame remove`,
/// which must work after the permission levels left directories locked. Missing is fine.
///
/// # Errors
///
/// [`FixtureError`] when an entry cannot be inspected, listed or removed.
pub fn remove_tree(path: &Path) -> Result<(), FixtureError> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if is_absent(&error) => return Ok(()),
        Err(error) => return Err(FixtureError::io("inspect", path, error)),
    };
    if !meta.is_dir() {
        return remove_entry(path, &meta);
    }
    let _ = fs::set_permissions(path, Permissions::from_mode(0o700));
    let entries = fs::read_dir(path).map_err(|error| FixtureError::io("list", path, error))?;
    for entry in entries {
        let entry = entry.map_err(|error| FixtureError::io("list", path, error))?;
        remove_tree(&entry.path())?;
    }
    fs::remove_dir(path).map_err(|error| FixtureError::io("remove directory", path, error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    fn mode_of(path: &Path) -> u32 {
        fs::symlink_metadata(path).unwrap().mode() & 0o7777
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    fn sample() -> Fixture {
        Fixture::new()
            .dirs(["alpha", "gamma/deep/a/b/c"])
            .file("alpha/inside.txt", "first step")
            .file("top.txt", "top")
            .file("gamma/deep/note.txt", "note\n")
    }

    #[test]
    fn builds_the_declared_tree_with_explicit_modes() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("level-1");
        sample().apply(&root).unwrap();

        assert_eq!(read(&root.join("alpha/inside.txt")), "first step");
        assert_eq!(read(&root.join("top.txt")), "top");
        assert!(root.join("gamma/deep/a/b/c").is_dir());
        for directory in [
            "",
            "alpha",
            "gamma",
            "gamma/deep",
            "gamma/deep/a",
            "gamma/deep/a/b",
            "gamma/deep/a/b/c",
        ] {
            assert_eq!(mode_of(&root.join(directory)), 0o755, "dir {directory:?}");
        }
        for file in ["alpha/inside.txt", "top.txt", "gamma/deep/note.txt"] {
            assert_eq!(mode_of(&root.join(file)), 0o644, "file {file}");
        }
    }

    #[test]
    fn apply_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        sample().apply(dir.path()).unwrap();
        sample().apply(dir.path()).unwrap();
        assert_eq!(read(&dir.path().join("alpha/inside.txt")), "first step");
        assert_eq!(mode_of(&dir.path().join("alpha")), 0o755);
    }

    #[test]
    fn declared_modes_are_applied_exactly() {
        let dir = tempfile::tempdir().unwrap();
        Fixture::new()
            .file_mode("bin/run.sh", "#!/bin/sh\n", 0o755)
            .file_mode("secret.txt", "x", 0o600)
            .file_mode("odd.txt", "x", 0o654)
            .file_mode("none.txt", "x", 0o000)
            .file_mode("sticky.txt", "x", 0o7777)
            .dir_mode("locked", 0o550)
            .dir_mode("open", 0o777)
            .apply(dir.path())
            .unwrap();
        assert_eq!(mode_of(&dir.path().join("bin/run.sh")), 0o755);
        assert_eq!(mode_of(&dir.path().join("bin")), 0o755);
        assert_eq!(mode_of(&dir.path().join("secret.txt")), 0o600);
        assert_eq!(mode_of(&dir.path().join("odd.txt")), 0o654);
        assert_eq!(mode_of(&dir.path().join("none.txt")), 0o000);
        assert_eq!(mode_of(&dir.path().join("sticky.txt")), 0o7777);
        assert_eq!(mode_of(&dir.path().join("locked")), 0o550);
        assert_eq!(mode_of(&dir.path().join("open")), 0o777);
    }

    #[test]
    fn declared_modes_are_restored_on_reapply() {
        let dir = tempfile::tempdir().unwrap();
        let fixture = Fixture::new()
            .file_mode("run.sh", "x", 0o754)
            .dir_mode("d", 0o750);
        fixture.apply(dir.path()).unwrap();
        // The player fiddles with them.
        fs::set_permissions(dir.path().join("run.sh"), Permissions::from_mode(0o666)).unwrap();
        fs::set_permissions(dir.path().join("d"), Permissions::from_mode(0o777)).unwrap();
        fixture.apply(dir.path()).unwrap();
        assert_eq!(mode_of(&dir.path().join("run.sh")), 0o754);
        assert_eq!(mode_of(&dir.path().join("d")), 0o750);
    }

    #[test]
    fn leftover_restrictive_directories_are_reset_to_755() {
        let dir = tempfile::tempdir().unwrap();
        // The 0o700 a repair used to leave behind, and a player's chmod 000.
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        fs::set_permissions(dir.path().join("a/b"), Permissions::from_mode(0o700)).unwrap();
        fs::set_permissions(dir.path().join("a"), Permissions::from_mode(0o000)).unwrap();
        Fixture::new()
            .file("a/b/f.txt", "x")
            .apply(dir.path())
            .unwrap();
        assert_eq!(mode_of(&dir.path().join("a")), 0o755);
        assert_eq!(mode_of(&dir.path().join("a/b")), 0o755);
        assert_eq!(read(&dir.path().join("a/b/f.txt")), "x");
    }

    #[test]
    fn repairs_a_chmod_000_directory_tree_before_cleaning_it() {
        let dir = tempfile::tempdir().unwrap();
        let fixture = Fixture::new()
            .clean(["challenge"])
            .file("challenge/inner/data.txt", "fresh")
            .dir("challenge/inner/deep");
        fixture.apply(dir.path()).unwrap();
        fs::write(dir.path().join("challenge/extra.txt"), "player").unwrap();
        fs::set_permissions(
            dir.path().join("challenge/inner/deep"),
            Permissions::from_mode(0o000),
        )
        .unwrap();
        fs::set_permissions(
            dir.path().join("challenge/inner"),
            Permissions::from_mode(0o000),
        )
        .unwrap();
        fs::set_permissions(dir.path().join("challenge"), Permissions::from_mode(0o000)).unwrap();

        fixture.apply(dir.path()).unwrap();

        assert!(!dir.path().join("challenge/extra.txt").exists());
        assert_eq!(read(&dir.path().join("challenge/inner/data.txt")), "fresh");
        assert_eq!(mode_of(&dir.path().join("challenge")), 0o755);
        assert_eq!(mode_of(&dir.path().join("challenge/inner")), 0o755);
        assert_eq!(mode_of(&dir.path().join("challenge/inner/deep")), 0o755);
    }

    #[test]
    fn repairs_without_clean_when_only_a_parent_was_locked() {
        let dir = tempfile::tempdir().unwrap();
        let fixture = Fixture::new().file("a/b/c.txt", "orig");
        fixture.apply(dir.path()).unwrap();
        fs::set_permissions(dir.path().join("a/b"), Permissions::from_mode(0o000)).unwrap();
        fs::set_permissions(dir.path().join("a"), Permissions::from_mode(0o000)).unwrap();
        fixture.apply(dir.path()).unwrap();
        assert_eq!(read(&dir.path().join("a/b/c.txt")), "orig");
    }

    #[test]
    fn replaces_a_read_only_file() {
        let dir = tempfile::tempdir().unwrap();
        let fixture = Fixture::new().file("notes.txt", "original");
        fixture.apply(dir.path()).unwrap();
        let path = dir.path().join("notes.txt");
        fs::write(&path, "player edit").unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o444)).unwrap();
        fixture.apply(dir.path()).unwrap();
        assert_eq!(read(&path), "original");
        assert_eq!(mode_of(&path), 0o644);
    }

    #[test]
    fn replaces_a_file_with_a_mode_000_by_fresh_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("notes.txt");
        fs::write(&path, "stale").unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o000)).unwrap();
        Fixture::new()
            .file("notes.txt", "fresh")
            .apply(dir.path())
            .unwrap();
        assert_eq!(read(&path), "fresh");
        assert_eq!(mode_of(&path), 0o644);
    }

    #[test]
    fn replaces_a_directory_where_a_file_belongs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config");
        fs::create_dir_all(path.join("sub/locked")).unwrap();
        fs::write(path.join("sub/locked/x"), "x").unwrap();
        fs::set_permissions(path.join("sub/locked"), Permissions::from_mode(0o000)).unwrap();
        Fixture::new()
            .file("config", "now a file")
            .apply(dir.path())
            .unwrap();
        assert!(path.is_file());
        assert_eq!(read(&path), "now a file");
    }

    #[test]
    fn replaces_a_file_where_a_directory_belongs() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("data"), "i am a file").unwrap();
        fs::set_permissions(dir.path().join("data"), Permissions::from_mode(0o000)).unwrap();
        Fixture::new()
            .dir("data/sub")
            .file("data/sub/x.txt", "x")
            .apply(dir.path())
            .unwrap();
        assert!(dir.path().join("data/sub").is_dir());
        assert_eq!(read(&dir.path().join("data/sub/x.txt")), "x");
    }

    #[test]
    fn replaces_a_file_that_is_in_the_middle_of_a_file_path() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a"), "blocker").unwrap();
        Fixture::new()
            .file("a/b/c.txt", "deep")
            .apply(dir.path())
            .unwrap();
        assert_eq!(read(&dir.path().join("a/b/c.txt")), "deep");
    }

    #[test]
    fn replaces_a_symlink_at_a_file_path_without_touching_its_target() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("precious.txt");
        fs::write(&target, "precious").unwrap();
        fs::set_permissions(&target, Permissions::from_mode(0o640)).unwrap();
        symlink(&target, dir.path().join("notes.txt")).unwrap();

        Fixture::new()
            .file("notes.txt", "fixture text")
            .apply(dir.path())
            .unwrap();

        let replaced = dir.path().join("notes.txt");
        assert!(
            !fs::symlink_metadata(&replaced)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(read(&replaced), "fixture text");
        assert_eq!(read(&target), "precious");
        assert_eq!(mode_of(&target), 0o640);
    }

    #[test]
    fn replaces_a_dangling_symlink_at_a_file_path() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("not-created-yet.txt");
        symlink(&target, dir.path().join("notes.txt")).unwrap();
        Fixture::new()
            .file("notes.txt", "x")
            .apply(dir.path())
            .unwrap();
        assert!(!target.exists(), "must not write through the link");
        assert_eq!(read(&dir.path().join("notes.txt")), "x");
    }

    #[test]
    fn replaces_a_symlink_at_a_directory_path_without_touching_its_target() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("keep.txt"), "keep").unwrap();
        fs::set_permissions(outside.path(), Permissions::from_mode(0o750)).unwrap();
        symlink(outside.path(), dir.path().join("alpha")).unwrap();

        Fixture::new()
            .dir("alpha")
            .file("alpha/inside.txt", "x")
            .apply(dir.path())
            .unwrap();

        assert!(dir.path().join("alpha").is_dir());
        assert!(
            !fs::symlink_metadata(dir.path().join("alpha"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(read(&dir.path().join("alpha/inside.txt")), "x");
        assert_eq!(read(&outside.path().join("keep.txt")), "keep");
        assert!(!outside.path().join("inside.txt").exists());
        assert_eq!(mode_of(outside.path()), 0o750, "target mode untouched");
    }

    #[test]
    fn replaces_a_symlinked_parent_without_writing_through_it() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), dir.path().join("link")).unwrap();
        Fixture::new()
            .file("link/deep/file.txt", "x")
            .dir_mode("link/deep", 0o700)
            .apply(dir.path())
            .unwrap();
        assert!(fs::read_dir(outside.path()).unwrap().next().is_none());
        assert_eq!(read(&dir.path().join("link/deep/file.txt")), "x");
    }

    #[test]
    fn a_directory_mode_on_a_symlink_replaces_it_and_spares_the_target() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::set_permissions(outside.path(), Permissions::from_mode(0o755)).unwrap();
        symlink(outside.path(), dir.path().join("locked")).unwrap();
        Fixture::new()
            .dir_mode("locked", 0o500)
            .apply(dir.path())
            .unwrap();
        assert_eq!(mode_of(&dir.path().join("locked")), 0o500);
        assert_eq!(mode_of(outside.path()), 0o755);
    }

    #[test]
    fn clean_removes_files_directories_and_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("keep.txt"), "keep").unwrap();
        fs::write(dir.path().join("file.txt"), "x").unwrap();
        fs::set_permissions(dir.path().join("file.txt"), Permissions::from_mode(0o000)).unwrap();
        fs::create_dir_all(dir.path().join("tree/inner")).unwrap();
        fs::write(dir.path().join("tree/inner/f"), "x").unwrap();
        fs::set_permissions(dir.path().join("tree/inner"), Permissions::from_mode(0o000)).unwrap();
        symlink(outside.path(), dir.path().join("linked-dir")).unwrap();
        symlink(
            outside.path().join("keep.txt"),
            dir.path().join("linked-file"),
        )
        .unwrap();
        symlink("nowhere", dir.path().join("dangling")).unwrap();

        Fixture::new()
            .clean([
                "file.txt",
                "tree",
                "linked-dir",
                "linked-file",
                "dangling",
                "never-existed",
            ])
            .apply(dir.path())
            .unwrap();

        for gone in ["file.txt", "tree", "linked-dir", "linked-file", "dangling"] {
            assert!(
                fs::symlink_metadata(dir.path().join(gone)).is_err(),
                "{gone} should be gone"
            );
        }
        assert_eq!(read(&outside.path().join("keep.txt")), "keep");
        assert!(outside.path().exists());
    }

    #[test]
    fn clean_never_follows_a_symlinked_parent() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::create_dir(outside.path().join("victim")).unwrap();
        fs::write(outside.path().join("victim/data.txt"), "keep").unwrap();
        symlink(outside.path(), dir.path().join("link")).unwrap();

        Fixture::new()
            .clean(["link/victim"])
            .apply(dir.path())
            .unwrap();

        assert_eq!(read(&outside.path().join("victim/data.txt")), "keep");
    }

    #[test]
    fn clean_repairs_a_locked_parent_of_the_target() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        fs::write(dir.path().join("a/b/target.txt"), "x").unwrap();
        fs::set_permissions(dir.path().join("a/b"), Permissions::from_mode(0o000)).unwrap();
        fs::set_permissions(dir.path().join("a"), Permissions::from_mode(0o000)).unwrap();
        Fixture::new()
            .clean(["a/b/target.txt"])
            .apply(dir.path())
            .unwrap();
        // Access was restored to the parents (to 0o755), and the target is gone.
        assert_eq!(mode_of(&dir.path().join("a")), 0o755);
        assert_eq!(mode_of(&dir.path().join("a/b")), 0o755);
        assert!(!dir.path().join("a/b/target.txt").exists());
    }

    #[test]
    fn clean_of_a_path_below_a_file_or_below_nothing_is_a_no_op() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("file"), "x").unwrap();
        Fixture::new()
            .clean(["file/below", "missing/below/deeper"])
            .apply(dir.path())
            .unwrap();
        assert_eq!(read(&dir.path().join("file")), "x");
    }

    #[test]
    fn clean_runs_before_anything_is_created() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("maze/old")).unwrap();
        fs::write(dir.path().join("maze/old/stale"), "x").unwrap();
        Fixture::new()
            .clean(["maze"])
            .dir("maze/entry")
            .file("maze/entry/GO_UP_1", "")
            .apply(dir.path())
            .unwrap();
        assert!(!dir.path().join("maze/old").exists());
        assert!(dir.path().join("maze/entry/GO_UP_1").is_file());
    }

    #[test]
    fn keep_file_preserves_a_regular_file() {
        let dir = tempfile::tempdir().unwrap();
        let fixture = Fixture::new().keep_file("alpha/inside.txt", "first step");
        fixture.apply(dir.path()).unwrap();
        assert_eq!(read(&dir.path().join("alpha/inside.txt")), "first step");

        let path = dir.path().join("alpha/inside.txt");
        fs::write(&path, "edited by the player").unwrap();
        fs::set_permissions(&path, Permissions::from_mode(0o600)).unwrap();
        fixture.apply(dir.path()).unwrap();
        assert_eq!(read(&path), "edited by the player");
        assert_eq!(mode_of(&path), 0o600, "a kept file is left alone");
    }

    #[test]
    fn keep_file_still_applies_a_declared_mode_and_repairs_mix_ups() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("t.txt");
        fs::write(&target, "t").unwrap();

        // A symlink is not a regular file: it is replaced.
        symlink(&target, dir.path().join("link.txt")).unwrap();
        // A directory is not a regular file either.
        fs::create_dir(dir.path().join("dir.txt")).unwrap();
        // A real file is kept (content) but gets the declared mode.
        fs::write(dir.path().join("real.txt"), "mine").unwrap();

        Fixture::new()
            .keep_file("link.txt", "fresh link")
            .keep_file("dir.txt", "fresh dir")
            .keep_file("missing.txt", "fresh missing")
            .apply(dir.path())
            .unwrap();
        assert_eq!(read(&dir.path().join("link.txt")), "fresh link");
        assert_eq!(read(&target), "t");
        assert_eq!(read(&dir.path().join("dir.txt")), "fresh dir");
        assert_eq!(read(&dir.path().join("missing.txt")), "fresh missing");
        assert_eq!(read(&dir.path().join("real.txt")), "mine");
    }

    #[test]
    fn dir_modes_apply_deepest_first() {
        let dir = tempfile::tempdir().unwrap();
        // Declared shallow-first: applying in that order would lock the child out
        // (a parent without `x` cannot be traversed to chmod the child).
        let fixture = Fixture::new()
            .dir_mode("a", 0o600)
            .dir_mode("a/b", 0o500)
            .dir_mode("a/b/c", 0o555)
            .file("a/b/c/f.txt", "x");
        fixture.apply(dir.path()).unwrap();
        // Apply again over the restricted tree: the reset path.
        fixture.apply(dir.path()).unwrap();

        assert_eq!(mode_of(&dir.path().join("a")), 0o600);
        // Open `a` just long enough to inspect its children, then restore.
        fs::set_permissions(dir.path().join("a"), Permissions::from_mode(0o700)).unwrap();
        assert_eq!(mode_of(&dir.path().join("a/b")), 0o500);
        assert_eq!(mode_of(&dir.path().join("a/b/c")), 0o555);
        assert_eq!(read(&dir.path().join("a/b/c/f.txt")), "x");

        // Leave nothing the temporary directory could not delete.
        for restricted in ["a/b/c", "a/b", "a"] {
            fs::set_permissions(dir.path().join(restricted), Permissions::from_mode(0o755))
                .unwrap();
        }
    }

    #[test]
    fn dir_mode_leaves_other_directories_at_755() {
        let dir = tempfile::tempdir().unwrap();
        Fixture::new()
            .dir_mode("directory-write/locked", 0o550)
            .file("directory-write/readme.txt", "x")
            .apply(dir.path())
            .unwrap();
        assert_eq!(mode_of(&dir.path().join("directory-write")), 0o755);
        assert_eq!(mode_of(&dir.path().join("directory-write/locked")), 0o550);
    }

    #[test]
    fn a_locked_directory_tree_can_be_cleaned_and_rebuilt() {
        // Section 8's reset: clean the tree that contains a 0o550 directory, rebuild it.
        let dir = tempfile::tempdir().unwrap();
        let fixture = Fixture::new()
            .clean(["directory-write"])
            .dir_mode("directory-write/locked", 0o550)
            .file("directory-write/readme.txt", "x");
        fixture.apply(dir.path()).unwrap();
        fixture.apply(dir.path()).unwrap();
        assert_eq!(mode_of(&dir.path().join("directory-write/locked")), 0o550);
    }

    #[test]
    fn a_fixture_cannot_target_its_own_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::set_permissions(dir.path(), Permissions::from_mode(0o700)).unwrap();
        for (fixture, what) in [
            (Fixture::new().file("", "x"), "file"),
            (Fixture::new().file(".", "x"), "file"),
            (Fixture::new().file("./", "x"), "file"),
            (Fixture::new().clean([""]), "cleanup"),
            (Fixture::new().clean(["."]), "cleanup"),
            (Fixture::new().clean(["./"]), "cleanup"),
            (Fixture::new().dir_mode("", 0o700), "directory mode"),
            (Fixture::new().dir_mode(".", 0o700), "directory mode"),
        ] {
            match fixture.apply(dir.path()) {
                Err(FixtureError::TargetsRoot { what: found, .. }) => assert_eq!(found, what),
                other => panic!("expected TargetsRoot({what}), got {other:?}"),
            }
        }
        // Nothing was touched.
        assert!(fs::read_dir(dir.path()).unwrap().next().is_none());
        assert_eq!(mode_of(dir.path()), 0o700);
    }

    #[test]
    fn declaring_the_root_as_a_plain_directory_is_harmless() {
        let dir = tempfile::tempdir().unwrap();
        Fixture::new().dir("").dir(".").apply(dir.path()).unwrap();
    }

    #[test]
    fn rejects_paths_that_escape_the_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        for fixture in [
            Fixture::new().file("../outside.txt", "x"),
            Fixture::new().file("a/../../outside.txt", "x"),
            Fixture::new().file("/abs.txt", "x"),
            Fixture::new().dir("../outside"),
            Fixture::new().clean(["../root-sibling"]),
            Fixture::new().dir_mode("../outside", 0o700),
        ] {
            let error = fixture.apply(&root).unwrap_err();
            assert!(matches!(error, FixtureError::InvalidPath(_)), "{error}");
        }
        assert!(
            !root.exists(),
            "an invalid fixture applies nothing, not even the root"
        );
        assert!(!dir.path().join("outside.txt").exists());
    }

    #[test]
    fn rejects_modes_outside_the_chmod_range() {
        let dir = tempfile::tempdir().unwrap();
        for fixture in [
            Fixture::new().dir_mode("d", 0o10000),
            Fixture::new().file_mode("f", "x", 0o17777),
        ] {
            assert!(matches!(
                fixture.apply(dir.path()),
                Err(FixtureError::InvalidMode { .. })
            ));
        }
        // The boundary values are fine.
        Fixture::new()
            .dir_mode("d", 0o7777)
            .file_mode("f", "x", 0)
            .apply(dir.path())
            .unwrap();
    }

    #[test]
    fn problems_lists_every_issue_without_touching_the_filesystem() {
        let fixture = Fixture::new()
            .file("", "x")
            .file("../a", "x")
            .clean([""])
            .dir("/abs")
            .dir_mode("m", 0o10000);
        let problems = fixture.problems();
        assert_eq!(problems.len(), 5, "{problems:?}");
        assert!(sample().problems().is_empty());
        assert!(Fixture::new().problems().is_empty());
    }

    #[test]
    fn a_symlinked_root_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        fs::create_dir(&real).unwrap();
        let linked = dir.path().join("linked");
        symlink(&real, &linked).unwrap();
        let error = Fixture::new()
            .file("x.txt", "x")
            .apply(&linked)
            .unwrap_err();
        assert!(matches!(
            error,
            FixtureError::Containment(ContainmentError::RootIsSymlink(_))
        ));
        assert!(!real.join("x.txt").exists());
    }

    #[test]
    fn a_root_that_is_a_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::write(&root, "not a dir").unwrap();
        assert!(Fixture::new().dir("a").apply(&root).is_err());
    }

    #[test]
    fn a_missing_root_is_created_with_an_explicit_mode() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("not/yet/there");
        Fixture::new().dir("a").apply(&root).unwrap();
        assert_eq!(mode_of(&root), 0o755);
        assert_eq!(mode_of(&root.join("a")), 0o755);
    }

    #[test]
    fn a_root_without_owner_permissions_is_repaired() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, Permissions::from_mode(0o000)).unwrap();
        Fixture::new().file("x.txt", "x").apply(&root).unwrap();
        assert_eq!(mode_of(&root), 0o755);
        assert_eq!(read(&root.join("x.txt")), "x");
    }

    #[test]
    fn a_usable_root_keeps_its_mode() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, Permissions::from_mode(0o700)).unwrap();
        Fixture::new().file("x.txt", "x").apply(&root).unwrap();
        assert_eq!(mode_of(&root), 0o700, "a private workspace stays private");
    }

    #[test]
    fn repair_section_root_clears_a_blocking_file_and_opens_a_locked_dir() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();

        let file_root = dir.path().join("level-1");
        fs::write(&file_root, "i am in the way").unwrap();
        fs::set_permissions(&file_root, Permissions::from_mode(0o000)).unwrap();
        repair_section_root(&file_root).unwrap();
        assert!(!file_root.exists());

        let locked = dir.path().join("level-2");
        fs::create_dir(&locked).unwrap();
        fs::set_permissions(&locked, Permissions::from_mode(0o000)).unwrap();
        repair_section_root(&locked).unwrap();
        assert_eq!(mode_of(&locked), 0o755);

        // A symlink is left alone (apply refuses it) and its target is untouched.
        let link = dir.path().join("level-3");
        symlink(outside.path(), &link).unwrap();
        repair_section_root(&link).unwrap();
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );

        // Missing: nothing to repair.
        repair_section_root(&dir.path().join("level-4")).unwrap();
    }

    #[test]
    fn content_conversions_and_bytes_are_exact() {
        let dir = tempfile::tempdir().unwrap();
        let binary: Vec<u8> = (0..=255u8).collect();
        let by_ref = String::from("by ref");
        let fixture = Fixture::new()
            .file("text.txt", "příliš žluťoučký\n")
            .file("owned.txt", String::from("owned"))
            .file("ref.txt", &by_ref)
            .file("vec.bin", binary.clone())
            .file("slice.bin", &binary[..16])
            .file("array.bin", b"\x89PNG\r\n\x1a\n")
            .file("empty", "");
        fixture.apply(dir.path()).unwrap();
        assert_eq!(read(&dir.path().join("text.txt")), "příliš žluťoučký\n");
        assert_eq!(read(&dir.path().join("owned.txt")), "owned");
        assert_eq!(read(&dir.path().join("ref.txt")), by_ref);
        assert_eq!(fs::read(dir.path().join("vec.bin")).unwrap(), binary);
        assert_eq!(
            fs::read(dir.path().join("slice.bin")).unwrap(),
            &binary[..16]
        );
        assert_eq!(
            fs::read(dir.path().join("array.bin")).unwrap(),
            b"\x89PNG\r\n\x1a\n"
        );
        assert_eq!(fs::read(dir.path().join("empty")).unwrap(), b"");
        assert_eq!(Content::from("x"), Content::Text("x".to_owned()));
        assert_eq!(Content::from(vec![1u8]), Content::Bytes(vec![1]));
        assert_eq!(Content::from(b"ab"), Content::Bytes(b"ab".to_vec()));
    }

    #[test]
    fn merge_appends_declarations_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let merged = Fixture::new()
            .file("a.txt", "first")
            .merge(Fixture::new().file("a.txt", "second").dir("d"));
        assert!(!merged.is_empty());
        assert!(Fixture::new().is_empty());
        assert_eq!(merged.dir_paths(), ["d"]);
        assert_eq!(merged.file_entries().count(), 2);
        merged.apply(dir.path()).unwrap();
        assert_eq!(
            read(&dir.path().join("a.txt")),
            "second",
            "later declarations win"
        );
        assert!(dir.path().join("d").is_dir());
    }

    #[test]
    fn introspection_reports_declarations_as_written() {
        let fixture = Fixture::new()
            .dir("d")
            .file("f", "x")
            .clean(["c"])
            .dir_mode("m", 0o700);
        assert_eq!(fixture.dir_paths(), ["d"]);
        assert_eq!(fixture.clean_paths(), ["c"]);
        assert_eq!(
            fixture.dir_mode_entries().collect::<Vec<_>>(),
            [("m", 0o700)]
        );
        assert_eq!(
            fixture.file_entries().collect::<Vec<_>>(),
            [("f", &Content::Text("x".to_owned()))]
        );
        assert_eq!(
            fixture.declared_paths().collect::<Vec<_>>(),
            ["d", "f", "c", "m"]
        );
    }

    #[test]
    fn files_in_new_directories_get_755_parents() {
        let dir = tempfile::tempdir().unwrap();
        Fixture::new()
            .file("one/two/three.txt", "x")
            .apply(dir.path())
            .unwrap();
        assert_eq!(mode_of(&dir.path().join("one")), 0o755);
        assert_eq!(mode_of(&dir.path().join("one/two")), 0o755);
        assert_eq!(mode_of(&dir.path().join("one/two/three.txt")), 0o644);
    }

    #[test]
    fn errors_describe_what_failed() {
        let dir = tempfile::tempdir().unwrap();
        let error = Fixture::new()
            .file("../x", "x")
            .apply(dir.path())
            .unwrap_err();
        assert!(error.to_string().contains("../x"), "{error}");
        let error = FixtureError::InvalidMode {
            path: "f".into(),
            mode: 0o10000,
        };
        assert!(error.to_string().contains("0o10000"), "{error}");
        let error = FixtureError::TargetsRoot {
            what: "file",
            path: ".".into(),
        };
        assert!(error.to_string().contains("root"), "{error}");
    }

    /// Run by [`explicit_modes_do_not_depend_on_the_umask`] in a child process whose umask
    /// is 077. Does nothing in a normal test run.
    #[test]
    fn umask_probe() {
        let Some(dir) = std::env::var_os("SHELLGAME_UMASK_PROBE_DIR") else {
            return;
        };
        let dir = PathBuf::from(dir);
        // Proof that the restrictive umask is really in force for this process.
        fs::write(dir.join("plain-file"), "x").unwrap();
        fs::create_dir(dir.join("plain-dir")).unwrap();
        Fixture::new()
            .dirs(["alpha", "gamma/deep"])
            .file("alpha/inside.txt", "x")
            .file("top.txt", "x")
            .file_mode("run.sh", "x", 0o755)
            .dir_mode("locked", 0o550)
            .apply(&dir.join("root"))
            .unwrap();
    }

    #[test]
    fn explicit_modes_do_not_depend_on_the_umask() {
        let dir = tempfile::tempdir().unwrap();
        let status = std::process::Command::new("sh")
            .args(["-c", "umask 077; exec \"$0\" \"$@\""])
            .arg(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "engine::fixture::tests::umask_probe",
                "--test-threads=1",
            ])
            .env("SHELLGAME_UMASK_PROBE_DIR", dir.path())
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "the umask-077 child run failed");

        // The premise: with umask 077 plain creation really is private.
        assert_eq!(mode_of(&dir.path().join("plain-file")), 0o600);
        assert_eq!(mode_of(&dir.path().join("plain-dir")), 0o700);

        // The fixture results are the declared ones regardless.
        let root = dir.path().join("root");
        assert_eq!(mode_of(&root), 0o755);
        for directory in ["alpha", "gamma", "gamma/deep"] {
            assert_eq!(mode_of(&root.join(directory)), 0o755, "{directory}");
        }
        for file in ["alpha/inside.txt", "top.txt"] {
            assert_eq!(mode_of(&root.join(file)), 0o644, "{file}");
        }
        assert_eq!(mode_of(&root.join("run.sh")), 0o755);
        assert_eq!(mode_of(&root.join("locked")), 0o550);
    }
}
