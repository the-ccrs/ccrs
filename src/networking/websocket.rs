#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub connect_timeout_secs: u64,
    pub close_timeout_secs: u64,
    pub heartbeat_interval_secs: u64,
    pub proxy_url: Option<String>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            connect_timeout_secs: 10,
            close_timeout_secs: 10,
            heartbeat_interval_secs: 10,
            proxy_url: None,
        }
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
        &self.url
    }

    pub async fn send(&self, message: impl Into<String>) -> anyhow::Result<()> {
        self.send_channel_tx
            .send(message.into())
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(())
    }

    pub async fn ping(&self, payload: impl Into<bytes::Bytes>) -> anyhow::Result<()> {
        self.ping_channel_tx
            .send(payload.into())
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(())
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
        self.normal_cancellation_token.cancel();
        self.cancellation_token.cancel();
    }
}

impl WebSocketClient {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn cancellation_token(&self) -> &tokio_util::sync::CancellationToken {
        &self.cancellation_token
    }

    pub async fn close(&mut self) {
        if !self.closed {
            crate::finest!("WebSocketClient: close called for url: {}", self.url);
            self.closed = true;
            self.normal_cancellation_token.cancel();
        }
    }

    pub fn set_closed(&mut self) {
        if !self.closed {
            crate::finest!("WebSocketClient: set_closed called for url: {}", self.url);
            self.closed = true;
            self.normal_cancellation_token.cancel();
            self.cancellation_token.cancel();
        }
    }

    pub fn sender(&self) -> WebSocketSender {
        self.websocket_sender.clone()
    }

    pub async fn read_next(&mut self) -> anyhow::Result<tokio_tungstenite::tungstenite::Message> {
        tokio::select! {

            websocket_message = futures::StreamExt::next(&mut self.reader) => {
                match websocket_message {
                    Some(Ok(message)) => {
                        Ok(message)
                    },
                    Some(Err(err)) => {
                        self.set_closed();
                        Err(anyhow::anyhow!(err))
                    },
                    None => {
                        self.set_closed();
                        Err(anyhow::anyhow!("Websocket closed"))
                    },
                }
            }

            sender_err = &mut self.sender_err_rx => {
                match sender_err {
                    Ok(err) => {
                        self.set_closed();
                        Err(anyhow::anyhow!(err))
                    },
                    Err(_) => {
                        self.set_closed();
                        Err(anyhow::anyhow!("Sender task dropped"))
                    },
                }
            }
        }
    }

    pub fn builder(
        url: impl Into<String>,
        websocket_config: WebSocketConfig,
        headers: Option<Vec<(String, String)>>,
    ) -> WebSocketClientBuilder {
        WebSocketClientBuilder {
            url: url.into(),
            websocket_config,
            headers,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct WebSocketClientBuilder {
    url: String,
    websocket_config: WebSocketConfig,
    headers: Option<Vec<(String, String)>>,
}

impl WebSocketClientBuilder {
    fn parse_host_port(&self) -> anyhow::Result<(String, u16), anyhow::Error> {
        let url = url::Url::parse(&self.url)?;
        let host = url
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid host: {}", url))?
            .to_string();
        let port = url
            .port_or_known_default()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine port: {}", url))?;
        Ok((host, port))
    }

    async fn connect_tcp(&self, host: &str, port: u16) -> anyhow::Result<tokio::net::TcpStream> {
        if let Some(proxy_url) = &self.websocket_config.proxy_url {
            let url = url::Url::parse(proxy_url)?;
            match url.scheme() {
                "socks5" => {
                    let proxy_addr = format!(
                        "{}:{}",
                        url.host_str().unwrap(),
                        url.port_or_known_default().unwrap_or(1080)
                    );
                    let dest_addr = format!("{}:{}", host, port);

                    let stream = tokio_socks::tcp::Socks5Stream::connect(
                        proxy_addr.as_str(),
                        dest_addr.as_str(),
                    )
                    .await?;
                    Ok(stream.into_inner())
                }
                "http" | "https" => {
                    let proxy_addr = format!(
                        "{}:{}",
                        url.host_str().unwrap(),
                        url.port_or_known_default().unwrap_or(8080)
                    );
                    let mut stream = tokio::net::TcpStream::connect(proxy_addr).await?;
                    let connect_req = format!(
                        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
                        host, port, host, port
                    );
                    tokio::io::AsyncWriteExt::write_all(&mut stream, connect_req.as_bytes())
                        .await?;
                    let mut response = vec![0u8; 1024];
                    let n = tokio::io::AsyncReadExt::read(&mut stream, &mut response).await?;
                    let resp_str = String::from_utf8_lossy(&response[..n]);
                    if !resp_str.contains("200 Connection established") {
                        return Err(anyhow::anyhow!("HTTP proxy CONNECT failed: {}", resp_str));
                    }
                    Ok(stream)
                }
                other => Err(anyhow::anyhow!("Unsupported proxy scheme: {}", other)),
            }
        } else {
            tokio::net::TcpStream::connect(format!("{}:{}", host, port))
                .await
                .map_err(|err| anyhow::anyhow!(err))
        }
    }

    pub async fn build(self) -> anyhow::Result<WebSocketClient> {
        let (host, port) = self.parse_host_port()?;

        let connect_future = async {
            let tcp_stream = self.connect_tcp(&host, port).await?;
            tcp_stream.set_nodelay(true)?;

            let stream: tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream> =
                if self.url.starts_with("wss://") {
                    let tls_connector =
                        tokio_native_tls::TlsConnector::from(native_tls::TlsConnector::new()?);
                    let tls_stream = tls_connector.connect(&host, tcp_stream).await?;
                    tokio_tungstenite::MaybeTlsStream::NativeTls(tls_stream)
                } else {
                    tokio_tungstenite::MaybeTlsStream::Plain(tcp_stream)
                };

            let mut request =
                tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(
                    self.url.clone(),
                )?;

            if let Some(headers) = &self.headers {
                for (name, value) in headers {
                    request.headers_mut().insert(
                        tokio_tungstenite::tungstenite::http::header::HeaderName::from_bytes(
                            name.as_bytes(),
                        )
                        .map_err(|e| anyhow::anyhow!("Invalid header name '{}': {}", name, e))?,
                        tokio_tungstenite::tungstenite::http::header::HeaderValue::from_str(value)
                            .map_err(|e| {
                                anyhow::anyhow!("Invalid header value for '{}': {}", name, e)
                            })?,
                    );
                }
            }

            crate::fine!("WebSocket handshake request:\n{:#?}", request);
            let result = tokio_tungstenite::client_async(request, stream).await;

            match result {
                Ok(ok) => Ok(ok),
                Err(err) => {
                    if let tokio_tungstenite::tungstenite::Error::Http(response) = &err
                        && let Some(body) = response.body()
                    {
                        let text = String::from_utf8_lossy(body);
                        crate::fine!("WebSocket handshake failed body:\n{}", text);
                    }

                    Err(anyhow::anyhow!(err))
                }
            }
        };

        let timeout_duration =
            std::time::Duration::from_secs(self.websocket_config.connect_timeout_secs);

        let (ws_stream, _) = tokio::time::timeout(timeout_duration, connect_future)
            .await
            .map_err(|_| anyhow::anyhow!("WebSocket connection timed out"))??;

        let (mut sender, reader) = futures::StreamExt::split(ws_stream);

        let (send_channel_tx, mut send_channel_rx) = tokio::sync::mpsc::channel::<String>(1);
        let (ping_channel_tx, mut ping_channel_rx) = tokio::sync::mpsc::channel::<bytes::Bytes>(1);

        let (sender_err_tx, sender_err_rx) =
            tokio::sync::oneshot::channel::<tokio_tungstenite::tungstenite::Error>();

        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let cancellation_token_clone = cancellation_token.clone();
        let normal_cancellation_token = tokio_util::sync::CancellationToken::new();
        let normal_cancellation_token_clone = normal_cancellation_token.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    maybe_text = send_channel_rx.recv() => {
                        match maybe_text {
                            Some(text) => {
                                let message = tokio_tungstenite::tungstenite::Message::Text(text.into());

                                if let Err(err) = futures::SinkExt::send(&mut sender, message).await {
                                    crate::error!("Websocket send error: {err}");
                                    let _ = sender_err_tx.send(err);
                                    break;
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    maybe_bytes = ping_channel_rx.recv() => {
                        match maybe_bytes {
                            Some(bytes) => {
                                let message = tokio_tungstenite::tungstenite::Message::Ping(bytes);

                                if let Err(err) = futures::SinkExt::send(&mut sender, message).await {
                                    crate::error!("Websocket send error: {err}");
                                    let _ = sender_err_tx.send(err);
                                    break;
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    _ = normal_cancellation_token_clone.cancelled() => {
                        crate::finest!("Sender task cancelled by token");
                        let _ = tokio::time::timeout(
                            std::time::Duration::from_secs(self.websocket_config.close_timeout_secs),
                            futures_util::sink::SinkExt::send(&mut sender, tokio_tungstenite::tungstenite::Message::Close(Some(tokio_tungstenite::tungstenite::protocol::frame::CloseFrame {
                            code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,

                            reason: <tokio_tungstenite::tungstenite::Utf8Bytes as std::convert::From<&str>>::from("Closing"),
                        })))
                        ).await;
                        break;
                    }
                    _ = cancellation_token_clone.cancelled() => {
                        crate::finest!("Sender task cancelled by token");
                        break;
                    }
                }
            }

            let _ = futures::SinkExt::close(&mut sender).await;
            crate::finest!("Sender task exited");
        });

        let websocket_sender = WebSocketSender {
            url: self.url.clone(),
            send_channel_tx,
            ping_channel_tx,
        };

        let fused_reader = futures::StreamExt::fuse(reader);

        let websocket_client = WebSocketClient {
            url: self.url.clone(),
            websocket_config: self.websocket_config.clone(),
            websocket_sender,
            sender_err_rx,
            reader: fused_reader,
            closed: false,
            normal_cancellation_token: normal_cancellation_token.clone(),
            cancellation_token: cancellation_token.clone(),
        };

        Ok(websocket_client)
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
        let json_payload = serde_json::from_str::<serde_json::Value>(text_bytes.as_str()).ok();

        Self {
            url: url.into(),
            text: text_bytes.to_string(),
            json_payload,
            ..Default::default()
        }
    }
}
