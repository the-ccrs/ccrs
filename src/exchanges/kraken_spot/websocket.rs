#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::kraken_spot::common::KrakenSpotClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::KrakenSpot(kraken_endpoint) => match kraken_endpoint {
                crate::types::KrakenSpotWebSocketEndpoint::MarketData => {
                    self.websocket_market_data_api_url.clone()
                }
                crate::types::KrakenSpotWebSocketEndpoint::AccountData => {
                    self.websocket_account_data_api_url.clone()
                }
                crate::types::KrakenSpotWebSocketEndpoint::Unknown => {
                    panic!("KrakenSpot websocket endpoint is Unknown")
                }
            },
            _ => {
                panic!("Websocket endpoint is not KrakenSpot")
            }
        }
    }

    async fn authenticate_websocket_connection(
        &self,
        _client: &mut crate::networking::websocket::WebSocketClient,
    ) -> anyhow::Result<()> {
        let credential = match &self.credential {
            Some(c) => c,
            None => return Ok(()),
        };

        let nonce = crate::exchange_client::common::Common::generate_next_nonce(self);
        let nonce_str = nonce.to_string();
        let post_data = format!("nonce={}", nonce_str);
        let path = "/0/private/GetWebSocketsToken";

        let sha_input = format!("{}{}", nonce_str, post_data);
        let mut sha256_hasher = sha2::Sha256::default();
        <sha2::Sha256 as sha2::Digest>::update(&mut sha256_hasher, sha_input.as_bytes());
        let sha256_hash = <sha2::Sha256 as sha2::Digest>::finalize(sha256_hasher);

        let mut hmac_message = Vec::<u8>::new();
        hmac_message.extend_from_slice(path.as_bytes());
        hmac_message.extend_from_slice(&sha256_hash);

        let decoded_secret = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            credential.api_secret.as_bytes(),
        )?;

        let mut mac =
            <hmac::Hmac<sha2::Sha512> as hmac::digest::KeyInit>::new_from_slice(&decoded_secret)
                .map_err(|e| anyhow::anyhow!("HMAC init error: {}", e))?;
        <hmac::Hmac<sha2::Sha512> as hmac::Mac>::update(&mut mac, &hmac_message);
        let signature_bytes = <hmac::Hmac<sha2::Sha512> as hmac::Mac>::finalize(mac).into_bytes();

        let signature =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature_bytes);

        let url = format!("{}{}", self.rest_api_base_url, path);
        let http_client = reqwest::Client::new();

        let response = http_client
            .post(&url)
            .header("API-Key", &credential.api_key)
            .header("API-Sign", &signature)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(post_data)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;

        let token = json["result"]["token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to obtain websocket token from response"))?
            .to_string();

        *self.websocket_token.lock().unwrap() = Some(token);

        Ok(())
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(move || {
            serde_json::json!({
                "method": "ping",
                "req_id": chrono::Utc::now().timestamp_millis()
            })
            .to_string()
        })
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let mut params = serde_json::Map::new();
        params.insert(
            "channel".to_string(),
            serde_json::Value::String("ticker".to_string()),
        );
        params.insert(
            "symbol".to_string(),
            serde_json::to_value(&subscribe_top_of_book_request.symbols).unwrap(),
        );

        let mut payload = serde_json::Map::new();
        payload.insert(
            "method".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        payload.insert("params".to_string(), serde_json::Value::Object(params));

        if let Some(id) = subscribe_top_of_book_request.id {
            payload.insert("req_id".to_string(), serde_json::Value::Number(id.into()));
        }

        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let mut params = serde_json::Map::new();
        params.insert(
            "channel".to_string(),
            serde_json::Value::String("trade".to_string()),
        );
        params.insert(
            "symbol".to_string(),
            serde_json::to_value(&subscribe_trade_request.symbols).unwrap(),
        );

        let mut payload = serde_json::Map::new();
        payload.insert(
            "method".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        payload.insert("params".to_string(), serde_json::Value::Object(params));

        if let Some(id) = subscribe_trade_request.id {
            payload.insert("req_id".to_string(), serde_json::Value::Number(id.into()));
        }

        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        let token = self
            .websocket_token
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_default();

        let mut params = serde_json::Map::new();
        params.insert(
            "channel".to_string(),
            serde_json::Value::String("executions".to_string()),
        );
        params.insert("token".to_string(), serde_json::Value::String(token));
        params.insert("snap_orders".to_string(), serde_json::Value::Bool(true));

        let mut payload = serde_json::Map::new();
        payload.insert(
            "method".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        payload.insert("params".to_string(), serde_json::Value::Object(params));

        if let Some(id) = subscribe_order_request.id {
            payload.insert("req_id".to_string(), serde_json::Value::Number(id.into()));
        }

        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        _: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        panic!()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload {
            if let Some(s) = json_payload.get("channel").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("channel".to_string(), s.to_string());
            }

            if let Some(s) = json_payload.get("type").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("type".to_string(), s.to_string());
            }

            if let Some(s) = json_payload.get("method").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("method".to_string(), s.to_string());
            }

            if let Some(v) = json_payload.get("success") {
                websocket_text
                    .payload_summary
                    .insert("success".to_string(), v.to_string());
            }

            if let Some(v) = json_payload.get("result").and_then(|r| r.get("channel")) {
                let s = if let Some(str_val) = v.as_str() {
                    str_val.to_string()
                } else {
                    v.to_string()
                };
                websocket_text
                    .payload_summary
                    .insert("result_channel".to_string(), s);
            }
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        match payload_summary.get("channel").map(String::as_str) {
            Some("heartbeat") => false,
            Some(_) => payload_summary.contains_key("type"),
            None => false,
        }
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("channel")
            .map(String::as_str)
            == Some("ticker")
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("channel")
            .map(String::as_str)
            == Some("trade")
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("channel")
            .map(String::as_str)
            == Some("executions")
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        _: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
    }

    fn is_websocket_text_unneeded_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("channel")
            .map(String::as_str)
            == Some("status")
    }

    fn create_subscribe_top_of_book_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();
        let data_array = json_payload.get("data").and_then(|v| v.as_array()).unwrap();

        let top_of_books: Vec<crate::types::TopOfBook> = data_array
            .iter()
            .map(|data| {
                let symbol = data
                    .get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let timestamp_str = data.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
                let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                let bid_price = data.get("bid").map(|v| v.to_string()).unwrap_or_default();
                let bid_size = data
                    .get("bid_qty")
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                let ask_price = data.get("ask").map(|v| v.to_string()).unwrap_or_default();
                let ask_size = data
                    .get("ask_qty")
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenSpot,
                    symbol,
                    timestamp,
                    bid_price,
                    bid_size,
                    ask_price,
                    ask_size,
                }
            })
            .collect();

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData { data: top_of_books },
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
                    .get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let timestamp_str = data.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
                let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                let price = data.get("price").map(|v| v.to_string()).unwrap_or_default();
                let size = data.get("qty").map(|v| v.to_string()).unwrap_or_default();

                let side = match data.get("side").and_then(|v| v.as_str()) {
                    Some("buy") => crate::types::TakerSide::Buy,
                    Some("sell") => crate::types::TakerSide::Sell,
                    _ => crate::types::TakerSide::Unknown,
                };

                crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenSpot,
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
            .map(|data| self.convert_json_value_to_order_from_executions(data))
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
            .filter(|data| data.get("exec_type").and_then(|v| v.as_str()) == Some("trade"))
            .map(|data| self.convert_json_value_to_fill_from_executions(data))
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
        payload_summary.get("success").map(String::as_str) == Some("true")
            || payload_summary.get("method").map(String::as_str) == Some("pong")
            || payload_summary.get("channel").map(String::as_str) == Some("heartbeat")
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
        payload_summary.get("method").map(String::as_str) == Some("subscribe")
            && payload_summary.get("success").map(String::as_str) == Some("true")
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary.get("method").map(String::as_str) == Some("pong")
            || payload_summary.get("channel").map(String::as_str) == Some("heartbeat")
    }

    fn create_subscribe_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let mut symbols: Vec<String> = Vec::new();

        if let Some(symbol) = websocket_text
            .json_payload
            .as_ref()
            .and_then(|payload| payload.get("result"))
            .and_then(|r| r.get("symbol"))
            .and_then(|v| v.as_str())
        {
            symbols.push(symbol.to_string());
        }

        let channel = websocket_text
            .payload_summary
            .get("result_channel")
            .map(String::as_str);

        let kind = if channel == Some("ticker") {
            Some(crate::exchange_client::common::SubscribeResponseKind::TopOfBook)
        } else if channel == Some("trade") {
            Some(crate::exchange_client::common::SubscribeResponseKind::Trade)
        } else if channel == Some("executions") {
            Some(crate::exchange_client::common::SubscribeResponseKind::Order)
        } else {
            None
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
        let id = json_payload.get("req_id").and_then(|v| v.as_u64());

        crate::exchange_client::common::Response::Authenticate(
            crate::exchange_client::common::AuthenticateResponse { id },
        )
    }

    fn create_heartbeat_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();
        let id = json_payload.get("req_id").and_then(|v| v.as_u64());

        crate::exchange_client::common::Response::Heartbeat(
            crate::exchange_client::common::HeartbeatResponse { id },
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
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
