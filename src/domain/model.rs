use std::path::PathBuf;

/// Default number of days to scan back for daily notes when no config overrides it.
pub const DEFAULT_SCAN_DAYS_BACK: u32 = 7;

/// The current interaction mode of the TUI.
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    /// Main browse mode - j/k navigation, q/Esc to exit.
    Browse,
    /// Inline search mode - / activates, Esc clears.
    Search,
    /// Detail overlay mode - Enter opens, Esc closes.
    Detail,
    /// Repository picker overlay mode - f opens, Esc closes.
    RepoPicker,
}

/// A single commit record parsed from an Obsidian daily note Markdown table.
///
/// Data contract from rusty-commit-saver:
///   FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL
#[derive(Debug, Clone, PartialEq)]
pub struct CommitRecord {
    /// The folder path where the commit was made (FOLDER column).
    pub folder: String,
    /// The time of the commit as a string "HH:MM" (TIME column).
    pub time: String,
    /// The full commit message (COMMIT MESSAGE column).
    pub message: String,
    /// The repository URL, or None if the column was absent/empty (REPOSITORY URL column).
    pub url: Option<String>,
    /// The date this commit was parsed from (derived from the note filename YYYY-MM-DD.md).
    pub date: String,
}

/// Application configuration loaded from config.toml and CLI flags.
///
/// Precedence: CLI flags > env vars > config.toml > defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    /// Absolute path to the Obsidian vault directory (may contain emoji like 📅).
    pub vault_path: PathBuf,
    /// Number of days back to scan for daily notes (must be > 0).
    pub scan_days_back: u32,
    /// Optional repository name pre-filter applied at load time.
    pub repo_filter: Option<String>,
    /// Whether the system clipboard is available (set after `probe()`).
    pub clipboard_available: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            vault_path: PathBuf::from(""),
            scan_days_back: DEFAULT_SCAN_DAYS_BACK,
            repo_filter: None,
            clipboard_available: false,
        }
    }
}

/// The full application state. Owned by the TUI event loop.
///
/// This is the Model in the Elm/MVU architecture.
/// All mutations produce a new `AppModel` - no shared mutable state.
#[derive(Debug, Clone)]
pub struct AppModel {
    pub config: AppConfig,
    /// All commit rows loaded from the vault scan (sorted newest-first).
    pub commit_rows: Vec<CommitRecord>,
    /// Currently visible rows after applying `search_query` and `repo_filter`.
    pub filtered_rows: Vec<CommitRecord>,
    /// Current TUI interaction mode.
    pub mode: AppMode,
    /// Index of the selected row in `filtered_rows`.
    pub cursor: usize,
    /// Active inline search query (empty string = no filter).
    pub search_query: String,
    /// Active TUI-level repository filter (None = show all repos).
    pub active_repo_filter: Option<String>,
    /// Index within the repo picker overlay (used in `RepoPicker` mode).
    pub picker_cursor: usize,
    /// Whether the tool is currently loading data.
    pub loading: bool,
    /// Error message to display in the status bar, if any.
    pub error_message: Option<String>,
    /// Confirmation message to display briefly in the status bar.
    pub status_message: Option<String>,
    /// URL pending clipboard write - set by 'c' in Detail mode, cleared by `ClipboardResult`.
    /// The event loop picks this up to fire the actual clipboard write effect.
    pub clipboard_pending: Option<String>,
    /// Whether the application should exit after the next event loop tick.
    pub quit: bool,
    /// Number of rows to scroll per `PageDown` / `PageUp` key press.
    pub page_size: usize,
}

impl AppModel {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            commit_rows: Vec::new(),
            filtered_rows: Vec::new(),
            mode: AppMode::Browse,
            cursor: 0,
            search_query: String::new(),
            active_repo_filter: None,
            picker_cursor: 0,
            loading: true,
            error_message: None,
            status_message: None,
            clipboard_pending: None,
            quit: false,
            page_size: 10,
        }
    }
}
