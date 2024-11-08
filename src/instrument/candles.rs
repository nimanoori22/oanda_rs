use std::collections::HashMap;
use crate::client::OandaClient;
use crate::error::APIError;
use serde::{Serialize, Deserialize};
use crate::error::APIError as Err;


#[derive(Debug, Clone)]
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


impl Granularity {
    pub fn from_str(s: &str) -> Result<Granularity, Err> {
        match s {
            "S5" => Ok(Granularity::S5),
            "S10" => Ok(Granularity::S10),
            "S15" => Ok(Granularity::S15),
            "S30" => Ok(Granularity::S30),
            "M1" => Ok(Granularity::M1),
            "M2" => Ok(Granularity::M2),
            "M4" => Ok(Granularity::M4),
            "M5" => Ok(Granularity::M5),
            "M10" => Ok(Granularity::M10),
            "M15" => Ok(Granularity::M15),
            "M30" => Ok(Granularity::M30),
            "H1" => Ok(Granularity::H1),
            "H2" => Ok(Granularity::H2),
            "H3" => Ok(Granularity::H3),
            "H4" => Ok(Granularity::H4),
            "H6" => Ok(Granularity::H6),
            "H8" => Ok(Granularity::H8),
            "H12" => Ok(Granularity::H12),
            "D" => Ok(Granularity::D),
            "W" => Ok(Granularity::W),
            "M" => Ok(Granularity::M),
            _ => Err(APIError::Other("Invalid granularity".to_string())),
        }
    }
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


#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct CandlesResponse {
    pub candles: Vec<Candle>,
    pub granularity: String,
    pub instrument: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Candle {
    pub complete: bool,
    pub mid: Mid,
    pub time: String,
    pub volume: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct Mid {
    pub c: String,
    pub h: String,
    pub l: String,
    pub o: String,
}


#[derive(Debug, Clone)]
pub enum CandleQueryParam {
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


impl ToString for CandleQueryParam {
    fn to_string(&self) -> String {
        match self {
            CandleQueryParam::Count(v) => v.to_string(),
            CandleQueryParam::From(v) => v.clone(),
            CandleQueryParam::To(v) => v.clone(),
            CandleQueryParam::Granularity(v) => v.to_string(),
            CandleQueryParam::Price(v) => v.clone(),
            CandleQueryParam::Smooth(v) => v.to_string(),
            CandleQueryParam::IncludeFirst(v) => v.to_string(),
            CandleQueryParam::DailyAlignment(v) => v.to_string(),
            CandleQueryParam::WeeklyAlignment(v) => v.to_string(),
            CandleQueryParam::AlignmentTimezone(v) => v.clone(),
        }
    }
}


#[derive(Debug, Clone)]
pub struct CandleQuery {
    parameters: HashMap<String, String>,
}


impl CandleQuery {
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
        }
    }

    pub fn add_param(&mut self, key: &str, value: CandleQueryParam) -> &mut Self {
        self.parameters.insert(key.to_string(), value.to_string());
        self
    }

    pub fn build(&self) -> HashMap<String, String> {
        self.parameters.clone()
    }
}


impl OandaClient
{
    pub async fn get_candles(
        &mut self,
        instrument: &str,
        query: HashMap<String, String>,
    ) -> Result<CandlesResponse, APIError> {
        let mut url = format!("/v3/instruments/{}/candles?", instrument);

        for (key, value) in query {
            url.push_str(&format!("{}={}&", key, value));
        }

        let response = self.get(&url).await?;
        let candles: CandlesResponse = serde_json::from_value(response)?;
        Ok(candles)
    }
}


mod tests {

    use std::{sync::{atomic::{AtomicUsize, Ordering}, Arc}, time::Instant};
    use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

    use futures::{stream, StreamExt};

    #[allow(unused_imports)]
    use super::*;

    #[tokio::test]
    async fn test_get_candles() {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID")
            .expect("OANDA_ACCOUNT_ID must be set");
        let client_result = OandaClient::new(
                    Some(&account_id), 
                    &api_key, 
                    100,
                    100,
                    100,
                    5
                );

        let mut client = match client_result {
            Ok(v) => v,
            Err(e) => {
                println!("Error: {}", e);
                assert!(false);
                return;
            }
        };

        let mut query = CandleQuery::new();
        query.add_param("count", CandleQueryParam::Count(5));
        query.add_param("granularity", CandleQueryParam::Granularity(Granularity::H1));

        let response = client.get_candles("EUR_USD", query.build()).await;

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

        let client_result = OandaClient::new(Some(&account_id), &api_key, 100, 100, 100, 5);
        let mut client = match client_result {
            Ok(v) => v,
            Err(e) => {
                println!("Error: {}", e);
                assert!(false);
                return;
            }
        };

        let mut query = CandleQuery::new();
        query.add_param("from", CandleQueryParam::From("2021-01-04T00:00:00Z".to_string()));
        query.add_param("to", CandleQueryParam::To("2021-01-05T00:00:00Z".to_string()));
        query.add_param("granularity", CandleQueryParam::Granularity(Granularity::H1));

        let response = client.get_candles("EUR_USD", query.build()).await;
        println!("Response: {:?}", response);
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
    async fn test_get_candles_async() -> Result<(), APIError> {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OANDA_API_KEY")
            .expect("OANDA_API_KEY must be set");
        let account_id = std::env::var("OANDA_ACCOUNT_ID")
            .expect("OANDA_ACCOUNT_ID must be set");

        let client = OandaClient::new(
            Some(&account_id), 
            &api_key, 
            100,
            100,
            100,
            5
        ).unwrap();

        let dates = vec![
                ["2023-12-30T12:00:00Z", "2024-01-02T23:20:00Z"], 
                ["2024-01-02T23:20:00Z", "2024-01-06T10:40:00Z"], 
                ["2024-01-06T10:40:00Z", "2024-01-09T22:00:00Z"], 
                ["2024-01-09T22:00:00Z", "2024-01-13T09:20:00Z"], 
                ["2024-01-13T09:20:00Z", "2024-01-16T20:40:00Z"], 
                ["2024-01-16T20:40:00Z", "2024-01-20T08:00:00Z"], 
                ["2024-01-20T08:00:00Z", "2024-01-23T19:20:00Z"], 
                ["2024-01-23T19:20:00Z", "2024-01-27T06:40:00Z"], 
                ["2024-01-27T06:40:00Z", "2024-01-30T18:00:00Z"], 
                ["2024-01-30T18:00:00Z", "2024-02-03T05:20:00Z"], 
                ["2024-02-03T05:20:00Z", "2024-02-06T16:40:00Z"], 
                ["2024-02-06T16:40:00Z", "2024-02-10T04:00:00Z"], 
                ["2024-02-10T04:00:00Z", "2024-02-13T15:20:00Z"], 
                ["2024-02-13T15:20:00Z", "2024-02-17T02:40:00Z"], 
                ["2024-02-17T02:40:00Z", "2024-02-20T14:00:00Z"], 
                ["2024-02-20T14:00:00Z", "2024-02-24T01:20:00Z"], 
                ["2024-02-24T01:20:00Z", "2024-02-27T12:40:00Z"], 
                ["2024-02-27T12:40:00Z", "2024-03-02T00:00:00Z"], 
                ["2024-03-02T00:00:00Z", "2024-03-05T11:20:00Z"], 
                ["2024-03-05T11:20:00Z", "2024-03-08T22:40:00Z"], 
                ["2024-03-08T22:40:00Z", "2024-03-12T10:00:00Z"], 
                ["2024-03-12T10:00:00Z", "2024-03-15T21:20:00Z"], 
                ["2024-03-15T21:20:00Z", "2024-03-19T08:40:00Z"], 
                ["2024-03-19T08:40:00Z", "2024-03-22T20:00:00Z"], 
                ["2024-03-22T20:00:00Z", "2024-03-26T07:20:00Z"], 
                ["2024-03-26T07:20:00Z", "2024-03-29T18:40:00Z"],
                ["2024-03-29T18:40:00Z", "2024-04-02T06:00:00Z"], 
                ["2024-04-02T06:00:00Z", "2024-04-05T17:20:00Z"], 
                ["2024-04-05T17:20:00Z", "2024-04-09T04:40:00Z"], 
                ["2024-04-09T04:40:00Z", "2024-04-12T16:00:00Z"], 
                ["2024-04-12T16:00:00Z", "2024-04-16T03:20:00Z"], 
                ["2024-04-16T03:20:00Z", "2024-04-19T14:40:00Z"], 
                ["2024-04-19T14:40:00Z", "2024-04-23T02:00:00Z"], 
                ["2024-04-23T02:00:00Z", "2024-04-26T13:20:00Z"], 
                ["2024-04-26T13:20:00Z", "2024-04-30T00:40:00Z"], 
                ["2024-04-30T00:40:00Z", "2024-05-03T12:00:00Z"], 
                ["2024-05-03T12:00:00Z", "2024-05-06T23:20:00Z"], 
                ["2024-05-06T23:20:00Z", "2024-05-10T10:40:00Z"], 
                ["2024-05-10T10:40:00Z", "2024-05-13T22:00:00Z"], 
                ["2024-05-13T22:00:00Z", "2024-05-17T09:20:00Z"], 
                ["2024-05-17T09:20:00Z", "2024-05-20T20:40:00Z"], 
                ["2024-05-20T20:40:00Z", "2024-05-24T08:00:00Z"], 
                ["2024-05-24T08:00:00Z", "2024-05-27T19:20:00Z"], 
                ["2024-05-27T19:20:00Z", "2024-05-31T06:40:00Z"], 
                ["2024-05-31T06:40:00Z", "2024-06-03T18:00:00Z"], 
                ["2024-06-03T18:00:00Z", "2024-06-07T05:20:00Z"], 
                ["2024-06-07T05:20:00Z", "2024-06-10T16:40:00Z"], 
                ["2024-06-10T16:40:00Z", "2024-06-14T04:00:00Z"], 
                ["2024-06-14T04:00:00Z", "2024-06-17T15:20:00Z"], 
                ["2024-06-17T15:20:00Z", "2024-06-21T02:40:00Z"], 
                ["2024-06-21T02:40:00Z", "2024-06-24T14:00:00Z"], 
                ["2024-06-24T14:00:00Z", "2024-06-28T01:20:00Z"], 
                ["2024-06-28T01:20:00Z", "2024-07-01T12:40:00Z"], 
                ["2024-07-01T12:40:00Z", "2024-07-05T00:00:00Z"], 
                ["2024-07-05T00:00:00Z", "2024-07-08T11:20:00Z"], 
                ["2024-07-08T11:20:00Z", "2024-07-11T22:40:00Z"], 
                ["2024-07-11T22:40:00Z", "2024-07-15T10:00:00Z"], 
                ["2024-07-15T10:00:00Z", "2024-07-18T21:20:00Z"], 
                ["2024-07-18T21:20:00Z", "2024-07-22T08:40:00Z"], 
                ["2024-07-22T08:40:00Z", "2024-07-25T20:00:00Z"], 
                ["2024-07-25T20:00:00Z", "2024-07-29T07:20:00Z"], 
                ["2024-07-29T07:20:00Z", "2024-08-01T18:40:00Z"], 
                ["2024-08-01T18:40:00Z", "2024-08-05T06:00:00Z"], 
                ["2024-08-05T06:00:00Z", "2024-08-08T17:20:00Z"], 
                ["2024-08-08T17:20:00Z", "2024-08-12T04:40:00Z"], 
                ["2024-08-12T04:40:00Z", "2024-08-15T16:00:00Z"], 
                ["2024-08-15T16:00:00Z", "2024-08-19T03:20:00Z"], 
                ["2024-08-19T03:20:00Z", "2024-08-22T14:40:00Z"], 
                ["2024-08-22T14:40:00Z", "2024-08-26T02:00:00Z"], 
                ["2024-08-26T02:00:00Z", "2024-08-29T13:20:00Z"], 
                ["2024-08-29T13:20:00Z", "2024-08-31T17:58:17Z"]
               ];

        // Start timing
        let start = Instant::now();
        
        let responses = stream::iter(dates)
        .map(|date|{
                let mut client = client.clone();
                async move {
                let mut query = CandleQuery::new();
                query.add_param("from", CandleQueryParam::From(date[0].to_string()));
                query.add_param("to", CandleQueryParam::To(date[1].to_string()));
                query.add_param("granularity", CandleQueryParam::Granularity(Granularity::M1));
                let json = client
                    .get_candles("EUR_USD", query.build())
                    .await?;
                Ok::<CandlesResponse, APIError>(json)
                }
            })
            .buffer_unordered(100);

        let count = Arc::new(AtomicUsize::new(0));


        responses
        .for_each(|response| async {
            let count = Arc::clone(&count);
            match response {
                Ok(body) => {
                    println!("-------------------------------------------------------");
                    println!("Body: {:?}", body.candles.len());
                    println!("-------------------------------------------------------");
                    count.fetch_add(1, Ordering::SeqCst);
                    println!("Count: {:?}", count.load(Ordering::SeqCst));

                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    count.fetch_add(1, Ordering::SeqCst);
                    println!("Count: {:?}", count.load(Ordering::SeqCst));
                }
            }
        })
        .await;

        // End timing
        let duration = start.elapsed();
        println!("Time elapsed in expensive_function() is: {:?}", duration);
        Ok(())
    }

// this function is here for comparing the perfomance
// of oanda_rs and raw reqwest client
#[tokio::test]
async fn test_get_candles_from_oanda() -> Result<(), APIError> {
    let client = reqwest::Client::new();

    let dates = vec![
                ["2023-12-30T12:00:00Z", "2024-01-02T23:20:00Z"], 
                ["2024-01-02T23:20:00Z", "2024-01-06T10:40:00Z"], 
                ["2024-01-06T10:40:00Z", "2024-01-09T22:00:00Z"], 
                ["2024-01-09T22:00:00Z", "2024-01-13T09:20:00Z"], 
                ["2024-01-13T09:20:00Z", "2024-01-16T20:40:00Z"], 
                ["2024-01-16T20:40:00Z", "2024-01-20T08:00:00Z"], 
                ["2024-01-20T08:00:00Z", "2024-01-23T19:20:00Z"], 
                ["2024-01-23T19:20:00Z", "2024-01-27T06:40:00Z"], 
                ["2024-01-27T06:40:00Z", "2024-01-30T18:00:00Z"], 
                ["2024-01-30T18:00:00Z", "2024-02-03T05:20:00Z"], 
                ["2024-02-03T05:20:00Z", "2024-02-06T16:40:00Z"], 
                ["2024-02-06T16:40:00Z", "2024-02-10T04:00:00Z"], 
                ["2024-02-10T04:00:00Z", "2024-02-13T15:20:00Z"], 
                ["2024-02-13T15:20:00Z", "2024-02-17T02:40:00Z"], 
                ["2024-02-17T02:40:00Z", "2024-02-20T14:00:00Z"], 
                ["2024-02-20T14:00:00Z", "2024-02-24T01:20:00Z"], 
                ["2024-02-24T01:20:00Z", "2024-02-27T12:40:00Z"], 
                ["2024-02-27T12:40:00Z", "2024-03-02T00:00:00Z"], 
                ["2024-03-02T00:00:00Z", "2024-03-05T11:20:00Z"], 
                ["2024-03-05T11:20:00Z", "2024-03-08T22:40:00Z"], 
                ["2024-03-08T22:40:00Z", "2024-03-12T10:00:00Z"], 
                ["2024-03-12T10:00:00Z", "2024-03-15T21:20:00Z"], 
                ["2024-03-15T21:20:00Z", "2024-03-19T08:40:00Z"], 
                ["2024-03-19T08:40:00Z", "2024-03-22T20:00:00Z"], 
                ["2024-03-22T20:00:00Z", "2024-03-26T07:20:00Z"], 
                ["2024-03-26T07:20:00Z", "2024-03-29T18:40:00Z"],
                ["2024-03-29T18:40:00Z", "2024-04-02T06:00:00Z"], 
                ["2024-04-02T06:00:00Z", "2024-04-05T17:20:00Z"], 
                ["2024-04-05T17:20:00Z", "2024-04-09T04:40:00Z"], 
                ["2024-04-09T04:40:00Z", "2024-04-12T16:00:00Z"], 
                ["2024-04-12T16:00:00Z", "2024-04-16T03:20:00Z"], 
                ["2024-04-16T03:20:00Z", "2024-04-19T14:40:00Z"], 
                ["2024-04-19T14:40:00Z", "2024-04-23T02:00:00Z"], 
                ["2024-04-23T02:00:00Z", "2024-04-26T13:20:00Z"], 
                ["2024-04-26T13:20:00Z", "2024-04-30T00:40:00Z"], 
                ["2024-04-30T00:40:00Z", "2024-05-03T12:00:00Z"], 
                ["2024-05-03T12:00:00Z", "2024-05-06T23:20:00Z"], 
                ["2024-05-06T23:20:00Z", "2024-05-10T10:40:00Z"], 
                ["2024-05-10T10:40:00Z", "2024-05-13T22:00:00Z"], 
                ["2024-05-13T22:00:00Z", "2024-05-17T09:20:00Z"], 
                ["2024-05-17T09:20:00Z", "2024-05-20T20:40:00Z"], 
                ["2024-05-20T20:40:00Z", "2024-05-24T08:00:00Z"], 
                ["2024-05-24T08:00:00Z", "2024-05-27T19:20:00Z"], 
                ["2024-05-27T19:20:00Z", "2024-05-31T06:40:00Z"], 
                ["2024-05-31T06:40:00Z", "2024-06-03T18:00:00Z"], 
                ["2024-06-03T18:00:00Z", "2024-06-07T05:20:00Z"], 
                ["2024-06-07T05:20:00Z", "2024-06-10T16:40:00Z"], 
                ["2024-06-10T16:40:00Z", "2024-06-14T04:00:00Z"], 
                ["2024-06-14T04:00:00Z", "2024-06-17T15:20:00Z"], 
                ["2024-06-17T15:20:00Z", "2024-06-21T02:40:00Z"], 
                ["2024-06-21T02:40:00Z", "2024-06-24T14:00:00Z"], 
                ["2024-06-24T14:00:00Z", "2024-06-28T01:20:00Z"], 
                ["2024-06-28T01:20:00Z", "2024-07-01T12:40:00Z"], 
                ["2024-07-01T12:40:00Z", "2024-07-05T00:00:00Z"], 
                ["2024-07-05T00:00:00Z", "2024-07-08T11:20:00Z"], 
                ["2024-07-08T11:20:00Z", "2024-07-11T22:40:00Z"], 
                ["2024-07-11T22:40:00Z", "2024-07-15T10:00:00Z"], 
                ["2024-07-15T10:00:00Z", "2024-07-18T21:20:00Z"], 
                ["2024-07-18T21:20:00Z", "2024-07-22T08:40:00Z"], 
                ["2024-07-22T08:40:00Z", "2024-07-25T20:00:00Z"], 
                ["2024-07-25T20:00:00Z", "2024-07-29T07:20:00Z"], 
                ["2024-07-29T07:20:00Z", "2024-08-01T18:40:00Z"], 
                ["2024-08-01T18:40:00Z", "2024-08-05T06:00:00Z"], 
                ["2024-08-05T06:00:00Z", "2024-08-08T17:20:00Z"], 
                ["2024-08-08T17:20:00Z", "2024-08-12T04:40:00Z"], 
                ["2024-08-12T04:40:00Z", "2024-08-15T16:00:00Z"], 
                ["2024-08-15T16:00:00Z", "2024-08-19T03:20:00Z"], 
                ["2024-08-19T03:20:00Z", "2024-08-22T14:40:00Z"], 
                ["2024-08-22T14:40:00Z", "2024-08-26T02:00:00Z"], 
                ["2024-08-26T02:00:00Z", "2024-08-29T13:20:00Z"], 
                ["2024-08-29T13:20:00Z", "2024-08-31T17:58:17Z"]
               ];

    let mut urls: Vec<String> = Vec::new();

    for date_batch in dates {
        let url = format!(
            "https://api-fxpractice.oanda.com/v3/instruments/EUR_USD/candles?from={}&to={}&granularity=M1",
            date_batch.first().unwrap(),
            date_batch.last().unwrap()
        );
        urls.push(url);
    }

    // Create headers
    dotenv::dotenv().ok();
    let api_key = std::env::var("OANDA_API_KEY").expect("OANDA_API_KEY must be set");
    let account_id = std::env::var("OANDA_ACCOUNT_ID").expect("OANDA_ACCOUNT_ID must be set");

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap());
    headers.insert("Account-ID", HeaderValue::from_str(&account_id).unwrap());

    // Start timing
    let start = Instant::now();

    let responses = stream::iter(urls)
        .map(|url| {
            let client = &client;
            let headers = headers.clone();
            async move {
                let response = client.get(&url).headers(headers).send().await?;
                let json: CandlesResponse = response.json().await?;
                Ok::<CandlesResponse, reqwest::Error>(json)
            }
        })
        .buffer_unordered(100);

    let count = Arc::new(AtomicUsize::new(0));    

    responses
        .for_each(|response| async {
            let count = Arc::clone(&count);
            match response {
                Ok(body) => {
                    println!("-------------------------------------------------------");
                    println!("Body: {:?}", body.candles.len());
                    println!("-------------------------------------------------------");
                    count.fetch_add(1, Ordering::SeqCst);
                    println!("Count: {:?}", count.load(Ordering::SeqCst));

                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    count.fetch_add(1, Ordering::SeqCst);
                    println!("Count: {:?}", count.load(Ordering::SeqCst));
                }
            }
        })
        .await;
    
    // End timing
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);

    Ok(())
}

}