use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::SubscribeTradeRequest;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::hyperliquid::common::HyperliquidClient;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::HyperliquidInstrumentType;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let hyperliquid_client = HyperliquidClient::builder()
        .instrument_type(HyperliquidInstrumentType::Perpetuals)
        .build();

    let mut websocket_client = match hyperliquid_client
        .create_websocket_client(
            WebSocketClientConfig::hyperliquid_market_data(),
            WebSocketConfig::default(),
        )
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create WebSocket client: {:#?}", err);
            return;
        }
    };

    let websocket_sender = websocket_client.sender();

    let mut subscribe_trade_request = SubscribeTradeRequest::default();

    let symbol_env = get_env_as_string("SYMBOL", "BTC");
    let symbol = match symbol_env.as_str() {
        "BTC" => "BTC".to_string(),
        "BTC/USDC" => "@142".to_string(),
        _ => panic!(),
    };

    subscribe_trade_request.symbols.push(symbol);
    let request = Request::SubscribeTrade(subscribe_trade_request);
    let _ = hyperliquid_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = hyperliquid_client
                    .read_next_websocket_message(&mut websocket_client)
                    .await;

                println!("{:#?}", response);
            }
        },
    )
    .await;

    websocket_client.close().await;

    println!("Done!");
}
