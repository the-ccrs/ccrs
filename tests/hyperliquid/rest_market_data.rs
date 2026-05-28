use ccrs::exchange_client::ExchangeClient;
use ccrs::exchange_client::common::GetInstrumentInfoRequest;
use ccrs::exchange_client::common::GetTopOfBookRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::hyperliquid::common::HyperliquidClient;
use ccrs::networking::http::HttpClient;
use ccrs::networking::http::HttpConfig;
use ccrs::types::HyperliquidInstrumentType;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let instrument_type = match get_env_as_string("INSTRUMENT_TYPE", "perpetuals")
        .to_lowercase()
        .as_str()
    {
        "perpetuals" => HyperliquidInstrumentType::Perpetuals,
        _ => HyperliquidInstrumentType::Spot,
    };

    let hyperliquid_client = HyperliquidClient::builder()
        .instrument_type(instrument_type)
        .build();

    let http_client = match hyperliquid_client
        .create_http_client(HttpConfig::default())
        .await
    {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    send_and_handle(
        &hyperliquid_client,
        &http_client,
        Request::GetInstrumentInfo(GetInstrumentInfoRequest {
            ..Default::default()
        }),
    )
    .await;

    let symbol_env = get_env_as_string("SYMBOL", "BTC");
    let symbol = match symbol_env.as_str() {
        "BTC" => "BTC".to_string(),
        "BTC/USDC" => "@142".to_string(),
        _ => panic!(),
    };

    send_and_handle(
        &hyperliquid_client,
        &http_client,
        Request::GetTopOfBook(GetTopOfBookRequest { symbol }),
    )
    .await;
}

async fn send_and_handle(client: &dyn ExchangeClient, http_client: &HttpClient, request: Request) {
    match client.send_http_request(http_client, request).await {
        Response::GetInstrumentInfo(data) => {
            println!("Got instrument info: {:#?}", data);
        }
        Response::GetTopOfBook(data) => {
            println!("Got top of book: {:#?}", data);
        }
        Response::HttpErrorResponse(http_resp) => {
            println!("HTTP error, status: {}", http_resp.status);
            println!("Headers: {:#?}", http_resp.headers);
            println!("Body: {:#?}", http_resp.body);
        }
        _ => unreachable!(),
    }
}
