use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::SubscribeTopOfBookRequest;
use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::bybit::common::BybitClient;
use ccrs::networking::websocket::WebSocketConfig;
use ccrs::types::BybitInstrumentType;
use ccrs::types::WebSocketClientConfig;
use ccrs::utils::get_env_as_number;

#[tokio::main]
async fn main() {
    let bybit_client = BybitClient::builder()
        .instrument_type(BybitInstrumentType::Spot)
        .build();

    let mut websocket_client = match bybit_client
        .create_websocket_client(
            WebSocketClientConfig::bybit_market_data(),
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
        .push("BTCUSDT".to_string());
    let request = Request::SubscribeTopOfBook(subscribe_top_of_book_request);
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
