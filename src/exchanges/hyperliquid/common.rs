pub struct HyperliquidCredential {
    pub signing_key: alloy::signers::local::PrivateKeySigner,
}

impl HyperliquidCredential {
    pub fn new(signing_key: alloy::signers::local::PrivateKeySigner) -> Self {
        Self { signing_key }
    }
}

impl std::fmt::Debug for HyperliquidCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = hex::encode(self.signing_key.to_bytes());
        let key = format!("0x{key}");

        f.debug_struct("HyperliquidCredential")
            .field("signing_key", &crate::utils::mask_hex_key(&key))
            .finish()
    }
}

pub struct HyperliquidClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_url: String,
    pub(super) instrument_type: crate::types::HyperliquidInstrumentType,
    pub(super) credential: Option<HyperliquidCredential>,
    pub(super) is_mainnet: bool,
    pub(super) wallet_address: String,
}

impl std::fmt::Debug for HyperliquidClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyperliquidClient")
            .field("rest_api_base_url", &self.rest_api_base_url)
            .field("instrument_type", &self.instrument_type)
            .field("credential", &self.credential)
            .field("is_mainnet", &self.is_mainnet)
            .finish()
    }
}

impl HyperliquidClient {
    pub fn builder() -> HyperliquidClientBuilder {
        HyperliquidClientBuilder::default()
    }

    pub(super) fn convert_order_side_to_bool(&self, side: crate::types::OrderSide) -> bool {
        match side {
            crate::types::OrderSide::Buy => true,
            crate::types::OrderSide::Sell => false,
            crate::types::OrderSide::Unknown => panic!("Invalid side"),
        }
    }

    pub(super) fn convert_side_str_to_order_side(&self, s: &str) -> crate::types::OrderSide {
        match s {
            "B" => crate::types::OrderSide::Buy,
            "A" => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        }
    }

    pub(super) fn convert_string_to_order_type(&self, s: &str) -> crate::types::OrderType {
        match s {
            "Market" => crate::types::OrderType::Market,
            "Limit" => crate::types::OrderType::Limit,
            _ => crate::types::OrderType::Unknown,
        }
    }

    pub(super) fn compute_connection_id(
        action: &serde_json::Value,
        nonce: u64,
    ) -> alloy::primitives::B256 {
        let mut bytes = rmp_serde::to_vec_named(action).expect("Failed to msgpack-encode action");
        bytes.extend_from_slice(&nonce.to_be_bytes());
        bytes.push(0u8);
        alloy::primitives::keccak256(&bytes)
    }

    pub(super) fn compute_agent_signing_hash(
        connection_id: alloy::primitives::B256,
        is_mainnet: bool,
    ) -> alloy::primitives::B256 {
        let domain_type_hash = alloy::primitives::keccak256(
            b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
        );
        let name_hash = alloy::primitives::keccak256(b"Exchange");
        let version_hash = alloy::primitives::keccak256(b"1");

        let mut chain_id_bytes = [0u8; 32];
        chain_id_bytes[30] = 0x05;
        chain_id_bytes[31] = 0x39;

        let verifying_contract_bytes = [0u8; 32];

        let mut domain_data = Vec::with_capacity(160);
        domain_data.extend_from_slice(domain_type_hash.as_slice());
        domain_data.extend_from_slice(name_hash.as_slice());
        domain_data.extend_from_slice(version_hash.as_slice());
        domain_data.extend_from_slice(&chain_id_bytes);
        domain_data.extend_from_slice(&verifying_contract_bytes);
        let domain_separator = alloy::primitives::keccak256(&domain_data);

        let agent_type_hash =
            alloy::primitives::keccak256(b"Agent(string source,bytes32 connectionId)");
        let source = if is_mainnet { "a" } else { "b" };
        let source_hash = alloy::primitives::keccak256(source.as_bytes());

        let mut struct_data = Vec::with_capacity(96);
        struct_data.extend_from_slice(agent_type_hash.as_slice());
        struct_data.extend_from_slice(source_hash.as_slice());
        struct_data.extend_from_slice(connection_id.as_slice());
        let struct_hash = alloy::primitives::keccak256(&struct_data);

        let mut signing_data = Vec::with_capacity(66);
        signing_data.push(0x19u8);
        signing_data.push(0x01u8);
        signing_data.extend_from_slice(domain_separator.as_slice());
        signing_data.extend_from_slice(struct_hash.as_slice());
        alloy::primitives::keccak256(&signing_data)
    }

    pub(super) fn sign_action(&self, action: &serde_json::Value, nonce: u64) -> serde_json::Value {
        let connection_id = Self::compute_connection_id(action, nonce);
        let signing_hash = Self::compute_agent_signing_hash(connection_id, self.is_mainnet);
        let credential = self
            .credential
            .as_ref()
            .expect("Credential required for signing");
        let sig = <alloy::signers::local::PrivateKeySigner as alloy::signers::SignerSync>::sign_hash_sync(
            &credential.signing_key,
            &signing_hash,
        )
        .expect("Failed to sign action");
        let r = format!("0x{:064x}", sig.r());
        let s = format!("0x{:064x}", sig.s());
        let v = 27u64 + (sig.v() as u64);
        serde_json::json!({ "r": r, "s": s, "v": v })
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        json_value: &serde_json::Value,
        order_status: crate::types::OrderStatus,
    ) -> crate::types::Order {
        let oid = json_value["oid"].as_u64().unwrap_or(0);
        let cloid = json_value
            .get("cloid")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Hyperliquid(
                self.instrument_type,
            ),
            symbol: json_value["coin"].as_str().unwrap_or("").to_string(),
            order_id: oid.to_string(),
            client_order_id: cloid,
            order_type: self
                .convert_string_to_order_type(json_value["orderType"].as_str().unwrap_or("")),
            side: self.convert_side_str_to_order_side(json_value["side"].as_str().unwrap_or("")),
            price: json_value["limitPx"].as_str().unwrap_or("").to_string(),
            quantity: json_value["origSz"].as_str().unwrap_or("").to_string(),
            remaining_quantity: json_value["sz"].as_str().unwrap_or("").to_string(),
            status: order_status,
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        let pos = &json_value["position"];
        let szi_str = pos["szi"].as_str().unwrap_or("0");
        let szi_float: f64 = szi_str.parse().unwrap_or(0.0);
        let side = if szi_float > 0.0 {
            crate::types::PositionSide::Long
        } else if szi_float < 0.0 {
            crate::types::PositionSide::Short
        } else {
            crate::types::PositionSide::Unknown
        };
        let quantity = szi_str.trim_start_matches('-').to_string();
        let leverage = pos["leverage"]["value"].as_u64().unwrap_or(0).to_string();
        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Hyperliquid(
                crate::types::HyperliquidInstrumentType::Perpetuals,
            ),
            symbol: pos["coin"].as_str().unwrap_or("").to_string(),
            side,
            entry_price: pos["entryPx"].as_str().unwrap_or("").to_string(),
            quantity,
            leverage,
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::Hyperliquid,
            asset: json_value["coin"].as_str().unwrap_or("").to_string(),
            quantity: json_value["total"].as_str().unwrap_or("").to_string(),
        }
    }

    pub(super) fn convert_symbol_to_instrument_type(
        &self,
        symbol: &str,
    ) -> crate::types::HyperliquidInstrumentType {
        if symbol.starts_with('@') || symbol == "PURR/USDC" {
            crate::types::HyperliquidInstrumentType::Spot
        } else {
            crate::types::HyperliquidInstrumentType::Perpetuals
        }
    }
}

#[derive(Debug, Default)]
pub struct HyperliquidClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_url: Option<String>,
    instrument_type: Option<crate::types::HyperliquidInstrumentType>,
    credential: Option<HyperliquidCredential>,
    is_mainnet: Option<bool>,
    wallet_address: Option<String>,
}

impl HyperliquidClientBuilder {
    pub fn rest_api_base_url(mut self, rest_api_base_url: impl Into<String>) -> Self {
        self.rest_api_base_url = Some(rest_api_base_url.into());
        self
    }

    pub fn websocket_url(mut self, websocket_url: impl Into<String>) -> Self {
        self.websocket_url = Some(websocket_url.into());
        self
    }

    pub fn instrument_type(
        mut self,
        instrument_type: crate::types::HyperliquidInstrumentType,
    ) -> Self {
        self.instrument_type = Some(instrument_type);
        self
    }

    pub fn credential(mut self, credential: Option<HyperliquidCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn is_mainnet(mut self, is_mainnet: bool) -> Self {
        self.is_mainnet = Some(is_mainnet);
        self
    }

    pub fn wallet_address(mut self, wallet_address: String) -> Self {
        self.wallet_address = Some(wallet_address);
        self
    }

    pub fn build(self) -> HyperliquidClient {
        let is_mainnet = self.is_mainnet.unwrap_or(true);
        let rest_api_base_url = self.rest_api_base_url.unwrap_or_else(|| {
            if is_mainnet {
                "https://api.hyperliquid.xyz".to_string()
            } else {
                "https://api.hyperliquid-testnet.xyz".to_string()
            }
        });
        let websocket_url = self.websocket_url.unwrap_or_else(|| {
            if is_mainnet {
                "wss://api.hyperliquid.xyz/ws".to_string()
            } else {
                "wss://api.hyperliquid-testnet.xyz/ws".to_string()
            }
        });

        let derived_wallet_address = self
            .credential
            .as_ref()
            .map(|c| c.signing_key.address().to_string())
            .unwrap_or_default();

        crate::finest!("derived_wallet_address is {}", derived_wallet_address);

        if let Some(wallet_address) = &self.wallet_address
            && !wallet_address.is_empty()
            && !derived_wallet_address.is_empty()
        {
            assert_eq!(
                wallet_address, &derived_wallet_address,
                "wallet address does not match signing key address"
            );
        }

        let wallet_address: String = self
            .wallet_address
            .as_ref()
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| derived_wallet_address.clone());

        HyperliquidClient {
            rest_api_base_url,
            websocket_url,
            instrument_type: self
                .instrument_type
                .unwrap_or(crate::types::HyperliquidInstrumentType::Unknown),
            credential: self.credential,
            is_mainnet,
            wallet_address,
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for HyperliquidClient {
    fn generate_next_client_order_id(&self) -> String {
        let (ts, seq) = crate::exchange_client::common::generate_client_order_id_parts();
        let value: i64 = format!("{}{:03}", ts, seq).parse().unwrap_or(0);
        format!("0x{:0>32x}", value)
    }
}
