//! Error module: defines the central error type for the application using thiserror.

use std::io;
use rusqlite;
use serde_json;
use std::time::SystemTimeError;
use thiserror::Error;
use reqwest;

#[derive(Error, Debug)]
pub enum SupabaseError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Timeout occurred")]
    Timeout,
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Other error: {0}")]
    Other(String),
}

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
    #[error("Supabase error: {0}")]
    Supabase(#[from] SupabaseError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use rusqlite;
    use serde_json;
    use std::time::SystemTimeError;
    use reqwest;

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

    #[tokio::test]
    async fn test_supabase_http_error_variant() {
        // Generate a reqwest::Error by making an invalid request
        let err = reqwest::get("http://nonexistent.invalid").await.err().unwrap();
        let err = SupabaseError::Http(err);
        assert!(format!("{}", err).contains("HTTP error"));
    }

    #[test]
    fn test_supabase_serde_error_variant() {
        let err = SupabaseError::Serde(serde_json::from_str::<u32>("not a number").unwrap_err());
        assert!(format!("{}", err).contains("Serialization error"));
    }

    #[test]
    fn test_supabase_timeout_error_variant() {
        let err = SupabaseError::Timeout;
        assert!(format!("{}", err).contains("Timeout occurred"));
    }

    #[test]
    fn test_supabase_config_error_variant() {
        let err = SupabaseError::Config("bad config".to_string());
        assert!(format!("{}", err).contains("Configuration error"));
    }

    #[test]
    fn test_supabase_api_error_variant() {
        let err = SupabaseError::Api("api error".to_string());
        assert!(format!("{}", err).contains("API error"));
    }

    #[test]
    fn test_supabase_other_error_variant() {
        let err = SupabaseError::Other("other error".to_string());
        assert!(format!("{}", err).contains("Other error"));
    }
} 
