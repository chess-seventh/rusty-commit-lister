//! Terminal UI layer: the pure `view` renderer and the crossterm event loop.
//!
//! `view` reads an `AppModel` and draws a frame (no mutation). `event_loop` owns
//! the terminal, translates crossterm events into `AppEvent`s, and drives the
//! `update` -> `view` cycle. Effects (reload, clipboard) are injected as closures.

pub mod event_loop;
pub mod view;
