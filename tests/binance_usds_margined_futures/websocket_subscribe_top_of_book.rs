use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::SubscribeTopOfBookRequest;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::binance_usds_margined_futures::common::BinanceUsdsMarginedFuturesClient;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let binance_usds_margined_futures_client = BinanceUsdsMarginedFuturesClient::builder().build();

    let mut websocket_client = match binance_usds_margined_futures_client
        .create_websocket_client(
            WebSocketClientConfig::binance_usds_margined_futures_market_data(),
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

    let websocket_sender = websocket_client.sender();

    let mut subscribe_top_of_book_request: SubscribeTopOfBookRequest =
        SubscribeTopOfBookRequest::default();
    subscribe_top_of_book_request
        .symbols
        .push(get_env_as_string("SYMBOL", "BTCUSDT"));
    let request = Request::SubscribeTopOfBook(subscribe_top_of_book_request);
    let _ = binance_usds_margined_futures_client
        .send_websocket_request(&websocket_sender, request)
        .await;

    let _ = tokio::time::timeout(
        tokio::time::Duration::from_secs(get_env_as_number::<u64>("STOP_TIME_SECS", 10)),
        async {
            loop {
                let response = binance_usds_margined_futures_client
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
