use serde::{Serialize, Deserialize};
use crate::client::OandaClient;
use crate::error::APIError;


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct AccountSummaryResponse {
    pub account: AccountSummaryDetail,
    pub lastTransactionID: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct AccountSummaryDetail {
    pub NAV: String,
    pub alias: String,
    pub balance: String,
    pub commission: String,
    pub createdByUserID: u32,
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
    pub openPositionCount: u32,
    pub openTradeCount: u32,
    pub pendingOrderCount: u32,
    pub pl: String,
    pub positionValue: String,
    pub resettablePL: String,
    pub resettablePLTime: String,
    pub unrealizedPL: String,
    pub withdrawalLimit: String,
}



impl OandaClient {
    /// Get a summary for a single Account that a client has access to.
    pub async fn get_account_summary(&mut self) -> Result<AccountSummaryResponse, APIError> {
        if let Some(account_id) = self.get_account_id().cloned() {
            let url = format!("/v3/accounts/{}/summary", account_id);
            let response = OandaClient::check_response(
                self.get(&url).await
            ).await?;
            let account: AccountSummaryResponse = serde_json::from_value(response).map_err(APIError::from)?;
            Ok(account)
        } else {
            Err(APIError::Other("Account ID Not Set".to_string()))
        }
    }
    
}


mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn test_get_account_summary() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID")
            .expect("OANDA_ACCOUNT_ID must be set");
        let mut client = OandaClient::new(
                    Some(&account_id), 
                    &api_key, 
                    100,
                    100,
                    100,
                    5
                )
                .unwrap();

        match client.get_account_summary().await {
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
