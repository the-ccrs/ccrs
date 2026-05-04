#[async_trait::async_trait]
pub trait Websocket {
    async fn create_websocket_client(
        &self,
        websocket_client_config: crate::types::WebSocketClientConfig,
        websocket_config: crate::networking::websocket::WebSocketConfig,
    ) -> anyhow::Result<crate::networking::websocket::WebSocketClient> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String;

    async fn authenticate_websocket_connection(
        &self,
        client: &mut crate::networking::websocket::WebSocketClient,
    ) -> anyhow::Result<()> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    async fn keep_websocket_connection_alive(
        &self,
        heartbeat_interval_secs: u64,
        websocket_sender: crate::networking::websocket::WebSocketSender,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<()> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_authenticate_websocket_request(&self) -> String {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String;

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String;

    fn create_subscribe_order_websocket_request(
        &self,
        subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String;

    fn create_subscribe_fill_websocket_request(
        &self,
        subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String;

    async fn send_websocket_request(
        &self,
        websocket_sender: &crate::networking::websocket::WebSocketSender,
        request: crate::exchange_client::common::Request,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn convert_binary_websocket_message_to_text(
        &self,
        _bytes: bytes::Bytes,
    ) -> tungstenite::Utf8Bytes {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    async fn read_next_websocket_message(
        &self,
        websocket_client: &mut crate::networking::websocket::WebSocketClient,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn handle_websocket_text(
        &self,
        websocket_client: &crate::networking::websocket::WebSocketClient,
        text_bytes: tungstenite::Utf8Bytes,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    );

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool;

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool;

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool;

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool;

    fn is_websocket_text_fill_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool;

    fn is_unexpected_websocket_text_subscription_data_benign(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool;

    fn create_subscribe_top_of_book_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response;

    fn create_subscribe_trade_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response;

    fn create_subscribe_order_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response;

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response;

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool;

    fn is_websocket_text_authenticate_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool;

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool;

    fn is_websocket_text_heartbeat_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool;

    fn create_subscribe_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response;

    fn create_authenticate_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response;

    fn create_heartbeat_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response;

    fn create_websocket_error_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response;
}
