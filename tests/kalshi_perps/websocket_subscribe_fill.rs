use ccrs::exchange_client::common::SubscribeFillRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::kalshi_perps::common::KalshiPerpsClient;
use ccrs::exchanges::kalshi_perps::common::KalshiPerpsCredential;
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

    let api_key = get_env_as_string("KALSHI_PERPS_API_KEY", "");
    let private_key_path = get_env_as_string("KALSHI_PERPS_PRIVATE_KEY_PATH", "");

    let credential = KalshiPerpsCredential::new(api_key, private_key_path);

    let mut kalshi_perps_client_builder = KalshiPerpsClient::builder();

    kalshi_perps_client_builder = kalshi_perps_client_builder
        .use_demo_trading(Some(get_env_as_bool("USE_DEMO_TRADING", false)));

    let kalshi_perps_client = kalshi_perps_client_builder
        .credential(Some(credential))
        .build();

    let mut websocket_client = match kalshi_perps_client
        .create_websocket_client(
            WebSocketClientConfig::kalshi_perps_account_data(),
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

    let subscribe_fill_request = SubscribeFillRequest::default();

    let request = Request::SubscribeFill(subscribe_fill_request);

    let websocket_sender = websocket_client.sender();
    let _ = kalshi_perps_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = kalshi_perps_client
                    .read_next_websocket_message(&mut websocket_client)
                    .await;

                println!("{:#?}", response);

                if let Response::WebSocketReadError(_) = response {
                    break;
                }
            }
        },
    )
    .await;

    websocket_client.close().await;

    println!("Done!");
}
