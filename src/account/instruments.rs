use crate::client::OandaClient;
use crate::errors::{Errors, OandaError};

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct FinaningDay {
    pub dayOfWeek: String,
    pub daysCharged: u32,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct Financing {
    pub financingDaysOfWeek: Vec<FinaningDay>,
    pub longRate: String,
    pub shortRate: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Tag {
    pub name : String,
    #[serde(rename = "type")]
    pub type_type : String,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct Instrument {
    pub displayName: String,
    pub displayPrecision: u32,
    pub financing: Financing,
    pub guaranteedStopLossOrderMode: String,
    pub marginRate: String,
    pub maximumOrderUnits: String,
    pub maximumPositionSize: String,
    pub maximumTrailingStopDistance: String,
    pub minimumTradeSize: String,
    pub minimumTrailingStopDistance: String,
    pub name: String,
    pub pipLocation: i32,
    pub tags: Vec<Tag>,
    pub tradeUnitsPrecision: u32,
    #[serde(rename = "type")]
    pub instrument_type: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct InstrumentsResponse {
    pub instruments: Vec<Instrument>,
    pub lastTransactionID: String,
}

/// Get a list of tradeable instruments for the given Account.
/// The list of tradeable instruments is dependent on the regulatory division that the Account is located in,
/// thus should be the same for all Accounts owned by a single user.
impl OandaClient {
    pub async fn get_account_instruments(&self) -> Result<InstrumentsResponse, Errors> {
        if let Some(account_id) = self.get_account_id() {
            let url = format!("/v3/accounts/{}/instruments", account_id);
            let response = self.check_response(
                self.make_request(&url).await
            ).await?;

            let instruments: InstrumentsResponse = serde_json::from_value(response)?;
            Ok(instruments)
        } else {
            Err(Errors::OandaError(OandaError::new("Account ID not set")))
        }
    }
}
// pub async fn get_account_instruments(client: &OandaClient) -> Result<InstrumentsResponse, Errors> {
//     if let Some(account_id) = client.get_account_id() {
//         let url = format!(
//             "/v3/accounts/{}/instruments",
//             account_id
//         );
//         let response = client.check_response(
//             client.make_request(&url).await
//         ).await?;

//         let instruments: InstrumentsResponse = serde_json::from_value(response)?;
//         Ok(instruments)
//     } else {
//         Err(Errors::OandaError(OandaError::new("Account ID not set")))
//     }
// }


mod tests {

    #[allow(unused_imports)]
    use super::*;


    #[tokio::test]
    async fn test_get_account_instruments() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY").expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID").expect("OANDA_ACCOUNT_ID must be set");
        let client = OandaClient::new(Some(&account_id), &api_key).unwrap();

        match client.get_account_instruments().await {
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
    async fn test_serialize_instruments_response() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY").expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID").expect("OANDA_ACCOUNT_ID must be set");
        let client = OandaClient::new(Some(&account_id), &api_key).unwrap();

        match client.get_account_instruments().await {
            Ok(response) => {
                let serialized = serde_json::to_string(&response).unwrap();
                println!("Serialized: {}", serialized);
                assert!(true);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                assert!(false);
            }
        }
    }
}