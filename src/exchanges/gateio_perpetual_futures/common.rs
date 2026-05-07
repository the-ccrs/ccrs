#[derive(Debug, Clone)]
pub struct GateioPerpetualFuturesCredential {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Default)]
pub struct GateioPerpetualFuturesClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) settle: String,
    pub(super) credential: Option<GateioPerpetualFuturesCredential>,
    pub(super) api_channel_id: String,
}

impl GateioPerpetualFuturesClient {
    pub(super) const REST_API_PREFIX: &'static str = "/api/v4";

    pub fn builder() -> GateioPerpetualFuturesClientBuilder {
        GateioPerpetualFuturesClientBuilder::default()
    }

    pub(super) fn convert_order_side_to_signed_size(
        &self,
        side: crate::types::OrderSide,
        quantity: i64,
    ) -> i64 {
        match side {
            crate::types::OrderSide::Buy => quantity,
            crate::types::OrderSide::Sell => -quantity,
            crate::types::OrderSide::Unknown => panic!("Invalid side"),
        }
    }

    pub(super) fn get_order_side_from_size(&self, size: i64) -> crate::types::OrderSide {
        if size > 0 {
            crate::types::OrderSide::Buy
        } else if size < 0 {
            crate::types::OrderSide::Sell
        } else {
            crate::types::OrderSide::Unknown
        }
    }

    pub(super) fn convert_order_type_to_tif_string(
        &self,
        order_type: crate::types::OrderType,
    ) -> &'static str {
        match order_type {
            crate::types::OrderType::Limit => "gtc",
            crate::types::OrderType::Market => "ioc",
            crate::types::OrderType::Unknown => panic!("Invalid order type"),
        }
    }

    pub(super) fn convert_tif_string_to_order_type(&self, tif: &str) -> crate::types::OrderType {
        match tif {
            "gtc" | "poc" | "fok" => crate::types::OrderType::Limit,
            "ioc" => crate::types::OrderType::Market,
            _ => crate::types::OrderType::Unknown,
        }
    }

    pub(super) fn convert_string_to_order_status(
        &self,
        status: &str,
        finish_as: &str,
    ) -> crate::types::OrderStatus {
        match status {
            "open" => crate::types::OrderStatus::Placed,
            "finished" => match finish_as {
                "filled" => crate::types::OrderStatus::Filled,
                "cancelled" | "ioc" | "auto_deleveraged" | "reduce_only" | "position_closed"
                | "reduce_out" | "stp" | "liquidated" => crate::types::OrderStatus::Canceled,
                _ => crate::types::OrderStatus::Unknown,
            },
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn get_position_side_from_mode_and_size(
        &self,
        mode: &str,
        size: i64,
    ) -> crate::types::PositionSide {
        match mode {
            "dual_long" => crate::types::PositionSide::Long,
            "dual_short" => crate::types::PositionSide::Short,
            _ => {
                if size > 0 {
                    crate::types::PositionSide::Long
                } else if size < 0 {
                    crate::types::PositionSide::Short
                } else {
                    crate::types::PositionSide::Unknown
                }
            }
        }
    }

    pub(super) fn extract_order_id_from_json(&self, json_value: &serde_json::Value) -> String {
        if let Some(id) = json_value.get("id") {
            if let Some(n) = id.as_i64() {
                return n.to_string();
            } else if let Some(s) = id.as_str() {
                return s.to_string();
            }
        }
        String::new()
    }

    pub(super) fn convert_json_value_to_instrument_info(
        &self,
        item: &serde_json::Value,
    ) -> crate::types::InstrumentInfo {
        let symbol = item["name"].as_str().unwrap_or("").to_string();
        let parts: Vec<&str> = symbol.splitn(2, '_').collect();
        let base_asset = parts.first().copied().unwrap_or("").to_string();
        let quote_asset = parts.get(1).copied().unwrap_or("").to_string();

        let order_size_min = if let Some(v) = item.get("order_size_min") {
            if let Some(n) = v.as_i64() {
                n.to_string()
            } else if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                "0".to_string()
            }
        } else {
            "0".to_string()
        };

        let order_size_max = if let Some(v) = item.get("order_size_max") {
            if let Some(n) = v.as_i64() {
                n.to_string()
            } else if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                "0".to_string()
            }
        } else {
            "0".to_string()
        };

        let contract_size = item
            .get("quanto_multiplier")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        crate::types::InstrumentInfo {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::GateioPerpetualFutures,
            symbol,
            base_asset,
            quote_asset,
            settle_asset: self.settle.to_uppercase(),
            contract_size,
            order_price_increment: item["order_price_round"].as_str().unwrap_or("").to_string(),
            order_quantity_increment: "1".to_string(),
            order_quantity_min: order_size_min,
            order_quantity_max: order_size_max,
            order_quote_quantity_min: "0".to_string(),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        let size = json_value["size"].as_i64().unwrap_or(0);
        let left = json_value["left"].as_i64().unwrap_or(0);
        let status_str = json_value["status"].as_str().unwrap_or("");
        let finish_as = json_value["finish_as"].as_str().unwrap_or("");
        let tif = json_value["tif"].as_str().unwrap_or("gtc");
        let price = json_value["price"].as_str().unwrap_or("").to_string();

        let order_id = self.extract_order_id_from_json(json_value);
        let status = self.convert_string_to_order_status(status_str, finish_as);
        let order_type = self.convert_tif_string_to_order_type(tif);
        let side = self.get_order_side_from_size(size);
        let quantity = size.abs().to_string();
        let remaining_quantity = left.abs().to_string();
        let cumulative_filled_quantity = (size.abs() - left.abs()).max(0).to_string();
        let fill_price = json_value["fill_price"].as_str().unwrap_or("").to_string();

        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::GateioPerpetualFutures,
            symbol: json_value["contract"].as_str().unwrap_or("").to_string(),
            order_id,
            client_order_id: json_value["text"].as_str().unwrap_or("").to_string(),
            order_type,
            side,
            price,
            quantity,
            remaining_quantity,
            cumulative_filled_quantity,
            average_filled_price: fill_price.clone(),
            fill_price,
            status,
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        let size = json_value["size"].as_i64().unwrap_or(0);
        let mode = json_value["mode"].as_str().unwrap_or("");
        let side = self.get_position_side_from_mode_and_size(mode, size);

        let leverage = {
            let v = &json_value["leverage"];
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(n) = v.as_f64() {
                n.to_string()
            } else {
                String::new()
            }
        };

        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::GateioPerpetualFutures,
            symbol: json_value["contract"].as_str().unwrap_or("").to_string(),
            side,
            entry_price: json_value["entry_price"].as_str().unwrap_or("").to_string(),
            quantity: size.to_string(),
            leverage,
            ..Default::default()
        }
    }

    pub(super) fn convert_balance_entry_to_balance(
        &self,
        currency: &str,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::GateioPerpetualFutures,
            asset: currency.to_string(),
            quantity: json_value["available"].as_str().unwrap_or("").to_string(),
        }
    }

    pub(super) fn sign_websocket_request(
        &self,
        channel: &str,
        event: &str,
        timestamp: i64,
    ) -> Option<serde_json::Value> {
        self.credential.as_ref().map(|cred| {
            let message = format!("channel={}&event={}&time={}", channel, event, timestamp);
            let mut mac = <hmac::Hmac<sha2::Sha512> as hmac::digest::KeyInit>::new_from_slice(
                cred.api_secret.as_bytes(),
            )
            .unwrap();
            <hmac::Hmac<sha2::Sha512> as hmac::Mac>::update(&mut mac, message.as_bytes());
            let sign =
                hex::encode(<hmac::Hmac<sha2::Sha512> as hmac::Mac>::finalize(mac).into_bytes());
            serde_json::json!({
                "method": "api_key",
                "KEY": cred.api_key,
                "SIGN": sign
            })
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct GateioPerpetualFuturesClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    settle: Option<String>,
    credential: Option<GateioPerpetualFuturesCredential>,
}

impl GateioPerpetualFuturesClientBuilder {
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

    pub fn settle(mut self, settle: impl Into<String>) -> Self {
        self.settle = Some(settle.into());
        self
    }

    pub fn credential(mut self, credential: Option<GateioPerpetualFuturesCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn build(self) -> GateioPerpetualFuturesClient {
        let settle = self
            .settle
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "usdt".to_string());

        let rest_api_base_url = self
            .rest_api_base_url
            .unwrap_or_else(|| "https://api.gateio.ws".to_string());

        let websocket_market_data_api_url = self
            .websocket_market_data_api_url
            .unwrap_or_else(|| format!("wss://fx-ws.gateio.ws/v4/ws/{}", settle));

        let websocket_account_data_api_url = self
            .websocket_account_data_api_url
            .unwrap_or_else(|| format!("wss://fx-ws.gateio.ws/v4/ws/{}", settle));

        GateioPerpetualFuturesClient {
            rest_api_base_url,
            websocket_market_data_api_url,
            websocket_account_data_api_url,
            settle,
            credential: self.credential,
            api_channel_id: "cryptochassis2".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for GateioPerpetualFuturesClient {
    fn prefix_client_order_id(&self, client_order_id: &mut String) {
        client_order_id.insert_str(0, "t-");
    }
}
