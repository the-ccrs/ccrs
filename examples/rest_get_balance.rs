use ccrs::exchange_client::common::GetBalanceRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::bybit::common::BybitClient;
use ccrs::exchanges::bybit::common::BybitCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::types::BybitInstrumentType;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;

#[tokio::main]
async fn main() {
    let api_key = get_env_as_string("BYBIT_API_KEY", "");
    let api_secret = get_env_as_string("BYBIT_API_SECRET", "");

    if api_key.is_empty() || api_secret.is_empty() {
        panic!("BYBIT_API_KEY and BYBIT_API_SECRET must be set");
    }

    let credential = BybitCredential {
        api_key,
        api_secret,
    };
    let use_testnet = get_env_as_bool("USE_TESTNET", false);

    let mut bybit_client_builder = BybitClient::builder();

    if use_testnet {
        bybit_client_builder =
            bybit_client_builder.rest_api_base_url("https://api-testnet.bybit.com");
    }

    let bybit_client = bybit_client_builder
        .instrument_type(BybitInstrumentType::Spot)
        .credential(Some(credential))
        .build();

    let http_client = match bybit_client.create_http_client(HttpConfig::default()).await {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match bybit_client
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
