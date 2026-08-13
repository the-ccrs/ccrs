#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::polymarket::common::PolymarketClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::Polymarket(polymarket_endpoint) => {
                match polymarket_endpoint {
                    crate::types::PolymarketWebSocketEndpoint::MarketData => {
                        self.market_websocket_url.clone()
                    }
                    crate::types::PolymarketWebSocketEndpoint::AccountData => {
                        self.user_websocket_url.clone()
                    }
                    crate::types::PolymarketWebSocketEndpoint::Unknown => {
                        panic!("Polymarket endpoint is Unknown")
                    }
                }
            }
            _ => {
                panic!("WebSocket endpoint is not Polymarket")
            }
        }
    }

    fn create_authenticate_websocket_request(&self) -> String {
        String::new()
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(move || "PING".to_string())
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let assets_ids: Vec<&String> = subscribe_top_of_book_request.symbols.iter().collect();
        serde_json::json!({
            "assets_ids": assets_ids,
            "type": "market",
            "custom_feature_enabled": true
        })
        .to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let assets_ids: Vec<&String> = subscribe_trade_request.symbols.iter().collect();
        serde_json::json!({
            "assets_ids": assets_ids,
            "type": "market"
        })
        .to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        let credential = self
            .credential
            .as_ref()
            .expect("Credential required for order subscription");
        let markets: Vec<&String> = subscribe_order_request.symbols.iter().collect();
        serde_json::json!({
            "auth": {
                "apiKey": credential.api_key,
                "secret": credential.api_secret,
                "passphrase": credential.api_passphrase
            },
            "markets": markets,
            "type": "user"
        })
        .to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        let credential = self
            .credential
            .as_ref()
            .expect("Credential required for fill subscription");
        let markets: Vec<&String> = subscribe_fill_request.symbols.iter().collect();
        serde_json::json!({
            "auth": {
                "apiKey": credential.api_key,
                "secret": credential.api_secret,
                "passphrase": credential.api_passphrase
            },
            "markets": markets,
            "type": "user"
        })
        .to_string()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload {
            let event_type = json_payload
                .get("event_type")
                .or_else(|| json_payload.get(0).and_then(|obj| obj.get("event_type")));

            if let Some(v) = event_type {
                let value_str = v.as_str().map_or_else(|| v.to_string(), str::to_string);
                websocket_text
                    .payload_summary
                    .insert("event_type".to_string(), value_str);
            }
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("event_type")
            .map(|event_type| {
                event_type == "best_bid_ask"
                    || event_type == "last_trade_price"
                    || event_type == "order"
                    || event_type == "trade"
                    || event_type == "book"
                    || event_type == "price_change"
                    || event_type == "tick_size_change"
                    || event_type == "new_market"
                    || event_type == "market_resolved"
            })
            .unwrap_or(false)
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("event_type")
            .map(|event_type| event_type == "best_bid_ask")
            .unwrap_or(false)
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("event_type")
            .map(|event_type| event_type == "last_trade_price")
            .unwrap_or(false)
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("event_type")
            .map(|event_type| event_type == "order")
            .unwrap_or(false)
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("event_type")
            .map(|event_type| event_type == "trade")
            .unwrap_or(false)
    }

    fn is_websocket_text_unneeded_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("event_type")
            .map(|event_type| {
                event_type == "book"
                    || event_type == "price_change"
                    || event_type == "tick_size_change"
                    || event_type == "new_market"
                    || event_type == "market_resolved"
            })
            .unwrap_or(false)
    }

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.text == "PONG"
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.text == "PONG"
    }

    fn create_subscribe_top_of_book_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let symbol = json_payload
            .get("asset_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let timestamp_ms: i64 = json_payload
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let timestamp =
            crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(timestamp_ms);

        let bid_price = json_payload
            .get("best_bid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let ask_price = json_payload
            .get("best_ask")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Polymarket,
                    symbol,
                    timestamp,
                    bid_price,
                    bid_size: String::new(),
                    ask_price,
                    ask_size: String::new(),
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
            .get("asset_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let timestamp_ms: i64 = json_payload
            .get("timestamp")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let timestamp =
            crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(timestamp_ms);

        let price = json_payload
            .get("price")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let size = json_payload
            .get("size")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let side = self.convert_side_str_to_taker_side(
            json_payload
                .get("side")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );

        crate::exchange_client::common::Response::TradeSubscription(
            crate::exchange_client::common::TradeSubscriptionData {
                data: vec![crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Polymarket,
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

        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData {
                data: vec![self.convert_json_value_to_order(json_payload)],
            },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let order_id = json_payload
            .get("taker_order_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let symbol = json_payload
            .get("asset_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let side = self.convert_side_str_to_order_side(
            json_payload
                .get("side")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );

        let price = json_payload
            .get("price")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let quantity = json_payload
            .get("size")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let fee_rate_bps = json_payload
            .get("fee_rate_bps")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let is_maker = json_payload
            .get("trader_side")
            .and_then(|v| v.as_str())
            .map(|s| s == "MAKER")
            .unwrap_or(false);

        let status = self.convert_fill_status_str_to_fill_status(
            json_payload
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
        );

        crate::exchange_client::common::Response::FillSubscription(
            crate::exchange_client::common::FillSubscriptionData {
                data: vec![crate::types::Fill {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Polymarket,
                    symbol,
                    order_id,
                    client_order_id: String::new(),
                    side,
                    price,
                    quantity,
                    quote_quantity: fee_rate_bps,
                    is_maker,
                    status,
                }],
            },
        )
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
                kind: None,
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

        new_websocket_text.error_message = json_payload
            .get("error")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| {
                json_payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            });

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
