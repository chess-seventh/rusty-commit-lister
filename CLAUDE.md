# rusty-commit-lister

## Development Paradigm

This project follows the **functional programming** paradigm.
Use `@nw-functional-software-crafter` for implementation.

See `docs/product/architecture/adr-003.md` for the full rationale.

### Elm/MVU Architecture

- Domain: `update(model: AppModel, event: AppEvent) -> AppModel` (pure — no I/O, no mutation)
- View: `view(model: &AppModel, frame: &mut Frame)` (pure render — reads model, writes frame)
- Effects: pushed to shell layer (config load, vault scan, clipboard write)
- State machine: `AppMode` enum (`Browse | Search | Detail | RepoPicker`)

All domain modules (`src/domain/`) must have zero imports from `src/adapters/` or `src/tui/`.
Dependency direction: adapters and TUI depend on domain; domain depends on nothing.

### Port Interfaces

- `ConfigPort`: `load(&self) -> Result<AppConfig>` — reads config.toml; expands ~ in vault_path
- `VaultScanPort`: `scan(&self, days_back: u32) -> Result<Vec<CommitRecord>>` — walks vault dir, calls parser
- `ClipboardPort`: `write(&self, text: &str) -> Result<()>` — writes to system clipboard; returns Err on SSH/headless

Every port trait has a `Probe` supertrait. Every adapter must implement `probe()`.
Composition root invariant: **wire → probe → use**. Adapters that fail a fatal probe prevent startup.
Clipboard probe failure is non-fatal — sets `AppConfig.clipboard_available = false` and degrades gracefully.

### Key Design Decisions

| Decision | Outcome | ADR |
|---|---|---|
| TUI architecture | Elm/MVU — pure update()/view() | ADR-001 |
| Async loading | Sync blocking for slice-01/02; async upgrade if > 100ms | ADR-002 |
| Development paradigm | Functional Programming throughout | ADR-003 |
| Clipboard crate | arboard (cross-platform, actively maintained) | ADR-004 |

### Module Structure

```
src/
├── main.rs               ← composition root: wire adapters → probe → run TUI
├── lib.rs                ← re-export port traits and domain types
├── error.rs              ← RustyCommitListerError (thiserror)
├── domain/
│   ├── model.rs          ← AppModel, AppMode, CommitRecord, AppConfig (pure types)
│   ├── events.rs         ← AppEvent enum
│   └── update.rs         ← update(AppModel, AppEvent) -> AppModel (pure)
├── ports/
│   ├── config_port.rs    ← trait ConfigPort (+ Probe supertrait)
│   ├── vault_port.rs     ← trait VaultScanPort (+ Probe supertrait)
│   └── clipboard_port.rs ← trait ClipboardPort (+ Probe supertrait)
├── adapters/
│   ├── toml_config.rs    ← TomlConfigAdapter: ConfigPort + probe()
│   ├── walkdir_vault.rs  ← WalkdirScanAdapter: VaultScanPort + probe()
│   └── arboard_clipboard.rs ← ArboardClipboardAdapter: ClipboardPort + probe() (slice-05)
├── parser/
│   └── mod.rs            ← parse_note(path: &Path) -> Vec<CommitRecord> (pure, no trait)
├── tui/
│   ├── view.rs           ← view(&AppModel, &mut Frame) (pure render)
│   └── event_loop.rs     ← crossterm event loop; drives update() + view()
```

### Rules

- `#![forbid(unsafe_code)]` at crate root — no exceptions
- No `RefCell`, `Mutex`, `Cell` in `src/domain/` — interior mutability is an adapter concern
- Parser is a pure function, not a trait — `parse_note(path: &Path) -> Vec<CommitRecord>`
- `AppModel` derives `Clone` — ownership threading through update() requires it
- Clipboard errors are `Result::Err`, never panics — US-08 fallback requirement

### Open Spikes (resolve before implementing affected slice)

- **OQ-1** (slice-01): Unicode OsString round-trip with `📅 Diaries` path segment on macOS
- **OQ-2** (slice-05): arboard SSH -Y (X11 forwarding) write round-trip verification
