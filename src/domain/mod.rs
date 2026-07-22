//! Pure domain core (Elm/MVU): model, events, and the `update` reducer.
//!
//! Everything here is side-effect free — no I/O, no interior mutability. State
//! transitions flow through `update(model, event) -> AppModel`. This module has
//! zero imports from `adapters` or `tui`.

pub mod events;
pub mod model;
pub mod update;
