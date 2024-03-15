# oanda_rs

This is an asynchronous Rust library for interacting with the OANDA API.

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

First, create an instance of OandaClient:
```rust
let api_key = "your_api_key";
let account_id = "your_account_id";
let client = OandaClient::new(Some(&account_id), &api_key);
```
You can then use this client to make requests to the OANDA API. For example, to get candle data:
```rust
let mut query = CandleQueryBuilder::new();
query.add("count", CandlesQueryBuilder::Count(5));
query.add("granularity", CandlesQueryBuilder::Granularity(Granularity::H1));

let response = get_candles(&client, "EUR_USD", query.build()).await;
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
