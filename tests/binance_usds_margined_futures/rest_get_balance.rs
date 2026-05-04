use ccrs::exchange_client::common::GetBalanceRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::binance_usds_margined_futures::common::BinanceUsdsMarginedFuturesClient;
use ccrs::exchanges::binance_usds_margined_futures::common::BinanceUsdsMarginedFuturesCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_bool;
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
            .rest_api_base_url("https://demo-fapi.binance.com");
    }

    let binance_usds_margined_futures_client = binance_usds_margined_futures_client_builder
        .credential(Some(credential))
        .build();

    let http_client = match binance_usds_margined_futures_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match binance_usds_margined_futures_client
        .send_http_request(
            &http_client,
            Request::GetBalance(GetBalanceRequest {
                ..Default::default()
            }),
        )
        .await
    {
        Response::GetBalance(data) => {
            println!("Got balance: {:#?}", data);
        }
        Response::HttpErrorResponse(http_response) => {
            println!("HTTP response: {:#?}", http_response);
        }
        _ => unreachable!(),
    }
}
