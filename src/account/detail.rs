use serde::{Serialize, Deserialize};
use crate::client::OandaClient;
use crate::errors::{Errors, OandaError};


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct AccountDetail {
    pub NAV: String,
    pub alias: String,
    pub balance: String,
    pub commission: String,
    pub createdByUserID: u64,
    pub createdTime: String,
    pub currency: String,
    pub dividendAdjustment: String,
    pub financing: String,
    pub guaranteedExecutionFees: String,
    pub guaranteedStopLossOrderMode: String,
    pub hedgingEnabled: bool,
    pub id: String,
    pub lastTransactionID: String,
    pub marginAvailable: String,
    pub marginCallMarginUsed: String,
    pub marginCallPercent: String,
    pub marginCloseoutMarginUsed: String,
    pub marginCloseoutNAV: String,
    pub marginCloseoutPercent: String,
    pub marginCloseoutPositionValue: String,
    pub marginCloseoutUnrealizedPL: String,
    pub marginRate: String,
    pub marginUsed: String,
    pub openPositionCount: u64,
    pub openTradeCount: u64,
    pub orders: Vec<String>,
    pub pendingOrderCount: u64,
    pub pl: String,
    pub positionValue: String,
    pub positions: Vec<String>,
    pub resettablePL: String,
    pub resettablePLTime: String,
    pub trades: Vec<String>,
    pub unrealizedPL: String,
    pub withdrawalLimit: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct AccountResponse {
    pub account: AccountDetail,
    pub lastTransactionID: String,
}


impl OandaClient {
    /// Get the full details for a single Account that a client has access to.
    /// Full pending Order, open Trade and open Position representations are provided.
    pub async fn get_account(&self) -> Result<AccountResponse, Errors> {
        if let Some(account_id) = self.get_account_id() {
            let url = format!("/v3/accounts/{}", account_id);
            let response = self.check_response(
                self.make_request(&url).await
            ).await?;
            let account: AccountResponse = serde_json::from_value(response).map_err(Errors::from)?;
            Ok(account)
        } else {
            Err(Errors::OandaError(OandaError::new("Account ID not set")))
        }
    }
}
// pub async fn get_account(client: &OandaClient) -> Result<AccountResponse, Errors> {
//     if let Some(account_id) = client.get_account_id() {
//         let url = format!("/v3/accounts/{}", account_id);
//         // let response = client.make_request(&url).await?;
//         // let account: AccountResponse = serde_json::from_value(response)?;
//         // Ok(account)
//         let response = client.check_response(
//             client.make_request(&url).await
//         ).await?;
//         let account: AccountResponse = serde_json::from_value(response).map_err(Errors::from)?;
//         Ok(account)
//     } else {
//         Err(Errors::OandaError(OandaError::new("Account ID not set")))
//     }
// }


mod tests {
    #[allow(unused_imports)]
    use super::*;
    
    #[tokio::test]
    async fn test_get_account() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID")
            .expect("OANDA_ACCOUNT_ID must be set");
        let client = OandaClient::new(Some(&account_id), &api_key).unwrap();
        
        match client.get_account().await {
            Ok(response) => {
                println!("Response: {:?}", response);
                assert!(response.account.id == account_id);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                assert!(false);
            }
        }
    }
}