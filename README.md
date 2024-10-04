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

First, set up your environment variables in a .env file:

```
OANDA_API_KEY=your_api_key
```

Get your OANDA_ACCOUNT_ID by running this command:

```bash
cargo test -- --nocapture test_get_accounts
```

Update your .env file:
```
OANDA_API_KEY=your_api_key
OANDA_ACCOUNT_ID=your_account_id
```

Example

Here is an example of how to use the library to get historical candle data:

```rust
use oanda_rs::client::OandaClient;
use oanda_rs::instrument::candles::{CandleQuery, Granularity};
use std::collections::HashMap;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let api_key = std::env::var("OANDA_API_KEY").expect("OANDA_API_KEY must be set");
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
    query.add("count", CandleQueryParam::Count(5));
    query.add("granularity", CandleQueryParam::Granularity(Granularity::H1));

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

### Rate Limiting and Retry

This package includes built-in rate limiting and retry capabilities. You can configure the rate limits and retry settings when initializing the OandaClient:

```rust
let mut client = OandaClient::new(Some(&account_id), &api_key, 100, 100, 100, 5).unwrap();

```
The parameters are:

**`buffer_size`**: The size of the buffer for the requests.

**`concurrency_limit`**: The maximum number of concurrent requests.

**`rate_limit`**: The maximum number of requests per second.

**`retry_attempts`**: The number of retry attempts for failed requests.




## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
