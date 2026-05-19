//! rusty-commit-lister — Obsidian commit browser TUI.
//!
//! This crate exposes port traits and domain types for integration tests.
//! Production wiring happens in `main.rs` (composition root).

#![forbid(unsafe_code)]

pub mod error;
pub use error::{Result, RustyCommitListerError};

pub mod adapters;
pub mod domain;
pub mod parser;
pub mod ports;
pub mod tui;
