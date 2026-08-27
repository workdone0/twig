//! Twig library crate.
//!
//! The CLI binary in `main.rs` is the public entrypoint; this `lib` exposes
//! the inner modules for integration tests and downstream consumers.

pub mod adapters;
pub mod cli;
pub mod core;
pub mod tui;
