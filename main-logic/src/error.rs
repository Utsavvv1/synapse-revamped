//! Error module: defines the central error type for the application using thiserror.

use std::io;
use rusqlite;
use serde_json;
use std::time::SystemTimeError;
use thiserror::Error;

/// The main error type for the application, covering IO, DB, serialization, time, config, platform, and other errors.
#[derive(Error, Debug)]
pub enum SynapseError {
    /// IO error (file, network, etc.)
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// SQLite database error
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    /// System time error
    #[error("Time error: {0}")]
    Time(#[from] SystemTimeError),
    /// Configuration error
    #[error("Config error: {0}")]
    Config(String),
    /// Platform-specific error
    #[error("Platform error: {0}")]
    Platform(String),
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use rusqlite;
    use serde_json;
    use std::time::SystemTimeError;

    #[test]
    fn test_io_error_variant() {
        let err = SynapseError::from(io::Error::new(io::ErrorKind::Other, "io error"));
        assert!(matches!(err, SynapseError::Io(_)));
        assert!(format!("{}", err).contains("IO error"));
    }

    #[test]
    fn test_db_error_variant() {
        let err = SynapseError::from(rusqlite::Error::InvalidQuery);
        assert!(matches!(err, SynapseError::Db(_)));
        assert!(format!("{}", err).contains("Database error"));
    }

    #[test]
    fn test_serde_error_variant() {
        let err = SynapseError::from(serde_json::from_str::<u32>("not a number").unwrap_err());
        assert!(matches!(err, SynapseError::Serde(_)));
        assert!(format!("{}", err).contains("Serialization error"));
    }

    #[test]
    fn test_time_error_variant() {
        let before_epoch = std::time::UNIX_EPOCH - std::time::Duration::from_secs(1);
        let err = before_epoch.duration_since(std::time::UNIX_EPOCH).unwrap_err();
        let err = SynapseError::from(err);
        assert!(matches!(err, SynapseError::Time(_)));
        assert!(format!("{}", err).contains("Time error"));
    }

    #[test]
    fn test_config_error_variant() {
        let err = SynapseError::Config("bad config".to_string());
        assert!(matches!(err, SynapseError::Config(_)));
        assert!(format!("{}", err).contains("Config error"));
    }

    #[test]
    fn test_platform_error_variant() {
        let err = SynapseError::Platform("bad platform".to_string());
        assert!(matches!(err, SynapseError::Platform(_)));
        assert!(format!("{}", err).contains("Platform error"));
    }

    #[test]
    fn test_other_error_variant() {
        let err = SynapseError::Other("other error".to_string());
        assert!(matches!(err, SynapseError::Other(_)));
        assert!(format!("{}", err).contains("Other error"));
    }
} 
