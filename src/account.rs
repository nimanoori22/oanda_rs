use crate::client::OandaClient;
use crate::errors::{Errors, OandaError};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AccountSummary {
    id: String,
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AccountsResponse {
    accounts: Vec<AccountSummary>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct AccountResponse {
    account: AccountDetail,
    lastTransactionID: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct AccountSummaryResponse {
    account: AccountSummaryDetail,
    lastTransactionID: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct AccountSummaryDetail {
    NAV: String,
    alias: String,
    balance: String,
    commission: String,
    createdByUserID: u32,
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
    openPositionCount: u32,
    openTradeCount: u32,
    pendingOrderCount: u32,
    pl: String,
    positionValue: String,
    resettablePL: String,
    resettablePLTime: String,
    unrealizedPL: String,
    withdrawalLimit: String,
}


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct FinaningDay {
    dayOfWeek: String,
    daysCharged: u32,
}


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct Financing {
    financingDaysOfWeek: Vec<FinaningDay>,
    longRate: String,
    shortRate: String,
}


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Tag {
    name : String,
    #[serde(rename = "type")]
    type_type : String,
}


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct Instrument {
    displayName: String,
    displayPrecision: u32,
    financing: Financing,
    guaranteedStopLossOrderMode: String,
    marginRate: String,
    maximumOrderUnits: String,
    maximumPositionSize: String,
    maximumTrailingStopDistance: String,
    minimumTradeSize: String,
    minimumTrailingStopDistance: String,
    name: String,
    pipLocation: i32,
    tags: Vec<Tag>,
    tradeUnitsPrecision: u32,
    #[serde(rename = "type")]
    instrument_type: String,
}


#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct InstrumentsResponse {
    instruments: Vec<Instrument>,
    lastTransactionID: String,
}

/// Get a list of all Accounts authorized for the provided token.
pub async fn get_accounts(client: &OandaClient) -> Result<AccountsResponse, Errors> {
    let url = "https://api-fxpractice.oanda.com/v3/accounts".to_string();
    let response = client.make_request(&url).await?;
    let accounts: AccountsResponse = serde_json::from_value(response)?;
    Ok(accounts)
}

/// Get the full details for a single Account that a client has access to.
/// Full pending Order, open Trade and open Position representations are provided.
pub async fn get_account(client: &OandaClient) -> Result<AccountResponse, Errors> {
    if let Some(account_id) = client.get_account_id() {
        let url = format!("https://api-fxpractice.oanda.com/v3/accounts/{}", account_id);
        let response = client.make_request(&url).await?;
        let account: AccountResponse = serde_json::from_value(response)?;
        Ok(account)
    } else {
        Err(Errors::OandaError(OandaError::new("Account ID not set")))
    }
}

/// Get a summary for a single Account that a client has access to.
pub async fn get_account_summary(client: &OandaClient) -> Result<AccountSummaryResponse, Errors> {
    if let Some(account_id) = client.get_account_id() {
        let url = format!(
            "https://api-fxpractice.oanda.com/v3/accounts/{}/summary",
            account_id
        );
        let response = client.make_request(&url).await?;
        let account: AccountSummaryResponse = serde_json::from_value(response)?;
        Ok(account)
    } else {
        Err(Errors::OandaError(OandaError::new("Account ID not set")))
    }
}

/// Get a list of tradeable instruments for the given Account.
/// The list of tradeable instruments is dependent on the regulatory division that the Account is located in,
/// thus should be the same for all Accounts owned by a single user.
pub async fn get_account_instruments(client: &OandaClient) -> Result<InstrumentsResponse, Errors> {
    if let Some(account_id) = client.get_account_id() {
        let url = format!(
            "https://api-fxpractice.oanda.com/v3/accounts/{}/instruments",
            account_id
        );
        let response = client.make_request(&url).await?;
        let instruments: InstrumentsResponse = serde_json::from_value(response)?;
        Ok(instruments)
    } else {
        Err(Errors::OandaError(OandaError::new("Account ID not set")))
    }
}

/// Set the client-confguable portions of an Account.
pub async fn patch_configuration(client: &OandaClient, alias: &str, margin_rate: &str) -> Result<(), Errors> {
    if let Some(account_id) = client.get_account_id() {
        let url = format!("https://api-fxpractice.oanda.com/v3/accounts/{}/configuration", account_id);
        let body = json!({
            "alias": alias,
            "marginRate": margin_rate
        });

        let res = client.patch(&url, &body).await?;

        match res.status() {
            StatusCode::OK => Ok(()),
            StatusCode::BAD_REQUEST => Err(Errors::OandaError(OandaError::new("The configuration specification was invalid"))),
            StatusCode::FORBIDDEN => Err(Errors::OandaError(OandaError::new("The configuration operation was forbidden on the Account"))),
            _ => Err(Errors::OandaError(OandaError::new("Unknown error"))),
        }
    } else {
        Err(Errors::OandaError(OandaError::new("Account ID not set")))
    }
}

/// Endpoint used to poll an Account for its current state and changes since a specified TransactionID
/// TODO: test this function with a valid transaction_id and add a struct for the response
pub async fn get_changes(client: &OandaClient, transaction_id: Option<&str>) -> Result<serde_json::Value, Errors> {
    if let Some(account_id) = client.get_account_id() {
        let url = format!("https://api-fxpractice.oanda.com/v3/accounts/{}/changes", account_id);
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
    async fn test_get_accounts() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let client = OandaClient::new(None, &api_key);
        
        match get_accounts(&client).await {
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

    #[tokio::test]
    async fn test_get_account_summary() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID")
            .expect("OANDA_ACCOUNT_ID must be set");
        let client = OandaClient::new(Some(&account_id), &api_key);

        match get_account_summary(&client).await {
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

    #[tokio::test]
    async fn test_get_account_instruments() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY").expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID").expect("OANDA_ACCOUNT_ID must be set");
        let client = OandaClient::new(Some(&account_id), &api_key);

        match get_account_instruments(&client).await {
            Ok(response) => {
                println!("Response: {:?}", response);
                assert!(response.instruments.len() > 0);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                assert!(false);
            }
        }
    }

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