#[derive(Clone)]
pub struct KalshiPerpsCredential {
    pub api_key: String,
    pub private_key: openssl::pkey::PKey<openssl::pkey::Private>,
}

impl KalshiPerpsCredential {
    pub fn new(api_key: impl Into<String>, private_key_path: impl AsRef<std::path::Path>) -> Self {
        let key_bytes =
            std::fs::read(private_key_path.as_ref()).expect("failed to read private key file");
        let private_key = openssl::pkey::PKey::private_key_from_pem(&key_bytes)
            .expect("failed to parse private key as PEM");

        Self {
            api_key: api_key.into(),
            private_key,
        }
    }
}

impl std::fmt::Debug for KalshiPerpsCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KalshiPerpsCredential")
            .field("api_key", &self.api_key)
            .field("private_key", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct KalshiPerpsClient {
    pub(super) rest_api_base_url: String,
    pub(super) websocket_api_url: String,
    pub(super) credential: Option<KalshiPerpsCredential>,
}

impl KalshiPerpsClient {
    pub fn builder() -> KalshiPerpsClientBuilder {
        KalshiPerpsClientBuilder::default()
    }

    pub(super) fn build_signature(&self, timestamp: &str, method: &str, path: &str) -> String {
        let credential = self.credential.as_ref().unwrap();
        let message = format!("{}{}{}", timestamp, method, path);

        let mut signer = openssl::sign::Signer::new(
            openssl::hash::MessageDigest::sha256(),
            &credential.private_key,
        )
        .unwrap();
        signer
            .set_rsa_padding(openssl::rsa::Padding::PKCS1_PSS)
            .unwrap();
        signer
            .set_rsa_pss_saltlen(openssl::sign::RsaPssSaltlen::DIGEST_LENGTH)
            .unwrap();
        signer.update(message.as_bytes()).unwrap();
        let signature = signer.sign_to_vec().unwrap();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &signature)
    }

    pub(super) fn convert_order_side_to_string(
        &self,
        side: crate::types::OrderSide,
    ) -> &'static str {
        match side {
            crate::types::OrderSide::Buy => "bid",
            crate::types::OrderSide::Sell => "ask",
            crate::types::OrderSide::Unknown => panic!("Invalid side"),
        }
    }

    pub(super) fn convert_string_to_order_side(&self, side: &str) -> crate::types::OrderSide {
        match side {
            "bid" => crate::types::OrderSide::Buy,
            "ask" => crate::types::OrderSide::Sell,
            _ => crate::types::OrderSide::Unknown,
        }
    }

    pub(super) fn convert_json_value_to_order(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Order {
        crate::types::Order {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KalshiPerps,
            symbol: json_value["ticker"].as_str().unwrap_or("").to_string(),
            order_id: json_value["order_id"].as_str().unwrap_or("").to_string(),
            client_order_id: json_value["client_order_id"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            order_type: crate::types::OrderType::Limit,
            side: self.convert_string_to_order_side(json_value["side"].as_str().unwrap_or("")),
            price: json_value["price"].as_str().unwrap_or("").to_string(),
            quantity: String::new(),
            cumulative_filled_quantity: json_value["fill_count"].as_str().unwrap_or("").to_string(),
            remaining_quantity: json_value["remaining_count"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            status: crate::types::OrderStatus::Open,
            ..Default::default()
        }
    }

    pub(super) fn convert_json_value_to_position(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Position {
        let position = json_value["position"].as_str().unwrap_or("0");
        let side = if position.starts_with('-') {
            crate::types::PositionSide::Short
        } else {
            crate::types::PositionSide::Long
        };

        crate::types::Position {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::KalshiPerps,
            symbol: json_value["market_ticker"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            side,
            entry_price: json_value["entry_price"].as_str().unwrap_or("").to_string(),
            quantity: position.strip_prefix('-').unwrap_or(position).to_string(),
            leverage: String::new(),
            position_asset: String::new(),
        }
    }

    pub(super) fn convert_json_value_to_balance(
        &self,
        json_value: &serde_json::Value,
    ) -> crate::types::Balance {
        crate::types::Balance {
            exchange: crate::types::Exchange::KalshiPerps,
            asset: "USD".to_string(),
            quantity: json_value["settled_funds"]
                .as_str()
                .unwrap_or("0")
                .to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct KalshiPerpsClientBuilder {
    rest_api_base_url: Option<String>,
    websocket_api_url: Option<String>,
    credential: Option<KalshiPerpsCredential>,
    use_demo_trading: Option<bool>,
}

impl KalshiPerpsClientBuilder {
    pub fn rest_api_base_url(mut self, rest_api_base_url: impl Into<String>) -> Self {
        self.rest_api_base_url = Some(rest_api_base_url.into());
        self
    }

    pub fn websocket_api_url(mut self, websocket_api_url: impl Into<String>) -> Self {
        self.websocket_api_url = Some(websocket_api_url.into());
        self
    }

    pub fn credential(mut self, credential: Option<KalshiPerpsCredential>) -> Self {
        self.credential = credential;
        self
    }

    pub fn use_demo_trading(mut self, use_demo_trading: Option<bool>) -> Self {
        self.use_demo_trading = use_demo_trading;
        self
    }

    pub fn build(self) -> KalshiPerpsClient {
        let use_demo_trading = self.use_demo_trading.unwrap_or(false);
        let rest_api_base_url = self.rest_api_base_url.unwrap_or_else(|| {
            if use_demo_trading {
                "https://external-api.demo.kalshi.co".to_string()
            } else {
                "https://external-api.kalshi.com".to_string()
            }
        });
        let websocket_api_url = self.websocket_api_url.unwrap_or_else(|| {
            if use_demo_trading {
                "wss://external-api-ws.demo.kalshi.co/trade-api/ws/v2".to_string()
            } else {
                "wss://external-api-ws.kalshi.com/trade-api/ws/v2".to_string()
            }
        });

        KalshiPerpsClient {
            rest_api_base_url,
            websocket_api_url,
            credential: self.credential,
        }
    }
}

#[async_trait::async_trait]
impl crate::exchange_client::common::Common for KalshiPerpsClient {}
