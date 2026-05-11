#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PositionSide {
    #[default]
    Unknown,
    Long,
    Short,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    #[default]
    Unknown,
    Buy,
    Sell,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TakerSide {
    #[default]
    Unknown,
    Buy,
    Sell,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    #[default]
    Unknown,
    Market,
    Limit,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    #[default]
    Unknown,
    Placed,
    Open,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Exchange {
    #[default]
    Unknown,
    Bybit,
    Coinbase,
    Okx,
    BinanceSpot,
    BinanceUsdsMarginedFutures,
    GateioSpotAndMargin,
    GateioPerpetualFutures,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GateioSpotAndMarginInstrumentType {
    #[default]
    Unknown,
    Spot,
    Margin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BybitInstrumentType {
    #[default]
    Unknown,
    Spot,
    Linear,
    Inverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OkxInstrumentType {
    #[default]
    Unknown,
    Spot,
    Margin,
    Swap,
    Futures,
    Option,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExchangeInstrumentType {
    #[default]
    Unknown,
    Bybit(BybitInstrumentType),
    Coinbase,
    Okx(OkxInstrumentType),
    BinanceSpot,
    BinanceUsdsMarginedFutures,
    GateioSpotAndMargin(GateioSpotAndMarginInstrumentType),
    GateioPerpetualFutures,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BybitWebSocketEndpoint {
    #[default]
    Unknown,
    MarketData,
    AccountData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OkxWebSocketEndpoint {
    #[default]
    Unknown,
    MarketData,
    AccountData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoinbaseWebSocketEndpoint {
    #[default]
    Unknown,
    MarketData,
    AccountData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BinanceSpotWebSocketEndpoint {
    #[default]
    Unknown,
    MarketData,
    AccountData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BinanceUsdsMarginedFuturesWebSocketEndpoint {
    #[default]
    Unknown,
    MarketData,
    AccountData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GateioSpotAndMarginWebSocketEndpoint {
    #[default]
    Unknown,
    MarketData,
    AccountData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GateioPerpetualFuturesWebSocketEndpoint {
    #[default]
    Unknown,
    MarketData,
    AccountData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WebSocketEndpoint {
    #[default]
    Unknown,
    Bybit(BybitWebSocketEndpoint),
    Okx(OkxWebSocketEndpoint),
    Coinbase(CoinbaseWebSocketEndpoint),
    BinanceSpot(BinanceSpotWebSocketEndpoint),
    BinanceUsdsMarginedFutures(BinanceUsdsMarginedFuturesWebSocketEndpoint),
    GateioSpotAndMargin(GateioSpotAndMarginWebSocketEndpoint),
    GateioPerpetualFutures(GateioPerpetualFuturesWebSocketEndpoint),
}

#[derive(Debug, Clone, Default)]
pub struct WebSocketClientConfig {
    pub endpoint: WebSocketEndpoint,
}

impl WebSocketClientConfig {
    pub fn new(endpoint: WebSocketEndpoint) -> Self {
        Self { endpoint }
    }

    pub fn bybit_market_data() -> Self {
        Self::new(WebSocketEndpoint::Bybit(BybitWebSocketEndpoint::MarketData))
    }

    pub fn bybit_account_data() -> Self {
        Self::new(WebSocketEndpoint::Bybit(
            BybitWebSocketEndpoint::AccountData,
        ))
    }

    pub fn okx_market_data() -> Self {
        Self::new(WebSocketEndpoint::Okx(OkxWebSocketEndpoint::MarketData))
    }

    pub fn okx_account_data() -> Self {
        Self::new(WebSocketEndpoint::Okx(OkxWebSocketEndpoint::AccountData))
    }

    pub fn coinbase_market_data() -> Self {
        Self::new(WebSocketEndpoint::Coinbase(
            CoinbaseWebSocketEndpoint::MarketData,
        ))
    }

    pub fn coinbase_account_data() -> Self {
        Self::new(WebSocketEndpoint::Coinbase(
            CoinbaseWebSocketEndpoint::AccountData,
        ))
    }

    pub fn binance_spot_market_data() -> Self {
        Self::new(WebSocketEndpoint::BinanceSpot(
            BinanceSpotWebSocketEndpoint::MarketData,
        ))
    }

    pub fn binance_spot_account_data() -> Self {
        Self::new(WebSocketEndpoint::BinanceSpot(
            BinanceSpotWebSocketEndpoint::AccountData,
        ))
    }

    pub fn binance_usds_margined_futures_market_data() -> Self {
        Self::new(WebSocketEndpoint::BinanceUsdsMarginedFutures(
            BinanceUsdsMarginedFuturesWebSocketEndpoint::MarketData,
        ))
    }

    pub fn binance_usds_margined_futures_account_data() -> Self {
        Self::new(WebSocketEndpoint::BinanceUsdsMarginedFutures(
            BinanceUsdsMarginedFuturesWebSocketEndpoint::AccountData,
        ))
    }

    pub fn gateio_spot_and_margin_market_data() -> Self {
        Self::new(WebSocketEndpoint::GateioSpotAndMargin(
            GateioSpotAndMarginWebSocketEndpoint::MarketData,
        ))
    }

    pub fn gateio_spot_and_margin_account_data() -> Self {
        Self::new(WebSocketEndpoint::GateioSpotAndMargin(
            GateioSpotAndMarginWebSocketEndpoint::AccountData,
        ))
    }

    pub fn gateio_perpetual_futures_market_data() -> Self {
        Self::new(WebSocketEndpoint::GateioPerpetualFutures(
            GateioPerpetualFuturesWebSocketEndpoint::MarketData,
        ))
    }

    pub fn gateio_perpetual_futures_account_data() -> Self {
        Self::new(WebSocketEndpoint::GateioPerpetualFutures(
            GateioPerpetualFuturesWebSocketEndpoint::AccountData,
        ))
    }
}

#[derive(Debug, Default)]
pub struct InstrumentInfo {
    pub exchange_instrument_type: ExchangeInstrumentType,
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub order_price_increment: String,
    pub order_quantity_increment: String,
    pub order_quantity_min: String,
    pub order_quantity_max: String,
    pub order_quote_quantity_min: String,
    pub order_quote_quantity_max: String,
    pub settle_asset: String,
    pub underlying_symbol: String,
    pub contract_size: String,
    pub contract_multiplier: String,
    pub expiry_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default)]
pub struct TopOfBook {
    pub exchange_instrument_type: ExchangeInstrumentType,
    pub symbol: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub bid_price: String,
    pub bid_size: String,
    pub ask_price: String,
    pub ask_size: String,
}

#[derive(Debug, Default)]
pub struct Trade {
    pub exchange_instrument_type: ExchangeInstrumentType,
    pub symbol: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub price: String,
    pub size: String,
    pub side: crate::types::TakerSide,
}

#[derive(Debug, Default)]
pub struct Order {
    pub exchange_instrument_type: ExchangeInstrumentType,
    pub symbol: String,
    pub order_id: String,
    pub client_order_id: String,
    pub order_type: crate::types::OrderType,
    pub side: crate::types::OrderSide,
    pub price: String,
    pub quantity: String,
    pub leverage: String,
    pub remaining_quantity: String,
    pub cumulative_filled_quantity: String,
    pub cumulative_filled_quote_quantity: String,
    pub average_filled_price: String,
    pub fill_price: String,
    pub fill_quantity: String,
    pub fill_quote_quantity: String,
    pub fill_is_maker: bool,
    pub status: crate::types::OrderStatus,
}

#[derive(Debug, Default)]
pub struct Fill {
    pub exchange_instrument_type: ExchangeInstrumentType,
    pub order_id: String,
    pub client_order_id: String,
    pub side: crate::types::OrderSide,
    pub price: String,
    pub quantity: String,
    pub quote_quantity: String,
    pub is_maker: bool,
}

#[derive(Debug, Default)]
pub struct Position {
    pub exchange_instrument_type: ExchangeInstrumentType,
    pub symbol: String,
    pub side: crate::types::PositionSide,
    pub entry_price: String,
    pub quantity: String,
    pub leverage: String,
    pub position_asset: String,
}

#[derive(Debug, Default)]
pub struct Balance {
    pub exchange: Exchange,
    pub asset: String,
    pub quantity: String,
}
