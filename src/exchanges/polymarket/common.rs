pub struct PolymarketCredential {
    pub signing_key: alloy::signers::local::PrivateKeySigner,
    pub api_key: String,
    pub api_secret: String,
    pub api_passphrase: String,
}

impl PolymarketCredential {
    pub fn new(
        signing_key: alloy::signers::local::PrivateKeySigner,
        api_key: impl Into<String>,
        api_secret: impl Into<String>,
        api_passphrase: impl Into<String>,
    ) -> Self {
        Self {
            signing_key,
            api_key: api_key.into(),
            api_secret: api_secret.into(),
            api_passphrase: api_passphrase.into(),
        }
    }

    pub async fn from_private_key(
        signing_key: alloy::signers::local::PrivateKeySigner,
    ) -> anyhow::Result<Self> {
        let sdk_signer = private_key_to_sdk_signer(&signing_key)?;

        let client = polymarket_client_sdk_v2::clob::Client::new(
            "https://clob.polymarket.com",
            polymarket_client_sdk_v2::clob::Config::default(),
        )?
        .authentication_builder(&sdk_signer)
        .authenticate()
        .await?;

        let creds = client.credentials();

        Ok(Self {
            signing_key,
            api_key: creds.key().to_string(),
            api_secret: polymarket_client_sdk_v2::auth::ExposeSecret::expose_secret(creds.secret())
                .to_string(),
            api_passphrase: polymarket_client_sdk_v2::auth::ExposeSecret::expose_secret(
                creds.passphrase(),
            )
            .to_string(),
        })
    }
}

impl std::fmt::Debug for PolymarketCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = hex::encode(self.signing_key.to_bytes());
        let key = format!("0x{key}");
        f.debug_struct("PolymarketCredential")
            .field("signing_key", &crate::utils::mask_hex_key(&key))
            .field("api_key", &self.api_key)
            .finish()
    }
}

pub struct PolymarketClient {
    pub(super) clob_api_base_url: String,
    pub(super) data_api_base_url: String,
    pub(super) gamma_api_base_url: String,
    pub(super) market_websocket_url: String,
    pub(super) user_websocket_url: String,
    pub(super) credential: Option<PolymarketCredential>,
    pub(super) sdk_client: Option<
        polymarket_client_sdk_v2::clob::Client<
            polymarket_client_sdk_v2::auth::state::Authenticated<
                polymarket_client_sdk_v2::auth::Normal,
            >,
        >,
    >,
    pub(super) is_mainnet: bool,
    pub(super) funder_address: String,
    pub(super) signer_address: String,
    pub(super) signature_type: i64,
    pub(super) api_builder_code: polymarket_client_sdk_v2::types::B256,
}

impl std::fmt::Debug for PolymarketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolymarketClient")
            .field("clob_api_base_url", &self.clob_api_base_url)
            .field("data_api_base_url", &self.data_api_base_url)
            .field("credential", &self.credential)
            .field("is_mainnet", &self.is_mainnet)
            .field("funder_address", &self.funder_address)
            .field("signer_address", &self.signer_address)
            .field("signature_type", &self.signature_type)
            .finish()
    }
}

impl PolymarketClient {
    pub fn builder() -> PolymarketClientBuilder {
        PolymarketClientBuilder::default()
    }

    pub async fn initialize_sdk_client(&mut self) -> anyhow::Result<()> {
        let credential = self
            .credential
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing Polymarket credential"))?;
        let sdk_signer = private_key_to_sdk_signer(&credential.signing_key)?;

        let config = polymarket_client_sdk_v2::clob::Config::builder()
            .builder_code(self.api_builder_code)
            .build();
        let client = polymarket_client_sdk_v2::clob::Client::new(&self.clob_api_base_url, config)?
            .authentication_builder(&sdk_signer)
            .funder(self.funder_address.parse()?)
            .signature_type(match self.signature_type {
                0 => polymarket_client_sdk_v2::clob::types::SignatureType::Eoa,
                1 => polymarket_client_sdk_v2::clob::types::SignatureType::Proxy,
                2 => polymarket_client_sdk_v2::clob::types::SignatureType::GnosisSafe,
                3 => polymarket_client_sdk_v2::clob::types::SignatureType::Poly1271,
                other => anyhow::bail!("invalid signature_type: {other}"),
            })
            .authenticate()
            .await?;
        self.sdk_client = Some(client);
        Ok(())
    }

    pub(super) fn convert_side_str_to_order_side(&self, s: &str) -> crate::types::OrderSide {
        match s {
            "BUY" => crate::types::OrderSide::Buy,
            "SELL" => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        }
    }

    pub(super) fn convert_side_str_to_taker_side(&self, s: &str) -> crate::types::TakerSide {
        match s {
            "BUY" => crate::types::TakerSide::Buy,
            "SELL" => crate::types::TakerSide::Sell,
            _ => crate::types::TakerSide::Unknown,
        }
    }

    pub(super) fn convert_status_str_to_order_status(&self, s: &str) -> crate::types::OrderStatus {
        match s {
            "LIVE" => crate::types::OrderStatus::Open,
            "MATCHED" => crate::types::OrderStatus::Filled,
            "PARTIALLY_MATCHED" => crate::types::OrderStatus::PartiallyFilled,
            "CANCELED" => crate::types::OrderStatus::Canceled,
            "EXPIRED" => crate::types::OrderStatus::Expired,
            _ => crate::types::OrderStatus::Unknown,
        }
    }

    pub(super) fn convert_fill_status_str_to_fill_status(
        &self,
        status: &str,
    ) -> crate::types::FillStatus {
        match status {
            "MATCHED" => crate::types::FillStatus::Matched,
            "MINED" => crate::types::FillStatus::Mined,
            "CONFIRMED" => crate::types::FillStatus::Confirmed,
            "RETRYING" => crate::types::FillStatus::Retrying,
            "FAILED" => crate::types::FillStatus::Failed,
            other => panic!("unknown fill status: {other}"),
        }
    }

    pub(super) fn compute_l2_hmac_signature(
        &self,
        timestamp: &str,
        method: &str,
        request_path: &str,
        body: &str,
        credential: &PolymarketCredential,
    ) -> String {
        let secret_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE,
            credential.api_secret.as_bytes(),
        )
        .expect("invalid API secret");

        let mut message = format!("{}{}{}", timestamp, method, request_path);

        if !body.is_empty() {
            message.push_str(&body.replace('\'', "\""));
        }

        let mut mac =
            <hmac::Hmac<sha2::Sha256> as hmac::digest::KeyInit>::new_from_slice(&secret_bytes)
                .expect("HMAC init failed");

        hmac::Mac::update(&mut mac, message.as_bytes());
        let sig_bytes = hmac::Mac::finalize(mac).into_bytes();

        base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE, sig_bytes)
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        let order_id = json_value["id"].as_str().unwrap_or("").to_string();
        let asset_id = json_value["asset_id"].as_str().unwrap_or("").to_string();
        let side_str = json_value["side"].as_str().unwrap_or("");
        let price = json_value["price"].as_str().unwrap_or("").to_string();
        let original_size = json_value["original_size"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let size_matched = json_value["size_matched"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let status_str = json_value["status"].as_str().unwrap_or("");
        let order_type_str = json_value["order_type"].as_str().unwrap_or("");

        let size_matched_f: f64 = size_matched.parse().unwrap_or(0.0);
        let original_f: f64 = original_size.parse().unwrap_or(0.0);
        let remaining = if original_f >= size_matched_f {
            format!("{}", original_f - size_matched_f)
        } else {
            "0".to_string()
        };

        let order_type = match order_type_str {
            "GTC" | "GTD" => crate::types::OrderType::Limit,
            "FOK" | "FAK" => crate::types::OrderType::Market,
            _ => crate::types::OrderType::Unknown,
        };

        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Polymarket,
            symbol: asset_id,
            order_id,
            client_order_id: String::new(),
            order_type,
            side: self.convert_side_str_to_order_side(side_str),
            price,
            quantity: original_size,
            remaining_quantity: remaining,
            cumulative_filled_quantity: size_matched,
            status: self.convert_status_str_to_order_status(status_str),
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        let asset = json_value["asset"].as_str().unwrap_or("").to_string();
        let size = json_value["size"].as_f64().unwrap_or(0.0);
        let avg_price = json_value["avgPrice"].as_f64().unwrap_or(0.0);

        let side = if size > 0.0 {
            crate::types::PositionSide::Long
        } else {
            crate::types::PositionSide::Unknown
        };

        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Polymarket,
            symbol: asset,
            side,
            entry_price: format!("{}", avg_price),
            quantity: format!("{}", size),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct PolymarketClientBuilder {
    clob_api_base_url: Option<String>,
    data_api_base_url: Option<String>,
    gamma_api_base_url: Option<String>,
    market_websocket_url: Option<String>,
    user_websocket_url: Option<String>,
    credential: Option<PolymarketCredential>,
    is_mainnet: Option<bool>,
    funder_address: Option<String>,
    signature_type: Option<i64>,
}

impl PolymarketClientBuilder {
    pub fn clob_api_base_url(mut self, url: impl Into<String>) -> Self {
        self.clob_api_base_url = Some(url.into());
        self
    }

    pub fn data_api_base_url(mut self, url: impl Into<String>) -> Self {
        self.data_api_base_url = Some(url.into());
        self
    }

    pub fn gamma_api_base_url(mut self, url: impl Into<String>) -> Self {
        self.gamma_api_base_url = Some(url.into());
        self
    }

    pub fn market_websocket_url(mut self, url: impl Into<String>) -> Self {
        self.market_websocket_url = Some(url.into());
        self
    }

    pub fn user_websocket_url(mut self, url: impl Into<String>) -> Self {
        self.user_websocket_url = Some(url.into());
        self
    }

    pub fn credential(mut self, credential: Option<PolymarketCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn is_mainnet(mut self, is_mainnet: bool) -> Self {
        self.is_mainnet = Some(is_mainnet);
        self
    }

    pub fn funder_address(mut self, funder_address: impl Into<String>) -> Self {
        self.funder_address = Some(funder_address.into());
        self
    }

    pub fn signature_type(mut self, signature_type: i64) -> Self {
        self.signature_type = Some(signature_type);
        self
    }

    pub fn build(self) -> PolymarketClient {
        let is_mainnet = self.is_mainnet.unwrap_or(true);

        let clob_api_base_url = self
            .clob_api_base_url
            .unwrap_or_else(|| "https://clob.polymarket.com".to_string());

        let data_api_base_url = self
            .data_api_base_url
            .unwrap_or_else(|| "https://data-api.polymarket.com".to_string());

        let gamma_api_base_url = self
            .gamma_api_base_url
            .unwrap_or_else(|| "https://gamma-api.polymarket.com".to_string());

        let market_websocket_url = self
            .market_websocket_url
            .unwrap_or_else(|| "wss://ws-subscriptions-clob.polymarket.com/ws/market".to_string());

        let user_websocket_url = self
            .user_websocket_url
            .unwrap_or_else(|| "wss://ws-subscriptions-clob.polymarket.com/ws/user".to_string());

        let derived_signer_address = self.credential.as_ref().map(|c| c.signing_key.address());

        let derived_signer_address_str = derived_signer_address
            .map(|a| a.to_string().to_lowercase())
            .unwrap_or_default();

        let signature_type = self.signature_type.unwrap_or(0);

        let api_builder_code =
            <polymarket_client_sdk_v2::types::B256 as std::str::FromStr>::from_str(
                "0xed570097c3c2f6991fde214960edf1866f95b9422e26ef9c40bcee3fc019bc8a",
            )
            .unwrap();

        PolymarketClient {
            clob_api_base_url,
            data_api_base_url,
            gamma_api_base_url,
            market_websocket_url,
            user_websocket_url,
            credential: self.credential,
            sdk_client: None,
            is_mainnet,
            funder_address: self.funder_address.unwrap_or_default(),
            signer_address: derived_signer_address_str,
            signature_type,
            api_builder_code,
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for PolymarketClient {
    fn generate_next_client_order_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

pub(super) fn private_key_to_sdk_signer(
    signing_key: &alloy::signers::local::PrivateKeySigner,
) -> anyhow::Result<
    polymarket_client_sdk_v2::auth::LocalSigner<alloy::signers::k256::ecdsa::SigningKey>,
> {
    let pk_hex = format!("0x{}", hex::encode(signing_key.credential().to_bytes()));

    let signer = <polymarket_client_sdk_v2::auth::LocalSigner<
        alloy::signers::k256::ecdsa::SigningKey,
    > as std::str::FromStr>::from_str(&pk_hex)?;

    Ok(<polymarket_client_sdk_v2::auth::LocalSigner<
        alloy::signers::k256::ecdsa::SigningKey,
    > as polymarket_client_sdk_v2::auth::Signer>::with_chain_id(
        signer,
        Some(polymarket_client_sdk_v2::POLYGON),
    ))
}
