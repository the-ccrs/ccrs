#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::gateio_perpetual_futures::common::GateioPerpetualFuturesClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::GateioPerpetualFutures(gateio_endpoint) => {
                match gateio_endpoint {
                    crate::types::GateioPerpetualFuturesWebSocketEndpoint::MarketData => {
                        self.websocket_market_data_api_url.to_string()
                    }
                    crate::types::GateioPerpetualFuturesWebSocketEndpoint::AccountData => {
                        self.websocket_account_data_api_url.to_string()
                    }
                    crate::types::GateioPerpetualFuturesWebSocketEndpoint::Unknown => {
                        panic!("GateioPerpetualFutures endpoint is Unknown")
                    }
                }
            }
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

        let timestamp = chrono::Utc::now().timestamp();

        let sign_str = format!("api\nfutures.login\n\n{}", timestamp);

        let mut mac = <hmac::Hmac<sha2::Sha512> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .unwrap();

        <hmac::Hmac<sha2::Sha512> as hmac::Mac>::update(&mut mac, sign_str.as_bytes());
        let signature =
            hex::encode(<hmac::Hmac<sha2::Sha512> as hmac::Mac>::finalize(mac).into_bytes());

        serde_json::json!({
            "id": timestamp * 1_000_000,
            "time": timestamp,
            "channel": "futures.login",
            "event": "api",
            "payload": {
                "req_id": format!("login-{}", timestamp),
                "api_key": credential.api_key,
                "req_header": {},
                "signature": signature,
                "timestamp": timestamp.to_string()
            }
        })
        .to_string()
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(move || {
            serde_json::json!({
                "time": chrono::Utc::now().timestamp(),
                "channel": "futures.ping"
            })
            .to_string()
        })
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let payload: Vec<String> = subscribe_top_of_book_request.symbols.clone();

        let mut msg = serde_json::Map::new();

        msg.insert(
            "time".to_string(),
            serde_json::json!(chrono::Utc::now().timestamp()),
        );

        if let Some(id) = subscribe_top_of_book_request.id {
            msg.insert("id".to_string(), serde_json::json!(id));
        }

        msg.insert(
            "channel".to_string(),
            serde_json::Value::String("futures.book_ticker".to_string()),
        );
        msg.insert(
            "event".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        msg.insert(
            "payload".to_string(),
            serde_json::to_value(payload).unwrap(),
        );

        serde_json::Value::Object(msg).to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let payload: Vec<String> = subscribe_trade_request.symbols.clone();

        let mut msg = serde_json::Map::new();

        msg.insert(
            "time".to_string(),
            serde_json::json!(chrono::Utc::now().timestamp()),
        );

        if let Some(id) = subscribe_trade_request.id {
            msg.insert("id".to_string(), serde_json::json!(id));
        }

        msg.insert(
            "channel".to_string(),
            serde_json::Value::String("futures.trades".to_string()),
        );
        msg.insert(
            "event".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        msg.insert(
            "payload".to_string(),
            serde_json::to_value(payload).unwrap(),
        );

        serde_json::Value::Object(msg).to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        _subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        let channel = "futures.orders";
        let event = "subscribe";
        let timestamp = chrono::Utc::now().timestamp();

        serde_json::json!({
            "time": timestamp,
            "channel": channel,
            "event": event,
            "payload": ["!all"],
            "auth": self.sign_websocket_request(channel, event, timestamp)
        })
        .to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        _subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        let channel = "futures.usertrades";
        let event = "subscribe";
        let timestamp = chrono::Utc::now().timestamp();

        serde_json::json!({
            "time": timestamp,
            "channel": channel,
            "event": event,
            "payload": ["!all"],
            "auth": self.sign_websocket_request(channel, event, timestamp)
        })
        .to_string()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload {
            if let Some(v) = json_payload.get("channel") {
                websocket_text.payload_summary.insert(
                    "channel".to_string(),
                    v.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| v.to_string()),
                );
            }

            if let Some(v) = json_payload.get("event") {
                websocket_text.payload_summary.insert(
                    "event".to_string(),
                    v.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| v.to_string()),
                );
            }

            if let Some(v) = json_payload.get("error") {
                websocket_text.payload_summary.insert(
                    "error".to_string(),
                    v.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| v.to_string()),
                );
            }

            if let Some(result) = json_payload.get("result")
                && let Some(status) = result.get("status")
            {
                websocket_text.payload_summary.insert(
                    "result.status".to_string(),
                    status.as_str().unwrap_or("").to_string(),
                );
            }

            if let Some(header) = json_payload.get("header") {
                let channel_is_login =
                    header.get("channel").and_then(|v| v.as_str()) == Some("futures.login");

                let status_is_200 = header.get("status").and_then(|v| v.as_str()) == Some("200");

                if channel_is_login && status_is_200 {
                    websocket_text
                        .payload_summary
                        .insert("login_success".to_string(), "true".to_string());
                }
            }
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.get("event") == Some(&"update".to_string())
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.get("channel") == Some(&"futures.book_ticker".to_string())
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.get("channel") == Some(&"futures.trades".to_string())
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.get("channel") == Some(&"futures.orders".to_string())
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.get("channel") == Some(&"futures.usertrades".to_string())
    }

    fn is_unexpected_websocket_text_subscription_data_benign(
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

        let result = json_payload.get("result").unwrap();

        let symbol = result
            .get("s")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let timestamp_ms = result.get("t").and_then(|v| v.as_i64()).unwrap();

        let timestamp =
            crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(timestamp_ms);

        let bid_price = result
            .get("b")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let bid_size = result
            .get("B")
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string())
            })
            .unwrap_or_default();

        let ask_price = result
            .get("a")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let ask_size = result
            .get("A")
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string())
            })
            .unwrap_or_default();

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type:
                        crate::types::ExchangeInstrumentType::GateioPerpetualFutures,
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

        let results = json_payload
            .get("result")
            .and_then(|v| v.as_array())
            .unwrap();

        let trades = results
            .iter()
            .map(|result| {
                let symbol = result
                    .get("contract")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let timestamp_ms = result
                    .get("create_time_ms")
                    .and_then(|v| v.as_i64())
                    .unwrap();

                let timestamp =
                    crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(timestamp_ms);

                let price = result
                    .get("price")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let size_val = result
                    .get("size")
                    .and_then(|v| {
                        if let Some(s) = v.as_str() {
                            s.parse::<i64>().ok()
                        } else {
                            v.as_i64()
                        }
                    })
                    .unwrap_or(0);

                let size = size_val.abs().to_string();

                let side = if size_val > 0 {
                    crate::types::TakerSide::Buy
                } else if size_val < 0 {
                    crate::types::TakerSide::Sell
                } else {
                    crate::types::TakerSide::Unknown
                };

                crate::types::Trade {
                    exchange_instrument_type:
                        crate::types::ExchangeInstrumentType::GateioPerpetualFutures,
                    symbol,
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

        let data_array = json_payload
            .get("result")
            .and_then(|v| v.as_array())
            .unwrap();

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

        let data_array = json_payload
            .get("result")
            .and_then(|v| v.as_array())
            .unwrap();

        let fills: Vec<crate::types::Fill> = data_array
            .iter()
            .map(|data| {
                let exchange_instrument_type =
                    crate::types::ExchangeInstrumentType::GateioPerpetualFutures;

                let symbol = data
                    .get("contract")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let order_id = data
                    .get("order_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let client_order_id = data
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let size_val = data.get("size").and_then(|v| v.as_i64()).unwrap_or(0);

                let side = if size_val > 0 {
                    crate::types::OrderSide::Buy
                } else if size_val < 0 {
                    crate::types::OrderSide::Sell
                } else {
                    crate::types::OrderSide::Unknown
                };

                let price = data
                    .get("price")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let quantity = size_val.abs().to_string();

                let is_maker = data.get("role").and_then(|v| v.as_str()) == Some("maker");

                crate::types::Fill {
                    exchange_instrument_type,
                    symbol,
                    order_id,
                    client_order_id,
                    side,
                    price,
                    quantity,
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

        payload_summary.get("result.status") == Some(&"success".to_string())
            || payload_summary.get("channel") == Some(&"futures.pong".to_string())
            || payload_summary.get("login_success") == Some(&"true".to_string())
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.get("login_success") == Some(&"true".to_string())
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;

        payload_summary.get("event") == Some(&"subscribe".to_string())
            && payload_summary.get("result.status") == Some(&"success".to_string())
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.get("channel") == Some(&"futures.pong".to_string())
    }

    fn create_authenticate_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let id = json_payload.get("id").and_then(|v| v.as_u64());

        crate::exchange_client::common::Response::Authenticate(
            crate::exchange_client::common::AuthenticateResponse { id },
        )
    }

    fn create_subscribe_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let id = json_payload.get("id").and_then(|v| v.as_u64());

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

        let id = json_payload.get("id").and_then(|v| v.as_u64());

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

        if let Some(error_obj) = json_payload.get("error").and_then(|v| v.as_object()) {
            new_websocket_text.error_code = error_obj.get("code").map(|v| v.to_string());

            new_websocket_text.error_message = error_obj
                .get("message")
                .and_then(|v| v.as_str())
                .map(str::to_string);
        }

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
