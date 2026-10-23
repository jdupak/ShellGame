//! What a level's files look like, applied with repair semantics.
//!
//! `shellgame reset` has to work after whatever the player did, so applying a setup makes things
//! accessible before replacing them (chmod 000 directories, read-only files), never follows a
//! symlink (a link at a declared path is replaced and its target left alone; files are created
//! with `O_EXCL`) and replaces a file where a directory belongs and the other way round.
//! Phases, whatever the order of the items: clean, dirs, files, builds, directory modes
//! (deepest first, so restricting a parent never blocks the chmod of a child). Everything
//! created or repaired ends with an explicit mode: 0o755 for directories, 0o644 for files.

use std::cmp::Reverse;
use std::fs::{self, OpenOptions, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

use super::file_text;
use super::paths::{absent, join, split};

#[derive(Clone, Copy)]
enum Body {
    Text(&'static str),
    Bytes(&'static [u8]),
}

#[derive(Clone, Copy)]
enum Item {
    Dir(&'static str),
    File(&'static str, Body),
    Clean(&'static str),
    Build(fn(&Path) -> io::Result<()>),
}

/// One declaration of a level's files; all paths are relative to the section root.
#[derive(Clone, Copy)]
pub struct Setup {
    item: Item,
    mode: Option<u32>,
}

const fn new(item: Item) -> Setup {
    Setup { item, mode: None }
}

#[rustfmt::skip]
mod vocabulary {
    use super::*;

    /// A directory (and its parents).
    pub const fn dir(path: &'static str) -> Setup { new(Item::Dir(path)) }
    /// A text file, rewritten on every prepare. A `text` that starts with a newline is an indented
    /// block (see [`dedent`](crate::game::dedent)) and the file ends with a newline.
    pub const fn file(path: &'static str, text: &'static str) -> Setup { new(Item::File(path, Body::Text(text))) }
    /// A file with raw bytes; `&[b'x'; N]` repeats a byte.
    pub const fn binary(path: &'static str, bytes: &'static [u8]) -> Setup { new(Item::File(path, Body::Bytes(bytes))) }
    /// Remove whatever is at this path (file, tree or symlink) before anything is created.
    pub const fn clean(path: &'static str) -> Setup { new(Item::Clean(path)) }
    /// Build files with code. It gets the section root; use [`mkdir`] and [`write()`] to touch it.
    pub const fn build(build: fn(&Path) -> io::Result<()>) -> Setup { new(Item::Build(build)) }

    impl Setup {
        /// An exact mode for a file or directory (`0..=0o7777`); directory modes are applied last.
        pub const fn mode(self, mode: u32) -> Self { Self { mode: Some(mode), ..self } }
    }
}
pub use vocabulary::*;

#[cfg(test)]
impl Setup {
    pub fn path(&self) -> Option<&'static str> {
        match self.item {
            Item::Dir(path) | Item::File(path, _) | Item::Clean(path) => Some(path),
            Item::Build(_) => None,
        }
    }
}

/// A file as `file` writes it (default mode), for `build` functions; see also [`mkdir`].
pub fn write(root: &Path, rel: &str, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    write_file(root, rel, bytes.as_ref(), None)
}

/// Make the existing `root` match `items`.
pub(super) fn apply(root: &Path, items: &[Setup]) -> io::Result<()> {
    let mut modes = vec![];
    for phase in 0..4 {
        for setup in items {
            match (phase, setup.item) {
                (0, Item::Clean(path)) => clean_path(root, path)?,
                (1, Item::Dir(path)) => {
                    mkdir(root, path)?;
                    modes.extend(setup.mode.map(|mode| (path, mode)));
                }
                (2, Item::File(path, body)) => {
                    let bytes = match body {
                        Body::Text(text) => file_text(text).into_bytes(),
                        Body::Bytes(bytes) => bytes.to_vec(),
                    };
                    write_file(root, path, &bytes, setup.mode)?;
                }
                (3, Item::Build(build)) => build(root)?,
                _ => {}
            }
        }
    }
    modes.sort_by_key(|(path, _)| Reverse(split(path).map_or(0, |parts| parts.len())));
    modes.into_iter().try_for_each(|(path, mode)| chmod(&join(root, path), mode))
}

/// A directory the game owns (the workspace, a section root): created when missing, made usable when
/// its owner permissions were taken away, and replaced when the player put a file or a symlink in its
/// place (the link is removed, never followed).
pub fn ensure_dir(dir: &Path) -> io::Result<()> {
    match fs::symlink_metadata(dir) {
        Ok(meta) if meta.is_dir() => {
            if meta.mode() & 0o700 == 0o700 {
                Ok(())
            } else {
                chmod(dir, 0o755)
            }
        }
        Ok(_) => {
            remove_tree(dir)?;
            make_dir(dir)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            at(fs::create_dir_all(dir), "vytvořit", dir)?;
            chmod(dir, 0o755)
        }
        Err(error) => at(Err(error), "prozkoumat", dir),
    }
}

/// Remove a file, symlink or directory tree (never following a symlink), recovering from
/// restricted permissions. A missing path is fine.
pub fn remove_tree(path: &Path) -> io::Result<()> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(error) if absent(&error) => return Ok(()),
        Err(error) => return at(Err(error), "prozkoumat", path),
    };
    if meta.is_dir() {
        let _ = fs::set_permissions(path, Permissions::from_mode(0o700));
        for entry in at(fs::read_dir(path), "vypsat", path)? {
            remove_tree(&at(entry, "vypsat", path)?.path())?;
        }
        return at(fs::remove_dir(path), "smazat adresář", path);
    }
    if !meta.is_symlink() {
        // chmod would follow a symlink, so a link is removed as it is.
        let _ = fs::set_permissions(path, Permissions::from_mode(0o600));
    }
    at(fs::remove_file(path), "smazat", path)
}

fn at<T>(result: io::Result<T>, action: &str, path: &Path) -> io::Result<T> {
    let describe = |error: io::Error| {
        io::Error::new(error.kind(), format!("nelze {action} {}: {error}", path.display()))
    };
    result.map_err(describe)
}

fn chmod(path: &Path, mode: u32) -> io::Result<()> {
    at(fs::set_permissions(path, Permissions::from_mode(mode)), "změnit oprávnění", path)
}

/// Remove what is at `rel`; a symlink along the way is removed instead of what lies beyond it,
/// and directories along the way are made usable, so a parent the player locked cannot block it.
fn clean_path(root: &Path, rel: &str) -> io::Result<()> {
    let parts = split(rel)?;
    let mut path = root.to_path_buf();
    for (index, part) in parts.iter().enumerate() {
        path.push(part);
        let meta = match fs::symlink_metadata(&path) {
            Ok(meta) => meta,
            Err(error) if absent(&error) => return Ok(()),
            Err(error) => return at(Err(error), "prozkoumat", &path),
        };
        if meta.is_symlink() || index + 1 == parts.len() {
            return remove_tree(&path);
        }
        if !meta.is_dir() {
            return Ok(());
        }
        if meta.mode() & 0o700 != 0o700 {
            let _ = fs::set_permissions(&path, Permissions::from_mode(0o755));
        }
    }
    Ok(())
}

/// Make `rel` a real directory, component by component: a symlink or file in the way is replaced,
/// an existing directory is reset to 0o755. Also for `build` functions.
pub fn mkdir(root: &Path, rel: &str) -> io::Result<()> {
    let mut path = root.to_path_buf();
    for part in split(rel)? {
        path.push(part);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_dir() => {
                let _ = fs::set_permissions(&path, Permissions::from_mode(0o755));
            }
            Ok(_) => {
                remove_tree(&path)?;
                make_dir(&path)?;
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => make_dir(&path)?,
            Err(error) => return at(Err(error), "prozkoumat", &path),
        }
    }
    Ok(())
}

fn make_dir(path: &Path) -> io::Result<()> {
    at(fs::create_dir(path), "vytvořit adresář", path)?;
    chmod(path, 0o755)
}

fn write_file(root: &Path, rel: &str, bytes: &[u8], mode: Option<u32>) -> io::Result<()> {
    let parts = split(rel)?;
    let Some((_, parents)) = parts.split_last() else {
        return at(Err(io::ErrorKind::InvalidInput.into()), "zapsat jako soubor", root);
    };
    mkdir(root, &parents.join("/"))?;
    let path = join(root, rel);
    remove_tree(&path)?;
    let open = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&path);
    let mut file = at(open, "vytvořit soubor", &path)?;
    at(file.write_all(bytes), "zapsat", &path)?;
    at(
        file.set_permissions(Permissions::from_mode(mode.unwrap_or(0o644))),
        "změnit oprávnění",
        &path,
    )
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;

    fn mode(path: &Path) -> u32 {
        fs::symlink_metadata(path).unwrap().mode() & 0o7777
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    fn chmod_all(root: &Path, mode: u32, paths: &[&str]) {
        for path in paths {
            fs::set_permissions(root.join(path), Permissions::from_mode(mode)).unwrap();
        }
    }

    #[test]
    fn builds_the_tree_with_explicit_modes_and_directory_modes_last() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let block = r#"
            one
              two
        "#;
        let items = [
            dir("a/b"),
            file("a/f.txt", "x"),
            file("run.sh", "#!").mode(0o755),
            binary("bin", b"\x00\xff"),
            build(|root| write(root, "made", "generated")),
            file("block", block),
            file("none", "").mode(0o000),
            dir("locked").mode(0o550),
            file("locked/in.txt", "y"),
            dir("deep").mode(0o500),
            dir("deep/er").mode(0o500),
        ];
        let paths = ["a", "a/b", "a/f.txt", "run.sh", "none", "locked", "deep", "deep/er"];
        for _ in 0..2 {
            apply(root, &items).unwrap();
            let modes = paths.map(|path| mode(&root.join(path)));
            assert_eq!(modes, [0o755, 0o755, 0o644, 0o755, 0, 0o550, 0o500, 0o500]);
            let texts = ["a/f.txt", "made", "locked/in.txt", "block"].map(|p| read(&root.join(p)));
            assert_eq!(texts, ["x", "generated", "y", "one\n  two\n"]);
            assert_eq!(fs::read(root.join("bin")).unwrap(), [0, 0xff]);
        }
        chmod_all(root, 0o755, &["locked", "deep/er", "deep"]);
    }

    #[test]
    fn repairs_whatever_the_player_left_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let items = [
            clean("junk"),
            dir("d/e"),
            file("d/e/f.txt", "new"),
            file("g.txt", "new"),
            file("h/i.txt", "new"),
        ];
        apply(root, &items).unwrap();
        // A read-only edited file, a directory where a file belongs, a file where a directory
        // belongs, a tree to clean and chmod 000 on directories.
        fs::write(root.join("g.txt"), "edited").unwrap();
        chmod_all(root, 0o444, &["g.txt"]);
        fs::remove_file(root.join("g.txt")).unwrap();
        fs::create_dir(root.join("g.txt")).unwrap();
        fs::remove_dir_all(root.join("h")).unwrap();
        fs::write(root.join("h"), "a file").unwrap();
        fs::create_dir_all(root.join("junk/deep")).unwrap();
        chmod_all(root, 0o000, &["junk/deep", "junk", "d/e", "d"]);
        apply(root, &items).unwrap();
        assert!(!root.join("junk").exists());
        assert_eq!(
            ["d/e/f.txt", "g.txt", "h/i.txt"].map(|path| read(&root.join(path))),
            ["new", "new", "new"]
        );
        assert_eq!(
            [mode(&root.join("d")), mode(&root.join("d/e")), mode(&root.join("g.txt"))],
            [0o755, 0o755, 0o644]
        );
    }

    #[test]
    fn a_symlink_at_a_declared_path_is_replaced_and_its_target_left_alone() {
        let (tmp, outside) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let root = tmp.path();
        fs::write(outside.path().join("secret"), "keep").unwrap();
        for link in ["d", "gone"] {
            symlink(outside.path(), root.join(link)).unwrap();
        }
        symlink(outside.path().join("secret"), root.join("f.txt")).unwrap();
        apply(
            root,
            &[clean("gone/secret"), dir("d/sub"), file("d/x.txt", "y"), file("f.txt", "new")],
        )
        .unwrap();
        assert_eq!(read(&outside.path().join("secret")), "keep");
        assert_eq!(
            fs::read_dir(outside.path()).unwrap().count(),
            1,
            "nothing was written through a link"
        );
        assert!(
            !root.join("gone").exists()
                && !root.join("d").is_symlink()
                && root.join("d/sub").is_dir()
        );
        assert_eq!(read(&root.join("f.txt")), "new");
    }

    #[test]
    fn declarations_stay_inside_and_the_game_directories_are_repaired() {
        let (tmp, outside) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        for bad in ["/etc/x", "../x", "a/../../x"] {
            assert!(
                [dir(bad), file(bad, "x"), clean(bad)].iter().all(|item| apply(
                    tmp.path(),
                    &[*item]
                )
                .is_err())
            );
        }
        assert!(apply(tmp.path(), &[file("", "x")]).is_err(), "the root is not a file");
        let root = tmp.path().join("level-1");
        ensure_dir(&root).unwrap();
        for mode_left in [0o000, 0o600] {
            chmod_all(tmp.path(), mode_left, &["level-1"]);
            ensure_dir(&root).unwrap();
            assert_eq!(mode(&root), 0o755);
        }
        fs::remove_dir(&root).unwrap();
        fs::write(&root, "a file").unwrap();
        ensure_dir(&root).unwrap();
        assert!(root.is_dir());
        // A link in its place is removed and its target left alone.
        fs::remove_dir(&root).unwrap();
        fs::write(outside.path().join("keep"), "mine").unwrap();
        symlink(outside.path(), &root).unwrap();
        ensure_dir(&root).unwrap();
        assert!(!root.is_symlink() && root.is_dir());
        assert_eq!(read(&outside.path().join("keep")), "mine");
        assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 1);
    }
}
