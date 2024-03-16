use reqwest::Client;
use crate::errors::{OandaError, Errors};
use serde_json::Value;

pub struct OandaClient {
    client: Client,
    account_id: Option<String>,
    api_key: String,
    base_url: String,
}


impl OandaClient {
    pub fn new(account_id: Option<&str>, api_key: &str) -> OandaClient {
        OandaClient {
            client: Client::new(),
            account_id: account_id.map(|s| s.to_string()),
            api_key: api_key.to_string(),
            base_url: "https://api-fxpractice.oanda.com".to_string(),
        }
    }

    pub fn set_account_id(&mut self, account_id: &str) {
        self.account_id = Some(account_id.to_string());
    }

    pub fn get_account_id(&self) -> Option<&String> {
        self.account_id.as_ref()
    }

    pub async fn make_request(&self, url: &str) -> Result<serde_json::Value, Errors> {
        let response = self.client.get(&format!("{}{}", self.base_url, url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn patch(&self, url: &str, body: &serde_json::Value) -> Result<reqwest::Response, Errors> {
        let response = self.client.patch(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(body)
            .send()
            .await?;
        Ok(response)
    }

    pub async fn check_response(&self, response: Result<Value, Errors>) -> Result<Value, Errors> {
        match response {
            Ok(value) => {
                if value.get("errorMessage").is_some() {
                    Err(Errors::OandaError(OandaError::new(value["errorMessage"].as_str().unwrap())))
                } else {
                    Ok(value)
                }
            },
            Err(Errors::ReqwestError(_)) => Err(Errors::OandaError(OandaError::new("Request failed"))),
            Err(Errors::OandaError(err)) => Err(Errors::OandaError(err)),
            Err(Errors::SerdeError(_)) => Err(Errors::OandaError(OandaError::new("Serialization failed"))),
            Err(Errors::CustomError(_)) => Err(Errors::OandaError(OandaError::new("Custom error"))),
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
}