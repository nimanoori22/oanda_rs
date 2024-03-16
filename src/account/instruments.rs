use crate::client::OandaClient;
use crate::errors::{Errors, OandaError};

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct FinaningDay {
    dayOfWeek: String,
    daysCharged: u32,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct Financing {
    financingDaysOfWeek: Vec<FinaningDay>,
    longRate: String,
    shortRate: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Tag {
    name : String,
    #[serde(rename = "type")]
    type_type : String,
}


#[derive(Debug, Serialize, Deserialize)]
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


#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct InstrumentsResponse {
    instruments: Vec<Instrument>,
    lastTransactionID: String,
}

/// Get a list of tradeable instruments for the given Account.
/// The list of tradeable instruments is dependent on the regulatory division that the Account is located in,
/// thus should be the same for all Accounts owned by a single user.
pub async fn get_account_instruments(client: &OandaClient) -> Result<InstrumentsResponse, Errors> {
    if let Some(account_id) = client.get_account_id() {
        let url = format!(
            "/v3/accounts/{}/instruments",
            account_id
        );
        let response = client.check_response(
            client.make_request(&url).await
        ).await?;

        let instruments: InstrumentsResponse = serde_json::from_value(response)?;
        Ok(instruments)
    } else {
        Err(Errors::OandaError(OandaError::new("Account ID not set")))
    }
}


mod tests {

    #[allow(unused_imports)]
    use super::*;


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
    async fn test_serialize_instruments_response() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY").expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID").expect("OANDA_ACCOUNT_ID must be set");
        let client = OandaClient::new(Some(&account_id), &api_key);

        match get_account_instruments(&client).await {
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