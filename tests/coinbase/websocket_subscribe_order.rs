use ccrs::exchange_client::common::SubscribeOrderRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::coinbase::common::CoinbaseClient;
use ccrs::exchanges::coinbase::common::CoinbaseCredential;
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

    let api_key = get_env_as_string("COINBASE_API_KEY", "");
    let api_secret = get_env_as_string("COINBASE_API_SECRET", "");
    let api_passphrase = get_env_as_string("COINBASE_API_PASSPHRASE", "");

    let credential = CoinbaseCredential {
        api_key,
        api_secret,
        api_passphrase,
    };
    let use_sandbox = get_env_as_bool("USE_SANDBOX", false);

    let mut coinbase_client_builder = CoinbaseClient::builder();

    if use_sandbox {
        coinbase_client_builder = coinbase_client_builder
            .websocket_account_data_api_url("wss://ws-feed-public.sandbox.exchange.coinbase.com");
    }

    let coinbase_client = coinbase_client_builder.credential(Some(credential)).build();

    let mut websocket_client = match coinbase_client
        .create_websocket_client(
            WebSocketClientConfig::coinbase_account_data(),
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

    let mut subscribe_order_request = SubscribeOrderRequest::default();
    subscribe_order_request
        .symbols
        .push(get_env_as_string("SYMBOL", "BTC-USD"));

    let request = Request::SubscribeOrder(subscribe_order_request);

    let websocket_sender = websocket_client.sender();
    let _ = coinbase_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = coinbase_client
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
