#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::bitget::common::BitgetClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::Bitget(bitget_endpoint) => match bitget_endpoint {
                crate::types::BitgetWebSocketEndpoint::MarketData => {
                    self.websocket_market_data_api_url.clone()
                }
                crate::types::BitgetWebSocketEndpoint::AccountData => {
                    self.websocket_account_data_api_url.clone()
                }
                crate::types::BitgetWebSocketEndpoint::Unknown => {
                    panic!("Bitget endpoint is Unknown")
                }
            },
            _ => panic!("Unexpected WebSocketEndpoint variant for Bitget"),
        }
    }

    fn create_authenticate_websocket_request(&self) -> String {
        let credential = match &self.credential {
            Some(c) => c,
            None => return String::new(),
        };

        let timestamp = chrono::Utc::now().timestamp().to_string();

        let prehash = format!("{}GET/user/verify", timestamp);

        let mut mac = hmac_sha256::HMAC::new(credential.api_secret.as_bytes());
        mac.update(prehash.as_bytes());

        let signature_bytes = mac.finalize();

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
                    "instType": self.category.to_lowercase(),
                    "topic": "books1",
                    "symbol": symbol
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
                    "instType": self.category.to_lowercase(),
                    "topic": "publicTrade",
                    "symbol": symbol
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
                "instType": "UTA",
                "topic": "order"
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
                "instType": "UTA",
                "topic": "fill"
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
            if let Some(v) = json_payload.get("event") {
                let s = if let Some(s) = v.as_str() {
                    s.to_string()
                } else {
                    v.to_string()
                };
                websocket_text
                    .payload_summary
                    .insert("event".to_string(), s);
            }

            if let Some(arg) = json_payload.get("arg") {
                if let Some(v) = arg.get("instType") {
                    let s = if let Some(s) = v.as_str() {
                        s.to_string()
                    } else {
                        v.to_string()
                    };
                    websocket_text
                        .payload_summary
                        .insert("arg.instType".to_string(), s);
                }

                if let Some(v) = arg.get("topic") {
                    let s = if let Some(s) = v.as_str() {
                        s.to_string()
                    } else {
                        v.to_string()
                    };
                    websocket_text
                        .payload_summary
                        .insert("arg.topic".to_string(), s);
                }
            }
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
        matches!(
            websocket_text.payload_summary.get("arg.topic"),
            Some(t) if t == "books1"
        )
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text.payload_summary.get("arg.topic"),
            Some(t) if t == "publicTrade"
        )
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text.payload_summary.get("arg.topic"),
            Some(t) if t == "order"
        )
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text.payload_summary.get("arg.topic"),
            Some(t) if t == "fill"
        )
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

        let symbol = json_payload["arg"]["symbol"].as_str().unwrap().to_string();

        let data = json_payload["data"].as_array().unwrap();

        let item = data.first().unwrap();

        let ts = item["ts"].as_str().unwrap().parse::<i64>().unwrap();

        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ts);

        let (bid_price, bid_size) = item
            .get("b")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|best| Some((best.get(0)?.as_str()?, best.get(1)?.as_str()?)))
            .map(|(p, s)| (p.to_string(), s.to_string()))
            .unwrap_or_else(|| (String::new(), String::new()));

        let (ask_price, ask_size) = item
            .get("a")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|best| Some((best.get(0)?.as_str()?, best.get(1)?.as_str()?)))
            .map(|(p, s)| (p.to_string(), s.to_string()))
            .unwrap_or_else(|| (String::new(), String::new()));

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitget(
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

        let arg = json_payload.get("arg").and_then(|v| v.as_object()).unwrap();

        let symbol = arg
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let data_array = json_payload.get("data").and_then(|v| v.as_array()).unwrap();

        let trades: Vec<crate::types::Trade> = data_array
            .iter()
            .map(|data| {
                let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
                    data.get("T")
                        .and_then(|v| v.as_str())
                        .unwrap()
                        .parse::<i64>()
                        .unwrap(),
                );

                let price = data.get("p").and_then(|v| v.as_str()).unwrap().to_string();

                let size = data.get("v").and_then(|v| v.as_str()).unwrap().to_string();

                let side = match data.get("S").and_then(|v| v.as_str()) {
                    Some("buy") => crate::types::TakerSide::Buy,
                    Some("sell") => crate::types::TakerSide::Sell,
                    _ => crate::types::TakerSide::Unknown,
                };

                crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitget(
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
        let orders: Vec<crate::types::Order> = websocket_text
            .json_payload
            .as_ref()
            .and_then(|payload| payload.get("data"))
            .and_then(|v| v.as_array())
            .map(|data_array| {
                data_array
                    .iter()
                    .map(|v| {
                        let status_str = v
                            .get("orderStatus")
                            .or_else(|| v.get("status"))
                            .and_then(|s| s.as_str())
                            .unwrap_or("");

                        crate::types::Order {
                            exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitget(
                                self.instrument_type,
                            ),
                            symbol: v["symbol"].as_str().unwrap_or("").to_string(),
                            order_id: v["orderId"].as_str().unwrap_or("").to_string(),
                            client_order_id: v["clientOid"].as_str().unwrap_or("").to_string(),
                            order_type: self.convert_string_to_order_type(
                                v["orderType"].as_str().unwrap_or(""),
                            ),
                            side: self
                                .convert_string_to_order_side(v["side"].as_str().unwrap_or("")),
                            price: v["price"].as_str().unwrap_or("").to_string(),
                            quantity: v["qty"].as_str().unwrap_or("").to_string(),
                            cumulative_filled_quantity: v["cumExecQty"]
                                .as_str()
                                .unwrap_or("")
                                .to_string(),
                            average_filled_price: v
                                .get("avgPrice")
                                .and_then(|f| f.as_str())
                                .unwrap_or("")
                                .to_string(),
                            status: self.convert_string_to_order_status(status_str),
                            leverage: v
                                .get("leverage")
                                .and_then(|f| f.as_str())
                                .unwrap_or("")
                                .to_string(),
                            fill_price: v
                                .get("fillPrice")
                                .and_then(|f| f.as_str())
                                .unwrap_or("")
                                .to_string(),
                            fill_quantity: v
                                .get("baseVolume")
                                .and_then(|f| f.as_str())
                                .unwrap_or("")
                                .to_string(),
                            fill_is_maker: v
                                .get("isMaker")
                                .and_then(|f| f.as_str())
                                .map(|s| s == "true")
                                .unwrap_or(false),
                            ..Default::default()
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData { data: orders },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let fills: Vec<crate::types::Fill> = websocket_text
            .json_payload
            .as_ref()
            .and_then(|payload| payload.get("data"))
            .and_then(|v| v.as_array())
            .map(|data_array| {
                data_array
                    .iter()
                    .map(|data| {
                        let is_maker = data
                            .get("tradeScope")
                            .and_then(|v| v.as_str())
                            .map(|s| s == "maker")
                            .unwrap_or(false);

                        crate::types::Fill {
                            exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitget(
                                self.instrument_type,
                            ),
                            symbol: data
                                .get("symbol")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            order_id: data
                                .get("orderId")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            client_order_id: data
                                .get("clientOid")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            side: match data.get("side").and_then(|v| v.as_str()) {
                                Some("buy") => crate::types::OrderSide::Buy,
                                Some("sell") => crate::types::OrderSide::Sell,
                                _ => crate::types::OrderSide::Unknown,
                            },
                            price: data
                                .get("execPrice")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            quantity: data
                                .get("execQty")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            is_maker,
                            ..Default::default()
                        }
                    })
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

        matches!(payload_summary.get("op"), Some(op) if op == "pong")
            || matches!(payload_summary.get("event"), Some(e) if e == "login")
            || matches!(payload_summary.get("event"), Some(e) if e == "subscribe")
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text.payload_summary.get("event"),
            Some(e) if e == "login"
        )
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text.payload_summary.get("event"),
            Some(e) if e == "subscribe"
        )
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text.payload_summary.get("op"),
            Some(op) if op == "pong"
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
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let (symbols, kind) = websocket_text
            .json_payload
            .as_ref()
            .and_then(|payload| payload["arg"].as_object())
            .map(|arg| {
                let symbol = arg
                    .get("symbol")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());

                let kind =
                    arg.get("topic")
                        .and_then(|t| t.as_str())
                        .and_then(|topic| match topic {
                            "books1" | "books" => Some(
                                crate::exchange_client::common::SubscribeResponseKind::TopOfBook,
                            ),
                            "publicTrade" => {
                                Some(crate::exchange_client::common::SubscribeResponseKind::Trade)
                            }
                            "order" => {
                                Some(crate::exchange_client::common::SubscribeResponseKind::Order)
                            }
                            "fill" => {
                                Some(crate::exchange_client::common::SubscribeResponseKind::Fill)
                            }
                            _ => None,
                        });

                (symbol.into_iter().collect::<Vec<_>>(), kind)
            })
            .unwrap_or_else(|| (vec![], None));

        crate::exchange_client::common::Response::Subscribe(
            crate::exchange_client::common::SubscribeResponse {
                id: None,
                symbols,
                kind,
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

        new_websocket_text.error_code = json_payload.get("code").map(|v| {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                v.to_string()
            }
        });

        new_websocket_text.error_message = json_payload
            .get("msg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
