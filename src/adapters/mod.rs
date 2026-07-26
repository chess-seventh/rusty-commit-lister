//! Driven adapters implementing the port traits with real I/O.
//!
//! Each adapter pairs a `Probe` implementation (structural health check) with its
//! port implementation: config from TOML, vault scan via `walkdir`, clipboard via
//! the OSC 52 terminal escape. Adapters depend on the domain; the domain never
//! depends on them.

pub mod osc52_clipboard;
pub mod toml_config;
pub mod walkdir_vault;
