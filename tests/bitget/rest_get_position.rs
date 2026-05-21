use ccrs::exchange_client::common::GetPositionRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::bitget::common::BitgetClient;
use ccrs::exchanges::bitget::common::BitgetCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::types::BitgetInstrumentType;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("BITGET_API_KEY", "");
    let api_secret = get_env_as_string("BITGET_API_SECRET", "");
    let passphrase = get_env_as_string("BITGET_API_PASSPHRASE", "");

    let credential = BitgetCredential {
        api_key,
        api_secret,
        passphrase,
    };

    let bitget_client = BitgetClient::builder()
        .use_demo_trading(Some(get_env_as_bool("USE_DEMO_TRADING", false)))
        .instrument_type(BitgetInstrumentType::UsdtFutures)
        .credential(Some(credential))
        .build();

    let http_client = match bitget_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match bitget_client
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
