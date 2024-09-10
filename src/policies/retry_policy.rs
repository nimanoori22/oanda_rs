use futures_util::future::{Ready, Either};
use tokio::time::{sleep, Duration};
use tower::retry::Policy;
use std::error::Error as StdError;
use crate::utils::try_clone::TryClone;
use crate::error::APIError;


// Define the retry policy
pub struct RetryPolicy<T> {
    pub attempts: usize,
    pub _marker: std::marker::PhantomData<T>,
}

impl<T> Clone for RetryPolicy<T> {
    fn clone(&self) -> Self {
        RetryPolicy {
            attempts: self.attempts,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, Res, E> Policy<T, Res, E> for RetryPolicy<T>
where
    T: TryClone,
    E: Into<APIError> + Into<Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
{
    type Future = Either<Ready<()>, tokio::time::Sleep>;

    fn retry(&mut self, _req: &mut T, result: &mut Result<Res, E>) -> Option<Self::Future> {

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

    fn clone_request(&mut self, req: &T) -> Option<T> {
        req.try_clone().ok() // Use try_clone for retries
    }
}