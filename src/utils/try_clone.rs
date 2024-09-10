use reqwest::Request;
use crate::error::APIError;


pub trait TryClone {
    fn try_clone(&self) -> Result<Self, APIError>
    where
        Self: Sized;
}


impl TryClone for Request {
    fn try_clone(&self) -> Result<Self, APIError> {
        self.try_clone().ok_or_else(|| {
            let error_message = format!("Failed to clone request: {:?}", self);
            APIError::Clone(error_message)
        })
    }
}