use ccrs::exchange_client::common::SubscribeOrderRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::bitget::common::BitgetClient;
use ccrs::exchanges::bitget::common::BitgetCredential;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::BitgetInstrumentType;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("BITGET_API_KEY", "");
    let api_secret = get_env_as_string("BITGET_API_SECRET", "");
    let passphrase = get_env_as_string("BITGET_API_PASSPHRASE", "");

    let credential = BitgetCredential {
        api_key,
        api_secret,
        passphrase,
    };

    let use_demo_trading = get_env_as_bool("USE_DEMO_TRADING", false);

    let mut bitget_client_builder = BitgetClient::builder();

    if use_demo_trading {
        bitget_client_builder = bitget_client_builder
            .websocket_account_data_api_url("wss://wspap.bitget.com/v3/ws/private");
    }

    let bitget_client = bitget_client_builder
        .instrument_type(BitgetInstrumentType::Spot)
        .credential(Some(credential))
        .build();

    let mut websocket_client = match bitget_client
        .create_websocket_client(
            WebSocketClientConfig::bitget_account_data(),
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
    let _ = bitget_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = bitget_client
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
