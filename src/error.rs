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