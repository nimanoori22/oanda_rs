use serde::{Serialize, Deserialize};

use crate::client::OandaClient;
use crate::error::APIError;


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ChangesResponse {
    pub changes: Changes,
    pub lastTransactionID: String,
    pub state: State,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Changes {
    pub ordersCancelled: Vec<Order>,
    pub ordersCreated: Vec<Order>,
    pub ordersFilled: Vec<Order>,
    pub ordersTriggered: Vec<Order>,
    pub positions: Vec<Position>,
    pub tradesClosed: Vec<Trade>,
    pub tradesOpened: Vec<Trade>,
    pub tradesReduced: Vec<Trade>,
    pub trasactions: Vec<Transaction>,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Order {
    pub createTime: String,
    pub filledTime: String,
    pub fillingTransactionID: String,
    pub id: String,
    pub instrument: String,
    pub positionFill: String,
    pub state: String,
    pub timeInForce: String,
    pub tradeOpenedID: String,
    pub type_: String,
    pub units: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Position {
    pub instrument: String,
    pub long: PositionDetails,
    pub pl: String,
    pub resettablePL: String,
    pub short: PositionDetails,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PositionDetails {
    #[serde(default)]
    pub averagePrice: String,
    pub pl: String,
    pub resettablePL: String,
    #[serde(default)]
    pub tradeIDs: Vec<String>,
    pub units: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Trade {
    pub currentUnits: String,
    pub financing: String,
    pub id: String,
    pub initialUnits: String,
    pub instrument: String,
    pub openTime: String,
    pub price: String,
    pub realizedPL: String,
    pub state: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Transaction {
    MarketOrder(MarketOrderTransaction),
    OrderFill(OrderFillTransaction),
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct MarketOrderTransaction {
    pub accountID: String,
    pub batchID: String,
    pub id: String,
    pub instrument: String,
    pub positionFill: String,
    pub reason: String,
    pub time: String,
    pub timeInForce: String,
    pub r#type: String,
    pub units: String,
    pub userID: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct OrderFillTransaction {
    pub accountBalance: String,
    pub accountID: String,
    pub batchID: String,
    pub financing: String,
    pub id: String,
    pub instrument: String,
    pub orderID: String,
    pub pl: String,
    pub price: String,
    pub reason: String,
    pub time: String,
    pub tradeOpened: TradeOpened,
    pub r#type: String,
    pub units: String,
    pub userID: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TradeOpened {
    pub tradeID: String,
    pub units: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct State {
    pub NAV: String,
    pub marginAvailable: String,
    pub marginCloseoutMarginUsed: String,
    pub marginCloseoutNAV: String,
    pub marginCloseoutPercent: String,
    pub marginCloseoutUnrealizedPL: String,
    pub marginUsed: String,
    pub orders: Vec<String>,
    pub positionValue: String,
    pub positions: Vec<StatePosition>,
    pub trades: Vec<StateTrade>,
    pub unrealizedPL: String,
    pub withdrawalLimit: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct StatePosition {
    pub instrument: String,
    pub longUnrealizedPL: String,
    pub netUnrealizedPL: String,
    pub shortUnrealizedPL: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct StateTrade {
    pub id: String,
    pub unrealizedPL: String,
}

/// Endpoint used to poll an Account for its current state and changes since a specified TransactionID
/// TODO: test this function with a valid transaction_id
impl OandaClient {
    pub async fn get_changes(&self, transaction_id: &String) -> Result<ChangesResponse, APIError> {
        if let Some(account_id) = self.get_account_id() {
            let url = format!(
                "/v3/accounts/{}/changes?sinceTransactionID={}",
                account_id,
                transaction_id
            );
            let response = self.check_response(
                self.make_request(&url).await
            ).await?;
            let changes : ChangesResponse = serde_json::from_value(response).map_err(APIError::from)?;
            Ok(changes)
        } else {
            Err(APIError::Other("Account ID Not Set".to_string()))
        }
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
        let client = OandaClient::new(Some(&account_id), &api_key).unwrap();
        let transaction_id = "6357".to_string();

        match client.get_changes(&transaction_id).await {
            Ok(response) => {
                println!("Response: {:?}", response);
                assert_eq!(response.lastTransactionID, transaction_id);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                assert!(false);
            }
        }
    }
}