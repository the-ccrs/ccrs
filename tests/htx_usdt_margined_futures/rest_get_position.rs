use ccrs::exchange_client::common::GetPositionRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient;
use ccrs::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("HTX_USDT_MARGINED_FUTURES_API_KEY", "");
    let api_secret = get_env_as_string("HTX_USDT_MARGINED_FUTURES_API_SECRET", "");

    let credential = HtxUsdtMarginedFuturesCredential {
        api_key,
        api_secret,
    };

    let htx_usdt_margined_futures_client = HtxUsdtMarginedFuturesClient::builder()
        .credential(Some(credential))
        .build();

    let http_client = match htx_usdt_margined_futures_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match htx_usdt_margined_futures_client
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
