use std::{error::Error, fmt};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OandaError {
    #[serde(rename = "errorMessage")]
    error_message: String,
}

impl OandaError {
    pub fn new(message: &str) -> OandaError {
        OandaError {
            error_message: message.to_string(),
        }
    }
}

impl fmt::Display for OandaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OandaError: {}", self.error_message)
    }
}


impl Error for OandaError {}


#[derive(Debug)]
pub enum Errors {
    ReqwestError(reqwest::Error),
    OandaError(OandaError),
    SerdeError(serde_json::Error),
    CustomError(String),
}

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Errors::ReqwestError(e) => write!(f, "Reqwest error: {}", e),
            Errors::OandaError(e) => write!(f, "Oanda error: {}", e),
            Errors::SerdeError(e) => write!(f, "Serde error: {}", e),
            Errors::CustomError(e) => write!(f, "Custom error: {}", e),
        }
    }
}

impl From<OandaError> for Errors {
    fn from(error: OandaError) -> Self {
        Errors::OandaError(error)
    }
}

impl From<serde_json::Error> for Errors {
    fn from(error: serde_json::Error) -> Self {
        Errors::SerdeError(error)
    }
}

impl From<reqwest::Error> for Errors {
    fn from(error: reqwest::Error) -> Self {
        Errors::ReqwestError(error)
    }
}

impl Error for Errors {}