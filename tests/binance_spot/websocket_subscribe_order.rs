use ccrs::exchange_client::common::SubscribeOrderRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::binance_spot::common::BinanceSpotClient;
use ccrs::exchanges::binance_spot::common::BinanceSpotCredential;
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

    let api_key = get_env_as_string("BINANCE_SPOT_API_KEY", "");
    let api_private_key_path = get_env_as_string("BINANCE_SPOT_API_PRIVATE_KEY_PATH", "");

    let credential = BinanceSpotCredential::from_pem_file(api_key, &api_private_key_path);
    let use_testnet = get_env_as_bool("USE_TESTNET", false);

    let mut binance_spot_client_builder = BinanceSpotClient::builder();

    if use_testnet {
        binance_spot_client_builder = binance_spot_client_builder
            .websocket_account_data_api_url("wss://ws-api.testnet.binance.vision/ws-api/v3");
    }

    let binance_spot_client = binance_spot_client_builder
        .credential(Some(credential))
        .build();

    let mut websocket_client = match binance_spot_client
        .create_websocket_client(
            WebSocketClientConfig::binance_spot_account_data(),
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

    let subscribe_order_request = SubscribeOrderRequest::default();

    let request = Request::SubscribeOrder(subscribe_order_request);

    let websocket_sender = websocket_client.sender();
    let _ = binance_spot_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = binance_spot_client
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
