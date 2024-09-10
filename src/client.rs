// External crates
use futures::future::poll_fn;
use reqwest::{Client, Request, RequestBuilder};
use serde_json::Value;
use tower::Service;

// Local modules
use crate::error::APIError;
use crate::policies::rate_limiter::RateLimiter;


pub struct OandaClient {
    client: RateLimiter<Client, Request>,
    account_id: Option<String>,
    api_key: String,
    base_url: String,
}

impl OandaClient {

    pub fn new(
        account_id: Option<&str>, 
        api_key: &str, 
        buffer_size: usize, 
        concurrency_limit: usize,
        rate_limit: usize, 
        retry_attempts: usize
    ) -> Result<OandaClient, APIError> {

        let client = Client::new();
        let service = RateLimiter::new(
            client, 
            rate_limit, 
            buffer_size, 
            concurrency_limit, 
            retry_attempts
        );

        let client = OandaClient {
            client: service?,
            account_id: account_id.map(|s| s.to_string()),
            api_key: api_key.to_string(),
            base_url: "https://api-fxpractice.oanda.com".to_string(),
        };

        Ok(client)
    }

    pub fn set_account_id(&mut self, account_id: &str) {
        self.account_id = Some(account_id.to_string());
    }

    pub fn get_account_id(&self) -> Option<&String> {
        self.account_id.as_ref()
    }

    async fn send_request(&mut self, request: RequestBuilder) -> Result<Value, APIError> {

        poll_fn(|cx| self.client.service.poll_ready(cx))
            .await
            .map_err(|e| APIError::Other(format!("Service not ready: {}", e)))?;

        let request = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .build()?;

        let response = self
            .client
            .call(request)
            .await
            .map_err(|e| APIError::Other(e.to_string()))?;

        let response = response
            .json::<Value>()
            .await
            .map_err(APIError::from)?;

        OandaClient::check_response(Ok(response)).await
    }

    pub async fn get(&mut self, url: &str) -> Result<Value, APIError> {
        let full_url = format!("{}{}", self.base_url, url);
        let request = Client::new().get(&full_url);
        self.send_request(request).await
    }

    pub async fn patch(&mut self, url: &str, body: &Value) -> Result<Value, APIError> {
        let full_url = format!("{}{}", self.base_url, url);
        let request = Client::new().patch(&full_url).json(body);
        self.send_request(request).await
    }

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
        let api_key = std::env::var("ANDA_API_KEY")
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
        let client = super::OandaClient::new(Some(&account_id), &api_key, 5, 10, 3, 5).unwrap();
        let client_id = client.get_account_id().unwrap();
        let client_clone_id = client.get_account_id().unwrap();
        assert_eq!(client_id, client_clone_id);
    }
}