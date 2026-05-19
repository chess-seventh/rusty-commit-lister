// SCAFFOLD: true
// Bootstrapped by DISTILL wave 2026-05-18.
// Replace panic!() bodies with real implementation in DELIVER wave.

use std::path::PathBuf;

use crate::domain::model::AppConfig;
use crate::error::Result;
use crate::ports::config_port::{ConfigPort, Probe};

/// Adapter that reads `~/.config/rusty-commit-lister/config.toml` and
/// produces a validated `AppConfig`.
///
/// Config precedence applied by the composition root:
/// CLI flags > env vars > TOML file > defaults.
///
/// TOML schema:
/// ```toml
/// vault_path = "~/Documents/Wiki/📅 Diaries/0. Journal"
/// scan_days_back = 7
/// repo_filter = "dotfiles"  # optional
/// ```
///
/// `~` in `vault_path` is expanded using the `HOME` environment variable.
pub struct TomlConfigAdapter {
    /// Path to the config file (usually `~/.config/rusty-commit-lister/config.toml`).
    pub config_path: PathBuf,
}

impl TomlConfigAdapter {
    // SCAFFOLD: true
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }
}

impl Probe for TomlConfigAdapter {
    // SCAFFOLD: true
    fn probe(&self) -> Result<()> {
        panic!("Not yet implemented -- RED scaffold")
    }
}

impl ConfigPort for TomlConfigAdapter {
    // SCAFFOLD: true
    fn load(&self) -> Result<AppConfig> {
        panic!("Not yet implemented -- RED scaffold")
    }
}
