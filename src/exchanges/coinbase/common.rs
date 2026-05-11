#[derive(Debug, Clone)]
pub struct CoinbaseCredential {
    pub api_key: String,
    pub api_secret: String,
    pub api_passphrase: String,
}

#[derive(Debug, Default)]
pub struct CoinbaseClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) credential: Option<CoinbaseCredential>,
}

impl CoinbaseClient {
    pub fn builder() -> CoinbaseClientBuilder {
        CoinbaseClientBuilder::default()
    }

    pub(super) fn convert_order_side_to_string(
        &self,
        side: crate::types::OrderSide,
    ) -> &'static str {
        match side {
            crate::types::OrderSide::Buy => "buy",
            crate::types::OrderSide::Sell => "sell",
            crate::types::OrderSide::Unknown => panic!("Invalid side"),
        }
    }

    pub(super) fn convert_string_to_order_side(&self, s: &str) -> crate::types::OrderSide {
        match s {
            "buy" => crate::types::OrderSide::Buy,
            "sell" => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        }
    }

    pub(super) fn convert_order_type_to_string(
        &self,
        order_type: crate::types::OrderType,
    ) -> &'static str {
        match order_type {
            crate::types::OrderType::Market => "market",
            crate::types::OrderType::Limit => "limit",
            crate::types::OrderType::Unknown => panic!("Invalid order type"),
        }
    }

    pub(super) fn convert_string_to_order_type(&self, s: &str) -> crate::types::OrderType {
        match s {
            "market" => crate::types::OrderType::Market,
            "limit" => crate::types::OrderType::Limit,
            _ => crate::types::OrderType::Unknown,
        }
    }

    pub(super) fn convert_string_to_order_status(&self, s: &str) -> crate::types::OrderStatus {
        match s {
            "open" | "pending" | "received" | "active" => crate::types::OrderStatus::Placed,
            "done" => crate::types::OrderStatus::Filled,
            "rejected" => crate::types::OrderStatus::Rejected,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
            symbol: json_value["product_id"].as_str().unwrap_or("").to_string(),
            order_id: json_value["id"].as_str().unwrap_or("").to_string(),
            client_order_id: json_value["client_oid"].as_str().unwrap_or("").to_string(),
            order_type: self
                .convert_string_to_order_type(json_value["type"].as_str().unwrap_or("")),
            side: self.convert_string_to_order_side(json_value["side"].as_str().unwrap_or("")),
            price: json_value["price"].as_str().unwrap_or("").to_string(),
            quantity: json_value["size"].as_str().unwrap_or("").to_string(),
            cumulative_filled_quantity: json_value["filled_size"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            cumulative_filled_quote_quantity: json_value["executed_value"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            status: self
                .convert_string_to_order_status(json_value["status"].as_str().unwrap_or("")),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::Coinbase,
            asset: json_value["currency"].as_str().unwrap_or("").to_string(),
            quantity: json_value["balance"].as_str().unwrap_or("").to_string(),
        }
    }

    fn sign_websocket_subscribe(&self, timestamp: &str) -> String {
        let credential = self
            .credential
            .as_ref()
            .expect("Coinbase credential required for WebSocket signing");
        let message = format!("{}GET/users/self/verify", timestamp);
        let key_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            credential.api_secret.as_bytes(),
        )
        .unwrap();
        let mut mac = hmac_sha256::HMAC::new(key_bytes.as_slice());
        mac.update(message.as_bytes());
        let signature_bytes = mac.finalize();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature_bytes)
    }

    pub(super) fn build_subscribe_message(
        &self,
        channel: &str,
        product_ids: &[String],
        with_auth: bool,
    ) -> String {
        let mut payload = serde_json::Map::new();
        payload.insert(
            "type".to_string(),
            serde_json::Value::String("subscribe".to_string()),
        );
        payload.insert(
            "channels".to_string(),
            serde_json::json!([{
                "name": channel,
                "product_ids": product_ids
            }]),
        );
        if with_auth && let Some(credential) = &self.credential {
            let timestamp = chrono::Utc::now().timestamp().to_string();
            let signature = self.sign_websocket_subscribe(&timestamp);
            payload.insert(
                "key".to_string(),
                serde_json::Value::String(credential.api_key.clone()),
            );
            payload.insert(
                "passphrase".to_string(),
                serde_json::Value::String(credential.api_passphrase.clone()),
            );
            payload.insert(
                "timestamp".to_string(),
                serde_json::Value::String(timestamp),
            );
            payload.insert(
                "signature".to_string(),
                serde_json::Value::String(signature),
            );
        }

        serde_json::Value::Object(payload).to_string()
    }

    pub(super) fn convert_ws_user_message_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        let msg_type = json_value["type"].as_str().unwrap_or("");
        match msg_type {
            "received" => crate::types::Order {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
                symbol: json_value["product_id"].as_str().unwrap_or("").to_string(),
                order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
                client_order_id: json_value
                    .get("client_oid")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                order_type: self.convert_string_to_order_type(
                    json_value
                        .get("order_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                ),
                side: self.convert_string_to_order_side(
                    json_value
                        .get("side")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                ),
                price: json_value
                    .get("price")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                quantity: json_value
                    .get("size")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                status: crate::types::OrderStatus::Placed,
                ..Default::default()
            },
            "open" => crate::types::Order {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
                symbol: json_value["product_id"].as_str().unwrap_or("").to_string(),
                order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
                side: self.convert_string_to_order_side(
                    json_value
                        .get("side")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                ),
                price: json_value
                    .get("price")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                remaining_quantity: json_value
                    .get("remaining_size")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                status: crate::types::OrderStatus::Open,
                ..Default::default()
            },
            "done" => {
                let reason = json_value
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let status = match reason {
                    "filled" => crate::types::OrderStatus::Filled,
                    "canceled" => crate::types::OrderStatus::Canceled,
                    _ => crate::types::OrderStatus::Unknown,
                };
                crate::types::Order {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
                    symbol: json_value["product_id"].as_str().unwrap_or("").to_string(),
                    order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
                    side: self.convert_string_to_order_side(
                        json_value
                            .get("side")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ),
                    price: json_value
                        .get("price")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    remaining_quantity: json_value
                        .get("remaining_size")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    status,
                    ..Default::default()
                }
            }
            "change" => crate::types::Order {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
                symbol: json_value["product_id"].as_str().unwrap_or("").to_string(),
                order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
                side: self.convert_string_to_order_side(
                    json_value
                        .get("side")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                ),
                price: json_value
                    .get("price")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                quantity: json_value
                    .get("new_size")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                status: crate::types::OrderStatus::Unknown,
                ..Default::default()
            },
            "activate" => crate::types::Order {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
                symbol: json_value["product_id"].as_str().unwrap_or("").to_string(),
                order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
                side: self.convert_string_to_order_side(
                    json_value
                        .get("side")
                        .and_then(|v| v.as_str())
                        .unwrap_or(""),
                ),
                quantity: json_value
                    .get("size")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                status: crate::types::OrderStatus::Placed,
                ..Default::default()
            },
            "match" => {
                let user_id = json_value
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let taker_user_id = json_value
                    .get("taker_user_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let order_id = if user_id == taker_user_id {
                    json_value
                        .get("taker_order_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                } else {
                    json_value
                        .get("maker_order_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                };

                crate::types::Order {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
                    symbol: json_value["product_id"].as_str().unwrap_or("").to_string(),
                    order_id: order_id.to_string(),
                    side: self.convert_string_to_order_side(
                        json_value
                            .get("side")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ),
                    fill_price: json_value
                        .get("price")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    fill_quantity: json_value
                        .get("size")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    status: crate::types::OrderStatus::Filled,
                    ..Default::default()
                }
            }
            _ => panic!("Unexpected user channel message type: {}", msg_type),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CoinbaseClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    credential: Option<CoinbaseCredential>,
}

impl CoinbaseClientBuilder {
    pub fn rest_api_base_url(mut self, rest_api_base_url: impl Into<String>) -> Self {
        self.rest_api_base_url = Some(rest_api_base_url.into());
        self
    }

    pub fn websocket_market_data_api_url(
        mut self,
        websocket_market_data_api_url: impl Into<String>,
    ) -> Self {
        self.websocket_market_data_api_url = Some(websocket_market_data_api_url.into());
        self
    }

    pub fn websocket_account_data_api_url(
        mut self,
        websocket_account_data_api_url: impl Into<String>,
    ) -> Self {
        self.websocket_account_data_api_url = Some(websocket_account_data_api_url.into());
        self
    }

    pub fn credential(mut self, credential: Option<CoinbaseCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn build(self) -> CoinbaseClient {
        let rest_api_base_url = self
            .rest_api_base_url
            .unwrap_or_else(|| "https://api.exchange.coinbase.com".to_string());

        let websocket_market_data_api_url = self
            .websocket_market_data_api_url
            .unwrap_or_else(|| "wss://ws-feed.exchange.coinbase.com".to_string());

        let websocket_account_data_api_url = self
            .websocket_account_data_api_url
            .unwrap_or_else(|| "wss://ws-feed.exchange.coinbase.com".to_string());

        CoinbaseClient {
            rest_api_base_url,
            websocket_market_data_api_url,
            websocket_account_data_api_url,
            credential: self.credential,
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for CoinbaseClient {
    fn generate_next_client_order_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
