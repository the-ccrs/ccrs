use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::aster_spot::common::AsterSpotClient;
use ccrs::exchanges::aster_spot::common::AsterSpotCredential;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let private_key = get_env_as_string("ASTER_SPOT_PRIVATE_KEY", "");
    let signing_key = private_key
        .parse::<alloy::signers::local::PrivateKeySigner>()
        .expect("Invalid ASTER_SPOT_PRIVATE_KEY");
    let credential = AsterSpotCredential::new(signing_key);

    let aster_spot_client = AsterSpotClient::builder()
        .credential(Some(credential))
        .build();

    let mut websocket_client = match aster_spot_client
        .create_websocket_client(
            WebSocketClientConfig::aster_spot_account_data(),
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

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = aster_spot_client
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
