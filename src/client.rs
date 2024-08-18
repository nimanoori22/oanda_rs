use crate::error::APIError;

use serde_json::Value;

use reqwest::{Client, RequestBuilder};
use tower::{limit::rate::RateLimit, ServiceBuilder};
use tower::Service;

use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::sync::MutexGuard;


#[derive(Clone, Debug)]
pub struct OandaClient
{
    client: Arc<Mutex<RateLimit<Client>>>,
    account_id: Option<String>,
    api_key: String,
    base_url: String,
}


impl OandaClient
{
    pub fn new(account_id: Option<&str>, api_key: &str) -> Result<OandaClient, APIError> {
        let client = Client::new();
        let service: RateLimit<Client> = ServiceBuilder::new()
            .rate_limit(100, Duration::from_secs(1))
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
        let mut client: MutexGuard<RateLimit<Client>> = self.client.lock().unwrap();
        let request = request
            .header("Authorization", format!("Bearer {}", self.api_key))
            .build()?;
        let response = client
            .call(request)
            .await?
            .json()
            .await?;
        self.check_response(Ok(response)).await
    }


    pub async fn make_request(&self, url: &str) -> Result<Value, APIError> {
        let full_url = format!("{}{}", self.base_url, url);
        let request = self.client
            .lock()
            .unwrap()
            .get_mut()
            .get(&full_url);
        self.send_request(request).await
    }


    pub async fn patch(&self, url: &str, body: &Value) -> Result<Value, APIError> {
        let full_url = format!("{}{}", self.base_url, url);
        let request = self.client
            .lock()
            .unwrap()
            .get_mut()
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
        let client = super::OandaClient::new(Some(&account_id), &api_key).unwrap();
        let client_clone = client.clone();
        let client_id = client.get_account_id().unwrap();
        let client_clone_id = client_clone.get_account_id().unwrap();
        assert_eq!(client_id, client_clone_id);
    }
}