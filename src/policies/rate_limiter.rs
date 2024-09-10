use tokio::time::Duration;
use tower::retry::Retry;
use tower::limit::{ConcurrencyLimit, RateLimit};
use tower::{Service, ServiceBuilder};
use tower::buffer::Buffer;

use std::error::Error as StdError;
use std::marker::PhantomData;

use crate::utils::try_clone::TryClone;
use crate::error::APIError;
use crate::policies::retry_policy::RetryPolicy;



pub struct RateLimiter<S, T>
where
    S: Service<T> + Clone + Send + 'static,
    S::Response: Send + 'static,
    S::Error: Into<APIError> + Into<Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
    S::Future: Send + 'static,
    T: TryClone + Send + 'static,
{
    pub service: Buffer<T, <ConcurrencyLimit<RateLimit<Retry<RetryPolicy<T>, S>>> as Service<T>>::Future>,
}

impl<S, T> RateLimiter<S, T>
where
    S: Service<T> + Clone + Send + 'static,
    S::Response: Send + 'static,
    S::Error: Into<APIError> + Into<Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
    S::Future: Send + 'static,
    T: TryClone + Send + 'static,
{
    
    pub fn new(
        service: S, 
        rate_limit: usize, 
        buffer_size: usize, 
        concurrency_limit: usize, 
        retry_attempts: usize
    ) -> Result<Self, APIError> {

        let rate_limit_u64: u64 = rate_limit
            .try_into()
            .map_err(|_| APIError::Other("Invalid rate limit value".to_string()))?;

        let retry_policy = RetryPolicy { attempts: retry_attempts, _marker: PhantomData };

        let rate_limited_service = ServiceBuilder::new()
            .buffer(buffer_size)
            .concurrency_limit(concurrency_limit)
            .rate_limit(rate_limit_u64, Duration::from_secs(1))
            .service(Retry::new(retry_policy, service)); // Apply the retry policy

        Ok(RateLimiter {
            service: rate_limited_service,
        })
    }

    pub async fn call(&mut self, request: T) -> Result<S::Response, APIError> {
        self
        .service
        .call(request)
        .await.map_err(|e| APIError::from(e))
    }
}