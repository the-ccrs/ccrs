use ccrs::exchange_client::websocket::Websocket;
use ccrs::exchanges::binance_usds_margined_futures::common::BinanceUsdsMarginedFuturesClient;
use ccrs::exchanges::binance_usds_margined_futures::common::BinanceUsdsMarginedFuturesCredential;
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

    let api_key = get_env_as_string("BINANCE_USDS_MARGINED_FUTURES_API_KEY", "");
    let api_private_key_path =
        get_env_as_string("BINANCE_USDS_MARGINED_FUTURES_API_PRIVATE_KEY_PATH", "");

    let credential =
        BinanceUsdsMarginedFuturesCredential::from_pem_file(api_key, &api_private_key_path);
    let use_demo_trading = get_env_as_bool("USE_DEMO_TRADING", false);

    let mut binance_usds_margined_futures_client_builder =
        BinanceUsdsMarginedFuturesClient::builder();

    if use_demo_trading {
        binance_usds_margined_futures_client_builder = binance_usds_margined_futures_client_builder
            .websocket_account_data_api_url(
                "wss://fstream.binancefuture.com/private/ws/{listen_key}",
            )
            .rest_api_base_url("https://demo-fapi.binance.com");
    }

    let binance_usds_margined_futures_client = binance_usds_margined_futures_client_builder
        .credential(Some(credential))
        .build();

    let mut websocket_client = match binance_usds_margined_futures_client
        .create_websocket_client(
            WebSocketClientConfig::binance_usds_margined_futures_account_data(),
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
