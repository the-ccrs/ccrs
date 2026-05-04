#[derive(Debug, Clone)]
pub struct BinanceUsdsMarginedFuturesCredential {
    pub api_key: String,
    pub signing_key: ed25519_dalek::SigningKey,
}

impl BinanceUsdsMarginedFuturesCredential {
    pub fn from_pem_file(api_key: String, path: &str) -> Self {
        let pem = std::fs::read_to_string(path).unwrap();
        Self {
            api_key,
            signing_key: crate::utils::signing_key_from_pkcs8_pem(&pem),
        }
    }
}

#[derive(Debug, Default)]
pub struct BinanceUsdsMarginedFuturesClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) credential: Option<BinanceUsdsMarginedFuturesCredential>,
    pub(super) api_link_id: String,
    pub(super) recv_window: i64,
}

impl BinanceUsdsMarginedFuturesClient {
    pub fn builder() -> BinanceUsdsMarginedFuturesClientBuilder {
        BinanceUsdsMarginedFuturesClientBuilder::default()
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
            exchange_instrument_type:
                crate::types::ExchangeInstrumentType::BinanceUsdsMarginedFutures,
            symbol: json_value["symbol"].as_str().unwrap().to_string(),
            order_id: json_value["orderId"].as_i64().unwrap().to_string(),
            client_order_id: json_value["clientOrderId"].as_str().unwrap().to_string(),
            order_type: self.convert_string_to_order_type(json_value["type"].as_str().unwrap()),
            side: self.convert_string_to_order_side(json_value["side"].as_str().unwrap()),
            price: json_value["price"].as_str().unwrap().to_string(),
            quantity: json_value["origQty"].as_str().unwrap().to_string(),
            cumulative_filled_quantity: json_value["executedQty"].as_str().unwrap().to_string(),
            cumulative_filled_quote_quantity: json_value["cumQuote"].as_str().unwrap().to_string(),
            average_filled_price: json_value["avgPrice"].as_str().unwrap().to_string(),
            status: self.convert_string_to_order_status(json_value["status"].as_str().unwrap()),
            ..Default::default()
        }
    }

    pub(super) fn convert_ws_order_trade_update_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        let o = &json_value["o"];

        crate::types::Order {
            exchange_instrument_type:
                crate::types::ExchangeInstrumentType::BinanceUsdsMarginedFutures,
            symbol: o["s"].as_str().unwrap().to_string(),
            order_id: o["i"].as_u64().unwrap().to_string(),
            client_order_id: o["c"].as_str().unwrap().to_string(),
            order_type: self.convert_string_to_order_type(o["o"].as_str().unwrap()),
            side: self.convert_string_to_order_side(o["S"].as_str().unwrap()),
            price: o["p"].as_str().unwrap().to_string(),
            quantity: o["q"].as_str().unwrap().to_string(),
            cumulative_filled_quantity: o["z"].as_str().unwrap().to_string(),
            status: self.convert_string_to_order_status(o["X"].as_str().unwrap()),
            fill_price: o["L"].as_str().unwrap().to_string(),
            fill_quantity: o["l"].as_str().unwrap().to_string(),
            fill_is_maker: o["m"].as_bool().unwrap_or_default(),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::BinanceUsdsMarginedFutures,
            asset: json_value["asset"].as_str().unwrap().to_string(),
            quantity: json_value["balance"].as_str().unwrap().to_string(),
        }
    }

    pub(super) fn convert_rest_json_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        let position_amt_str = json_value["positionAmt"].as_str().unwrap();
        let position_amt: f64 = position_amt_str.parse().unwrap();

        let side = match json_value["positionSide"].as_str().unwrap() {
            "LONG" => crate::types::PositionSide::Long,
            "SHORT" => crate::types::PositionSide::Short,
            _ => {
                if position_amt > 0.0 {
                    crate::types::PositionSide::Long
                } else if position_amt < 0.0 {
                    crate::types::PositionSide::Short
                } else {
                    crate::types::PositionSide::Unknown
                }
            }
        };

        let quantity = position_amt_str.trim_start_matches('-').to_string();

        crate::types::Position {
            exchange_instrument_type:
                crate::types::ExchangeInstrumentType::BinanceUsdsMarginedFutures,
            symbol: json_value["symbol"].as_str().unwrap().to_string(),
            side,
            entry_price: json_value["entryPrice"].as_str().unwrap().to_string(),
            quantity,
            position_asset: String::new(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BinanceUsdsMarginedFuturesClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    credential: Option<BinanceUsdsMarginedFuturesCredential>,
    recv_window: Option<i64>,
}

impl BinanceUsdsMarginedFuturesClientBuilder {
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

    pub fn credential(mut self, credential: Option<BinanceUsdsMarginedFuturesCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn recv_window(mut self, recv_window: i64) -> Self {
        self.recv_window = Some(recv_window);
        self
    }

    pub fn build(self) -> BinanceUsdsMarginedFuturesClient {
        let rest_api_base_url = self
            .rest_api_base_url
            .unwrap_or_else(|| "https://fapi.binance.com".to_string());

        let websocket_market_data_api_url = self
            .websocket_market_data_api_url
            .unwrap_or_else(|| "wss://fstream.binance.com/public/stream".to_string());

        let websocket_account_data_api_url = self
            .websocket_account_data_api_url
            .unwrap_or_else(|| "wss://fstream.binance.com/private/ws/{listen_key}".to_string());

        BinanceUsdsMarginedFuturesClient {
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
impl crate::exchange_client::common::Common for BinanceUsdsMarginedFuturesClient {
    fn prefix_client_order_id(&self, client_order_id: &mut String) {
        let prefix = format!("x-{}-", self.api_link_id);
        client_order_id.insert_str(0, &prefix);
    }
}
