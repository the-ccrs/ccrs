#[async_trait::async_trait]
pub trait Websocket {
    async fn create_websocket_client(
        &self,
        websocket_client_config: crate::types::WebSocketClientConfig,
        websocket_config: crate::networking::websocket::WebSocketConfig,
    ) -> anyhow::Result<crate::networking::websocket::WebSocketClient> {
        let mut websocket_client = crate::networking::websocket::WebSocketClient::builder(
            self.websocket_api_url(websocket_client_config.endpoint),
            websocket_config.clone(),
            None,
        )
        .build()
        .await?;

        self.authenticate_websocket_connection(&mut websocket_client)
            .await?;

        self.keep_websocket_client_alive(
            websocket_config.heartbeat_interval_secs,
            websocket_client.sender().clone(),
            websocket_client.cancellation_token().clone(),
        )
        .await?;

        crate::finer!("Created websocket_client: {:#?}", websocket_client);

        Ok(websocket_client)
    }

    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String;

    async fn authenticate_websocket_connection(
        &self,
        client: &mut crate::networking::websocket::WebSocketClient,
    ) -> anyhow::Result<()> {
        let authenticate_websocket_request = self.create_authenticate_websocket_request();

        crate::fine!(
            "authenticate_websocket_request: {}",
            serde_json::from_str::<serde_json::Value>(&authenticate_websocket_request)
                .map(|v| serde_json::to_string_pretty(&v).unwrap_or_default())
                .unwrap_or_else(|_| authenticate_websocket_request.clone())
        );

        if authenticate_websocket_request.is_empty() {
            return Ok(());
        }

        client.sender().send(authenticate_websocket_request).await?;

        loop {
            let response = self.read_next_websocket_message(client).await;

            match response {
                crate::exchange_client::common::Response::WebSocketWriteError(_)
                | crate::exchange_client::common::Response::WebSocketReadError(_)
                | crate::exchange_client::common::Response::WebSocketCloseMessage(_) => {
                    return Err(anyhow::anyhow!("websocket closed during authentication"));
                }

                crate::exchange_client::common::Response::Authenticate(authenticate_response) => {
                    crate::fine!("Auth success: {:#?}", authenticate_response);
                    return Ok(());
                }

                crate::exchange_client::common::Response::WebSocketErrorResponse(
                    websocket_text,
                ) => {
                    crate::fine!("Websocket authentication error: {:#?}", websocket_text);
                    return Err(anyhow::anyhow!("websocket authentication error"));
                }

                _ => {
                    continue;
                }
            }
        }
    }

    async fn keep_websocket_client_alive(
        &self,
        heartbeat_interval_secs: u64,
        websocket_sender: crate::networking::websocket::WebSocketSender,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> anyhow::Result<()> {
        crate::finest!(
            "Starting heartbeat task with interval {} seconds\n",
            heartbeat_interval_secs
        );

        let heartbeat_factory = self.create_heartbeat_websocket_request_factory();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(heartbeat_interval_secs));

            interval.tick().await;

            loop {
                tokio::select! {
                    _ = interval.tick() => {

                        let payload = heartbeat_factory();

                        if payload.is_empty() {
                            if let Err(err) = websocket_sender.ping(vec![]).await {
                                crate::error!("Heartbeat failed: {err}");
                                break;
                            }
                        }
                        else if let Err(err) = websocket_sender.send(payload).await {
                            crate::error!("Heartbeat failed: {err}");
                            break;
                        }

                        crate::fine!("Heartbeat sent successfully");
                    }
                    _ = cancellation_token.cancelled() => {
                        crate::finer!("Heartbeat task received cancellation signal");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    fn create_authenticate_websocket_request(&self) -> String {
        "".to_string()
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(|| "".to_string())
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
        let websocket_request: String = match &request {
            crate::exchange_client::common::Request::SubscribeTopOfBook(
                subscribe_top_of_book_request,
            ) => self.create_subscribe_top_of_book_websocket_request(subscribe_top_of_book_request),
            crate::exchange_client::common::Request::SubscribeTrade(subscribe_trade_request) => {
                self.create_subscribe_trade_websocket_request(subscribe_trade_request)
            }
            crate::exchange_client::common::Request::SubscribeOrder(subscribe_order_request) => {
                self.create_subscribe_order_websocket_request(subscribe_order_request)
            }
            crate::exchange_client::common::Request::SubscribeFill(subscribe_fill_request) => {
                self.create_subscribe_fill_websocket_request(subscribe_fill_request)
            }
            _ => panic!(),
        };

        crate::fine!("=== WebSocket REQUEST ===");
        crate::fine!("{} {}", websocket_sender.url(), websocket_request);

        if let Err(err) = websocket_sender.send(websocket_request).await {
            return crate::exchange_client::common::Response::WebSocketWriteError(err);
        }

        crate::exchange_client::common::Response::None
    }

    fn convert_binary_websocket_message_to_text(
        &self,
        _bytes: bytes::Bytes,
    ) -> tungstenite::Utf8Bytes {
        panic!()
    }

    async fn handle_websocket_message(
        &self,
        websocket_client: &mut crate::networking::websocket::WebSocketClient,
        message: tokio_tungstenite::tungstenite::Message,
    ) -> Result<crate::exchange_client::common::Response, crate::exchange_client::common::Response>
    {
        match message {
            tokio_tungstenite::tungstenite::Message::Text(text_bytes) => {
                crate::fine!("Text received: {}", text_bytes);
                Ok(self.handle_websocket_text(websocket_client, text_bytes))
            }
            tokio_tungstenite::tungstenite::Message::Binary(bytes) => {
                let text_bytes = self.convert_binary_websocket_message_to_text(bytes);
                crate::fine!("Binary converted: {}", text_bytes);
                Ok(self.handle_websocket_text(websocket_client, text_bytes))
            }
            tokio_tungstenite::tungstenite::Message::Pong(payload) => {
                Ok(crate::exchange_client::common::Response::WebSocketPongMessage(payload))
            }
            tokio_tungstenite::tungstenite::Message::Ping(payload) => {
                if let Err(err) = websocket_client
                    .sender()
                    .clone()
                    .ping(payload.clone())
                    .await
                {
                    return Err(crate::exchange_client::common::Response::WebSocketWriteError(err));
                }
                Ok(crate::exchange_client::common::Response::WebSocketPingMessage(payload))
            }
            tokio_tungstenite::tungstenite::Message::Close(close_frame) => {
                websocket_client.set_closed();
                Ok(crate::exchange_client::common::Response::WebSocketCloseMessage(close_frame))
            }
            _ => panic!(),
        }
    }

    async fn read_next_websocket_message(
        &self,
        websocket_client: &mut crate::networking::websocket::WebSocketClient,
    ) -> crate::exchange_client::common::Response {
        if websocket_client.is_closed() {
            return crate::exchange_client::common::Response::WebSocketReadError(anyhow::anyhow!(
                "websocket closed"
            ));
        }

        let message = match websocket_client.read_next().await {
            Ok(message) => message,
            Err(err) => return crate::exchange_client::common::Response::WebSocketReadError(err),
        };

        match self
            .handle_websocket_message(websocket_client, message)
            .await
        {
            Ok(response) | Err(response) => response,
        }
    }

    async fn read_next_websocket_message_batch(
        &self,
        websocket_client: &mut crate::networking::websocket::WebSocketClient,
    ) -> Vec<crate::exchange_client::common::Response> {
        if websocket_client.is_closed() {
            return vec![
                crate::exchange_client::common::Response::WebSocketReadError(anyhow::anyhow!(
                    "websocket closed"
                )),
            ];
        }

        let mut responses = Vec::new();

        match websocket_client.read_next().await {
            Err(err) => {
                responses.push(crate::exchange_client::common::Response::WebSocketReadError(err));
                return responses;
            }
            Ok(message) => {
                match self
                    .handle_websocket_message(websocket_client, message)
                    .await
                {
                    Ok(response) => responses.push(response),
                    Err(err_response) => {
                        responses.push(err_response);
                        return responses;
                    }
                }
            }
        }

        loop {
            match futures::FutureExt::now_or_never(websocket_client.read_next()) {
                None => break,
                Some(Err(err)) => {
                    responses
                        .push(crate::exchange_client::common::Response::WebSocketReadError(err));
                    break;
                }
                Some(Ok(message)) => {
                    match self
                        .handle_websocket_message(websocket_client, message)
                        .await
                    {
                        Ok(response) => responses.push(response),
                        Err(err_response) => {
                            responses.push(err_response);
                            break;
                        }
                    }
                }
            }
        }

        responses
    }

    fn handle_websocket_text(
        &self,
        websocket_client: &crate::networking::websocket::WebSocketClient,
        text_bytes: tungstenite::Utf8Bytes,
    ) -> crate::exchange_client::common::Response {
        crate::finer!(
            "handle_websocket_text: websocket_client.url() = {}, text_bytes = {}",
            websocket_client.url(),
            text_bytes
        );
        let mut websocket_text = crate::networking::websocket::WebSocketText::from_text(
            websocket_client.url(),
            text_bytes,
        );

        crate::fine!("websocket_text: {:#?}", websocket_text);

        self.populate_websocket_text_payload_summary(&mut websocket_text);
        crate::fine!(
            "websocket_text payload summary: {:#?}",
            websocket_text.payload_summary
        );

        if self.is_websocket_text_subscription_data(&websocket_text) {
            if self.is_websocket_text_top_of_book_subscription_data(&websocket_text) {
                return self
                    .create_subscribe_top_of_book_websocket_subscription_data(&websocket_text);
            } else if self.is_websocket_text_trade_subscription_data(&websocket_text) {
                return self.create_subscribe_trade_websocket_subscription_data(&websocket_text);
            } else if self.is_websocket_text_order_subscription_data(&websocket_text) {
                return self.create_subscribe_order_websocket_subscription_data(&websocket_text);
            } else if self.is_websocket_text_fill_subscription_data(&websocket_text) {
                return self.create_subscribe_fill_websocket_subscription_data(&websocket_text);
            } else if self.is_websocket_text_unneeded_subscription_data(&websocket_text) {
                return crate::exchange_client::common::Response::Unneeded(websocket_text);
            }
            panic!(
                "Unexpected websocket subscription data: {:#?}",
                websocket_text
            );
        } else if self.is_websocket_text_success_response(&websocket_text) {
            if self.is_websocket_text_heartbeat_response(&websocket_text) {
                return self.create_heartbeat_websocket_response(&websocket_text);
            } else if self.is_websocket_text_subscribe_success_response(&websocket_text) {
                return self.create_subscribe_websocket_response(&websocket_text);
            } else if self.is_websocket_text_authenticate_success_response(&websocket_text) {
                return self.create_authenticate_websocket_response(&websocket_text);
            }
            panic!(
                "Unexpected websocket success response: {:#?}",
                websocket_text
            );
        } else {
            self.create_websocket_error_response(&websocket_text)
        }
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

    fn is_websocket_text_unneeded_subscription_data(
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
