use ccrs::exchange_client::common::SubscribeOrderRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::cryptocom::common::CryptocomClient;
use ccrs::exchanges::cryptocom::common::CryptocomCredential;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("CRYPTOCOM_API_KEY", "");
    let api_secret = get_env_as_string("CRYPTOCOM_API_SECRET", "");

    let credential = CryptocomCredential {
        api_key,
        api_secret,
    };

    let websocket_account_data_api_url =
        get_env_as_string("CRYPTOCOM_WEBSOCKET_ACCOUNT_DATA_API_URL", "");

    let mut cryptocom_client_builder = CryptocomClient::builder();

    if !websocket_account_data_api_url.is_empty() {
        cryptocom_client_builder =
            cryptocom_client_builder.websocket_account_data_api_url(websocket_account_data_api_url)
    }

    let cryptocom_client = cryptocom_client_builder
        .credential(Some(credential))
        .build();

    let mut websocket_client = match cryptocom_client
        .create_websocket_client(
            WebSocketClientConfig::cryptocom_account_data(),
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
    let _ = cryptocom_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = cryptocom_client
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
