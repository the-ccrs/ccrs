#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub connect_timeout_secs: u64,
    pub close_timeout_secs: u64,
    pub heartbeat_interval_secs: u64,
    pub proxy_url: Option<String>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}
}

#[derive(Debug, Clone)]
pub struct WebSocketSender {
    url: String,
    send_channel_tx: tokio::sync::mpsc::Sender<String>,
    ping_channel_tx: tokio::sync::mpsc::Sender<bytes::Bytes>,
}

impl WebSocketSender {
    pub fn url(&self) -> &str {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    pub async fn send(&self, message: impl Into<String>) -> anyhow::Result<()> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    pub async fn ping(&self, payload: impl Into<bytes::Bytes>) -> anyhow::Result<()> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}
}

#[derive(Debug)]
pub struct WebSocketClient {
    url: String,
    #[allow(dead_code)]
    websocket_config: WebSocketConfig,
    websocket_sender: WebSocketSender,
    sender_err_rx: tokio::sync::oneshot::Receiver<tokio_tungstenite::tungstenite::Error>,
    reader: futures_util::stream::Fuse<
        futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    >,
    closed: bool,
    normal_cancellation_token: tokio_util::sync::CancellationToken,
    cancellation_token: tokio_util::sync::CancellationToken,
}

impl Drop for WebSocketClient {
    fn drop(&mut self) {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}
}

impl WebSocketClient {
    pub fn url(&self) -> &str {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    pub fn is_closed(&self) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    pub fn cancellation_token(&self) -> &tokio_util::sync::CancellationToken {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    pub async fn close(&mut self) {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    pub fn set_closed(&mut self) {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    pub fn sender(&self) -> WebSocketSender {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    pub async fn read_next(&mut self) -> anyhow::Result<tokio_tungstenite::tungstenite::Message> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    pub fn builder(
        url: impl Into<String>,
        websocket_config: WebSocketConfig,
    ) -> WebSocketClientBuilder {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}
}

#[derive(Debug, Default, Clone)]
pub struct WebSocketClientBuilder {
    url: String,
    websocket_config: WebSocketConfig,
}

impl WebSocketClientBuilder {
    fn parse_host_port(&self) -> anyhow::Result<(String, u16), anyhow::Error> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    async fn connect_tcp(&self, host: &str, port: u16) -> anyhow::Result<tokio::net::TcpStream> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    pub async fn build(self) -> anyhow::Result<WebSocketClient> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}
}

#[derive(Debug, Default, Clone)]
pub struct WebSocketText {
    pub url: String,
    pub text: String,
    pub json_payload: Option<serde_json::Value>,
    pub payload_summary: std::collections::HashMap<String, String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

impl WebSocketText {
    pub fn from_text(
        url: impl Into<String>,
        text_bytes: tokio_tungstenite::tungstenite::Utf8Bytes,
    ) -> Self {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}
}
