#[derive(Debug, Clone)]
pub struct BinanceSpotCredential {
    pub api_key: String,
    pub signing_key: ed25519_dalek::SigningKey,
}

impl BinanceSpotCredential {
    pub fn from_pem_file(api_key: String, path: &str) -> Self {
        let pem = std::fs::read_to_string(path).unwrap();
        Self {
            api_key,
            signing_key: crate::utils::signing_key_from_pkcs8_pem(&pem),
        }
    }
}

#[derive(Debug, Default)]
pub struct BinanceSpotClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) credential: Option<BinanceSpotCredential>,
    pub(super) api_link_id: String,
    pub(super) recv_window: i64,
}

impl BinanceSpotClient {
    pub fn builder() -> BinanceSpotClientBuilder {
        BinanceSpotClientBuilder::default()
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

    pub(super) fn convert_string_to_order_side(&self, s: &str) -> crate::types::OrderSide {
        match s {
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

    pub(super) fn convert_string_to_order_type(&self, s: &str) -> crate::types::OrderType {
        match s {
            "MARKET" => crate::types::OrderType::Market,
            "LIMIT" => crate::types::OrderType::Limit,
            _ => crate::types::OrderType::Unknown,
        }
    }

    pub(super) fn convert_string_to_order_status(&self, s: &str) -> crate::types::OrderStatus {
        match s {
            "NEW" => crate::types::OrderStatus::Placed,
            "PARTIALLY_FILLED" => crate::types::OrderStatus::PartiallyFilled,
            "FILLED" => crate::types::OrderStatus::Filled,
            "CANCELED" => crate::types::OrderStatus::Canceled,
            "PENDING_CANCEL" => crate::types::OrderStatus::Canceled,
            "REJECTED" => crate::types::OrderStatus::Rejected,
            "EXPIRED" => crate::types::OrderStatus::Expired,
            "EXPIRED_IN_MATCH" => crate::types::OrderStatus::Expired,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn convert_rest_json_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::BinanceSpot,
            symbol: json_value["symbol"].as_str().unwrap().to_string(),
            order_id: json_value["orderId"].as_i64().unwrap().to_string(),
            client_order_id: json_value["clientOrderId"].as_str().unwrap().to_string(),
            order_type: self.convert_string_to_order_type(json_value["type"].as_str().unwrap()),
            side: self.convert_string_to_order_side(json_value["side"].as_str().unwrap()),
            price: json_value["price"].as_str().unwrap().to_string(),
            quantity: json_value["origQty"].as_str().unwrap().to_string(),
            cumulative_filled_quantity: json_value["executedQty"].as_str().unwrap().to_string(),
            cumulative_filled_quote_quantity: json_value["cummulativeQuoteQty"]
                .as_str()
                .unwrap()
                .to_string(),
            status: self.convert_string_to_order_status(json_value["status"].as_str().unwrap()),
            ..Default::default()
        }
    }

    pub(super) fn convert_ws_executionreport_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        let event = &json_value["event"];

        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::BinanceSpot,
            symbol: event["s"].as_str().unwrap().to_string(),
            order_id: event["i"].as_u64().unwrap().to_string(),
            client_order_id: event["c"].as_str().unwrap().to_string(),
            order_type: self.convert_string_to_order_type(event["o"].as_str().unwrap()),
            side: self.convert_string_to_order_side(event["S"].as_str().unwrap()),
            price: event["p"].as_str().unwrap().to_string(),
            quantity: event["q"].as_str().unwrap().to_string(),
            cumulative_filled_quantity: event["z"].as_str().unwrap().to_string(),
            status: self.convert_string_to_order_status(event["X"].as_str().unwrap()),
            fill_price: event["L"].as_str().unwrap().to_string(),
            fill_quantity: event["l"].as_str().unwrap().to_string(),
            fill_quote_quantity: event["Z"].as_str().unwrap().to_string(),
            fill_is_maker: event["m"].as_bool().unwrap_or(false),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::BinanceSpot,
            asset: json_value["asset"].as_str().unwrap().to_string(),
            quantity: json_value["free"].as_str().unwrap().to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BinanceSpotClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    credential: Option<BinanceSpotCredential>,
    recv_window: Option<i64>,
}

impl BinanceSpotClientBuilder {
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

    pub fn credential(mut self, credential: Option<BinanceSpotCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn recv_window(mut self, recv_window: i64) -> Self {
        self.recv_window = Some(recv_window);
        self
    }

    pub fn build(self) -> BinanceSpotClient {
        let rest_api_base_url = self
            .rest_api_base_url
            .unwrap_or_else(|| "https://api.binance.com".to_string());

        let websocket_market_data_api_url = self
            .websocket_market_data_api_url
            .unwrap_or_else(|| "wss://stream.binance.com:443/stream".to_string());

        let websocket_account_data_api_url = self
            .websocket_account_data_api_url
            .unwrap_or_else(|| "wss://ws-api.binance.com/ws-api/v3".to_string());

        BinanceSpotClient {
            rest_api_base_url,
            websocket_market_data_api_url,
            websocket_account_data_api_url,
            credential: self.credential,
            api_link_id: "XHKUG2CH".to_string(),
            recv_window: self.recv_window.unwrap_or(5000),
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for BinanceSpotClient {
    fn prefix_client_order_id(&self, client_order_id: &mut String) {
        let prefix = format!("x-{}-", self.api_link_id);
        client_order_id.insert_str(0, &prefix);
    }
}
