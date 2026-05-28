use ccrs::exchange_client::common::CancelOrderRequest;
use ccrs::exchange_client::common::Common;
use ccrs::exchange_client::common::GetOpenOrderRequest;
use ccrs::exchange_client::common::PlaceOrderRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::hyperliquid::common::HyperliquidClient;
use ccrs::exchanges::hyperliquid::common::HyperliquidCredential;
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

    let private_key = get_env_as_string("HYPERLIQUID_PRIVATE_KEY", "");
    let use_testnet = get_env_as_bool("USE_TESTNET", false);

    let credential = if private_key.is_empty() {
        None
    } else {
        let signing_key = private_key
            .parse::<alloy::signers::local::PrivateKeySigner>()
            .expect("Invalid HYPERLIQUID_PRIVATE_KEY");
        Some(HyperliquidCredential::new(signing_key))
    };

    let mut hyperliquid_client_builder = HyperliquidClient::builder();

    if use_testnet {
        hyperliquid_client_builder = hyperliquid_client_builder.is_mainnet(false);
    }

    let hyperliquid_client = hyperliquid_client_builder.credential(credential).build();

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

    let price = get_env_as_string("PRICE", "");
    let symbol_env = get_env_as_string("SYMBOL", "BTC");
    let symbol = match symbol_env.as_str() {
        "BTC" => "0".to_string(),
        "BTC/USDC" => "10142".to_string(),
        _ => panic!(),
    };
    let side = get_env_as_string("SIDE", "");

    match hyperliquid_client
        .send_http_request(
            &http_client,
            Request::PlaceOrder(PlaceOrderRequest {
                symbol: symbol.clone(),
                client_order_id: hyperliquid_client.generate_next_client_order_id(),
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

    let orders: Vec<String> = match hyperliquid_client
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
        match hyperliquid_client
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
