use ccrs::exchange_client::common::GetPositionRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::bitstamp::common::BitstampClient;
use ccrs::exchanges::bitstamp::common::BitstampCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("BITSTAMP_API_KEY", "");
    let api_secret = get_env_as_string("BITSTAMP_API_SECRET", "");

    let credential = BitstampCredential {
        api_key,
        api_secret,
    };

    let use_sandbox = get_env_as_bool("USE_SANDBOX", false);

    let mut bitstamp_client_builder = BitstampClient::builder();

    if use_sandbox {
        bitstamp_client_builder =
            bitstamp_client_builder.rest_api_base_url("https://sandbox.bitstamp.net");
    }

    let bitstamp_client = bitstamp_client_builder.credential(Some(credential)).build();

    let http_client = match bitstamp_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match bitstamp_client
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
