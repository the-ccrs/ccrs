use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::SubscribeTradeRequest;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::kalshi::common::KalshiClient;
use ccrs::exchanges::kalshi::common::KalshiCredential;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("KALSHI_API_KEY", "");
    let private_key_path = get_env_as_string("KALSHI_PRIVATE_KEY_PATH", "");

    let credential = KalshiCredential::new(api_key, private_key_path);

    let mut kalshi_client_builder = KalshiClient::builder();

    kalshi_client_builder =
        kalshi_client_builder.use_demo_trading(Some(get_env_as_bool("USE_DEMO_TRADING", false)));

    let kalshi_client = kalshi_client_builder.credential(Some(credential)).build();

    let mut websocket_client = match kalshi_client
        .create_websocket_client(
            WebSocketClientConfig::kalshi_market_data(),
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
    subscribe_trade_request
        .symbols
        .push(get_env_as_string("SYMBOL", ""));
    let request = Request::SubscribeTrade(subscribe_trade_request);
    let _ = kalshi_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = kalshi_client
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
