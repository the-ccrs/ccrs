use ccrs::exchange_client::common::GetBalanceRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::binance_spot::common::BinanceSpotClient;
use ccrs::exchanges::binance_spot::common::BinanceSpotCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("BINANCE_SPOT_API_KEY", "");
    let api_private_key_path = get_env_as_string("BINANCE_SPOT_API_PRIVATE_KEY_PATH", "");

    let credential = BinanceSpotCredential::from_pem_file(api_key, &api_private_key_path);
    let use_testnet = get_env_as_bool("USE_TESTNET", false);

    let mut binance_spot_client_builder = BinanceSpotClient::builder();

    if use_testnet {
        binance_spot_client_builder =
            binance_spot_client_builder.rest_api_base_url("https://testnet.binance.vision");
    }

    let binance_spot_client = binance_spot_client_builder
        .credential(Some(credential))
        .build();

    let http_client = match binance_spot_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match binance_spot_client
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
