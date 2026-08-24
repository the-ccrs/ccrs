use ccrs::exchange_client::common::GetPositionRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::kalshi_perps::common::KalshiPerpsClient;
use ccrs::exchanges::kalshi_perps::common::KalshiPerpsCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("KALSHI_PERPS_API_KEY", "");
    let private_key_path = get_env_as_string("KALSHI_PERPS_PRIVATE_KEY_PATH", "");

    let credential = KalshiPerpsCredential::new(api_key, private_key_path);

    let mut kalshi_perps_client_builder = KalshiPerpsClient::builder();

    kalshi_perps_client_builder = kalshi_perps_client_builder
        .use_demo_trading(Some(get_env_as_bool("USE_DEMO_TRADING", false)));

    let kalshi_perps_client = kalshi_perps_client_builder
        .credential(Some(credential))
        .build();

    let http_client = match kalshi_perps_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match kalshi_perps_client
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
