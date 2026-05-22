use std::ffi::OsStr;
use std::path::PathBuf;

use chrono::Local;
use walkdir::WalkDir;

use crate::domain::model::CommitRecord;
use crate::error::{Result, RustyCommitListerError};
use crate::parser::parse_note;
use crate::ports::config_port::Probe;
use crate::ports::vault_port::VaultScanPort;

/// Maximum directory depth to walk when scanning the vault.
/// Prevents accidental full-filesystem traversal if `vault_path` is misconfigured.
const VAULT_SCAN_MAX_DEPTH: usize = 10;

/// Adapter that walks the Obsidian vault directory using `walkdir 2`,
/// filters daily notes by date range using `chrono 0.4`, and calls
/// `parse_note()` on each discovered file.
///
/// Unicode path handling: `vault_path` may contain emoji (e.g. `📅 Diaries`).
/// The adapter relies on `PathBuf` / `OsString` for all path operations - no
/// manual string manipulation of paths. `WalkDir` handles `OsStr` natively; OQ-1
/// is resolved.
///
/// Probe contract: verify `vault_path` exists and is a directory.
pub struct WalkdirScanAdapter {
    /// The root vault directory to scan.
    pub vault_path: PathBuf,
}

impl WalkdirScanAdapter {
    pub fn new(vault_path: PathBuf) -> Self {
        Self { vault_path }
    }
}

impl Probe for WalkdirScanAdapter {
    fn probe(&self) -> Result<()> {
        if self.vault_path.is_dir() {
            Ok(())
        } else {
            Err(RustyCommitListerError::vault(format!(
                "vault path {:?} does not exist or is not a directory",
                self.vault_path.display()
            )))
        }
    }
}

impl VaultScanPort for WalkdirScanAdapter {
    fn scan(&self, days_back: u32) -> Result<Vec<CommitRecord>> {
        let today = Local::now().date_naive();
        let window_start = today - chrono::Duration::days(i64::from(days_back));

        let mut records: Vec<CommitRecord> = WalkDir::new(&self.vault_path)
            .max_depth(VAULT_SCAN_MAX_DEPTH)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.path().extension() == Some(OsStr::new("md")))
            .filter_map(|entry| {
                let stem = entry.path().file_stem().and_then(|s| s.to_str())?;
                let note_date = chrono::NaiveDate::parse_from_str(stem, "%Y-%m-%d").ok()?;
                if note_date >= window_start {
                    Some(entry)
                } else {
                    None
                }
            })
            .flat_map(|entry| parse_note(entry.path()))
            .collect();

        records.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.time.cmp(&a.time)));

        Ok(records)
    }
}
