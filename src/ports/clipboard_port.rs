// SCAFFOLD: true
// Bootstrapped by DISTILL wave 2026-05-18.

use crate::error::Result;
use crate::ports::config_port::Probe;

/// Driven port for writing to the system clipboard.
///
/// Probe contract: `probe()` writes sentinel string "rcl-probe-sentinel" and reads it back.
/// On SSH/headless environments, `write()` returns `Err` (never panics).
/// Clipboard probe failure is NON-FATAL — the composition root sets
/// `AppConfig.clipboard_available = false` and the TUI degrades gracefully.
pub trait ClipboardPort: Probe {
    /// Write `text` to the system clipboard.
    ///
    /// # Errors
    ///
    /// - `RustyCommitListerError::ClipboardUnavailable` if the clipboard is inaccessible
    ///   (SSH session, headless environment, arboard init failure).
    fn write(&self, text: &str) -> Result<()>;
}

/// In-memory fake clipboard implementing `ClipboardPort` for tests.
///
/// `write()` captures to an internal Vec. `probe()` always returns Ok.
/// Validates input contracts identically to `ArboardClipboardAdapter`
/// (empty string is rejected — real clipboard would silently no-op but that hides wiring bugs).
#[cfg(test)]
pub mod fake {
    use super::*;
    use crate::ports::config_port::Probe;
    use std::cell::RefCell;

    pub struct FakeClipboard {
        pub written: RefCell<Vec<String>>,
    }

    impl FakeClipboard {
        pub fn new() -> Self {
            Self {
                written: RefCell::new(Vec::new()),
            }
        }

        pub fn last_written(&self) -> Option<String> {
            self.written.borrow().last().cloned()
        }
    }

    impl Probe for FakeClipboard {
        fn probe(&self) -> Result<()> {
            Ok(())
        }
    }

    impl ClipboardPort for FakeClipboard {
        fn write(&self, text: &str) -> Result<()> {
            assert!(
                !text.is_empty(),
                "ClipboardPort::write called with empty string — contract violation"
            );
            self.written.borrow_mut().push(text.to_string());
            Ok(())
        }
    }
}
