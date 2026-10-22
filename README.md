# oanda_rs

An asynchronous Rust library for the OANDA v20 REST API, with account data,
historical candles, rate limiting, retries, and optional proxy support.

## Getting started

```bash
git clone https://github.com/nimanoori22/oanda_rs.git
cd oanda_rs
cargo build
```

## Usage

Create a `.env` file in your project directory and add your OANDA API key:

```dotenv
OANDA_API_TOKEN=your_api_key
```

Load it with `dotenv` before creating the client. `.env` is ignored by Git.

### Get hourly candles

```rust
use dotenv::dotenv;
use oanda_rs::{
    client::{ClientConfig, OandaClient},
    instrument::candles::{CandleQuery, CandleQueryParam, Granularity},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv()?;
    let api_key = std::env::var("OANDA_API_TOKEN")?;
    let mut client = OandaClient::new_with_config(None, &api_key, ClientConfig::default())?;

    let mut query = CandleQuery::new();
    query.add_param("count", CandleQueryParam::Count(100));
    query.add_param("granularity", CandleQueryParam::Granularity(Granularity::H1));

    let response = client.get_candles("EUR_USD", query.build()).await?;
    println!("{:?}", response);
    Ok(())
}
```

### Find your account ID

Your API token authenticates requests and grants access to your subaccounts.
Account-specific endpoints need an account ID to select a subaccount. Candle
requests and account listing do not require one.

Using the API key loaded above, list your account IDs with `get_accounts()`:

```rust
let mut client = OandaClient::new_with_config(None, &api_key, ClientConfig::default())?;
let accounts = client.get_accounts().await?;
for account in accounts.accounts {
    println!("{}", account.id);
}
```

This calls [`GET /v3/accounts`](https://developer.oanda.com/rest-live-v20/account-ep/)
using your bearer token. Choose the account you want to use and add its ID to
`.env`:

```dotenv
OANDA_ACCOUNT_ID=your_account_id
```

Load and select it before calling an account-specific endpoint:

```rust
let account_id = std::env::var("OANDA_ACCOUNT_ID")?;
client.set_account_id(&account_id);
let summary = client.get_account_summary().await?;
```

You can also pass `Some(&account_id)` as the first constructor argument.
Account details, summaries, instruments, configuration, and account changes all
use the selected account ID in their request URL.

## Configuration

`ClientConfig::default()` uses the practice environment, 8 concurrent requests,
100 requests per second, and up to 5 retries after the initial attempt.

```rust
use oanda_rs::client::{ClientConfig, Environment, OandaClient};

let mut client = OandaClient::new_with_config(
    None,
    &api_key,
    ClientConfig {
        environment: Environment::Practice,
        concurrency_limit: 4,
        rate_limit: 50,
        retry_attempts: 3,
        proxy: None,
    },
)?;
```

Use `Environment::Live` with a live account token to select the production API.
The environment is selected in code.

### Proxy

Set `proxy` to an HTTP/HTTPS proxy URL:

```rust
let config = ClientConfig {
    proxy: Some("http://proxy.example.com:8080".into()),
    ..ClientConfig::default()
};
let client = OandaClient::new_with_config(None, &api_key, config)?;
```

An explicit proxy overrides environment proxy settings and applies to all
requests, retries, and clones. With `proxy: None`, reqwest uses environment
proxy settings such as `HTTPS_PROXY`. Invalid proxy URLs fail during
construction. SOCKS support is not enabled.

### Rate limits and retries

Requests use [rustopus](https://github.com/nimanoori22/rustopus). All clones
share the connection pool, request rate limit, and concurrency limit.

- `rate_limit` must be between 1 and 100 requests per second, with no initial burst.
- `concurrency_limit` must be positive and includes response body reads.
- `retry_attempts` counts retries after the initial attempt; zero disables retries.
- Transient transport failures and HTTP 408, 429, 500, 502, 503, and 504 are retried
  with backoff. Numeric `Retry-After` pauses the shared arm for up to 60 seconds;
  HTTP-date values use backoff.

Connection attempts are spaced at least 500 ms apart across client instances
in the same process. Request limits are shared across clones, rather than
separately constructed clients. OANDA applies limits per IP, so other processes
using the same IP must also fit within that budget. DNS address fallback and
proxies may create additional underlying sockets.

These defaults follow OANDA's [development guide](https://developer.oanda.com/rest-live-v20/development-guide/)
and [best practices](https://developer.oanda.com/rest-live-v20/best-practices/).
PATCH requests also use the retry policy; set `retry_attempts` to zero when
replaying a write would be unsafe.

### Compatibility

The existing `OandaClient::new` and `OandaClient::new_with_environment`
constructors remain supported. Their `buffer_size` argument has no effect with
rustopus. The old Tower-specific `policies` module and `client::ClientWrapper`
have been removed. The library now uses reqwest 0.13 and requires a toolchain
supporting Rust edition 2024 for rustopus.

## Testing

Run the local HTTP regression tests without OANDA credentials:

```bash
cargo test --lib client::tests
```

With a practice API key in `.env`, check account access, an account summary,
and candle retrieval:

```bash
OANDA_ENVIRONMENT=practice cargo test --test practice practice_credentials_and_read_only_endpoints -- --exact
```

The test uses the first accessible account unless `OANDA_ACCOUNT_ID` is set.
The practice integration tests accept an optional `OANDA_PROXY` URL.

To save the latest 100 EUR/USD hourly candles to `eur_usd_h1_100.json` in the
project directory:

```bash
OANDA_ENVIRONMENT=practice cargo test --test practice save_latest_100_hourly_candles -- --exact
```

This replaces the previous export. The newest candle may still be forming;
check its `complete` field. The export is ignored by Git.

Other account and candle tests also contact OANDA and require credentials.
Streaming and automatic account snapshot synchronization are not implemented.

## Contributing

Contributions are welcome through pull requests.

## License

[MIT](LICENSE).
