use crate::error::APIError;

use serde_json::Value;

use reqwest::{Client, RequestBuilder, Request};
use tower::buffer::Buffer;
use tower::{limit::{rate::RateLimit, ConcurrencyLimit}, ServiceBuilder, Service};
use tower::limit::concurrency::future::ResponseFuture;

use std::error::Error as StdError;
use std::{future::poll_fn, time::Duration};


pub struct RateLimiter<S, T>
where
    S: Service<T>,
    S::Response: Send + 'static,
    S::Error: Into<APIError> + Into<Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
    S::Future: Send + 'static,
    T: Send + 'static,
{
    service: Buffer<T, ResponseFuture<<S as Service<T>>::Future>>,
}

impl<S, T> RateLimiter<S, T>
where
    S: Service<T> + Send + 'static,
    S::Response: Send + 'static,
    S::Error: Into<APIError> + Into<Box<dyn StdError + Send + Sync + 'static>> + Send + Sync + 'static,
    S::Future: Send + 'static,
    T: Send + 'static,
{
    pub fn new(service: S, rate_limit: usize, buffer_size: usize, concurrency_limit: usize) -> Result<Self, APIError> {
        let rate_limit_u64: u64 = rate_limit
            .try_into()
            .map_err(|_| APIError::Other("Invalid rate limit value".to_string()))?;

        let rate_limited_service: Buffer<T, ResponseFuture<<S as Service<T>>::Future>> = ServiceBuilder::new()
            .buffer(buffer_size)
            .concurrency_limit(concurrency_limit)
            .rate_limit(rate_limit_u64, Duration::from_secs(1))
            .service(service);

        Ok(RateLimiter {
            service: rate_limited_service,
        })
    }


    pub async fn call(&mut self, request: T) -> Result<S::Response, APIError> {
        self.service.call(request).await.map_err(|e| APIError::from(e))
    }
}


/// A client for interacting with the Oanda API.
pub struct OandaClient
{
    client: RateLimiter<Client, Request>,
    account_id: Option<String>,
    api_key: String,
    base_url: String,
}


impl OandaClient
{
    /// Creates a new `OandaClient`.
    ///
    /// # Arguments
    ///
    /// * `account_id` - An optional account ID for the Oanda account.
    /// * `api_key` - The API key for authenticating with the Oanda API.
    /// * `rate_limit` - The rate limit for API requests. This limits the number of requests
    ///   that can be made per second.
    /// * `buffer_size` - The buffer size for the rate limiter. This determines the number of
    ///   requests that can be queued when the service is not ready to process them.
    /// * `concurrency_limit` - The concurrency limit for the rate limiter. This limits the
    ///   number of in-flight requests that can be processed concurrently.
    ///
    /// # Returns
    ///
    /// A result containing the new `OandaClient` or an `APIError`.
    ///
    /// # Examples
    ///
    /// Creating a new `OandaClient` with specific rate limits and concurrency settings:
    ///
    /// ```
    /// let client = OandaClient::new(
    ///     Some("account_id"),
    ///     "api_key",
    ///     5, // rate limit: 5 requests per second
    ///     10, // buffer size: 10 requests
    ///     3 // concurrency limit: 3 in-flight requests
    /// ).unwrap();
    /// ```
    pub fn new(
        account_id: Option<&str>, 
        api_key: &str, 
        rate_limit: usize, 
        buffer_size: usize, 
        concurrency_limit: usize
    ) -> Result<OandaClient, APIError> {

        let client = Client::new();
        let service = RateLimiter::new(
            client, 
            rate_limit, 
            buffer_size, 
            concurrency_limit
        );

        let client = OandaClient {
            client: service?,
            account_id: account_id.map(|s| s.to_string()),
            api_key: api_key.to_string(),
            base_url: "https://api-fxpractice.oanda.com".to_string(),
        };

        Ok(client)
    }


    /// Sets the account ID for the Oanda account.
    ///
    /// # Arguments
    ///
    /// * `account_id` - The account ID to set.
    pub fn set_account_id(&mut self, account_id: &str) {
        self.account_id = Some(account_id.to_string());
    }


    /// Gets the account ID for the Oanda account.
    ///
    /// # Returns
    ///
    /// An optional reference to the account ID.
    pub fn get_account_id(&self) -> Option<&String> {
        self.account_id.as_ref()
    }


    /// Sends a request to the Oanda API.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to send.
    ///
    /// # Returns
    ///
    /// A result containing the response value or an `APIError`.
    async fn send_request(&mut self, request: RequestBuilder) -> Result<Value, APIError> {
        poll_fn(|cx| self.client.service.poll_ready(cx))
            .await
            .map_err(|e| APIError::Other(format!("Service not ready: {}", e)))?;

        let request = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .build()?;

        let response = self.client.call(request).await.map_err(|e| APIError::Other(e.to_string()))?;

        let response = response.json::<Value>().await.map_err(APIError::from)?;

        OandaClient::check_response(Ok(response)).await
    }


    /// Sends a GET request to the Oanda API.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to send the GET request to.
    ///
    /// # Returns
    ///
    /// A result containing the response value or an `APIError`.
    pub async fn get(&mut self, url: &str) -> Result<Value, APIError> {
        let full_url = format!("{}{}", self.base_url, url);
        let request = Client::new()
            .get(&full_url);
        self.send_request(request).await
    }


    /// Sends a PATCH request to the Oanda API.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to send the PATCH request to.
    /// * `body` - The body of the PATCH request.
    ///
    /// # Returns
    ///
    /// A result containing the response value or an `APIError`.
    pub async fn patch(&mut self, url: &str, body: &Value) -> Result<Value, APIError> {
        let full_url = format!("{}{}", self.base_url, url);
        let request = Client::new()
            .patch(&full_url)
            .json(body);
        self.send_request(request).await
    }


    /// Checks the response from the Oanda API for errors.
    ///
    /// # Arguments
    ///
    /// * `response` - The response to check.
    ///
    /// # Returns
    ///
    /// A result containing the response value or an `APIError`.
    pub async fn check_response(response: Result<Value, APIError>) -> Result<Value, APIError> {
        match response {
            Ok(value) => {
                if let Some(error_message) = value.get("errorMessage").and_then(|v| v.as_str()) {
                    Err(APIError::Other(error_message.to_string()))
                } else {
                    Ok(value)
                }
            },
            Err(err) => Err(err),
        }
    }
}


mod tests {

    #[test]
    fn print_api_key() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        println!("API Key: {}", api_key);
    }


    #[tokio::test]
    async fn test_clone_client() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID")
            .expect("OANDA_ACCOUNT_ID must be set");
        let client = super::OandaClient::new(Some(&account_id), &api_key, 100, 100, 100).unwrap();
        let client_id = client.get_account_id().unwrap();
        let client_clone_id = client.get_account_id().unwrap();
        assert_eq!(client_id, client_clone_id);
    }
}