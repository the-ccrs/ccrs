use ccrs::exchange_client::common::CancelOrderRequest;
use ccrs::exchange_client::common::Common;
use ccrs::exchange_client::common::GetOpenOrderRequest;
use ccrs::exchange_client::common::PlaceOrderRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::bybit::common::BybitClient;
use ccrs::exchanges::bybit::common::BybitCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::types::BybitInstrumentType;
use ccrs::types::OrderSide;
use ccrs::types::OrderType;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("BYBIT_API_KEY", "");
    let api_secret = get_env_as_string("BYBIT_API_SECRET", "");

    let credential = BybitCredential {
        api_key,
        api_secret,
    };
    let use_testnet = get_env_as_bool("USE_TESTNET", false);

    let mut bybit_client_builder = BybitClient::builder();

    if use_testnet {
        bybit_client_builder =
            bybit_client_builder.rest_api_base_url("https://api-testnet.bybit.com");
    }

    let bybit_client = bybit_client_builder
        .instrument_type(BybitInstrumentType::Spot)
        .credential(Some(credential))
        .build();

    let http_client = match bybit_client.create_http_client(HttpConfig::default()).await {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    let price = get_env_as_string("PRICE", "");

    match bybit_client
        .send_http_request(
            &http_client,
            Request::PlaceOrder(PlaceOrderRequest {
                symbol: get_env_as_string("SYMBOL", "BTCUSDT"),
                client_order_id: bybit_client.generate_next_client_order_id(),
                order_type: if price.is_empty() {
                    OrderType::Market
                } else {
                    OrderType::Limit
                },
                side: OrderSide::Buy,
                price,
                quantity: get_env_as_string("QUANTITY", ""),
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

    let order_ids: Vec<String> = match bybit_client
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
        match bybit_client
            .send_http_request(
                &http_client,
                Request::CancelOrder(CancelOrderRequest {
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
