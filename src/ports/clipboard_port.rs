// SCAFFOLD: true
// Bootstrapped by DISTILL wave 2026-05-18.
//! The [`ClipboardPort`] trait and an in-memory fake for tests.

use crate::error::Result;
use crate::ports::config_port::Probe;

/// Driven port for writing to the system clipboard.
///
/// Probe contract: `probe()` reports whether copying is possible without side
/// effects (the OSC 52 adapter needs no backend, so it always succeeds).
/// Probe failure is NON-FATAL - the composition root sets
/// `AppConfig.clipboard_available = false` and the TUI degrades gracefully.
pub trait ClipboardPort: Probe {
    /// Write `text` to the system clipboard.
    ///
    /// # Errors
    ///
    /// - `RustyCommitListerError::ClipboardUnavailable` if the copy could not be
    ///   performed (e.g. failure writing the OSC 52 escape to the terminal).
    fn write(&self, text: &str) -> Result<()>;
}

/// In-memory fake clipboard implementing `ClipboardPort` for tests.
///
/// `write()` captures to an internal Vec. `probe()` always returns Ok.
/// Rejects the empty string to surface wiring bugs (a real clipboard would
/// silently no-op).
#[cfg(test)]
pub mod fake {
    use super::*;
    use crate::ports::config_port::Probe;
    use std::cell::RefCell;

    /// Test double capturing every `write()` in an internal buffer.
    pub struct FakeClipboard {
        /// Every string passed to `write()`, in call order.
        pub written: RefCell<Vec<String>>,
    }

    impl Default for FakeClipboard {
        fn default() -> Self {
            Self::new()
        }
    }

    impl FakeClipboard {
        /// Create an empty fake clipboard.
        pub fn new() -> Self {
            Self {
                written: RefCell::new(Vec::new()),
            }
        }

        /// Return the most recently written string, if any.
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
                "ClipboardPort::write called with empty string - contract violation"
            );
            self.written.borrow_mut().push(text.to_string());
            Ok(())
        }
    }
}
