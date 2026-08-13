#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::coinbase::common::CoinbaseClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::Coinbase(coinbase_endpoint) => match coinbase_endpoint
            {
                crate::types::CoinbaseWebSocketEndpoint::MarketData => {
                    self.websocket_market_data_api_url.clone()
                }
                crate::types::CoinbaseWebSocketEndpoint::AccountData => {
                    self.websocket_account_data_api_url.clone()
                }
                crate::types::CoinbaseWebSocketEndpoint::Unknown => {
                    panic!("Coinbase WebSocket endpoint is Unknown")
                }
            },
            _ => panic!("WebSocket endpoint is not Coinbase"),
        }
    }

    fn create_authenticate_websocket_request(&self) -> String {
        String::new()
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(|| "".to_string())
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        self.build_subscribe_message("ticker", &subscribe_top_of_book_request.symbols, false)
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        self.build_subscribe_message("matches", &subscribe_trade_request.symbols, false)
    }

    fn create_subscribe_order_websocket_request(
        &self,
        subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        self.build_subscribe_message("user", &subscribe_order_request.symbols, true)
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
            for key in &["type", "taker_user_id", "maker_user_id"] {
                if let Some(v) = json_payload.get(*key) {
                    let value_str = v.as_str().map_or_else(|| v.to_string(), str::to_string);
                    websocket_text
                        .payload_summary
                        .insert(key.to_string(), value_str);
                }
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
                .map(|s| s.as_str()),
            Some("ticker")
                | Some("match")
                | Some("last_match")
                | Some("received")
                | Some("open")
                | Some("done")
                | Some("change")
                | Some("activate")
        )
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text
                .payload_summary
                .get("type")
                .map(|s| s.as_str()),
            Some("ticker")
        )
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let is_match_type = matches!(
            websocket_text
                .payload_summary
                .get("type")
                .map(|s| s.as_str()),
            Some("match") | Some("last_match")
        );

        let has_user_ids = websocket_text.payload_summary.contains_key("taker_user_id")
            || websocket_text.payload_summary.contains_key("maker_user_id");

        is_match_type && !has_user_ids
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let summary = &websocket_text.payload_summary;

        let is_order_type = matches!(
            summary.get("type").map(|s| s.as_str()),
            Some("received") | Some("open") | Some("done") | Some("change") | Some("activate")
        );

        let is_user_match = matches!(
            summary.get("type").map(|s| s.as_str()),
            Some("match") | Some("last_match")
        ) && (summary.contains_key("taker_user_id")
            || summary.contains_key("maker_user_id"));

        is_order_type || is_user_match
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
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

        let symbol = json_payload["product_id"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let timestamp = json_payload["time"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let bid_price = json_payload["best_bid"].as_str().unwrap_or("").to_string();
        let bid_size = json_payload["best_bid_size"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let ask_price = json_payload["best_ask"].as_str().unwrap_or("").to_string();
        let ask_size = json_payload["best_ask_size"]
            .as_str()
            .unwrap_or("")
            .to_string();

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
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

        let symbol = json_payload["product_id"].as_str().unwrap().to_string();

        let timestamp =
            chrono::DateTime::parse_from_rfc3339(json_payload["time"].as_str().unwrap())
                .unwrap()
                .with_timezone(&chrono::Utc);

        let price = json_payload["price"].as_str().unwrap().to_string();
        let size = json_payload["size"].as_str().unwrap().to_string();

        let maker_side = json_payload["side"].as_str().unwrap_or("");
        let taker_side = if maker_side == "sell" {
            crate::types::TakerSide::Buy
        } else {
            crate::types::TakerSide::Sell
        };

        crate::exchange_client::common::Response::TradeSubscription(
            crate::exchange_client::common::TradeSubscriptionData {
                data: vec![crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
                    symbol,
                    timestamp,
                    price,
                    size,
                    side: taker_side,
                }],
            },
        )
    }

    fn create_subscribe_order_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let order = self.convert_ws_user_message_to_order(json_payload);

        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData { data: vec![order] },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        panic!()
    }

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("type")
            .map(|t| t == "subscriptions")
            .unwrap_or(false)
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
        websocket_text
            .payload_summary
            .get("type")
            .map(|t| t == "subscriptions")
            .unwrap_or(false)
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
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

    fn create_authenticate_websocket_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
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

        new_websocket_text.error_code = json_payload
            .get("reason")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
