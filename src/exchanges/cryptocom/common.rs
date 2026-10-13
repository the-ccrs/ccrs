#[derive(Debug, Clone)]
pub struct CryptocomCredential {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Default)]
pub struct CryptocomClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) credential: Option<CryptocomCredential>,
}

impl CryptocomClient {
    pub fn builder() -> CryptocomClientBuilder {
        CryptocomClientBuilder::default()
    }

    pub(super) fn convert_order_side_to_string(
        &self,
        side: crate::types::OrderSide,
    ) -> &'static str {
        match side {
            crate::types::OrderSide::Buy => "BUY",
            crate::types::OrderSide::Sell => "SELL",
            crate::types::OrderSide::Unknown => panic!("Invalid side"),
        }
    }

    pub(super) fn convert_string_to_order_side(&self, value: &str) -> crate::types::OrderSide {
        match value {
            "BUY" => crate::types::OrderSide::Buy,
            "SELL" => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        }
    }

    pub(super) fn convert_order_type_to_string(
        &self,
        order_type: crate::types::OrderType,
    ) -> &'static str {
        match order_type {
            crate::types::OrderType::Market => "MARKET",
            crate::types::OrderType::Limit => "LIMIT",
            crate::types::OrderType::Unknown => panic!("Invalid order type"),
        }
    }

    pub(super) fn convert_string_to_order_type(&self, value: &str) -> crate::types::OrderType {
        match value {
            "MARKET" => crate::types::OrderType::Market,
            "LIMIT" => crate::types::OrderType::Limit,
            _ => crate::types::OrderType::Unknown,
        }
    }

    pub(super) fn convert_string_to_order_status(&self, value: &str) -> crate::types::OrderStatus {
        match value {
            "PENDING" => crate::types::OrderStatus::Placed,
            "NEW" => crate::types::OrderStatus::Open,
            "ACTIVE" => crate::types::OrderStatus::Open,
            "FILLED" => crate::types::OrderStatus::Filled,
            "CANCELED" => crate::types::OrderStatus::Canceled,
            "REJECTED" => crate::types::OrderStatus::Rejected,
            "EXPIRED" => crate::types::OrderStatus::Expired,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn json_value_to_string(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(value) => value.clone(),
            serde_json::Value::Number(value) => value.to_string(),
            _ => String::new(),
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        value: &serde_json::Value,
    ) -> crate::types::Order {
        let quantity = Self::json_value_to_string(&value["quantity"]);
        let cumulative_quantity = Self::json_value_to_string(&value["cumulative_quantity"]);
        let status = if value["status"].as_str() == Some("ACTIVE")
            && cumulative_quantity.parse::<f64>().unwrap_or(0.0) > 0.0
        {
            crate::types::OrderStatus::PartiallyFilled
        } else {
            self.convert_string_to_order_status(value["status"].as_str().unwrap_or(""))
        };
        let remaining_quantity = match (quantity.parse::<f64>(), cumulative_quantity.parse::<f64>())
        {
            (Ok(quantity), Ok(cumulative)) => (quantity - cumulative).to_string(),
            _ => String::new(),
        };
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Cryptocom,
            symbol: value["instrument_name"].as_str().unwrap_or("").to_string(),
            order_id: Self::json_value_to_string(&value["order_id"]),
            client_order_id: value["client_oid"].as_str().unwrap_or("").to_string(),
            order_type: self
                .convert_string_to_order_type(value["order_type"].as_str().unwrap_or("")),
            side: self.convert_string_to_order_side(value["side"].as_str().unwrap_or("")),
            price: Self::json_value_to_string(&value["limit_price"]),
            quantity,
            remaining_quantity,
            cumulative_filled_quantity: cumulative_quantity,
            cumulative_filled_quote_quantity: Self::json_value_to_string(
                &value["cumulative_value"],
            ),
            average_filled_price: Self::json_value_to_string(&value["avg_price"]),
            status,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CryptocomClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    credential: Option<CryptocomCredential>,
}

impl CryptocomClientBuilder {
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

    pub fn credential(mut self, credential: Option<CryptocomCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn build(self) -> CryptocomClient {
        CryptocomClient {
            rest_api_base_url: self
                .rest_api_base_url
                .unwrap_or_else(|| "https://api.crypto.com/exchange/v1".to_string()),
            websocket_market_data_api_url: self
                .websocket_market_data_api_url
                .unwrap_or_else(|| "wss://stream.crypto.com/exchange/v1/market".to_string()),
            websocket_account_data_api_url: self
                .websocket_account_data_api_url
                .unwrap_or_else(|| "wss://stream.crypto.com/exchange/v1/user".to_string()),
            credential: self.credential,
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for CryptocomClient {}
