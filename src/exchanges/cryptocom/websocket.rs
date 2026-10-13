#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::cryptocom::common::CryptocomClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::Cryptocom(endpoint) => match endpoint {
                crate::types::CryptocomWebSocketEndpoint::MarketData => {
                    self.websocket_market_data_api_url.clone()
                }
                crate::types::CryptocomWebSocketEndpoint::AccountData => {
                    self.websocket_account_data_api_url.clone()
                }
                crate::types::CryptocomWebSocketEndpoint::Unknown => {
                    panic!("Cryptocom websocket endpoint is Unknown")
                }
            },
            _ => panic!("Websocket endpoint is not Cryptocom"),
        }
    }

    async fn authenticate_websocket_connection(
        &self,
        client: &mut crate::networking::websocket::WebSocketClient,
    ) -> anyhow::Result<()> {
        if client.url() != self.websocket_account_data_api_url {
            return Ok(());
        }
        let request = self.create_authenticate_websocket_request();
        if request.is_empty() {
            return Ok(());
        }
        client.sender().send(request).await?;
        loop {
            match self.read_next_websocket_message(client).await {
                crate::exchange_client::common::Response::Authenticate(_) => return Ok(()),
                crate::exchange_client::common::Response::WebSocketWriteError(_)
                | crate::exchange_client::common::Response::WebSocketReadError(_)
                | crate::exchange_client::common::Response::WebSocketCloseMessage(_)
                | crate::exchange_client::common::Response::WebSocketErrorResponse(_) => {
                    return Err(anyhow::anyhow!("websocket authentication failed"));
                }
                _ => {}
            }
        }
    }

    fn create_authenticate_websocket_request(&self) -> String {
        let credential = match &self.credential {
            Some(credential) => credential,
            None => return String::new(),
        };
        let nonce = chrono::Utc::now().timestamp_millis();
        let id = nonce;
        let method = "public/auth";
        let signature_payload = format!("{}{}{}{}", method, id, credential.api_key, nonce);
        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .unwrap();
        <hmac::Hmac<sha2::Sha256> as hmac::Mac>::update(&mut mac, signature_payload.as_bytes());
        let signature =
            hex::encode(<hmac::Hmac<sha2::Sha256> as hmac::Mac>::finalize(mac).into_bytes());
        serde_json::json!({
            "id": id,
            "method": method,
            "api_key": credential.api_key,
            "sig": signature,
            "nonce": nonce
        })
        .to_string()
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(String::new)
    }

    async fn handle_websocket_message(
        &self,
        websocket_client: &mut crate::networking::websocket::WebSocketClient,
        message: tokio_tungstenite::tungstenite::Message,
    ) -> Result<crate::exchange_client::common::Response, crate::exchange_client::common::Response>
    {
        match message {
            tokio_tungstenite::tungstenite::Message::Text(text) => {
                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(text.as_str())
                    && payload.get("method").and_then(|value| value.as_str())
                        == Some("public/heartbeat")
                {
                    let id = payload
                        .get("id")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    let response = serde_json::json!({
                        "id": id,
                        "method": "public/respond-heartbeat"
                    })
                    .to_string();
                    if let Err(error) = websocket_client.sender().send(response).await {
                        return Err(
                            crate::exchange_client::common::Response::WebSocketWriteError(error),
                        );
                    }
                }
                Ok(self.handle_websocket_text(websocket_client, text))
            }
            tokio_tungstenite::tungstenite::Message::Binary(bytes) => {
                let text = self.convert_binary_websocket_message_to_text(bytes);
                Ok(self.handle_websocket_text(websocket_client, text))
            }
            tokio_tungstenite::tungstenite::Message::Pong(payload) => {
                Ok(crate::exchange_client::common::Response::WebSocketPongMessage(payload))
            }
            tokio_tungstenite::tungstenite::Message::Ping(payload) => {
                if let Err(error) = websocket_client
                    .sender()
                    .clone()
                    .ping(payload.clone())
                    .await
                {
                    return Err(
                        crate::exchange_client::common::Response::WebSocketWriteError(error),
                    );
                }
                Ok(crate::exchange_client::common::Response::WebSocketPingMessage(payload))
            }
            tokio_tungstenite::tungstenite::Message::Close(frame) => {
                websocket_client.set_closed();
                Ok(crate::exchange_client::common::Response::WebSocketCloseMessage(frame))
            }
            _ => panic!(),
        }
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        serde_json::json!({
            "id": request.id.unwrap_or_else(|| chrono::Utc::now().timestamp_millis() as u64),
            "method": "subscribe",
            "nonce": chrono::Utc::now().timestamp_millis(),
            "params": {
                "channels": request.symbols.iter().map(|symbol| format!("ticker.{}", symbol)).collect::<Vec<String>>(),
            }
        })
        .to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        serde_json::json!({
            "id": request.id.unwrap_or_else(|| chrono::Utc::now().timestamp_millis() as u64),
            "method": "subscribe",
            "nonce": chrono::Utc::now().timestamp_millis(),
            "params": {
                "channels": request.symbols.iter().map(|symbol| format!("trade.{}", symbol)).collect::<Vec<String>>()
            }
        })
        .to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        serde_json::json!({
            "id": request.id.unwrap_or_else(|| chrono::Utc::now().timestamp_millis() as u64),
            "method": "subscribe",
            "nonce": chrono::Utc::now().timestamp_millis(),
            "params": { "channels": ["user.order"] }
        })
        .to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        serde_json::json!({
            "id": request.id.unwrap_or_else(|| chrono::Utc::now().timestamp_millis() as u64),
            "method": "subscribe",
            "nonce": chrono::Utc::now().timestamp_millis(),
            "params": { "channels": ["user.trade"] }
        })
        .to_string()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(payload) = &websocket_text.json_payload {
            for key in ["id", "method", "code"] {
                if let Some(value) = payload.get(key) {
                    websocket_text.payload_summary.insert(
                        key.to_string(),
                        value
                            .as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| value.to_string()),
                    );
                }
            }
            if let Some(result) = payload.get("result") {
                for key in ["instrument_name", "channel"] {
                    if let Some(value) = result.get(key) {
                        websocket_text.payload_summary.insert(
                            format!("result_{key}"),
                            value
                                .as_str()
                                .map(str::to_string)
                                .unwrap_or_else(|| value.to_string()),
                        );
                    }
                }
            }
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .json_payload
            .as_ref()
            .and_then(|payload| payload.get("result"))
            .and_then(|result| result.get("data"))
            .and_then(|data| data.as_array())
            .is_some()
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        Self::result_channel(websocket_text).is_some_and(|channel| channel == "ticker")
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        Self::result_channel(websocket_text).is_some_and(|channel| channel == "trade")
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        Self::result_channel(websocket_text).is_some_and(|channel| channel == "user.order")
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        Self::result_channel(websocket_text).is_some_and(|channel| channel == "user.trade")
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
        let result = &websocket_text.json_payload.as_ref().unwrap()["result"];
        let symbol = result["instrument_name"].as_str().unwrap_or("");
        let data = result["data"].as_array().map(Vec::as_slice).unwrap_or(&[]);
        let books = data
            .iter()
            .map(|entry| crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Cryptocom,
                symbol: symbol.to_string(),
                timestamp: entry["t"]
                    .as_i64()
                    .and_then(chrono::DateTime::<chrono::Utc>::from_timestamp_millis)
                    .unwrap_or_default(),
                bid_price: Self::json_value_to_string(&entry["b"]),
                bid_size: Self::json_value_to_string(&entry["bs"]),
                ask_price: Self::json_value_to_string(&entry["k"]),
                ask_size: Self::json_value_to_string(&entry["ks"]),
            })
            .collect();
        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData { data: books },
        )
    }

    fn create_subscribe_trade_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let result = &websocket_text.json_payload.as_ref().unwrap()["result"];
        let trades = result["data"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .map(|entry| crate::types::Trade {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Cryptocom,
                symbol: entry["i"].as_str().unwrap_or("").to_string(),
                timestamp: chrono::DateTime::from_timestamp_millis(
                    entry["t"].as_i64().unwrap_or_default(),
                )
                .unwrap_or_default(),
                price: Self::json_value_to_string(&entry["p"]),
                size: Self::json_value_to_string(&entry["q"]),
                side: match entry["s"].as_str() {
                    Some("BUY") => crate::types::TakerSide::Buy,
                    Some("SELL") => crate::types::TakerSide::Sell,
                    _ => crate::types::TakerSide::Unknown,
                },
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
        let orders = websocket_text.json_payload.as_ref().unwrap()["result"]["data"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .map(|value| self.convert_json_value_to_order(value))
            .collect();
        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData { data: orders },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let fills = websocket_text.json_payload.as_ref().unwrap()["result"]["data"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .map(|value| {
                let price = Self::json_value_to_string(&value["traded_price"]);
                let quantity = Self::json_value_to_string(&value["traded_quantity"]);
                let quote_quantity = match (price.parse::<f64>(), quantity.parse::<f64>()) {
                    (Ok(price), Ok(quantity)) => (price * quantity).to_string(),
                    _ => String::new(),
                };
                crate::types::Fill {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Cryptocom,
                    symbol: value["instrument_name"].as_str().unwrap_or("").to_string(),
                    order_id: Self::json_value_to_string(&value["order_id"]),
                    client_order_id: value["client_oid"].as_str().unwrap_or("").to_string(),
                    side: self.convert_string_to_order_side(value["side"].as_str().unwrap_or("")),
                    price,
                    quantity,
                    quote_quantity,
                    is_maker: value["taker_side"].as_str() == Some("MAKER"),
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
        websocket_text
            .payload_summary
            .get("code")
            .map(String::as_str)
            == Some("0")
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("method")
            .map(String::as_str)
            == Some("public/auth")
            && self.is_websocket_text_success_response(websocket_text)
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("method")
            .map(String::as_str)
            == Some("subscribe")
            && self.is_websocket_text_success_response(websocket_text)
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        matches!(
            websocket_text
                .payload_summary
                .get("method")
                .map(String::as_str),
            Some("public/heartbeat" | "public/respond-heartbeat")
        )
    }

    fn create_subscribe_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let result = &websocket_text.json_payload.as_ref().unwrap()["result"];
        let channel = result["channel"].as_str().unwrap_or("");
        let symbol = result["instrument_name"].as_str().unwrap_or("");
        let kind = match channel {
            "ticker" => Some(crate::exchange_client::common::SubscribeResponseKind::TopOfBook),
            "trade" => Some(crate::exchange_client::common::SubscribeResponseKind::Trade),
            "user.order" => Some(crate::exchange_client::common::SubscribeResponseKind::Order),
            "user.trade" => Some(crate::exchange_client::common::SubscribeResponseKind::Fill),
            _ => None,
        };
        crate::exchange_client::common::Response::Subscribe(
            crate::exchange_client::common::SubscribeResponse {
                symbols: if symbol.is_empty() {
                    Vec::new()
                } else {
                    vec![symbol.to_string()]
                },
                kind,
                ..Default::default()
            },
        )
    }

    fn create_authenticate_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        crate::exchange_client::common::Response::Authenticate(
            crate::exchange_client::common::AuthenticateResponse {
                id: websocket_text
                    .json_payload
                    .as_ref()
                    .and_then(|payload| payload["id"].as_u64()),
            },
        )
    }

    fn create_heartbeat_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        crate::exchange_client::common::Response::Heartbeat(
            crate::exchange_client::common::HeartbeatResponse {
                id: websocket_text
                    .json_payload
                    .as_ref()
                    .and_then(|payload| payload["id"].as_u64()),
            },
        )
    }

    fn create_websocket_error_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let mut response = websocket_text.clone();
        response.error_message = response
            .json_payload
            .as_ref()
            .and_then(|payload| payload.get("message"))
            .and_then(|message| message.as_str())
            .map(str::to_string);
        crate::exchange_client::common::Response::WebSocketErrorResponse(response)
    }
}

impl crate::exchanges::cryptocom::common::CryptocomClient {
    fn result_channel(
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> Option<&str> {
        websocket_text
            .json_payload
            .as_ref()?
            .get("result")?
            .get("channel")?
            .as_str()
    }
}
