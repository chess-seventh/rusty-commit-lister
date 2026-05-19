// SCAFFOLD: true
// Bootstrapped by DISTILL wave 2026-05-18.

use ratatui::Frame;

use crate::domain::model::AppModel;

/// Pure render function — Elm/MVU View.
///
/// Takes a reference to the current AppModel and a mutable Frame reference.
/// Does NOT mutate model state.
/// Renders the appropriate widget tree for the current AppMode.
pub fn view(_model: &AppModel, _frame: &mut Frame) {
    panic!("Not yet implemented -- RED scaffold")
}
