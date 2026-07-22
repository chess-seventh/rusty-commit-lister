//! Port traits — the hexagonal boundary between domain and adapters.
//!
//! Each port is a trait with a `Probe` supertrait for startup health checks:
//! `ConfigPort` (load config), `VaultScanPort` (scan the vault), `ClipboardPort`
//! (write to the system clipboard). Adapters in [`crate::adapters`] implement them.

pub mod clipboard_port;
pub mod config_port;
pub mod vault_port;
