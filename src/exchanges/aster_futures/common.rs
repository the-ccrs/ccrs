#[derive(Clone)]
pub struct AsterFuturesCredential {
    pub signing_key: alloy::signers::local::PrivateKeySigner,
}

impl AsterFuturesCredential {
    pub fn new(signing_key: alloy::signers::local::PrivateKeySigner) -> Self {
        Self { signing_key }
    }
}

impl std::fmt::Debug for AsterFuturesCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = format!("0x{}", hex::encode(self.signing_key.to_bytes()));
        f.debug_struct("AsterFuturesCredential")
            .field("signing_key", &crate::utils::mask_hex_key(&key))
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct AsterFuturesClient {
    pub(super) rest_api_base_url: String,
    #[allow(dead_code)]
    pub(super) websocket_market_data_api_url: String,
    #[allow(dead_code)]
    pub(super) websocket_account_data_api_url: String,
    pub(super) credential: Option<AsterFuturesCredential>,
}

impl AsterFuturesClient {
    pub fn builder() -> AsterFuturesClientBuilder {
        AsterFuturesClientBuilder::default()
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
            "NEW" => crate::types::OrderStatus::Placed,
            "PARTIALLY_FILLED" => crate::types::OrderStatus::PartiallyFilled,
            "FILLED" => crate::types::OrderStatus::Filled,
            "CANCELED" => crate::types::OrderStatus::Canceled,
            "REJECTED" => crate::types::OrderStatus::Rejected,
            "EXPIRED" => crate::types::OrderStatus::Expired,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn convert_rest_json_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::AsterFutures,
            symbol: json_value["symbol"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            order_id: json_value["orderId"]
                .as_i64()
                .unwrap_or_default()
                .to_string(),
            client_order_id: json_value["clientOrderId"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            order_type: self
                .convert_string_to_order_type(json_value["type"].as_str().unwrap_or_default()),
            side: self
                .convert_string_to_order_side(json_value["side"].as_str().unwrap_or_default()),
            price: json_value["price"].as_str().unwrap_or_default().to_string(),
            quantity: json_value["origQty"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            cumulative_filled_quantity: json_value["executedQty"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            cumulative_filled_quote_quantity: json_value["cumQuote"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            average_filled_price: json_value["avgPrice"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            status: self
                .convert_string_to_order_status(json_value["status"].as_str().unwrap_or_default()),
            ..Default::default()
        }
    }

    pub(super) fn convert_ws_order_trade_update_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        let order = &json_value["o"];

        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::AsterFutures,
            symbol: order["s"].as_str().unwrap().to_string(),
            order_id: order["i"].as_u64().unwrap().to_string(),
            client_order_id: order["c"].as_str().unwrap().to_string(),
            order_type: self.convert_string_to_order_type(order["o"].as_str().unwrap()),
            side: self.convert_string_to_order_side(order["S"].as_str().unwrap()),
            price: order["p"].as_str().unwrap().to_string(),
            quantity: order["q"].as_str().unwrap().to_string(),
            cumulative_filled_quantity: order["z"].as_str().unwrap().to_string(),
            average_filled_price: order["ap"].as_str().unwrap().to_string(),
            status: self.convert_string_to_order_status(order["X"].as_str().unwrap()),
            fill_price: order["L"].as_str().unwrap().to_string(),
            fill_quantity: order["l"].as_str().unwrap().to_string(),
            fill_is_maker: order["m"].as_bool().unwrap_or_default(),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::AsterFutures,
            asset: json_value["asset"].as_str().unwrap_or_default().to_string(),
            quantity: json_value["balance"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        }
    }

    pub(super) fn convert_rest_json_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        let position_amount = json_value["positionAmt"].as_str().unwrap_or_default();
        let parsed_position_amount = position_amount.parse::<f64>().unwrap_or_default();
        let side = match json_value["positionSide"].as_str().unwrap_or_default() {
            "LONG" => crate::types::PositionSide::Long,
            "SHORT" => crate::types::PositionSide::Short,
            _ if parsed_position_amount > 0.0 => crate::types::PositionSide::Long,
            _ if parsed_position_amount < 0.0 => crate::types::PositionSide::Short,
            _ => crate::types::PositionSide::Unknown,
        };

        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::AsterFutures,
            symbol: json_value["symbol"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            side,
            entry_price: json_value["entryPrice"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            quantity: position_amount.trim_start_matches('-').to_string(),
            leverage: json_value["leverage"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            ..Default::default()
        }
    }

    pub(super) fn compute_signing_hash(message: &str) -> alloy::primitives::B256 {
        let domain_type_hash = alloy::primitives::keccak256(
            b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        );
        let name_hash = alloy::primitives::keccak256(b"AsterSignTransaction");
        let version_hash = alloy::primitives::keccak256(b"1");
        let mut chain_id = [0u8; 32];
        chain_id[30..].copy_from_slice(&1666u16.to_be_bytes());
        let mut domain_data = Vec::with_capacity(160);
        domain_data.extend_from_slice(domain_type_hash.as_slice());
        domain_data.extend_from_slice(name_hash.as_slice());
        domain_data.extend_from_slice(version_hash.as_slice());
        domain_data.extend_from_slice(&chain_id);
        domain_data.extend_from_slice(&[0u8; 32]);
        let domain_separator = alloy::primitives::keccak256(domain_data);

        let message_type_hash = alloy::primitives::keccak256(b"Message(string msg)");
        let message_hash = alloy::primitives::keccak256(message.as_bytes());
        let mut message_data = Vec::with_capacity(64);
        message_data.extend_from_slice(message_type_hash.as_slice());
        message_data.extend_from_slice(message_hash.as_slice());
        let struct_hash = alloy::primitives::keccak256(message_data);

        let mut signing_data = Vec::with_capacity(66);
        signing_data.extend_from_slice(&[0x19, 0x01]);
        signing_data.extend_from_slice(domain_separator.as_slice());
        signing_data.extend_from_slice(struct_hash.as_slice());
        alloy::primitives::keccak256(signing_data)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AsterFuturesClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_market_data_api_url: Option<String>,
    websocket_account_data_api_url: Option<String>,
    credential: Option<AsterFuturesCredential>,
}

impl AsterFuturesClientBuilder {
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

    pub fn credential(mut self, credential: Option<AsterFuturesCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn build(self) -> AsterFuturesClient {
        AsterFuturesClient {
            rest_api_base_url: self
                .rest_api_base_url
                .unwrap_or_else(|| "https://fapi.asterdex.com".to_string()),
            websocket_market_data_api_url: self
                .websocket_market_data_api_url
                .unwrap_or_else(|| "wss://fstream.asterdex.com/stream".to_string()),
            websocket_account_data_api_url: self
                .websocket_account_data_api_url
                .unwrap_or_else(|| "wss://fstream.asterdex.com/ws/{listen_key}".to_string()),
            credential: self.credential,
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for AsterFuturesClient {}
