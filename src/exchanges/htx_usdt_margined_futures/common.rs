#[derive(Debug, Clone)]
pub struct HtxUsdtMarginedFuturesCredential {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Default)]
pub struct HtxUsdtMarginedFuturesClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) credential: Option<HtxUsdtMarginedFuturesCredential>,
    pub(super) api_broker_id: String,
}

impl HtxUsdtMarginedFuturesClient {
    pub fn builder() -> HtxUsdtMarginedFuturesClientBuilder {
        HtxUsdtMarginedFuturesClientBuilder::default()
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
            crate::types::OrderType::Limit => "limit",
            crate::types::OrderType::Market => "market",
            crate::types::OrderType::Unknown => panic!("Invalid order type"),
        }
    }

    pub(super) fn convert_string_to_order_type(&self, s: &str) -> crate::types::OrderType {
        match s {
            "limit" | "post_only" | "maker_only" | "ioc" | "fok" => crate::types::OrderType::Limit,
            _ => crate::types::OrderType::Market,
        }
    }

    pub(super) fn convert_status_to_order_status(&self, status: &str) -> crate::types::OrderStatus {
        match status {
            "new" => crate::types::OrderStatus::Placed,
            "partially_filled" => crate::types::OrderStatus::PartiallyFilled,
            "filled" => crate::types::OrderStatus::Filled,
            "partially_canceled" | "canceled" => crate::types::OrderStatus::Canceled,
            "rejected" => crate::types::OrderStatus::Rejected,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn json_number_to_string(value: &serde_json::Value) -> String {
        if let Some(s) = value.as_str() {
            s.to_string()
        } else if let Some(n) = value.as_f64() {
            format!("{}", n)
        } else {
            String::new()
        }
    }

    pub(super) fn percent_encode_htx(s: &str) -> String {
        let mut result = String::new();
        for byte in s.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                _ => {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        result
    }

    pub(super) fn convert_json_value_to_instrument_info(
        &self,
        item: &serde_json::Value,
    ) -> crate::types::InstrumentInfo {
        let symbol = item["contract_code"].as_str().unwrap_or("").to_string();
        let base_asset = item["symbol"].as_str().unwrap_or("").to_string();
        let quote_asset = "USDT".to_string();
        let settle_asset = "USDT".to_string();
        let order_price_increment = Self::json_number_to_string(&item["price_tick"]);
        let contract_size = Self::json_number_to_string(&item["contract_size"]);

        let expiry_timestamp = {
            let delivery_time_str = item
                .get("delivery_time")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if delivery_time_str.is_empty() || delivery_time_str == "0" {
                chrono::DateTime::<chrono::Utc>::default()
            } else {
                match delivery_time_str.parse::<i64>() {
                    Ok(ms) => crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ms),
                    Err(_) => chrono::DateTime::<chrono::Utc>::default(),
                }
            }
        };

        crate::types::InstrumentInfo {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxUsdtMarginedFutures,
            symbol,
            base_asset,
            quote_asset,
            settle_asset,
            contract_size,
            order_price_increment,
            order_quantity_increment: "1".to_string(),
            order_quantity_min: "1".to_string(),
            order_quantity_max: String::new(),
            order_quote_quantity_min: String::new(),
            order_quote_quantity_max: String::new(),
            expiry_timestamp,
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        let order_id = json_value["order_id"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_default();

        let client_order_id = json_value["client_order_id"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_default();

        let order_type = json_value["type"].as_str().unwrap_or("");

        let side = json_value["side"].as_str().unwrap_or("");

        let status = json_value["state"].as_str().unwrap_or("");

        let quantity = Self::json_number_to_string(&json_value["volume"]);

        let cumulative_filled_quantity = Self::json_number_to_string(&json_value["trade_volume"]);

        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxUsdtMarginedFutures,
            symbol: json_value["contract_code"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            order_id,
            client_order_id,
            order_type: self.convert_string_to_order_type(order_type),
            side: self.convert_string_to_order_side(side),
            price: Self::json_number_to_string(&json_value["price"]),
            quantity,
            leverage: json_value["lever_rate"]
                .as_i64()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            cumulative_filled_quantity,
            cumulative_filled_quote_quantity: Self::json_number_to_string(
                &json_value["trade_turnover"],
            ),
            average_filled_price: Self::json_number_to_string(&json_value["trade_avg_price"]),
            fill_price: Self::json_number_to_string(&json_value["trade_avg_price"]),
            fill_quantity: Self::json_number_to_string(&json_value["trade_volume"]),
            fill_quote_quantity: Self::json_number_to_string(&json_value["trade_turnover"]),
            fill_is_maker: false,
            status: self.convert_status_to_order_status(status),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        let direction = json_value["direction"].as_str().unwrap_or("");

        let side = match direction {
            "buy" => crate::types::PositionSide::Long,
            "sell" => crate::types::PositionSide::Short,
            _ => crate::types::PositionSide::Unknown,
        };

        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxUsdtMarginedFutures,
            symbol: json_value["contract_code"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            side,
            entry_price: Self::json_number_to_string(&json_value["open_avg_price"]),
            quantity: Self::json_number_to_string(&json_value["volume"]),
            leverage: json_value["lever_rate"]
                .as_i64()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            position_asset: "USDT".to_string(),
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::HtxUsdtMarginedFutures,
            asset: json_value["currency"].as_str().unwrap_or("").to_string(),
            quantity: Self::json_number_to_string(&json_value["equity"]),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct HtxUsdtMarginedFuturesClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    credential: Option<HtxUsdtMarginedFuturesCredential>,
    lever_rate: Option<u32>,
    offset: Option<String>,
}

impl HtxUsdtMarginedFuturesClientBuilder {
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

    pub fn credential(mut self, credential: Option<HtxUsdtMarginedFuturesCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn lever_rate(mut self, lever_rate: u32) -> Self {
        self.lever_rate = Some(lever_rate);
        self
    }

    pub fn offset(mut self, offset: impl Into<String>) -> Self {
        self.offset = Some(offset.into());
        self
    }

    pub fn build(self) -> HtxUsdtMarginedFuturesClient {
        let rest_api_base_url = self
            .rest_api_base_url
            .unwrap_or_else(|| "https://api.hbdm.com".to_string());

        let websocket_market_data_api_url = self
            .websocket_market_data_api_url
            .unwrap_or_else(|| "wss://api.hbdm.com/linear-swap-ws".to_string());

        let websocket_account_data_api_url = self
            .websocket_account_data_api_url
            .unwrap_or_else(|| "wss://api.hbdm.com/ws/v5/notification".to_string());

        HtxUsdtMarginedFuturesClient {
            rest_api_base_url,
            websocket_market_data_api_url,
            websocket_account_data_api_url,
            credential: self.credential,
            api_broker_id: "AA3b46363e".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for HtxUsdtMarginedFuturesClient {}
