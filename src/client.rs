use crate::error::APIError;

use serde_json::Value;

use reqwest::{Client, RequestBuilder, Request};
use tower::buffer::Buffer;
use tower::{limit::rate::RateLimit, ServiceBuilder, Service};
use tokio::sync::{Mutex, MutexGuard};

use std::{future::poll_fn, time::Duration, sync::Arc};



#[derive(Clone, Debug)]
pub struct OandaClient
{
    client: Arc<Mutex<Buffer<RateLimit<Client>, Request>>>,
    account_id: Option<String>,
    api_key: String,
    base_url: String,
}


impl OandaClient
{
    pub fn new(account_id: Option<&str>, api_key: &str, buffer: u64, rate_limit: u64) -> Result<OandaClient, APIError> {
        let client = Client::new();
        let service:Buffer<RateLimit<Client>, Request>  = ServiceBuilder::new()
            .buffer(buffer.try_into().unwrap())
            .rate_limit(rate_limit, Duration::from_secs(1))
            .service(client);

        Ok(OandaClient {
            client: Arc::new(Mutex::new(service)),
            account_id: account_id.map(|s| s.to_string()),
            api_key: api_key.to_string(),
            base_url: "https://api-fxpractice.oanda.com".to_string(),
        })
    }


    pub fn set_account_id(&mut self, account_id: &str) {
        self.account_id = Some(account_id.to_string());
    }

    pub fn get_account_id(&self) -> Option<&String> {
        self.account_id.as_ref()
    }


    async fn send_request(&self, request: RequestBuilder) -> Result<Value, APIError> {
        let mut client: MutexGuard<Buffer<RateLimit<Client>, Request>> = self.client.lock().await;

        poll_fn(|cx| client.poll_ready(cx))
            .await
            .map_err(|e| APIError::Other(format!("Service not ready: {}", e)))?;

        let request = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .build()?;

        let response = client
            .call(request)
            .await
            .map_err(|e| APIError::Other(e.to_string()))?;

        let response = response.json::<Value>().await.map_err(APIError::from)?;

        self.check_response(Ok(response)).await
    }


    pub async fn get(&self, url: &str) -> Result<Value, APIError> {
        let full_url = format!("{}{}", self.base_url, url);
        let request = Client::new()
            .get(&full_url);
        self.send_request(request).await
    }


    pub async fn patch(&self, url: &str, body: &Value) -> Result<Value, APIError> {
        let full_url = format!("{}{}", self.base_url, url);
        let request = Client::new()
            .patch(&full_url)
            .json(body);
        self.send_request(request).await
    }


    pub async fn check_response(&self, response: Result<Value, APIError>) -> Result<Value, APIError> {
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
        let client = super::OandaClient::new(Some(&account_id), &api_key, 100, 100).unwrap();
        let client_clone = client.clone();
        let client_id = client.get_account_id().unwrap();
        let client_clone_id = client_clone.get_account_id().unwrap();
        assert_eq!(client_id, client_clone_id);
    }
}