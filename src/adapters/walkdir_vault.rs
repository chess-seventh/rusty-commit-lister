//! `walkdir`-backed [`VaultScanPort`] adapter with date-window filtering.

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

/// Supplies "today" as a [`NaiveDate`](chrono::NaiveDate).
///
/// Injected into [`WalkdirScanAdapter`] so the `scan(days_back)` window is
/// deterministic under test: the real wall-clock never leaks into the date
/// filter, so date-based fixtures cannot rot. Production uses [`SystemClock`].
pub trait Clock {
    /// The current local date.
    fn today(&self) -> chrono::NaiveDate;
}

/// Production [`Clock`] backed by the system's local time zone.
pub struct SystemClock;

impl Clock for SystemClock {
    fn today(&self) -> chrono::NaiveDate {
        Local::now().date_naive()
    }
}

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
    /// Source of "today" for the scan window (injected; see [`Clock`]).
    clock: Box<dyn Clock>,
}

impl WalkdirScanAdapter {
    /// Create an adapter that scans `vault_path` for daily notes,
    /// using the [`SystemClock`] for the scan window.
    pub fn new(vault_path: PathBuf) -> Self {
        Self::with_clock(vault_path, Box::new(SystemClock))
    }

    /// Create an adapter with an injected [`Clock`].
    ///
    /// Lets tests pin "today" to a fixed date so the `scan(days_back)` window is
    /// deterministic and fixture filenames need not track the real calendar.
    pub fn with_clock(vault_path: PathBuf, clock: Box<dyn Clock>) -> Self {
        Self { vault_path, clock }
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
        let today = self.clock.today();
        // days_back == 0 means "no window": include every validly-dated note.
        let window_start =
            (days_back > 0).then(|| today - chrono::Duration::days(i64::from(days_back)));

        let mut records: Vec<CommitRecord> = WalkDir::new(&self.vault_path)
            .max_depth(VAULT_SCAN_MAX_DEPTH)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.path().extension() == Some(OsStr::new("md")))
            .filter_map(|entry| {
                let stem = entry.path().file_stem().and_then(|s| s.to_str())?;
                let note_date = chrono::NaiveDate::parse_from_str(stem, "%Y-%m-%d").ok()?;
                if window_start.map_or(true, |start| note_date >= start) {
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
