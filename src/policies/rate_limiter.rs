use tokio::time::Duration;
use tower::{Service, ServiceBuilder};
use tower::util::BoxCloneService;

use std::error::Error as StdError;

use crate::utils::clonable_request::ClonableRequest;
use crate::error::APIError;
use crate::policies::retry_policy::RetryPolicy;


#[derive(Clone, Debug)]
pub struct RateLimiter<S>
where
    S: Service<ClonableRequest> + Clone + Send + 'static,
    S::Response: Send + 'static,
    S::Error: Into<APIError> + Into<Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    pub service: BoxCloneService<ClonableRequest, <S as Service<ClonableRequest>>::Response, Box<dyn StdError + Send + Sync>>,
}

impl<S> RateLimiter<S>
where
    S: Service<ClonableRequest> + Clone + Send + 'static,
    S::Response: Send + 'static,
    S::Error: Into<APIError> + Into<Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
    S::Future: Send + 'static,
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

        let retry_policy = RetryPolicy { attempts: retry_attempts };

        let rate_limited_service: BoxCloneService<ClonableRequest, <S as Service<ClonableRequest>>::Response, Box<dyn StdError + Send + Sync>> = ServiceBuilder::new()
            .boxed_clone()
            .buffer(buffer_size)
            .concurrency_limit(concurrency_limit)
            .rate_limit(rate_limit_u64, Duration::from_secs(1))
            .retry(retry_policy)
            .service(service); // Apply the retry policy and box the service

        Ok(RateLimiter {
            service: rate_limited_service,
        })
    }

    pub async fn call(&mut self, request: ClonableRequest) -> Result<S::Response, APIError> {
        self
        .service
        .call(request)
        .await.map_err(|e| APIError::from(e))
    }
}