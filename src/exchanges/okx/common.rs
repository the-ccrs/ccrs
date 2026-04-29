#[derive(Debug, Default, Clone)]
pub struct OkxCredential {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: String,
}

#[derive(Debug, Default)]
pub struct OkxClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) instrument_type: crate::types::OkxInstrumentType,
    pub(super) inst_type_str: String,
    pub(super) td_mode: String,
    pub(super) credential: Option<OkxCredential>,
    pub(super) use_demo_trading: bool,
}

impl OkxClient {
    pub fn builder() -> OkxClientBuilder {
        OkxClientBuilder::default()
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
            "live" => crate::types::OrderStatus::Placed,
            "partially_filled" => crate::types::OrderStatus::PartiallyFilled,
            "filled" => crate::types::OrderStatus::Filled,
            "canceled" | "mmp_canceled" => crate::types::OrderStatus::Canceled,
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
        let leverage = {
            let v = &json_value["lever"];
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(n) = v.as_f64() {
                n.to_string()
            } else {
                String::new()
            }
        };

        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Okx(
                self.instrument_type,
            ),
            symbol: json_value["instId"].as_str().unwrap().to_string(),
            order_id: json_value["ordId"].as_str().unwrap().to_string(),
            client_order_id: json_value["clOrdId"].as_str().unwrap().to_string(),
            order_type: self.convert_string_to_order_type(json_value["ordType"].as_str().unwrap()),
            side: self.convert_string_to_order_side(json_value["side"].as_str().unwrap()),
            price: json_value["px"].as_str().unwrap().to_string(),
            quantity: json_value["sz"].as_str().unwrap().to_string(),
            cumulative_filled_quantity: json_value["accFillSz"].as_str().unwrap().to_string(),
            average_filled_price: json_value["avgPx"].as_str().unwrap().to_string(),
            status: self.convert_string_to_order_status(json_value["state"].as_str().unwrap()),
            leverage,
            fill_price: json_value["fillPx"].as_str().unwrap().to_string(),
            fill_quantity: json_value["fillSz"].as_str().unwrap().to_string(),
            fill_is_maker: json_value
                .get("execType")
                .and_then(|v| v.as_str())
                .map(|s| s == "M")
                .unwrap_or(false),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        let leverage = {
            let v = &json_value["lever"];
            if let Some(s) = v.as_str() {
                s.to_string()
            } else if let Some(n) = v.as_f64() {
                n.to_string()
            } else {
                String::new()
            }
        };

        let pos = json_value["pos"].as_str().unwrap();
        let pos_side = json_value["posSide"].as_str().unwrap();
        let symbol = json_value["instId"].as_str().unwrap();

        let mut position_asset = String::new();

        let side = if self.instrument_type == crate::types::OkxInstrumentType::Margin {
            position_asset = json_value["posCcy"].as_str().unwrap_or("").to_string();
            crate::types::PositionSide::Unknown
        } else if !pos.parse::<f64>().unwrap_or(0.0).abs().eq(&0.0) {
            if pos_side == "long" {
                crate::types::PositionSide::Long
            } else if pos_side == "short" {
                crate::types::PositionSide::Short
            } else if matches!(
                self.instrument_type,
                crate::types::OkxInstrumentType::Futures
                    | crate::types::OkxInstrumentType::Swap
                    | crate::types::OkxInstrumentType::Option
            ) {
                if pos.starts_with('-') {
                    crate::types::PositionSide::Short
                } else {
                    crate::types::PositionSide::Long
                }
            } else {
                self.convert_string_to_position_side(pos_side)
            }
        } else {
            self.convert_string_to_position_side(pos_side)
        };

        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Okx(
                self.instrument_type,
            ),
            symbol: symbol.to_string(),
            side,
            entry_price: json_value["avgPx"].as_str().unwrap().to_string(),
            quantity: pos.to_string(),
            leverage,
            position_asset,
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::Okx,
            asset: json_value["ccy"].as_str().unwrap().to_string(),
            quantity: json_value["cashBal"].as_str().unwrap().to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct OkxClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    instrument_type: Option<crate::types::OkxInstrumentType>,
    td_mode: Option<String>,
    credential: Option<OkxCredential>,
    use_demo_trading: Option<bool>,
}

impl OkxClientBuilder {
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

    pub fn instrument_type(mut self, instrument_type: crate::types::OkxInstrumentType) -> Self {
        self.instrument_type = Some(instrument_type);
        self
    }

    pub fn td_mode(mut self, td_mode: impl Into<String>) -> Self {
        self.td_mode = Some(td_mode.into());
        self
    }

    pub fn credential(mut self, credential: Option<OkxCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn use_demo_trading(mut self, use_demo_trading: Option<bool>) -> Self {
        self.use_demo_trading = use_demo_trading;
        self
    }

    pub fn build(self) -> OkxClient {
        let rest_api_base_url = match self.rest_api_base_url {
            Some(u) => u,
            None => "https://www.okx.com".to_string(),
        };

        let instrument_type = match self.instrument_type {
            Some(t) => t,
            None => crate::types::OkxInstrumentType::Spot,
        };

        let inst_type_str = match instrument_type {
            crate::types::OkxInstrumentType::Spot => "SPOT".to_string(),
            crate::types::OkxInstrumentType::Margin => "MARGIN".to_string(),
            crate::types::OkxInstrumentType::Swap => "SWAP".to_string(),
            crate::types::OkxInstrumentType::Futures => "FUTURES".to_string(),
            crate::types::OkxInstrumentType::Option => "OPTION".to_string(),
            crate::types::OkxInstrumentType::Unknown => {
                panic!("OkxInstrumentType::Unknown is not allowed here");
            }
        };

        let td_mode = match self.td_mode {
            Some(m) => m,
            None => match instrument_type {
                crate::types::OkxInstrumentType::Spot => "cash".to_string(),
                _ => "cross".to_string(),
            },
        };

        let websocket_market_data_api_url = match self.websocket_market_data_api_url {
            Some(u) => u,
            None => "wss://ws.okx.com:8443/ws/v5/public".to_string(),
        };

        let websocket_account_data_api_url = match self.websocket_account_data_api_url {
            Some(u) => u,
            None => "wss://ws.okx.com:8443/ws/v5/private".to_string(),
        };

        let use_demo_trading = self.use_demo_trading.unwrap_or(false);

        OkxClient {
            rest_api_base_url,
            websocket_market_data_api_url,
            websocket_account_data_api_url,
            instrument_type,
            inst_type_str,
            td_mode,
            credential: self.credential,
            use_demo_trading,
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for OkxClient {}
