use ccrs::exchange_client::common::SubscribeFillRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::htx_spot::common::HtxSpotClient;
use ccrs::exchanges::htx_spot::common::HtxSpotCredential;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("HTX_SPOT_API_KEY", "");
    let api_secret = get_env_as_string("HTX_SPOT_API_SECRET", "");

    let credential = HtxSpotCredential {
        api_key,
        api_secret,
    };

    let account_id = get_env_as_string("HTX_SPOT_ACCOUNT_ID", "");

    let htx_spot_client = HtxSpotClient::builder()
        .credential(Some(credential))
        .account_id(account_id)
        .build();

    let mut websocket_client = match htx_spot_client
        .create_websocket_client(
            WebSocketClientConfig::htx_spot_account_data(),
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
    let _ = htx_spot_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = htx_spot_client
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
