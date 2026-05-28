#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::binance_spot::common::BinanceSpotClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::BinanceSpot(binance_endpoint) => {
                match binance_endpoint {
                    crate::types::BinanceSpotWebSocketEndpoint::MarketData => {
                        self.websocket_market_data_api_url.to_string()
                    }
                    crate::types::BinanceSpotWebSocketEndpoint::AccountData => {
                        self.websocket_account_data_api_url.to_string()
                    }
                    crate::types::BinanceSpotWebSocketEndpoint::Unknown => {
                        panic!("BinanceSpot WebSocket endpoint is Unknown")
                    }
                }
            }
            _ => panic!("WebSocket endpoint is not BinanceSpot"),
        }
    }

    fn create_authenticate_websocket_request(&self) -> String {
        let credential = match &self.credential {
            Some(c) => c,
            None => return String::new(),
        };

        let timestamp = chrono::Utc::now().timestamp_millis();
        let payload = format!("apiKey={}&timestamp={}", credential.api_key, timestamp);
        let signature_bytes =
            ed25519_dalek::Signer::sign(&credential.signing_key, payload.as_bytes());
        let signature = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            signature_bytes.to_bytes(),
        );

        serde_json::json!({
            "id": timestamp,
            "method": "session.logon",
            "params": {
                "apiKey": credential.api_key,
                "signature": signature,
                "timestamp": timestamp
            }
        })
        .to_string()
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(|| "".to_string())
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let params: Vec<String> = subscribe_top_of_book_request
            .symbols
            .iter()
            .map(|symbol| format!("{}@bookTicker", symbol.to_lowercase()))
            .collect();

        let mut payload = serde_json::Map::new();

        if let Some(id) = subscribe_top_of_book_request.id {
            payload.insert("id".to_string(), serde_json::json!(id));
        }

        payload.insert(
            "method".to_string(),
            serde_json::Value::String("SUBSCRIBE".to_string()),
        );
        payload.insert("params".to_string(), serde_json::to_value(params).unwrap());

        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let params: Vec<String> = subscribe_trade_request
            .symbols
            .iter()
            .map(|symbol| format!("{}@trade", symbol.to_lowercase()))
            .collect();

        let mut payload = serde_json::Map::new();

        if let Some(id) = subscribe_trade_request.id {
            payload.insert("id".to_string(), serde_json::json!(id));
        }

        payload.insert(
            "method".to_string(),
            serde_json::Value::String("SUBSCRIBE".to_string()),
        );
        payload.insert("params".to_string(), serde_json::to_value(params).unwrap());

        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        _subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        let timestamp = chrono::Utc::now().timestamp_millis();

        serde_json::json!({
            "id": timestamp,
            "method": "userDataStream.subscribe"
        })
        .to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        _subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        panic!()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload {
            if let Some(v) = json_payload.get("stream").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("stream".to_string(), v.to_string());
            }

            if let Some(v) = json_payload
                .get("event")
                .and_then(|d| d.get("e"))
                .and_then(|v| v.as_str())
            {
                websocket_text
                    .payload_summary
                    .insert("event.e".to_string(), v.to_string());
            }

            if let Some(v) = json_payload.get("status").and_then(|v| v.as_i64()) {
                websocket_text
                    .payload_summary
                    .insert("status".to_string(), v.to_string());
            }

            if json_payload.get("result").is_some_and(|v| v.is_null()) {
                websocket_text
                    .payload_summary
                    .insert("result_null".to_string(), String::new());
            }

            if json_payload.get("error").is_some_and(|v| !v.is_null()) {
                websocket_text
                    .payload_summary
                    .insert("error".to_string(), String::new());
            }

            if let Some(v) = json_payload
                .get("result")
                .and_then(|r| r.get("apiKey"))
                .and_then(|v| v.as_str())
            {
                websocket_text
                    .payload_summary
                    .insert("api_key".to_string(), v.to_string());
            }

            if let Some(v) = json_payload
                .get("result")
                .and_then(|r| r.get("subscriptionId"))
                .and_then(|v| v.as_i64())
            {
                websocket_text
                    .payload_summary
                    .insert("subscription_id".to_string(), v.to_string());
            }
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary.contains_key("stream") || payload_summary.contains_key("event.e")
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("stream")
            .map(|stream| stream.ends_with("bookTicker"))
            .unwrap_or_default()
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("stream")
            .map(|stream| stream.ends_with("trade"))
            .unwrap_or_default()
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("event.e")
            .map(|e| e == "executionReport")
            .unwrap_or_default()
    }

    fn is_websocket_text_fill_subscription_data(
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

        let data = json_payload.get("data").unwrap();

        let symbol = data.get("s").and_then(|v| v.as_str()).unwrap().to_string();
        let timestamp = chrono::Utc::now();

        let bid_price = data.get("b").and_then(|v| v.as_str()).unwrap().to_string();
        let bid_size = data.get("B").and_then(|v| v.as_str()).unwrap().to_string();
        let ask_price = data.get("a").and_then(|v| v.as_str()).unwrap().to_string();
        let ask_size = data.get("A").and_then(|v| v.as_str()).unwrap().to_string();

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::BinanceSpot,
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

        let data = &json_payload["data"];

        let symbol = data["s"].as_str().unwrap().to_string();

        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
            data["T"].as_i64().unwrap(),
        );

        let price = data["p"].as_str().unwrap().to_string();
        let size = data["q"].as_str().unwrap().to_string();

        let is_buyer_maker = data["m"].as_bool().unwrap();

        let side = if is_buyer_maker {
            crate::types::TakerSide::Sell
        } else {
            crate::types::TakerSide::Buy
        };

        crate::exchange_client::common::Response::TradeSubscription(
            crate::exchange_client::common::TradeSubscriptionData {
                data: vec![crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::BinanceSpot,
                    symbol,
                    timestamp,
                    price,
                    size,
                    side,
                }],
            },
        )
    }

    fn create_subscribe_order_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let order = self.convert_ws_executionreport_to_order(json_payload);

        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData { data: vec![order] },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        _: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        panic!()
    }

    fn is_unexpected_websocket_text_subscription_data_benign(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.contains_key("event.e")
    }

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("status")
            .map(|v| v == "200")
            .unwrap_or(false)
            || (websocket_text.payload_summary.contains_key("result_null")
                && !websocket_text.payload_summary.contains_key("error"))
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.contains_key("api_key")
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.contains_key("result_null")
            && !websocket_text.payload_summary.contains_key("error")
            && !websocket_text.payload_summary.contains_key("api_key")
            || websocket_text
                .payload_summary
                .get("status")
                .map(|v| v == "200")
                .unwrap_or(false)
                && websocket_text
                    .payload_summary
                    .contains_key("subscription_id")
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
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

        new_websocket_text.error_message = json_payload
            .get("msg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
