use crate::client::OandaClient;
use crate::errors::Errors;
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
    pub async fn get_accounts(&self) -> Result<AccountsResponse, Errors> {
        let url = "/v3/accounts".to_string();

        let response = self.check_response(
            self.make_request(&url).await
        ).await?;

        let accounts: AccountsResponse = serde_json::from_value(response).map_err(Errors::from)?;
        Ok(accounts)
    }
}
// pub async fn get_accounts(client: &OandaClient) -> Result<AccountsResponse, Errors> {
//     let url = "/v3/accounts".to_string();

//     let response = client.check_response(
//         client.make_request(&url).await
//     ).await?;

//     let accounts: AccountsResponse = serde_json::from_value(response).map_err(Errors::from)?;
//     Ok(accounts)
// }




mod tests {

    #[allow(unused_imports)]
    use super::*;


    #[tokio::test]
    async fn test_get_accounts() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let client = OandaClient::new(None, &api_key).unwrap();
        
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