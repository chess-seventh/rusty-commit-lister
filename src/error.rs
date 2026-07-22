//! Crate error type and `Result` alias.
//!
//! [`RustyCommitListerError`] is the single error enum threaded through the ports
//! and adapters. Constructors take `impl Into<String>` so call sites can pass
//! either `&str` or owned messages.

use thiserror::Error;

/// The unified error type for rusty-commit-lister.
#[derive(Error, Debug)]
pub enum RustyCommitListerError {
    /// An underlying `std::io` operation failed.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration was present but invalid (bad value, unreadable file).
    #[error("Configuration error: {message}")]
    Config {
        /// Human-readable description, naming the invalid field and config path.
        message: String,
    },

    /// A domain value failed validation.
    #[error("Validation error: {field} is invalid")]
    Validation {
        /// Name of the field that failed validation.
        field: String,
    },

    /// Vault directory or file access failed.
    #[error("Vault error: {message}")]
    Vault {
        /// Human-readable description of the vault access failure.
        message: String,
    },

    /// Parser encountered unrecoverable structure (rare — normal skip-and-log is non-error).
    #[error("Parse error: {message}")]
    Parse {
        /// Human-readable description of the parse failure.
        message: String,
    },

    /// System clipboard is unavailable (SSH session, headless environment).
    /// This is NON-FATAL — the composition root degrades clipboard capability.
    #[error("Clipboard unavailable: {reason}")]
    ClipboardUnavailable {
        /// Why the clipboard could not be used (e.g. headless, arboard init failure).
        reason: String,
    },

    /// An external service call failed.
    #[error("External service error: {service}")]
    ExternalService {
        /// Name of the external service that failed.
        service: String,
    },

    /// A catch-all error carrying a free-form message.
    #[error("Generic error: {0}")]
    Generic(String),
}

impl RustyCommitListerError {
    /// Build a [`RustyCommitListerError::Config`] from any message.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Build a [`RustyCommitListerError::Validation`] for the named field.
    pub fn validation(field: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
        }
    }

    /// Build a [`RustyCommitListerError::Vault`] from any message.
    pub fn vault(message: impl Into<String>) -> Self {
        Self::Vault {
            message: message.into(),
        }
    }

    /// Build a [`RustyCommitListerError::Parse`] from any message.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }

    /// Build a [`RustyCommitListerError::ClipboardUnavailable`] from any reason.
    pub fn clipboard_unavailable(reason: impl Into<String>) -> Self {
        Self::ClipboardUnavailable {
            reason: reason.into(),
        }
    }

    /// Build a [`RustyCommitListerError::ExternalService`] for the named service.
    pub fn external_service(service: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
        }
    }
}

/// Convenience `Result` alias fixing the error type to [`RustyCommitListerError`].
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
