use ccrs::exchange_client::common::SubscribeOrderRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::okx::common::OkxClient;
use ccrs::exchanges::okx::common::OkxCredential;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::OkxInstrumentType;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("OKX_API_KEY", "");
    let api_secret = get_env_as_string("OKX_API_SECRET", "");
    let passphrase = get_env_as_string("OKX_PASSPHRASE", "");

    let credential = OkxCredential {
        api_key,
        api_secret,
        passphrase,
    };
    let use_demo_trading = get_env_as_bool("USE_DEMO_TRADING", false);

    let mut okx_client_builder = OkxClient::builder();

    if use_demo_trading {
        okx_client_builder = okx_client_builder
            .websocket_account_data_api_url("wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999");
    }

    let okx_client = okx_client_builder
        .instrument_type(OkxInstrumentType::Spot)
        .credential(Some(credential))
        .build();

    let mut websocket_client = match okx_client
        .create_websocket_client(
            WebSocketClientConfig::okx_account_data(),
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
    let _ = okx_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = okx_client
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
