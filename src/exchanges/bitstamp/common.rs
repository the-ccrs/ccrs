#[derive(Debug, Clone)]
pub struct BitstampCredential {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Default)]
pub struct BitstampClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) credential: Option<BitstampCredential>,
    pub(super) websocket_token: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    pub(super) websocket_user_id: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

impl BitstampClient {
    pub fn builder() -> BitstampClientBuilder {
        BitstampClientBuilder::default()
    }

    pub fn is_instrument_derivatives(symbol: &str) -> bool {
        let suffix = "-perp";

        if symbol.len() < suffix.len() {
            return false;
        }

        let end_slice = &symbol[symbol.len() - suffix.len()..];

        end_slice.eq_ignore_ascii_case(suffix)
    }

    pub(super) fn convert_string_to_order_side(
        &self,
        value: &serde_json::Value,
    ) -> crate::types::OrderSide {
        match value
            .as_str()
            .or_else(|| value.as_i64().map(|v| if v == 0 { "0" } else { "1" }))
        {
            Some("0") => crate::types::OrderSide::Buy,
            Some("1") => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        }
    }

    pub(super) fn convert_string_to_order_type(&self, value: &str) -> crate::types::OrderType {
        match value {
            "LIMIT"
            | "STOP_LOSS_LIMIT"
            | "TAKE_PROFIT_LIMIT"
            | "TRAILING_STOP_LOSS_LIMIT"
            | "TRAILING_TAKE_PROFIT_LIMIT" => crate::types::OrderType::Limit,
            "MARKET"
            | "STOP_LOSS"
            | "TAKE_PROFIT"
            | "TRAILING_STOP_LOSS"
            | "TRAILING_TAKE_PROFIT" => crate::types::OrderType::Market,
            _ => crate::types::OrderType::Unknown,
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        value: &serde_json::Value,
    ) -> crate::types::Order {
        let quantity = value["amount_at_create"]
            .as_str()
            .or_else(|| value["amount"].as_str())
            .unwrap_or("")
            .to_string();
        let remaining_quantity = value["amount"].as_str().unwrap_or("").to_string();
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitstamp,
            symbol: value["market"]
                .as_str()
                .or_else(|| value["currency_pair"].as_str())
                .unwrap_or("")
                .to_string(),
            order_id: value["id"].as_str().map(str::to_string).unwrap_or_else(|| {
                value["id"]
                    .as_u64()
                    .map(|v| v.to_string())
                    .unwrap_or_default()
            }),
            client_order_id: value["client_order_id"].as_str().unwrap_or("").to_string(),
            order_type: self.convert_string_to_order_type(value["subtype"].as_str().unwrap_or("")),
            side: self.convert_string_to_order_side(&value["type"]),
            price: value["price"].as_str().unwrap_or("").to_string(),
            quantity,
            leverage: value["leverage"].as_str().unwrap_or("").to_string(),
            remaining_quantity,
            status: crate::types::OrderStatus::Open,
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_position(
        &self,
        value: &serde_json::Value,
    ) -> crate::types::Position {
        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitstamp,
            symbol: value["market"].as_str().unwrap_or("").to_string(),
            side: match value["side"].as_str() {
                Some("LONG") => crate::types::PositionSide::Long,
                Some("SHORT") => crate::types::PositionSide::Short,
                _ => crate::types::PositionSide::Unknown,
            },
            entry_price: value["entry_price"].as_str().unwrap_or("").to_string(),
            quantity: value["size"]
                .as_str()
                .unwrap_or("")
                .strip_prefix('-')
                .unwrap_or("")
                .to_string(),
            leverage: value["leverage"].as_str().unwrap_or("").to_string(),
            position_asset: value["settlement_currency"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BitstampClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    credential: Option<BitstampCredential>,
}

impl BitstampClientBuilder {
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

    pub fn credential(mut self, credential: Option<BitstampCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn build(self) -> BitstampClient {
        BitstampClient {
            rest_api_base_url: self
                .rest_api_base_url
                .unwrap_or_else(|| "https://www.bitstamp.net".to_string()),
            websocket_market_data_api_url: self
                .websocket_market_data_api_url
                .unwrap_or_else(|| "wss://ws.bitstamp.net".to_string()),
            websocket_account_data_api_url: self
                .websocket_account_data_api_url
                .unwrap_or_else(|| "wss://ws.bitstamp.net".to_string()),
            credential: self.credential,
            websocket_token: std::sync::Arc::new(std::sync::Mutex::new(None)),
            websocket_user_id: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for BitstampClient {}
