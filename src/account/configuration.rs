use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::client::OandaClient;
use crate::error::APIError;


#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ClientConfigureTransaction {
    pub accountID: String,
    pub batchID: String,
    pub id: String,
    pub marginRate: Option<String>,
    pub alias: Option<String>,
    pub time: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub userID: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ConfigurationResponse {
    pub clientConfigureTransaction: ClientConfigureTransaction,
    pub lastTransactionID: String,
}


impl OandaClient {
    /// Set the client-confguable portions of an Account.
    pub async fn patch_configuration(&self, alias: Option<String>, margin_rate: Option<String>) -> Result<ConfigurationResponse, APIError> {
        if let Some(account_id) = self.get_account_id() {
            let url = format!("/v3/accounts/{}/configuration", account_id);
            let body = json!({
                "alias": alias,
                "marginRate": margin_rate
            });

            let response: Value = self.patch(&url, &body).await?;

            match response.get("status").and_then(|v| v.as_u64()) {
                Some(200) => {
                    let response_body: ConfigurationResponse = serde_json::from_value(response)
                        .map_err(APIError::from)?;
                    Ok(response_body)
                },
                Some(400) => Err(APIError::Other("The configuration specification was invalid".to_string())),
                Some(403) => Err(APIError::Other("The configuration operation was forbidden on the Account".to_string())),
                _ => Err(APIError::Other("Unknown error".to_string())),
            }
        } else {
            Err(APIError::Other("Account ID Not Set".to_string()))
        }
    }
}
