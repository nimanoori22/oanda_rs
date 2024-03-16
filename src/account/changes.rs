use serde::{Serialize, Deserialize};

use crate::client::OandaClient;
use crate::errors::{Errors, OandaError};


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ChangesResponse {
    changes: Changes,
    lastTransactionID: String,
    state: State,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Changes {
    ordersCancelled: Vec<Order>,
    ordersCreated: Vec<Order>,
    ordersFilled: Vec<Order>,
    ordersTriggered: Vec<Order>,
    positions: Vec<Position>,
    tradesClosed: Vec<Trade>,
    tradesOpened: Vec<Trade>,
    tradesReduced: Vec<Trade>,
    trasactions: Vec<Transaction>,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Order {
    createTime: String,
    filledTime: String,
    fillingTransactionID: String,
    id: String,
    instrument: String,
    positionFill: String,
    state: String,
    timeInForce: String,
    tradeOpenedID: String,
    type_: String,
    units: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Position {
    instrument: String,
    long: PositionDetails,
    pl: String,
    resettablePL: String,
    short: PositionDetails,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PositionDetails {
    #[serde(default)]
    averagePrice: String,
    pl: String,
    resettablePL: String,
    #[serde(default)]
    tradeIDs: Vec<String>,
    units: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Trade {
    currentUnits: String,
    financing: String,
    id: String,
    initialUnits: String,
    instrument: String,
    openTime: String,
    price: String,
    realizedPL: String,
    state: String,
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
    accountID: String,
    batchID: String,
    id: String,
    instrument: String,
    positionFill: String,
    reason: String,
    time: String,
    timeInForce: String,
    r#type: String,
    units: String,
    userID: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct OrderFillTransaction {
    accountBalance: String,
    accountID: String,
    batchID: String,
    financing: String,
    id: String,
    instrument: String,
    orderID: String,
    pl: String,
    price: String,
    reason: String,
    time: String,
    tradeOpened: TradeOpened,
    r#type: String,
    units: String,
    userID: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TradeOpened {
    tradeID: String,
    units: String,
}


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct State {
    NAV: String,
    marginAvailable: String,
    marginCloseoutMarginUsed: String,
    marginCloseoutNAV: String,
    marginCloseoutPercent: String,
    marginCloseoutUnrealizedPL: String,
    marginUsed: String,
    orders: Vec<String>,
    positionValue: String,
    positions: Vec<StatePosition>,
    trades: Vec<StateTrade>,
    unrealizedPL: String,
    withdrawalLimit: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct StatePosition {
    instrument: String,
    longUnrealizedPL: String,
    netUnrealizedPL: String,
    shortUnrealizedPL: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct StateTrade {
    id: String,
    unrealizedPL: String,
}

/// Endpoint used to poll an Account for its current state and changes since a specified TransactionID
/// TODO: test this function with a valid transaction_id
pub async fn get_changes(client: &OandaClient, transaction_id: &String) -> Result<ChangesResponse, Errors> {
    if let Some(account_id) = client.get_account_id() {
        let url = format!(
            "/v3/accounts/{}/changes?sinceTransactionID={}",
            account_id,
            transaction_id
        );
        let response = client.check_response(
            client.make_request(&url).await
        ).await?;
        let changes : ChangesResponse = serde_json::from_value(response).map_err(Errors::from)?;
        Ok(changes)
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
        let transaction_id = "6357".to_string();

        match get_changes(&client, &transaction_id).await {
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