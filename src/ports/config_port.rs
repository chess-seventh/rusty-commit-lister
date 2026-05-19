// SCAFFOLD: true
// Bootstrapped by DISTILL wave 2026-05-18.

use crate::domain::model::AppConfig;
use crate::error::Result;

/// Probe supertrait: every port adapter must implement structural health verification.
///
/// Probe contract per architecture brief:
/// - `TomlConfigAdapter::probe()`: read config path; parse and validate required fields.
/// - Returns structured error on any failure.
pub trait Probe {
    fn probe(&self) -> Result<()>;
}

/// Driving port for loading application configuration.
///
/// Config precedence: CLI flags > env vars > config.toml > defaults.
/// `~` in `vault_path` must be expanded to the user's home directory.
pub trait ConfigPort: Probe {
    /// Load and validate configuration. Returns `AppConfig` with all fields populated.
    ///
    /// # Errors
    ///
    /// - `RustyCommitListerError::Config` if the file is present but contains invalid values.
    ///   The error message must name the invalid field and the config file path.
    /// - Missing config file is NOT an error — callers receive defaults with a notice.
    fn load(&self) -> Result<AppConfig>;
}
