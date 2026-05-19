// SCAFFOLD: true
// Bootstrapped by DISTILL wave 2026-05-18.

use anyhow::Result;

use crate::domain::model::AppModel;

/// The TUI event loop struct. Owns the terminal and drives the Elm update→view cycle.
///
/// On creation: enters raw mode and alt screen via crossterm.
/// On drop (or explicit `restore()`): exits raw mode and alt screen, restoring the terminal.
/// SIGINT (Ctrl+C) is handled via a registered handler that calls restore() before exit.
pub struct TuiEventLoop;

impl TuiEventLoop {
    // SCAFFOLD: true
    pub fn new() -> Result<Self> {
        panic!("Not yet implemented -- RED scaffold")
    }

    // SCAFFOLD: true
    pub fn run(&mut self, _initial_model: AppModel) -> Result<()> {
        panic!("Not yet implemented -- RED scaffold")
    }

    // SCAFFOLD: true
    pub fn restore(&mut self) -> Result<()> {
        panic!("Not yet implemented -- RED scaffold")
    }
}

impl Drop for TuiEventLoop {
    fn drop(&mut self) {
        // Ensure terminal is always restored, even on panic.
        // Real implementation calls crossterm::execute! to exit alt screen and raw mode.
        // SCAFFOLD: no-op drop — real implementation required.
    }
}
