use serde::{Serialize, Deserialize};
use crate::client::OandaClient;
use crate::errors::{Errors, OandaError};


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct AccountDetail {
    NAV: String,
    alias: String,
    balance: String,
    commission: String,
    createdByUserID: u64,
    createdTime: String,
    currency: String,
    dividendAdjustment: String,
    financing: String,
    guaranteedExecutionFees: String,
    guaranteedStopLossOrderMode: String,
    hedgingEnabled: bool,
    id: String,
    lastTransactionID: String,
    marginAvailable: String,
    marginCallMarginUsed: String,
    marginCallPercent: String,
    marginCloseoutMarginUsed: String,
    marginCloseoutNAV: String,
    marginCloseoutPercent: String,
    marginCloseoutPositionValue: String,
    marginCloseoutUnrealizedPL: String,
    marginRate: String,
    marginUsed: String,
    openPositionCount: u64,
    openTradeCount: u64,
    orders: Vec<String>,
    pendingOrderCount: u64,
    pl: String,
    positionValue: String,
    positions: Vec<String>,
    resettablePL: String,
    resettablePLTime: String,
    trades: Vec<String>,
    unrealizedPL: String,
    withdrawalLimit: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct AccountResponse {
    account: AccountDetail,
    lastTransactionID: String,
}


/// Get the full details for a single Account that a client has access to.
/// Full pending Order, open Trade and open Position representations are provided.
pub async fn get_account(client: &OandaClient) -> Result<AccountResponse, Errors> {
    if let Some(account_id) = client.get_account_id() {
        let url = format!("/v3/accounts/{}", account_id);
        let response = client.make_request(&url).await?;
        let account: AccountResponse = serde_json::from_value(response)?;
        Ok(account)
    } else {
        Err(Errors::OandaError(OandaError::new("Account ID not set")))
    }
}


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
        let client = OandaClient::new(Some(&account_id), &api_key);
        
        match get_account(&client).await {
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