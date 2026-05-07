use ccrs::exchange_client::common::GetBalanceRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::gateio_perpetual_futures::common::GateioPerpetualFuturesClient;
use ccrs::exchanges::gateio_perpetual_futures::common::GateioPerpetualFuturesCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("GATEIO_PERPETUAL_FUTURES_API_KEY", "");
    let api_secret = get_env_as_string("GATEIO_PERPETUAL_FUTURES_API_SECRET", "");

    let credential = GateioPerpetualFuturesCredential {
        api_key,
        api_secret,
    };

    let mut gateio_perpetual_futures_client_builder = GateioPerpetualFuturesClient::builder();
    if get_env_as_bool("USE_TESTNET", false) {
        gateio_perpetual_futures_client_builder = gateio_perpetual_futures_client_builder
            .rest_api_base_url("https://api-testnet.gateapi.io");
    }

    let gateio_client = gateio_perpetual_futures_client_builder
        .credential(Some(credential))
        .build();

    let http_client = match gateio_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match gateio_client
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
