use ccrs::exchange_client::common::SubscribeOrderRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::gateio_spot_and_margin::common::GateioSpotAndMarginClient;
use ccrs::exchanges::gateio_spot_and_margin::common::GateioSpotAndMarginCredential;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::GateioSpotAndMarginInstrumentType;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("GATEIO_SPOT_AND_MARGIN_API_KEY", "");
    let api_secret = get_env_as_string("GATEIO_SPOT_AND_MARGIN_API_SECRET", "");

    let credential = GateioSpotAndMarginCredential {
        api_key,
        api_secret,
    };
    let use_testnet = get_env_as_bool("USE_TESTNET", false);

    let mut gateio_client_builder = GateioSpotAndMarginClient::builder();

    if use_testnet {
        gateio_client_builder = gateio_client_builder
            .websocket_account_data_api_url("wss://ws-testnet.gate.com/v4/ws/spot");
    }

    let gateio_client = gateio_client_builder
        .instrument_type(GateioSpotAndMarginInstrumentType::Spot)
        .credential(Some(credential))
        .build();

    let mut websocket_client = match gateio_client
        .create_websocket_client(
            WebSocketClientConfig::gateio_spot_and_margin_account_data(),
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
    let _ = gateio_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = gateio_client
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
