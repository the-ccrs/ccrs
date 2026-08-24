#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::bitstamp::common::BitstampClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::Bitstamp(
                crate::types::BitstampWebSocketEndpoint::MarketData,
            ) => self.websocket_market_data_api_url.clone(),
            crate::types::WebSocketEndpoint::Bitstamp(
                crate::types::BitstampWebSocketEndpoint::AccountData,
            ) => self.websocket_account_data_api_url.clone(),
            crate::types::WebSocketEndpoint::Bitstamp(
                crate::types::BitstampWebSocketEndpoint::Unknown,
            ) => panic!("Bitstamp websocket endpoint is Unknown"),
            _ => panic!("Websocket endpoint is not Bitstamp"),
        }
    }

    async fn authenticate_websocket_connection(
        &self,
        _client: &mut crate::networking::websocket::WebSocketClient,
    ) -> anyhow::Result<()> {
        let credential = match &self.credential {
            Some(credential) => credential,
            None => return Ok(()),
        };
        let path = "/api/v2/websockets_token/";
        let nonce = uuid::Uuid::new_v4().to_string().to_lowercase();
        let timestamp = chrono::Utc::now().timestamp_millis().to_string();
        let version = "v2";
        let x_auth = format!("BITSTAMP {}", credential.api_key);
        let host = url::Url::parse(&self.rest_api_base_url)?
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("Bitstamp REST URL has no host"))?
            .to_string();
        let message = format!(
            "{}POST{}{}{}{}{}",
            x_auth, host, path, nonce, timestamp, version
        );
        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .map_err(|error| anyhow::anyhow!("HMAC init error: {}", error))?;
        <hmac::Hmac<sha2::Sha256> as hmac::Mac>::update(&mut mac, message.as_bytes());
        let signature =
            hex::encode(<hmac::Hmac<sha2::Sha256> as hmac::Mac>::finalize(mac).into_bytes());
        let response = reqwest::Client::new()
            .post(format!("{}{}", self.rest_api_base_url, path))
            .header("X-Auth", x_auth)
            .header("X-Auth-Signature", signature)
            .header("X-Auth-Nonce", nonce)
            .header("X-Auth-Timestamp", timestamp)
            .header("X-Auth-Version", version)
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to obtain Bitstamp websocket token: HTTP {}",
                response.status()
            ));
        }
        let json: serde_json::Value = response.json().await?;
        let token = json
            .get("token")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Bitstamp websocket token is missing"))?
            .to_string();
        let user_id = json
            .get("user_id")
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| value.to_string())
            })
            .ok_or_else(|| anyhow::anyhow!("Bitstamp websocket user_id is missing"))?;
        *self.websocket_token.lock().unwrap() = Some(token);
        *self.websocket_user_id.lock().unwrap() = Some(user_id);
        Ok(())
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(|| serde_json::json!({"event": "bts:heartbeat"}).to_string())
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let symbol = request.symbols.first().cloned().unwrap_or_default();
        serde_json::json!({
            "event": "bts:subscribe",
            "data": {"channel": format!("order_book_{}", symbol)}
        })
        .to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let symbol = request.symbols.first().cloned().unwrap_or_default();
        serde_json::json!({
            "event": "bts:subscribe",
            "data": {"channel": format!("live_trades_{}", symbol)}
        })
        .to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        let symbol = request.symbols.first().cloned().unwrap_or_default();
        let token = self
            .websocket_token
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_default();
        let user_id = self
            .websocket_user_id
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_default();
        serde_json::json!({
            "event": "bts:subscribe",
            "data": {
                "channel": format!("private-my_orders_{}-{}", symbol, user_id),
                "auth": token
            }
        })
        .to_string()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        let symbol = request.symbols.first().cloned().unwrap_or_default();
        let token = self
            .websocket_token
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_default();
        let user_id = self
            .websocket_user_id
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_default();
        serde_json::json!({
            "event": "bts:subscribe",
            "data": {
                "channel": format!("private-my_trades_{}-{}", symbol, user_id),
                "auth": token
            }
        })
        .to_string()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload {
            for key in ["event", "channel"] {
                if let Some(value) = json_payload.get(key) {
                    websocket_text.payload_summary.insert(
                        key.to_string(),
                        value
                            .as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| value.to_string()),
                    );
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
                .get("event")
                .map(String::as_str),
            Some("data" | "trade" | "order_created" | "order_changed" | "order_deleted")
        )
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("channel")
            .is_some_and(|channel| channel.starts_with("order_book_"))
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("channel")
            .is_some_and(|channel| channel.starts_with("live_trades_"))
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("channel")
            .is_some_and(|channel| channel.starts_with("private-my_orders_"))
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("channel")
            .is_some_and(|channel| channel.starts_with("private-my_trades_"))
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
        let data = json_payload.get("data").unwrap_or(&serde_json::Value::Null);
        let channel = websocket_text
            .payload_summary
            .get("channel")
            .map(String::as_str)
            .unwrap_or("");
        let symbol = channel
            .strip_prefix("order_book_")
            .unwrap_or("")
            .to_string();
        let timestamp = data
            .get("microtimestamp")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<i64>().ok())
            .and_then(chrono::DateTime::<chrono::Utc>::from_timestamp_micros)
            .or_else(|| {
                data.get("timestamp")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|value| value.parse::<i64>().ok())
                    .and_then(|value| chrono::DateTime::<chrono::Utc>::from_timestamp(value, 0))
            })
            .unwrap_or_default();
        let bid = data
            .get("bids")
            .and_then(serde_json::Value::as_array)
            .and_then(|values| values.first())
            .and_then(serde_json::Value::as_array);
        let ask = data
            .get("asks")
            .and_then(serde_json::Value::as_array)
            .and_then(|values| values.first())
            .and_then(serde_json::Value::as_array);
        let value_to_string = |value: Option<&serde_json::Value>| {
            value
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| value.to_string())
                })
                .unwrap_or_default()
        };
        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitstamp,
                    symbol,
                    timestamp,
                    bid_price: value_to_string(bid.and_then(|values| values.first())),
                    bid_size: value_to_string(bid.and_then(|values| values.get(1))),
                    ask_price: value_to_string(ask.and_then(|values| values.first())),
                    ask_size: value_to_string(ask.and_then(|values| values.get(1))),
                }],
            },
        )
    }

    fn create_subscribe_trade_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();
        let data = json_payload.get("data").unwrap_or(&serde_json::Value::Null);
        let channel = websocket_text
            .payload_summary
            .get("channel")
            .map(String::as_str)
            .unwrap_or("");
        let symbol = channel
            .strip_prefix("live_trades_")
            .unwrap_or("")
            .to_string();
        let timestamp = data
            .get("microtimestamp")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| value.parse::<i64>().ok())
            .and_then(chrono::DateTime::<chrono::Utc>::from_timestamp_micros)
            .or_else(|| {
                data.get("timestamp")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|value| value.parse::<i64>().ok())
                    .and_then(|value| chrono::DateTime::<chrono::Utc>::from_timestamp(value, 0))
            })
            .unwrap_or_default();
        let value_to_string = |key: &str| {
            data.get(key)
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| value.to_string())
                })
                .unwrap_or_default()
        };
        let side = match data.get("type").and_then(serde_json::Value::as_i64) {
            Some(0) => crate::types::TakerSide::Buy,
            Some(1) => crate::types::TakerSide::Sell,
            _ => crate::types::TakerSide::Unknown,
        };
        crate::exchange_client::common::Response::TradeSubscription(
            crate::exchange_client::common::TradeSubscriptionData {
                data: vec![crate::types::Trade {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitstamp,
                    symbol,
                    timestamp,
                    price: value_to_string("price"),
                    size: value_to_string("amount"),
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
        let data = json_payload.get("data").unwrap_or(&serde_json::Value::Null);
        let event = websocket_text
            .payload_summary
            .get("event")
            .map(String::as_str)
            .unwrap_or("");
        let channel = websocket_text
            .payload_summary
            .get("channel")
            .map(String::as_str)
            .unwrap_or("");
        let private_suffix = channel.strip_prefix("private-my_orders_").unwrap_or("");
        let symbol = private_suffix
            .rsplit_once('-')
            .map(|(symbol, _)| symbol)
            .unwrap_or(private_suffix)
            .to_string();
        let value_to_string = |key: &str| {
            data.get(key)
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| value.to_string())
                })
                .unwrap_or_default()
        };
        let remaining_quantity = value_to_string("amount");
        let status = match event {
            "order_created" | "order_changed" => crate::types::OrderStatus::Open,
            "order_deleted" if remaining_quantity.parse::<f64>().unwrap_or(1.0) == 0.0 => {
                crate::types::OrderStatus::Filled
            }
            "order_deleted" => crate::types::OrderStatus::Canceled,
            _ => crate::types::OrderStatus::Unknown,
        };
        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData {
                data: vec![crate::types::Order {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitstamp,
                    symbol,
                    order_id: value_to_string("id"),
                    client_order_id: value_to_string("client_order_id"),
                    side: self.convert_string_to_order_side(
                        data.get("order_type").unwrap_or(&serde_json::Value::Null),
                    ),
                    price: value_to_string("price"),
                    quantity: value_to_string("amount"),
                    remaining_quantity,
                    status,
                    ..Default::default()
                }],
            },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();
        let data = json_payload.get("data").unwrap_or(&serde_json::Value::Null);
        let channel = websocket_text
            .payload_summary
            .get("channel")
            .map(String::as_str)
            .unwrap_or("");
        let private_suffix = channel.strip_prefix("private-my_trades_").unwrap_or("");
        let symbol = private_suffix
            .rsplit_once('-')
            .map(|(symbol, _)| symbol)
            .unwrap_or(private_suffix)
            .to_string();
        let value_to_string = |key: &str| {
            data.get(key)
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| value.to_string())
                })
                .unwrap_or_default()
        };
        let side = match data.get("type").and_then(serde_json::Value::as_i64) {
            Some(0) => crate::types::OrderSide::Buy,
            Some(1) => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        };
        let price = value_to_string("price");
        let quantity = value_to_string("amount");
        let quote_quantity = match (price.parse::<f64>(), quantity.parse::<f64>()) {
            (Ok(price), Ok(quantity)) => (price * quantity).to_string(),
            _ => String::new(),
        };
        crate::exchange_client::common::Response::FillSubscription(
            crate::exchange_client::common::FillSubscriptionData {
                data: vec![crate::types::Fill {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitstamp,
                    symbol,
                    order_id: value_to_string("order_id"),
                    client_order_id: value_to_string("client_order_id"),
                    side,
                    price,
                    quantity,
                    quote_quantity,
                    is_maker: false,
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
                .get("event")
                .map(String::as_str),
            Some("bts:subscription_succeeded" | "bts:heartbeat")
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
        websocket_text
            .payload_summary
            .get("event")
            .map(String::as_str)
            == Some("bts:subscription_succeeded")
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("event")
            .map(String::as_str)
            == Some("bts:heartbeat")
    }

    fn create_subscribe_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let channel = websocket_text
            .payload_summary
            .get("channel")
            .map(String::as_str)
            .unwrap_or("");
        let (symbol, kind) = if let Some(symbol) = channel.strip_prefix("order_book_") {
            (
                symbol,
                Some(crate::exchange_client::common::SubscribeResponseKind::TopOfBook),
            )
        } else if let Some(symbol) = channel.strip_prefix("live_trades_") {
            (
                symbol,
                Some(crate::exchange_client::common::SubscribeResponseKind::Trade),
            )
        } else if let Some(suffix) = channel.strip_prefix("private-my_orders_") {
            (
                suffix
                    .rsplit_once('-')
                    .map(|(symbol, _)| symbol)
                    .unwrap_or(suffix),
                Some(crate::exchange_client::common::SubscribeResponseKind::Order),
            )
        } else if let Some(suffix) = channel.strip_prefix("private-my_trades_") {
            (
                suffix
                    .rsplit_once('-')
                    .map(|(symbol, _)| symbol)
                    .unwrap_or(suffix),
                Some(crate::exchange_client::common::SubscribeResponseKind::Fill),
            )
        } else {
            ("", None)
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
        let mut response = websocket_text.clone();
        response.error_message = websocket_text
            .json_payload
            .as_ref()
            .and_then(|payload| payload.get("data"))
            .and_then(|data| data.get("message"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);
        crate::exchange_client::common::Response::WebSocketErrorResponse(response)
    }
}

#[cfg(test)]
mod tests {

    use crate::exchange_client::common::Response;
    use crate::exchange_client::websocket::Websocket;
    use crate::exchanges::bitstamp::common::BitstampClient;
    use crate::networking::websocket::WebSocketText;
    use crate::types::{ExchangeInstrumentType, OrderStatus};

    fn sample_order_created_json() -> serde_json::Value {
        serde_json::json!({
            "event": "order_created",
            "channel": "private-my_orders_btcusd-123456",
            "trade_account_id": 0,
            "event_id": "019f4ac8-5e3a-11ef-af00-3e9012000020",
            "pre_event_id": "019f4ac8-5e39-11ef-af00-3e9012000019",
            "order_source": "orderbook",
            "data": {
                "id": 1500000001,
                "id_str": "1500000001",
                "client_order_id": "my-order-001",
                "amount": 0.5,
                "amount_str": "0.50000000",
                "amount_traded": "0.00000000",
                "amount_at_create": "0.50000000",
                "price": 61200.00,
                "price_str": "61200.00",
                "order_type": 0,
                "order_subtype": 0,
                "datetime": "1712131200",
                "microtimestamp": "1712131200000000",
                "trade_account_id": 0,
                "is_liquidation": false
            }
        })
    }

    fn websocket_text_with_json(json_payload: serde_json::Value) -> WebSocketText {
        WebSocketText {
            json_payload: Some(json_payload),
            ..Default::default()
        }
    }

    fn bitstamp_client() -> BitstampClient {
        BitstampClient::default()
    }

    fn websocket_text_with_summary_populated(
        client: &BitstampClient,
        json_payload: serde_json::Value,
    ) -> WebSocketText {
        let mut websocket_text = websocket_text_with_json(json_payload);
        client.populate_websocket_text_payload_summary(&mut websocket_text);
        websocket_text
    }

    #[test]
    fn test_populate_websocket_text_payload_summary() {
        let client = bitstamp_client();
        let mut websocket_text = websocket_text_with_json(sample_order_created_json());

        client.populate_websocket_text_payload_summary(&mut websocket_text);

        assert_eq!(
            websocket_text
                .payload_summary
                .get("event")
                .map(String::as_str),
            Some("order_created")
        );
        assert_eq!(
            websocket_text
                .payload_summary
                .get("channel")
                .map(String::as_str),
            Some("private-my_orders_btcusd-123456")
        );
    }

    #[test]
    fn test_populate_websocket_text_payload_summary_no_json_payload_is_noop() {
        let client = bitstamp_client();
        let mut websocket_text = WebSocketText {
            json_payload: None,
            ..Default::default()
        };

        client.populate_websocket_text_payload_summary(&mut websocket_text);

        assert!(websocket_text.payload_summary.is_empty());
    }

    #[test]
    fn test_is_websocket_text_subscription_data_true_for_order_created() {
        let client = bitstamp_client();
        let websocket_text =
            websocket_text_with_summary_populated(&client, sample_order_created_json());

        assert!(client.is_websocket_text_subscription_data(&websocket_text));
    }

    #[test]
    fn test_is_websocket_text_subscription_data_false_for_non_subscription_event() {
        let client = bitstamp_client();
        let mut json_payload = sample_order_created_json();
        json_payload["event"] = serde_json::json!("bts:subscription_succeeded");
        let websocket_text = websocket_text_with_summary_populated(&client, json_payload);

        assert!(!client.is_websocket_text_subscription_data(&websocket_text));
    }

    #[test]
    fn test_is_websocket_text_order_subscription_data_true_for_order_channel() {
        let client = bitstamp_client();
        let websocket_text =
            websocket_text_with_summary_populated(&client, sample_order_created_json());

        assert!(client.is_websocket_text_order_subscription_data(&websocket_text));
    }

    #[test]
    fn test_is_websocket_text_order_subscription_data_false_for_other_channel() {
        let client = bitstamp_client();
        let mut json_payload = sample_order_created_json();
        json_payload["channel"] = serde_json::json!("live_trades_btcusd");
        let websocket_text = websocket_text_with_summary_populated(&client, json_payload);

        assert!(!client.is_websocket_text_order_subscription_data(&websocket_text));
    }

    #[test]
    fn test_create_subscribe_order_websocket_subscription_data() {
        let client = bitstamp_client();
        let websocket_text =
            websocket_text_with_summary_populated(&client, sample_order_created_json());

        let response = client.create_subscribe_order_websocket_subscription_data(&websocket_text);

        match response {
            Response::OrderSubscription(order_subscription_data) => {
                assert_eq!(order_subscription_data.data.len(), 1);
                let order = &order_subscription_data.data[0];

                assert_eq!(
                    order.exchange_instrument_type,
                    ExchangeInstrumentType::Bitstamp
                );
                assert_eq!(order.symbol, "btcusd");
                assert_eq!(order.order_id, "1500000001");
                assert_eq!(order.client_order_id, "my-order-001");
                assert_eq!(order.price, "61200.0");
                assert_eq!(order.quantity, "0.5");
                assert_eq!(order.remaining_quantity, "0.5");
                assert_eq!(order.status, OrderStatus::Open);
            }
            other => panic!("expected Response::OrderSubscription, got {:?}", other),
        }
    }

    #[test]
    fn test_create_subscribe_order_websocket_subscription_data_deleted_zero_amount_is_filled() {
        let client = bitstamp_client();
        let mut json_payload = sample_order_created_json();
        json_payload["event"] = serde_json::json!("order_deleted");
        json_payload["data"]["amount"] = serde_json::json!(0);
        json_payload["data"]["amount_str"] = serde_json::json!("0.00000000");
        let websocket_text = websocket_text_with_summary_populated(&client, json_payload);

        let response = client.create_subscribe_order_websocket_subscription_data(&websocket_text);

        match response {
            Response::OrderSubscription(order_subscription_data) => {
                assert_eq!(order_subscription_data.data[0].status, OrderStatus::Filled);
            }
            other => panic!("expected Response::OrderSubscription, got {:?}", other),
        }
    }

    #[test]
    fn test_create_subscribe_order_websocket_subscription_data_deleted_nonzero_amount_is_canceled()
    {
        let client = bitstamp_client();
        let mut json_payload = sample_order_created_json();
        json_payload["event"] = serde_json::json!("order_deleted");
        let websocket_text = websocket_text_with_summary_populated(&client, json_payload);

        let response = client.create_subscribe_order_websocket_subscription_data(&websocket_text);

        match response {
            Response::OrderSubscription(order_subscription_data) => {
                assert_eq!(
                    order_subscription_data.data[0].status,
                    OrderStatus::Canceled
                );
            }
            other => panic!("expected Response::OrderSubscription, got {:?}", other),
        }
    }

    fn sample_trade_json() -> serde_json::Value {
        serde_json::json!({
            "event": "trade",
            "channel": "private-my_trades_btcusd-123456",
            "data": {
                "id": 296050733,
                "id_str": "296050733",
                "trade_uti": "BSTP-20240403-296050733",
                "order_id": 1500000001,
                "client_order_id": "my-order-001",
                "amount": 0.1,
                "price": 61200.00,
                "fee": "0.00",
                "side": "buy",
                "microtimestamp": "1712131200500000",
                "trade_account_id": 0,
                "position_id": null,
                "is_liquidation": null,
                "trade_type": "ORDERBOOK"
            }
        })
    }

    #[test]
    fn test_populate_websocket_text_payload_summary_for_trade_event() {
        let client = bitstamp_client();
        let mut websocket_text = websocket_text_with_json(sample_trade_json());

        client.populate_websocket_text_payload_summary(&mut websocket_text);

        assert_eq!(
            websocket_text
                .payload_summary
                .get("event")
                .map(String::as_str),
            Some("trade")
        );
        assert_eq!(
            websocket_text
                .payload_summary
                .get("channel")
                .map(String::as_str),
            Some("private-my_trades_btcusd-123456")
        );
    }

    #[test]
    fn test_is_websocket_text_subscription_data_true_for_trade_event() {
        let client = bitstamp_client();
        let websocket_text = websocket_text_with_summary_populated(&client, sample_trade_json());

        assert!(client.is_websocket_text_subscription_data(&websocket_text));
    }

    #[test]
    fn test_is_websocket_text_fill_subscription_data_true_for_trade_channel() {
        let client = bitstamp_client();
        let websocket_text = websocket_text_with_summary_populated(&client, sample_trade_json());

        assert!(client.is_websocket_text_fill_subscription_data(&websocket_text));
    }

    #[test]
    fn test_is_websocket_text_fill_subscription_data_false_for_order_channel() {
        let client = bitstamp_client();
        let mut json_payload = sample_trade_json();
        json_payload["channel"] = serde_json::json!("private-my_orders_btcusd-123456");
        let websocket_text = websocket_text_with_summary_populated(&client, json_payload);

        assert!(!client.is_websocket_text_fill_subscription_data(&websocket_text));
    }

    #[test]
    fn test_create_subscribe_fill_websocket_subscription_data() {
        let client = bitstamp_client();
        let websocket_text = websocket_text_with_summary_populated(&client, sample_trade_json());

        let response = client.create_subscribe_fill_websocket_subscription_data(&websocket_text);

        match response {
            Response::FillSubscription(fill_subscription_data) => {
                assert_eq!(fill_subscription_data.data.len(), 1);
                let fill = &fill_subscription_data.data[0];

                assert_eq!(
                    fill.exchange_instrument_type,
                    ExchangeInstrumentType::Bitstamp
                );
                assert_eq!(fill.symbol, "btcusd");
                assert_eq!(fill.order_id, "1500000001");
                assert_eq!(fill.client_order_id, "my-order-001");
                assert_eq!(fill.price, "61200.0");
                assert_eq!(fill.quantity, "0.1");
                assert_eq!(fill.quote_quantity, "6120");
                assert!(!fill.is_maker);
                assert_eq!(fill.side, crate::types::OrderSide::Unknown);
            }
            other => panic!("expected Response::FillSubscription, got {:?}", other),
        }
    }
}
