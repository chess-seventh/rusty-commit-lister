use thiserror::Error;

/// Custom error types for rusty-commit-lister
#[derive(Error, Debug)]
pub enum RustyCommitListerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Validation error: {field} is invalid")]
    Validation { field: String },

    /// Vault directory or file access failed.
    #[error("Vault error: {message}")]
    Vault { message: String },

    /// Parser encountered unrecoverable structure (rare — normal skip-and-log is non-error).
    #[error("Parse error: {message}")]
    Parse { message: String },

    /// System clipboard is unavailable (SSH session, headless environment).
    /// This is NON-FATAL — the composition root degrades clipboard capability.
    #[error("Clipboard unavailable: {reason}")]
    ClipboardUnavailable { reason: String },

    #[error("External service error: {service}")]
    ExternalService { service: String },

    #[error("Generic error: {0}")]
    Generic(String),
}

impl RustyCommitListerError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    pub fn validation(field: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
        }
    }

    pub fn vault(message: impl Into<String>) -> Self {
        Self::Vault {
            message: message.into(),
        }
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }

    pub fn clipboard_unavailable(reason: impl Into<String>) -> Self {
        Self::ClipboardUnavailable {
            reason: reason.into(),
        }
    }

    pub fn external_service(service: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
        }
    }
}

/// Result type for this crate
pub type Result<T> = std::result::Result<T, RustyCommitListerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = RustyCommitListerError::config("test message");
        assert!(matches!(error, RustyCommitListerError::Config { .. }));
    }

    #[test]
    fn test_error_display() {
        let error = RustyCommitListerError::validation("test_field");
        assert_eq!(error.to_string(), "Validation error: test_field is invalid");
    }
}
