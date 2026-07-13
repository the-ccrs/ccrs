use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::common::SubscribeOrderRequest;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::polymarket::common::PolymarketClient;
use ccrs::exchanges::polymarket::common::PolymarketCredential;
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

    let private_key = get_env_as_string("POLYMARKET_PRIVATE_KEY", "");
    let use_testnet = get_env_as_bool("USE_TESTNET", false);

    let signing_key = private_key
        .parse::<alloy::signers::local::PrivateKeySigner>()
        .expect("Invalid POLYMARKET_PRIVATE_KEY");

    let credential = PolymarketCredential::from_private_key(signing_key)
        .await
        .unwrap();

    let mut polymarket_client_builder = PolymarketClient::builder();

    if use_testnet {
        polymarket_client_builder = polymarket_client_builder.is_mainnet(false);
    }

    let polymarket_client = polymarket_client_builder
        .credential(Some(credential))
        .build();

    let mut websocket_client = match polymarket_client
        .create_websocket_client(
            WebSocketClientConfig::polymarket_account_data(),
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
    let _ = polymarket_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = polymarket_client
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
