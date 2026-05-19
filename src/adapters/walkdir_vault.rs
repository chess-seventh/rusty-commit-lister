// SCAFFOLD: true
// Bootstrapped by DISTILL wave 2026-05-18.
// Replace panic!() bodies with real implementation in DELIVER wave.

use std::path::PathBuf;

use crate::domain::model::CommitRecord;
use crate::error::Result;
use crate::ports::config_port::Probe;
use crate::ports::vault_port::VaultScanPort;

/// Adapter that walks the Obsidian vault directory using `walkdir 2`,
/// filters daily notes by date range using `chrono 0.4`, and calls
/// `parse_note()` on each discovered file.
///
/// Unicode path handling: `vault_path` may contain emoji (e.g. `📅 Diaries`).
/// The adapter relies on `PathBuf` / `OsString` for all path operations — no
/// manual string manipulation of paths.
///
/// Probe contract: verify `vault_path` exists and is a directory; open one file
/// in the path; round-trip the emoji path segment via OsString and verify identity.
pub struct WalkdirScanAdapter {
    /// The root vault directory to scan.
    pub vault_path: PathBuf,
}

impl WalkdirScanAdapter {
    // SCAFFOLD: true
    pub fn new(vault_path: PathBuf) -> Self {
        Self { vault_path }
    }
}

impl Probe for WalkdirScanAdapter {
    // SCAFFOLD: true
    fn probe(&self) -> Result<()> {
        panic!("Not yet implemented -- RED scaffold")
    }
}

impl VaultScanPort for WalkdirScanAdapter {
    // SCAFFOLD: true
    fn scan(&self, days_back: u32) -> Result<Vec<CommitRecord>> {
        panic!("Not yet implemented -- RED scaffold")
    }
}
