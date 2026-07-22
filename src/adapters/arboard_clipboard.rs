//! `arboard`-backed [`ClipboardPort`] adapter (ADR-004).

use arboard::Clipboard;

use crate::error::Result;
use crate::ports::clipboard_port::ClipboardPort;
use crate::ports::config_port::Probe;

/// Adapter implementing `ClipboardPort` via the arboard crate.
///
/// Zero-size struct — no state stored. `arboard::Clipboard` is NOT `Send` or `Sync`,
/// so it is created fresh on every `write()` and `probe()` call rather than stored.
///
/// Probe contract: writes sentinel "rcl-probe-sentinel" and reads it back.
/// Returns `Err(ClipboardUnavailable)` on SSH sessions, headless environments,
/// or any arboard initialisation failure.
pub struct ArboardClipboardAdapter;

impl ArboardClipboardAdapter {
    /// Create a new adapter. Holds no state — the clipboard is opened per call.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ArboardClipboardAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Probe for ArboardClipboardAdapter {
    fn probe(&self) -> Result<()> {
        let mut cb = Clipboard::new().map_err(|e| {
            crate::error::RustyCommitListerError::clipboard_unavailable(e.to_string())
        })?;
        cb.set_text("rcl-probe-sentinel").map_err(|e| {
            crate::error::RustyCommitListerError::clipboard_unavailable(e.to_string())
        })?;
        cb.get_text().map_err(|e| {
            crate::error::RustyCommitListerError::clipboard_unavailable(e.to_string())
        })?;
        Ok(())
    }
}

impl ClipboardPort for ArboardClipboardAdapter {
    fn write(&self, text: &str) -> Result<()> {
        let mut cb = Clipboard::new().map_err(|e| {
            crate::error::RustyCommitListerError::clipboard_unavailable(e.to_string())
        })?;
        cb.set_text(text).map_err(|e| {
            crate::error::RustyCommitListerError::clipboard_unavailable(e.to_string())
        })?;
        Ok(())
    }
}
