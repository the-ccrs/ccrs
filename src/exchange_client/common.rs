static CLIENT_ORDER_ID_GENERATOR_STATE: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

#[async_trait::async_trait]
pub trait Common {
    fn generate_next_client_order_id(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let (ts, seq) = loop {
            let current =
                CLIENT_ORDER_ID_GENERATOR_STATE.load(std::sync::atomic::Ordering::Relaxed);

            let last_ts = current >> 32;
            let last_seq = current & 0xffff_ffff;

            let (next_ts, next_seq) = if last_ts == now {
                (now, last_seq + 1)
            } else {
                (now, 0)
            };

            let next = (next_ts << 32) | next_seq;

            if CLIENT_ORDER_ID_GENERATOR_STATE
                .compare_exchange(
                    current,
                    next,
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                )
                .is_ok()
            {
                break (next_ts, next_seq);
            }
        };

        let mut client_order_id = String::new();

        self.prefix_client_order_id(&mut client_order_id);

        client_order_id.push_str(&ts.to_string());

        let padded = format!("{:0width$}", seq, width = 3);

        client_order_id.push_str(&padded);

        client_order_id
    }

    fn prefix_client_order_id(&self, _client_order_id: &mut String) {}
}

#[derive(Debug)]
pub enum Request {
    GetInstrumentInfo(GetInstrumentInfoRequest),
    GetTopOfBook(GetTopOfBookRequest),

    PlaceOrder(PlaceOrderRequest),
    CancelOrder(CancelOrderRequest),
    GetOpenOrder(GetOpenOrderRequest),
    GetPosition(GetPositionRequest),
    GetBalance(GetBalanceRequest),
    GetAccountInfo(GetAccountInfoRequest),

    SubscribeTopOfBook(SubscribeTopOfBookRequest),
    SubscribeTrade(SubscribeTradeRequest),
    SubscribeOrder(SubscribeOrderRequest),
    SubscribeFill(SubscribeFillRequest),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Response {
    None,
    GetInstrumentInfo(GetInstrumentInfoResponse),
    GetTopOfBook(GetTopOfBookResponse),
    PlaceOrder(PlaceOrderResponse),
    CancelOrder(CancelOrderResponse),
    GetOpenOrder(GetOpenOrderResponse),
    GetPosition(GetPositionResponse),
    GetBalance(GetBalanceResponse),
    GetAccountInfo(GetAccountInfoResponse),
    HttpRequestError(anyhow::Error),
    HttpErrorResponse(crate::networking::http::HttpResponse),

    Authenticate(AuthenticateResponse),
    Subscribe(SubscribeResponse),
    Heartbeat(HeartbeatResponse),
    TopOfBookSubscription(TopOfBookSubscriptionData),
    TradeSubscription(TradeSubscriptionData),
    OrderSubscription(OrderSubscriptionData),
    FillSubscription(FillSubscriptionData),
    WebSocketWriteError(anyhow::Error),
    WebSocketReadError(anyhow::Error),
    WebSocketPingMessage(bytes::Bytes),
    WebSocketPongMessage(bytes::Bytes),
    WebSocketHeartbeatMessage(crate::networking::websocket::WebSocketText),
    WebSocketCloseMessage(Option<tokio_tungstenite::tungstenite::protocol::CloseFrame>),
    WebSocketErrorResponse(crate::networking::websocket::WebSocketText),
}

#[derive(Debug, Default)]
pub struct GetInstrumentInfoRequest {
    pub symbol: String,
    pub limit: u32,
    pub next_page_cursor: String,
}

#[derive(Debug, Default)]
pub struct GetTopOfBookRequest {
    pub symbol: String,
}

#[derive(Debug, Default)]
pub struct PlaceOrderRequest {
    pub symbol: String,
    pub client_order_id: String,
    pub order_type: crate::types::OrderType,
    pub side: crate::types::OrderSide,
    pub price: String,
    pub quantity: String,
}

#[derive(Debug, Default)]
pub struct CancelOrderRequest {
    pub symbol: String,
    pub order_id: String,
    pub client_order_id: String,
}

#[derive(Debug, Default)]
pub struct GetOpenOrderRequest {
    pub symbol: String,
    pub limit: u32,
    pub next_page_cursor: String,
}

#[derive(Debug, Default)]
pub struct GetPositionRequest {
    pub symbol: String,
    pub settle_asset: String,
    pub limit: u32,
    pub next_page_cursor: String,
}

#[derive(Debug, Default)]
pub struct GetBalanceRequest {
    pub asset: String,
}

#[derive(Debug, Default)]
pub struct GetAccountInfoRequest {}

#[derive(Debug, Default)]
pub struct SubscribeTopOfBookRequest {
    pub id: Option<u64>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Default)]
pub struct SubscribeTradeRequest {
    pub id: Option<u64>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Default)]
pub struct SubscribeOrderRequest {
    pub id: Option<u64>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Default)]
pub struct SubscribeFillRequest {
    pub id: Option<u64>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Default)]
pub struct GetInstrumentInfoResponse {
    pub data: Vec<crate::types::InstrumentInfo>,
    pub next_page_cursor: String,
}

#[derive(Debug, Default)]
pub struct GetTopOfBookResponse {
    pub data: Vec<crate::types::TopOfBook>,
}

#[derive(Debug, Default)]
pub struct PlaceOrderResponse {
    pub order_id: String,
}

#[derive(Debug, Default)]
pub struct CancelOrderResponse {}

#[derive(Debug, Default)]
pub struct GetOpenOrderResponse {
    pub data: Vec<crate::types::Order>,
    pub next_page_cursor: String,
}

#[derive(Debug, Default)]
pub struct GetPositionResponse {
    pub data: Vec<crate::types::Position>,
    pub next_page_cursor: String,
}

#[derive(Debug, Default)]
pub struct GetBalanceResponse {
    pub data: Vec<crate::types::Balance>,
}

#[derive(Debug, Default)]
pub struct GetAccountInfoResponse {
    pub data: Vec<crate::types::AccountInfo>,
}

#[derive(Debug, Default)]
pub struct TopOfBookSubscriptionData {
    pub data: Vec<crate::types::TopOfBook>,
}

#[derive(Debug, Default)]
pub struct TradeSubscriptionData {
    pub data: Vec<crate::types::Trade>,
}

#[derive(Debug, Default)]
pub struct OrderSubscriptionData {
    pub data: Vec<crate::types::Order>,
}

#[derive(Debug, Default)]
pub struct FillSubscriptionData {
    pub data: Vec<crate::types::Fill>,
}

#[derive(Debug, Default)]
pub struct AuthenticateResponse {
    pub id: Option<u64>,
}

#[derive(Debug)]
pub enum SubscribeResponseKind {
    TopOfBook,
    Trade,
    Order,
    Fill,
}

#[derive(Debug, Default)]
pub struct SubscribeResponse {
    pub id: Option<u64>,
    pub symbols: Vec<String>,
    pub kind: Option<SubscribeResponseKind>,
}

#[derive(Debug, Default)]
pub struct HeartbeatResponse {
    pub id: Option<u64>,
}
