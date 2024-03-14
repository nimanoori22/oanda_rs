use crate::client::OandaClient;
use crate::errors::{Errors, OandaError};


/// Endpoint used to poll an Account for its current state and changes since a specified TransactionID
/// TODO: test this function with a valid transaction_id and add a struct for the response
pub async fn get_changes(client: &OandaClient, transaction_id: Option<&str>) -> Result<serde_json::Value, Errors> {
    if let Some(account_id) = client.get_account_id() {
        let url = format!("/v3/accounts/{}/changes", account_id);
        let url = if let Some(transaction_id) = transaction_id {
            format!("{}?sinceTransactionID={}", url, transaction_id)
        } else {
            url
        };
        let response = client.make_request(&url).await?;
        Ok(response)
    } else {
        Err(Errors::OandaError(OandaError::new("Account ID not set")))
    }
}


mod tests {

    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn test_get_changes() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY").expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID").expect("OANDA_ACCOUNT_ID must be set");
        let client = OandaClient::new(Some(&account_id), &api_key);

        match get_changes(&client, None).await {
            Ok(response) => {
                println!("Response: {:?}", response);
                assert!(response.is_object());
            }
            Err(e) => {
                println!("Error: {:?}", e);
                assert!(false);
            }
        }
    }
}