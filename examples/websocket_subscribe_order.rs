use ccrs::exchange_client::common::SubscribeOrderRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::bybit::common::BybitClient;
use ccrs::exchanges::bybit::common::BybitCredential;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::BybitInstrumentType;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;

#[tokio::main]
async fn main() {
    let api_key = get_env_as_string("BYBIT_API_KEY", "");
    let api_secret = get_env_as_string("BYBIT_API_SECRET", "");

    if api_key.is_empty() || api_secret.is_empty() {
        panic!("BYBIT_API_KEY and BYBIT_API_SECRET must be set");
    }

    let credential = BybitCredential {
        api_key,
        api_secret,
    };
    let use_testnet = get_env_as_bool("USE_TESTNET", false);

    let mut bybit_client_builder = BybitClient::builder();

    if use_testnet {
        bybit_client_builder = bybit_client_builder
            .websocket_account_data_api_url("wss://stream-testnet.bybit.com/v5/private");
    }

    let bybit_client = bybit_client_builder
        .instrument_type(BybitInstrumentType::Spot)
        .credential(Some(credential))
        .build();

    let mut websocket_client = match bybit_client
        .create_websocket_client(
            WebSocketClientConfig::bybit_account_data(),
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
    let _ = bybit_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = bybit_client
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
