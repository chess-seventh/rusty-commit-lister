// SCAFFOLD: true
// Bootstrapped by DISTILL wave 2026-05-18.

use crossterm::event::KeyEvent;

use crate::domain::model::CommitRecord;

/// All events that can drive state transitions in the Elm/MVU update function.
///
/// The TUI event loop translates crossterm keyboard events into `AppEvent` variants
/// and dispatches them to `update(model, event) -> AppModel`.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// A raw keyboard event from crossterm (translated to domain events by the loop).
    KeyPress(KeyEvent),
    /// Vault scan completed successfully with the loaded commit records.
    LoadComplete(Vec<CommitRecord>),
    /// Vault scan failed; the string is a human-readable error message.
    LoadFailed(String),
    /// Clipboard write result: Ok(()) on success, Err(msg) on failure.
    ClipboardResult(Result<(), String>),
    /// Periodic tick event (250ms) for spinner animation or deferred refresh.
    Tick,
}
