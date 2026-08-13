#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket for crate::exchanges::bybit::common::BybitClient {
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::Bybit(bybit_endpoint) => match bybit_endpoint {
                crate::types::BybitWebSocketEndpoint::MarketData => {
                    self.websocket_market_data_api_url.to_string()
                }
                crate::types::BybitWebSocketEndpoint::AccountData => {
                    self.websocket_account_data_api_url.to_string()
                }
                crate::types::BybitWebSocketEndpoint::Unknown => {
                    panic!("Bybit endpoint is Unknown")
                }
            },
            _ => {
                panic!("Websocket endpoint is Unknown")
            }
        }
    }

    fn create_authenticate_websocket_request(&self) -> String {
        let credential = match &self.credential {
            Some(c) => c,
            None => return String::new(),
        };

        let expires = chrono::Utc::now().timestamp_millis() + self.api_receive_window_milliseconds;

        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .unwrap();

        let payload = format!("GET/realtime{}", expires);
        hmac::Mac::update(&mut mac, payload.as_bytes());
        let signature = hex::encode(hmac::Mac::finalize(mac).into_bytes());

        serde_json::json!({
            "op": "auth",
            "args": [credential.api_key, expires, signature]
        })
        .to_string()
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(move || {
            serde_json::json!({
                "req_id": chrono::Utc::now().timestamp_millis().to_string(),
                "op": "ping"
            })
            .to_string()
        })
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let args: Vec<String> = subscribe_top_of_book_request
            .symbols
            .iter()
            .map(|symbol| format!("orderbook.1.{}", symbol))
            .collect();

        let mut payload = serde_json::Map::new();

        if let Some(id) = subscribe_top_of_book_request.id {
            payload.insert(
                "req_id".to_string(),
                serde_json::Value::String(id.to_string()),
            );
        }

        payload.insert(
            "op".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        payload.insert("args".to_string(), serde_json::to_value(args).unwrap());

        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let args: Vec<String> = subscribe_trade_request
            .symbols
            .iter()
            .map(|symbol| format!("publicTrade.{}", symbol))
            .collect();

        let mut payload = serde_json::Map::new();

        if let Some(id) = subscribe_trade_request.id {
            payload.insert(
                "req_id".to_string(),
                serde_json::Value::String(id.to_string()),
            );
        }

        payload.insert(
            "op".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        payload.insert("args".to_string(), serde_json::to_value(args).unwrap());

        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        _subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        let args = vec![format!("order.{}", self.category)];

        let payload = serde_json::json!({
            "op": "subscribe",
            "args": args
        });

        payload.to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        _subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        let args = vec![format!("execution.{}", self.category)];

        let payload = serde_json::json!({
            "op": "subscribe",
            "args": args
        });

        payload.to_string()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload {
            if let Some(b) = json_payload.get("success").and_then(|v| v.as_bool()) {
                websocket_text
                    .payload_summary
                    .insert("success".to_string(), b.to_string());
            }

            if let Some(s) = json_payload.get("op").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("op".to_string(), s.to_string());
            }

            if let Some(s) = json_payload.get("topic").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("topic".to_string(), s.to_string());
            }

            if let Some(s) = json_payload.get("retCode").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("retCode".to_string(), s.to_string());
            }
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary.contains_key("topic")
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("topic")
            .map(|topic| topic.starts_with("orderbook.1."))
            .unwrap()
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("topic")
            .map(|topic| topic.starts_with("publicTrade."))
            .unwrap()
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("topic")
            .map(|topic| topic.starts_with("order"))
            .unwrap()
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("topic")
            .map(|topic| topic.starts_with("execution"))
            .unwrap()
    }

    fn is_websocket_text_unneeded_subscription_data(
        &self,
        _: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
    }

    fn create_subscribe_top_of_book_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let symbol = json_payload
            .get("topic")
            .and_then(|v| v.as_str())
            .and_then(|topic| topic.rsplit('.').next())
            .unwrap()
            .to_string();

        let timestamp: chrono::DateTime<chrono::Utc> =
            crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
                json_payload.get("cts").and_then(|v| v.as_i64()).unwrap(),
            );

        let data = json_payload.get("data").unwrap();

        let bid_price = data
            .get("b")
            .and_then(|b| b.get(0))
            .and_then(|l| l.get(0))
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let bid_size = data
            .get("b")
            .and_then(|b| b.get(0))
            .and_then(|l| l.get(1))
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let ask_price = data
            .get("a")
            .and_then(|a| a.get(0))
            .and_then(|l| l.get(0))
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let ask_size = data
            .get("a")
            .and_then(|a| a.get(0))
            .and_then(|l| l.get(1))
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Bybit(
                        self.instrument_type,
                    ),
                    symbol,
                    timestamp,
                    bid_price,
                    bid_size,
                    ask_price,
                    ask_size,
                }],
            },
        )
    }

    fn create_subscribe_trade_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let symbol = json_payload
            .get("topic")
            .and_then(|v| v.as_str())
            .and_then(|topic| topic.rsplit('.').next())
            .unwrap()
            .to_string();

        let data_array = json_payload.get("data").and_then(|v| v.as_array()).unwrap();

        let trades: Vec<crate::types::Trade> = data_array
            .iter()
            .map(|data| {
                let timestamp: chrono::DateTime<chrono::Utc> =
                    crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
                        data.get("T").and_then(|v| v.as_i64()).unwrap(),
                    );

                let price = data.get("p").and_then(|v| v.as_str()).unwrap().to_string();

                let size = data.get("v").and_then(|v| v.as_str()).unwrap().to_string();

                let side = match data.get("S").and_then(|v| v.as_str()) {
                    Some("Buy") => crate::types::TakerSide::Buy,
                    Some("Sell") => crate::types::TakerSide::Sell,
                    _ => crate::types::TakerSide::Unknown,
                };

                crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Bybit(
                        self.instrument_type,
                    ),
                    symbol: symbol.clone(),
                    timestamp,
                    price,
                    size,
                    side,
                }
            })
            .collect();

        crate::exchange_client::common::Response::TradeSubscription(
            crate::exchange_client::common::TradeSubscriptionData { data: trades },
        )
    }

    fn create_subscribe_order_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let data_array = json_payload.get("data").and_then(|v| v.as_array()).unwrap();

        let orders: Vec<crate::types::Order> = data_array
            .iter()
            .map(|order_value| self.convert_json_value_to_order(order_value))
            .collect();

        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData { data: orders },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let data_array = json_payload.get("data").and_then(|v| v.as_array()).unwrap();

        let fills: Vec<crate::types::Fill> = data_array
            .iter()
            .map(|data| {
                let exchange_instrument_type =
                    crate::types::ExchangeInstrumentType::Bybit(self.instrument_type);
                let order_id = data
                    .get("orderId")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let symbol = data
                    .get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let client_order_id = data
                    .get("orderLinkId")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let side = match data.get("side").and_then(|v| v.as_str()) {
                    Some("Buy") => crate::types::OrderSide::Buy,
                    Some("Sell") => crate::types::OrderSide::Sell,
                    _ => crate::types::OrderSide::Unknown,
                };

                let price = data
                    .get("execPrice")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let quantity = data
                    .get("execQty")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let quote_quantity = data
                    .get("execValue")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let is_maker = data.get("isMaker").and_then(|v| v.as_bool()).unwrap();

                crate::types::Fill {
                    exchange_instrument_type,
                    symbol,
                    order_id,
                    client_order_id,
                    side,
                    price,
                    quantity,
                    quote_quantity,
                    is_maker,
                    ..Default::default()
                }
            })
            .collect();

        crate::exchange_client::common::Response::FillSubscription(
            crate::exchange_client::common::FillSubscriptionData { data: fills },
        )
    }

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;

        payload_summary.get("success") == Some(&"true".to_string())
            || payload_summary.get("retCode") == Some(&"0".to_string())
            || payload_summary.get("op") == Some(&"pong".to_string())
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;

        payload_summary.get("success") == Some(&"true".to_string())
            && payload_summary.get("op") == Some(&"auth".to_string())
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;

        payload_summary
            .get("op")
            .map(|op| op == "subscribe")
            .unwrap()
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;

        payload_summary
            .get("op")
            .map(|op| op == "ping" || op == "pong")
            .unwrap()
    }

    fn create_authenticate_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let id = json_payload.get("req_id").and_then(|v| v.as_u64());

        crate::exchange_client::common::Response::Authenticate(
            crate::exchange_client::common::AuthenticateResponse { id },
        )
    }

    fn create_subscribe_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let id = json_payload.get("req_id").and_then(|v| v.as_u64());

        crate::exchange_client::common::Response::Subscribe(
            crate::exchange_client::common::SubscribeResponse {
                id,
                ..Default::default()
            },
        )
    }

    fn create_heartbeat_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let id = json_payload
            .get("req_id")
            .and_then(|v| {
                if v.is_string() {
                    let s = v.as_str().unwrap();
                    if s.is_empty() {
                        None
                    } else {
                        s.parse::<u64>().ok()
                    }
                } else {
                    v.as_u64()
                }
            })
            .or_else(|| {
                json_payload
                    .get("args")
                    .and_then(|args| args.get(0))
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<u64>().ok())
            });

        crate::exchange_client::common::Response::Heartbeat(
            crate::exchange_client::common::HeartbeatResponse { id },
        )
    }

    fn create_websocket_error_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = match websocket_text.json_payload.as_ref() {
            Some(payload) => payload,
            None => {
                return crate::exchange_client::common::Response::WebSocketErrorResponse(
                    websocket_text.clone(),
                );
            }
        };

        let mut new_websocket_text = websocket_text.clone();

        new_websocket_text.error_message = json_payload
            .get("ret_msg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
