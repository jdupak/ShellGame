//! Strict `cd` levels: a level teaches one movement command and the game blocks everything else.
//!
//! A level's `cd` is a flat list of clauses. *Scope* ([`when_in`]) says which moves the clauses are
//! about; *rules* ([`exact`], [`absolute`], [`within`], [`custom`]) say what an in-scope move must
//! satisfy, checked in order, the first failure blocking with its message; [`arrive`] narrows
//! evidence to one destination.
//!
//! The shell asks twice per `cd`. [`before`]: out of scope -> allow; *lifted* (the level's
//! `.cd_done` exists, or the destination is the level's start directory, so a wrong step can
//! always be undone) -> allow; otherwise the first failing rule blocks. [`after`], once the move
//! succeeded: unless a [`custom`] clause exists, record `.cd_done` when the move was in scope,
//! every rule passes and the player landed on the `arrive` directory (if any); lifting never
//! grants or denies evidence. Custom `track` functions run after every successful move and decide
//! evidence themselves. Directories are compared canonically, except what custom code compares.

use std::path::{Path, PathBuf};

use super::paths::{canonical, join, normalize, same, split};
use super::{CD_DONE, Ctx, Pos, start_dir};

/// One attempted (or completed) `cd`.
#[derive(Clone, Copy)]
pub struct CdMove<'a> {
    /// The first non-option argument: `""` for a bare `cd`, `"-"` for `cd -`.
    pub target: &'a str,
    /// The logical directory the player is in before the move, as the shell reports it.
    pub pwd: &'a Path,
    /// The shell's previous directory, which `cd -` goes to.
    pub prev: Option<&'a Path>,
    /// After the move: the logical `$PWD` the shell ended up in.
    pub dest: Option<&'a Path>,
    pub ctx: &'a Ctx<'a>,
}

impl CdMove<'_> {
    pub fn is_bare(&self) -> bool {
        self.target.is_empty()
    }

    /// Where the move goes, as the shell's logical `$PWD` will read: `dest` after the move, before
    /// it the target resolved like `cd -L` (`""` is `$HOME`, `-` the previous directory, a
    /// relative one is relative to `pwd`, `.` and `..` resolved textually). `None` when unknown.
    pub fn destination(&self) -> Option<PathBuf> {
        if let Some(dest) = self.dest {
            return Some(dest.to_path_buf());
        }
        let raw = match self.target {
            "" => self.ctx.home.to_path_buf(),
            "-" => self.prev?.to_path_buf(),
            target => self.pwd.join(target),
        };
        Some(normalize(&raw)).filter(|path| path.is_absolute())
    }
}

type Rule = fn(&CdMove) -> Result<(), String>;
type Track = fn(&CdMove);

#[derive(Clone, Copy)]
enum Kind {
    WhenIn(&'static str),
    Exact(&'static str, &'static str),
    Absolute(&'static str),
    Within(&'static str, &'static str),
    Arrive(&'static str),
    Custom(Rule, Track),
}

#[derive(Clone, Copy)]
pub struct Cd(Kind);

#[rustfmt::skip]
mod vocabulary {
    use super::*;

    /// Scope: only moves that start in this directory.
    pub const fn when_in(path: &'static str) -> Cd { Cd(Kind::WhenIn(path)) }
    /// Rule: the target is this command, ignoring whitespace and trailing slashes (`../` is `..`).
    pub const fn exact(command: &'static str, msg: &'static str) -> Cd { Cd(Kind::Exact(command, msg)) }
    /// Rule: the target is an absolute path (a bare `cd` fails it).
    pub const fn absolute(msg: &'static str) -> Cd { Cd(Kind::Absolute(msg)) }
    /// Rule: the destination stays inside this directory (a bare `cd`, which goes home, does not).
    pub const fn within(path: &'static str, msg: &'static str) -> Cd { Cd(Kind::Within(path, msg)) }
    /// Evidence only when the move lands in this directory.
    pub const fn arrive(path: &'static str) -> Cd { Cd(Kind::Arrive(path)) }
    /// A rule with its own messages (run before the move and again for evidence) and a tracker that
    /// runs after every successful move and records evidence itself (see `Ctx::mark`).
    pub const fn custom(rule: Rule, track: Track) -> Cd { Cd(Kind::Custom(rule, track)) }
}
pub use vocabulary::*;

#[cfg(test)]
impl Cd {
    pub fn paths(&self) -> Vec<&'static str> {
        match self.0 {
            Kind::WhenIn(p) | Kind::Within(p, _) | Kind::Arrive(p) => vec![p],
            _ => vec![],
        }
    }
}

/// `Err` carries the message of a blocked move (the UI adds `CD_RECOVERY_TIP`).
pub fn before(pos: Pos, mv: &CdMove) -> Result<(), String> {
    decide(pos.level().cd, &start_dir(pos, mv.ctx.workspace), mv)
}

pub fn after(pos: Pos, mv: &CdMove) {
    record(pos.level().cd, mv);
}

fn decide(clauses: &[Cd], start: &Path, mv: &CdMove) -> Result<(), String> {
    let lifted =
        mv.ctx.marker(CD_DONE).exists() || mv.destination().is_some_and(|dest| same(&dest, start));
    if !in_scope(clauses, mv) || lifted {
        return Ok(());
    }
    first_failure(clauses, mv).map_or(Ok(()), Err)
}

fn record(clauses: &[Cd], mv: &CdMove) {
    if clauses.is_empty() {
        return;
    }
    let mut tracked = false;
    for clause in clauses {
        if let Kind::Custom(_, track) = clause.0 {
            tracked = true;
            track(mv);
        }
    }
    let arrived = clauses.iter().all(|clause| match clause.0 {
        Kind::Arrive(path) => {
            dir(mv.ctx, path).zip(mv.destination()).is_some_and(|(dir, dest)| same(&dest, &dir))
        }
        _ => true,
    });
    if !tracked && arrived && in_scope(clauses, mv) && first_failure(clauses, mv).is_none() {
        // Best effort: a marker that cannot be written must not break the player's `cd`.
        let _ = mv.ctx.mark(CD_DONE, "");
    }
}

fn dir(ctx: &Ctx, rel: &str) -> Option<PathBuf> {
    split(rel).ok().map(|_| join(&ctx.section_root, rel))
}

/// True when `pwd` (reported by the shell, so absolute) is this directory.
fn at_dir(ctx: &Ctx, pwd: &Path, rel: &str) -> bool {
    pwd.is_absolute() && dir(ctx, rel).is_some_and(|dir| same(pwd, &dir))
}

fn in_scope(clauses: &[Cd], mv: &CdMove) -> bool {
    clauses.iter().all(|clause| match clause.0 {
        Kind::WhenIn(path) => at_dir(mv.ctx, mv.pwd, path),
        _ => true,
    })
}

fn first_failure(clauses: &[Cd], mv: &CdMove) -> Option<String> {
    clauses.iter().find_map(|clause| {
        let msg = match clause.0 {
            Kind::Exact(command, msg) => (compare(mv.target) != compare(command)).then_some(msg),
            Kind::Absolute(msg) => (!mv.target.starts_with('/')).then_some(msg),
            Kind::Within(path, msg) => {
                // A destination nobody can know (`cd -` without history) is the shell's to refuse.
                let inside = mv
                    .destination()
                    .zip(dir(mv.ctx, path))
                    .is_none_or(|(dest, dir)| canonical(&dest).starts_with(canonical(&dir)));
                (!inside).then_some(msg)
            }
            Kind::Custom(rule, _) => return rule(mv).err(),
            _ => None,
        }?;
        Some(msg.to_owned())
    })
}

/// A target without spacing or trailing slashes (but `/` stays `/`).
fn compare(target: &str) -> &str {
    let target = target.trim();
    match target.trim_end_matches('/') {
        "" if target.len() > 1 => "/",
        trimmed if target.len() > 1 => trimmed,
        _ => target,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::game::testing::{World, err};

    const ALLOW: Result<(), String> = Ok(());

    /// A level's clauses played in section 1 of a workspace; `start` is its start directory.
    struct Drill {
        w: World,
        clauses: Vec<Cd>,
        start: String,
    }

    impl Drill {
        fn new(clauses: &[Cd], start: &str) -> Self {
            Drill { w: World::new(), clauses: clauses.to_vec(), start: start.to_owned() }
        }

        /// A move from `pwd` (`cd -` goes to `prev`): the decision and, if the move happened,
        /// whether it was recorded as the drill.
        fn go(&self, pwd: &str, target: &str, prev: Option<&str>) -> (Result<(), String>, bool) {
            let ctx = self.w.ctx(None);
            let (pwd, prev) = (self.w.root().join(pwd), prev.map(|prev| self.w.root().join(prev)));
            let mv = CdMove { target, pwd: &pwd, prev: prev.as_deref(), dest: None, ctx: &ctx };
            let decision = decide(&self.clauses, &self.w.root().join(&self.start), &mv);
            if decision.is_ok()
                && let Some(dest) = mv.destination()
            {
                record(&self.clauses, &CdMove { dest: Some(&dest), ..mv });
            }
            let recorded = ctx.marker(CD_DONE).exists();
            let _ = fs::remove_file(ctx.marker(CD_DONE));
            (decision, recorded)
        }

        fn decide(&self, pwd: &str, target: &str) -> Result<(), String> {
            self.go(pwd, target, None).0
        }

        fn touch(&self, marker: &str) {
            fs::write(self.w.ws.join(marker), "").unwrap();
        }
    }

    #[test]
    fn rules_are_checked_in_order_inside_their_scope() {
        let d = Drill::new(
            &[when_in("a/b/c"), exact("../..", "exactly"), absolute("absolute")],
            "a/b/c",
        );
        assert_eq!(
            [d.decide("a/b/c", "/etc"), d.decide("a/b/c", "../../")],
            [err("exactly"), err("absolute")]
        );
        assert_eq!(d.decide("a", "/etc"), ALLOW, "out of scope");
        let d = Drill::new(&[exact("../..", "exactly")], "a");
        assert_eq!(
            ["../..", "../../", " ../.. "].map(|target| d.decide("a/b/c", target)),
            [ALLOW; 3]
        );
        assert_eq!(d.decide("a/b/c", "/"), err("exactly"));
        // `cd -` is a target like any other, and a scope does not need a rule to be useful.
        let d = Drill::new(&[when_in("b"), exact("-", "back")], "a");
        assert_eq!([d.decide("a", ".."), d.decide("b", "-")], [ALLOW, ALLOW]);
        assert_eq!(d.decide("b", ".."), err("back"));
    }

    #[test]
    fn a_bare_cd_goes_home_and_gets_the_message_of_the_rule_it_fails() {
        let d = Drill::new(&[within("a", "stay")], "a");
        assert_eq!([d.decide("a/b", ""), d.decide("a/b", "../..")], [err("stay"), err("stay")]);
        assert_eq!(d.decide("a/b", "-"), ALLOW, "an unknown destination is the shell's to refuse");
        let d = Drill::new(&[absolute("absolute")], "a");
        assert_eq!(d.decide("a/b", ""), err("absolute"));
    }

    #[test]
    fn enforcement_is_lifted_by_the_drill_and_by_the_way_back_to_the_start() {
        let d = Drill::new(&[exact("..", "only ..")], "a");
        assert_eq!(d.decide("a/b", "../.."), err("only .."));
        // The start directory is always reachable: by path, by `cd -` and by a longer way round.
        let start = d.w.root().join("a").display().to_string();
        let back = [
            d.decide("a/b/c", &start),
            d.go("a/b/c", "-", Some("a")).0,
            d.decide("x", "../a/b/.."),
        ];
        assert_eq!(back, [ALLOW; 3]);
        d.touch(CD_DONE);
        assert_eq!(d.decide("a/b", "../.."), ALLOW);
    }

    #[test]
    fn evidence_needs_every_rule_and_the_arrival_but_lifting_does_not_matter() {
        let d = Drill::new(&[when_in("x"), exact("-", "only -"), arrive("a")], "a");
        // `cd -` back to the start (a lifted move) still counts when the rules pass.
        assert_eq!(d.go("x", "-", Some("a")), (ALLOW, true));
        assert_eq!(d.go("x", "-", Some("x")), (ALLOW, false), "wrong arrival");
        assert_eq!(d.go("a", "-", Some("a")), (ALLOW, false), "out of scope");
        assert_eq!(d.go("x", "..", None), (err("only -"), false), "a failed rule");
        let d = Drill::new(&[exact("..", "only ..")], "a");
        assert_eq!(
            [d.go("a/b", "..", None), d.go("a/b", "../..", None)],
            [(ALLOW, true), (err("only .."), false)]
        );
    }
}
