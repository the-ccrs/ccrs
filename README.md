# ccrs
* A Rust library for trading on crypto exchanges. Super simple to use.
* Unified API for different exchanges. Supported exchanges:
  * [Bybit](https://www.bybit.com/invite?ref=XNYP2K).
  * [Okx](https://www.okx.com/join/47636709).
  * Many more coming soon.
* For any questions, contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5.
* We provide end-to-end development services for crypto trading infrastructures, covering everything from integration and strategy implementation to real-time data handling and execution.

## Installation
Add this to your `Cargo.toml`
```toml
[dependencies]
ccrs = { git = "https://github.com/the-crypto-connect/ccrs.git" }
```
The default main branch may include experimental or in-progress features. For a stable experience, please use specific tags.
```toml
[dependencies]
ccrs = { git = "https://github.com/the-crypto-connect/ccrs.git", tag = "v1.2.3" }
```

## Documentation
Our documentation is centered around comprehensive, real-world examples that demonstrate how to use each feature in practice.
* [REST market data](examples/rest_market_data.rs)
* [REST order execution](examples/rest_order_execution.rs)
* [REST get position](examples/rest_get_position.rs)
* [REST get balance](examples/rest_get_balance.rs)
* [WebSocket subscribe top of book](examples/websocket_subscribe_top_of_book.rs)
* [WebSocket subscribe trade](examples/websocket_subscribe_trade.rs)
* [WebSocket subscribe order](examples/websocket_subscribe_order.rs)
* [WebSocket subscribe fill](examples/websocket_subscribe_fill.rs)
* [Enable library logging](examples/enable_library_logging.rs)
* [Connect to proxy](examples/connect_to_proxy.rs)

## Philosophy
> "Everything should be made as simple as possible, but not simpler."
> — *Albert Einstein*

## Source Code Visibility
* REST API source code is fully available.
* WebSocket API source code is partially available. Full access can be obtained separately—please contact us at Telegram https://t.me/+NvPBKXi6kFNkYmE5.
