use ccrs::exchange_client::common::GetBalanceRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::kraken_derivatives::common::KrakenDerivativesClient;
use ccrs::exchanges::kraken_derivatives::common::KrakenDerivativesCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("KRAKEN_DERIVATIVES_API_KEY", "");
    let api_secret = get_env_as_string("KRAKEN_DERIVATIVES_API_SECRET", "");

    let credential = KrakenDerivativesCredential {
        api_key,
        api_secret,
    };

    let rest_api_base_url = get_env_as_string("KRAKEN_DERIVATIVES_REST_API_BASE_URL", "");

    let mut kraken_derivatives_client_builder = KrakenDerivativesClient::builder();

    if !rest_api_base_url.is_empty() {
        kraken_derivatives_client_builder =
            kraken_derivatives_client_builder.rest_api_base_url(rest_api_base_url)
    }

    let kraken_derivatives_client = kraken_derivatives_client_builder
        .credential(Some(credential))
        .build();

    let http_client = match kraken_derivatives_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match kraken_derivatives_client
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
        Response::HttpErrorResponse(http_resp) => {
            println!("HTTP error, status: {}", http_resp.status);
            println!("Headers: {:#?}", http_resp.headers);
            println!("Body: {:#?}", http_resp.body);
        }
        _ => unreachable!(),
    }
}
