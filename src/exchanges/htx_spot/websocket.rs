#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::htx_spot::common::HtxSpotClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::HtxSpot(htx_endpoint) => match htx_endpoint {
                crate::types::HtxSpotWebSocketEndpoint::MarketData => {
                    self.websocket_market_data_api_url.to_string()
                }
                crate::types::HtxSpotWebSocketEndpoint::AccountData => {
                    self.websocket_account_data_api_url.to_string()
                }
                crate::types::HtxSpotWebSocketEndpoint::Unknown => {
                    panic!("HtxSpot WebSocket endpoint is Unknown")
                }
            },
            _ => {
                panic!("WebSocket endpoint is not HtxSpot")
            }
        }
    }

    fn create_authenticate_websocket_request(&self) -> String {
        let credential = match &self.credential {
            Some(c) => c,
            None => return String::new(),
        };

        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        let timestamp_encoded = Self::percent_encode_htx(&timestamp);

        let query_string = format!(
            "accessKey={}&signatureMethod=HmacSHA256&signatureVersion=2.1&timestamp={}",
            credential.api_key, timestamp_encoded
        );

        let url_parsed = url::Url::parse(&self.websocket_account_data_api_url)
            .unwrap_or_else(|_| url::Url::parse("wss://api.huobi.pro/ws/v2").unwrap());

        let host = url_parsed.host_str().unwrap_or("api.huobi.pro").to_string();

        let path = url_parsed.path().to_string();

        let prehash = format!("GET\n{}\n{}\n{}", host, path, query_string);

        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .unwrap();

        <hmac::Hmac<sha2::Sha256> as hmac::Mac>::update(&mut mac, prehash.as_bytes());
        let signature_bytes = <hmac::Hmac<sha2::Sha256> as hmac::Mac>::finalize(mac).into_bytes();
        let signature =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature_bytes);

        serde_json::json!({
            "action": "req",
            "ch": "auth",
            "params": {
                "authType": "api",
                "accessKey": credential.api_key,
                "signatureMethod": "HmacSHA256",
                "signatureVersion": "2.1",
                "timestamp": timestamp,
                "signature": signature
            }
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
        subscribe_top_of_book_request
            .symbols
            .iter()
            .enumerate()
            .map(|(i, symbol)| {
                serde_json::json!({
                    "sub": format!("market.{}.bbo", symbol),
                    "id": format!("bbo-{}", i)
                })
                .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        subscribe_trade_request
            .symbols
            .iter()
            .enumerate()
            .map(|(i, symbol)| {
                serde_json::json!({
                    "sub": format!("market.{}.trade.detail", symbol),
                    "id": format!("trade-{}", i)
                })
                .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn create_subscribe_order_websocket_request(
        &self,
        subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        if subscribe_order_request.symbols.is_empty() {
            return serde_json::json!({
                "action": "sub",
                "ch": "orders#*"
            })
            .to_string();
        }

        subscribe_order_request
            .symbols
            .iter()
            .map(|symbol| {
                serde_json::json!({
                    "action": "sub",
                    "ch": format!("orders#{}", symbol)
                })
                .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        if subscribe_fill_request.symbols.is_empty() {
            return serde_json::json!({
                "action": "sub",
                "ch": "trade.clearing#*#0"
            })
            .to_string();
        }

        subscribe_fill_request
            .symbols
            .iter()
            .map(|symbol| {
                serde_json::json!({
                    "action": "sub",
                    "ch": format!("trade.clearing#{}#0", symbol)
                })
                .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    async fn send_websocket_request(
        &self,
        websocket_sender: &crate::networking::websocket::WebSocketSender,
        request: crate::exchange_client::common::Request,
    ) -> crate::exchange_client::common::Response {
        let websocket_request: String = match &request {
            crate::exchange_client::common::Request::SubscribeTopOfBook(req) => {
                self.create_subscribe_top_of_book_websocket_request(req)
            }
            crate::exchange_client::common::Request::SubscribeTrade(req) => {
                self.create_subscribe_trade_websocket_request(req)
            }
            crate::exchange_client::common::Request::SubscribeOrder(req) => {
                self.create_subscribe_order_websocket_request(req)
            }
            crate::exchange_client::common::Request::SubscribeFill(req) => {
                self.create_subscribe_fill_websocket_request(req)
            }
            _ => panic!(),
        };

        for msg in websocket_request.split('\n').filter(|s| !s.is_empty()) {
            crate::fine!("=== WebSocket REQUEST ===");
            crate::fine!("{} {}", websocket_sender.url(), msg);

            if let Err(err) = websocket_sender.send(msg.to_string()).await {
                return crate::exchange_client::common::Response::WebSocketWriteError(err);
            }
        }

        crate::exchange_client::common::Response::None
    }

    fn convert_binary_websocket_message_to_text(
        &self,
        bytes: bytes::Bytes,
    ) -> tungstenite::Utf8Bytes {
        let mut decoder = flate2::read::GzDecoder::new(bytes.as_ref());
        let mut decompressed = String::new();
        std::io::Read::read_to_string(&mut decoder, &mut decompressed)
            .expect("HTX market WS: GZIP decompression failed");
        tungstenite::Utf8Bytes::from(decompressed)
    }

    async fn handle_websocket_message(
        &self,
        websocket_client: &mut crate::networking::websocket::WebSocketClient,
        message: tokio_tungstenite::tungstenite::Message,
    ) -> Result<crate::exchange_client::common::Response, crate::exchange_client::common::Response>
    {
        match message {
            tokio_tungstenite::tungstenite::Message::Binary(bytes) => {
                let text_bytes = self.convert_binary_websocket_message_to_text(bytes);
                crate::fine!("Binary converted: {}", text_bytes);

                let pong_msg = serde_json::from_str::<serde_json::Value>(text_bytes.as_str())
                    .ok()
                    .and_then(|v| {
                        v.get("ping")
                            .and_then(|ts| ts.as_i64())
                            .map(|ts| serde_json::json!({ "pong": ts }).to_string())
                    });

                if let Some(pong) = pong_msg {
                    if let Err(err) = websocket_client.sender().send(pong).await {
                        return Err(
                            crate::exchange_client::common::Response::WebSocketWriteError(err),
                        );
                    }
                    return Ok(crate::exchange_client::common::Response::Heartbeat(
                        crate::exchange_client::common::HeartbeatResponse { id: None },
                    ));
                }

                Ok(self.handle_websocket_text(websocket_client, text_bytes))
            }
            tokio_tungstenite::tungstenite::Message::Text(text_bytes) => {
                crate::fine!("Text received: {}", text_bytes);

                let pong_msg = serde_json::from_str::<serde_json::Value>(text_bytes.as_str())
                    .ok()
                    .and_then(|v| {
                        if v.get("action").and_then(|a| a.as_str()) == Some("ping") {
                            let ts = v
                                .get("data")
                                .and_then(|d| d.get("ts"))
                                .cloned()
                                .unwrap_or(serde_json::Value::Null);
                            Some(
                                serde_json::json!({ "action": "pong", "data": { "ts": ts } })
                                    .to_string(),
                            )
                        } else {
                            None
                        }
                    });

                if let Some(pong) = pong_msg {
                    if let Err(err) = websocket_client.sender().send(pong).await {
                        return Err(
                            crate::exchange_client::common::Response::WebSocketWriteError(err),
                        );
                    }
                    return Ok(crate::exchange_client::common::Response::Heartbeat(
                        crate::exchange_client::common::HeartbeatResponse { id: None },
                    ));
                }

                Ok(self.handle_websocket_text(websocket_client, text_bytes))
            }
            tokio_tungstenite::tungstenite::Message::Pong(payload) => {
                Ok(crate::exchange_client::common::Response::WebSocketPongMessage(payload))
            }
            tokio_tungstenite::tungstenite::Message::Ping(payload) => {
                if let Err(err) = websocket_client.sender().ping(payload.clone()).await {
                    return Err(crate::exchange_client::common::Response::WebSocketWriteError(err));
                }
                Ok(crate::exchange_client::common::Response::WebSocketPingMessage(payload))
            }
            tokio_tungstenite::tungstenite::Message::Close(close_frame) => {
                websocket_client.set_closed();
                Ok(crate::exchange_client::common::Response::WebSocketCloseMessage(close_frame))
            }
            _ => panic!(),
        }
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload {
            for key in &["status", "subbed", "action", "code", "ch"] {
                if let Some(value) = json_payload.get(*key) {
                    if let Some(s) = value.as_str() {
                        websocket_text
                            .payload_summary
                            .insert(key.to_string(), s.to_string());
                    } else {
                        websocket_text
                            .payload_summary
                            .insert(key.to_string(), value.to_string());
                    }
                }
            }
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let ps = &websocket_text.payload_summary;

        let is_market_push = ps.contains_key("ch")
            && !ps.contains_key("status")
            && !ps.contains_key("subbed")
            && !ps.contains_key("action");

        let is_account_push = ps
            .get("action")
            .map(|v| v.as_str() == "push")
            .unwrap_or(false);

        is_market_push || is_account_push
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("ch")
            .map(|ch| ch.ends_with(".bbo"))
            .unwrap_or(false)
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("ch")
            .map(|ch| ch.ends_with(".trade.detail"))
            .unwrap_or(false)
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("ch")
            .map(|ch| ch.starts_with("orders#"))
            .unwrap_or(false)
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("ch")
            .map(|ch| ch.starts_with("trade.clearing#"))
            .unwrap_or(false)
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

        let ch = json_payload["ch"].as_str().unwrap();
        let symbol = ch.split('.').nth(1).unwrap_or("").to_string();

        let ts = json_payload["ts"].as_i64().unwrap_or(0);
        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ts);

        let tick = &json_payload["tick"];

        let bid_price = Self::json_number_to_string(&tick["bid"]);
        let bid_size = Self::json_number_to_string(&tick["bidSize"]);
        let ask_price = Self::json_number_to_string(&tick["ask"]);
        let ask_size = Self::json_number_to_string(&tick["askSize"]);

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxSpot,
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

        let ch = json_payload["ch"].as_str().unwrap();
        let symbol = ch.split('.').nth(1).unwrap_or("").to_string();

        let data_array = json_payload["tick"]["data"].as_array().unwrap();

        let trades: Vec<crate::types::Trade> = data_array
            .iter()
            .map(|data| {
                let ts = data["ts"].as_i64().unwrap_or(0);
                let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ts);

                let price = Self::json_number_to_string(&data["price"]);
                let size = Self::json_number_to_string(&data["amount"]);

                let side = match data.get("direction").and_then(|v| v.as_str()) {
                    Some("buy") => crate::types::TakerSide::Buy,
                    Some("sell") => crate::types::TakerSide::Sell,
                    _ => crate::types::TakerSide::Unknown,
                };

                crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxSpot,
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
        let data = &json_payload["data"];

        let order_id = if let Some(s) = data["orderId"].as_str() {
            s.to_string()
        } else if let Some(n) = data["orderId"].as_u64() {
            n.to_string()
        } else {
            String::new()
        };

        let type_str = data["type"].as_str().unwrap_or("");

        let order = crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxSpot,
            symbol: data["symbol"].as_str().unwrap_or("").to_string(),
            order_id,
            client_order_id: data["clientOrderId"].as_str().unwrap_or("").to_string(),
            order_type: self.convert_string_to_order_type(type_str),
            side: self.convert_string_to_order_side(type_str),
            price: data["orderPrice"].as_str().unwrap_or("").to_string(),
            quantity: data["orderSize"].as_str().unwrap_or("").to_string(),
            cumulative_filled_quantity: data["filledAmount"].as_str().unwrap_or("").to_string(),
            cumulative_filled_quote_quantity: data["filledCashAmount"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            status: self.convert_string_to_order_status(data["orderStatus"].as_str().unwrap_or("")),
            fill_price: data["tradePrice"].as_str().unwrap_or("").to_string(),
            fill_quantity: data["tradeVolume"].as_str().unwrap_or("").to_string(),
            fill_is_maker: !data["aggressor"].as_bool().unwrap_or(true),
            ..Default::default()
        };

        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData { data: vec![order] },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();
        let data = &json_payload["data"];

        let order_id = if let Some(s) = data["orderId"].as_str() {
            s.to_string()
        } else if let Some(n) = data["orderId"].as_u64() {
            n.to_string()
        } else {
            String::new()
        };

        let side = match data.get("orderSide").and_then(|v| v.as_str()) {
            Some("buy") => crate::types::OrderSide::Buy,
            Some("sell") => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        };

        let fill = crate::types::Fill {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxSpot,
            symbol: data["symbol"].as_str().unwrap_or("").to_string(),
            order_id,
            client_order_id: data["clientOrderId"].as_str().unwrap_or("").to_string(),
            side,
            price: data["tradePrice"].as_str().unwrap_or("").to_string(),
            quantity: data["tradeVolume"].as_str().unwrap_or("").to_string(),
            is_maker: !data["aggressor"].as_bool().unwrap_or(true),
            ..Default::default()
        };

        crate::exchange_client::common::Response::FillSubscription(
            crate::exchange_client::common::FillSubscriptionData { data: vec![fill] },
        )
    }

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let ps = &websocket_text.payload_summary;

        let is_market_sub =
            ps.get("status").map(|v| v == "ok").unwrap_or(false) && ps.contains_key("subbed");

        let is_auth = ps.get("action").map(|v| v == "req").unwrap_or(false)
            && ps.get("ch").map(|v| v == "auth").unwrap_or(false)
            && ps.get("code").map(|v| v == "200").unwrap_or(false);

        let is_account_sub = ps.get("action").map(|v| v == "sub").unwrap_or(false)
            && ps.get("code").map(|v| v == "200").unwrap_or(false);

        is_market_sub || is_auth || is_account_sub
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let ps = &websocket_text.payload_summary;

        ps.get("action").map(|v| v == "req").unwrap_or(false)
            && ps.get("ch").map(|v| v == "auth").unwrap_or(false)
            && ps.get("code").map(|v| v == "200").unwrap_or(false)
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let ps = &websocket_text.payload_summary;

        let is_market =
            ps.get("status").map(|v| v == "ok").unwrap_or(false) && ps.contains_key("subbed");

        let is_account = ps.get("action").map(|v| v == "sub").unwrap_or(false)
            && ps.get("code").map(|v| v == "200").unwrap_or(false);

        is_market || is_account
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

        new_websocket_text.error_code = json_payload.get("code").and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else {
                v.as_i64().map(|n| n.to_string())
            }
        });

        new_websocket_text.error_message = json_payload
            .get("message")
            .or_else(|| json_payload.get("err-msg"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
