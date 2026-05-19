use ccrs::exchange_client::common::SubscribeOrderRequest;

use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient;
use ccrs::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesCredential;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("HTX_USDT_MARGINED_FUTURES_API_KEY", "");
    let api_secret = get_env_as_string("HTX_USDT_MARGINED_FUTURES_API_SECRET", "");

    let credential = HtxUsdtMarginedFuturesCredential {
        api_key,
        api_secret,
    };

    let htx_usdt_margined_futures_client = HtxUsdtMarginedFuturesClient::builder()
        .credential(Some(credential))
        .build();

    let mut websocket_client = match htx_usdt_margined_futures_client
        .create_websocket_client(
            WebSocketClientConfig::htx_usdt_margined_futures_account_data(),
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
    let _ = htx_usdt_margined_futures_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = htx_usdt_margined_futures_client
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
