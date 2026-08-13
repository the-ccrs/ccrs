#[derive(Debug, Clone)]
pub struct BitgetCredential {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: String,
}

#[derive(Debug, Default)]
pub struct BitgetClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) instrument_type: crate::types::BitgetInstrumentType,
    pub(super) credential: Option<BitgetCredential>,
    pub(super) category: String,
    pub(super) api_channel_api_code: String,
    pub(super) use_demo_trading: bool,
}

impl BitgetClient {
    pub fn builder() -> BitgetClientBuilder {
        BitgetClientBuilder::default()
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
            "new" | "init" | "live" => crate::types::OrderStatus::Placed,
            "partially_filled" => crate::types::OrderStatus::PartiallyFilled,
            "filled" => crate::types::OrderStatus::Filled,
            "canceled" | "cancelled" => crate::types::OrderStatus::Canceled,
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
        let size_str = json_value["qty"].as_str().unwrap_or("0");
        let filled_qty_str = json_value["cumExecQty"].as_str().unwrap_or("0");

        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitget(
                self.instrument_type,
            ),
            symbol: json_value["symbol"].as_str().unwrap_or("").to_string(),
            order_id: json_value["orderId"].as_str().unwrap_or("").to_string(),
            client_order_id: json_value["clientOid"].as_str().unwrap_or("").to_string(),
            order_type: self
                .convert_string_to_order_type(json_value["orderType"].as_str().unwrap_or("")),
            side: self.convert_string_to_order_side(json_value["side"].as_str().unwrap_or("")),
            price: json_value["price"].as_str().unwrap_or("").to_string(),
            quantity: size_str.to_string(),
            cumulative_filled_quantity: filled_qty_str.to_string(),
            cumulative_filled_quote_quantity: json_value["cumExecValue"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            average_filled_price: json_value["avgPrice"].as_str().unwrap_or("").to_string(),
            status: self
                .convert_string_to_order_status(json_value["orderStatus"].as_str().unwrap_or("")),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitget(
                self.instrument_type,
            ),
            symbol: json_value["symbol"].as_str().unwrap_or("").to_string(),
            side: self
                .convert_string_to_position_side(json_value["posSide"].as_str().unwrap_or("")),
            entry_price: json_value["avgPrice"].as_str().unwrap_or("").to_string(),
            quantity: json_value["total"].as_str().unwrap_or("").to_string(),
            leverage: json_value["leverage"].as_str().unwrap_or("").to_string(),
            position_asset: json_value["marginCoin"].as_str().unwrap_or("").to_string(),
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::Bitget,
            asset: json_value["coin"].as_str().unwrap_or("").to_string(),
            quantity: json_value["available"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BitgetClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    instrument_type: Option<crate::types::BitgetInstrumentType>,
    credential: Option<BitgetCredential>,
    use_demo_trading: Option<bool>,
}

impl BitgetClientBuilder {
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

    pub fn instrument_type(mut self, instrument_type: crate::types::BitgetInstrumentType) -> Self {
        self.instrument_type = Some(instrument_type);
        self
    }

    pub fn credential(mut self, credential: Option<BitgetCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn use_demo_trading(mut self, use_demo_trading: Option<bool>) -> Self {
        self.use_demo_trading = use_demo_trading;
        self
    }

    pub fn build(self) -> BitgetClient {
        let rest_api_base_url = self
            .rest_api_base_url
            .unwrap_or_else(|| "https://api.bitget.com".to_string());

        let instrument_type = self
            .instrument_type
            .unwrap_or(crate::types::BitgetInstrumentType::Spot);

        let category = match instrument_type {
            crate::types::BitgetInstrumentType::Spot => "SPOT".to_string(),
            crate::types::BitgetInstrumentType::UsdtFutures => "USDT-FUTURES".to_string(),
            crate::types::BitgetInstrumentType::CoinFutures => "COIN-FUTURES".to_string(),
            crate::types::BitgetInstrumentType::Unknown => {
                panic!("BitgetInstrumentType::Unknown is not allowed here");
            }
        };

        let websocket_market_data_api_url = self
            .websocket_market_data_api_url
            .unwrap_or_else(|| "wss://ws.bitget.com/v3/ws/public".to_string());

        let websocket_account_data_api_url = self
            .websocket_account_data_api_url
            .unwrap_or_else(|| "wss://ws.bitget.com/v3/ws/private".to_string());

        let use_demo_trading = self.use_demo_trading.unwrap_or(false);

        BitgetClient {
            rest_api_base_url,
            websocket_market_data_api_url,
            websocket_account_data_api_url,
            instrument_type,
            credential: self.credential,
            category,
            api_channel_api_code: "95mpa".to_string(),
            use_demo_trading,
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for BitgetClient {}

pub(super) fn category_to_instrument_type(category: &str) -> crate::types::BitgetInstrumentType {
    match category.to_uppercase().as_str() {
        "SPOT" => crate::types::BitgetInstrumentType::Spot,
        "USDT-FUTURES" => crate::types::BitgetInstrumentType::UsdtFutures,
        "COIN-FUTURES" => crate::types::BitgetInstrumentType::CoinFutures,
        other => panic!("Unknown instrument category: {other}"),
    }
}
