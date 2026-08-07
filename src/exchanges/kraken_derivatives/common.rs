#[derive(Debug, Clone)]
pub struct KrakenDerivativesCredential {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Default)]
pub struct KrakenDerivativesClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_api_url: String,
    pub(super) credential: Option<KrakenDerivativesCredential>,
    pub(super) original_challenge: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    pub(super) signed_challenge: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

impl KrakenDerivativesClient {
    pub fn builder() -> KrakenDerivativesClientBuilder {
        KrakenDerivativesClientBuilder::default()
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
            crate::types::OrderType::Market => "mkt",
            crate::types::OrderType::Limit => "lmt",
            crate::types::OrderType::Unknown => panic!("Invalid order type"),
        }
    }

    pub(super) fn convert_string_to_order_type(&self, s: &str) -> crate::types::OrderType {
        match s {
            "lmt" | "post" | "ioc" | "fok" => crate::types::OrderType::Limit,
            "mkt" => crate::types::OrderType::Market,
            _ => crate::types::OrderType::Unknown,
        }
    }

    pub(super) fn convert_string_to_order_status(&self, s: &str) -> crate::types::OrderStatus {
        match s {
            "untouched" => crate::types::OrderStatus::Open,
            "partiallyFilled" => crate::types::OrderStatus::PartiallyFilled,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn convert_string_to_position_side(&self, s: &str) -> crate::types::PositionSide {
        match s {
            "long" => crate::types::PositionSide::Long,
            "short" => crate::types::PositionSide::Short,
            _ => crate::types::PositionSide::Unknown,
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        let filled = json_value["filledSize"].as_f64().unwrap_or(0.0);
        let unfilled = json_value["unfilledSize"].as_f64().unwrap_or(0.0);
        let total = filled + unfilled;

        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenDerivatives,
            symbol: json_value["symbol"].as_str().unwrap_or("").to_string(),
            order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
            client_order_id: json_value["cliOrdId"].as_str().unwrap_or("").to_string(),
            order_type: self
                .convert_string_to_order_type(json_value["orderType"].as_str().unwrap_or("")),
            side: self.convert_string_to_order_side(json_value["side"].as_str().unwrap_or("")),
            price: json_value["limitPrice"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default(),
            quantity: total.to_string(),
            remaining_quantity: unfilled.to_string(),
            cumulative_filled_quantity: filled.to_string(),
            status: self
                .convert_string_to_order_status(json_value["status"].as_str().unwrap_or("")),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenDerivatives,
            symbol: json_value["symbol"].as_str().unwrap_or("").to_string(),
            side: self.convert_string_to_position_side(json_value["side"].as_str().unwrap_or("")),
            entry_price: json_value["price"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default(),
            quantity: json_value["size"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default(),
            ..Default::default()
        }
    }

    pub(super) fn convert_entry_to_balance(
        &self,
        asset: &str,
        quantity: &str,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::KrakenDerivatives,
            asset: asset.to_string(),
            quantity: quantity.to_string(),
        }
    }

    pub(super) fn convert_ws_order_json_to_order(
        &self,
        json_value: &serde_json::Value,
        is_cancel: bool,
    ) -> crate::types::Order {
        let qty = json_value["qty"].as_f64().unwrap_or(0.0);
        let filled = json_value["filled"].as_f64().unwrap_or(0.0);

        let status = if is_cancel {
            crate::types::OrderStatus::Canceled
        } else if qty > 0.0 && filled >= qty {
            crate::types::OrderStatus::Filled
        } else if filled > 0.0 {
            crate::types::OrderStatus::PartiallyFilled
        } else {
            crate::types::OrderStatus::Open
        };

        let side = match json_value["direction"].as_i64() {
            Some(0) => crate::types::OrderSide::Buy,
            Some(1) => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        };

        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenDerivatives,
            symbol: json_value["instrument"].as_str().unwrap_or("").to_string(),
            order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
            client_order_id: json_value["cli_ord_id"].as_str().unwrap_or("").to_string(),
            order_type: self
                .convert_string_to_order_type(json_value["type"].as_str().unwrap_or("")),
            side,
            price: json_value["limit_price"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default(),
            quantity: qty.to_string(),
            cumulative_filled_quantity: filled.to_string(),
            remaining_quantity: (qty - filled).to_string(),
            status,
            ..Default::default()
        }
    }

    pub(super) fn convert_ws_cancel_json_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenDerivatives,
            order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
            client_order_id: json_value["cli_ord_id"].as_str().unwrap_or("").to_string(),
            status: crate::types::OrderStatus::Canceled,
            ..Default::default()
        }
    }

    pub(super) fn convert_ws_fill_json_to_fill(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Fill {
        let side = if json_value["buy"].as_bool().unwrap_or(false) {
            crate::types::OrderSide::Buy
        } else {
            crate::types::OrderSide::Sell
        };

        let is_maker = json_value["fill_type"].as_str().unwrap_or("") == "maker";

        crate::types::Fill {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenDerivatives,
            symbol: json_value["instrument"].as_str().unwrap_or("").to_string(),
            order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
            client_order_id: json_value["cli_ord_id"].as_str().unwrap_or("").to_string(),
            side,
            price: json_value["price"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default(),
            quantity: json_value["qty"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default(),
            quote_quantity: String::new(),
            is_maker,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct KrakenDerivativesClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_api_url: Option<String>,
    credential: Option<KrakenDerivativesCredential>,
}

impl KrakenDerivativesClientBuilder {
    pub fn rest_api_base_url(mut self, rest_api_base_url: impl Into<String>) -> Self {
        self.rest_api_base_url = Some(rest_api_base_url.into());
        self
    }

    pub fn websocket_api_url(mut self, websocket_api_url: impl Into<String>) -> Self {
        self.websocket_api_url = Some(websocket_api_url.into());
        self
    }

    pub fn credential(mut self, credential: Option<KrakenDerivativesCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn build(self) -> KrakenDerivativesClient {
        KrakenDerivativesClient {
            rest_api_base_url: self
                .rest_api_base_url
                .unwrap_or_else(|| "https://futures.kraken.com".to_string()),
            websocket_api_url: self
                .websocket_api_url
                .unwrap_or_else(|| "wss://futures.kraken.com/ws/v1".to_string()),
            credential: self.credential,
            original_challenge: std::sync::Arc::new(std::sync::Mutex::new(None)),
            signed_challenge: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for KrakenDerivativesClient {}
