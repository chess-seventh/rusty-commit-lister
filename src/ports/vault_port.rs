// SCAFFOLD: true
// Bootstrapped by DISTILL wave 2026-05-18.

use crate::domain::model::CommitRecord;
use crate::error::Result;
use crate::ports::config_port::Probe;

/// Driving port for scanning the Obsidian vault for commit records.
///
/// The adapter (WalkdirScanAdapter) walks `vault_path` for files named `YYYY-MM-DD.md`
/// within the `days_back` window, calls `parse_note()` on each, and merges results
/// sorted newest-first.
pub trait VaultScanPort: Probe {
    /// Scan the vault for commit records within the given date window.
    ///
    /// Returns all `CommitRecord` values sorted newest-first (by date + time).
    /// A missing vault path detected at scan time is a non-fatal empty result with a warning.
    ///
    /// # Errors
    ///
    /// - `RustyCommitListerError::Vault` if the vault directory is inaccessible.
    fn scan(&self, days_back: u32) -> Result<Vec<CommitRecord>>;
}
