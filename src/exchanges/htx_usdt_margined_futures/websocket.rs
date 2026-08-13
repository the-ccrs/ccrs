#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::HtxUsdtMarginedFutures(htx_endpoint) => {
                match htx_endpoint {
                    crate::types::HtxUsdtMarginedFuturesWebSocketEndpoint::MarketData => {
                        self.websocket_market_data_api_url.to_string()
                    }
                    crate::types::HtxUsdtMarginedFuturesWebSocketEndpoint::AccountData => {
                        self.websocket_account_data_api_url.to_string()
                    }
                    crate::types::HtxUsdtMarginedFuturesWebSocketEndpoint::Unknown => {
                        panic!("HtxUsdtMarginedFutures WebSocket endpoint is Unknown")
                    }
                }
            }
            _ => {
                panic!("WebSocket endpoint is not HtxUsdtMarginedFutures")
            }
        }
    }

    fn create_authenticate_websocket_request(&self) -> String {
        let credential = match &self.credential {
            Some(c) => c,
            None => return String::new(),
        };

        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

        let timestamp_encoded =
            crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::percent_encode_htx(&timestamp);

        let query_string = format!(
            "AccessKeyId={}&SignatureMethod=HmacSHA256&SignatureVersion=2&Timestamp={}",
            credential.api_key, timestamp_encoded
        );

        let url_parsed =
            url::Url::parse(&self.websocket_account_data_api_url).unwrap_or_else(|_| {
                url::Url::parse("wss://api.hbdm.com/linear-swap-notification").unwrap()
            });

        let host = url_parsed.host_str().unwrap_or("api.hbdm.com").to_string();
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
            "op": "auth",
            "type": "api",
            "AccessKeyId": credential.api_key,
            "SignatureMethod": "HmacSHA256",
            "SignatureVersion": "2",
            "Timestamp": timestamp,
            "Signature": signature
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
                "op": "sub",
                "topic": "orders",
                "contract_code": "*"
            })
            .to_string();
        }

        subscribe_order_request
            .symbols
            .iter()
            .map(|symbol| {
                serde_json::json!({
                    "op": "sub",
                    "topic": "orders",
                    "contract_code": symbol
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
                "op": "sub",
                "topic": "trade",
                "contract_code": "*"
            })
            .to_string();
        }

        subscribe_fill_request
            .symbols
            .iter()
            .map(|symbol| {
                serde_json::json!({
                    "op": "sub",
                    "topic": "trade",
                    "contract_code": symbol
                })
                .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn convert_binary_websocket_message_to_text(
        &self,
        bytes: bytes::Bytes,
    ) -> tungstenite::Utf8Bytes {
        let mut decoder = flate2::read::GzDecoder::new(bytes.as_ref());
        let mut decompressed = String::new();
        std::io::Read::read_to_string(&mut decoder, &mut decompressed)
            .expect("HTX USDT-M futures market WS: GZIP decompression failed");
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
                        if v.get("op").and_then(|a| a.as_str()) == Some("ping") {
                            let ts = v.get("ts").cloned().unwrap_or(serde_json::Value::Null);
                            Some(serde_json::json!({ "op": "pong", "ts": ts }).to_string())
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
            for key in &[
                "status", "subbed", "ch", "op", "topic", "ping", "pong", "err-code",
            ] {
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
            && !ps.contains_key("op");

        let is_account_push = ps.get("op").map(|v| v == "notify").unwrap_or(false);

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
            .get("topic")
            .map(|topic| topic == "orders")
            .unwrap_or(false)
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("topic")
            .map(|topic| topic == "trade")
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

        let ch = json_payload["ch"].as_str().unwrap();
        let symbol = ch.split('.').nth(1).unwrap_or("").to_string();

        let ts = json_payload["ts"].as_i64().unwrap_or(0);
        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ts);

        let tick = &json_payload["tick"];

        let bid_price = crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
            tick["bid"]
                .as_array()
                .and_then(|a| a.first())
                .unwrap_or(&serde_json::Value::Null),
        );
        let bid_size = crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
            tick["bid"]
                .as_array()
                .and_then(|a| a.get(1))
                .unwrap_or(&serde_json::Value::Null),
        );
        let ask_price = crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
            tick["ask"]
                .as_array()
                .and_then(|a| a.first())
                .unwrap_or(&serde_json::Value::Null),
        );
        let ask_size = crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
            tick["ask"]
                .as_array()
                .and_then(|a| a.get(1))
                .unwrap_or(&serde_json::Value::Null),
        );

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type:
                        crate::types::ExchangeInstrumentType::HtxUsdtMarginedFutures,
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
                let timestamp =
                    crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ts);

                let price = crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                    &data["price"],
                );
                let size = crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                    &data["amount"],
                );

                let side = match data.get("direction").and_then(|v| v.as_str()) {
                    Some("buy") => crate::types::TakerSide::Buy,
                    Some("sell") => crate::types::TakerSide::Sell,
                    _ => crate::types::TakerSide::Unknown,
                };

                crate::types::Trade {
                    exchange_instrument_type:
                        crate::types::ExchangeInstrumentType::HtxUsdtMarginedFutures,
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

        let order = self.convert_json_value_to_order(json_payload.get("data").unwrap());

        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData { data: vec![order] },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let fills: Vec<crate::types::Fill> = if let Some(trades) =
            json_payload.get("data").and_then(|v| v.as_array())
        {
            trades
            .iter()
            .map(|trade| {
                let symbol = trade["contract_code"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_default();

                let order_id = trade["order_id"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_default();

                let client_order_id = trade["client_order_id"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        trade["client_order_id"]
                            .as_i64()
                            .map(|v| v.to_string())
                            .unwrap_or_default()
                    });

                let side = match trade["direction"].as_str().unwrap_or("") {
                    "buy" => crate::types::OrderSide::Buy,
                    "sell" => crate::types::OrderSide::Sell,
                    _ => crate::types::OrderSide::Unknown,
                };

                let price = crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                    &trade["trade_price"],
                );
                let quantity = crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                    &trade["trade_volume"],
                );
                let quote_quantity = crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                    &trade["trade_turnover"],
                );
                let is_maker = trade.get("role").and_then(|v| v.as_str()) == Some("maker");

                crate::types::Fill {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxUsdtMarginedFutures,
                    symbol,
                    order_id,
                    client_order_id,
                    side,
                    price,
                    quantity,
                    quote_quantity,
                    is_maker,
                    ..Default::default()
                }
            })
            .collect()
        } else {
            vec![]
        };

        crate::exchange_client::common::Response::FillSubscription(
            crate::exchange_client::common::FillSubscriptionData { data: fills },
        )
    }

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        self.is_websocket_text_authenticate_success_response(websocket_text)
            || self.is_websocket_text_subscribe_success_response(websocket_text)
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let ps = &websocket_text.payload_summary;

        ps.get("op").map(|v| v == "auth").unwrap_or(false)
            && ps.get("err-code").map(|v| v == "0").unwrap_or(false)
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let ps = &websocket_text.payload_summary;

        let is_market =
            ps.get("status").map(|v| v == "ok").unwrap_or(false) && ps.contains_key("subbed");

        let is_account = ps.get("op").map(|v| v == "sub").unwrap_or(false)
            && ps.get("err-code").map(|v| v == "0").unwrap_or(false);

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

        new_websocket_text.error_code = json_payload.get("err-code").and_then(|v| {
            if let Some(s) = v.as_str() {
                Some(s.to_string())
            } else {
                v.as_i64().map(|n| n.to_string())
            }
        });

        new_websocket_text.error_message = json_payload
            .get("err-msg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
