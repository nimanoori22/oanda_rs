use std::error::Error as StdError;
use thiserror::Error as ErrorMacro;

#[derive(Debug, ErrorMacro)]
pub enum APIError {
    #[error("HTTP error: {0}")]
    HTTP(#[from] rustopus::reqwest::Error),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Custom error: {0}")]
    Other(String),
    #[error("Clone error: {0}")]
    Clone(String),
}

impl From<Box<dyn StdError + Send + Sync>> for APIError {
    fn from(error: Box<dyn StdError + Send + Sync>) -> Self {
        APIError::Other(error.to_string())
    }
}

impl From<rustopus::Error<rustopus::HttpStatusError>> for APIError {
    fn from(error: rustopus::Error<rustopus::HttpStatusError>) -> Self {
        match error {
            rustopus::Error::Policy(error) => http_error(error.status.as_u16(), &error.body),
            rustopus::Error::Exhausted { last, .. } => match *last {
                Ok(outcome) => http_error(outcome.status.as_u16(), &outcome.body),
                Err(error) => Self::Other(error.to_string()),
            },
            error => Self::Other(error.to_string()),
        }
    }
}

fn http_error(status: u16, body: &[u8]) -> APIError {
    let message = serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|value| value.get("errorMessage")?.as_str().map(str::to_owned))
        .unwrap_or_else(|| String::from_utf8_lossy(&body[..body.len().min(200)]).into_owned());
    APIError::Other(format!("HTTP {status}: {message}"))
}
