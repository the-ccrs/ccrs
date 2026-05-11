use ccrs::exchange_client::common::GetBalanceRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::coinbase::common::CoinbaseClient;
use ccrs::exchanges::coinbase::common::CoinbaseCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("COINBASE_API_KEY", "");
    let api_secret = get_env_as_string("COINBASE_API_SECRET", "");
    let api_passphrase = get_env_as_string("COINBASE_API_PASSPHRASE", "");

    let credential = CoinbaseCredential {
        api_key,
        api_secret,
        api_passphrase,
    };
    let use_sandbox = get_env_as_bool("USE_SANDBOX", false);

    let mut coinbase_client_builder = CoinbaseClient::builder();

    if use_sandbox {
        coinbase_client_builder = coinbase_client_builder
            .rest_api_base_url("https://api-public.sandbox.exchange.coinbase.com");
    }

    let coinbase_client = coinbase_client_builder.credential(Some(credential)).build();

    let http_client = match coinbase_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    match coinbase_client
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
