use serde::{Deserialize, Serialize};
use serde_json::json;
use reqwest::StatusCode;

use crate::client::OandaClient;
use crate::errors::{Errors, OandaError};


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
    pub async fn patch_configuration(&self, alias: Option<String>, margin_rate: Option<String>) -> Result<ConfigurationResponse, Errors> {
        if let Some(account_id) = self.get_account_id() {
            let url = format!("/v3/accounts/{}/configuration", account_id);
            let body = json!({
                "alias": alias,
                "marginRate": margin_rate
            });

            let response = self.patch(&url, &body).await?;

            match response.status() {
                StatusCode::OK => {
                    let response_body: ConfigurationResponse = serde_json::from_str(
                        &response.text().await?
                    ).map_err(Errors::from)?;
                    Ok(response_body)
                },
                StatusCode::BAD_REQUEST => Err(Errors::OandaError(OandaError::new("The configuration specification was invalid"))),
                StatusCode::FORBIDDEN => Err(Errors::OandaError(OandaError::new("The configuration operation was forbidden on the Account"))),
                _ => Err(Errors::OandaError(OandaError::new("Unknown error"))),
            }
        } else {
            Err(Errors::OandaError(OandaError::new("Account ID not set")))
        }
    }
}

// pub async fn patch_configuration(client: &OandaClient, alias: Option<String>, margin_rate: Option<String>) -> Result<ConfigurationResponse, Errors> {
//     if let Some(account_id) = client.get_account_id() {
//         let url = format!("/v3/accounts/{}/configuration", account_id);
//         let body = json!({
//             "alias": alias,
//             "marginRate": margin_rate
//         });

//         let res = client.patch(&url, &body).await?;

//         match res.status() {
//             StatusCode::OK => {
//                 let response_body: ConfigurationResponse = serde_json::from_str(
//                     &res.text().await?
//                 ).map_err(Errors::from)?;
//                 Ok(response_body)
//             },
//             StatusCode::BAD_REQUEST => Err(Errors::OandaError(OandaError::new("The configuration specification was invalid"))),
//             StatusCode::FORBIDDEN => Err(Errors::OandaError(OandaError::new("The configuration operation was forbidden on the Account"))),
//             _ => Err(Errors::OandaError(OandaError::new("Unknown error"))),
//         }
//     } else {
//         Err(Errors::OandaError(OandaError::new("Account ID not set")))
//     }
// }