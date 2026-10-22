use oanda_rs::{
    client::{ClientConfig, OandaClient},
    instrument::candles::{CandleQuery, CandleQueryParam, Granularity},
};

#[test]
fn practice_credentials_and_read_only_endpoints() {
    dotenv::dotenv().expect("Could not load .env; link it to ~/.config/oanda/credentials");
    assert_eq!(
        std::env::var("OANDA_ENVIRONMENT").as_deref(),
        Ok("practice"),
        "This smoke test requires OANDA_ENVIRONMENT=practice"
    );
    let token = std::env::var("OANDA_API_TOKEN").expect("OANDA_API_TOKEN must be set");
    assert!(
        !token.trim().is_empty(),
        "OANDA_API_TOKEN must not be empty"
    );
    let mut client = OandaClient::new_with_config(
        None,
        &token,
        ClientConfig {
            concurrency_limit: 1,
            rate_limit: 10,
            retry_attempts: 1,
            proxy: std::env::var("OANDA_PROXY").ok(),
            ..ClientConfig::default()
        },
    )
    .unwrap();
    let checks = async {
        let accounts = client.get_accounts().await.expect("Account listing failed");
        assert!(
            !accounts.accounts.is_empty(),
            "No accessible practice accounts"
        );
        let account_id = std::env::var("OANDA_ACCOUNT_ID")
            .ok()
            .filter(|id| !id.trim().is_empty() && id != "your_demo_account_id")
            .unwrap_or_else(|| accounts.accounts[0].id.clone());
        assert!(
            accounts
                .accounts
                .iter()
                .any(|account| account.id == account_id),
            "Configured account is not accessible with this token"
        );
        client.set_account_id(&account_id);
        let summary = client
            .get_account_summary()
            .await
            .expect("Account summary failed");
        assert!(
            summary.account.id == account_id,
            "Account summary ID mismatch"
        );
        assert!(
            !summary.lastTransactionID.is_empty(),
            "Missing transaction ID"
        );
        let mut query = CandleQuery::new();
        query.add_param("count", CandleQueryParam::Count(5));
        query.add_param(
            "granularity",
            CandleQueryParam::Granularity(Granularity::H1),
        );
        let candles = client
            .get_candles("EUR_USD", query.build())
            .await
            .expect("Candles failed");
        assert_eq!(candles.candles.len(), 5, "Expected five hourly candles");
    };
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime
        .block_on(async { tokio::time::timeout(std::time::Duration::from_secs(45), checks).await });
    // DNS resolution may outlive cancellation; do not wait for its blocking worker.
    runtime.shutdown_background();
    result.expect("Practice API checks exceeded 45 seconds");
}

#[test]
fn save_latest_100_hourly_candles() {
    dotenv::dotenv().expect("Could not load .env; link it to ~/.config/oanda/credentials");
    assert_eq!(
        std::env::var("OANDA_ENVIRONMENT").as_deref(),
        Ok("practice")
    );
    let token = std::env::var("OANDA_API_TOKEN").expect("OANDA_API_TOKEN must be set");
    let mut client = OandaClient::new_with_config(
        None,
        &token,
        ClientConfig {
            proxy: std::env::var("OANDA_PROXY").ok(),
            retry_attempts: 1,
            ..ClientConfig::default()
        },
    )
    .unwrap();
    let mut query = CandleQuery::new();
    query.add_param("count", CandleQueryParam::Count(100));
    query.add_param(
        "granularity",
        CandleQueryParam::Granularity(Granularity::H1),
    );
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(async {
        tokio::time::timeout(
            std::time::Duration::from_secs(45),
            client.get_candles("EUR_USD", query.build()),
        )
        .await
    });
    runtime.shutdown_background();
    let response = result
        .expect("Candle request exceeded 45 seconds")
        .expect("Failed to fetch hourly candles");
    assert_eq!(response.instrument, "EUR_USD");
    assert_eq!(response.granularity, "H1");
    assert_eq!(response.candles.len(), 100);
    assert!(
        response
            .candles
            .windows(2)
            .all(|pair| pair[0].time < pair[1].time),
        "Candles must be in chronological order"
    );
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("eur_usd_h1_100.json");
    let json = serde_json::to_vec_pretty(&response).expect("Could not serialize candles");
    std::fs::write(&output, json).expect("Could not save candle file");
    println!("Saved 100 EUR_USD H1 candles to {}", output.display());
}
