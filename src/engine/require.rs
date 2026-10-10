//! Requirements: conditions on the player's work that are *always* checked.
//!
//! A level's requirements run before its answer, unconditionally: a correct answer next to
//! a failed requirement is still a failure. That is what makes an answer impossible to
//! shortcut (submitting `level-1` without having used `pwd` fails), and what lets a level
//! with no answer at all (a pure `chmod` task) be graded on the filesystem alone.
//!
//! Each constructor ([`at`], [`file()`], [`content`], ...) returns a small builder with the
//! message overrides that make sense for it; every builder converts into a [`Requirement`],
//! which is what `Level::require` takes. A requirement's *success* text is never shown, so
//! none is kept.
//!
//! All declared paths are relative to the level's section root and go through the one
//! containment function. A path that passes through a symlink fails with
//! [`messages::PATH_ESCAPES_WORKSPACE`] rather than being followed.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::engine::pytext::{py_splitlines_count, py_strip, universal_newlines};
use crate::engine::{Check, Ctx};
use crate::markers::Marker;
use crate::messages;
use crate::paths::{resolve_within, soft_canonicalize};

const MAX_MODE: u32 = 0o7777;

/// A condition on the player's work. Build one with the free functions of this module.
#[derive(Debug, Clone)]
pub struct Requirement {
    kind: Kind,
}

#[derive(Debug, Clone)]
enum Kind {
    At(At),
    AtHome,
    Evidence { marker: Marker, message: String },
    Presence(Presence),
    Content(FileContent),
    LineCount(LineCount),
    Same(Same),
    Moved(Moved),
    PermBits(PermBits),
    ModeIs(ModeIs),
    Custom(fn(&Ctx) -> Check),
}

macro_rules! into_requirement {
    ($($builder:ident => $variant:ident),+ $(,)?) => {
        $(impl From<$builder> for Requirement {
            fn from(builder: $builder) -> Self {
                Requirement { kind: Kind::$variant(builder) }
            }
        })+
    };
}

into_requirement! {
    At => At,
    Presence => Presence,
    FileContent => Content,
    LineCount => LineCount,
    Same => Same,
    Moved => Moved,
    PermBits => PermBits,
    ModeIs => ModeIs,
}

/// The player must be standing in this section-relative directory.
///
/// Compares *canonical* paths of the player's directory and of the target, so a directory
/// of the same name elsewhere never matches. `at("")` is the section root.
pub fn at(path: impl Into<String>) -> At {
    At {
        path: path.into(),
        message: None,
    }
}

/// The player must be standing in `$HOME` (compared canonically).
pub fn at_home() -> Requirement {
    Requirement { kind: Kind::AtHome }
}

/// The marker file must exist: the player did something only the shell could witness.
pub fn evidence(marker: Marker, message: impl Into<String>) -> Requirement {
    Requirement {
        kind: Kind::Evidence {
            marker,
            message: message.into(),
        },
    }
}

/// This level's own `cd` drill must have been performed ([`Marker::Cd`]). Needs a `cd`
/// policy on the level, which is what writes the marker.
pub fn cd_evidence(message: impl Into<String>) -> Requirement {
    evidence(Marker::Cd, message)
}

/// This level's own file-descriptor drill must have been performed ([`Marker::Fd`]).
/// Needs an `on_fd` rule on the level.
pub fn fd_evidence(message: impl Into<String>) -> Requirement {
    evidence(Marker::Fd, message)
}

/// A regular file must exist here.
pub fn file(path: impl Into<String>) -> Presence {
    Presence::new(path.into(), false, true)
}

/// A directory must exist here.
pub fn dir(path: impl Into<String>) -> Presence {
    Presence::new(path.into(), true, true)
}

/// Nothing may remain here (a leftover symlink, even a dangling one, counts as
/// remaining). The messages talk about a file.
pub fn no_file(path: impl Into<String>) -> Presence {
    Presence::new(path.into(), false, false)
}

/// Nothing may remain here (a leftover symlink, even a dangling one, counts as
/// remaining). The messages talk about a directory.
pub fn no_dir(path: impl Into<String>) -> Presence {
    Presence::new(path.into(), true, false)
}

/// Conditions on a text file's content.
pub fn content(path: impl Into<String>) -> FileContent {
    FileContent {
        path: path.into(),
        exact: None,
        contains: Vec::new(),
        excludes: Vec::new(),
        strip: false,
        message: None,
        missing: None,
        unreadable: None,
    }
}

/// The file must have exactly `expected` lines, counted like Python's `str.splitlines()`.
pub fn line_count(
    path: impl Into<String>,
    expected: usize,
    message: impl Into<String>,
) -> LineCount {
    LineCount {
        path: path.into(),
        expected,
        message: message.into(),
        missing: None,
    }
}

/// `destination` must be an exact copy of `source`: same names, same bytes, dotfiles
/// included, symlinks compared by their link target and never followed.
pub fn same(source: impl Into<String>, destination: impl Into<String>) -> Same {
    Same {
        source: source.into(),
        destination: destination.into(),
        message: None,
        source_error: None,
        destination_error: None,
    }
}

/// `source` must be gone and `destination` must exist.
pub fn moved(source: impl Into<String>, destination: impl Into<String>) -> Moved {
    Moved {
        source: source.into(),
        destination: destination.into(),
        source_error: None,
        destination_error: None,
    }
}

/// Permission bits a file must have and must not have.
pub fn perm_bits(path: impl Into<String>) -> PermBits {
    PermBits {
        path: path.into(),
        has: 0,
        lacks: 0,
        message: None,
    }
}

/// A file's permission bits must be exactly `mode`.
pub fn mode(path: impl Into<String>, mode: u32) -> ModeIs {
    ModeIs {
        path: path.into(),
        mode,
        directory: false,
        message: None,
    }
}

/// A directory's permission bits must be exactly `mode`.
pub fn dir_mode(path: impl Into<String>, mode: u32) -> ModeIs {
    ModeIs {
        path: path.into(),
        mode,
        directory: true,
        message: None,
    }
}

/// An arbitrary check, for the rare condition nothing else expresses. Prefer the
/// declarative requirements: they are checked by the catalog lint, this is not.
pub fn check(rule: fn(&Ctx) -> Check) -> Requirement {
    Requirement {
        kind: Kind::Custom(rule),
    }
}

impl Requirement {
    /// Check the requirement against the player's current situation.
    pub fn check(&self, ctx: &Ctx) -> Check {
        match &self.kind {
            Kind::At(requirement) => requirement.check(ctx),
            Kind::AtHome => check_at_home(ctx),
            Kind::Evidence { marker, message } => {
                if ctx.markers().exists(marker) {
                    Check::Pass
                } else {
                    Check::fail(message.clone())
                }
            }
            Kind::Presence(requirement) => requirement.check(ctx),
            Kind::Content(requirement) => requirement.check(ctx),
            Kind::LineCount(requirement) => requirement.check(ctx),
            Kind::Same(requirement) => requirement.check(ctx),
            Kind::Moved(requirement) => requirement.check(ctx),
            Kind::PermBits(requirement) => requirement.check(ctx),
            Kind::ModeIs(requirement) => requirement.check(ctx),
            Kind::Custom(rule) => rule(ctx),
        }
    }

    /// The marker this requirement reads, if it is evidence.
    pub fn marker(&self) -> Option<Marker> {
        match &self.kind {
            Kind::Evidence { marker, .. } => Some(*marker),
            _ => None,
        }
    }

    /// Every section-relative path this requirement declares, as written.
    pub fn paths(&self) -> Vec<&str> {
        match &self.kind {
            Kind::At(r) => vec![&r.path],
            Kind::Presence(r) => vec![&r.path],
            Kind::Content(r) => vec![&r.path],
            Kind::LineCount(r) => vec![&r.path],
            Kind::Same(r) => vec![&r.source, &r.destination],
            Kind::Moved(r) => vec![&r.source, &r.destination],
            Kind::PermBits(r) => vec![&r.path],
            Kind::ModeIs(r) => vec![&r.path],
            Kind::AtHome | Kind::Evidence { .. } | Kind::Custom(_) => Vec::new(),
        }
        .into_iter()
        .map(String::as_str)
        .collect()
    }

    /// For [`at`] with a custom message: the declared directory and the message, so the
    /// catalog lint can check the message does not name the target.
    pub fn custom_at_message(&self) -> Option<(&str, &str)> {
        match &self.kind {
            Kind::At(At {
                path,
                message: Some(message),
            }) => Some((path, message)),
            _ => None,
        }
    }

    /// Problems with how the requirement was built (modes out of range, contradictory
    /// permission bits).
    pub fn lint(&self) -> Vec<String> {
        let mut problems = Vec::new();
        match &self.kind {
            Kind::PermBits(r) => {
                if r.has & r.lacks != 0 {
                    problems.push(format!(
                        "perm_bits({:?}) requires and forbids the same bits",
                        r.path
                    ));
                }
                if r.has > MAX_MODE || r.lacks > MAX_MODE {
                    problems.push(format!("perm_bits({:?}) has bits outside 0o7777", r.path));
                }
            }
            Kind::ModeIs(r) if r.mode > MAX_MODE => {
                problems.push(format!(
                    "mode {:#o} for {:?} is outside 0o0000..=0o7777",
                    r.mode, r.path
                ));
            }
            _ => {}
        }
        problems
    }
}

fn escapes() -> Check {
    Check::fail(messages::PATH_ESCAPES_WORKSPACE)
}

/// What is at a path, without ever failing: "nothing there" is a normal answer.
enum Probe {
    Absent,
    Present(fs::Metadata),
    Failed(io::Error),
}

impl Probe {
    /// Inspect `path`, following symlinks when `follow` is set. A missing path or a
    /// missing parent is `Absent`; any other error (such as a parent the player made
    /// unreadable) is `Failed`, because we cannot say what is there.
    fn of(path: &Path, follow: bool) -> Self {
        let result = if follow {
            fs::metadata(path)
        } else {
            fs::symlink_metadata(path)
        };
        match result {
            Ok(meta) => Self::Present(meta),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
                ) =>
            {
                Self::Absent
            }
            Err(error) => Self::Failed(error),
        }
    }
}

fn filesystem_error(error: &io::Error) -> Check {
    Check::fail(messages::with_error(messages::FILESYSTEM_ERROR, error))
}

fn check_at_home(ctx: &Ctx) -> Check {
    let Some(cwd) = &ctx.cwd else {
        return Check::fail(messages::CWD_MISSING);
    };
    if soft_canonicalize(cwd) == soft_canonicalize(ctx.home) {
        Check::Pass
    } else {
        Check::fail(messages::not_at_home(cwd.display()))
    }
}

/// See [`at`].
#[derive(Debug, Clone)]
pub struct At {
    path: String,
    message: Option<String>,
}

impl At {
    /// Shown instead of [`messages::WRONG_DIRECTORY`]. It must not name the target
    /// directory (the catalog lint checks).
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    fn check(&self, ctx: &Ctx) -> Check {
        let Ok(target) = ctx.path(&self.path) else {
            return escapes();
        };
        let Some(cwd) = &ctx.cwd else {
            return Check::fail(messages::CWD_MISSING);
        };
        if soft_canonicalize(cwd) == soft_canonicalize(&target) {
            Check::Pass
        } else {
            Check::fail(
                self.message
                    .clone()
                    .unwrap_or_else(|| messages::WRONG_DIRECTORY.to_owned()),
            )
        }
    }
}

/// See [`file()`], [`dir`], [`no_file`] and [`no_dir`].
#[derive(Debug, Clone)]
pub struct Presence {
    path: String,
    directory: bool,
    should_exist: bool,
    message: Option<String>,
}

impl Presence {
    fn new(path: String, directory: bool, should_exist: bool) -> Self {
        Self {
            path,
            directory,
            should_exist,
            message: None,
        }
    }

    /// Shown instead of the default "missing" / "still exists" text.
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    fn failure(&self, default: &str) -> Check {
        Check::fail(
            self.message
                .clone()
                .unwrap_or_else(|| messages::with_path(default, &self.path)),
        )
    }

    fn check(&self, ctx: &Ctx) -> Check {
        // When something must be gone, a symlink left at the path is a leftover to report
        // ("still exists"), not an escape.
        let Ok(target) = resolve_within(&ctx.section_root, &self.path, !self.should_exist) else {
            return escapes();
        };
        if self.should_exist {
            match Probe::of(&target, true) {
                Probe::Present(meta)
                    if meta.is_dir() == self.directory && (meta.is_dir() || meta.is_file()) =>
                {
                    Check::Pass
                }
                Probe::Present(_) | Probe::Absent => self.failure(if self.directory {
                    messages::DIR_NOT_EXISTS
                } else {
                    messages::FILE_NOT_EXISTS
                }),
                Probe::Failed(error) => filesystem_error(&error),
            }
        } else {
            match Probe::of(&target, false) {
                Probe::Absent => Check::Pass,
                Probe::Present(_) => self.failure(if self.directory {
                    messages::DIR_STILL_EXISTS
                } else {
                    messages::FILE_STILL_EXISTS
                }),
                Probe::Failed(error) => filesystem_error(&error),
            }
        }
    }
}

/// See [`content`].
#[derive(Debug, Clone)]
pub struct FileContent {
    path: String,
    exact: Option<String>,
    contains: Vec<String>,
    excludes: Vec<String>,
    strip: bool,
    message: Option<String>,
    missing: Option<String>,
    unreadable: Option<String>,
}

impl FileContent {
    /// The whole content must equal this text.
    #[must_use]
    pub fn exact(mut self, expected: impl Into<String>) -> Self {
        self.exact = Some(expected.into());
        self
    }

    /// The content must contain every one of these substrings.
    #[must_use]
    pub fn contains<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.contains.extend(values.into_iter().map(Into::into));
        self
    }

    /// The content must contain none of these substrings.
    #[must_use]
    pub fn excludes<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.excludes.extend(values.into_iter().map(Into::into));
        self
    }

    /// Strip surrounding whitespace from the content before comparing (Python's
    /// `str.strip`).
    #[must_use]
    pub fn strip(mut self) -> Self {
        self.strip = true;
        self
    }

    /// Shown when the content does not match, instead of
    /// [`messages::FILE_CONTENT_MISMATCH`].
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Shown when the file does not exist, instead of [`messages::FILE_NOT_EXISTS`].
    #[must_use]
    pub fn missing(mut self, message: impl Into<String>) -> Self {
        self.missing = Some(message.into());
        self
    }

    /// Shown when the file cannot be read (permissions, not UTF-8), instead of
    /// [`messages::FILE_READ_ERROR`].
    #[must_use]
    pub fn unreadable(mut self, message: impl Into<String>) -> Self {
        self.unreadable = Some(message.into());
        self
    }

    fn check(&self, ctx: &Ctx) -> Check {
        let raw = match read_text(
            ctx,
            &self.path,
            self.missing.as_deref(),
            self.unreadable.as_deref(),
        ) {
            Ok(text) => text,
            Err(failure) => return failure,
        };
        // Python reads in text mode, which turns \r\n and \r into \n.
        let text = universal_newlines(&raw);
        let text: &str = if self.strip { py_strip(&text) } else { &text };
        let mismatch = self
            .exact
            .as_deref()
            .is_some_and(|expected| text != expected)
            || self
                .contains
                .iter()
                .any(|value| !text.contains(value.as_str()))
            || self
                .excludes
                .iter()
                .any(|value| text.contains(value.as_str()));
        if mismatch {
            Check::fail(
                self.message
                    .clone()
                    .unwrap_or_else(|| messages::FILE_CONTENT_MISMATCH.to_owned()),
            )
        } else {
            Check::Pass
        }
    }
}

/// Read a section-relative file as UTF-8 text, turning every way that can fail into the
/// failure a player should see.
fn read_text(
    ctx: &Ctx,
    path: &str,
    missing: Option<&str>,
    unreadable: Option<&str>,
) -> Result<String, Check> {
    let target = ctx.path(path).map_err(|_| escapes())?;
    let missing_check = || {
        Check::fail(
            missing
                .map(str::to_owned)
                .unwrap_or_else(|| messages::with_path(messages::FILE_NOT_EXISTS, path)),
        )
    };
    match Probe::of(&target, true) {
        Probe::Present(meta) if meta.is_file() => {}
        Probe::Present(_) | Probe::Absent => return Err(missing_check()),
        Probe::Failed(error) => return Err(filesystem_error(&error)),
    }
    let unreadable_check = |error: &dyn std::fmt::Display| {
        Check::fail(
            unreadable
                .map(str::to_owned)
                .unwrap_or_else(|| messages::with_error(messages::FILE_READ_ERROR, error)),
        )
    };
    let bytes = fs::read(&target).map_err(|error| unreadable_check(&error))?;
    String::from_utf8(bytes).map_err(|error| unreadable_check(&error))
}

/// See [`line_count`].
#[derive(Debug, Clone)]
pub struct LineCount {
    path: String,
    expected: usize,
    message: String,
    missing: Option<String>,
}

impl LineCount {
    /// Shown when the file does not exist, instead of [`messages::FILE_NOT_EXISTS`].
    #[must_use]
    pub fn missing(mut self, message: impl Into<String>) -> Self {
        self.missing = Some(message.into());
        self
    }

    fn check(&self, ctx: &Ctx) -> Check {
        match read_text(ctx, &self.path, self.missing.as_deref(), None) {
            Ok(text) if py_splitlines_count(&text) == self.expected => Check::Pass,
            Ok(_) => Check::fail(self.message.clone()),
            Err(failure) => failure,
        }
    }
}

/// See [`same`].
#[derive(Debug, Clone)]
pub struct Same {
    source: String,
    destination: String,
    message: Option<String>,
    source_error: Option<String>,
    destination_error: Option<String>,
}

impl Same {
    /// Shown when the two differ, instead of [`messages::COPY_CONTENT_MISMATCH`].
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Shown when the source is gone, instead of [`messages::COPY_SOURCE_MISSING`].
    #[must_use]
    pub fn source_error(mut self, message: impl Into<String>) -> Self {
        self.source_error = Some(message.into());
        self
    }

    /// Shown when the destination is missing, instead of [`messages::COPY_DEST_MISSING`].
    #[must_use]
    pub fn destination_error(mut self, message: impl Into<String>) -> Self {
        self.destination_error = Some(message.into());
        self
    }

    fn check(&self, ctx: &Ctx) -> Check {
        let (Ok(source), Ok(destination)) = (ctx.path(&self.source), ctx.path(&self.destination))
        else {
            return escapes();
        };
        for (path, declared, custom, default) in [
            (
                &source,
                &self.source,
                &self.source_error,
                messages::COPY_SOURCE_MISSING,
            ),
            (
                &destination,
                &self.destination,
                &self.destination_error,
                messages::COPY_DEST_MISSING,
            ),
        ] {
            match Probe::of(path, true) {
                Probe::Present(_) => {}
                Probe::Absent => {
                    return Check::fail(
                        custom
                            .clone()
                            .unwrap_or_else(|| messages::with_path(default, declared)),
                    );
                }
                Probe::Failed(error) => return filesystem_error(&error),
            }
        }
        match paths_match(&source, &destination) {
            Ok(true) => Check::Pass,
            Ok(false) => Check::fail(
                self.message
                    .clone()
                    .unwrap_or_else(|| messages::COPY_CONTENT_MISMATCH.to_owned()),
            ),
            Err(error) => Check::fail(messages::with_error(messages::COMPARE_FAILED, error)),
        }
    }
}

/// Deep comparison of two paths: a symlink equals another symlink with the same target, a
/// file equals a file with the same bytes, and a directory equals a directory with the
/// same entry names (dotfiles included) whose entries are pairwise equal. Symlinks are
/// compared, never followed, so a link to a parent cannot make the walk loop.
fn paths_match(a: &Path, b: &Path) -> io::Result<bool> {
    let (meta_a, meta_b) = (fs::symlink_metadata(a)?, fs::symlink_metadata(b)?);
    let (kind_a, kind_b) = (meta_a.file_type(), meta_b.file_type());
    if kind_a.is_symlink() || kind_b.is_symlink() {
        return Ok(kind_a.is_symlink()
            && kind_b.is_symlink()
            && fs::read_link(a)? == fs::read_link(b)?);
    }
    if kind_a.is_file() || kind_b.is_file() {
        return Ok(kind_a.is_file()
            && kind_b.is_file()
            && meta_a.len() == meta_b.len()
            && fs::read(a)? == fs::read(b)?);
    }
    if !kind_a.is_dir() || !kind_b.is_dir() {
        return Ok(false);
    }
    let names_a = entry_names(a)?;
    if names_a != entry_names(b)? {
        return Ok(false);
    }
    for name in names_a {
        if !paths_match(&a.join(&name), &b.join(&name))? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn entry_names(dir: &Path) -> io::Result<BTreeSet<OsString>> {
    fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect()
}

/// See [`moved`].
#[derive(Debug, Clone)]
pub struct Moved {
    source: String,
    destination: String,
    source_error: Option<String>,
    destination_error: Option<String>,
}

impl Moved {
    /// Shown when the source still exists, instead of [`messages::MOVE_SOURCE_EXISTS`].
    #[must_use]
    pub fn source_error(mut self, message: impl Into<String>) -> Self {
        self.source_error = Some(message.into());
        self
    }

    /// Shown when the destination is missing, instead of [`messages::MOVE_DEST_MISSING`].
    #[must_use]
    pub fn destination_error(mut self, message: impl Into<String>) -> Self {
        self.destination_error = Some(message.into());
        self
    }

    fn check(&self, ctx: &Ctx) -> Check {
        let (Ok(source), Ok(destination)) = (ctx.path(&self.source), ctx.path(&self.destination))
        else {
            return escapes();
        };
        match Probe::of(&source, true) {
            Probe::Absent => {}
            Probe::Present(_) => {
                return Check::fail(self.source_error.clone().unwrap_or_else(|| {
                    messages::with_path(messages::MOVE_SOURCE_EXISTS, &self.source)
                }));
            }
            Probe::Failed(error) => return filesystem_error(&error),
        }
        match Probe::of(&destination, true) {
            Probe::Present(_) => Check::Pass,
            Probe::Absent => Check::fail(self.destination_error.clone().unwrap_or_else(|| {
                messages::with_path(messages::MOVE_DEST_MISSING, &self.destination)
            })),
            Probe::Failed(error) => filesystem_error(&error),
        }
    }
}

/// See [`perm_bits`].
#[derive(Debug, Clone)]
pub struct PermBits {
    path: String,
    has: u32,
    lacks: u32,
    message: Option<String>,
}

impl PermBits {
    /// Every one of these permission bits must be set (such as `0o100`, owner execute).
    #[must_use]
    pub fn has(mut self, bits: u32) -> Self {
        self.has |= bits;
        self
    }

    /// None of these permission bits may be set (such as `0o222`, all write bits).
    #[must_use]
    pub fn lacks(mut self, bits: u32) -> Self {
        self.lacks |= bits;
        self
    }

    /// Shown when the bits are wrong, instead of [`messages::PERMISSION_WRONG`].
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    fn check(&self, ctx: &Ctx) -> Check {
        match permission_bits(ctx, &self.path, false) {
            Ok(actual) if actual & self.has == self.has && actual & self.lacks == 0 => Check::Pass,
            Ok(_) => Check::fail(
                self.message
                    .clone()
                    .unwrap_or_else(|| messages::PERMISSION_WRONG.to_owned()),
            ),
            Err(failure) => failure,
        }
    }
}

/// See [`mode`] and [`dir_mode`].
#[derive(Debug, Clone)]
pub struct ModeIs {
    path: String,
    mode: u32,
    directory: bool,
    message: Option<String>,
}

impl ModeIs {
    /// Shown when the mode is wrong, instead of [`messages::PERMISSION_WRONG`].
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    fn check(&self, ctx: &Ctx) -> Check {
        match permission_bits(ctx, &self.path, self.directory) {
            Ok(actual) if actual == self.mode => Check::Pass,
            Ok(_) => Check::fail(
                self.message
                    .clone()
                    .unwrap_or_else(|| messages::PERMISSION_WRONG.to_owned()),
            ),
            Err(failure) => failure,
        }
    }
}

/// The permission bits (`0o7777` mask) of a file or directory, or the failure to show when
/// it is missing.
fn permission_bits(ctx: &Ctx, path: &str, directory: bool) -> Result<u32, Check> {
    let target = ctx.path(path).map_err(|_| escapes())?;
    match Probe::of(&target, true) {
        Probe::Present(meta) if meta.is_dir() == directory && (meta.is_dir() || meta.is_file()) => {
            Ok(meta.mode() & MAX_MODE)
        }
        Probe::Present(_) | Probe::Absent => Err(Check::fail(messages::with_path(
            if directory {
                messages::DIR_NOT_EXISTS
            } else {
                messages::FILE_NOT_EXISTS
            },
            path,
        ))),
        Probe::Failed(error) => Err(filesystem_error(&error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::LevelId;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::path::PathBuf;

    /// A workspace with one section (`level-1`) and a home directory, all canonical.
    struct Env {
        _dir: tempfile::TempDir,
        workspace: PathBuf,
        root: PathBuf,
        home: PathBuf,
    }

    impl Env {
        fn new() -> Self {
            let dir = tempfile::tempdir().unwrap();
            let base = fs::canonicalize(dir.path()).unwrap();
            let workspace = base.join("ws");
            let root = workspace.join("level-1");
            let home = base.join("home/ada");
            fs::create_dir_all(&root).unwrap();
            fs::create_dir_all(&home).unwrap();
            Self {
                _dir: dir,
                workspace,
                root,
                home,
            }
        }

        fn ctx(&self) -> Ctx<'_> {
            Ctx::new(
                &self.workspace,
                self.root.clone(),
                Some(self.root.clone()),
                &self.home,
                LevelId::new(1, 4),
            )
        }

        fn ctx_at(&self, cwd: &Path) -> Ctx<'_> {
            Ctx {
                cwd: Some(cwd.to_path_buf()),
                ..self.ctx()
            }
        }

        fn write(&self, rel: &str, text: &str) -> PathBuf {
            let path = self.root.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, text).unwrap();
            path
        }

        fn mkdir(&self, rel: &str) -> PathBuf {
            let path = self.root.join(rel);
            fs::create_dir_all(&path).unwrap();
            path
        }
    }

    fn run(requirement: impl Into<Requirement>, ctx: &Ctx) -> Check {
        requirement.into().check(ctx)
    }

    #[test]
    fn at_passes_in_the_target_directory() {
        let env = Env::new();
        let target = env.mkdir("gamma/deep");
        assert_eq!(run(at("gamma/deep"), &env.ctx_at(&target)), Check::Pass);
        assert_eq!(run(at(""), &env.ctx()), Check::Pass, "the section root");
        assert_eq!(run(at("."), &env.ctx()), Check::Pass);
    }

    #[test]
    fn at_fails_elsewhere_with_the_default_or_a_custom_message() {
        let env = Env::new();
        env.mkdir("gamma/deep");
        assert_eq!(
            run(at("gamma/deep"), &env.ctx()),
            Check::fail(messages::WRONG_DIRECTORY)
        );
        assert_eq!(
            run(at("gamma/deep").message("Ještě tam nejste."), &env.ctx()),
            Check::fail("Ještě tam nejste.")
        );
    }

    #[test]
    fn at_compares_canonical_paths_so_a_same_named_directory_elsewhere_never_matches() {
        let env = Env::new();
        env.mkdir("alpha");
        let elsewhere = env.workspace.join("other/alpha");
        fs::create_dir_all(&elsewhere).unwrap();
        let inside_home = env.home.join("alpha");
        fs::create_dir_all(&inside_home).unwrap();
        for wrong in [&elsewhere, &inside_home, &env.workspace.join("level-2")] {
            assert_eq!(
                run(at("alpha"), &env.ctx_at(wrong)),
                Check::fail(messages::WRONG_DIRECTORY),
                "{wrong:?}"
            );
        }
        // The same directory reached through a symlinked alias still matches.
        let alias = env.workspace.join("alias");
        symlink(env.root.join("alpha"), &alias).unwrap();
        assert_eq!(run(at("alpha"), &env.ctx_at(&alias)), Check::Pass);
    }

    #[test]
    fn at_reports_a_deleted_working_directory() {
        let env = Env::new();
        let ctx = Ctx {
            cwd: None,
            ..env.ctx()
        };
        assert_eq!(run(at(""), &ctx), Check::fail(messages::CWD_MISSING));
        assert_eq!(run(at_home(), &ctx), Check::fail(messages::CWD_MISSING));
    }

    #[test]
    fn at_works_for_a_target_that_does_not_exist_yet() {
        let env = Env::new();
        // The player is "in" the section root; the target is a not-yet-created child.
        assert_eq!(
            run(at("not/yet"), &env.ctx()),
            Check::fail(messages::WRONG_DIRECTORY)
        );
    }

    #[test]
    fn at_refuses_a_symlinked_target_path() {
        let env = Env::new();
        let outside = tempfile::tempdir().unwrap();
        let outside_path = fs::canonicalize(outside.path()).unwrap();
        symlink(&outside_path, env.root.join("link")).unwrap();
        assert_eq!(
            run(at("link"), &env.ctx_at(&outside_path)),
            Check::fail(messages::PATH_ESCAPES_WORKSPACE)
        );
        assert_eq!(
            run(at("../elsewhere"), &env.ctx()),
            Check::fail(messages::PATH_ESCAPES_WORKSPACE)
        );
    }

    #[test]
    fn at_checks_containment_before_the_working_directory() {
        let env = Env::new();
        let ctx = Ctx {
            cwd: None,
            ..env.ctx()
        };
        assert_eq!(
            run(at("../x"), &ctx),
            Check::fail(messages::PATH_ESCAPES_WORKSPACE)
        );
    }

    #[test]
    fn at_home_compares_against_the_canonical_home() {
        let env = Env::new();
        assert_eq!(run(at_home(), &env.ctx_at(&env.home)), Check::Pass);
        let message = match run(at_home(), &env.ctx()) {
            Check::Fail(message) => message,
            Check::Pass => panic!("should fail"),
        };
        assert_eq!(
            message,
            messages::not_at_home(env.root.display()),
            "the message names where the player is"
        );
        assert!(message.starts_with("Nejste doma. Jste v: "));

        // $HOME given through a symlink still matches the canonical cwd.
        let alias = env.workspace.join("home-alias");
        symlink(&env.home, &alias).unwrap();
        let ctx = Ctx {
            home: &alias,
            cwd: Some(env.home.clone()),
            ..env.ctx()
        };
        assert_eq!(run(at_home(), &ctx), Check::Pass);
    }

    #[test]
    fn evidence_needs_the_marker() {
        let env = Env::new();
        let requirement = evidence(Marker::Pwd, "Nejdřív použijte pwd.");
        assert_eq!(
            run(requirement.clone(), &env.ctx()),
            Check::fail("Nejdřív použijte pwd.")
        );
        env.ctx().markers().create(&Marker::Pwd).unwrap();
        assert_eq!(run(requirement, &env.ctx()), Check::Pass);
    }

    #[test]
    fn cd_and_fd_evidence_use_the_levels_own_markers() {
        let env = Env::new();
        let cd = cd_evidence("cd chybí");
        let fd = fd_evidence("fd chybí");
        assert_eq!(cd.marker(), Some(Marker::Cd));
        assert_eq!(fd.marker(), Some(Marker::Fd));
        assert_eq!(run(cd.clone(), &env.ctx()), Check::fail("cd chybí"));
        assert_eq!(run(fd.clone(), &env.ctx()), Check::fail("fd chybí"));
        fs::write(env.workspace.join(".cd-1.4"), "").unwrap();
        assert_eq!(run(cd, &env.ctx()), Check::Pass);
        assert_eq!(run(fd.clone(), &env.ctx()), Check::fail("fd chybí"));
        fs::write(env.workspace.join(".fd-1.4"), "").unwrap();
        assert_eq!(run(fd, &env.ctx()), Check::Pass);
        // Another level's marker does not count.
        let other = Ctx {
            level: LevelId::new(1, 6),
            ..env.ctx()
        };
        assert!(run(cd_evidence("x"), &other).is_fail());
    }

    #[test]
    fn file_requires_a_regular_file() {
        let env = Env::new();
        let ctx = env.ctx();
        assert_eq!(
            run(file("a/b.txt"), &ctx),
            Check::fail("Soubor 'a/b.txt' neexistuje")
        );
        env.write("a/b.txt", "x");
        assert_eq!(run(file("a/b.txt"), &ctx), Check::Pass);
        // A directory is not a file.
        assert_eq!(run(file("a"), &ctx), Check::fail("Soubor 'a' neexistuje"));
        assert_eq!(
            run(file("nope").message("Vytvořte soubor nope."), &ctx),
            Check::fail("Vytvořte soubor nope.")
        );
    }

    #[test]
    fn dir_requires_a_directory() {
        let env = Env::new();
        let ctx = env.ctx();
        assert_eq!(run(dir("d"), &ctx), Check::fail("Adresář 'd' neexistuje"));
        env.mkdir("d");
        assert_eq!(run(dir("d"), &ctx), Check::Pass);
        env.write("f", "x");
        assert_eq!(run(dir("f"), &ctx), Check::fail("Adresář 'f' neexistuje"));
        assert_eq!(
            run(dir("zzz").message("Vytvořte zzz."), &ctx),
            Check::fail("Vytvořte zzz.")
        );
    }

    #[test]
    fn no_file_and_no_dir_pass_only_when_nothing_remains() {
        let env = Env::new();
        let ctx = env.ctx();
        assert_eq!(run(no_file("gone.txt"), &ctx), Check::Pass);
        assert_eq!(run(no_dir("gone"), &ctx), Check::Pass);

        env.write("f.txt", "x");
        env.mkdir("d");
        assert_eq!(
            run(no_file("f.txt"), &ctx),
            Check::fail("Soubor 'f.txt' stále existuje")
        );
        assert_eq!(
            run(no_dir("d"), &ctx),
            Check::fail("Adresář 'd' stále existuje")
        );
        // Nothing means nothing: a file where a directory was expected gone also fails.
        assert_eq!(
            run(no_dir("f.txt"), &ctx),
            Check::fail("Adresář 'f.txt' stále existuje")
        );
        assert_eq!(
            run(no_file("d").message("Smažte d."), &ctx),
            Check::fail("Smažte d.")
        );
    }

    #[test]
    fn a_dangling_symlink_counts_as_remaining() {
        let env = Env::new();
        symlink("does-not-exist", env.root.join("dangling")).unwrap();
        assert_eq!(
            run(no_file("dangling"), &env.ctx()),
            Check::fail("Soubor 'dangling' stále existuje")
        );
        assert_eq!(
            run(no_dir("dangling"), &env.ctx()),
            Check::fail("Adresář 'dangling' stále existuje")
        );
    }

    #[test]
    fn a_symlink_below_the_path_is_a_workspace_escape() {
        let env = Env::new();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("secret.txt"), "x").unwrap();
        symlink(outside.path(), env.root.join("link")).unwrap();
        let escape = Check::fail(messages::PATH_ESCAPES_WORKSPACE);
        let ctx = env.ctx();
        for requirement in [
            Requirement::from(file("link/secret.txt")),
            dir("link").into(),
            file("link").into(),
            no_file("link/secret.txt").into(),
            no_dir("link/x").into(),
            content("link/secret.txt").exact("x").into(),
            line_count("link/secret.txt", 1, "m").into(),
            same("link/secret.txt", "other").into(),
            moved("a", "link/secret.txt").into(),
            perm_bits("link/secret.txt").has(0o400).into(),
            mode("link/secret.txt", 0o644).into(),
            dir_mode("link", 0o755).into(),
        ] {
            assert_eq!(requirement.check(&ctx), escape, "{requirement:?}");
        }
    }

    #[test]
    fn a_symlink_leaf_is_an_escape_for_checks_that_read_it() {
        let env = Env::new();
        env.write("real.txt", "x");
        symlink(env.root.join("real.txt"), env.root.join("leaf.txt")).unwrap();
        let escape = Check::fail(messages::PATH_ESCAPES_WORKSPACE);
        let ctx = env.ctx();
        assert_eq!(run(file("leaf.txt"), &ctx), escape);
        assert_eq!(run(content("leaf.txt").exact("x"), &ctx), escape);
        assert_eq!(run(perm_bits("leaf.txt").has(0o400), &ctx), escape);
        // ...but for "must be gone", the link is simply something that remains.
        assert_eq!(
            run(no_file("leaf.txt"), &ctx),
            Check::fail("Soubor 'leaf.txt' stále existuje")
        );
    }

    #[test]
    fn a_path_that_escapes_the_section_is_refused() {
        let env = Env::new();
        let escape = Check::fail(messages::PATH_ESCAPES_WORKSPACE);
        assert_eq!(run(file("../level-2/x"), &env.ctx()), escape);
        assert_eq!(run(file("/etc/passwd"), &env.ctx()), escape);
    }

    #[test]
    fn content_exact_contains_excludes_and_strip() {
        let env = Env::new();
        env.write("log.txt", "  ERROR one\nOK two\n  ");
        let ctx = env.ctx();
        let mismatch = Check::fail(messages::FILE_CONTENT_MISMATCH);

        assert_eq!(
            run(content("log.txt").exact("  ERROR one\nOK two\n  "), &ctx),
            Check::Pass
        );
        assert_eq!(
            run(content("log.txt").exact("ERROR one\nOK two"), &ctx),
            mismatch
        );
        assert_eq!(
            run(content("log.txt").strip().exact("ERROR one\nOK two"), &ctx),
            Check::Pass
        );
        assert_eq!(
            run(content("log.txt").contains(["ERROR", "OK"]), &ctx),
            Check::Pass
        );
        assert_eq!(
            run(content("log.txt").contains(["ERROR", "FATAL"]), &ctx),
            mismatch
        );
        assert_eq!(
            run(content("log.txt").excludes(["FATAL"]), &ctx),
            Check::Pass
        );
        assert_eq!(run(content("log.txt").excludes(["OK"]), &ctx), mismatch);
        assert_eq!(
            run(
                content("log.txt")
                    .contains(["ERROR"])
                    .excludes(["FATAL"])
                    .exact("nope")
                    .message("Obsah nesedí."),
                &ctx
            ),
            Check::fail("Obsah nesedí.")
        );
        assert_eq!(
            run(content("log.txt"), &ctx),
            Check::Pass,
            "no conditions, only readability"
        );
    }

    #[test]
    fn content_strip_uses_python_whitespace() {
        let env = Env::new();
        env.write("f.txt", "\u{1c}\u{a0}hello\u{3000}\n");
        assert_eq!(
            run(content("f.txt").strip().exact("hello"), &env.ctx()),
            Check::Pass
        );
    }

    #[test]
    fn content_sees_universal_newlines_like_python_text_mode() {
        let env = Env::new();
        env.write("crlf.txt", "a\r\nb\rc\n");
        assert_eq!(
            run(content("crlf.txt").exact("a\nb\nc\n"), &env.ctx()),
            Check::Pass
        );
    }

    #[test]
    fn content_reports_missing_and_unreadable_files() {
        let env = Env::new();
        let ctx = env.ctx();
        assert_eq!(
            run(content("none.txt").exact("x"), &ctx),
            Check::fail("Soubor 'none.txt' neexistuje")
        );
        assert_eq!(
            run(content("none.txt").missing("Soubor chybí."), &ctx),
            Check::fail("Soubor chybí.")
        );
        env.mkdir("is-a-dir");
        assert_eq!(
            run(content("is-a-dir").exact("x"), &ctx),
            Check::fail("Soubor 'is-a-dir' neexistuje")
        );

        fs::write(env.root.join("binary.bin"), [0xff, 0xfe, 0x00]).unwrap();
        let Check::Fail(message) = run(content("binary.bin").exact("x"), &ctx) else {
            panic!("invalid UTF-8 must fail")
        };
        assert!(message.starts_with("Nelze přečíst soubor: "), "{message}");
        assert!(
            message.ends_with("Obnovte level příkazem `shellgame reset`."),
            "{message}"
        );
        assert_eq!(
            run(content("binary.bin").unreadable("Není to text."), &ctx),
            Check::fail("Není to text.")
        );
    }

    #[test]
    fn content_reports_a_file_the_player_made_unreadable() {
        let env = Env::new();
        let path = env.write("locked.txt", "x");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        let result = run(content("locked.txt").exact("x"), &env.ctx());
        // As root the file is still readable; otherwise the player gets the reset hint.
        if let Check::Fail(message) = result {
            assert!(message.contains("shellgame reset"), "{message}");
        }
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    }

    #[test]
    fn line_count_counts_like_python_splitlines() {
        let env = Env::new();
        let ctx = env.ctx();
        let cases: [(&str, usize); 12] = [
            ("", 0),
            ("one", 1),
            ("one\n", 1),
            ("one\ntwo\n", 2),
            ("one\ntwo", 2),
            ("a\r\nb\r\nc", 3),
            ("a\rb\rc", 3),
            ("a\u{0b}b\u{0c}c", 3),
            ("a\u{1c}b\u{1d}c\u{1e}d", 4),
            ("a\u{85}b", 2),
            ("a\u{2028}b\u{2029}c", 3),
            ("\n\n\n", 3),
        ];
        for (text, expected) in cases {
            env.write("f.txt", text);
            assert_eq!(
                run(line_count("f.txt", expected, "špatně"), &ctx),
                Check::Pass,
                "{text:?} should have {expected} lines"
            );
            assert_eq!(
                run(line_count("f.txt", expected + 1, "špatně"), &ctx),
                Check::fail("špatně")
            );
        }
    }

    #[test]
    fn line_count_reports_missing_files_and_bad_utf8() {
        let env = Env::new();
        let ctx = env.ctx();
        assert_eq!(
            run(line_count("none", 1, "m"), &ctx),
            Check::fail("Soubor 'none' neexistuje")
        );
        assert_eq!(
            run(line_count("none", 1, "m").missing("Chybí."), &ctx),
            Check::fail("Chybí.")
        );
        fs::write(env.root.join("bin"), [0xff]).unwrap();
        let Check::Fail(message) = run(line_count("bin", 1, "m"), &ctx) else {
            panic!()
        };
        assert!(message.starts_with("Nelze přečíst soubor"), "{message}");
    }

    #[test]
    fn same_accepts_an_identical_copy_of_a_file() {
        let env = Env::new();
        env.write("a.txt", "hello");
        env.write("b.txt", "hello");
        assert_eq!(run(same("a.txt", "b.txt"), &env.ctx()), Check::Pass);
        env.write("b.txt", "hellO");
        assert_eq!(
            run(same("a.txt", "b.txt"), &env.ctx()),
            Check::fail(messages::COPY_CONTENT_MISMATCH)
        );
        assert_eq!(
            run(same("a.txt", "b.txt").message("Liší se."), &env.ctx()),
            Check::fail("Liší se.")
        );
    }

    #[test]
    fn same_compares_trees_deeply_including_dotfiles() {
        let env = Env::new();
        for base in ["src", "copy"] {
            env.write(&format!("{base}/a.txt"), "a");
            env.write(&format!("{base}/sub/b.txt"), "b");
            env.write(&format!("{base}/sub/.hidden"), "h");
            env.mkdir(&format!("{base}/empty"));
        }
        let ctx = env.ctx();
        assert_eq!(run(same("src", "copy"), &ctx), Check::Pass);

        // An extra dotfile deep down.
        env.write("copy/sub/.extra", "x");
        assert!(run(same("src", "copy"), &ctx).is_fail());
        fs::remove_file(env.root.join("copy/sub/.extra")).unwrap();
        assert_eq!(run(same("src", "copy"), &ctx), Check::Pass);

        // A missing dotfile, a differing dotfile, a missing empty directory.
        fs::write(env.root.join("copy/sub/.hidden"), "H").unwrap();
        assert!(run(same("src", "copy"), &ctx).is_fail());
        fs::write(env.root.join("copy/sub/.hidden"), "h").unwrap();
        fs::remove_dir(env.root.join("copy/empty")).unwrap();
        assert!(run(same("src", "copy"), &ctx).is_fail());
    }

    #[test]
    fn same_rejects_a_file_against_a_directory() {
        let env = Env::new();
        env.write("f", "x");
        env.mkdir("d");
        assert!(run(same("f", "d"), &env.ctx()).is_fail());
        assert!(run(same("d", "f"), &env.ctx()).is_fail());
    }

    #[test]
    fn same_compares_nested_symlinks_by_target_and_never_follows_them() {
        let env = Env::new();
        for base in ["src", "copy"] {
            env.mkdir(base);
            env.write(&format!("{base}/real.txt"), "r");
        }
        symlink("real.txt", env.root.join("src/link")).unwrap();
        symlink("real.txt", env.root.join("copy/link")).unwrap();
        let ctx = env.ctx();
        assert_eq!(run(same("src", "copy"), &ctx), Check::Pass);

        // A different target is a different link, even if both resolve to equal content.
        env.write("src/other.txt", "r");
        fs::remove_file(env.root.join("copy/link")).unwrap();
        symlink("other.txt", env.root.join("copy/link")).unwrap();
        assert!(run(same("src", "copy"), &ctx).is_fail());

        // A link versus a regular file with the same content is not the same.
        fs::remove_file(env.root.join("copy/link")).unwrap();
        env.write("copy/link", "r");
        assert!(run(same("src", "copy"), &ctx).is_fail());
    }

    #[test]
    fn same_does_not_loop_on_a_symlink_to_a_parent() {
        let env = Env::new();
        for base in ["src", "copy"] {
            env.mkdir(&format!("{base}/sub"));
            symlink("..", env.root.join(format!("{base}/sub/up"))).unwrap();
        }
        assert_eq!(run(same("src", "copy"), &env.ctx()), Check::Pass);
    }

    #[test]
    fn same_reports_missing_ends_with_their_own_messages() {
        let env = Env::new();
        env.write("exists", "x");
        let ctx = env.ctx();
        assert_eq!(
            run(same("gone", "exists"), &ctx),
            Check::fail(
                "Zdrojový soubor 'gone' chybí (možná jste ho přesunuli místo zkopírování?)."
            )
        );
        assert_eq!(
            run(same("exists", "gone"), &ctx),
            Check::fail("Cílový soubor 'gone' neexistuje.")
        );
        assert_eq!(
            run(same("gone", "exists").source_error("Zdroj zmizel."), &ctx),
            Check::fail("Zdroj zmizel.")
        );
        assert_eq!(
            run(
                same("exists", "gone").destination_error("Chybí kopie."),
                &ctx
            ),
            Check::fail("Chybí kopie.")
        );
        // Source is checked first.
        assert_eq!(
            run(
                same("gone", "gone2")
                    .source_error("a")
                    .destination_error("b"),
                &ctx
            ),
            Check::fail("a")
        );
    }

    #[test]
    fn same_reports_an_unreadable_directory_as_a_comparison_failure() {
        let env = Env::new();
        for base in ["src", "copy"] {
            env.write(&format!("{base}/locked/f"), "x");
        }
        let locked = env.root.join("src/locked");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        let result = run(same("src", "copy"), &env.ctx());
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        if let Check::Fail(message) = result {
            assert!(
                message.starts_with("Chyba při kontrole souborů: "),
                "{message}"
            );
        }
    }

    #[test]
    fn moved_needs_the_source_gone_and_the_destination_present() {
        let env = Env::new();
        let ctx = env.ctx();
        env.write("old.txt", "x");
        assert_eq!(
            run(moved("old.txt", "new.txt"), &ctx),
            Check::fail(
                "Původní soubor 'old.txt' stále existuje (možná jste ho zkopírovali místo přesunutí?)."
            )
        );
        fs::rename(env.root.join("old.txt"), env.root.join("new.txt")).unwrap();
        assert_eq!(run(moved("old.txt", "new.txt"), &ctx), Check::Pass);
        fs::remove_file(env.root.join("new.txt")).unwrap();
        assert_eq!(
            run(moved("old.txt", "new.txt"), &ctx),
            Check::fail("Cílový soubor 'new.txt' neexistuje.")
        );
    }

    #[test]
    fn moved_custom_messages_apply_to_the_matching_end() {
        let env = Env::new();
        env.write("old.txt", "x");
        let ctx = env.ctx();
        let requirement = moved("old.txt", "new.txt")
            .source_error("Zdroj tu stále je.")
            .destination_error("Cíl chybí.");
        assert_eq!(
            run(requirement.clone(), &ctx),
            Check::fail("Zdroj tu stále je.")
        );
        fs::remove_file(env.root.join("old.txt")).unwrap();
        assert_eq!(run(requirement, &ctx), Check::fail("Cíl chybí."));
    }

    #[test]
    fn moved_works_for_directories() {
        let env = Env::new();
        env.write("projekt/a.txt", "x");
        fs::create_dir_all(env.root.join("archiv")).unwrap();
        assert!(run(moved("projekt", "archiv/projekt"), &env.ctx()).is_fail());
        fs::rename(env.root.join("projekt"), env.root.join("archiv/projekt")).unwrap();
        assert_eq!(
            run(moved("projekt", "archiv/projekt"), &env.ctx()),
            Check::Pass
        );
    }

    #[test]
    fn perm_bits_has_and_lacks() {
        let env = Env::new();
        let path = env.write("run.sh", "#!/bin/sh");
        let ctx = env.ctx();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            run(perm_bits("run.sh").has(0o100), &ctx),
            Check::fail(messages::PERMISSION_WRONG)
        );
        assert_eq!(
            run(perm_bits("run.sh").has(0o100).message("Chybí x."), &ctx),
            Check::fail("Chybí x.")
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o744)).unwrap();
        assert_eq!(run(perm_bits("run.sh").has(0o100), &ctx), Check::Pass);
        assert_eq!(run(perm_bits("run.sh").has(0o700), &ctx), Check::Pass);
        assert_eq!(
            run(perm_bits("run.sh").has(0o110), &ctx),
            Check::fail(messages::PERMISSION_WRONG)
        );

        // lacks
        assert!(
            run(perm_bits("run.sh").lacks(0o222), &ctx).is_fail(),
            "owner can still write"
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
        assert_eq!(run(perm_bits("run.sh").lacks(0o222), &ctx), Check::Pass);
        assert_eq!(
            run(perm_bits("run.sh").has(0o400).lacks(0o222), &ctx),
            Check::Pass
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o454)).unwrap();
        assert!(run(perm_bits("run.sh").lacks(0o222), &ctx).is_pass());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o464)).unwrap();
        assert!(run(perm_bits("run.sh").lacks(0o222), &ctx).is_fail());
    }

    #[test]
    fn perm_bits_accumulates_calls_and_ignores_unrelated_bits() {
        let env = Env::new();
        let path = env.write("f", "x");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        let ctx = env.ctx();
        assert_eq!(
            run(
                perm_bits("f")
                    .has(0o100)
                    .has(0o010)
                    .lacks(0o002)
                    .lacks(0o020),
                &ctx
            ),
            Check::Pass
        );
        // No conditions: only the file's existence matters.
        assert_eq!(run(perm_bits("f"), &ctx), Check::Pass);
    }

    #[test]
    fn perm_bits_and_mode_need_a_regular_file() {
        let env = Env::new();
        env.mkdir("d");
        let ctx = env.ctx();
        assert_eq!(
            run(perm_bits("nope").has(0o400), &ctx),
            Check::fail("Soubor 'nope' neexistuje")
        );
        assert_eq!(
            run(mode("nope", 0o644), &ctx),
            Check::fail("Soubor 'nope' neexistuje")
        );
        assert_eq!(
            run(perm_bits("d").has(0o400), &ctx),
            Check::fail("Soubor 'd' neexistuje")
        );
    }

    #[test]
    fn mode_compares_the_exact_permission_bits() {
        let env = Env::new();
        let path = env.write("deploy.sh", "x");
        let ctx = env.ctx();
        for actual in [0o755u32, 0o654, 0o600, 0o000, 0o4755] {
            fs::set_permissions(&path, fs::Permissions::from_mode(actual)).unwrap();
            assert_eq!(
                run(mode("deploy.sh", actual), &ctx),
                Check::Pass,
                "{actual:o}"
            );
            assert_eq!(
                run(mode("deploy.sh", actual ^ 0o001), &ctx),
                Check::fail(messages::PERMISSION_WRONG)
            );
        }
        assert_eq!(
            run(mode("deploy.sh", 0o644).message("Chmod 644."), &ctx),
            Check::fail("Chmod 644.")
        );
    }

    #[test]
    fn dir_mode_compares_a_directorys_bits() {
        let env = Env::new();
        let path = env.mkdir("locked");
        let ctx = env.ctx();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o550)).unwrap();
        assert_eq!(run(dir_mode("locked", 0o550), &ctx), Check::Pass);
        assert_eq!(
            run(dir_mode("locked", 0o755), &ctx),
            Check::fail(messages::PERMISSION_WRONG)
        );
        assert_eq!(
            run(dir_mode("locked", 0o755).message("Odemkněte."), &ctx),
            Check::fail("Odemkněte.")
        );
        assert_eq!(
            run(dir_mode("nope", 0o755), &ctx),
            Check::fail("Adresář 'nope' neexistuje")
        );
        env.write("file", "x");
        assert_eq!(
            run(dir_mode("file", 0o644), &ctx),
            Check::fail("Adresář 'file' neexistuje")
        );
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn custom_checks_get_the_context() {
        let env = Env::new();
        let requirement = check(|ctx| {
            if ctx.home_name() == "ada" {
                Check::Pass
            } else {
                Check::fail("nejste Ada")
            }
        });
        assert_eq!(requirement.check(&env.ctx()), Check::Pass);
        let other_home = PathBuf::from("/home/bob");
        let ctx = Ctx {
            home: &other_home,
            ..env.ctx()
        };
        assert_eq!(requirement.check(&ctx), Check::fail("nejste Ada"));
    }

    #[test]
    fn introspection_exposes_paths_markers_and_custom_at_messages() {
        let at_requirement: Requirement = at("deep/start").message("Zpět na start.").into();
        assert_eq!(at_requirement.paths(), ["deep/start"]);
        assert_eq!(
            at_requirement.custom_at_message(),
            Some(("deep/start", "Zpět na start."))
        );
        assert_eq!(Requirement::from(at("x")).custom_at_message(), None);
        assert_eq!(Requirement::from(same("a", "b")).paths(), ["a", "b"]);
        assert_eq!(Requirement::from(moved("a", "b")).paths(), ["a", "b"]);
        assert_eq!(Requirement::from(file("f")).paths(), ["f"]);
        assert_eq!(Requirement::from(dir_mode("d", 0o755)).paths(), ["d"]);
        assert!(at_home().paths().is_empty());
        assert!(evidence(Marker::Pwd, "m").paths().is_empty());
        assert_eq!(evidence(Marker::Pwd, "m").marker(), Some(Marker::Pwd));
        assert_eq!(at_home().marker(), None);
        assert_eq!(Requirement::from(file("f")).marker(), None);
    }

    #[test]
    fn lint_flags_impossible_bits_and_modes() {
        assert!(
            Requirement::from(perm_bits("f").has(0o100).lacks(0o022))
                .lint()
                .is_empty()
        );
        assert_eq!(
            Requirement::from(perm_bits("f").has(0o100).lacks(0o100))
                .lint()
                .len(),
            1
        );
        assert_eq!(
            Requirement::from(perm_bits("f").has(0o10000)).lint().len(),
            1
        );
        assert_eq!(Requirement::from(mode("f", 0o10000)).lint().len(), 1);
        assert!(Requirement::from(mode("f", 0o7777)).lint().is_empty());
        assert!(at_home().lint().is_empty());
    }
}
