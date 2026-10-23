//! Conditions on the player's work, always checked (in order) before the answer, so a correct
//! answer next to unfinished work still fails. Declared paths are relative to the section root; a
//! path through a symlink fails with `PATH_ESCAPES_WORKSPACE` instead of being followed.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use super::messages::{self as m, fill, or};
use super::paths::{self, absent};
use super::{CD_DONE, Ctx, PWD_USED, file_text};

#[derive(Clone, Copy)]
enum Text {
    Is(&'static str),
    Has(&'static [&'static str]),
}

#[derive(Clone, Copy)]
enum Bits {
    Is(u32),
    Has(u32),
    Lacks(u32),
}

#[derive(Clone, Copy)]
enum Kind {
    At(&'static str),
    AtHome,
    Exists(&'static str, bool),
    Gone(&'static str),
    Content(&'static str, Text),
    Same(&'static str, &'static str),
    Moved(&'static str, &'static str),
    Mode(&'static str, Bits),
    Marker(&'static str),
}

/// The texts are empty when the default applies.
#[derive(Clone, Copy)]
pub struct Check {
    kind: Kind,
    msg: &'static str,
    missing: &'static str,
}

#[rustfmt::skip]
mod vocabulary {
    use super::*;

    const fn new(kind: Kind) -> Check { Check { kind, msg: "", missing: "" } }

    /// The player stands in this directory (compared canonically).
    pub const fn at(path: &'static str) -> Check { new(Kind::At(path)) }
    pub const fn at_home() -> Check { new(Kind::AtHome) }
    pub const fn is_file(path: &'static str) -> Check { new(Kind::Exists(path, false)) }
    pub const fn is_dir(path: &'static str) -> Check { new(Kind::Exists(path, true)) }
    /// Nothing remains here, whatever it was, not even a dangling symlink.
    pub const fn no_file(path: &'static str) -> Check { new(Kind::Gone(path)) }
    /// The whole content equals `text`; a block, as in `file`, ends with a newline.
    pub const fn content_is(path: &'static str, text: &'static str) -> Check { new(Kind::Content(path, Text::Is(text))) }
    pub const fn contains(path: &'static str, parts: &'static [&'static str]) -> Check { new(Kind::Content(path, Text::Has(parts))) }
    /// `dst` is an exact copy of `src`: names, bytes and dotfiles; symlinks compared, never followed.
    /// `.msg` is the text when it differs, `.missing` when `dst` is missing.
    pub const fn same(src: &'static str, dst: &'static str) -> Check { new(Kind::Same(src, dst)) }
    /// `src` is gone and `dst` exists. `.msg` is the text when `src` is still there, `.missing`
    /// when `dst` is missing.
    pub const fn moved(src: &'static str, dst: &'static str) -> Check { new(Kind::Moved(src, dst)) }
    /// The exact permission bits of a file or a directory.
    pub const fn mode_is(path: &'static str, mode: u32) -> Check { new(Kind::Mode(path, Bits::Is(mode))) }
    /// Every one of these permission bits is set.
    pub const fn mode_has(path: &'static str, bits: u32) -> Check { new(Kind::Mode(path, Bits::Has(bits))) }
    /// None of these permission bits is set.
    pub const fn mode_lacks(path: &'static str, bits: u32) -> Check { new(Kind::Mode(path, Bits::Lacks(bits))) }
    /// The player has run `pwd` (the shell wrapper leaves the evidence).
    pub const fn pwd_used(msg: &'static str) -> Check { new(Kind::Marker(PWD_USED)).msg(msg) }
    /// The level's `cd` drill was done; needs `cd` clauses, which record it.
    pub const fn cd_done(msg: &'static str) -> Check { new(Kind::Marker(CD_DONE)).msg(msg) }

    impl Check {
        /// The text on failure, instead of the default.
        pub const fn msg(self, msg: &'static str) -> Self { Self { msg, ..self } }
        /// The text when the file the check reads (or the copy, destination) does not exist.
        pub const fn missing(self, missing: &'static str) -> Self { Self { missing, ..self } }
    }
}
pub use vocabulary::*;

impl Check {
    #[cfg(test)]
    pub fn paths(&self) -> Vec<&'static str> {
        match self.kind {
            Kind::At(p)
            | Kind::Exists(p, _)
            | Kind::Gone(p)
            | Kind::Content(p, _)
            | Kind::Mode(p, _) => vec![p],
            Kind::Same(a, b) | Kind::Moved(a, b) => vec![a, b],
            Kind::AtHome | Kind::Marker(_) => vec![],
        }
    }

    /// The texts a level wrote for a failure.
    #[cfg(test)]
    pub fn texts(&self) -> Vec<&'static str> {
        [self.msg, self.missing].into_iter().filter(|text| !text.is_empty()).collect()
    }

    /// The directory an `at` check wants the player in.
    #[cfg(test)]
    pub fn target_dir(&self) -> Option<&'static str> {
        match self.kind {
            Kind::At(path) => Some(path),
            _ => None,
        }
    }

    pub fn run(&self, ctx: &Ctx) -> Result<(), String> {
        let msg = |default: String| or(self.msg, default);
        let missing = |default: String| or(self.missing, default);
        match self.kind {
            Kind::At(rel) => {
                let (target, cwd) =
                    (ctx.path(rel, false)?, ctx.cwd.as_ref().ok_or(m::CWD_MISSING)?);
                paths::same(cwd, &target)
                    .then_some(())
                    .ok_or_else(|| msg(m::WRONG_DIRECTORY.into()))
            }
            Kind::AtHome => {
                let cwd = ctx.cwd.as_ref().ok_or(m::CWD_MISSING)?;
                paths::same(cwd, ctx.home)
                    .then_some(())
                    .ok_or_else(|| msg(fill(m::NOT_AT_HOME, cwd.display())))
            }
            Kind::Exists(rel, dir) => find(ctx, rel, dir)?.map(drop).ok_or_else(|| {
                msg(fill([m::FILE_NOT_EXISTS, m::DIR_NOT_EXISTS][usize::from(dir)], rel))
            }),
            Kind::Gone(rel) => match probe(&ctx.path(rel, true)?, false)? {
                None => Ok(()),
                Some(_) => Err(msg(fill(m::FILE_STILL_EXISTS, rel))),
            },
            Kind::Content(rel, want) => {
                let text = read(ctx, rel, self.missing)?;
                let holds = match want {
                    Text::Is(expected) => text == file_text(expected),
                    Text::Has(parts) => parts.iter().all(|part| text.contains(part)),
                };
                holds.then_some(()).ok_or_else(|| msg(m::FILE_CONTENT_MISMATCH.into()))
            }
            Kind::Same(src, dst) => {
                let (from, to) = (ctx.path(src, false)?, ctx.path(dst, false)?);
                if probe(&from, true)?.is_none() {
                    return Err(fill(m::COPY_SOURCE_MISSING, src));
                }
                if probe(&to, true)?.is_none() {
                    return Err(missing(fill(m::DEST_MISSING, dst)));
                }
                match same_tree(&from, &to) {
                    Ok(equal) => {
                        equal.then_some(()).ok_or_else(|| msg(m::COPY_CONTENT_MISMATCH.into()))
                    }
                    Err(error) => Err(fill(m::COMPARE_FAILED, error)),
                }
            }
            Kind::Moved(src, dst) => {
                let (from, to) = (ctx.path(src, false)?, ctx.path(dst, false)?);
                if probe(&from, true)?.is_some() {
                    return Err(msg(fill(m::MOVE_SOURCE_EXISTS, src)));
                }
                probe(&to, true)?.map(drop).ok_or_else(|| missing(fill(m::DEST_MISSING, dst)))
            }
            Kind::Mode(rel, want) => {
                let meta = probe(&ctx.path(rel, false)?, true)?;
                let meta = meta.ok_or_else(|| missing(fill(m::FILE_NOT_EXISTS, rel)))?;
                let actual = meta.mode() & 0o7777;
                let holds = match want {
                    Bits::Is(mode) => actual == mode,
                    Bits::Has(bits) => actual & bits == bits,
                    Bits::Lacks(bits) => actual & bits == 0,
                };
                holds.then_some(()).ok_or_else(|| msg(m::PERMISSION_WRONG.into()))
            }
            Kind::Marker(name) => {
                ctx.marker(name).exists().then_some(()).ok_or_else(|| msg(String::new()))
            }
        }
    }
}

/// What is at `path`: nothing is a normal answer, an unreadable parent is an error to report.
fn probe(path: &Path, follow: bool) -> Result<Option<fs::Metadata>, String> {
    match if follow { fs::metadata(path) } else { fs::symlink_metadata(path) } {
        Ok(meta) => Ok(Some(meta)),
        Err(error) if absent(&error) => Ok(None),
        Err(error) => Err(fill(m::FILESYSTEM_ERROR, error)),
    }
}

/// A regular file or a directory (as asked) at a declared path.
fn find(ctx: &Ctx, rel: &str, dir: bool) -> Result<Option<fs::Metadata>, String> {
    let meta = probe(&ctx.path(rel, false)?, true)?;
    Ok(meta.filter(|meta| meta.is_dir() == dir && (meta.is_dir() || meta.is_file())))
}

fn read(ctx: &Ctx, rel: &str, missing: &str) -> Result<String, String> {
    let path = ctx.path(rel, false)?;
    if !probe(&path, true)?.is_some_and(|meta| meta.is_file()) {
        return Err(or(missing, fill(m::FILE_NOT_EXISTS, rel)));
    }
    let unreadable = |error: &dyn std::fmt::Display| fill(m::FILE_READ_ERROR, error);
    let bytes = fs::read(&path).map_err(|error| unreadable(&error))?;
    String::from_utf8(bytes).map_err(|error| unreadable(&error))
}

/// Deep comparison without following symlinks: a link equals a link with the same target, a file
/// a file with the same bytes, a directory one with the same entry names and equal entries.
fn same_tree(a: &Path, b: &Path) -> io::Result<bool> {
    let (kind_a, kind_b) =
        (fs::symlink_metadata(a)?.file_type(), fs::symlink_metadata(b)?.file_type());
    if kind_a.is_symlink() || kind_b.is_symlink() {
        return Ok(kind_a.is_symlink()
            && kind_b.is_symlink()
            && fs::read_link(a)? == fs::read_link(b)?);
    }
    if kind_a.is_file() || kind_b.is_file() {
        return Ok(kind_a.is_file() && kind_b.is_file() && fs::read(a)? == fs::read(b)?);
    }
    let names = |dir: &Path| -> io::Result<BTreeSet<OsString>> {
        fs::read_dir(dir)?.map(|entry| entry.map(|entry| entry.file_name())).collect()
    };
    if !(kind_a.is_dir() && kind_b.is_dir()) || names(a)? != names(b)? {
        return Ok(false);
    }
    for name in names(a)? {
        if !same_tree(&a.join(&name), &b.join(&name))? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::{PermissionsExt, symlink};

    use super::*;
    use crate::game::testing::{World, err};

    #[test]
    fn at_compares_canonical_directories_and_notices_a_deleted_one() {
        let w = World::new();
        let ctx = |dir: &str| w.ctx(Some(w.root().join(dir)));
        fs::create_dir_all(w.root().join("a/b")).unwrap();
        fs::create_dir_all(w.root().join("other/b")).unwrap();
        assert_eq!(at("a/b").run(&ctx("a/b")), Ok(()));
        assert_eq!(
            at("a/b").run(&ctx("other/b")),
            err(m::WRONG_DIRECTORY),
            "a same-named directory elsewhere"
        );
        assert_eq!(at("a/b").msg("custom").run(&ctx("")), err("custom"));
        assert_eq!(at("a/b").run(&w.ctx(None)), err(m::CWD_MISSING));
        assert_eq!(at_home().run(&w.ctx(Some(w.home.clone()))), Ok(()));
        assert!(at_home().run(&ctx("")).unwrap_err().contains(&w.root().display().to_string()));
        assert_eq!(at_home().run(&w.ctx(None)), err(m::CWD_MISSING));
    }

    #[test]
    fn a_path_through_a_symlink_fails_and_gone_means_nothing_remains() {
        let w = World::new();
        w.put("real/file.txt", "x");
        symlink(w.root().join("real"), w.root().join("link")).unwrap();
        for check in
            [is_file("link/file.txt"), content_is("link/file.txt", "x"), no_file("link/file.txt")]
        {
            assert_eq!(w.run(check), err(m::PATH_ESCAPES_WORKSPACE));
        }
        assert!(w.run(is_file("real/file.txt")).is_ok() && w.run(is_dir("real")).is_ok());
        assert_eq!(w.run(is_file("real")), err("Soubor 'real' neexistuje"));
        assert_eq!(w.run(no_file("real/file.txt")), err("Soubor 'real/file.txt' stále existuje"));
        assert!(w.run(no_file("real")).is_err(), "a directory is something too");
        symlink(w.root().join("nowhere"), w.root().join("dangling")).unwrap();
        assert!(w.run(no_file("dangling")).is_err());
        fs::remove_file(w.root().join("dangling")).unwrap();
        assert!(w.run(no_file("dangling")).is_ok() && w.run(no_file("never-was")).is_ok());
    }

    #[test]
    fn content_checks_are_exact_and_explain() {
        let w = World::new();
        w.put("log.txt", "one\ntwo\n");
        assert_eq!(w.run(content_is("log.txt", "one\ntwo\n")), Ok(()));
        // A block expectation ends with a newline like the file.
        let block = r#"
            one
            two
        "#;
        assert_eq!(w.run(content_is("log.txt", block)), Ok(()));
        for wrong in ["one\ntwo", "one\n", "one\ntwo\n\n", " one\ntwo\n"] {
            assert_eq!(w.run(content_is("log.txt", wrong)), err(m::FILE_CONTENT_MISMATCH));
        }
        assert_eq!(w.run(contains("log.txt", &["two", "one"])), Ok(()));
        assert_eq!(w.run(contains("log.txt", &["three"]).msg("lacks")), err("lacks"));
        assert_eq!(w.run(contains("none.txt", &["x"]).missing("write it")), err("write it"));
        fs::write(w.root().join("binary"), [0xff, 0xfe]).unwrap();
        assert!(w.run(contains("binary", &["x"])).unwrap_err().starts_with("Nelze přečíst soubor"));
    }

    #[test]
    fn same_compares_trees_without_following_links() {
        let w = World::new();
        for side in ["src", "dst"] {
            for (name, content) in [("a.txt", "a"), (".hidden", "h"), ("sub/b.txt", "b")] {
                w.put(&format!("{side}/{name}"), content);
            }
        }
        symlink("a.txt", w.root().join("src/link")).unwrap();
        symlink("..", w.root().join("src/up")).unwrap();
        assert!(w.run(same("src", "dst")).is_err(), "links only on one side");
        symlink("a.txt", w.root().join("dst/link")).unwrap();
        symlink("..", w.root().join("dst/up")).unwrap();
        assert_eq!(w.run(same("src", "dst")), Ok(()));
        fs::write(w.root().join("dst/.hidden"), "changed").unwrap();
        assert_eq!(w.run(same("src", "dst").msg("differs")), err("differs"));
        assert_eq!(w.run(same("nope", "dst")), err(&fill(m::COPY_SOURCE_MISSING, "nope")));
        assert_eq!(w.run(same("src", "nope")), err("Cílový soubor 'nope' neexistuje."));
        assert_eq!(w.run(same("src", "nope").msg("differs").missing("copy it")), err("copy it"));
    }

    #[test]
    fn moved_and_modes() {
        let w = World::new();
        w.put("old.txt", "x");
        assert_eq!(w.run(moved("old.txt", "new.txt").msg("still there")), err("still there"));
        fs::rename(w.root().join("old.txt"), w.root().join("new.txt")).unwrap();
        assert_eq!(w.run(moved("old.txt", "new.txt")), Ok(()));
        let lost = moved("old.txt", "other.txt").msg("still there").missing("lost");
        assert_eq!(w.run(lost), err("lost"));
        let set = |name: &str, mode| {
            fs::set_permissions(w.root().join(name), fs::Permissions::from_mode(mode)).unwrap()
        };
        fs::create_dir(w.root().join("dir")).unwrap();
        set("new.txt", 0o754);
        set("dir", 0o750);
        let mode = |check| w.run(check).is_ok();
        assert!(
            mode(mode_is("new.txt", 0o754))
                && mode(mode_has("new.txt", 0o100))
                && mode(mode_lacks("new.txt", 0o003))
                && mode(mode_is("dir", 0o750))
        );
        assert!(
            !mode(mode_is("new.txt", 0o755))
                && !mode(mode_has("new.txt", 0o002))
                && !mode(mode_lacks("new.txt", 0o004))
                && !mode(mode_is("dir", 0o755))
        );
        assert_eq!(w.run(mode_is("nothing", 0o755)), err("Soubor 'nothing' neexistuje"));
        assert_eq!(w.run(mode_is("nothing", 0o755).missing("none")), err("none"));
    }
}
