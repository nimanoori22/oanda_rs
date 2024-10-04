use futures_util::future::{Ready, Either};
use tokio::time::{sleep, Duration};
use tower::retry::Policy;
use std::error::Error as StdError;
use crate::error::APIError;
use crate::utils::clonable_request::ClonableRequest;


pub struct RetryPolicy {
    pub attempts: usize,
}

impl Clone for RetryPolicy {
    fn clone(&self) -> Self {
        RetryPolicy {
            attempts: self.attempts,
        }
    }
}

impl<Res, E> Policy<ClonableRequest, Res, E> for RetryPolicy
where
    E: Into<APIError> + Into<Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
{
    type Future = Either<Ready<()>, tokio::time::Sleep>;

    fn retry(&mut self, _req: &mut ClonableRequest, result: &mut Result<Res, E>) -> Option<Self::Future> {
        match result {
            Ok(_) => None, // Don't retry on success
            Err(_) => {
                if self.attempts > 0 {
                    self.attempts -= 1;
                    let backoff = Duration::from_secs(2u64.pow((self.attempts.min(3)) as u32));
                    Some(Either::Right(sleep(backoff)))
                } else {
                    None // No attempts left, don't retry
                }
            }
        }
    }

    fn clone_request(&mut self, req: &ClonableRequest) -> Option<ClonableRequest> {
        Some(req.clone()) // Use clone for retries
    }
}