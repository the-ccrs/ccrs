#[async_trait::async_trait]
impl crate::exchange_client::websocket::Websocket
    for crate::exchanges::binance_usds_margined_futures::common::BinanceUsdsMarginedFuturesClient
{
    fn websocket_api_url(&self, endpoint: crate::types::WebSocketEndpoint) -> String {
        match endpoint {
            crate::types::WebSocketEndpoint::BinanceUsdsMarginedFutures(binance_endpoint) => {
                match binance_endpoint {
                    crate::types::BinanceUsdsMarginedFuturesWebSocketEndpoint::MarketData => {
                        self.websocket_market_data_api_url.to_string()
                    }
                    crate::types::BinanceUsdsMarginedFuturesWebSocketEndpoint::AccountData => {
                        self.websocket_account_data_api_url.to_string()
                    }
                    crate::types::BinanceUsdsMarginedFuturesWebSocketEndpoint::Unknown => {
                        panic!("BinanceUsdsMarginedFutures WebSocket endpoint is Unknown")
                    }
                }
            }
            _ => panic!("WebSocket endpoint is not BinanceUsdsMarginedFutures"),
        }
    }

    async fn create_websocket_client(
        &self,
        websocket_client_config: crate::types::WebSocketClientConfig,
        websocket_config: crate::networking::websocket::WebSocketConfig,
    ) -> anyhow::Result<crate::networking::websocket::WebSocketClient> {
        let websocket_client = match &websocket_client_config.endpoint {
            crate::types::WebSocketEndpoint::BinanceUsdsMarginedFutures(
                crate::types::BinanceUsdsMarginedFuturesWebSocketEndpoint::MarketData,
            ) => {
                crate::networking::websocket::WebSocketClient::builder(
                    self.websocket_api_url(websocket_client_config.endpoint),
                    websocket_config.clone(),
                )
                .build()
                .await?
            }

            crate::types::WebSocketEndpoint::BinanceUsdsMarginedFutures(
                crate::types::BinanceUsdsMarginedFuturesWebSocketEndpoint::AccountData,
            ) => {
                let mut http_request = crate::networking::http::HttpRequest::new(
                    &self.rest_api_base_url,
                    reqwest::Method::POST,
                    "/fapi/v1/listenKey",
                    None,
                    Some(std::collections::HashMap::new()),
                    None,
                );
                let now = chrono::Utc::now();
                crate::exchange_client::rest::Rest::sign_http_request(self, &mut http_request, now);

                let http_client = match crate::exchange_client::rest::Rest::create_http_client(
                    self,
                    crate::networking::http::HttpConfig::default(),
                )
                .await
                {
                    Ok(client) => client,
                    Err(err) => {
                        return Err(anyhow::anyhow!("Unable to create http client: {:#?}", err));
                    }
                };

                let http_response = match crate::exchange_client::rest::execute_http_request(
                    &http_client,
                    http_request,
                )
                .await
                {
                    Ok(http_response) => http_response,
                    Err(err) => {
                        return Err(anyhow::anyhow!(
                            "Unable to start user data stream: {:#?}",
                            err
                        ));
                    }
                };

                let listen_key = http_response.json_payload.unwrap()["listenKey"]
                    .as_str()
                    .unwrap()
                    .to_string();

                let url = self
                    .websocket_account_data_api_url
                    .replace("{listen_key}", &listen_key);

                let websocket_client = crate::networking::websocket::WebSocketClient::builder(
                    url,
                    websocket_config.clone(),
                )
                .build()
                .await?;

                {
                    let rest_api_base_url = self.rest_api_base_url.clone();
                    let listen_key_clone = listen_key.clone();
                    let cancellation_token = websocket_client.cancellation_token().clone();

                    tokio::spawn(async move {
                        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                            3600_u64.div_ceil(4),
                        ));
                        interval.tick().await;

                        loop {
                            tokio::select! {
                                _ = interval.tick() => {
                                    let mut params = std::collections::HashMap::new();
                                    params.insert("listenKey".to_string(), listen_key_clone.clone());

                                    let http_request = crate::networking::http::HttpRequest::new(
                                        &rest_api_base_url,
                                        reqwest::Method::PUT,
                                        "/fapi/v1/listenKey",
                                        None,
                                        Some(params),
                                        None,
                                    );

                                    let _ = crate::exchange_client::rest::execute_http_request(&http_client, http_request).await;
                                }
                                _ = cancellation_token.cancelled() => {
                                    crate::finer!("Listen key refresh task received cancellation signal");
                                    break;
                                }
                            }
                        }
                    });
                }

                websocket_client
            }

            _ => panic!(),
        };

        self.keep_websocket_client_alive(
            websocket_config.heartbeat_interval_secs,
            websocket_client.sender().clone(),
            websocket_client.cancellation_token().clone(),
        )
        .await?;

        crate::finer!("Created websocket_client: {:#?}", websocket_client);

        Ok(websocket_client)
    }

    fn create_heartbeat_websocket_request_factory(&self) -> Box<dyn Fn() -> String + Send> {
        Box::new(|| "".to_string())
    }

    fn create_subscribe_top_of_book_websocket_request(
        &self,
        subscribe_top_of_book_request: &crate::exchange_client::common::SubscribeTopOfBookRequest,
    ) -> String {
        let params: Vec<String> = subscribe_top_of_book_request
            .symbols
            .iter()
            .map(|symbol| format!("{}@bookTicker", symbol.to_lowercase()))
            .collect();

        let mut payload = serde_json::Map::new();

        if let Some(id) = subscribe_top_of_book_request.id {
            payload.insert("id".to_string(), serde_json::json!(id));
        }

        payload.insert(
            "method".to_string(),
            serde_json::Value::String("SUBSCRIBE".to_string()),
        );
        payload.insert("params".to_string(), serde_json::to_value(params).unwrap());

        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_trade_websocket_request(
        &self,
        subscribe_trade_request: &crate::exchange_client::common::SubscribeTradeRequest,
    ) -> String {
        let params: Vec<String> = subscribe_trade_request
            .symbols
            .iter()
            .map(|symbol| format!("{}@trade", symbol.to_lowercase()))
            .collect();

        let mut payload = serde_json::Map::new();

        if let Some(id) = subscribe_trade_request.id {
            payload.insert("id".to_string(), serde_json::json!(id));
        }

        payload.insert(
            "method".to_string(),
            serde_json::Value::String("SUBSCRIBE".to_string()),
        );
        payload.insert("params".to_string(), serde_json::to_value(params).unwrap());

        serde_json::Value::Object(payload).to_string()
    }

    fn create_subscribe_order_websocket_request(
        &self,
        _subscribe_order_request: &crate::exchange_client::common::SubscribeOrderRequest,
    ) -> String {
        panic!()
    }

    fn create_subscribe_fill_websocket_request(
        &self,
        _subscribe_fill_request: &crate::exchange_client::common::SubscribeFillRequest,
    ) -> String {
        panic!()
    }

    fn populate_websocket_text_payload_summary(
        &self,
        websocket_text: &mut crate::networking::websocket::WebSocketText,
    ) {
        if let Some(json_payload) = &websocket_text.json_payload {
            if let Some(v) = json_payload.get("stream").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("stream".to_string(), v.to_string());
            }

            if let Some(v) = json_payload.get("e").and_then(|v| v.as_str()) {
                websocket_text
                    .payload_summary
                    .insert("e".to_string(), v.to_string());
            }

            if let Some(v) = json_payload.get("status").and_then(|v| v.as_i64()) {
                websocket_text
                    .payload_summary
                    .insert("status".to_string(), v.to_string());
            }

            if json_payload.get("result").is_some_and(|v| v.is_null()) {
                websocket_text
                    .payload_summary
                    .insert("result_null".to_string(), String::new());
            }

            if json_payload.get("error").is_some_and(|v| !v.is_null()) {
                websocket_text
                    .payload_summary
                    .insert("error".to_string(), String::new());
            }
        }
    }

    fn is_websocket_text_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary.contains_key("stream") || payload_summary.contains_key("e")
    }

    fn is_websocket_text_top_of_book_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("stream")
            .map(|stream| stream.ends_with("bookTicker"))
            .unwrap_or_default()
    }

    fn is_websocket_text_trade_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("stream")
            .map(|stream| stream.ends_with("trade"))
            .unwrap_or_default()
    }

    fn is_websocket_text_order_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        let payload_summary = &websocket_text.payload_summary;
        payload_summary
            .get("e")
            .map(|e| e == "ORDER_TRADE_UPDATE")
            .unwrap_or_default()
    }

    fn is_websocket_text_fill_subscription_data(
        &self,
        _: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
    }

    fn create_subscribe_top_of_book_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let data = json_payload.get("data").unwrap();

        let symbol = data.get("s").and_then(|v| v.as_str()).unwrap().to_string();
        let timestamp = chrono::Utc::now();

        let bid_price = data.get("b").and_then(|v| v.as_str()).unwrap().to_string();
        let bid_size = data.get("B").and_then(|v| v.as_str()).unwrap().to_string();
        let ask_price = data.get("a").and_then(|v| v.as_str()).unwrap().to_string();
        let ask_size = data.get("A").and_then(|v| v.as_str()).unwrap().to_string();

        crate::exchange_client::common::Response::TopOfBookSubscription(
            crate::exchange_client::common::TopOfBookSubscriptionData {
                data: vec![crate::types::TopOfBook {
                    exchange_instrument_type:
                        crate::types::ExchangeInstrumentType::BinanceUsdsMarginedFutures,
                    symbol,
                    timestamp,
                    bid_price,
                    bid_size,
                    ask_price,
                    ask_size,
                }],
            },
        )
    }

    fn create_subscribe_trade_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let data = &json_payload["data"];

        let symbol = data["s"].as_str().unwrap().to_string();

        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
            data["T"].as_i64().unwrap(),
        );

        let price = data["p"].as_str().unwrap().to_string();
        let size = data["q"].as_str().unwrap().to_string();

        let is_buyer_maker = data["m"].as_bool().unwrap();

        let side = if is_buyer_maker {
            crate::types::TakerSide::Sell
        } else {
            crate::types::TakerSide::Buy
        };

        crate::exchange_client::common::Response::TradeSubscription(
            crate::exchange_client::common::TradeSubscriptionData {
                data: vec![crate::types::Trade {
                    exchange_instrument_type:
                        crate::types::ExchangeInstrumentType::BinanceUsdsMarginedFutures,
                    symbol,
                    timestamp,
                    price,
                    size,
                    side,
                }],
            },
        )
    }

    fn create_subscribe_order_websocket_subscription_data(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let order = self.convert_ws_order_trade_update_to_order(json_payload);

        crate::exchange_client::common::Response::OrderSubscription(
            crate::exchange_client::common::OrderSubscriptionData { data: vec![order] },
        )
    }

    fn create_subscribe_fill_websocket_subscription_data(
        &self,
        _: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        panic!()
    }

    fn is_unexpected_websocket_text_subscription_data_benign(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.contains_key("e")
    }

    fn is_websocket_text_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text
            .payload_summary
            .get("status")
            .map(|v| v == "200")
            .unwrap_or_default()
            || (websocket_text.payload_summary.contains_key("result_null")
                && !websocket_text.payload_summary.contains_key("error"))
    }

    fn is_websocket_text_authenticate_success_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
    }

    fn is_websocket_text_subscribe_success_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        websocket_text.payload_summary.contains_key("result_null")
            && !websocket_text.payload_summary.contains_key("error")
    }

    fn is_websocket_text_heartbeat_response(
        &self,
        _websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> bool {
        false
    }

    fn create_authenticate_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let id = json_payload.get("id").and_then(|v| v.as_u64());

        crate::exchange_client::common::Response::Authenticate(
            crate::exchange_client::common::AuthenticateResponse { id },
        )
    }

    fn create_subscribe_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let id = json_payload.get("id").and_then(|v| v.as_u64());

        crate::exchange_client::common::Response::Subscribe(
            crate::exchange_client::common::SubscribeResponse {
                id,
                ..Default::default()
            },
        )
    }

    fn create_heartbeat_websocket_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = websocket_text.json_payload.as_ref().unwrap();

        let id = json_payload.get("id").and_then(|v| v.as_u64());

        crate::exchange_client::common::Response::Heartbeat(
            crate::exchange_client::common::HeartbeatResponse { id },
        )
    }

    fn create_websocket_error_response(
        &self,
        websocket_text: &crate::networking::websocket::WebSocketText,
    ) -> crate::exchange_client::common::Response {
        let json_payload = match websocket_text.json_payload.as_ref() {
            Some(payload) => payload,
            None => {
                return crate::exchange_client::common::Response::WebSocketErrorResponse(
                    websocket_text.clone(),
                );
            }
        };

        let mut new_websocket_text = websocket_text.clone();

        new_websocket_text.error_message = json_payload
            .get("msg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::WebSocketErrorResponse(new_websocket_text)
    }
}
