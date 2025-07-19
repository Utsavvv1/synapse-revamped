use std::io;
use rusqlite;
use serde_json;
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SynapseError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Time error: {0}")]
    Time(#[from] SystemTimeError),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Platform error: {0}")]
    Platform(String),
    #[error("Other error: {0}")]
    Other(String),
} 
