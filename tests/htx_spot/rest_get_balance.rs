use ccrs::exchange_client::common::GetBalanceRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::htx_spot::common::HtxSpotClient;
use ccrs::exchanges::htx_spot::common::HtxSpotCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("HTX_SPOT_API_KEY", "");
    let api_secret = get_env_as_string("HTX_SPOT_API_SECRET", "");

    let credential = HtxSpotCredential {
        api_key,
        api_secret,
    };

    let account_id = get_env_as_string("HTX_SPOT_ACCOUNT_ID", "");

    let htx_spot_client = HtxSpotClient::builder()
        .credential(Some(credential.clone()))
        .account_id(account_id)
        .build();

    let http_client = match htx_spot_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match htx_spot_client
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
            return;
        }
        _ => unreachable!(),
    }
}
