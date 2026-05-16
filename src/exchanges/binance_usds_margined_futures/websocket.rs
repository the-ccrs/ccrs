use crate::exchange_client::rest::Rest;

#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::binance_usds_margined_futures::common::BinanceUsdsMarginedFuturesClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    async fn create_websocket_client(
        &self,
        websocket_client_config: crate::types::WebSocketClientConfig,
        websocket_config: crate::networking::websocket::WebSocketConfig,
    ) -> anyhow::Result<crate::networking::websocket::WebSocketClient> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_subscribe_order_websocket_request(
        &self,
        _subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_subscribe_fill_websocket_request(
        &self,
        _subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn is_websocket_text_fill_subscription_data(
        &self,
        _: &crate::networking::websocket::WebSocketText,
    ) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_subscribe_top_of_book_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_subscribe_trade_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_subscribe_order_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        _: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn is_unexpected_websocket_text_subscription_data_benign(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn is_websocket_text_authenticate_success_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn is_websocket_text_heartbeat_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_authenticate_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_subscribe_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_heartbeat_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}

    fn create_websocket_error_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
    panic!("This feature requires purchase, please contact us on Telegram https://t.me/+NvPBKXi6kFNkYmE5");
}
}
