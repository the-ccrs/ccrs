use ccrs::exchange_client::common::CancelOrderRequest;
use ccrs::exchange_client::common::Common;
use ccrs::exchange_client::common::GetOpenOrderRequest;
use ccrs::exchange_client::common::PlaceOrderRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::bitstamp::common::BitstampClient;
use ccrs::exchanges::bitstamp::common::BitstampCredential;
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

    let price = get_env_as_string("PRICE", "");
    let side = match get_env_as_string("SIDE", "buy").to_lowercase().as_str() {
        "sell" => OrderSide::Sell,
        _ => OrderSide::Buy,
    };
    let symbol = get_env_as_string("SYMBOL", "btcusd");

    match bitstamp_client
        .send_http_request(
            &http_client,
            Request::PlaceOrder(PlaceOrderRequest {
                symbol: symbol.clone(),
                client_order_id: bitstamp_client.generate_next_client_order_id(),
                order_type: if price.is_empty() {
                    OrderType::Market
                } else {
                    OrderType::Limit
                },
                side,
                price,
                quantity: get_env_as_string("QUANTITY", ""),
                leverage: if BitstampClient::is_instrument_derivatives(&symbol) {
                    "1".to_string()
                } else {
                    String::new()
                },
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

    let order_ids: Vec<String> = match bitstamp_client
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
        match bitstamp_client
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
