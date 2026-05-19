use std::path::PathBuf;

use serde::Deserialize;

use crate::domain::model::AppConfig;
use crate::error::{Result, RustyCommitListerError};
use crate::ports::config_port::{ConfigPort, Probe};

/// Private serde struct matching the TOML schema.
/// All fields are optional so missing keys fall back to AppConfig defaults.
#[derive(Deserialize)]
struct TomlFileConfig {
    vault_path: Option<String>,
    scan_days_back: Option<u32>,
    repo_filter: Option<String>,
}

/// Adapter that reads `config.toml` from a given path and produces a
/// validated `AppConfig`.
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
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }
}

impl Probe for TomlConfigAdapter {
    /// Verifies that the config file, if present, is readable.
    /// Absent file is not an error — returns `Ok(())`.
    fn probe(&self) -> Result<()> {
        if self.config_path.exists() {
            std::fs::File::open(&self.config_path).map_err(|e| {
                RustyCommitListerError::config(format!(
                    "Cannot read config file {:?}: {}",
                    self.config_path, e
                ))
            })?;
        }
        Ok(())
    }
}

impl ConfigPort for TomlConfigAdapter {
    fn load(&self) -> Result<AppConfig> {
        if !self.config_path.exists() {
            return Ok(AppConfig::default());
        }

        let content = std::fs::read_to_string(&self.config_path).map_err(|e| {
            RustyCommitListerError::config(format!(
                "Failed to read config {:?}: {}",
                self.config_path, e
            ))
        })?;

        let file_config: TomlFileConfig = toml::from_str(&content).map_err(|e| {
            RustyCommitListerError::config(format!("Invalid TOML in {:?}: {}", self.config_path, e))
        })?;

        let scan_days_back = file_config.scan_days_back.unwrap_or(7);
        if scan_days_back == 0 {
            return Err(RustyCommitListerError::config(format!(
                "scan_days_back must be > 0 in config file {:?}",
                self.config_path
            )));
        }

        let vault_path = match file_config.vault_path {
            Some(raw_path) => {
                let expanded = expand_tilde(raw_path);
                PathBuf::from(expanded)
            }
            None => AppConfig::default().vault_path,
        };

        Ok(AppConfig {
            vault_path,
            scan_days_back,
            repo_filter: file_config.repo_filter,
            clipboard_available: false,
        })
    }
}

/// Expands a leading `~` in a path string to the value of the `HOME`
/// environment variable. If `HOME` is unset, the `~` is left in place.
fn expand_tilde(path: String) -> String {
    if path.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_default();
        path.replacen('~', &home, 1)
    } else {
        path
    }
}
