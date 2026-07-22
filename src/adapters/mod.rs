//! Driven adapters implementing the port traits with real I/O.
//!
//! Each adapter pairs a `Probe` implementation (structural health check) with its
//! port implementation: config from TOML, vault scan via `walkdir`, clipboard via
//! `arboard`. Adapters depend on the domain; the domain never depends on them.

pub mod arboard_clipboard;
pub mod toml_config;
pub mod walkdir_vault;
