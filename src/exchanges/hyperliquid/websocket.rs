#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::hyperliquid::common::HyperliquidClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::Hyperliquid(hyperliquid_endpoint) => {
                match hyperliquid_endpoint {
                    crate::types::HyperliquidWebSocketEndpoint::MarketData => {
                        self.websocket_url.clone()
                    }
                    crate::types::HyperliquidWebSocketEndpoint::AccountData => {
                        self.websocket_url.clone()
                    }
                    crate::types::HyperliquidWebSocketEndpoint::Unknown => {
                        panic!("Hyperliquid endpoint is Unknown")
                    }
                }
            }
            _ => {
                panic!("WebSocket endpoint is not Hyperliquid")
            }
        }
    }

    fn create_authenticate_websocket_request(&self) -> String {
        String::new()
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(move || {
            serde_json::json!({
                "method": "ping"
            })
            .to_string()
        })
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let coin = subscribe_top_of_book_request
            .symbols
            .first()
            .cloned()
            .unwrap_or_default();

        serde_json::json!({
            "method": "subscribe",
            "subscription": {
                "type": "bbo",
                "coin": coin
            }
        })
        .to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let coin = subscribe_trade_request
            .symbols
            .first()
            .cloned()
            .unwrap_or_default();

        serde_json::json!({
            "method": "subscribe",
            "subscription": {
                "type": "trades",
                "coin": coin
            }
        })
        .to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        _subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        let user = &self.wallet_address;

        serde_json::json!({
            "method": "subscribe",
            "subscription": {
                "type": "orderUpdates",
                "user": user
            }
        })
        .to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        _subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        let user = &self.wallet_address;

        serde_json::json!({
            "method": "subscribe",
            "subscription": {
                "type": "userFills",
                "user": user
            }
        })
        .to_string()
    }

    async fn send_websocket_request(
        &self,
        websocket_sender: &crate::networking::websocket::WebSocketSender,
        request: crate::exchange_client::common::Request,
    ) -> crate::exchange_client::common::Response {
        match &request {
            crate::exchange_client::common::Request::SubscribeTopOfBook(
                subscribe_top_of_book_request,
            ) => {
                for symbol in &subscribe_top_of_book_request.symbols {
                    let msg = serde_json::json!({
                        "method": "subscribe",
                        "subscription": {
                            "type": "bbo",
                            "coin": symbol
                        }
                    })
                    .to_string();

                    crate::fine!("=== WebSocket REQUEST ===");
                    crate::fine!("{} {}", websocket_sender.url(), msg);

                    if let Err(err) = websocket_sender.send(msg).await {
                        return crate::exchange_client::common::Response::WebSocketWriteError(err);
                    }
                }
                crate::exchange_client::common::Response::None
            }
            crate::exchange_client::common::Request::SubscribeTrade(subscribe_trade_request) => {
                for symbol in &subscribe_trade_request.symbols {
                    let msg = serde_json::json!({
                        "method": "subscribe",
                        "subscription": {
                            "type": "trades",
                            "coin": symbol
                        }
                    })
                    .to_string();

                    crate::fine!("=== WebSocket REQUEST ===");
                    crate::fine!("{} {}", websocket_sender.url(), msg);

                    if let Err(err) = websocket_sender.send(msg).await {
                        return crate::exchange_client::common::Response::WebSocketWriteError(err);
                    }
                }
                crate::exchange_client::common::Response::None
            }
            crate::exchange_client::common::Request::SubscribeOrder(_subscribe_order_request) => {
                let msg = self.create_subscribe_order_websocket_request(_subscribe_order_request);

                crate::fine!("=== WebSocket REQUEST ===");
                crate::fine!("{} {}", websocket_sender.url(), msg);

                if let Err(err) = websocket_sender.send(msg).await {
                    return crate::exchange_client::common::Response::WebSocketWriteError(err);
                }
                crate::exchange_client::common::Response::None
            }
            crate::exchange_client::common::Request::SubscribeFill(_subscribe_fill_request) => {
                let msg = self.create_subscribe_fill_websocket_request(_subscribe_fill_request);

                crate::fine!("=== WebSocket REQUEST ===");
                crate::fine!("{} {}", websocket_sender.url(), msg);

                if let Err(err) = websocket_sender.send(msg).await {
                    return crate::exchange_client::common::Response::WebSocketWriteError(err);
                }
                crate::exchange_client::common::Response::None
            }
            _ => panic!(),
        }
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload
            && let Some(v) = json_payload.get("channel")
        {
            let value_str = v.as_str().map_or_else(|| v.to_string(), str::to_string);
            websocket_text
                .payload_summary
                .insert("channel".to_string(), value_str);
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("channel")
            .map(|channel| {
                channel == "bbo"
                    || channel == "trades"
                    || channel == "orderUpdates"
                    || channel == "userFills"
            })
            .unwrap_or(false)
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("channel")
            .map(|channel| channel == "bbo")
            .unwrap_or(false)
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("channel")
            .map(|channel| channel == "trades")
            .unwrap_or(false)
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("channel")
            .map(|channel| channel == "orderUpdates")
            .unwrap_or(false)
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("channel")
            .map(|channel| channel == "userFills")
            .unwrap_or(false)
    }

    fn is_unexpected_websocket_text_subscription_data_benign(
        &self,
        _: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
    }

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("channel")
            .map(|channel| channel == "subscriptionResponse" || channel == "pong")
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
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("channel")
            .map(|channel| channel == "subscriptionResponse")
            .unwrap_or(false)
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("channel")
            .map(|channel| channel == "pong")
            .unwrap_or(false)
    }

    fn create_subscribe_top_of_book_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();
        let data = json_payload.get("data").unwrap();

        let symbol = data
            .get("coin")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
            data.get("time").and_then(|v| v.as_i64()).unwrap(),
        );

        let bbo = data.get("bbo").and_then(|v| v.as_array()).unwrap();

        let best_bid = bbo.first().unwrap();
        let best_ask = bbo.get(1).unwrap();

        let bid_price = best_bid
            .get("px")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let bid_size = best_bid
            .get("sz")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let ask_price = best_ask
            .get("px")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let ask_size = best_ask
            .get("sz")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Hyperliquid(
                        self.convert_symbol_to_instrument_type(&symbol),
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

        let data_array = json_payload.get("data").and_then(|v| v.as_array()).unwrap();

        let trades: Vec<crate::types::Trade> = data_array
            .iter()
            .map(|data| {
                let symbol = data
                    .get("coin")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
                    data.get("time").and_then(|v| v.as_i64()).unwrap(),
                );

                let price = data.get("px").and_then(|v| v.as_str()).unwrap().to_string();

                let size = data.get("sz").and_then(|v| v.as_str()).unwrap().to_string();

                let side = match data.get("side").and_then(|v| v.as_str()) {
                    Some("B") => crate::types::TakerSide::Buy,
                    Some("A") => crate::types::TakerSide::Sell,
                    _ => crate::types::TakerSide::Unknown,
                };

                crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Hyperliquid(
                        self.convert_symbol_to_instrument_type(&symbol),
                    ),
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

        let data_array = json_payload.get("data").and_then(|v| v.as_array()).unwrap();

        let orders: Vec<crate::types::Order> = data_array
            .iter()
            .map(|item| {
                let order = item.get("order").unwrap();
                let status_str = item.get("status").and_then(|v| v.as_str()).unwrap_or("");

                let status = match status_str.to_lowercase().as_str() {
                    s if s.ends_with("canceled") => crate::types::OrderStatus::Canceled,
                    s if s.ends_with("rejected") => crate::types::OrderStatus::Rejected,
                    "open" => crate::types::OrderStatus::Open,
                    "filled" => crate::types::OrderStatus::Filled,
                    _ => crate::types::OrderStatus::Unknown,
                };

                let oid = order.get("oid").and_then(|v| v.as_u64()).unwrap_or(0);

                let cloid = order
                    .get("cloid")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let symbol = order
                    .get("coin")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let side = self.convert_side_str_to_order_side(
                    order.get("side").and_then(|v| v.as_str()).unwrap_or(""),
                );

                let price = order
                    .get("limitPx")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let quantity = order
                    .get("origSz")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let remaining_quantity = order
                    .get("sz")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                crate::types::Order {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Hyperliquid(
                        self.convert_symbol_to_instrument_type(&symbol),
                    ),
                    symbol,
                    order_id: oid.to_string(),
                    client_order_id: cloid,
                    order_type: crate::types::OrderType::Limit,
                    side,
                    price,
                    quantity,
                    remaining_quantity,
                    status,
                    ..Default::default()
                }
            })
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

        let data = json_payload.get("data").unwrap();

        let fills_array = data.get("fills").and_then(|v| v.as_array()).unwrap();

        let fills: Vec<crate::types::Fill> = fills_array
            .iter()
            .map(|fill| {
                let symbol = fill
                    .get("coin")
                    .and_then(|v| v.as_str())
                    .unwrap()
                    .to_string();

                let order_id = fill
                    .get("oid")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .to_string();

                let client_order_id = fill
                    .get("cloid")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let side = match fill.get("side").and_then(|v| v.as_str()) {
                    Some("B") => crate::types::OrderSide::Buy,
                    Some("A") => crate::types::OrderSide::Sell,
                    _ => crate::types::OrderSide::Unknown,
                };

                let price = fill.get("px").and_then(|v| v.as_str()).unwrap().to_string();

                let quantity = fill.get("sz").and_then(|v| v.as_str()).unwrap().to_string();

                let fee = fill
                    .get("fee")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let crossed = fill
                    .get("crossed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                crate::types::Fill {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Hyperliquid(
                        self.convert_symbol_to_instrument_type(&symbol),
                    ),
                    symbol,
                    order_id,
                    client_order_id,
                    side,
                    price,
                    quantity,
                    quote_quantity: fee,
                    is_maker: !crossed,
                }
            })
            .collect();

        crate::exchange_client::common::Response::FillSubscription(
            crate::exchange_client::common::FillSubscriptionData { data: fills },
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
        let json_payload = _websocket_text.json_payload.as_ref().unwrap();

        let kind = json_payload["data"]["subscription"]["type"]
            .as_str()
            .and_then(|t| match t {
                "orderUpdates" => {
                    Some(crate::exchange_client::common::SubscribeResponseKind::Order)
                }
                "userFills" => Some(crate::exchange_client::common::SubscribeResponseKind::Fill),
                _ => None,
            });

        crate::exchange_client::common::Response::Subscribe(
            crate::exchange_client::common::SubscribeResponse {
                id: None,
                kind,
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
            .get("data")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| {
                json_payload
                    .get("error")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            });

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
