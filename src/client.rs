use reqwest::Client;
use crate::errors::OandaError;


pub struct OandaClient {
    client: Client,
    account_id: Option<String>,
    api_key: String,
}


impl OandaClient {
    pub fn new(account_id: Option<&str>, api_key: &str) -> OandaClient {
        OandaClient {
            client: Client::new(),
            account_id: account_id.map(|s| s.to_string()),
            api_key: api_key.to_string(),
        }
    }

    pub fn set_account_id(&mut self, account_id: &str) {
        self.account_id = Some(account_id.to_string());
    }

    pub fn get_account_id(&self) -> Option<&String> {
        self.account_id.as_ref()
    }

    pub async fn make_request(&self, url: &str) -> Result<serde_json::Value, OandaError> {
        let response = self.client.get(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn patch(&self, url: &str, body: &serde_json::Value) -> Result<reqwest::Response, OandaError> {
        let response = self.client.patch(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(body)
            .send()
            .await?;
        Ok(response)
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