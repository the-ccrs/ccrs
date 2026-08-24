use ccrs::exchange_client::common::CancelOrderRequest;
use ccrs::exchange_client::common::Common;
use ccrs::exchange_client::common::GetOpenOrderRequest;
use ccrs::exchange_client::common::PlaceOrderRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::polymarket::common::PolymarketClient;
use ccrs::exchanges::polymarket::common::PolymarketCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::types::OrderSide;
use ccrs::types::OrderType;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_number;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let private_key = get_env_as_string("POLYMARKET_PRIVATE_KEY", "");
    let use_testnet = get_env_as_bool("USE_TESTNET", false);

    // https://docs.polymarket.com/trading/overview#signature-types
    // Important: funder_address and signature_type must be correct. If incorrect, the order will be rejected.
    let funder_address = get_env_as_string("POLYMARKET_FUNDER_ADDRESS", "");
    let signature_type = get_env_as_number::<i64>("POLYMARKET_SIGNATURE_TYPE", 0);

    let signing_key = private_key
        .parse::<alloy::signers::local::PrivateKeySigner>()
        .expect("Invalid POLYMARKET_PRIVATE_KEY");

    let credential = PolymarketCredential::from_private_key(signing_key)
        .await
        .unwrap();

    let mut polymarket_client_builder = PolymarketClient::builder();

    if use_testnet {
        polymarket_client_builder = polymarket_client_builder.is_mainnet(false);
    }

    let mut polymarket_client = polymarket_client_builder
        .credential(Some(credential))
        .funder_address(funder_address)
        .signature_type(signature_type)
        .build();

    polymarket_client.initialize_sdk_client().await.unwrap();

    let http_client = match polymarket_client
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
    let side = get_env_as_string("SIDE", "");

    match polymarket_client
        .send_http_request(
            &http_client,
            Request::PlaceOrder(PlaceOrderRequest {
                symbol: symbol.clone(),
                client_order_id: polymarket_client.generate_next_client_order_id(),
                order_type: if price.is_empty() {
                    OrderType::Market
                } else {
                    OrderType::Limit
                },
                side: match side.to_lowercase().as_str() {
                    "sell" => OrderSide::Sell,
                    _ => OrderSide::Buy,
                },
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

    let orders: Vec<String> = match polymarket_client
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

    for order_id in orders {
        match polymarket_client
            .send_http_request(
                &http_client,
                Request::CancelOrder(CancelOrderRequest {
                    order_id,
                    symbol: symbol.clone(),
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
