#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::kraken_derivatives::common::KrakenDerivativesClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::KrakenDerivatives(kraken_endpoint) => {
                match kraken_endpoint {
                    crate::types::KrakenDerivativesWebSocketEndpoint::MarketData => {
                        self.websocket_api_url.clone()
                    }
                    crate::types::KrakenDerivativesWebSocketEndpoint::AccountData => {
                        self.websocket_api_url.clone()
                    }
                    crate::types::KrakenDerivativesWebSocketEndpoint::Unknown => {
                        panic!("KrakenDerivatives websocket endpoint is Unknown")
                    }
                }
            }
            _ => {
                panic!("Websocket endpoint is not KrakenDerivatives")
            }
        }
    }

    fn create_authenticate_websocket_request(&self) -> String {
        let credential = match &self.credential {
            Some(c) => c,
            None => return String::new(),
        };
        serde_json::json!({
            "event": "challenge",
            "api_key": credential.api_key
        })
        .to_string()
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(String::new)
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "event".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        payload.insert(
            "feed".to_string(),
            serde_json::Value::String("ticker_lite".to_string()),
        );
        payload.insert(
            "product_ids".to_string(),
            serde_json::to_value(&subscribe_top_of_book_request.symbols).unwrap(),
        );
        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "event".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        payload.insert(
            "feed".to_string(),
            serde_json::Value::String("trade".to_string()),
        );
        payload.insert(
            "product_ids".to_string(),
            serde_json::to_value(&subscribe_trade_request.symbols).unwrap(),
        );
        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        _subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "event".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        payload.insert(
            "feed".to_string(),
            serde_json::Value::String("open_orders".to_string()),
        );
        if let Some(credential) = &self.credential {
            payload.insert(
                "api_key".to_string(),
                serde_json::Value::String(credential.api_key.clone()),
            );
            let original = self
                .original_challenge
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_default();
            let signed = self
                .signed_challenge
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_default();
            payload.insert(
                "original_challenge".to_string(),
                serde_json::Value::String(original),
            );
            payload.insert(
                "signed_challenge".to_string(),
                serde_json::Value::String(signed),
            );
        }
        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "event".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        payload.insert(
            "feed".to_string(),
            serde_json::Value::String("fills".to_string()),
        );
        if let Some(credential) = &self.credential {
            payload.insert(
                "api_key".to_string(),
                serde_json::Value::String(credential.api_key.clone()),
            );
            let original = self
                .original_challenge
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_default();
            let signed = self
                .signed_challenge
                .lock()
                .unwrap()
                .clone()
                .unwrap_or_default();
            payload.insert(
                "original_challenge".to_string(),
                serde_json::Value::String(original),
            );
            payload.insert(
                "signed_challenge".to_string(),
                serde_json::Value::String(signed),
            );
        }
        if !subscribe_fill_request.symbols.is_empty() {
            payload.insert(
                "product_ids".to_string(),
                serde_json::to_value(&subscribe_fill_request.symbols).unwrap(),
            );
        }
        serde_json::Value::Object(payload).to_string()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload {
            if let Some(s) = json_payload.get("event").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("event".to_string(), s.to_string());
            }
            if let Some(s) = json_payload.get("feed").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("feed".to_string(), s.to_string());
            }
            if let Some(s) = json_payload.get("api_key").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("api_key".to_string(), s.to_string());
            }
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary.contains_key("feed")
            && !payload_summary.contains_key("event")
            && payload_summary.get("feed").map(String::as_str) != Some("heartbeat")
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("feed")
            .map(String::as_str)
            == Some("ticker_lite")
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text
                .payload_summary
                .get("feed")
                .map(String::as_str),
            Some("trade") | Some("trade_snapshot")
        )
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text
                .payload_summary
                .get("feed")
                .map(String::as_str),
            Some("open_orders") | Some("open_orders_snapshot")
        )
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text
                .payload_summary
                .get("feed")
                .map(String::as_str),
            Some("fills") | Some("fills_snapshot")
        )
    }

    fn is_websocket_text_unneeded_subscription_data(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
    }

    fn create_subscribe_top_of_book_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let symbol = json_payload
            .get("product_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let bid_price = json_payload
            .get("bid")
            .map(|v| v.to_string())
            .unwrap_or_default();

        let ask_price = json_payload
            .get("ask")
            .map(|v| v.to_string())
            .unwrap_or_default();

        let top_of_book = crate::types::TopOfBook {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenDerivatives,
            symbol,
            timestamp: chrono::Utc::now(),
            bid_price,
            bid_size: String::new(),
            ask_price,
            ask_size: String::new(),
        };

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![top_of_book],
            },
        )
    }

    fn create_subscribe_trade_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let feed = json_payload
            .get("feed")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let parse_trade = |obj: &serde_json::Value, fallback_symbol: &str| -> crate::types::Trade {
            let symbol = obj
                .get("product_id")
                .and_then(|v| v.as_str())
                .unwrap_or(fallback_symbol)
                .to_string();

            let time_ms = obj.get("time").and_then(|v| v.as_i64()).unwrap_or(0);
            let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(time_ms)
                .unwrap_or_else(chrono::Utc::now);

            let price = obj.get("price").map(|v| v.to_string()).unwrap_or_default();
            let size = obj.get("qty").map(|v| v.to_string()).unwrap_or_default();

            let side = match obj.get("side").and_then(|v| v.as_str()) {
                Some("buy") => crate::types::TakerSide::Buy,
                Some("sell") => crate::types::TakerSide::Sell,
                _ => crate::types::TakerSide::Unknown,
            };

            crate::types::Trade {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenDerivatives,
                symbol,
                timestamp,
                price,
                size,
                side,
            }
        };

        let top_level_symbol = json_payload
            .get("product_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let trades = if feed == "trade_snapshot" {
            json_payload
                .get("trades")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|t| parse_trade(t, &top_level_symbol))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            vec![parse_trade(json_payload, &top_level_symbol)]
        };

        crate::exchange_client::common::Response::TradeSubscription(
            crate::exchange_client::common::TradeSubscriptionData { data: trades },
        )
    }

    fn create_subscribe_order_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();
        let feed = json_payload
            .get("feed")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let orders: Vec<crate::types::Order> = if feed == "open_orders_snapshot" {
            json_payload
                .get("orders")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|o| self.convert_ws_order_json_to_order(o, false))
                        .collect()
                })
                .unwrap_or_default()
        } else if let Some(order_json) = json_payload.get("order") {
            let is_cancel = json_payload
                .get("is_cancel")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            vec![self.convert_ws_order_json_to_order(order_json, is_cancel)]
        } else if json_payload
            .get("is_cancel")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            vec![self.convert_ws_cancel_json_to_order(json_payload)]
        } else {
            vec![]
        };

        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData { data: orders },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let fills: Vec<crate::types::Fill> = json_payload
            .get("fills")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|f| self.convert_ws_fill_json_to_fill(f))
                    .collect()
            })
            .unwrap_or_default();

        crate::exchange_client::common::Response::FillSubscription(
            crate::exchange_client::common::FillSubscriptionData { data: fills },
        )
    }

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        let event = payload_summary.get("event").map(String::as_str);
        let feed = payload_summary.get("feed").map(String::as_str);
        matches!(event, Some("challenge") | Some("subscribed") | Some("info"))
            || (event.is_none() && feed == Some("heartbeat"))
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("event")
            .map(String::as_str)
            == Some("challenge")
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("event")
            .map(String::as_str)
            == Some("subscribed")
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        let event = payload_summary.get("event").map(String::as_str);
        let feed = payload_summary.get("feed").map(String::as_str);
        event == Some("info") || (event.is_none() && feed == Some("heartbeat"))
    }

    fn create_subscribe_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let symbols: Vec<String> = json_payload
            .get("product_ids")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        let feed = json_payload.get("feed").and_then(|v| v.as_str());
        let kind = match feed {
            Some("ticker_lite") => {
                Some(crate::exchange_client::common::SubscribeResponseKind::TopOfBook)
            }
            Some("trade") | Some("trade_snapshot") => {
                Some(crate::exchange_client::common::SubscribeResponseKind::Trade)
            }
            Some("open_orders") => {
                Some(crate::exchange_client::common::SubscribeResponseKind::Order)
            }
            Some("fills") => Some(crate::exchange_client::common::SubscribeResponseKind::Fill),
            _ => None,
        };

        crate::exchange_client::common::Response::Subscribe(
            crate::exchange_client::common::SubscribeResponse {
                symbols,
                kind,
                ..Default::default()
            },
        )
    }

    fn create_authenticate_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();
        let challenge = json_payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if let Some(credential) = &self.credential {
            let mut sha256_hasher = sha2::Sha256::default();
            <sha2::Sha256 as sha2::Digest>::update(&mut sha256_hasher, challenge.as_bytes());
            let sha256_hash = <sha2::Sha256 as sha2::Digest>::finalize(sha256_hasher);

            if let Ok(decoded_secret) = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                credential.api_secret.as_bytes(),
            ) && let Ok(mut mac) =
                <hmac::Hmac<sha2::Sha512> as hmac::digest::KeyInit>::new_from_slice(&decoded_secret)
            {
                <hmac::Hmac<sha2::Sha512> as hmac::Mac>::update(&mut mac, &sha256_hash);
                let signature_bytes =
                    <hmac::Hmac<sha2::Sha512> as hmac::Mac>::finalize(mac).into_bytes();
                let signed = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    signature_bytes,
                );
                *self.original_challenge.lock().unwrap() = Some(challenge.to_string());
                *self.signed_challenge.lock().unwrap() = Some(signed);
            }
        }

        crate::exchange_client::common::Response::Authenticate(
            crate::exchange_client::common::AuthenticateResponse { id: None },
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
        new_websocket_text.error_message = json_payload
            .get("message")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
