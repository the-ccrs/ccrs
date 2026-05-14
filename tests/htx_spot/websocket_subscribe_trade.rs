use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::SubscribeTradeRequest;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::htx_spot::common::HtxSpotClient;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let htx_spot_client = HtxSpotClient::builder().build();

    let mut websocket_client = match htx_spot_client
        .create_websocket_client(
            WebSocketClientConfig::htx_spot_market_data(),
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
        .push(get_env_as_string("SYMBOL", "btcusdt"));
    let request = Request::SubscribeTrade(subscribe_trade_request);
    let _ = htx_spot_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = htx_spot_client
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
