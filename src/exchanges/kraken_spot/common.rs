#[derive(Debug, Clone)]
pub struct KrakenSpotCredential {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Default)]
pub struct KrakenSpotClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) credential: Option<KrakenSpotCredential>,
    pub(super) websocket_token: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

impl KrakenSpotClient {
    pub fn builder() -> KrakenSpotClientBuilder {
        KrakenSpotClientBuilder::default()
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
            "pending" => crate::types::OrderStatus::Placed,
            "open" => crate::types::OrderStatus::Open,
            "closed" => crate::types::OrderStatus::Filled,
            "canceled" => crate::types::OrderStatus::Canceled,
            "expired" => crate::types::OrderStatus::Expired,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        txid: &str,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenSpot,
            symbol: json_value["descr"]["pair"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            order_id: txid.to_string(),
            client_order_id: json_value["cl_ord_id"].as_str().unwrap_or("").to_string(),
            order_type: self.convert_string_to_order_type(
                json_value["descr"]["ordertype"].as_str().unwrap_or(""),
            ),
            side: self
                .convert_string_to_order_side(json_value["descr"]["type"].as_str().unwrap_or("")),
            price: json_value["descr"]["price"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            quantity: json_value["vol"].as_str().unwrap_or("").to_string(),
            remaining_quantity: String::new(),
            cumulative_filled_quantity: json_value["vol_exec"].as_str().unwrap_or("").to_string(),
            cumulative_filled_quote_quantity: json_value["cost"].as_str().unwrap_or("").to_string(),
            average_filled_price: json_value["price"].as_str().unwrap_or("").to_string(),
            status: self
                .convert_string_to_order_status(json_value["status"].as_str().unwrap_or("")),
            ..Default::default()
        }
    }

    pub(super) fn convert_entry_to_balance(
        &self,
        asset: &str,
        quantity: &str,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::KrakenSpot,
            asset: asset.to_string(),
            quantity: quantity.to_string(),
        }
    }

    pub(super) fn convert_executions_order_status(&self, s: &str) -> crate::types::OrderStatus {
        match s {
            "pending_new" => crate::types::OrderStatus::Placed,
            "new" => crate::types::OrderStatus::Open,
            "partially_filled" => crate::types::OrderStatus::PartiallyFilled,
            "filled" => crate::types::OrderStatus::Filled,
            "canceled" => crate::types::OrderStatus::Canceled,
            "expired" => crate::types::OrderStatus::Expired,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn convert_json_value_to_order_from_executions(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenSpot,
            symbol: json_value["symbol"].as_str().unwrap_or("").to_string(),
            order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
            client_order_id: json_value["cl_ord_id"].as_str().unwrap_or("").to_string(),
            order_type: self
                .convert_string_to_order_type(json_value["order_type"].as_str().unwrap_or("")),
            side: self.convert_string_to_order_side(json_value["side"].as_str().unwrap_or("")),
            price: json_value.get("limit_price").map_or(String::new(), |v| {
                v.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| v.to_string())
            }),
            quantity: json_value.get("order_qty").map_or(String::new(), |v| {
                v.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| v.to_string())
            }),
            cumulative_filled_quantity: json_value.get("cum_qty").map_or(String::new(), |v| {
                v.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| v.to_string())
            }),
            cumulative_filled_quote_quantity: json_value.get("cum_cost").map_or(
                String::new(),
                |v| {
                    v.as_str()
                        .map(String::from)
                        .unwrap_or_else(|| v.to_string())
                },
            ),
            average_filled_price: json_value.get("avg_price").map_or(String::new(), |v| {
                v.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| v.to_string())
            }),
            fill_price: json_value.get("last_price").map_or(String::new(), |v| {
                v.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| v.to_string())
            }),
            fill_quantity: json_value.get("last_qty").map_or(String::new(), |v| {
                v.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| v.to_string())
            }),
            fill_quote_quantity: json_value.get("cost").map_or(String::new(), |v| {
                v.as_str()
                    .map(String::from)
                    .unwrap_or_else(|| v.to_string())
            }),
            fill_is_maker: json_value
                .get("liquidity_ind")
                .and_then(|v| v.as_str())
                .map(|s| s == "m")
                .unwrap_or(false),
            status: self
                .convert_executions_order_status(json_value["order_status"].as_str().unwrap_or("")),

            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_fill_from_executions(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Fill {
        crate::types::Fill {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenSpot,
            symbol: json_value["symbol"].as_str().unwrap_or("").to_string(),
            order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
            client_order_id: json_value["cl_ord_id"].as_str().unwrap_or("").to_string(),
            side: match json_value["side"].as_str() {
                Some("buy") => crate::types::OrderSide::Buy,
                Some("sell") => crate::types::OrderSide::Sell,
                _ => crate::types::OrderSide::Unknown,
            },
            price: json_value["last_price"].to_string(),
            quantity: json_value["last_qty"].to_string(),
            quote_quantity: json_value["cost"].to_string(),
            is_maker: false,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct KrakenSpotClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    credential: Option<KrakenSpotCredential>,
}

impl KrakenSpotClientBuilder {
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

    pub fn credential(mut self, credential: Option<KrakenSpotCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn build(self) -> KrakenSpotClient {
        KrakenSpotClient {
            rest_api_base_url: self
                .rest_api_base_url
                .unwrap_or_else(|| "https://api.kraken.com".to_string()),
            websocket_market_data_api_url: self
                .websocket_market_data_api_url
                .unwrap_or_else(|| "wss://ws.kraken.com/v2".to_string()),
            websocket_account_data_api_url: self
                .websocket_account_data_api_url
                .unwrap_or_else(|| "wss://ws-auth.kraken.com/v2".to_string()),
            credential: self.credential,
            websocket_token: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for KrakenSpotClient {}
