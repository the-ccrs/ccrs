use ccrs::exchange_client::ExchangeClient;
use ccrs::exchange_client::common::GetInstrumentInfoRequest;
use ccrs::exchange_client::common::GetTopOfBookRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::bybit::common::BybitClient;
use ccrs::networking::http::HttpClient;
use ccrs::networking::http::HttpConfig;
use ccrs::types::BybitInstrumentType;
use ccrs::utils::get_env_as_number;

#[tokio::main]
async fn main() {
    let bybit_client = BybitClient::builder()
        .instrument_type(BybitInstrumentType::Linear)
        .build();

    let http_client = match bybit_client.create_http_client(HttpConfig::default()).await {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    send_and_handle(
        &bybit_client,
        &http_client,
        Request::GetInstrumentInfo(GetInstrumentInfoRequest {
            limit: get_env_as_number::<u32>("GET_INSTRUMENT_INFO_LIMIT", 2),
            ..Default::default()
        }),
    )
    .await;

    send_and_handle(
        &bybit_client,
        &http_client,
        Request::GetTopOfBook(GetTopOfBookRequest {
            symbol: "BTCUSDT".into(),
        }),
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
