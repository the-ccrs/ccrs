use ccrs::exchange_client::common::GetPositionRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::polymarket::common::PolymarketClient;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let use_testnet = get_env_as_bool("USE_TESTNET", false);
    let funder_address = get_env_as_string("POLYMARKET_FUNDER_ADDRESS", "");

    let mut polymarket_client_builder = PolymarketClient::builder();

    if use_testnet {
        polymarket_client_builder = polymarket_client_builder.is_mainnet(false);
    }

    let polymarket_client = polymarket_client_builder
        .funder_address(funder_address)
        .build();

    let http_client = match polymarket_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match polymarket_client
        .send_http_request(
            &http_client,
            Request::GetPosition(GetPositionRequest {
                ..Default::default()
            }),
        )
        .await
    {
        Response::GetPosition(data) => {
            println!("Got position: {:#?}", data);
        }
        Response::HttpErrorResponse(http_resp) => {
            println!("HTTP error, status: {}", http_resp.status);
            println!("Headers: {:#?}", http_resp.headers);
            println!("Body: {:#?}", http_resp.body);
        }
        _ => unreachable!(),
    }
}
