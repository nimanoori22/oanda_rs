use std::error::Error as StdError;

use thiserror::Error as ErrorMacro;


#[derive(Debug, ErrorMacro)]
pub enum APIError {
    #[error("HTTP error: {0}")]
    HTTP(#[from] reqwest::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Custom error: {0}")]
    Other(String),
}


impl From<Box<dyn StdError + Send + Sync>> for APIError {
    fn from(error: Box<dyn StdError + Send + Sync>) -> Self {
        APIError::Other(error.to_string())
    }
}