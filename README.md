# oanda_rs

This is an asynchronous Rust library for interacting with the OANDA API.


## Features

- Retrieve account details, instruments, and historical candle data.
- Rate limiting and retry capabilities.
- Asynchronous support using `tokio`.


## Getting Started

1. Clone the repository:

```bash
git clone https://github.com/nimanoori22/oanda_rs.git
cd oanda_rs
```

2. Install the dependencies:
```bash
cargo build
```

## Usage

**Initialization**

Store your practice credentials in `~/.config/oanda/credentials` (directory
permissions `700`, file permissions `600`):

```dotenv
OANDA_API_TOKEN=your_api_token
OANDA_ACCOUNT_ID=your_practice_account_id
OANDA_ENVIRONMENT=practice
```

Link the project dotenv file to that external file:

```bash
ln -s ~/.config/oanda/credentials .env
```

`.env` is ignored by Git. `dotenv::dotenv()` follows this symlink, so credentials
stay outside the repository. Run commands from the project directory. Existing
exported environment variables take precedence over dotenv values.

Run the read-only practice smoke test:

```bash
cargo test --test practice
```

It checks authentication/account listing, an account summary, and five hourly
EUR/USD candles without printing the token or account responses. If
`OANDA_ACCOUNT_ID` is empty, absent, or still `your_demo_account_id`, it uses the
first accessible practice account for the test without changing the credentials
file. A configured account must belong to the token.

The existing endpoint tests use the practice environment. For live applications,
select `Environment::Live` explicitly; `OANDA_ENVIRONMENT` does not change the
constructor's environment automatically.

Example

Here is an example of how to use the library to get historical candle data:

```rust
use oanda_rs::client::OandaClient;
use oanda_rs::instrument::candles::{CandleQuery, CandleQueryParam, Granularity};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().expect("Could not load the project .env file");
    let api_key = std::env::var("OANDA_API_TOKEN").expect("OANDA_API_TOKEN must be set");
    let account_id = std::env::var("OANDA_ACCOUNT_ID").expect("OANDA_ACCOUNT_ID must be set");

    let client_result = OandaClient::new(Some(&account_id), &api_key, 100, 100, 100, 5);
    let mut client = match client_result {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
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
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
```

### Configure a proxy in code

```rust
use oanda_rs::client::{ClientConfig, OandaClient};

let mut client = OandaClient::new_with_config(
    Some(&account_id),
    &api_key,
    ClientConfig {
        proxy: Some("http://127.0.0.1:10808".into()),
        ..ClientConfig::default()
    },
)?;
```

`ClientConfig` defaults to practice, concurrency 8, 100 requests/second, and
5 retries. Its `environment`, `concurrency_limit`, `rate_limit`, and
`retry_attempts` fields can also be set explicitly. Existing constructors still
work. An explicit HTTP/HTTPS proxy overrides environment proxy settings and
applies to GET, PATCH, retries, and all clones. With `proxy: None`, reqwest
continues to discover proxies from environment variables such as `HTTPS_PROXY`.
Invalid proxy URLs fail during construction. SOCKS support is not enabled.

The practice smoke test accepts `OANDA_PROXY` to exercise this configuration:

```bash
OANDA_PROXY=http://127.0.0.1:10808 cargo test --test practice
```

### Environments, connections, rate limiting, and retries

Requests run through [rustopus](https://github.com/nimanoori22/rustopus), pinned to
revision `3d2b8120343e58b3ed7c92052c54700a164afa51`. The default environment remains
fxTrade Practice. Select fxTrade explicitly for a live account:

```rust
use oanda_rs::client::{Environment, OandaClient};

let client = OandaClient::new_with_environment(
    Some(&account_id), &api_key, 100, 8, 100, 5, Environment::Live,
)?;
```

The existing `OandaClient::new(account_id, api_key, buffer_size,
concurrency_limit, rate_limit, retry_attempts)` call remains supported:

- `buffer_size` is retained for source compatibility and has no effect. Rustopus
  waits in the caller's future without a background request queue.
- `concurrency_limit` must be positive and bounds in-flight requests, including
  response body reads.
- `rate_limit` must be between 1 and 100 requests/second. Requests are paced
  without an initial burst; every retry consumes a rate token.
- `retry_attempts` counts retries **after** the initial attempt; zero disables
  retries. Rustopus retries transient transport failures and HTTP 408, 429, 500,
  502, 503, and 504 with exponential backoff and jitter. Numeric `Retry-After`
  pauses the shared arm (capped at 60 seconds); HTTP-date values use backoff.

One persistent HTTP/1.1 client and one rustopus arm are shared by all clones.
Bodies are fully read before slots are released, allowing connection reuse.
Connection attempts are spaced at least 500 ms apart across all OandaClient
instances in this process. The two small `tower-layer`/`tower-service` trait
crates attach that connection hook to reqwest; Tower's buffer, limit, and retry
middleware has been removed. Reqwest itself still depends on Tower internally.
Automatic redirects and reqwest's additional retry policy are disabled so
rustopus controls request attempts.

These settings follow OANDA's [development guide](https://developer.oanda.com/rest-live-v20/development-guide/)
and [best practices](https://developer.oanda.com/rest-live-v20/best-practices/):
100 requests/second is the recommended persistent-connection rate, below the
120 requests/second REST ceiling. Request limits are shared across **clones**,
not separately constructed clients. OANDA applies limits per IP, so other
clients and processes using that IP must also fit within the budget. The
connection hook limits connector attempts; DNS address fallback or proxies may
create additional underlying sockets.

HTTP failures are returned even when the body has no `errorMessage`. OANDA's
error message is retained on terminal failures and exhausted retries. JSON
request bodies use `Content-Type: application/json` and requests use bearer
authentication. PATCH requests also use the configured retry policy; set
`retry_attempts` to zero when replaying a write would be unsafe.

Migration notes: the old public Tower-specific `policies` module and
`client::ClientWrapper` have been removed. `reqwest` is upgraded to 0.13 to match
rustopus, which also requires a Rust toolchain supporting edition 2024. Endpoint
methods and the existing constructor retain their signatures.

This migration covers REST requests. Streaming (including its 20-stream cap)
and automatic account snapshot synchronization are not implemented. For account
updates, use Account Details first, retain its `lastTransactionID`, then poll
Account Changes with the most recently returned transaction ID, as described
in OANDA's best practices.

### Testing

Run the local HTTP regression suite without OANDA credentials:

```bash
cargo test --lib client::tests
cargo test --all-targets --no-run
```

The existing account and candle tests contact OANDA and require credentials.
Run those explicitly against a practice account when needed. The local suite
uses dummy tokens and checks retries, terminal errors, headers, shared pacing,
connection reuse, and connection attempt spacing.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Export the latest 100 hourly candles

```bash
OANDA_PROXY=http://127.0.0.1:10808 cargo test --test practice save_latest_100_hourly_candles -- --exact --nocapture
```

This read-only practice test saves EUR/USD candles to `eur_usd_h1_100.json` in
the project directory, replacing the previous export. The file includes the
instrument, granularity, and 100 candles in chronological order. The newest
candle may still be forming; check its `complete` field. The export is ignored
by Git.
