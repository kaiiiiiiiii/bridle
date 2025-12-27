//! Error types for bridle CLI.

#![allow(dead_code)]

use thiserror::Error;

/// Result type alias using bridle's Error.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in bridle.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration file not found.
    #[error("config not found: {0}")]
    ConfigNotFound(String),

    /// Failed to read or write configuration.
    #[error("config error: {0}")]
    Config(String),

    /// IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// TOML parsing error.
    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    /// JSON error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Harness error.
    #[error(transparent)]
    Harness(#[from] get_harness::Error),
}
