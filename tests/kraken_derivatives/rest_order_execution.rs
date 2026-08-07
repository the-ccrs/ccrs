use ccrs::exchange_client::common::CancelOrderRequest;
use ccrs::exchange_client::common::Common;
use ccrs::exchange_client::common::GetOpenOrderRequest;
use ccrs::exchange_client::common::PlaceOrderRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::kraken_derivatives::common::KrakenDerivativesClient;
use ccrs::exchanges::kraken_derivatives::common::KrakenDerivativesCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::types::OrderSide;
use ccrs::types::OrderType;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("KRAKEN_DERIVATIVES_API_KEY", "");
    let api_secret = get_env_as_string("KRAKEN_DERIVATIVES_API_SECRET", "");

    let credential = KrakenDerivativesCredential {
        api_key,
        api_secret,
    };

    let rest_api_base_url = get_env_as_string("KRAKEN_DERIVATIVES_REST_API_BASE_URL", "");

    let mut kraken_derivatives_client_builder = KrakenDerivativesClient::builder();

    if !rest_api_base_url.is_empty() {
        kraken_derivatives_client_builder =
            kraken_derivatives_client_builder.rest_api_base_url(rest_api_base_url)
    }

    let kraken_derivatives_client = kraken_derivatives_client_builder
        .credential(Some(credential))
        .build();

    let http_client = match kraken_derivatives_client
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

    let side = match get_env_as_string("SIDE", "BUY").to_uppercase().as_str() {
        "BUY" => OrderSide::Buy,
        "SELL" => OrderSide::Sell,
        other => panic!("Invalid SIDE '{}'. Expected BUY or SELL.", other),
    };

    match kraken_derivatives_client
        .send_http_request(
            &http_client,
            Request::PlaceOrder(PlaceOrderRequest {
                symbol: get_env_as_string("SYMBOL", "PF_XBTUSD"),
                client_order_id: kraken_derivatives_client.generate_next_client_order_id(),
                order_type: if price.is_empty() {
                    OrderType::Market
                } else {
                    OrderType::Limit
                },
                side,
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

    let order_ids: Vec<String> = match kraken_derivatives_client
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
        match kraken_derivatives_client
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
