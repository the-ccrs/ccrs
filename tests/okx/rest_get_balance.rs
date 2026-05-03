use ccrs::exchange_client::common::GetBalanceRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::okx::common::OkxClient;
use ccrs::exchanges::okx::common::OkxCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::types::OkxInstrumentType;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("OKX_API_KEY", "");
    let api_secret = get_env_as_string("OKX_API_SECRET", "");
    let passphrase = get_env_as_string("OKX_PASSPHRASE", "");

    let credential = OkxCredential {
        api_key,
        api_secret,
        passphrase,
    };

    let mut okx_client_builder = OkxClient::builder();

    okx_client_builder =
        okx_client_builder.use_demo_trading(Some(get_env_as_bool("USE_DEMO_TRADING", false)));

    let okx_client = okx_client_builder
        .instrument_type(OkxInstrumentType::Spot)
        .credential(Some(credential))
        .build();

    let http_client = match okx_client.create_http_client(HttpConfig::default()).await {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match okx_client
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
