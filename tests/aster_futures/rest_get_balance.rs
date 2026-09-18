use ccrs::exchange_client::common::GetBalanceRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::aster_futures::common::AsterFuturesClient;
use ccrs::exchanges::aster_futures::common::AsterFuturesCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let private_key = get_env_as_string("ASTER_FUTURES_PRIVATE_KEY", "");
    let signing_key = private_key
        .parse::<alloy::signers::local::PrivateKeySigner>()
        .expect("Invalid ASTER_FUTURES_PRIVATE_KEY");
    let credential = AsterFuturesCredential::new(signing_key);

    let aster_futures_client = AsterFuturesClient::builder()
        .credential(Some(credential))
        .build();

    let http_client = match aster_futures_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match aster_futures_client
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
