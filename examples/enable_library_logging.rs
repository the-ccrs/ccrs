// cargo run --example enable_library_logging --features max_log_level_finest

use ccrs::exchange_client::common::GetInstrumentInfoRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::bybit::common::BybitClient;
use ccrs::networking::http::HttpConfig;
use ccrs::types::BybitInstrumentType;

#[tokio::main]
async fn main() {
    ccrs::logger::init_logger(|level, file, line, msg| {
        println!("[{:#?}] {}:{} {}", level, file, line, msg);
    });

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

    let request = Request::GetInstrumentInfo(GetInstrumentInfoRequest {
        limit: 1,
        next_page_cursor: "first%3D0GUSDT%26last%3D0GUSDT".into(),
        ..Default::default()
    });

    let _ = bybit_client.send_http_request(&http_client, request).await;
}
