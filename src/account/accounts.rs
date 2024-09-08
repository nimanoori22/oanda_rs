use crate::client::OandaClient;
use crate::error::APIError;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AccountSummary {
    pub id: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AccountsResponse {
    pub accounts: Vec<AccountSummary>,
}


/// Get a list of all Accounts authorized for the provided token.
impl OandaClient {
    pub async fn get_accounts(&mut self) -> Result<AccountsResponse, APIError> {
        let url = "/v3/accounts".to_string();

        let response = OandaClient::check_response(
            self.get(&url).await
        ).await?;

        let accounts: AccountsResponse = serde_json::from_value(response).map_err(APIError::from)?;
        Ok(accounts)
    }
}


mod tests {

    #[allow(unused_imports)]
    use super::*;


    #[tokio::test]
    async fn test_get_accounts() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let mut client = OandaClient::new(None, &api_key, 100, 100, 100, 5).unwrap();
        
        match client.get_accounts().await {
            Ok(response) => {
                println!("Response: {:?}", response);
                assert!(response.accounts.len() > 0);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                assert!(false);
            }
        }
    }
}