use serde_json::json;
use reqwest::StatusCode;

use crate::client::OandaClient;
use crate::errors::{Errors, OandaError};


/// Set the client-confguable portions of an Account.
pub async fn patch_configuration(client: &OandaClient, alias: &str, margin_rate: &str) -> Result<(), Errors> {
    if let Some(account_id) = client.get_account_id() {
        let url = format!("/v3/accounts/{}/configuration", account_id);
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