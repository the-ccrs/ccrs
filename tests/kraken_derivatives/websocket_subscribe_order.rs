use ccrs::exchange_client::common::SubscribeOrderRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::kraken_derivatives::common::KrakenDerivativesClient;
use ccrs::exchanges::kraken_derivatives::common::KrakenDerivativesCredential;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("KRAKEN_DERIVATIVES_API_KEY", "");
    let api_secret = get_env_as_string("KRAKEN_DERIVATIVES_API_SECRET", "");

    let credential = KrakenDerivativesCredential {
        api_key,
        api_secret,
    };

    let websocket_api_url = get_env_as_string("KRAKEN_DERIVATIVES_WEBSOCKET_API_URL", "");

    let mut kraken_derivatives_client_builder = KrakenDerivativesClient::builder();

    if !websocket_api_url.is_empty() {
        kraken_derivatives_client_builder =
            kraken_derivatives_client_builder.websocket_api_url(websocket_api_url)
    }

    let kraken_derivatives_client = kraken_derivatives_client_builder
        .credential(Some(credential))
        .build();

    let mut websocket_client = match kraken_derivatives_client
        .create_websocket_client(
            WebSocketClientConfig::kraken_derivatives_account_data(),
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
    let _ = kraken_derivatives_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = kraken_derivatives_client
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
