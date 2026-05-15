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
