#[derive(Debug, Clone)]
pub struct HtxSpotCredential {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Default)]
pub struct HtxSpotClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) credential: Option<HtxSpotCredential>,
    pub(super) account_id: String,
    pub(super) api_broker_id: String,
}

impl HtxSpotClient {
    pub fn builder() -> HtxSpotClientBuilder {
        HtxSpotClientBuilder::default()
    }

    pub(super) fn convert_order_side_and_type_to_string(
        &self,
        side: crate::types::OrderSide,
        order_type: crate::types::OrderType,
    ) -> &'static str {
        match (side, order_type) {
            (crate::types::OrderSide::Buy, crate::types::OrderType::Market) => "buy-market",
            (crate::types::OrderSide::Buy, crate::types::OrderType::Limit) => "buy-limit",
            (crate::types::OrderSide::Sell, crate::types::OrderType::Market) => "sell-market",
            (crate::types::OrderSide::Sell, crate::types::OrderType::Limit) => "sell-limit",
            _ => panic!("Invalid order side/type combination"),
        }
    }

    pub(super) fn convert_string_to_order_side(&self, s: &str) -> crate::types::OrderSide {
        if s.starts_with("buy") {
            crate::types::OrderSide::Buy
        } else if s.starts_with("sell") {
            crate::types::OrderSide::Sell
        } else {
            crate::types::OrderSide::Unknown
        }
    }

    pub(super) fn convert_string_to_order_type(&self, s: &str) -> crate::types::OrderType {
        if s.ends_with("-market") {
            crate::types::OrderType::Market
        } else if s.ends_with("-limit") || s.ends_with("-maker") || s.ends_with("-ioc") {
            crate::types::OrderType::Limit
        } else {
            crate::types::OrderType::Unknown
        }
    }

    pub(super) fn convert_string_to_order_status(&self, state: &str) -> crate::types::OrderStatus {
        match state {
            "submitted" | "pre-submitted" => crate::types::OrderStatus::Placed,
            "partial-filled" => crate::types::OrderStatus::PartiallyFilled,
            "filled" => crate::types::OrderStatus::Filled,
            "partial-canceled" | "canceled" => crate::types::OrderStatus::Canceled,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn precision_to_increment_string(&self, precision: usize) -> String {
        if precision == 0 {
            "1".to_string()
        } else {
            let mut s = String::from("0.");
            for i in 0..precision {
                if i == precision - 1 {
                    s.push('1');
                } else {
                    s.push('0');
                }
            }
            s
        }
    }

    pub(super) fn json_number_to_string(value: &serde_json::Value) -> String {
        if let Some(n) = value.as_f64() {
            format!("{}", n)
        } else if let Some(s) = value.as_str() {
            s.to_string()
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
        let price_precision = item["price-precision"].as_i64().unwrap_or(0) as usize;
        let qty_precision = item["amount-precision"].as_i64().unwrap_or(0) as usize;

        crate::types::InstrumentInfo {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxSpot,
            symbol: item["symbol"].as_str().unwrap_or("").to_string(),
            base_asset: item["base-currency"].as_str().unwrap_or("").to_string(),
            quote_asset: item["quote-currency"].as_str().unwrap_or("").to_string(),
            order_price_increment: self.precision_to_increment_string(price_precision),
            order_quantity_increment: self.precision_to_increment_string(qty_precision),
            order_quantity_min: Self::json_number_to_string(&item["min-order-amt"]),
            order_quantity_max: Self::json_number_to_string(&item["max-order-amt"]),
            order_quote_quantity_min: Self::json_number_to_string(&item["min-order-value"]),
            order_quote_quantity_max: Self::json_number_to_string(&item["max-order-value"]),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        let type_str = json_value["type"].as_str().unwrap_or("");
        let order_id = json_value["id"]
            .as_u64()
            .map(|v| v.to_string())
            .unwrap_or_default();
        let amount: f64 = json_value["amount"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);
        let filled: f64 = json_value["field-amount"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0.0);

        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxSpot,
            symbol: json_value["symbol"].as_str().unwrap_or("").to_string(),
            order_id,
            client_order_id: json_value["client-order-id"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            order_type: self.convert_string_to_order_type(type_str),
            side: self.convert_string_to_order_side(type_str),
            price: json_value["price"].as_str().unwrap_or("").to_string(),
            quantity: json_value["amount"].as_str().unwrap_or("").to_string(),
            remaining_quantity: format!("{}", amount - filled),
            cumulative_filled_quote_quantity: json_value["field-cash-amount"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            status: self.convert_string_to_order_status(json_value["state"].as_str().unwrap_or("")),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::HtxSpot,
            asset: json_value["currency"].as_str().unwrap_or("").to_string(),
            quantity: json_value["balance"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct HtxSpotClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    credential: Option<HtxSpotCredential>,
    account_id: Option<String>,
}

impl HtxSpotClientBuilder {
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

    pub fn credential(mut self, credential: Option<HtxSpotCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn account_id(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    pub fn build(self) -> HtxSpotClient {
        let rest_api_base_url = self
            .rest_api_base_url
            .unwrap_or_else(|| "https://api.huobi.pro".to_string());

        let websocket_market_data_api_url = self
            .websocket_market_data_api_url
            .unwrap_or_else(|| "wss://api.huobi.pro/ws".to_string());

        let websocket_account_data_api_url = self
            .websocket_account_data_api_url
            .unwrap_or_else(|| "wss://api.huobi.pro/ws/v2".to_string());

        HtxSpotClient {
            rest_api_base_url,
            websocket_market_data_api_url,
            websocket_account_data_api_url,
            credential: self.credential,
            account_id: self.account_id.unwrap_or_default(),
            api_broker_id: "AA3b46363e".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for HtxSpotClient {
    fn prefix_client_order_id(&self, client_order_id: &mut String) {
        let prefix = format!("{}-", self.api_broker_id);
        client_order_id.insert_str(0, &prefix);
    }
}
