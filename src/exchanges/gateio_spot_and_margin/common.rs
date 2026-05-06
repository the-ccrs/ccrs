#[derive(Debug, Clone)]
pub struct GateioSpotAndMarginCredential {
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, Default)]
pub struct GateioSpotAndMarginClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_market_data_api_url: String,
    pub(super) websocket_account_data_api_url: String,
    pub(super) instrument_type: crate::types::GateioSpotAndMarginInstrumentType,
    pub(super) credential: Option<GateioSpotAndMarginCredential>,
    pub(super) account: String,
    pub(super) api_channel_id: String,
}

impl GateioSpotAndMarginClient {
    pub(super) const REST_API_PREFIX: &'static str = "/api/v4";

    pub fn builder() -> GateioSpotAndMarginClientBuilder {
        GateioSpotAndMarginClientBuilder::default()
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

    pub(super) fn convert_string_to_order_status(
        &self,
        event: &str,
        finish_as: &str,
    ) -> crate::types::OrderStatus {
        match event {
            "put" => crate::types::OrderStatus::Placed,
            "update" => crate::types::OrderStatus::PartiallyFilled,
            "finish" => match finish_as {
                "open" => crate::types::OrderStatus::Placed,
                "filled" => crate::types::OrderStatus::Filled,
                "cancelled"
                | "ioc"
                | "stp"
                | "poc"
                | "fok"
                | "trader_not_enough"
                | "depth_not_enough"
                | "small"
                | "liquidate_cancelled" => crate::types::OrderStatus::Canceled,
                _ => crate::types::OrderStatus::Unknown,
            },
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

    pub(super) fn convert_json_value_to_instrument_info(
        &self,
        item: &serde_json::Value,
    ) -> crate::types::InstrumentInfo {
        let price_precision = item["precision"].as_i64().unwrap_or(0) as usize;
        let qty_precision = item["amount_precision"].as_i64().unwrap_or(0) as usize;

        crate::types::InstrumentInfo {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::GateioSpotAndMargin(
                self.instrument_type,
            ),
            symbol: item["id"].as_str().unwrap_or("").to_string(),
            base_asset: item["base"].as_str().unwrap_or("").to_string(),
            quote_asset: item["quote"].as_str().unwrap_or("").to_string(),
            order_price_increment: self.precision_to_increment_string(price_precision),
            order_quantity_increment: self.precision_to_increment_string(qty_precision),
            order_quantity_min: item["min_base_amount"].as_str().unwrap_or("").to_string(),
            order_quantity_max: item["max_base_amount"].as_str().unwrap_or("").to_string(),
            order_quote_quantity_min: item["min_quote_amount"].as_str().unwrap_or("").to_string(),
            order_quote_quantity_max: item["max_quote_amount"].as_str().unwrap_or("").to_string(),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::GateioSpotAndMargin(
                self.instrument_type,
            ),
            symbol: json_value["currency_pair"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            order_id: json_value["id"].as_str().unwrap_or("").to_string(),
            client_order_id: json_value["text"].as_str().unwrap_or("").to_string(),
            order_type: self
                .convert_string_to_order_type(json_value["type"].as_str().unwrap_or("")),
            side: self.convert_string_to_order_side(json_value["side"].as_str().unwrap_or("")),
            price: json_value["price"].as_str().unwrap_or("").to_string(),
            quantity: json_value["amount"].as_str().unwrap_or("").to_string(),
            remaining_quantity: json_value["left"].as_str().unwrap_or("").to_string(),
            cumulative_filled_quote_quantity: json_value["filled_total"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            status: self.convert_string_to_order_status(
                json_value["event"].as_str().unwrap_or(""),
                json_value["finish_as"].as_str().unwrap_or(""),
            ),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::GateioSpotAndMargin,
            asset: json_value["currency"].as_str().unwrap_or("").to_string(),
            quantity: json_value["available"].as_str().unwrap_or("").to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GateioSpotAndMarginClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    instrument_type: Option<crate::types::GateioSpotAndMarginInstrumentType>,
    credential: Option<GateioSpotAndMarginCredential>,
}

impl GateioSpotAndMarginClientBuilder {
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

    pub fn instrument_type(
        mut self,
        instrument_type: crate::types::GateioSpotAndMarginInstrumentType,
    ) -> Self {
        self.instrument_type = Some(instrument_type);
        self
    }

    pub fn credential(mut self, credential: Option<GateioSpotAndMarginCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn build(self) -> GateioSpotAndMarginClient {
        let instrument_type = self
            .instrument_type
            .unwrap_or(crate::types::GateioSpotAndMarginInstrumentType::Spot);

        let account = match instrument_type {
            crate::types::GateioSpotAndMarginInstrumentType::Spot => "spot".to_string(),
            crate::types::GateioSpotAndMarginInstrumentType::Margin => "margin".to_string(),
            crate::types::GateioSpotAndMarginInstrumentType::Unknown => {
                panic!("GateioSpotAndMarginInstrumentType::Unknown is not allowed here");
            }
        };

        let rest_api_base_url = self
            .rest_api_base_url
            .unwrap_or_else(|| "https://api.gateio.ws".to_string());

        let websocket_market_data_api_url = self
            .websocket_market_data_api_url
            .unwrap_or_else(|| "wss://api.gateio.ws/ws/v4/".to_string());

        let websocket_account_data_api_url = self
            .websocket_account_data_api_url
            .unwrap_or_else(|| "wss://api.gateio.ws/ws/v4/".to_string());

        GateioSpotAndMarginClient {
            rest_api_base_url,
            websocket_market_data_api_url,
            websocket_account_data_api_url,
            instrument_type,
            credential: self.credential,
            account,
            api_channel_id: "cryptochassis2".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for GateioSpotAndMarginClient {
    fn prefix_client_order_id(&self, client_order_id: &mut String) {
        let prefix = "t-";
        client_order_id.insert_str(0, prefix);
    }
}
