//! ShellGame: an interactive terminal course for filesystem navigation and shell basics.
//!
//! The binary runs inside a wrapped bash or fish subshell. Rust owns all game rules
//! and persistence; the shell templates only forward events and apply a small,
//! versioned command protocol.

pub mod authoring;
pub mod cli;
pub mod engine;
pub mod env;
pub mod levels;
pub mod markers;
pub mod messages;
pub mod paths;
pub mod session;
pub mod shell;
pub mod state;
pub mod ui;
