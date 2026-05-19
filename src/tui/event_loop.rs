use std::io::Stdout;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::Event;

use crate::domain::events::AppEvent;
use crate::domain::model::{AppModel, CommitRecord};

/// Event poll interval — balances responsiveness with CPU usage.
const POLL_INTERVAL: Duration = Duration::from_millis(250);

/// The TUI event loop struct. Owns the terminal and drives the Elm update→view cycle.
///
/// On creation: enters raw mode and alt screen via crossterm.
/// On drop (or explicit `restore()`): exits raw mode and alt screen, restoring the terminal.
pub struct TuiEventLoop {
    terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<Stdout>>,
    restored: bool,
}

impl TuiEventLoop {
    /// Create a new TuiEventLoop, entering raw mode and the alternate screen.
    pub fn new() -> Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
        let backend = ratatui::backend::CrosstermBackend::new(std::io::stdout());
        let terminal = ratatui::Terminal::new(backend)?;
        Ok(Self {
            terminal,
            restored: false,
        })
    }

    /// Drive the Elm update→view cycle until `model.quit` is true.
    ///
    /// Polls for crossterm events every 250ms. On each event, translates to an
    /// AppEvent, calls `update()`, and re-renders via `view()`. On timeout, sends
    /// a Tick event to allow spinner animation or deferred state refresh.
    ///
    /// When `model.loading` becomes true after an update (e.g. from pressing `r`
    /// in Browse mode), `reload_fn` is called synchronously to fetch fresh commit
    /// records, and a `LoadComplete` event is immediately dispatched before the
    /// next draw — completing the re-scan within the same iteration.
    pub fn run(
        &mut self,
        initial_model: AppModel,
        mut reload_fn: impl FnMut() -> Vec<CommitRecord>,
    ) -> Result<()> {
        let mut model = initial_model;
        loop {
            self.terminal
                .draw(|frame| crate::tui::view::view(&model, frame))?;
            if crossterm::event::poll(POLL_INTERVAL)? {
                let evt = crossterm::event::read()?;
                let app_event = translate_event(evt);
                model = crate::domain::update::update(model, app_event);
                if model.loading {
                    let records = reload_fn();
                    model = crate::domain::update::update(
                        model,
                        AppEvent::LoadComplete(records),
                    );
                }
                if model.quit {
                    break;
                }
            } else {
                model = crate::domain::update::update(model, crate::domain::events::AppEvent::Tick);
            }
        }
        Ok(())
    }

    /// Exit raw mode and the alternate screen, restoring the original terminal state.
    ///
    /// Safe to call multiple times — subsequent calls after the first are no-ops.
    pub fn restore(&mut self) -> Result<()> {
        if !self.restored {
            crossterm::terminal::disable_raw_mode()?;
            crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
            self.restored = true;
        }
        Ok(())
    }
}

impl Drop for TuiEventLoop {
    /// Ensure terminal is always restored, even on panic.
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Translate a raw crossterm Event into an AppEvent for domain dispatch.
fn translate_event(evt: Event) -> AppEvent {
    match evt {
        Event::Key(key_event) => AppEvent::KeyPress(key_event),
        _ => AppEvent::Tick,
    }
}
