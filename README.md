# ccrs
* A Rust library for trading on crypto exchanges. Super simple to use.
* Unified API for different exchanges. Supported exchanges:
  * [Binance](https://accounts.maxweb.black/register?ref=1116718520)
  * [Bybit](https://www.bybit.com/invite?ref=XNYP2K).
  * [Coinbase](https://advanced.coinbase.com/join/CKGCX6U).
  * [Gate](https://www.gate.com/signup/VLUQXVFWAW?ref_type=103).
  * [HTX](https://www.htx.com/invite/en-us/1f?invite_code=rmw7d223).
  * [OKX](https://www.okx.com/join/47636709).
  * Many more coming soon.
* For any questions, contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5.
* We provide end-to-end development services for crypto trading infrastructures, covering everything from integration and strategy implementation to real-time data handling and execution.

## Installation
Add this to your `Cargo.toml`
```toml
[dependencies]
ccrs = { git = "https://github.com/the-ccrs/ccrs.git" }
```
The default main branch may include experimental or in-progress features. For a stable experience, please use specific tags.
```toml
[dependencies]
ccrs = { git = "https://github.com/the-ccrs/ccrs.git", tag = "v1.2.3" }
```

## Documentation
Our documentation is centered around comprehensive, real-world examples that demonstrate how to use each feature in practice.

* [REST market data](examples/rest_market_data.rs) — `cargo run --example rest_market_data`
* [REST order execution](examples/rest_order_execution.rs) — `env USE_TESTNET=true BYBIT_API_KEY='...' BYBIT_API_SECRET='...' SYMBOL=BTCUSDT PRICE=50000 QUANTITY=0.001 cargo run --example rest_order_execution`
* [REST get position](examples/rest_get_position.rs) — `env USE_TESTNET=true BYBIT_API_KEY='...' BYBIT_API_SECRET='...' cargo run --example rest_get_position`
* [REST get balance](examples/rest_get_balance.rs) — `env USE_TESTNET=true BYBIT_API_KEY='...' BYBIT_API_SECRET='...' cargo run --example rest_get_balance`
* [WebSocket subscribe top of book](examples/websocket_subscribe_top_of_book.rs) — `cargo run --example websocket_subscribe_top_of_book`
* [WebSocket subscribe trade](examples/websocket_subscribe_trade.rs) — `cargo run --example websocket_subscribe_trade`
* [WebSocket subscribe order](examples/websocket_subscribe_order.rs) — `env USE_TESTNET=true BYBIT_API_KEY='...' BYBIT_API_SECRET='...' cargo run --example websocket_subscribe_order`
* [WebSocket subscribe fill](examples/websocket_subscribe_fill.rs) — `env USE_TESTNET=true BYBIT_API_KEY='...' BYBIT_API_SECRET='...' cargo run --example websocket_subscribe_fill`
* [Websocket read batch](examples/websocket_read_batch.rs) — `cargo run --example websocket_read_batch`
* [Enable library logging](examples/enable_library_logging.rs) — `cargo run --example enable_library_logging --features max_log_level_finest`
* [Connect to proxy](examples/connect_to_proxy.rs) — `cargo run --example connect_to_proxy`

More exchange specific examples can be found in the [`tests`](tests) directory.

## Philosophy
> "Everything should be made as simple as possible, but not simpler."
> — *Albert Einstein*

## Source Code Visibility
* REST API source code is fully available.
* WebSocket API source code is partially available. Full access can be obtained separately — please contact us at Telegram https://t.me/+NvPBKXi6kFNkYmE5.
