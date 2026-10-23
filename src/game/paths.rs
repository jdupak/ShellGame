//! Declared paths are relative to the section root and cannot leave it; paths the player is at
//! or is going to are compared after resolving symlinks.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Nothing there (a missing path or parent), as opposed to a real error.
pub fn absent(error: &io::Error) -> bool {
    matches!(error.kind(), io::ErrorKind::NotFound | io::ErrorKind::NotADirectory)
}

fn refused(what: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, what)
}

/// The components of a declared path (`""` and `"."` are the root). Absolute paths, `..` and NUL
/// are refused.
pub fn split(rel: &str) -> io::Result<Vec<&str>> {
    if rel.starts_with('/') || rel.contains('\0') || rel.split('/').any(|part| part == "..") {
        return Err(refused(format!("není prostá relativní cesta: {rel:?}")));
    }
    Ok(rel.split('/').filter(|part| !matches!(*part, "" | ".")).collect())
}

/// `rel` below `root`, lexically; a refused path is the root itself.
pub fn join(root: &Path, rel: &str) -> PathBuf {
    split(rel)
        .unwrap_or_default()
        .into_iter()
        .fold(root.to_path_buf(), |path, part| path.join(part))
}

/// A declared path below `root` that no symlink leads out of: the root and every component (the
/// last one only unless `leaf_link_ok`) must not be a symlink. Nothing is ever followed.
pub fn resolve(root: &Path, rel: &str, leaf_link_ok: bool) -> io::Result<PathBuf> {
    let parts = split(rel)?;
    let is_link = |path: &Path| fs::symlink_metadata(path).is_ok_and(|meta| meta.is_symlink());
    let mut path = root.to_path_buf();
    if is_link(&path) {
        return Err(refused(format!("{} je symbolický odkaz", path.display())));
    }
    for (index, part) in parts.iter().enumerate() {
        path.push(part);
        if !(leaf_link_ok && index + 1 == parts.len()) && is_link(&path) {
            return Err(refused(format!("{rel:?} vede přes symbolický odkaz")));
        }
    }
    Ok(path)
}

/// Resolve `.` and `..` textually, as `cd -L` does for the logical `$PWD`.
pub fn normalize(path: &Path) -> PathBuf {
    let mut parts: Vec<Component> = vec![];
    for part in path.components() {
        match (part, parts.last()) {
            (Component::CurDir, _) | (Component::ParentDir, Some(Component::RootDir)) => {}
            (Component::ParentDir, Some(Component::Normal(_))) => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    if parts.is_empty() { PathBuf::from(".") } else { parts.into_iter().collect() }
}

/// Canonicalize the longest existing prefix of `path` and append the rest lexically, so a
/// directory that does not exist (yet) or a workspace below a symlink still compares correctly.
pub fn canonical(path: &Path) -> PathBuf {
    let parts: Vec<Component> = path.components().collect();
    let existing = (1..=parts.len()).rev().find_map(|end| {
        let real = fs::canonicalize(parts[..end].iter().collect::<PathBuf>()).ok();
        real.map(|real| (end, real))
    });
    match existing {
        Some((end, real)) => normalize(&real.join(parts[end..].iter().collect::<PathBuf>())),
        None => normalize(path),
    }
}

pub fn same(a: &Path, b: &Path) -> bool {
    canonical(a) == canonical(b)
}

/// Whether `path` resolves to somewhere strictly inside `/tmp` (`/tmp` itself does not count): where
/// the game may delete (the workspace) and the shell may `chmod`.
pub fn inside_tmp(path: &Path) -> bool {
    let (tmp, real) = (canonical(Path::new("/tmp")), canonical(path));
    real != tmp && real.starts_with(&tmp)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::symlink;

    use super::*;

    #[test]
    fn declared_paths_cannot_leave_the_root_or_pass_through_a_symlink() {
        assert!(["/etc", "../x", "a/../b", "a\0b"].iter().all(|bad| split(bad).is_err()));
        assert_eq!(split("a//b/./c/").unwrap(), ["a", "b", "c"]);
        let (tmp, outside) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let root = tmp.path().join("root");
        fs::create_dir_all(root.join("real")).unwrap();
        symlink(outside.path(), root.join("link")).unwrap();
        symlink(outside.path(), root.join("real/leaf")).unwrap();
        assert!(resolve(&root, "real/not-yet-there", false).is_ok());
        assert!(
            resolve(&root, "link/file", false).is_err()
                && resolve(&root, "real/leaf", false).is_err()
        );
        assert!(
            resolve(&root, "real/leaf", true).is_ok(),
            "a leftover link is for the caller to report"
        );
        assert!(resolve(&root.join("link"), "x", false).is_err(), "the root itself");
    }

    #[test]
    fn directories_compare_after_resolving_links_even_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let (real, link) = (tmp.path().join("real"), tmp.path().join("link"));
        fs::create_dir(&real).unwrap();
        symlink(&real, &link).unwrap();
        assert!(same(&link, &real) && same(&link.join("missing/../new"), &real.join("new")));
        assert!(!same(&real, tmp.path()));
        assert_eq!(normalize(Path::new("/a/b/../c/./d")), Path::new("/a/c/d"));
        assert_eq!(normalize(Path::new("a/../../b")), Path::new("../b"));
    }
}
