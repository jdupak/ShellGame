//! Section-relative paths and containment: the one implementation.
//!
//! Level authors declare *paths*, not locations. A fixture, a requirement, a start
//! directory and a `cd` rule all speak the same vocabulary, and all of them go through
//! this module, so a level can never disagree with itself about where it lives, nor
//! reach outside the workspace the game owns by way of `..`, an absolute path or a
//! symlinked parent.
//!
//! Two different jobs live here:
//!
//! * **Declared paths** ([`RelPath`], [`resolve_within`]): strict. Anything that is not a
//!   plain relative path, or that passes through a symlink, is rejected.
//! * **Observed paths** ([`normalize_lexically`], [`soft_canonicalize`]): lenient. They
//!   describe where the player *is* or *is going*, which may not exist yet, and they
//!   never fail.

use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

/// Why a declared path was refused as a plain relative path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    /// The path starts with `/`.
    Absolute(String),
    /// The path contains a `..` component.
    ParentComponent(String),
    /// The path contains a NUL byte, which no filesystem accepts.
    NulByte(String),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absolute(value) => {
                write!(f, "path must be relative to the section root: {value:?}")
            }
            Self::ParentComponent(value) => {
                write!(f, "path must not contain a '..' component: {value:?}")
            }
            Self::NulByte(value) => write!(f, "path must not contain a NUL byte: {value:?}"),
        }
    }
}

impl std::error::Error for PathError {}

/// A declared path that is relative to a section root and cannot leave it.
///
/// The value is normalized the way Python's `Path(value)` does it: empty and `.`
/// components are dropped, so `"a//b/./c/"` and `"a/b/c"` are the same path, and both
/// `""` and `"."` are the root. Absolute paths and any `..` component are rejected:
/// a level declares paths, not locations, and a `..` could let it read or write
/// outside the workspace.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct RelPath(String);

impl RelPath {
    /// Validate and normalize a declared path.
    pub fn parse(value: &str) -> Result<Self, PathError> {
        if value.starts_with('/') {
            return Err(PathError::Absolute(value.to_owned()));
        }
        if value.contains('\0') {
            return Err(PathError::NulByte(value.to_owned()));
        }
        let mut normalized = String::with_capacity(value.len());
        for part in value.split('/') {
            match part {
                "" | "." => {}
                ".." => return Err(PathError::ParentComponent(value.to_owned())),
                _ => {
                    if !normalized.is_empty() {
                        normalized.push('/');
                    }
                    normalized.push_str(part);
                }
            }
        }
        Ok(Self(normalized))
    }

    /// The root itself (no components).
    pub fn root() -> Self {
        Self(String::new())
    }

    /// True for `""` and `"."`: the section root.
    pub fn is_root(&self) -> bool {
        self.0.is_empty()
    }

    /// The normalized text: components joined by `/`, and `""` for the root.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The path components, outermost first.
    pub fn components(&self) -> impl DoubleEndedIterator<Item = &str> {
        self.0.split('/').filter(|part| !part.is_empty())
    }

    /// Number of components; the root has depth 0.
    pub fn depth(&self) -> usize {
        self.components().count()
    }

    /// The last component, or `None` for the root.
    pub fn file_name(&self) -> Option<&str> {
        self.components().next_back()
    }

    /// The path without its last component, or `None` for the root.
    pub fn parent(&self) -> Option<RelPath> {
        if self.is_root() {
            return None;
        }
        Some(Self(
            self.0
                .rsplit_once('/')
                .map(|(parent, _)| parent.to_owned())
                .unwrap_or_default(),
        ))
    }

    /// True when `self` is `other` or lies below it (component-wise, so `maze2` is not
    /// inside `maze`). Every path is inside the root.
    pub fn starts_with(&self, other: &RelPath) -> bool {
        let mut mine = self.components();
        other.components().all(|theirs| mine.next() == Some(theirs))
    }

    /// Place this path under `root`. The root path maps to `root` itself, without the
    /// trailing slash that `Path::join("")` would add.
    pub fn under(&self, root: &Path) -> PathBuf {
        if self.is_root() {
            root.to_path_buf()
        } else {
            root.join(&self.0)
        }
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for RelPath {
    type Err = PathError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

/// A declared path does not stay inside the root it is resolved against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainmentError {
    /// The root itself is a symlink. Ancestors of the root may be symlinks (macOS
    /// `/tmp` is one), but the root must be a real directory.
    RootIsSymlink(PathBuf),
    /// The declared path is not a plain relative path.
    InvalidPath(PathError),
    /// A component of the declared path is a symlink.
    Symlink {
        /// The declared path, as written.
        path: String,
    },
}

impl fmt::Display for ContainmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RootIsSymlink(root) => write!(f, "root cannot be a symlink: {}", root.display()),
            Self::InvalidPath(error) => error.fmt(f),
            Self::Symlink { path } => write!(f, "path contains a symlink: {path:?}"),
        }
    }
}

impl std::error::Error for ContainmentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPath(error) => Some(error),
            _ => None,
        }
    }
}

impl From<PathError> for ContainmentError {
    fn from(error: PathError) -> Self {
        Self::InvalidPath(error)
    }
}

/// True when `path` itself (not its target) is a symlink. Missing paths and
/// unreadable ones are not symlinks as far as containment is concerned: nothing can be
/// reached through them either.
pub(crate) fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}

/// Resolve a declared path against `root`, refusing to escape it.
///
/// The root must not be a symlink, `rel` must be a plain relative path (see
/// [`RelPath`]), and no existing prefix of it may be a symlink. Each prefix is checked
/// with `symlink_metadata`, so nothing is ever followed. With `allow_leaf_symlink` the
/// final component may be a symlink: fixtures use that to replace one, and `gone`-style
/// checks use it to notice a leftover link. Checking stops at the first prefix that does
/// not exist, because nothing below a missing directory can be a symlink.
///
/// Ancestors of `root` are deliberately not inspected.
pub fn resolve_within(
    root: &Path,
    rel: &str,
    allow_leaf_symlink: bool,
) -> Result<PathBuf, ContainmentError> {
    if is_symlink(root) {
        return Err(ContainmentError::RootIsSymlink(root.to_path_buf()));
    }
    let relative = RelPath::parse(rel)?;
    let count = relative.depth();
    let mut current = root.to_path_buf();
    for (index, part) in relative.components().enumerate() {
        current.push(part);
        if allow_leaf_symlink && index + 1 == count {
            break;
        }
        match fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(ContainmentError::Symlink {
                    path: rel.to_owned(),
                });
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    Ok(relative.under(root))
}

/// Resolve `.` and `..` textually, the way `cd -L` does for the logical `$PWD`.
///
/// No filesystem access: `/a/link/..` is `/a`, whatever `link` points to. `..` at the
/// root stays at the root. A relative path keeps the leading `..` components it cannot
/// cancel, and an empty result is `"."`.
pub fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => out.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => match out.components().next_back() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                Some(Component::RootDir) => {}
                _ => out.push(".."),
            },
            Component::Normal(part) => out.push(part),
        }
    }
    if out.as_os_str().is_empty() {
        out.push(".");
    }
    out
}

/// Canonicalize the longest existing prefix of `path` and append the rest.
///
/// Where the player is going may not exist yet (a `cd` to a typo), and the workspace
/// may live below a symlink (macOS `/tmp` is `/private/tmp`), so strict `canonicalize`
/// is too brittle for comparing directories. The existing part is resolved physically
/// by the OS; the missing tail is appended lexically (`.` dropped, `..` cancels the
/// previous component). It never fails: with nothing resolvable the lexical
/// normalization of the input is returned.
///
/// Expects an absolute path; a relative one is resolved by the OS against the process
/// working directory, which the engine never relies on.
pub fn soft_canonicalize(path: &Path) -> PathBuf {
    if let Ok(real) = fs::canonicalize(path) {
        return real;
    }
    let components: Vec<Component<'_>> = path.components().collect();
    for existing in (1..components.len()).rev() {
        let prefix: PathBuf = components[..existing].iter().collect();
        if let Ok(mut real) = fs::canonicalize(&prefix) {
            for component in &components[existing..] {
                match component {
                    Component::Normal(part) => real.push(part),
                    Component::ParentDir => {
                        real.pop();
                    }
                    Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
                }
            }
            return real;
        }
    }
    normalize_lexically(path)
}

/// True when two paths name the same location: both are soft-canonicalized first, so a
/// path through a symlinked ancestor (macOS `/tmp`) equals its canonical spelling and a
/// directory that does not exist yet can still be compared.
pub fn same_path(a: &Path, b: &Path) -> bool {
    soft_canonicalize(a) == soft_canonicalize(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    fn rel(value: &str) -> RelPath {
        RelPath::parse(value).expect("valid path")
    }

    #[test]
    fn relpath_normalizes_like_python_path() {
        assert_eq!(rel("a/b").as_str(), "a/b");
        assert_eq!(rel("a//b/./c/").as_str(), "a/b/c");
        assert_eq!(rel("./a").as_str(), "a");
        assert_eq!(rel("a/./").as_str(), "a");
        assert_eq!(rel("...").as_str(), "...");
        assert_eq!(rel(".hidden/x").as_str(), ".hidden/x");
        assert_eq!(rel("a b/c").as_str(), "a b/c");
    }

    #[test]
    fn relpath_root_is_empty_or_dot() {
        for value in ["", ".", "./", "././"] {
            let path = rel(value);
            assert!(path.is_root(), "{value:?}");
            assert_eq!(path.as_str(), "");
            assert_eq!(path.depth(), 0);
            assert_eq!(path.components().count(), 0);
            assert_eq!(path, RelPath::root());
        }
    }

    #[test]
    fn relpath_rejects_absolute_and_parent_components() {
        for value in ["/", "/a", "//a", "/a/b"] {
            assert!(
                matches!(RelPath::parse(value), Err(PathError::Absolute(_))),
                "{value:?}"
            );
        }
        for value in ["..", "a/..", "../a", "a/../b", "./..", "a/b/../.."] {
            assert!(
                matches!(RelPath::parse(value), Err(PathError::ParentComponent(_))),
                "{value:?}"
            );
        }
        assert!(matches!(RelPath::parse("a\0b"), Err(PathError::NulByte(_))));
    }

    #[test]
    fn relpath_components_depth_parent_and_name() {
        let path = rel("deep/structure/start");
        assert_eq!(
            path.components().collect::<Vec<_>>(),
            ["deep", "structure", "start"]
        );
        assert_eq!(path.depth(), 3);
        assert_eq!(path.file_name(), Some("start"));
        assert_eq!(path.parent(), Some(rel("deep/structure")));
        assert_eq!(rel("a").parent(), Some(RelPath::root()));
        assert_eq!(RelPath::root().parent(), None);
        assert_eq!(RelPath::root().file_name(), None);
    }

    #[test]
    fn relpath_starts_with_is_component_wise() {
        assert!(rel("maze/entry").starts_with(&rel("maze")));
        assert!(rel("maze").starts_with(&rel("maze")));
        assert!(rel("maze").starts_with(&RelPath::root()));
        assert!(!rel("maze2/entry").starts_with(&rel("maze")));
        assert!(!rel("maze").starts_with(&rel("maze/entry")));
    }

    #[test]
    fn relpath_under_has_no_trailing_slash_for_the_root() {
        let root = Path::new("/ws/level-1");
        assert_eq!(RelPath::root().under(root), PathBuf::from("/ws/level-1"));
        assert_eq!(rel("a/b").under(root), PathBuf::from("/ws/level-1/a/b"));
    }

    #[test]
    fn relpath_from_str_and_display() {
        let path: RelPath = "a/./b".parse().unwrap();
        assert_eq!(path.to_string(), "a/b");
        assert!("../x".parse::<RelPath>().is_err());
    }

    #[test]
    fn error_messages_name_the_offending_value() {
        let error = RelPath::parse("../x").unwrap_err();
        assert!(error.to_string().contains("../x"));
        let error = ContainmentError::from(RelPath::parse("/x").unwrap_err());
        assert!(error.to_string().contains("/x"));
    }

    #[test]
    fn resolve_within_joins_existing_and_missing_paths() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a/b")).unwrap();
        assert_eq!(
            resolve_within(dir.path(), "a/b", false).unwrap(),
            dir.path().join("a/b")
        );
        assert_eq!(
            resolve_within(dir.path(), "a/missing/deeper/still", false).unwrap(),
            dir.path().join("a/missing/deeper/still")
        );
        assert_eq!(resolve_within(dir.path(), "", false).unwrap(), dir.path());
        assert_eq!(resolve_within(dir.path(), ".", false).unwrap(), dir.path());
    }

    #[test]
    fn resolve_within_rejects_absolute_and_parent_paths() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            resolve_within(dir.path(), "/etc/passwd", false),
            Err(ContainmentError::InvalidPath(PathError::Absolute(_)))
        ));
        assert!(matches!(
            resolve_within(dir.path(), "../outside", false),
            Err(ContainmentError::InvalidPath(PathError::ParentComponent(_)))
        ));
        assert!(matches!(
            resolve_within(dir.path(), "a/../..", true),
            Err(ContainmentError::InvalidPath(PathError::ParentComponent(_)))
        ));
    }

    #[test]
    fn resolve_within_rejects_a_symlinked_parent() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::create_dir(outside.path().join("inner")).unwrap();
        let root = dir.path().join("root");
        fs::create_dir(&root).unwrap();
        symlink(outside.path(), root.join("link")).unwrap();

        for allow_leaf in [false, true] {
            assert!(
                matches!(
                    resolve_within(&root, "link/inner", allow_leaf),
                    Err(ContainmentError::Symlink { .. })
                ),
                "a symlinked parent is rejected even when the leaf may be a link"
            );
        }
        assert!(matches!(
            resolve_within(&root, "link/inner/deeper/missing", false),
            Err(ContainmentError::Symlink { .. })
        ));
    }

    #[test]
    fn resolve_within_handles_a_symlinked_leaf() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), dir.path().join("leaf")).unwrap();
        symlink("does-not-exist", dir.path().join("dangling")).unwrap();

        assert!(matches!(
            resolve_within(dir.path(), "leaf", false),
            Err(ContainmentError::Symlink { .. })
        ));
        assert!(matches!(
            resolve_within(dir.path(), "dangling", false),
            Err(ContainmentError::Symlink { .. })
        ));
        assert_eq!(
            resolve_within(dir.path(), "leaf", true).unwrap(),
            dir.path().join("leaf")
        );
        assert_eq!(
            resolve_within(dir.path(), "dangling", true).unwrap(),
            dir.path().join("dangling")
        );
    }

    #[test]
    fn resolve_within_rejects_a_symlinked_root_but_not_its_ancestors() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        fs::create_dir_all(real.join("section/inner")).unwrap();

        // The root itself is a link: refused.
        let linked_root = dir.path().join("linked-root");
        symlink(real.join("section"), &linked_root).unwrap();
        assert!(matches!(
            resolve_within(&linked_root, "inner", false),
            Err(ContainmentError::RootIsSymlink(_))
        ));
        assert!(matches!(
            resolve_within(&linked_root, "", false),
            Err(ContainmentError::RootIsSymlink(_))
        ));

        // An ancestor of the root is a link (macOS /tmp -> /private/tmp): fine.
        let linked_parent = dir.path().join("linked-parent");
        symlink(&real, &linked_parent).unwrap();
        let root_below_link = linked_parent.join("section");
        assert_eq!(
            resolve_within(&root_below_link, "inner", false).unwrap(),
            root_below_link.join("inner")
        );
    }

    #[test]
    fn resolve_within_stops_checking_below_a_missing_prefix() {
        let dir = tempfile::tempdir().unwrap();
        // `missing` does not exist, so nothing beneath it can be a link.
        assert!(resolve_within(dir.path(), "missing/a/b", false).is_ok());
    }

    #[test]
    fn resolve_within_tolerates_a_file_in_the_middle_of_the_path() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("file"), "x").unwrap();
        assert_eq!(
            resolve_within(dir.path(), "file/below", false).unwrap(),
            dir.path().join("file/below")
        );
    }

    #[test]
    fn normalize_lexically_resolves_dots_without_touching_the_filesystem() {
        let cases = [
            ("/a/b/../c", "/a/c"),
            ("/a/./b//c/", "/a/b/c"),
            ("/..", "/"),
            ("/a/../../b", "/b"),
            ("/", "/"),
            ("/a/b/../..", "/"),
            ("a/../..", ".."),
            ("a/b/..", "a"),
            ("a/..", "."),
            ("", "."),
            ("./x", "x"),
            ("../../x", "../../x"),
        ];
        for (input, expected) in cases {
            assert_eq!(
                normalize_lexically(Path::new(input)),
                PathBuf::from(expected),
                "{input:?}"
            );
        }
    }

    #[test]
    fn normalize_lexically_keeps_the_logical_path_through_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        fs::create_dir_all(real.join("deep")).unwrap();
        symlink(real.join("deep"), dir.path().join("link")).unwrap();
        // `cd -L link/..` stays in the logical parent, not in real/.
        let logical = dir.path().join("link/..");
        assert_eq!(normalize_lexically(&logical), dir.path());
    }

    #[test]
    fn soft_canonicalize_resolves_existing_paths() {
        let dir = tempfile::tempdir().unwrap();
        let real = fs::canonicalize(dir.path()).unwrap();
        fs::create_dir_all(real.join("a/b")).unwrap();
        assert_eq!(soft_canonicalize(&real.join("a/b")), real.join("a/b"));
        assert_eq!(soft_canonicalize(&real.join("a/./b/..")), real.join("a"));
    }

    #[test]
    fn soft_canonicalize_resolves_symlinks_in_the_existing_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let real = fs::canonicalize(dir.path()).unwrap();
        fs::create_dir_all(real.join("target")).unwrap();
        symlink(real.join("target"), real.join("alias")).unwrap();

        assert_eq!(
            soft_canonicalize(&real.join("alias/missing/deeper")),
            real.join("target/missing/deeper")
        );
        assert_eq!(soft_canonicalize(&real.join("alias")), real.join("target"));
    }

    #[test]
    fn soft_canonicalize_appends_a_missing_tail_lexically() {
        let dir = tempfile::tempdir().unwrap();
        let real = fs::canonicalize(dir.path()).unwrap();
        assert_eq!(
            soft_canonicalize(&real.join("missing/child")),
            real.join("missing/child")
        );
        assert_eq!(
            soft_canonicalize(&real.join("missing/./child/../other")),
            real.join("missing/other")
        );
    }

    #[test]
    fn soft_canonicalize_never_fails() {
        assert_eq!(
            soft_canonicalize(Path::new("/definitely/not/a/real/path-7f3a")),
            PathBuf::from("/definitely/not/a/real/path-7f3a")
        );
        assert_eq!(soft_canonicalize(Path::new("/")), PathBuf::from("/"));
        assert_eq!(
            soft_canonicalize(Path::new("no-such-relative-dir-7f3a/x/../y")),
            PathBuf::from("no-such-relative-dir-7f3a/y")
        );
    }

    #[test]
    fn soft_canonicalize_makes_aliased_paths_comparable() {
        let dir = tempfile::tempdir().unwrap();
        let real = fs::canonicalize(dir.path()).unwrap();
        fs::create_dir_all(real.join("ws/level-1")).unwrap();
        symlink(real.join("ws"), real.join("tmp-alias")).unwrap();
        // Same directory spelled through the alias, and a not-yet-existing child of it.
        assert_eq!(
            soft_canonicalize(&real.join("tmp-alias/level-1")),
            soft_canonicalize(&real.join("ws/level-1"))
        );
        assert_eq!(
            soft_canonicalize(&real.join("tmp-alias/level-1/new")),
            soft_canonicalize(&real.join("ws/level-1/new"))
        );
    }

    #[test]
    fn same_path_sees_through_aliases_and_missing_tails() {
        let dir = tempfile::tempdir().unwrap();
        let real = fs::canonicalize(dir.path()).unwrap();
        fs::create_dir_all(real.join("ws")).unwrap();
        symlink(real.join("ws"), real.join("alias")).unwrap();
        assert!(same_path(&real.join("ws"), &real.join("alias")));
        assert!(same_path(&real.join("ws/new/x"), &real.join("alias/new/x")));
        assert!(same_path(&real.join("ws/a/../b"), &real.join("alias/b")));
        assert!(!same_path(&real.join("ws"), &real.join("ws/child")));
        assert!(!same_path(&real.join("ws/a"), &real.join("ws/b")));
    }
}
