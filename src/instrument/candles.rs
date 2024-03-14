use std::collections::HashMap;
use crate::client::OandaClient;
use crate::errors::{Errors, OandaError};
use serde::Deserialize;


#[derive(Debug)]
pub enum Granularity {
    S5,
    S10,
    S15,
    S30,
    M1,
    M2,
    M4,
    M5,
    M10,
    M15,
    M30,
    H1,
    H2,
    H3,
    H4,
    H6,
    H8,
    H12,
    D,
    W,
    M,
}

impl ToString for Granularity {
    fn to_string(&self) -> String {
        match self {
            Granularity::S5 => "S5",
            Granularity::S10 => "S10",
            Granularity::S15 => "S15",
            Granularity::S30 => "S30",
            Granularity::M1 => "M1",
            Granularity::M2 => "M2",
            Granularity::M4 => "M4",
            Granularity::M5 => "M5",
            Granularity::M10 => "M10",
            Granularity::M15 => "M15",
            Granularity::M30 => "M30",
            Granularity::H1 => "H1",
            Granularity::H2 => "H2",
            Granularity::H3 => "H3",
            Granularity::H4 => "H4",
            Granularity::H6 => "H6",
            Granularity::H8 => "H8",
            Granularity::H12 => "H12",
            Granularity::D => "D",
            Granularity::W => "W",
            Granularity::M => "M",
        }
        .to_string()
    }
}


#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct CandlesResponse {
    candles: Vec<Candle>,
    granularity: String,
    instrument: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Candle {
    complete: bool,
    mid: Mid,
    time: String,
    volume: i32,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct Mid {
    c: String,
    h: String,
    l: String,
    o: String,
}


pub enum CandlesQueryBuilder {
    Count(i32),
    From(String),
    To(String),
    Granularity(Granularity),
    Price(String),
    Smooth(bool),
    IncludeFirst(bool),
    DailyAlignment(i32),
    WeeklyAlignment(i32),
    AlignmentTimezone(String),
}


impl ToString for CandlesQueryBuilder {
    fn to_string(&self) -> String {
        match self {
            CandlesQueryBuilder::Count(v) => v.to_string(),
            CandlesQueryBuilder::From(v) => v.clone(),
            CandlesQueryBuilder::To(v) => v.clone(),
            CandlesQueryBuilder::Granularity(v) => v.to_string(),
            CandlesQueryBuilder::Price(v) => v.clone(),
            CandlesQueryBuilder::Smooth(v) => v.to_string(),
            CandlesQueryBuilder::IncludeFirst(v) => v.to_string(),
            CandlesQueryBuilder::DailyAlignment(v) => v.to_string(),
            CandlesQueryBuilder::WeeklyAlignment(v) => v.to_string(),
            CandlesQueryBuilder::AlignmentTimezone(v) => v.clone(),
        }
    }
}


pub struct CandleQueryBuilder {
    params: HashMap<String, String>,
}


impl CandleQueryBuilder {
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: &str, value: CandlesQueryBuilder) -> &mut Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn build(&self) -> HashMap<String, String> {
        self.params.clone()
    }
}


pub async fn get_candles(
    client: &OandaClient,
    instrument: &str,
    query: HashMap<String, String>,
) -> Result<CandlesResponse, Errors> {
    let mut url = format!("/v3/instruments/{}/candles?",instrument);

    for (key, value) in query {
        url.push_str(&format!("{}={}&", key, value));
    }
    
    let response = client.make_request(&url).await?;
    let candles_response: Result<CandlesResponse, _> = serde_json::from_value(response.clone());

    if let Ok(candles) = candles_response {
        Ok(candles)
    } else {
        let api_error: Result<OandaError, _> = serde_json::from_value(response.clone());
        match api_error {
            Ok(e) => Err(Errors::OandaError(e)),
            Err(_) => Err(Errors::CustomError(String::from("Failed to deserialize response"))),
        }
    }
}


mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn test_get_candles() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID")
            .expect("OANDA_ACCOUNT_ID must be set");
        let client = OandaClient::new(Some(&account_id), &api_key);

        let mut query = CandleQueryBuilder::new();
        query.add("count", CandlesQueryBuilder::Count(5));
        query.add("granularity", CandlesQueryBuilder::Granularity(Granularity::H1));

        let response = get_candles(&client, "EUR_USD", query.build()).await;

        match response {
            Ok(v) => {
                println!("Response: {:?}", v);
                assert!(true);
            }
            Err(e) => {
                println!("Error: {}", e);
                assert!(false);
            }
        }
    }


    #[tokio::test]
    async fn test_get_candles_from_to() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID")
            .expect("OANDA_ACCOUNT_ID must be set");
        let client = OandaClient::new(Some(&account_id), &api_key);

        let mut query = CandleQueryBuilder::new();
        query.add("from", CandlesQueryBuilder::From("2021-01-04T00:00:00Z".to_string()));
        query.add("to", CandlesQueryBuilder::To("2021-01-05T00:00:00Z".to_string()));
        query.add("granularity", CandlesQueryBuilder::Granularity(Granularity::H1));

        let response = get_candles(&client, "EUR_USD", query.build()).await;

        match response {
            Ok(v) => {
                println!("Response: {:?}", v);
                assert!(true);
            }
            Err(e) => {
                println!("Error: {}", e);
                assert!(false);
            }
        }
    }

}