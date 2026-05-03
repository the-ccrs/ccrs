use ccrs::exchange_client::common::CancelOrderRequest;
use ccrs::exchange_client::common::Common;
use ccrs::exchange_client::common::GetOpenOrderRequest;
use ccrs::exchange_client::common::PlaceOrderRequest;
use ccrs::exchange_client::common::Request;
use ccrs::exchange_client::common::Response;
use ccrs::exchange_client::rest::Rest;
use ccrs::exchanges::okx::common::OkxClient;
use ccrs::exchanges::okx::common::OkxCredential;
use ccrs::networking::http::HttpConfig;
use ccrs::types::OkxInstrumentType;
use ccrs::types::OrderSide;
use ccrs::types::OrderType;
use ccrs::utils::get_env_as_bool;
use ccrs::utils::get_env_as_string;
#[path = "../common.rs"]
mod common;

#[tokio::test]
async fn main() {
    common::setup();

    let api_key = get_env_as_string("OKX_API_KEY", "");
    let api_secret = get_env_as_string("OKX_API_SECRET", "");
    let passphrase = get_env_as_string("OKX_PASSPHRASE", "");

    let credential = OkxCredential {
        api_key,
        api_secret,
        passphrase,
    };

    let mut okx_client_builder = OkxClient::builder();

    okx_client_builder =
        okx_client_builder.use_demo_trading(Some(get_env_as_bool("USE_DEMO_TRADING", false)));

    let td_mode = get_env_as_string("TD_MODE", "cash");
    let okx_client = okx_client_builder
        .instrument_type(OkxInstrumentType::Spot)
        .credential(Some(credential))
        .td_mode(td_mode)
        .build();

    let http_client = match okx_client.create_http_client(HttpConfig::default()).await {
        Ok(client) => client,
        Err(err) => {
            println!("Failed to create HTTP client: {:#?}", err);
            return;
        }
    };

    let price = get_env_as_string("PRICE", "");
    let symbol = get_env_as_string("SYMBOL", "BTC-USDT");

    match okx_client
        .send_http_request(
            &http_client,
            Request::PlaceOrder(PlaceOrderRequest {
                symbol: symbol.clone(),
                client_order_id: okx_client.generate_next_client_order_id(),
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

    let order_ids: Vec<String> = match okx_client
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
        match okx_client
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
