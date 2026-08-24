static NEXT_MSG_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::kalshi_perps::common::KalshiPerpsClient
{
    async fn create_websocket_client(
        &self,
        websocket_client_config: crate::types::WebSocketClientConfig,
        websocket_config: crate::networking::websocket::WebSocketConfig,
    ) -> anyhow::Result<crate::networking::websocket::WebSocketClient> {
        let url = self.websocket_api_url(websocket_client_config.endpoint);

        let headers = match &self.credential {
            Some(credential) => {
                let timestamp = chrono::Utc::now().timestamp_millis().to_string();
                let signature = self.build_signature(&timestamp, "GET", "/trade-api/ws/v2");
                vec![
                    ("kalshi-access-key".to_string(), credential.api_key.clone()),
                    ("kalshi-access-timestamp".to_string(), timestamp),
                    ("kalshi-access-signature".to_string(), signature),
                ]
            }
            None => vec![],
        };

        let websocket_client = crate::networking::websocket::WebSocketClient::builder(
            url,
            websocket_config.clone(),
            Some(headers),
        )
        .build()
        .await?;

        self.keep_websocket_client_alive(
            websocket_config.heartbeat_interval_secs,
            websocket_client.sender(),
            websocket_client.cancellation_token().clone(),
        )
        .await?;

        crate::finer!("Created websocket_client: {:#?}", websocket_client);

        Ok(websocket_client)
    }

    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::KalshiPerps(_) => self.websocket_api_url.clone(),
            _ => panic!("Websocket endpoint is not KalshiPerps"),
        }
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(|| "".to_string())
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let id = NEXT_MSG_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let market_tickers: Vec<serde_json::Value> = subscribe_top_of_book_request
            .symbols
            .iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect();
        serde_json::json!({
            "id": id,
            "cmd": "subscribe",
            "params": {
                "channels": ["ticker"],
                "market_tickers": market_tickers
            }
        })
        .to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let id = NEXT_MSG_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let market_tickers: Vec<serde_json::Value> = subscribe_trade_request
            .symbols
            .iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect();
        serde_json::json!({
            "id": id,
            "cmd": "subscribe",
            "params": {
                "channels": ["trade"],
                "market_tickers": market_tickers
            }
        })
        .to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        _subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        let id = NEXT_MSG_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        serde_json::json!({
            "id": id,
            "cmd": "subscribe",
            "params": {
                "channels": ["user_orders"]
            }
        })
        .to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        _subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        let id = NEXT_MSG_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        serde_json::json!({
            "id": id,
            "cmd": "subscribe",
            "params": {
                "channels": ["fill"]
            }
        })
        .to_string()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload
            && let Some(type_val) = json_payload.get("type")
        {
            if let Some(s) = type_val.as_str() {
                websocket_text
                    .payload_summary
                    .insert("type".to_string(), s.to_string());
            } else {
                websocket_text
                    .payload_summary
                    .insert("type".to_string(), type_val.to_string());
            }
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text
                .payload_summary
                .get("type")
                .map(String::as_str),
            Some("ticker") | Some("trade") | Some("user_order") | Some("fill")
        )
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("type")
            .map(|v| v == "ticker")
            .unwrap_or(false)
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("type")
            .map(|v| v == "trade")
            .unwrap_or(false)
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("type")
            .map(|v| v == "user_order")
            .unwrap_or(false)
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("type")
            .map(|v| v == "fill")
            .unwrap_or(false)
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
        let msg = &json_payload["msg"];
        let symbol = msg["market_ticker"].as_str().unwrap_or("").to_string();
        let ts_ms = msg.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);
        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ts_ms);
        let bid_price = msg["bid"].as_str().unwrap_or("").to_string();
        let bid_size = msg["bid_size"].as_str().unwrap_or("").to_string();
        let ask_price = msg["ask"].as_str().unwrap_or("").to_string();
        let ask_size = msg["ask_size"].as_str().unwrap_or("").to_string();
        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::KalshiPerps,
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
        let msg = &json_payload["msg"];
        let symbol = msg["market_ticker"].as_str().unwrap_or("").to_string();
        let ts_ms = msg.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);
        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ts_ms);
        let price = msg["price"].as_str().unwrap_or("").to_string();
        let size = msg["count"].as_str().unwrap_or("").to_string();
        let side = match msg.get("taker_side").and_then(|v| v.as_str()) {
            Some("bid") => crate::types::TakerSide::Buy,
            Some("ask") => crate::types::TakerSide::Sell,
            _ => crate::types::TakerSide::Unknown,
        };
        crate::exchange_client::common::Response::TradeSubscription(
            crate::exchange_client::common::TradeSubscriptionData {
                data: vec![crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::KalshiPerps,
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
        let msg = &json_payload["msg"];
        let order = self.convert_json_value_to_order(msg);
        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData { data: vec![order] },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();
        let msg = &json_payload["msg"];
        let symbol = msg["market_ticker"].as_str().unwrap_or("").to_string();
        let order_id = msg["order_id"].as_str().unwrap_or("").to_string();
        let client_order_id = msg
            .get("client_order_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let side = match msg.get("side").and_then(|v| v.as_str()) {
            Some("bid") => crate::types::OrderSide::Buy,
            Some("ask") => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        };
        let price = msg["price"].as_str().unwrap_or("").to_string();
        let quantity = msg["count"].as_str().unwrap_or("").to_string();
        let is_maker = msg
            .get("is_taker")
            .and_then(|v| v.as_bool())
            .map(|is_taker| !is_taker)
            .unwrap_or(false);
        crate::exchange_client::common::Response::FillSubscription(
            crate::exchange_client::common::FillSubscriptionData {
                data: vec![crate::types::Fill {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::KalshiPerps,
                    symbol,
                    order_id,
                    client_order_id,
                    side,
                    price,
                    quantity,
                    is_maker,
                    ..Default::default()
                }],
            },
        )
    }

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text
                .payload_summary
                .get("type")
                .map(String::as_str),
            Some("subscribed") | Some("ok")
        )
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text
                .payload_summary
                .get("type")
                .map(String::as_str),
            Some("subscribed") | Some("ok")
        )
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
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
            .get("msg")
            .and_then(|m| m.get("code"))
            .map(|v| v.to_string());

        new_websocket_text.error_message = json_payload
            .get("msg")
            .and_then(|m| m.get("msg"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
