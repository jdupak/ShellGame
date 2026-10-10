//! The game engine: levels, fixtures, answers, requirements and `cd` policies.
//!
//! Level authors need one import, [`prelude`]:
//!
//! ```
//! use shellgame::engine::prelude::*;
//!
//! // One function per level; the section lists them. Ids are explicit, so reordering
//! // never renumbers anything.
//! fn challenge() -> Level {
//!     level(8, "Integrační výzva")
//!         .text("Najděte heslo v sousedním adresáři.")
//!         .start(".")
//!         .hints(["Ze startu přejděte do 'challenge/room1'.", "Přečtěte hint.txt."])
//!         .file("challenge/room1/hint.txt", "Heslo je v sousedním adresáři room2.\n")
//!         .file("challenge/room2/password.txt", "navigator\n")
//!         .answer(
//!             exact("navigator")
//!                 .ignore_case()
//!                 .error("Heslo není správné. Hledejte password.txt v room2.")
//!                 .required("Musíte zadat heslo: shellgame submit <hodnota>"),
//!         )
//!         .require(at("challenge/room2"))
//!         .solution("cd challenge/room2\nshellgame submit \"$(cat password.txt)\"")
//!         .success("Výborně!")
//! }
//!
//! let section = Section::new(2, "level-2")
//!     .intro(intro("Práce se soubory", "Úvod."))
//!     .levels([challenge()]);
//! let catalog = Catalog::new(vec![section]);
//! assert!(catalog.validate().is_empty());
//! assert_eq!(catalog.next("2.0".parse().unwrap()), Ok(Some(LevelId::new(2, 8))));
//! ```
//!
//! The pieces, from the bottom up:
//!
//! * [`fixture`]: what a level's files look like, applied with repair semantics.
//! * [`answer`] and [`require`]: how work is judged. Requirements are unconditional and run
//!   before the answer; feedback is data ([`Check`]), never an error.
//! * [`cd`]: strict `cd` policies and the anti-soft-lock contract.
//! * [`level`] and [`section`]: the builder API and run-time behaviour
//!   ([`Level::prepare`](level::Level::prepare), [`Level::validate`](level::Level::validate)).
//! * [`catalog`]: ordering, navigation and [`Catalog::validate`](catalog::Catalog::validate).
//!
//! Nothing here reads the process working directory, `HOME` or the environment: the caller
//! passes a [`Ctx`] explicitly, so every rule is a plain function of its inputs.

pub mod answer;
pub mod catalog;
pub mod cd;
pub mod check;
pub mod ctx;
pub mod fd;
pub mod fixture;
pub mod id;
pub mod level;
mod lint;
mod pytext;
pub mod require;
#[cfg(test)]
mod sample;
pub mod section;

pub use check::Check;
pub use ctx::Ctx;
pub use fixture::{Content, Fixture, FixtureError};
pub use id::LevelId;
pub use section::Section;

/// Everything a level author needs, in one import: the builder functions and the types they
/// return or take.
pub mod prelude {
    pub use crate::engine::answer::{
        Answer, MistakeKey, exact, exact_with, int, one_of, parts, suffix,
    };
    pub use crate::engine::catalog::{Catalog, UnknownLevelError};
    pub use crate::engine::cd::{CdDecision, CdMove, CdPolicy, policy};
    pub use crate::engine::check::Check;
    pub use crate::engine::ctx::Ctx;
    pub use crate::engine::fd::{FdTarget, Fds};
    pub use crate::engine::fixture::{Content, Fixture};
    pub use crate::engine::id::LevelId;
    pub use crate::engine::level::{Level, LevelKind, Solution, Start, intro, level};
    pub use crate::engine::require::{
        Requirement, at, at_home, cd_evidence, check, content, dir, dir_mode, evidence,
        fd_evidence, file, line_count, mode, moved, no_dir, no_file, perm_bits, same,
    };
    pub use crate::engine::section::Section;
    pub use crate::markers::{Marker, Markers};
    pub use crate::messages;
}
