//! Declarative `cd` policies for strict navigation levels.
//!
//! Several levels teach one exact movement command (`cd ..`, `cd ../../..`, `cd -`, an
//! absolute path) and must reject anything else. Hand-written hooks all repeated the same
//! steps, and each had to remember the anti-soft-lock contract. A [`CdPolicy`] states only
//! what the level requires; the engine performs the contract, so the guarantee is
//! structural instead of a convention every author has to re-implement.
//!
//! A policy has two kinds of clause:
//!
//! * **scope** ([`CdPolicy::from_dir`], [`CdPolicy::target`]) decides whether the policy
//!   has anything to say about this move. A move outside the scope is allowed silently.
//! * **rules** ([`CdPolicy::exact`], [`absolute`](CdPolicy::absolute),
//!   [`source`](CdPolicy::source), [`after`](CdPolicy::after), [`within`](CdPolicy::within),
//!   [`rule`](CdPolicy::rule)) decide whether an in-scope move is acceptable. They are
//!   checked in the order they were added, and the first failing rule blocks the move with
//!   its own message.
//!
//! # The contract
//!
//! The shell asks twice per `cd`: once before the move, once after it succeeded.
//!
//! 1. **Before the move** ([`CdPolicy::pre_move`]): a move out of scope is allowed. A move
//!    that is *lifted* is allowed: the level's `cd` marker exists (the player has already
//!    shown the command), or the move's destination is the level's start directory (the
//!    recovery move: without it one extra step could leave a player in a directory from
//!    which every permitted move leads further away). Otherwise the first failing rule
//!    blocks. The engine only returns the decision; printing the red panel and
//!    [`CD_RECOVERY_TIP`](crate::messages::CD_RECOVERY_TIP) is the UI's job.
//! 2. **After a successful move** ([`CdPolicy::post_move`]): the level's `cd` marker is
//!    recorded when the move was in scope, every rule passes (judged with the real
//!    destination) and, if the policy has an [`arrive`](CdPolicy::arrive) directory, the
//!    player landed there. Evidence comes from scope, rules and `arrive` only: *lifting
//!    waives blocking but never grants or denies evidence*, so `cd -` back to level 2.2's
//!    start directory (a lifted move) still counts. A move that failed in the shell records
//!    nothing, because the shell never reports it.
//! 3. **`on_move`** runs after every successful move, lifted or not, in scope or not. A
//!    level that needs its own bookkeeping (the 1.9 walk home) uses it together with
//!    [`manual_evidence`](CdPolicy::manual_evidence).
//!
//! Directories are compared with [`soft_canonicalize`] on
//! both sides, so a workspace below a symlink (macOS `/tmp`) and a destination that does not
//! exist yet both compare correctly.

use std::path::{Path, PathBuf};

use crate::engine::Ctx;
use crate::engine::level::Level;
use crate::engine::pytext::py_strip;
use crate::markers::{Marker, Markers};
use crate::paths::{RelPath, normalize_lexically, same_path, soft_canonicalize};

/// One attempted (or completed) `cd`, with everything a clause may need to judge it.
#[derive(Debug, Clone, Copy)]
pub struct CdMove<'a> {
    /// The first non-option argument of `cd`: `""` for a bare `cd`, `"-"` for `cd -`.
    pub target: &'a str,
    /// All arguments of `cd`, options included, for custom rules.
    pub args: &'a [String],
    /// The logical directory the player is in before the move, as the shell reports it.
    pub pwd: &'a Path,
    /// The shell's previous directory, used to resolve `cd -` before the move.
    pub prev: Option<&'a Path>,
    /// After the move only: the logical `$PWD` the shell ended up in.
    pub dest: Option<&'a Path>,
    /// The situation: workspace, section root, home, level.
    pub ctx: &'a Ctx<'a>,
}

impl<'a> CdMove<'a> {
    /// A move about to happen: no previous directory, no destination yet.
    pub fn new(ctx: &'a Ctx<'a>, target: &'a str, args: &'a [String], pwd: &'a Path) -> Self {
        Self {
            target,
            args,
            pwd,
            prev: None,
            dest: None,
            ctx,
        }
    }

    /// Set the shell's previous directory (what `cd -` goes to).
    #[must_use]
    pub fn with_prev(mut self, prev: Option<&'a Path>) -> Self {
        self.prev = prev;
        self
    }

    /// Set the directory the shell actually ended up in (a move that has happened).
    #[must_use]
    pub fn with_dest(mut self, dest: &'a Path) -> Self {
        self.dest = Some(dest);
        self
    }

    /// True for a bare `cd`.
    pub fn is_bare(&self) -> bool {
        self.target.is_empty()
    }

    /// Where the move goes, as the shell's logical `$PWD` will read.
    ///
    /// After the move this is [`CdMove::dest`] as given. Before it, the target is resolved
    /// the way `cd -L` does: `""` is `$HOME`, `-` is the previous directory, an absolute
    /// target is itself, anything else is relative to `pwd`; then `.` and `..` are resolved
    /// lexically (no symlink is followed). `None` when it cannot be known: `cd -` without a
    /// previous directory, or a relative target with no usable `pwd`.
    pub fn destination(&self) -> Option<PathBuf> {
        if let Some(dest) = self.dest {
            return Some(dest.to_path_buf());
        }
        let raw = match self.target {
            "" => self.ctx.home.to_path_buf(),
            "-" => self.prev?.to_path_buf(),
            target if target.starts_with('/') => PathBuf::from(target),
            target => self.pwd.join(target),
        };
        let resolved = normalize_lexically(&raw);
        resolved.is_absolute().then_some(resolved)
    }
}

/// What to do about a `cd` that is about to happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CdDecision {
    /// Let the shell move.
    Allow,
    /// Refuse the move and show this message (the UI adds the recovery tip).
    Block(String),
}

impl CdDecision {
    /// True for [`CdDecision::Block`].
    pub fn is_block(&self) -> bool {
        matches!(self, Self::Block(_))
    }

    /// The message of a block.
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Allow => None,
            Self::Block(message) => Some(message),
        }
    }
}

/// Start an empty policy: `policy().exact("..", "Použijte přesně příkaz 'cd ..'.")`.
pub fn policy() -> CdPolicy {
    CdPolicy::default()
}

#[derive(Debug, Clone)]
enum Scope {
    FromDir(String),
    Target(String),
}

#[derive(Debug, Clone)]
enum Rule {
    Exact { command: String, message: String },
    Absolute { message: String },
    Source { path: String, message: String },
    After { marker: Marker, message: String },
    Within { path: String, message: String },
    Custom(fn(&CdMove) -> Result<(), String>),
}

/// The outcome of judging a move before it happens.
enum Judgement {
    /// The policy has nothing to say about this move.
    OutOfScope,
    /// In scope, but enforcement is lifted.
    Lifted,
    /// In scope, not lifted, and a rule failed.
    Blocked(String),
    /// In scope, not lifted, and every rule passed.
    Accepted,
}

/// A `cd` policy; see the [module documentation](self) for the contract.
///
/// ```
/// use shellgame::engine::prelude::*;
///
/// // Level 1.12: from the deepest directory, after `pwd`, one exact command.
/// let drill = policy()
///     .from_dir("gamma/deep/a/b/c")
///     .after(Marker::Pwd, "Nejdřív použijte příkaz 'pwd'.")
///     .exact("../../../..", "Použijte jeden příkaz 'cd ../../../..'.");
/// # let _ = drill;
/// ```
#[derive(Debug, Clone, Default)]
pub struct CdPolicy {
    scope: Vec<Scope>,
    rules: Vec<Rule>,
    bare_cd: Option<String>,
    arrive: Option<String>,
    on_move: Option<fn(&CdMove, &Markers)>,
    manual_evidence: bool,
}

impl CdPolicy {
    /// Scope: only moves that start in this section-relative directory.
    #[must_use]
    pub fn from_dir(mut self, path: impl Into<String>) -> Self {
        self.scope.push(Scope::FromDir(path.into()));
        self
    }

    /// Scope: only moves whose raw target is exactly this text (such as `"-"`).
    #[must_use]
    pub fn target(mut self, value: impl Into<String>) -> Self {
        self.scope.push(Scope::Target(value.into()));
        self
    }

    /// Rule: the target must equal `command`, compared after stripping whitespace and
    /// trailing slashes (unless the whole target is `/`), so `../` is `..`.
    #[must_use]
    pub fn exact(mut self, command: impl Into<String>, message: impl Into<String>) -> Self {
        self.rules.push(Rule::Exact {
            command: command.into(),
            message: message.into(),
        });
        self
    }

    /// Rule: the target must be an absolute path (start with `/`). A bare `cd` fails it.
    #[must_use]
    pub fn absolute(mut self, message: impl Into<String>) -> Self {
        self.rules.push(Rule::Absolute {
            message: message.into(),
        });
        self
    }

    /// Rule: the move must start in this section-relative directory.
    #[must_use]
    pub fn source(mut self, path: impl Into<String>, message: impl Into<String>) -> Self {
        self.rules.push(Rule::Source {
            path: path.into(),
            message: message.into(),
        });
        self
    }

    /// Rule: this marker must exist, so an earlier drill (such as [`Marker::Pwd`]) must
    /// have been performed first.
    #[must_use]
    pub fn after(mut self, marker: Marker, message: impl Into<String>) -> Self {
        self.rules.push(Rule::After {
            marker,
            message: message.into(),
        });
        self
    }

    /// Rule: the destination must stay inside this section-relative directory.
    #[must_use]
    pub fn within(mut self, path: impl Into<String>, message: impl Into<String>) -> Self {
        self.rules.push(Rule::Within {
            path: path.into(),
            message: message.into(),
        });
        self
    }

    /// Rule: a custom check. It runs before the move (with [`CdMove::dest`] `None`) and
    /// again, for evidence, after it (with `dest` set), and returns the block message on
    /// failure. Custom rules produce their own messages, which a bare `cd` never replaces.
    #[must_use]
    pub fn rule(mut self, rule: fn(&CdMove) -> Result<(), String>) -> Self {
        self.rules.push(Rule::Custom(rule));
        self
    }

    /// The message to show instead of a built-in rule's when the move is a bare `cd` (which
    /// goes home, outside wherever the level is).
    #[must_use]
    pub fn bare_cd(mut self, message: impl Into<String>) -> Self {
        self.bare_cd = Some(message.into());
        self
    }

    /// Record evidence only when the move lands in this section-relative directory, not for
    /// any move that satisfies the rules (such as `cd /`, which is absolute but not the
    /// target).
    #[must_use]
    pub fn arrive(mut self, path: impl Into<String>) -> Self {
        self.arrive = Some(path.into());
        self
    }

    /// Run after every successful move (see the contract), with the markers of the level.
    #[must_use]
    pub fn on_move(mut self, on_move: fn(&CdMove, &Markers)) -> Self {
        self.on_move = Some(on_move);
        self
    }

    /// Never record evidence automatically; [`CdPolicy::on_move`] decides when the drill is
    /// done (by creating the level's [`Marker::Cd`] itself).
    #[must_use]
    pub fn manual_evidence(mut self) -> Self {
        self.manual_evidence = true;
        self
    }

    /// The markers read by `after` rules, so preparing the level clears them.
    pub fn after_markers(&self) -> impl Iterator<Item = Marker> + '_ {
        self.rules.iter().filter_map(|rule| match rule {
            Rule::After { marker, .. } => Some(*marker),
            _ => None,
        })
    }

    /// Every section-relative path the policy declares, as written.
    pub fn paths(&self) -> Vec<&str> {
        let scoped = self.scope.iter().filter_map(|scope| match scope {
            Scope::FromDir(path) => Some(path.as_str()),
            Scope::Target(_) => None,
        });
        let ruled = self.rules.iter().filter_map(|rule| match rule {
            Rule::Source { path, .. } | Rule::Within { path, .. } => Some(path.as_str()),
            _ => None,
        });
        scoped.chain(ruled).chain(self.arrive.as_deref()).collect()
    }

    /// Problems with how the policy was built.
    pub fn lint(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let built_in = self
            .rules
            .iter()
            .any(|rule| !matches!(rule, Rule::Custom(_)));
        if self.bare_cd.is_some() && !built_in {
            problems.push(
                ".bare_cd() has no effect: it only replaces the messages of built-in rules"
                    .to_owned(),
            );
        }
        if self.manual_evidence && self.on_move.is_none() {
            problems.push(
                ".manual_evidence() without .on_move(): nothing would ever record the evidence"
                    .to_owned(),
            );
        }
        if self.manual_evidence && self.arrive.is_some() {
            problems.push(".arrive() has no effect with .manual_evidence()".to_owned());
        }
        problems
    }

    fn in_scope(&self, mv: &CdMove) -> bool {
        self.scope.iter().all(|scope| match scope {
            Scope::FromDir(path) => at_dir(mv.ctx, mv.pwd, path),
            Scope::Target(value) => mv.target == value,
        })
    }

    /// The message of the first failing rule, in the order the rules were added.
    fn first_failure(&self, mv: &CdMove) -> Option<String> {
        self.rules.iter().find_map(|rule| {
            let message = rule.failure(mv)?;
            let replaceable = mv.is_bare() && !matches!(rule, Rule::Custom(_));
            Some(match &self.bare_cd {
                Some(bare) if replaceable => bare.clone(),
                _ => message,
            })
        })
    }

    fn arrives(&self, mv: &CdMove) -> bool {
        match &self.arrive {
            None => true,
            Some(path) => match (mv.destination(), section_dir(mv.ctx, path)) {
                (Some(dest), Some(target)) => same_path(&dest, &target),
                _ => false,
            },
        }
    }

    fn judge(&self, level: &Level, mv: &CdMove) -> Judgement {
        debug_assert_eq!(
            level.id, mv.ctx.level,
            "the context belongs to another level"
        );
        if !self.in_scope(mv) {
            return Judgement::OutOfScope;
        }
        if level.cd_enforcement_lifted(mv) {
            return Judgement::Lifted;
        }
        match self.first_failure(mv) {
            Some(message) => Judgement::Blocked(message),
            None => Judgement::Accepted,
        }
    }

    fn record_evidence(mv: &CdMove) {
        // Evidence is best effort: a marker that cannot be written must not break the
        // player's `cd`, and the level simply stays incomplete.
        let _ = mv.ctx.markers().create(&Marker::Cd);
    }

    /// Decide whether a move that is about to happen is allowed. See the
    /// [module documentation](self) for the contract.
    pub fn pre_move(&self, level: &Level, mv: &CdMove) -> CdDecision {
        match self.judge(level, mv) {
            Judgement::Blocked(message) => CdDecision::Block(message),
            Judgement::OutOfScope | Judgement::Lifted | Judgement::Accepted => CdDecision::Allow,
        }
    }

    /// Record the outcome of a move that succeeded: evidence (unless it is manual), then
    /// [`CdPolicy::on_move`]. `mv.dest` must be the directory the shell ended up in and
    /// `mv.pwd` the directory it started in. See the [module documentation](self).
    pub fn post_move(&self, level: &Level, mv: &CdMove) {
        debug_assert_eq!(
            level.id, mv.ctx.level,
            "the context belongs to another level"
        );
        if !self.manual_evidence
            && self.in_scope(mv)
            && self.first_failure(mv).is_none()
            && self.arrives(mv)
        {
            Self::record_evidence(mv);
        }
        if let Some(on_move) = self.on_move {
            on_move(mv, &mv.ctx.markers());
        }
    }

    /// [`CdPolicy::pre_move`] for a shell started by the Python version of the game.
    ///
    /// That shell reports a move only *before* it happens, and the Python hook recorded the
    /// evidence right then. So when the move is allowed because it is in scope, not lifted
    /// and every rule passes, and the policy has no `arrive` directory and no manual
    /// evidence, the evidence is recorded immediately. (The new hook records it after the
    /// move, so a move that fails in the shell earns nothing.)
    pub fn legacy_pre_move(&self, level: &Level, mv: &CdMove) -> CdDecision {
        match self.judge(level, mv) {
            Judgement::Blocked(message) => CdDecision::Block(message),
            Judgement::Accepted => {
                if self.arrive.is_none() && !self.manual_evidence {
                    Self::record_evidence(mv);
                }
                CdDecision::Allow
            }
            Judgement::OutOfScope | Judgement::Lifted => CdDecision::Allow,
        }
    }

    /// [`CdPolicy::post_move`] for a shell started by the Python version of the game, which
    /// reports only the directory it ended up in (`cd-hook --post-move $PWD`). Evidence is
    /// recorded when the policy has an `arrive` directory and `dest` is it (the pre-move
    /// report covers every other policy), then `on_move` runs with `pwd` set to `dest`.
    pub fn legacy_post_move(&self, level: &Level, ctx: &Ctx, dest: &Path) {
        let args: [String; 0] = [];
        let mv = CdMove {
            target: "",
            args: &args,
            pwd: dest,
            prev: None,
            dest: Some(dest),
            ctx,
        };
        debug_assert_eq!(level.id, ctx.level, "the context belongs to another level");
        if !self.manual_evidence && self.arrive.is_some() && self.arrives(&mv) {
            Self::record_evidence(&mv);
        }
        if let Some(on_move) = self.on_move {
            on_move(&mv, &ctx.markers());
        }
    }
}

impl Rule {
    /// The block message if this rule rejects the move.
    fn failure(&self, mv: &CdMove) -> Option<String> {
        let rejected = match self {
            Self::Exact { command, message } => {
                (normalize_command(mv.target) != normalize_command(command)).then_some(message)
            }
            Self::Absolute { message } => (!mv.target.starts_with('/')).then_some(message),
            Self::Source { path, message } => (!at_dir(mv.ctx, mv.pwd, path)).then_some(message),
            Self::After { marker, message } => {
                (!mv.ctx.markers().exists(marker)).then_some(message)
            }
            Self::Within { path, message } => {
                // A destination that cannot be known (`cd -` with no history) is not
                // something this rule can judge, and the shell will refuse it anyway.
                let outside = match (mv.destination(), section_dir(mv.ctx, path)) {
                    (Some(dest), Some(dir)) => {
                        !soft_canonicalize(&dest).starts_with(soft_canonicalize(&dir))
                    }
                    _ => false,
                };
                outside.then_some(message)
            }
            Self::Custom(rule) => return rule(mv).err(),
        };
        rejected.cloned()
    }
}

/// Compare `cd` targets without tripping over spacing or a trailing slash: strip, then
/// drop trailing slashes unless that would leave nothing (the target `/`).
fn normalize_command(command: &str) -> &str {
    let stripped = py_strip(command);
    if stripped.len() > 1 {
        let trimmed = stripped.trim_end_matches('/');
        if trimmed.is_empty() { "/" } else { trimmed }
    } else {
        stripped
    }
}

fn section_dir(ctx: &Ctx, rel: &str) -> Option<PathBuf> {
    RelPath::parse(rel)
        .ok()
        .map(|rel| rel.under(&ctx.section_root))
}

/// True when `dir` (a directory the shell reported) is this section-relative directory.
/// A relative or empty `dir` is unknown, never "here": resolving it would consult the
/// process working directory, which the engine must not read.
fn at_dir(ctx: &Ctx, dir: &Path, rel: &str) -> bool {
    dir.is_absolute() && section_dir(ctx, rel).is_some_and(|expected| same_path(dir, &expected))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::LevelId;
    use crate::engine::level::level;
    use std::fs;
    use std::os::unix::fs::symlink;

    /// A workspace with section 1, a home and a few directories, all canonical.
    struct World {
        _tmp: tempfile::TempDir,
        ws: PathBuf,
        root: PathBuf,
        home: PathBuf,
    }

    impl World {
        fn new() -> Self {
            let tmp = tempfile::tempdir().unwrap();
            let base = fs::canonicalize(tmp.path()).unwrap();
            let ws = base.join("ws");
            let root = ws.join("level-1");
            let home = base.join("home/ada");
            fs::create_dir_all(&root).unwrap();
            fs::create_dir_all(&home).unwrap();
            Self {
                _tmp: tmp,
                ws,
                root,
                home,
            }
        }

        fn dir(&self, rel: &str) -> PathBuf {
            let path = self.root.join(rel);
            fs::create_dir_all(&path).unwrap();
            path
        }

        fn ctx(&self, id: LevelId) -> Ctx<'_> {
            Ctx::new(&self.ws, self.root.clone(), None, &self.home, id)
        }

        fn marker(&self, id: LevelId, marker: &Marker) -> bool {
            Markers::new(&self.ws, id).exists(marker)
        }
    }

    fn lvl(number: u16) -> Level {
        let mut level = level(number, "test");
        level.id.section = 1;
        level
    }

    fn args_of(target: &str) -> Vec<String> {
        if target.is_empty() {
            Vec::new()
        } else {
            vec![target.to_owned()]
        }
    }

    fn pre(w: &World, level: &Level, target: &str, pwd: &Path) -> CdDecision {
        pre_prev(w, level, target, pwd, None)
    }

    fn pre_prev(
        w: &World,
        level: &Level,
        target: &str,
        pwd: &Path,
        prev: Option<&Path>,
    ) -> CdDecision {
        let ctx = w.ctx(level.id);
        let args = args_of(target);
        let mv = CdMove::new(&ctx, target, &args, pwd).with_prev(prev);
        level.cd_pre_move(&mv)
    }

    fn post(w: &World, level: &Level, target: &str, pwd: &Path, dest: &Path) {
        post_prev(w, level, target, pwd, None, dest);
    }

    fn post_prev(
        w: &World,
        level: &Level,
        target: &str,
        pwd: &Path,
        prev: Option<&Path>,
        dest: &Path,
    ) {
        let ctx = w.ctx(level.id);
        let args = args_of(target);
        let mv = CdMove::new(&ctx, target, &args, pwd)
            .with_prev(prev)
            .with_dest(dest);
        level.cd_post_move(&mv);
    }

    fn blocked(decision: CdDecision) -> String {
        match decision {
            CdDecision::Block(message) => message,
            CdDecision::Allow => panic!("expected a block"),
        }
    }

    fn allowed(decision: &CdDecision) {
        assert_eq!(
            *decision,
            CdDecision::Allow,
            "expected the move to be allowed"
        );
    }

    // ----------------------------------------------------------------- rules --------

    #[test]
    fn exact_blocks_everything_but_its_command() {
        let w = World::new();
        let level = lvl(4)
            .start("alpha")
            .cd(policy().exact("..", "Použijte přesně 'cd ..'."));
        let alpha = w.dir("alpha");
        w.dir("alpha/inner");

        allowed(&pre(&w, &level, "..", &alpha));
        allowed(&pre(&w, &level, "../", &alpha));
        allowed(&pre(&w, &level, "..//", &alpha));
        allowed(&pre(&w, &level, "  ..  ", &alpha));
        for wrong in ["../..", "/", "inner", "./..", "../alpha/..", "", "~", "-"] {
            assert_eq!(
                blocked(pre(&w, &level, wrong, &alpha)),
                "Použijte přesně 'cd ..'.",
                "{wrong:?} should be blocked"
            );
        }
        // Staying in the start directory is the recovery move, whatever the command.
        allowed(&pre(&w, &level, ".", &alpha));
    }

    #[test]
    fn exact_normalizes_both_sides_and_keeps_the_root_target() {
        let w = World::new();
        let alpha = w.dir("alpha");
        let slashed = lvl(4).cd(policy().exact("../../", "m"));
        allowed(&pre(&w, &slashed, "../..", &alpha));
        allowed(&pre(&w, &slashed, "../../", &alpha));
        assert!(pre(&w, &slashed, "../", &alpha).is_block());

        let root = lvl(5).cd(policy().exact("/", "m"));
        allowed(&pre(&w, &root, "/", &alpha));
        allowed(&pre(&w, &root, "//", &alpha));
        assert!(pre(&w, &root, "", &alpha).is_block());
        assert!(pre(&w, &root, "/tmp", &alpha).is_block());
    }

    #[test]
    fn absolute_blocks_relative_and_bare_cd() {
        let w = World::new();
        let level = lvl(8).cd(policy().absolute("Použijte absolutní cestu."));
        let start = w.dir("");
        for wrong in ["absolute-target", "./absolute-target", "..", "", "~"] {
            assert_eq!(
                blocked(pre(&w, &level, wrong, &start)),
                "Použijte absolutní cestu.",
                "{wrong:?}"
            );
        }
        allowed(&pre(&w, &level, "/", &start));
        allowed(&pre(&w, &level, "/tmp/anything", &start));
    }

    #[test]
    fn source_requires_the_move_to_start_in_the_directory() {
        let w = World::new();
        let start = w.dir("deep/structure/start");
        let elsewhere = w.dir("deep/other");
        let level = lvl(3).cd(policy().source("deep/structure/start", "Ze startu."));
        allowed(&pre(&w, &level, "anything", &start));
        assert_eq!(
            blocked(pre(&w, &level, "anything", &elsewhere)),
            "Ze startu."
        );
        // A directory of the same name elsewhere is not the source.
        let impostor = w.ws.join("other/deep/structure/start");
        fs::create_dir_all(&impostor).unwrap();
        assert!(pre(&w, &level, "x", &impostor).is_block());
    }

    #[test]
    fn source_compares_canonical_paths() {
        let w = World::new();
        let start = w.dir("deep/start");
        let alias = w.ws.join("alias");
        symlink(&w.root, &alias).unwrap();
        let level = lvl(3).cd(policy().source("deep/start", "Ze startu."));
        // The shell reports the logical path through an alias of the workspace.
        allowed(&pre(&w, &level, "x", &alias.join("deep/start")));
        allowed(&pre(&w, &level, "x", &start));
    }

    #[test]
    fn after_requires_an_earlier_drill() {
        let w = World::new();
        let level = lvl(12).cd(policy().after(Marker::Pwd, "Nejdřív použijte 'pwd'."));
        let here = w.dir("gamma");
        assert_eq!(
            blocked(pre(&w, &level, "x", &here)),
            "Nejdřív použijte 'pwd'."
        );
        fs::write(w.ws.join(".pwd_used"), "").unwrap();
        allowed(&pre(&w, &level, "x", &here));
    }

    #[test]
    fn within_keeps_the_destination_inside_a_directory() {
        let w = World::new();
        let entry = w.dir("maze/entry");
        w.dir("maze/entry/hall");
        w.dir("maze2");
        let level = lvl(7)
            .start("maze/entry")
            .cd(policy().within("maze", "Nemůžete opustit bludiště."));
        let hall = w.root.join("maze/entry/hall");
        for inside in ["hall", "..", "../entry/hall", hall.to_str().unwrap()] {
            allowed(&pre(&w, &level, inside, &entry));
        }
        for outside in ["../..", "../../..", "/", "../../maze2", "../../level-2"] {
            assert_eq!(
                blocked(pre(&w, &level, outside, &entry)),
                "Nemůžete opustit bludiště.",
                "{outside:?}"
            );
        }
    }

    #[test]
    fn within_is_component_wise_not_a_string_prefix() {
        let w = World::new();
        let maze = w.dir("maze");
        w.dir("maze-extra");
        let level = lvl(7).cd(policy().within("maze", "ven"));
        assert!(pre(&w, &level, "../maze-extra", &maze).is_block());
    }

    #[test]
    fn within_follows_symlinks_when_comparing() {
        let w = World::new();
        let entry = w.dir("maze/entry");
        let outside = w.dir("outside");
        symlink(&outside, w.root.join("maze/entry/portal")).unwrap();
        let level = lvl(7).cd(policy().within("maze", "ven"));
        // The link lives inside the maze but leads out of it.
        assert_eq!(blocked(pre(&w, &level, "portal", &entry)), "ven");
    }

    #[test]
    fn within_resolves_cd_dash_through_the_previous_directory() {
        let w = World::new();
        let inside = w.dir("maze/entry");
        let outside = w.dir("elsewhere");
        let level = lvl(7).cd(policy().within("maze", "ven"));
        assert!(pre_prev(&w, &level, "-", &inside, Some(&outside)).is_block());
        allowed(&pre_prev(&w, &level, "-", &outside, Some(&inside)));
        // No history: nothing to judge, and the shell refuses `cd -` by itself.
        allowed(&pre_prev(&w, &level, "-", &inside, None));
    }

    #[test]
    fn within_uses_the_real_destination_after_the_move() {
        let w = World::new();
        let inside = w.dir("maze/entry");
        let outside = w.dir("elsewhere");
        let level = lvl(7).cd(policy().within("maze", "ven"));
        // The shell says the move ended outside, whatever the target text suggests.
        post(&w, &level, "hall", &inside, &outside);
        assert!(!w.marker(level.id, &Marker::Cd));
        post(&w, &level, "hall", &inside, &w.dir("maze/entry/hall"));
        assert!(w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn custom_rules_decide_with_their_own_messages() {
        fn forbid_dotdot(mv: &CdMove) -> Result<(), String> {
            if mv.args.iter().any(|arg| arg == "..") {
                Err("Žádné '..'.".to_owned())
            } else {
                Ok(())
            }
        }
        let w = World::new();
        let here = w.dir("a");
        let level = lvl(9).cd(policy().rule(forbid_dotdot));
        assert_eq!(blocked(pre(&w, &level, "..", &here)), "Žádné '..'.");
        allowed(&pre(&w, &level, "b", &here));
    }

    #[test]
    fn custom_rules_see_every_argument_the_target_and_the_context() {
        fn inspect(mv: &CdMove) -> Result<(), String> {
            let flag = mv.args.first().map_or("", String::as_str);
            Err(format!(
                "flag={flag} target={} home={} bare={} dest={}",
                mv.target,
                mv.ctx.home_name(),
                mv.is_bare(),
                mv.dest.is_some()
            ))
        }
        let w = World::new();
        let here = w.dir("a");
        let level = lvl(9).cd(policy().rule(inspect));
        let ctx = w.ctx(level.id);
        let args = vec!["-P".to_owned(), "/tmp".to_owned()];
        let mv = CdMove::new(&ctx, "/tmp", &args, &here);
        assert_eq!(
            blocked(level.cd_pre_move(&mv)),
            "flag=-P target=/tmp home=ada bare=false dest=false"
        );
    }

    #[test]
    fn the_first_failing_rule_in_insertion_order_wins() {
        let w = World::new();
        let start = w.dir("start");
        let other = w.dir("other");
        let level = lvl(3).cd(policy().source("start", "zdroj").exact("../x", "příkaz"));
        assert_eq!(blocked(pre(&w, &level, "../x", &other)), "zdroj");
        assert_eq!(blocked(pre(&w, &level, "../y", &start)), "příkaz");
        assert_eq!(blocked(pre(&w, &level, "../y", &other)), "zdroj");
        // Reversed insertion order, reversed priority.
        let reversed = lvl(3).cd(policy().exact("../x", "příkaz").source("start", "zdroj"));
        assert_eq!(blocked(pre(&w, &reversed, "../y", &other)), "příkaz");
    }

    #[test]
    fn a_policy_without_rules_never_blocks() {
        let w = World::new();
        let here = w.dir("a");
        let level = lvl(3).cd(policy());
        allowed(&pre(&w, &level, "/anywhere", &here));
        allowed(&pre(&w, &level, "", &here));
    }

    // ---------------------------------------------------------------- bare cd -------

    #[test]
    fn bare_cd_replaces_the_messages_of_built_in_rules_only() {
        fn custom(_: &CdMove) -> Result<(), String> {
            Err("vlastní".to_owned())
        }
        let w = World::new();
        let here = w.dir("a");

        for policy in [
            policy().absolute("a").bare_cd("Musíte zadat cestu."),
            policy().exact("..", "e").bare_cd("Musíte zadat cestu."),
            policy().source("zzz", "s").bare_cd("Musíte zadat cestu."),
            policy()
                .after(Marker::Pwd, "m")
                .bare_cd("Musíte zadat cestu."),
            policy().within("zzz", "w").bare_cd("Musíte zadat cestu."),
        ] {
            let level = lvl(8).cd(policy);
            assert_eq!(blocked(pre(&w, &level, "", &here)), "Musíte zadat cestu.");
            // A non-empty target keeps the rule's own message.
            let message = blocked(pre(&w, &level, "x", &here));
            assert_ne!(message, "Musíte zadat cestu.");
        }

        // Custom rules always speak for themselves.
        let level = lvl(8).cd(policy().rule(custom).bare_cd("bare"));
        assert_eq!(blocked(pre(&w, &level, "", &here)), "vlastní");
    }

    #[test]
    fn the_bare_cd_message_only_appears_when_a_rule_actually_fails() {
        let w = World::new();
        // Home is inside the maze here, so a bare `cd` stays inside and passes.
        let inside_home = w.dir("maze/home");
        let pwd = w.dir("maze/entry");
        let level = lvl(7).cd(policy().within("maze", "ven").bare_cd("bare"));
        let ctx = Ctx::new(&w.ws, w.root.clone(), None, &inside_home, level.id);
        let mv = CdMove::new(&ctx, "", &[], &pwd);
        allowed(&level.cd_pre_move(&mv));
    }

    // ----------------------------------------------------------------- scope --------

    #[test]
    fn from_dir_scopes_the_policy_to_moves_starting_there() {
        let w = World::new();
        let deep = w.dir("gamma/deep/a/b/c");
        let elsewhere = w.dir("gamma");
        let level = lvl(12).cd(policy()
            .from_dir("gamma/deep/a/b/c")
            .exact("../../../..", "Jeden příkaz."));
        assert_eq!(blocked(pre(&w, &level, "..", &deep)), "Jeden příkaz.");
        allowed(&pre(&w, &level, "../../../..", &deep));
        // From anywhere else the policy says nothing.
        allowed(&pre(&w, &level, "..", &elsewhere));
        allowed(&pre(&w, &level, "/", &elsewhere));
    }

    #[test]
    fn target_scopes_the_policy_to_one_literal_argument() {
        let w = World::new();
        let b = w.dir("location-B");
        let a = w.dir("location-A");
        let level = lvl(2).start("location-A").cd(policy()
            .target("-")
            .source("location-B", "Až z location-B."));
        // `cd -` from A with an unrelated previous directory: blocked.
        assert_eq!(
            blocked(pre_prev(&w, &level, "-", &a, Some(&b))),
            "Až z location-B."
        );
        // Other targets are out of scope.
        allowed(&pre(&w, &level, "../location-B", &a));
        allowed(&pre(&w, &level, "/", &a));
        allowed(&pre(&w, &level, "-x", &a));
        allowed(&pre(&w, &level, " -", &a));
    }

    #[test]
    fn multiple_scope_clauses_must_all_apply() {
        let w = World::new();
        let a = w.dir("a");
        let b = w.dir("b");
        let level = lvl(5).cd(policy().from_dir("a").target("x").exact("y", "jen y"));
        assert!(pre(&w, &level, "x", &a).is_block());
        allowed(&pre(&w, &level, "z", &a));
        allowed(&pre(&w, &level, "x", &b));
    }

    // ---------------------------------------------------------------- lifting -------

    #[test]
    fn evidence_lifts_enforcement() {
        let w = World::new();
        let alpha = w.dir("alpha");
        let level = lvl(4).start("alpha").cd(policy().exact("..", "Jen '..'."));
        assert!(pre(&w, &level, "../..", &alpha).is_block());
        fs::write(w.ws.join(".cd-1.4"), "").unwrap();
        allowed(&pre(&w, &level, "../..", &alpha));
        allowed(&pre(&w, &level, "/", &alpha));
    }

    #[test]
    fn the_evidence_of_another_level_does_not_lift() {
        let w = World::new();
        let alpha = w.dir("alpha");
        let level = lvl(4).start("alpha").cd(policy().exact("..", "Jen '..'."));
        fs::write(w.ws.join(".cd-1.6"), "").unwrap();
        assert!(pre(&w, &level, "/", &alpha).is_block());
    }

    #[test]
    fn returning_to_the_start_directory_is_always_allowed() {
        let w = World::new();
        let start = w.dir("alpha");
        let far = w.dir("gamma/deep");
        let level = lvl(4).start("alpha").cd(policy().exact("..", "Jen '..'."));

        // Relative, with dots, absolute, with a trailing slash, through an alias.
        allowed(&pre(&w, &level, "../../alpha", &far));
        allowed(&pre(&w, &level, "../../alpha/", &far));
        allowed(&pre(&w, &level, start.to_str().unwrap(), &far));
        allowed(&pre(&w, &level, "../deep/../../alpha", &far));
        let alias = w.ws.join("alias");
        symlink(&w.root, &alias).unwrap();
        allowed(&pre(
            &w,
            &level,
            alias.join("alpha").to_str().unwrap(),
            &far,
        ));
        // Anything else from there is still blocked.
        assert!(pre(&w, &level, "../..", &far).is_block());
    }

    #[test]
    fn cd_dash_back_to_the_start_directory_is_lifted() {
        let w = World::new();
        let a = w.dir("location-A");
        let b = w.dir("location-B");
        let level = lvl(2)
            .start("location-A")
            .cd(policy().exact("../elsewhere", "m"));
        // `cd -` from B, previous = A (the start): lifted. Previous = B: not lifted.
        allowed(&pre_prev(&w, &level, "-", &b, Some(&a)));
        assert!(pre_prev(&w, &level, "-", &a, Some(&b)).is_block());
        // Unknown previous directory: nothing to lift with.
        assert!(pre_prev(&w, &level, "-", &b, None).is_block());
    }

    #[test]
    fn a_level_without_a_start_is_never_lifted_by_destination() {
        let w = World::new();
        let here = w.dir("a");
        let level = lvl(4).cd(policy().exact("..", "Jen '..'."));
        assert!(pre(&w, &level, "/", &here).is_block());
        assert!(pre(&w, &level, ".", &here).is_block());
    }

    #[test]
    fn the_workspace_start_lifts_moves_to_the_workspace_root() {
        let w = World::new();
        let deep = w.dir("a/b");
        let level = lvl(9).start_at_workspace().cd(policy().exact("x", "m"));
        allowed(&pre(&w, &level, w.ws.to_str().unwrap(), &deep));
        allowed(&pre(&w, &level, "../../..", &deep));
        assert!(pre(&w, &level, "..", &deep).is_block());
    }

    #[test]
    fn a_bare_cd_is_lifted_when_it_lands_in_the_start_directory() {
        let w = World::new();
        let far = w.dir("far");
        let start = w.dir("alpha");
        let level = lvl(4).start("alpha").cd(policy().exact("..", "m"));
        // Normally `cd` goes home, which is not the start: blocked.
        assert!(pre(&w, &level, "", &far).is_block());
        // If home happens to be the start directory, it is the recovery move.
        let ctx = Ctx::new(&w.ws, w.root.clone(), None, &start, level.id);
        let mv = CdMove::new(&ctx, "", &[], &far);
        allowed(&level.cd_pre_move(&mv));
    }

    #[test]
    fn out_of_scope_moves_are_allowed_without_consulting_anything() {
        let w = World::new();
        let here = w.dir("a");
        let other = w.dir("b");
        let level = lvl(12).cd(policy().from_dir("b").exact("..", "m"));
        allowed(&pre(&w, &level, "/", &here));
        assert!(pre(&w, &level, "/", &other).is_block());
    }

    #[test]
    fn levels_without_a_policy_allow_every_move() {
        let w = World::new();
        let here = w.dir("a");
        let level = lvl(3);
        allowed(&pre(&w, &level, "/", &here));
        post(&w, &level, "/", &here, Path::new("/"));
        assert!(!w.marker(level.id, &Marker::Cd));
        // The legacy entry points are no-ops too.
        let ctx = w.ctx(level.id);
        level.cd_legacy_post_move(&ctx, Path::new("/"));
        let args = args_of("/");
        let mv = CdMove::new(&ctx, "/", &args, &here);
        allowed(&level.cd_legacy_pre_move(&mv));
    }

    // --------------------------------------------------------------- evidence -------

    #[test]
    fn a_passing_in_scope_move_records_evidence_after_it_succeeds() {
        let w = World::new();
        let alpha = w.dir("alpha");
        let level = lvl(4).start("alpha").cd(policy().exact("..", "Jen '..'."));
        // Allowed, but nothing is recorded until the shell reports the move.
        allowed(&pre(&w, &level, "..", &alpha));
        assert!(!w.marker(level.id, &Marker::Cd));
        post(&w, &level, "..", &alpha, &w.root);
        assert!(w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn a_move_that_breaks_a_rule_earns_no_evidence() {
        let w = World::new();
        let alpha = w.dir("alpha");
        let level = lvl(4).start("alpha").cd(policy().exact("..", "Jen '..'."));
        post(&w, &level, "../..", &alpha, w.root.parent().unwrap());
        post(&w, &level, "", &alpha, &w.home);
        assert!(!w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn an_out_of_scope_move_earns_no_evidence() {
        let w = World::new();
        let here = w.dir("a");
        let other = w.dir("b");
        let level = lvl(12).cd(policy().from_dir("b").exact("..", "m"));
        post(&w, &level, "..", &here, &w.root);
        assert!(!w.marker(level.id, &Marker::Cd));
        post(&w, &level, "..", &other, &w.root);
        assert!(w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn evidence_is_judged_by_scope_evaluated_with_the_directory_before_the_move() {
        let w = World::new();
        let deep = w.dir("gamma/deep/a/b/c");
        let level = lvl(12).cd(policy()
            .from_dir("gamma/deep/a/b/c")
            .after(Marker::Pwd, "pwd")
            .exact("../../../..", "příkaz"));
        // Without the pwd evidence the drill does not count, even if the shell moved.
        post(&w, &level, "../../../..", &deep, &w.dir("gamma"));
        assert!(!w.marker(level.id, &Marker::Cd));
        fs::write(w.ws.join(".pwd_used"), "").unwrap();
        post(&w, &level, "../../../..", &deep, &w.dir("gamma"));
        assert!(w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn lifting_never_grants_evidence() {
        let w = World::new();
        let far = w.dir("gamma/deep");
        w.dir("alpha");
        let level = lvl(4).start("alpha").cd(policy().exact("..", "Jen '..'."));
        // The move to the start directory is lifted (allowed) but is not `cd ..`.
        allowed(&pre(&w, &level, "../../alpha", &far));
        post(&w, &level, "../../alpha", &far, &w.root.join("alpha"));
        assert!(
            !w.marker(level.id, &Marker::Cd),
            "a recovery move is not the drill"
        );
    }

    #[test]
    fn lifting_never_denies_evidence() {
        // Level 2.2: `cd -` back to the start directory is lifted, and must still count.
        let w = World::new();
        let a = w.dir("location-A");
        let b = w.dir("location-B");
        let level = lvl(2).start("location-A").cd(policy()
            .target("-")
            .source("location-B", "Až z location-B."));
        allowed(&pre_prev(&w, &level, "-", &b, Some(&a)));
        post_prev(&w, &level, "-", &b, Some(&a), &a);
        assert!(w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn lifting_by_evidence_does_not_stop_evidence_either() {
        let w = World::new();
        let alpha = w.dir("alpha");
        let level = lvl(4).start("alpha").cd(policy().exact("..", "m"));
        // Pretend the marker was removed between pre and post: recorded again, harmlessly.
        post(&w, &level, "..", &alpha, &w.root);
        assert!(w.marker(level.id, &Marker::Cd));
        post(&w, &level, "..", &alpha, &w.root);
        assert!(w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn arrive_gates_evidence_on_the_destination() {
        let w = World::new();
        let start = w.dir("");
        let target = w.dir("absolute-target");
        let level = lvl(8)
            .start("")
            .cd(policy().absolute("Absolutně.").arrive("absolute-target"));

        // `cd /` is absolute: allowed, but it is not the target, so no evidence.
        allowed(&pre(&w, &level, "/", &start));
        post(&w, &level, "/", &start, Path::new("/"));
        assert!(!w.marker(level.id, &Marker::Cd));

        // The real thing.
        let absolute = target.to_str().unwrap();
        allowed(&pre(&w, &level, absolute, &start));
        post(&w, &level, absolute, &start, &target);
        assert!(w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn arrive_still_requires_the_rules_to_pass() {
        let w = World::new();
        let start = w.dir("");
        let target = w.dir("absolute-target");
        let level = lvl(8).cd(policy().absolute("Absolutně.").arrive("absolute-target"));
        // Relative `cd absolute-target` lands in the target but breaks the rule.
        post(&w, &level, "absolute-target", &start, &target);
        assert!(!w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn arrive_compares_canonical_destinations() {
        let w = World::new();
        let start = w.dir("");
        w.dir("absolute-target");
        let alias = w.ws.join("alias");
        symlink(&w.root, &alias).unwrap();
        let level = lvl(8).cd(policy().absolute("m").arrive("absolute-target"));
        let through_alias = alias.join("absolute-target");
        post(
            &w,
            &level,
            through_alias.to_str().unwrap(),
            &start,
            &through_alias,
        );
        assert!(w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn manual_evidence_never_records_automatically() {
        fn decide(mv: &CdMove, markers: &Markers) {
            if mv.dest.is_some_and(|dest| dest.ends_with("alpha")) {
                markers.create(&Marker::Cd).unwrap();
            }
        }
        let w = World::new();
        let here = w.dir("");
        let alpha = w.dir("alpha");
        let level = lvl(9).cd(policy()
            .exact("alpha", "m")
            .manual_evidence()
            .on_move(decide));
        post(&w, &level, "alpha", &here, &w.dir("beta"));
        assert!(
            !w.marker(level.id, &Marker::Cd),
            "rules passing is not enough"
        );
        post(&w, &level, "alpha", &here, &alpha);
        assert!(w.marker(level.id, &Marker::Cd), "on_move decided");
    }

    #[test]
    fn manual_evidence_without_on_move_records_nothing() {
        let w = World::new();
        let here = w.dir("a");
        let level = lvl(9).cd(policy().manual_evidence());
        post(&w, &level, "x", &here, &w.dir("b"));
        assert!(!w.marker(level.id, &Marker::Cd));
    }

    // ---------------------------------------------------------------- on_move -------

    #[test]
    fn on_move_runs_after_every_successful_move_lifted_or_not() {
        fn count(mv: &CdMove, markers: &Markers) {
            let seen = markers.read(&Marker::Own("moves")).unwrap_or_default();
            let line = mv.dest.map(|d| d.display().to_string()).unwrap_or_default();
            markers
                .create_with(&Marker::Own("moves"), &format!("{seen}{line}\n"))
                .unwrap();
        }
        let w = World::new();
        let alpha = w.dir("alpha");
        let far = w.dir("far");
        let level = lvl(4)
            .start("alpha")
            .cd(policy().from_dir("alpha").exact("..", "m").on_move(count));

        // In scope and passing.
        post(&w, &level, "..", &alpha, &w.root);
        // In scope, breaking the rule.
        post(&w, &level, "x", &alpha, &far);
        // Out of scope.
        post(&w, &level, "x", &far, &alpha);
        // Lifted by the marker the first call created.
        let log = Markers::new(&w.ws, level.id)
            .read(&Marker::Own("moves"))
            .unwrap();
        assert_eq!(log.lines().count(), 3, "{log:?}");
        assert!(log.contains(&w.root.display().to_string()));
        assert!(log.contains(&far.display().to_string()));
    }

    #[test]
    fn on_move_runs_even_when_the_level_has_no_rules() {
        fn touch(_: &CdMove, markers: &Markers) {
            markers.create(&Marker::Own("seen")).unwrap();
        }
        let w = World::new();
        let here = w.dir("a");
        let level = lvl(9).cd(policy().on_move(touch));
        post(&w, &level, "x", &here, &w.dir("b"));
        assert!(w.marker(level.id, &Marker::Own("seen")));
    }

    // ----------------------------------------------------------------- legacy -------

    #[test]
    fn legacy_pre_move_records_evidence_immediately_as_python_did() {
        let w = World::new();
        let alpha = w.dir("alpha");
        let level = lvl(4).start("alpha").cd(policy().exact("..", "Jen '..'."));
        let ctx = w.ctx(level.id);
        let args = args_of("..");
        let mv = CdMove::new(&ctx, "..", &args, &alpha);
        allowed(&level.cd_legacy_pre_move(&mv));
        assert!(w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn legacy_pre_move_records_nothing_when_blocked_lifted_or_out_of_scope() {
        let w = World::new();
        let alpha = w.dir("alpha");
        let far = w.dir("far");
        let level = lvl(4)
            .start("alpha")
            .cd(policy().from_dir("alpha").exact("..", "Jen '..'."));
        let ctx = w.ctx(level.id);

        // Blocked.
        let args = args_of("../..");
        let mv = CdMove::new(&ctx, "../..", &args, &alpha);
        assert!(level.cd_legacy_pre_move(&mv).is_block());
        // Out of scope.
        let args = args_of("..");
        let mv = CdMove::new(&ctx, "..", &args, &far);
        allowed(&level.cd_legacy_pre_move(&mv));
        // Lifted by returning to the start (here: in scope, wrong command, destination alpha).
        let args = args_of(".");
        let mv = CdMove::new(&ctx, ".", &args, &alpha);
        allowed(&level.cd_legacy_pre_move(&mv));
        assert!(!w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn legacy_pre_move_leaves_arrive_and_manual_policies_to_the_post_report() {
        let w = World::new();
        let start = w.dir("");
        let level = lvl(8).cd(policy().absolute("m").arrive("absolute-target"));
        let ctx = w.ctx(level.id);
        let args = args_of("/");
        let mv = CdMove::new(&ctx, "/", &args, &start);
        allowed(&level.cd_legacy_pre_move(&mv));
        assert!(!w.marker(level.id, &Marker::Cd));

        let manual = lvl(9).cd(policy().exact("/", "m").manual_evidence());
        let ctx = w.ctx(manual.id);
        let mv = CdMove::new(&ctx, "/", &args, &start);
        allowed(&manual.cd_legacy_pre_move(&mv));
        assert!(!w.marker(manual.id, &Marker::Cd));
    }

    #[test]
    fn legacy_pre_move_still_blocks_like_the_new_hook() {
        let w = World::new();
        let start = w.dir("");
        let level = lvl(8).cd(policy().absolute("Absolutně.").bare_cd("Zadejte cestu."));
        let ctx = w.ctx(level.id);
        let args = args_of("alpha");
        let mv = CdMove::new(&ctx, "alpha", &args, &start);
        assert_eq!(blocked(level.cd_legacy_pre_move(&mv)), "Absolutně.");
        let mv = CdMove::new(&ctx, "", &[], &start);
        assert_eq!(blocked(level.cd_legacy_pre_move(&mv)), "Zadejte cestu.");
    }

    #[test]
    fn legacy_post_move_records_arrive_evidence_and_runs_on_move() {
        fn record(mv: &CdMove, markers: &Markers) {
            assert_eq!(
                mv.pwd,
                mv.dest.unwrap(),
                "legacy reports only the destination"
            );
            assert!(mv.target.is_empty());
            assert!(mv.args.is_empty());
            markers.create(&Marker::Own("moved")).unwrap();
        }
        let w = World::new();
        let target = w.dir("absolute-target");
        let level = lvl(8).cd(policy()
            .absolute("m")
            .arrive("absolute-target")
            .on_move(record));
        let ctx = w.ctx(level.id);

        level.cd_legacy_post_move(&ctx, &w.root);
        assert!(!w.marker(level.id, &Marker::Cd));
        assert!(
            w.marker(level.id, &Marker::Own("moved")),
            "on_move runs for every move"
        );

        level.cd_legacy_post_move(&ctx, &target);
        assert!(w.marker(level.id, &Marker::Cd));
    }

    #[test]
    fn legacy_post_move_without_arrive_or_with_manual_evidence_records_nothing() {
        let w = World::new();
        let alpha = w.dir("alpha");
        let no_arrive = lvl(4).cd(policy().exact("..", "m"));
        no_arrive.cd_legacy_post_move(&w.ctx(no_arrive.id), &alpha);
        assert!(!w.marker(no_arrive.id, &Marker::Cd));

        let manual = lvl(8).cd(policy().arrive("alpha").manual_evidence());
        manual.cd_legacy_post_move(&w.ctx(manual.id), &alpha);
        assert!(!w.marker(manual.id, &Marker::Cd));
    }

    // ------------------------------------------------------------ destination -------

    #[test]
    fn destination_resolves_the_way_the_shell_will() {
        let w = World::new();
        let ctx = w.ctx(LevelId::new(1, 1));
        let pwd = w.root.join("gamma/deep");
        let prev = w.root.join("alpha");
        let mv = |target: &'static str| CdMove::new(&ctx, target, &[], &pwd).with_prev(Some(&prev));

        assert_eq!(mv("").destination(), Some(w.home.clone()));
        assert_eq!(mv("-").destination(), Some(prev.clone()));
        assert_eq!(
            mv("/tmp/x/../y").destination(),
            Some(PathBuf::from("/tmp/y"))
        );
        assert_eq!(mv("..").destination(), Some(w.root.join("gamma")));
        assert_eq!(mv("../../alpha/").destination(), Some(w.root.join("alpha")));
        assert_eq!(mv("a/./b").destination(), Some(pwd.join("a/b")));
        assert_eq!(
            mv("../../../../../../../..").destination(),
            Some(PathBuf::from("/"))
        );
    }

    #[test]
    fn destination_is_unknown_when_it_cannot_be_resolved() {
        let w = World::new();
        let ctx = w.ctx(LevelId::new(1, 1));
        let pwd = w.root.clone();
        assert_eq!(CdMove::new(&ctx, "-", &[], &pwd).destination(), None);
        // A relative target with no usable working directory.
        assert_eq!(
            CdMove::new(&ctx, "x", &[], Path::new("")).destination(),
            None
        );
        // ...but an absolute one does not need it.
        assert_eq!(
            CdMove::new(&ctx, "/x", &[], Path::new("")).destination(),
            Some(PathBuf::from("/x"))
        );
    }

    #[test]
    fn a_relative_working_directory_is_unknown_not_resolved_against_the_process() {
        let w = World::new();
        w.dir("");
        let level = lvl(3).cd(policy().from_dir("").exact("x", "m"));
        // `.` would resolve to wherever the test process happens to be; it must not match
        // the section root, and an empty pwd must not either.
        for pwd in [Path::new("."), Path::new(""), Path::new("level-1")] {
            allowed(&pre(&w, &level, "/", pwd));
        }
        let sourced = lvl(3).cd(policy().source("", "m"));
        for pwd in [Path::new("."), Path::new("")] {
            assert!(pre(&w, &sourced, "/", pwd).is_block());
        }
    }

    #[test]
    fn destination_after_the_move_is_the_reported_pwd() {
        let w = World::new();
        let ctx = w.ctx(LevelId::new(1, 1));
        let pwd = w.root.clone();
        let reported = w.home.clone();
        let mv = CdMove::new(&ctx, "..", &[], &pwd).with_dest(&reported);
        assert_eq!(mv.destination(), Some(reported));
    }

    #[test]
    fn destination_is_resolved_lexically_not_through_symlinks() {
        let w = World::new();
        let real = w.dir("real/deep");
        let link = w.root.join("link");
        symlink(&real, &link).unwrap();
        let ctx = w.ctx(LevelId::new(1, 1));
        // From the logical `link`, `..` is the logical parent, as in `cd -L`.
        let mv = CdMove::new(&ctx, "..", &[], &link);
        assert_eq!(mv.destination(), Some(w.root.clone()));
    }

    // -------------------------------------------------------------- introspection ---

    #[test]
    fn introspection_reports_markers_and_paths() {
        let p = policy()
            .from_dir("a")
            .target("-")
            .exact("..", "m")
            .after(Marker::Pwd, "m")
            .after(Marker::Own("x"), "m")
            .source("b", "m")
            .within("c", "m")
            .absolute("m")
            .arrive("d");
        assert_eq!(
            p.after_markers().collect::<Vec<_>>(),
            [Marker::Pwd, Marker::Own("x")]
        );
        assert_eq!(p.paths(), ["a", "b", "c", "d"]);
        assert!(policy().paths().is_empty());
        assert_eq!(policy().after_markers().count(), 0);
    }

    #[test]
    fn lint_flags_policies_that_cannot_work() {
        fn custom(_: &CdMove) -> Result<(), String> {
            Ok(())
        }
        fn noop(_: &CdMove, _: &Markers) {}
        assert!(policy().exact("..", "m").bare_cd("b").lint().is_empty());
        assert_eq!(policy().rule(custom).bare_cd("b").lint().len(), 1);
        assert_eq!(policy().bare_cd("b").lint().len(), 1);
        assert_eq!(policy().manual_evidence().lint().len(), 1);
        assert!(policy().manual_evidence().on_move(noop).lint().is_empty());
        assert_eq!(
            policy()
                .manual_evidence()
                .on_move(noop)
                .arrive("x")
                .lint()
                .len(),
            1
        );
    }

    #[test]
    fn decisions_expose_their_message() {
        assert_eq!(CdDecision::Allow.message(), None);
        assert!(!CdDecision::Allow.is_block());
        let block = CdDecision::Block("x".to_owned());
        assert_eq!(block.message(), Some("x"));
        assert!(block.is_block());
    }

    // ------------------------------------------- generic hooks: the maze and the walk --

    const SANCTUARY: &str = "maze/entry/a/b/sanctuary";

    fn maze_level(w: &World) -> Level {
        w.dir(SANCTUARY);
        lvl(7).start("maze/entry").cd(policy()
            .within("maze", "Nemůžete opustit bludiště před jeho dokončením.")
            .bare_cd("Nemůžete opustit bludiště. Pokračujte v navigaci uvnitř maze/.")
            .arrive(SANCTUARY))
    }

    #[test]
    fn the_maze_confines_the_player_until_the_sanctuary() {
        let w = World::new();
        let level = maze_level(&w);
        let entry = w.root.join("maze/entry");
        let a = entry.join("a");
        let b = a.join("b");
        let sanctuary = w.root.join(SANCTUARY);

        // Inside the maze everything is fine; leaving it, bare `cd` and `cd /` are not.
        allowed(&pre(&w, &level, "a", &entry));
        allowed(&pre(&w, &level, "b", &a));
        allowed(&pre(&w, &level, "../..", &b));
        assert_eq!(
            blocked(pre(&w, &level, "", &a)),
            "Nemůžete opustit bludiště. Pokračujte v navigaci uvnitř maze/."
        );
        for leave in ["/", "../../..", "../../../.."] {
            assert_eq!(
                blocked(pre(&w, &level, leave, &a)),
                "Nemůžete opustit bludiště před jeho dokončením.",
                "{leave:?}"
            );
        }
        // The recovery move to the start is allowed from anywhere, even from outside.
        allowed(&pre(&w, &level, entry.to_str().unwrap(), &w.root));

        // Walking around the maze earns nothing; reaching the sanctuary does.
        post(&w, &level, "a", &entry, &a);
        post(&w, &level, "b", &a, &b);
        assert!(!w.marker(level.id, &Marker::Cd));
        post(&w, &level, "sanctuary", &b, &sanctuary);
        assert!(w.marker(level.id, &Marker::Cd));

        // And once it is done, the cage opens.
        allowed(&pre(&w, &level, "/", &sanctuary));
        allowed(&pre(&w, &level, "", &sanctuary));
    }

    /// A port of the 1.9 walk home, to prove custom rules, `on_move` and manual evidence
    /// carry a stateful level: compares the *logical* `$PWD` with `$HOME`, never canonical
    /// paths, so a symlink shortcut cannot complete it.
    fn walk_rule(mv: &CdMove) -> Result<(), String> {
        if mv.is_bare() {
            return Err("Skoky nejsou povoleny. Jděte krok za krokem.".to_owned());
        }
        if mv.target == "/" {
            return Ok(());
        }
        if mv.target.starts_with('/') {
            return Err("Absolutní skoky nejsou povoleny (kromě cd /).".to_owned());
        }
        if mv.target.trim_end_matches('/').contains('/') {
            return Err("Cestujte po jednom segmentu (adresáři).".to_owned());
        }
        Ok(())
    }

    fn walk_progress(mv: &CdMove, markers: &Markers) {
        let Some(dest) = mv.dest else { return };
        let progress = Marker::Own("walk");
        if dest == Path::new("/") {
            markers.create_with(&progress, "/").unwrap();
            return;
        }
        let Some(text) = markers.read(&progress) else {
            return;
        };
        let Some(last) = text.lines().last() else {
            return;
        };
        let last = Path::new(last);
        if dest.parent() == Some(last) {
            markers
                .create_with(&progress, &format!("{text}\n{}", dest.display()))
                .unwrap();
            if dest == mv.ctx.home {
                markers.create(&Marker::Cd).unwrap();
            }
        } else if dest != last {
            markers.remove(&progress).unwrap();
        }
    }

    fn walk_level() -> Level {
        lvl(9)
            .start_at_workspace()
            .resets([Marker::Own("walk")])
            .cd(policy()
                .rule(walk_rule)
                .on_move(walk_progress)
                .manual_evidence())
    }

    #[test]
    fn the_home_walk_needs_every_step_from_the_root() {
        let w = World::new();
        let level = walk_level();
        let root = Path::new("/");
        // A fake home keeps the test independent of the machine: the walk is logical.
        let home = PathBuf::from("/home/ada");
        let ctx = Ctx::new(&w.ws, w.root.clone(), None, &home, level.id);
        let step = |target: &str, pwd: &Path, dest: &Path| {
            let args = args_of(target);
            let mv = CdMove::new(&ctx, target, &args, pwd);
            let decision = level.cd_pre_move(&mv);
            if decision == CdDecision::Allow {
                level.cd_post_move(&mv.with_dest(dest));
            }
            decision
        };
        let done = || Markers::new(&w.ws, level.id).exists(&Marker::Cd);

        // Jumps are blocked with their own messages.
        assert_eq!(
            blocked(step("", &w.ws, &home)),
            "Skoky nejsou povoleny. Jděte krok za krokem."
        );
        assert_eq!(
            blocked(step("/home/ada", root, &home)),
            "Absolutní skoky nejsou povoleny (kromě cd /)."
        );
        assert_eq!(
            blocked(step("home/ada", root, &home)),
            "Cestujte po jednom segmentu (adresáři)."
        );

        // The proper walk.
        allowed(&step("/", &w.ws, root));
        allowed(&step("home", root, Path::new("/home")));
        assert!(!done());
        allowed(&step("ada", Path::new("/home"), &home));
        assert!(done(), "arriving home along the walk completes it");

        // After completion, enforcement is lifted.
        allowed(&step("", &home, &home));
        allowed(&step("/anything", &home, Path::new("/anything")));
    }

    #[test]
    fn the_home_walk_restarts_when_the_player_jumps_sideways_or_up() {
        let w = World::new();
        let level = walk_level();
        let home = PathBuf::from("/home/ada");
        let ctx = Ctx::new(&w.ws, w.root.clone(), None, &home, level.id);
        let markers = Markers::new(&w.ws, level.id);
        let go = |target: &str, pwd: &str, dest: &str| {
            let (pwd, dest) = (PathBuf::from(pwd), PathBuf::from(dest));
            let args = args_of(target);
            let mv = CdMove::new(&ctx, target, &args, &pwd);
            allowed(&level.cd_pre_move(&mv));
            level.cd_post_move(&mv.with_dest(&dest));
        };
        go("/", "/tmp", "/");
        go("home", "/", "/home");
        assert_eq!(
            markers.read(&Marker::Own("walk")).as_deref(),
            Some("/\n/home")
        );
        // Up: invalid step, progress is dropped.
        go("..", "/home", "/");
        // `..` lands on "/" which restarts the walk.
        assert_eq!(markers.read(&Marker::Own("walk")).as_deref(), Some("/"));
        go("usr", "/", "/usr");
        go("..", "/usr", "/");
        go("home", "/", "/home");
        go("ada", "/home", "/home/ada");
        assert!(markers.exists(&Marker::Cd));
    }

    #[test]
    fn the_home_walk_is_logical_so_a_symlinked_shortcut_does_not_complete_it() {
        let w = World::new();
        let level = walk_level();
        let home = PathBuf::from("/home/ada");
        let ctx = Ctx::new(&w.ws, w.root.clone(), None, &home, level.id);
        let go = |target: &str, pwd: &str, dest: &str| {
            let (pwd, dest) = (PathBuf::from(pwd), PathBuf::from(dest));
            let args = args_of(target);
            let mv = CdMove::new(&ctx, target, &args, &pwd);
            level.cd_post_move(&mv.with_dest(&dest));
        };
        go("/", "/tmp", "/");
        // The shortcut is a directory named "ada" that is really somewhere else: the
        // logical path is /shortcut/ada, which never equals $HOME.
        go("shortcut", "/", "/shortcut");
        go("ada", "/shortcut", "/shortcut/ada");
        assert!(!Markers::new(&w.ws, level.id).exists(&Marker::Cd));
    }

    #[test]
    fn preparing_the_walk_level_clears_its_private_progress() {
        let w = World::new();
        let level = walk_level();
        let markers = Markers::new(&w.ws, level.id);
        markers
            .create_with(&Marker::Own("walk"), "/\n/home")
            .unwrap();
        markers.create(&Marker::Cd).unwrap();
        let section = crate::engine::Section::new(1, "level-1");
        level.prepare(&section, &w.ws).unwrap();
        assert!(!markers.exists(&Marker::Own("walk")));
        assert!(!markers.exists(&Marker::Cd), "its cd marker is cleared too");
    }
}
