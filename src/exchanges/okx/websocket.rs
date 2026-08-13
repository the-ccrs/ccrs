#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket for crate::exchanges::okx::common::OkxClient {
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::Okx(okx_endpoint) => match okx_endpoint {
                crate::types::OkxWebSocketEndpoint::MarketData => {
                    self.websocket_market_data_api_url.to_string()
                }
                crate::types::OkxWebSocketEndpoint::AccountData => {
                    self.websocket_account_data_api_url.to_string()
                }
                crate::types::OkxWebSocketEndpoint::Unknown => {
                    panic!("Okx endpoint is Unknown")
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

        let timestamp = chrono::Utc::now().timestamp().to_string();

        let prehash = format!("{}GET/users/self/verify", timestamp);

        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .unwrap();

        hmac::Mac::update(&mut mac, prehash.as_bytes());
        let signature_bytes = hmac::Mac::finalize(mac).into_bytes();
        let signature =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature_bytes);

        serde_json::json!({
            "op": "login",
            "args": [{
                "apiKey": credential.api_key,
                "passphrase": credential.passphrase,
                "timestamp": timestamp,
                "sign": signature
            }]
        })
        .to_string()
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(move || "ping".to_string())
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let args: Vec<serde_json::Value> = subscribe_top_of_book_request
            .symbols
            .iter()
            .map(|symbol| {
                serde_json::json!({
                    "channel": "bbo-tbt",
                    "instId": symbol
                })
            })
            .collect();

        serde_json::json!({
            "op": "subscribe",
            "args": args
        })
        .to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let args: Vec<serde_json::Value> = subscribe_trade_request
            .symbols
            .iter()
            .map(|symbol| {
                serde_json::json!({
                    "channel": "trades",
                    "instId": symbol
                })
            })
            .collect();

        serde_json::json!({
            "op": "subscribe",
            "args": args
        })
        .to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        _subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        serde_json::json!({
            "op": "subscribe",
            "args": [{
                "channel": "orders",
                "instType": self.inst_type_str
            }]
        })
        .to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        _subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        serde_json::json!({
            "op": "subscribe",
            "args": [{
                "channel": "fills",
                "instType": self.inst_type_str
            }]
        })
        .to_string()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if websocket_text.text == "pong" {
            websocket_text
                .payload_summary
                .insert("op".to_string(), "pong".to_string());
            return;
        }

        if let Some(json_payload) = &websocket_text.json_payload {
            let payload_summary: std::collections::HashMap<String, String> = [
                ("event", json_payload.get("event").and_then(|v| v.as_str())),
                ("op", json_payload.get("op").and_then(|v| v.as_str())),
                (
                    "channel",
                    json_payload
                        .get("arg")
                        .and_then(|v| v.get("channel"))
                        .and_then(|v| v.as_str()),
                ),
                ("code", json_payload.get("code").and_then(|v| v.as_str())),
            ]
            .into_iter()
            .filter_map(|(k, v)| v.map(|s| (k.to_string(), s.to_string())))
            .collect();

            websocket_text.payload_summary = payload_summary;
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        !websocket_text.payload_summary.contains_key("event")
            && !websocket_text.payload_summary.contains_key("op")
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("channel")
            .map(|v| v == "bbo-tbt")
            .unwrap()
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("channel")
            .map(|v| v == "trades")
            .unwrap()
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text.payload_summary.get("channel"),
            Some(c) if c == "orders"
        )
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text.payload_summary.get("channel"),
            Some(c) if c == "fills"
        )
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

        let symbol = json_payload["arg"]["instId"].as_str().unwrap().to_string();

        let data = json_payload.get("data").and_then(|v| v.as_array()).unwrap();

        let item = data.first().unwrap();

        let ts = item["ts"].as_str().unwrap().parse::<i64>().unwrap();

        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ts);

        let (bid_price, bid_size) = item
            .get("bids")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|best| Some((best.get(0)?.as_str()?, best.get(1)?.as_str()?)))
            .map(|(p, s)| (p.to_string(), s.to_string()))
            .unwrap_or_else(|| (String::new(), String::new()));

        let (ask_price, ask_size) = item
            .get("asks")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|best| Some((best.get(0)?.as_str()?, best.get(1)?.as_str()?)))
            .map(|(p, s)| (p.to_string(), s.to_string()))
            .unwrap_or_else(|| (String::new(), String::new()));

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Okx(
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

        let symbol = json_payload["arg"]["instId"].as_str().unwrap().to_string();

        let data_array = json_payload.get("data").and_then(|v| v.as_array()).unwrap();

        let trades: Vec<crate::types::Trade> = data_array
            .iter()
            .map(|data| {
                let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
                    data["ts"].as_str().unwrap().parse::<i64>().unwrap(),
                );

                let price = data["px"].as_str().unwrap().to_string();

                let size = data["sz"].as_str().unwrap().to_string();

                let side = match data.get("side").and_then(|v| v.as_str()) {
                    Some("buy") => crate::types::TakerSide::Buy,
                    Some("sell") => crate::types::TakerSide::Sell,
                    _ => crate::types::TakerSide::Unknown,
                };

                crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Okx(
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
                    crate::types::ExchangeInstrumentType::Okx(self.instrument_type);

                let symbol = data["instId"].as_str().unwrap().to_string();

                let order_id = data["ordId"].as_str().unwrap().to_string();

                let client_order_id = data["clOrdId"].as_str().unwrap().to_string();

                let side = match data.get("side").and_then(|v| v.as_str()) {
                    Some("buy") => crate::types::OrderSide::Buy,
                    Some("sell") => crate::types::OrderSide::Sell,
                    _ => crate::types::OrderSide::Unknown,
                };

                let price = data["fillPx"].as_str().unwrap().to_string();

                let quantity = data["fillSz"].as_str().unwrap().to_string();

                let is_maker = data.get("execType").and_then(|v| v.as_str()) == Some("M");

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

        matches!(payload_summary.get("op"), Some(op) if op == "pong")
            || (matches!(payload_summary.get("event"), Some(e) if e == "login")
                && matches!(payload_summary.get("code"), Some(c) if c == "0"))
            || matches!(payload_summary.get("event"), Some(e) if e == "subscribe")
            || matches!(payload_summary.get("event"), Some(e) if e == "channel-conn-count")
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;

        matches!(payload_summary.get("event"), Some(e) if e == "login")
            && matches!(payload_summary.get("code"), Some(c) if c == "0")
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;

        matches!(payload_summary.get("event"), Some(e) if e == "subscribe")
            || matches!(payload_summary.get("event"), Some(e) if e == "channel-conn-count")
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;

        matches!(payload_summary.get("op"), Some(op) if op == "pong")
    }

    fn create_authenticate_websocket_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        crate::exchange_client::common::Response::Authenticate(
            crate::exchange_client::common::AuthenticateResponse { id: None },
        )
    }

    fn create_subscribe_websocket_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        crate::exchange_client::common::Response::Subscribe(
            crate::exchange_client::common::SubscribeResponse {
                id: None,
                ..Default::default()
            },
        )
    }

    fn create_heartbeat_websocket_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        crate::exchange_client::common::Response::Heartbeat(
            crate::exchange_client::common::HeartbeatResponse { id: None },
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

        new_websocket_text.error_code = json_payload
            .get("code")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        new_websocket_text.error_message = json_payload
            .get("msg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_create_subscribe_fill_websocket_subscription_data() {
        let json_payload = serde_json::json!({
            "arg": {
                "channel": "fills",
                "instId": "BTC-USDT-SWAP",
                "uid": "614488474791111"
            },
            "data":[
                {
                    "instId": "BTC-USDT-SWAP",
                    "fillSz": "100",
                    "fillPx": "70000",
                    "side": "buy",
                    "ts": "1705449605015",
                    "ordId": "680800019749904384",
                    "clOrdId": "1234567890",
                    "tradeId": "12345",
                    "execType": "T",
                    "count": "10"
                }
            ]
        });

        let websocket_text = crate::networking::websocket::WebSocketText {
            json_payload: Some(json_payload),
            ..Default::default()
        };

        let instrument_type = crate::types::OkxInstrumentType::Swap;
        let client = crate::exchanges::okx::common::OkxClient {
            instrument_type,
            ..Default::default()
        };

        let response = crate::exchange_client::websocket::Websocket::create_subscribe_fill_websocket_subscription_data(&client, &websocket_text);

        match response {
            crate::exchange_client::common::Response::FillSubscription(data) => {
                assert_eq!(data.data.len(), 1);

                let fill = &data.data[0];

                assert_eq!(
                    fill.exchange_instrument_type,
                    crate::types::ExchangeInstrumentType::Okx(instrument_type)
                );
                assert_eq!(fill.order_id, "680800019749904384");
                assert_eq!(fill.client_order_id, "1234567890");
                assert_eq!(fill.price, "70000");
                assert_eq!(fill.quantity, "100");
                assert!(matches!(fill.side, crate::types::OrderSide::Buy));
                assert!(!fill.is_maker);
            }
            _ => panic!(),
        }
    }
}
