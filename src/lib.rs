//! Twig library crate.
//!
//! The CLI binary in `main.rs` is the public entrypoint; this `lib` exposes
//! the inner modules for integration tests and downstream consumers.

pub mod core;
pub mod adapters;
pub mod tui;
pub mod cli;