#[derive(Debug, Default, Clone)]
pub struct BybitCredential {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Default)]
pub struct BybitClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) instrument_type: crate::types::BybitInstrumentType,
    pub(super) credential: Option<BybitCredential>,
    pub(super) api_broker_id: String,
    pub(super) category: String,
    pub(super) api_receive_window_milliseconds: i64,
}

impl BybitClient {
    pub fn builder() -> BybitClientBuilder {
        BybitClientBuilder::default()
    }

    pub(super) fn convert_order_side_to_string(
        &self,
        side: crate::types::OrderSide,
    ) -> &'static str {
        match side {
            crate::types::OrderSide::Buy => "Buy",
            crate::types::OrderSide::Sell => "Sell",
            crate::types::OrderSide::Unknown => panic!("Invalid side"),
        }
    }

    pub(super) fn convert_string_to_order_side(&self, s: &str) -> crate::types::OrderSide {
        match s {
            "Buy" => crate::types::OrderSide::Buy,
            "Sell" => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        }
    }

    pub(super) fn convert_order_type_to_string(
        &self,
        order_type: crate::types::OrderType,
    ) -> &'static str {
        match order_type {
            crate::types::OrderType::Market => "Market",
            crate::types::OrderType::Limit => "Limit",
            crate::types::OrderType::Unknown => panic!("Invalid order type"),
        }
    }

    pub(super) fn convert_string_to_order_type(&self, s: &str) -> crate::types::OrderType {
        match s {
            "Market" => crate::types::OrderType::Market,
            "Limit" => crate::types::OrderType::Limit,
            _ => crate::types::OrderType::Unknown,
        }
    }

    pub(super) fn convert_string_to_order_status(&self, s: &str) -> crate::types::OrderStatus {
        match s {
            "New" => crate::types::OrderStatus::Placed,
            "PartiallyFilled" => crate::types::OrderStatus::PartiallyFilled,
            "Rejected" => crate::types::OrderStatus::Rejected,
            "PartiallyFilledCanceled" => crate::types::OrderStatus::Canceled,
            "Filled" => crate::types::OrderStatus::Filled,
            "Cancelled" => crate::types::OrderStatus::Canceled,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn convert_string_to_position_side(&self, s: &str) -> crate::types::PositionSide {
        match s {
            "Buy" => crate::types::PositionSide::Long,
            "Sell" => crate::types::PositionSide::Short,
            _ => crate::types::PositionSide::Unknown,
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Bybit(
                self.instrument_type,
            ),
            symbol: json_value["symbol"].as_str().unwrap().to_string(),
            order_id: json_value["orderId"].as_str().unwrap().to_string(),
            client_order_id: json_value["orderLinkId"].as_str().unwrap().to_string(),
            order_type: self
                .convert_string_to_order_type(json_value["orderType"].as_str().unwrap()),
            side: self.convert_string_to_order_side(json_value["side"].as_str().unwrap()),
            price: json_value["price"].as_str().unwrap().to_string(),
            quantity: json_value["qty"].as_str().unwrap().to_string(),
            remaining_quantity: json_value["leavesQty"].as_str().unwrap().to_string(),
            cumulative_filled_quantity: json_value["cumExecQty"].as_str().unwrap().to_string(),
            cumulative_filled_quote_quantity: json_value["cumExecValue"]
                .as_str()
                .unwrap()
                .to_string(),
            average_filled_price: json_value["avgPrice"].as_str().unwrap().to_string(),
            status: self
                .convert_string_to_order_status(json_value["orderStatus"].as_str().unwrap()),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Bybit(
                self.instrument_type,
            ),
            symbol: json_value["symbol"].as_str().unwrap().to_string(),
            side: self.convert_string_to_position_side(json_value["side"].as_str().unwrap()),
            entry_price: json_value["avgPrice"].as_str().unwrap().to_string(),
            quantity: json_value["size"].as_str().unwrap().to_string(),
            leverage: json_value["leverage"].as_str().unwrap().to_string(),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::Bybit,

            asset: json_value["coin"].as_str().unwrap().to_string(),

            quantity: json_value["walletBalance"].as_str().unwrap().to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BybitClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    instrument_type: Option<crate::types::BybitInstrumentType>,
    credential: Option<BybitCredential>,
    api_receive_window_milliseconds: Option<i64>,
}

impl BybitClientBuilder {
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

    pub fn instrument_type(mut self, instrument_type: crate::types::BybitInstrumentType) -> Self {
        self.instrument_type = Some(instrument_type);
        self
    }

    pub fn credential(mut self, credential: Option<BybitCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn api_receive_window_milliseconds(mut self, api_receive_window_milliseconds: i64) -> Self {
        self.api_receive_window_milliseconds = Some(api_receive_window_milliseconds);
        self
    }

    pub fn build(self) -> BybitClient {
        let rest_api_base_url = self
            .rest_api_base_url
            .unwrap_or_else(|| "https://api.bybit.com".to_string());

        let instrument_type = self
            .instrument_type
            .unwrap_or(crate::types::BybitInstrumentType::Spot);

        let category = match instrument_type {
            crate::types::BybitInstrumentType::Spot => "spot".to_string(),
            crate::types::BybitInstrumentType::Linear => "linear".to_string(),
            crate::types::BybitInstrumentType::Inverse => "inverse".to_string(),
            crate::types::BybitInstrumentType::Unknown => {
                panic!("BybitInstrumentType::Unknown is not allowed here");
            }
        };

        let websocket_market_data_api_url = self
            .websocket_market_data_api_url
            .unwrap_or_else(|| format!("wss://stream.bybit.com/v5/public/{}", category));

        let websocket_account_data_api_url = self
            .websocket_account_data_api_url
            .unwrap_or_else(|| "wss://stream.bybit.com/v5/private".to_string());

        BybitClient {
            rest_api_base_url,
            websocket_market_data_api_url,
            websocket_account_data_api_url,
            instrument_type: self
                .instrument_type
                .unwrap_or(crate::types::BybitInstrumentType::Spot),
            credential: self.credential,
            api_broker_id: "Vs000261".to_string(),
            category,
            api_receive_window_milliseconds: self.api_receive_window_milliseconds.unwrap_or(5000),
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for BybitClient {}
