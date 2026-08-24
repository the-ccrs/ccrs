use ccrs::exchange_client::common::CancelOrderRequest;
use ccrs::exchange_client::common::Common;
use ccrs::exchange_client::common::GetOpenOrderRequest;
use ccrs::exchange_client::common::PlaceOrderRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::kalshi_perps::common::KalshiPerpsClient;
use ccrs::exchanges::kalshi_perps::common::KalshiPerpsCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::types::OrderSide;
use ccrs::types::OrderType;
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

    let price = get_env_as_string("PRICE", "");
    let symbol = get_env_as_string("SYMBOL", "");
    let side = match get_env_as_string("SIDE", "buy").to_lowercase().as_str() {
        "sell" => OrderSide::Sell,
        _ => OrderSide::Buy,
    };

    match kalshi_perps_client
        .send_http_request(
            &http_client,
            Request::PlaceOrder(PlaceOrderRequest {
                symbol: symbol.clone(),
                client_order_id: kalshi_perps_client.generate_next_client_order_id(),
                order_type: if price.is_empty() {
                    OrderType::Market
                } else {
                    OrderType::Limit
                },
                side,
                price,
                quantity: get_env_as_string("QUANTITY", ""),
                ..Default::default()
            }),
        )
        .await
    {
        Response::PlaceOrder(data) => {
            println!("Place order acknowledged: {:#?}", data);
        }
        Response::HttpErrorResponse(http_resp) => {
            println!("HTTP error, status: {}", http_resp.status);
            println!("Headers: {:#?}", http_resp.headers);
            println!("Body: {:#?}", http_resp.body);
        }
        _ => unreachable!(),
    }

    let order_ids: Vec<String> = match kalshi_perps_client
        .send_http_request(
            &http_client,
            Request::GetOpenOrder(GetOpenOrderRequest {
                ..Default::default()
            }),
        )
        .await
    {
        Response::GetOpenOrder(data) => {
            println!("Got open order: {:#?}", data);
            data.data.iter().map(|o| o.order_id.clone()).collect()
        }
        _ => Vec::new(),
    };

    for order_id in order_ids {
        match kalshi_perps_client
            .send_http_request(
                &http_client,
                Request::CancelOrder(CancelOrderRequest {
                    symbol: symbol.clone(),
                    order_id,
                    ..Default::default()
                }),
            )
            .await
        {
            Response::CancelOrder(data) => {
                println!("Cancel order acknowledged: {:#?}", data);
            }
            _ => unreachable!(),
        }
    }
}
