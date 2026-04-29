static DEFAULT_USER_AGENT: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| env!("CARGO_PKG_NAME").to_string());

#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub timeout_secs: u64,
    pub proxy_url: Option<String>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 10,
            proxy_url: None,
        }
    }
}

#[derive(Debug)]
pub struct HttpClient {
    #[allow(dead_code)]
    http_config: HttpConfig,
    client: reqwest::Client,
}

#[derive(Debug, Default, Clone)]
pub struct HttpClientBuilder {
    http_config: HttpConfig,
}

impl HttpClient {
    pub fn builder(http_config: HttpConfig) -> HttpClientBuilder {
        HttpClientBuilder { http_config }
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl HttpClientBuilder {
    pub fn build(self) -> anyhow::Result<HttpClient> {
        let mut builder = reqwest::Client::builder().tcp_nodelay(true);

        if let Some(proxy_url) = &self.http_config.proxy_url {
            builder = builder.proxy(reqwest::Proxy::all(proxy_url).unwrap());
        }

        builder = builder.timeout(std::time::Duration::from_secs(
            self.http_config.timeout_secs,
        ));

        let client = builder.build()?;

        Ok(HttpClient {
            http_config: self.http_config.clone(),
            client,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct HttpRequest {
    pub base_url: String,
    pub method: reqwest::Method,
    pub path: String,
    pub headers: Option<reqwest::header::HeaderMap>,
    pub query_params: Option<std::collections::HashMap<String, String>>,
    pub query_string: Option<String>,
    pub json_payload: Option<serde_json::Value>,
    pub payload: Option<String>,
}

impl HttpRequest {
    pub fn new(
        base_url: &str,
        method: reqwest::Method,
        path: &str,
        headers: Option<reqwest::header::HeaderMap>,
        query_params: Option<std::collections::HashMap<String, String>>,
        json_payload: Option<serde_json::Value>,
    ) -> Self {
        let mut headers = headers.unwrap_or_else(reqwest::header::HeaderMap::new);

        if !headers.contains_key(reqwest::header::USER_AGENT) {
            headers.insert(
                reqwest::header::USER_AGENT,
                DEFAULT_USER_AGENT.clone().parse().unwrap(),
            );
        }

        if json_payload.is_some() && !headers.contains_key(reqwest::header::CONTENT_TYPE) {
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                "application/json".parse().unwrap(),
            );
        }

        let query_string = if let Some(ref params) = query_params {
            if !params.is_empty() {
                let mut pairs: Vec<String> =
                    params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();

                pairs.sort();
                Some(pairs.join("&"))
            } else {
                None
            }
        } else {
            None
        };

        let payload = json_payload
            .as_ref()
            .map(|json| serde_json::to_string(json).unwrap());

        Self {
            base_url: base_url.to_string(),
            method,
            path: path.to_string(),
            headers: Some(headers),
            query_params,
            query_string,
            json_payload,
            payload,
        }
    }

    pub fn to_request(&self, client: &reqwest::Client) -> reqwest::Request {
        let mut url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            self.path.trim_start_matches('/')
        );
        if let Some(ref query_string) = self.query_string {
            url.push('?');
            url.push_str(query_string);
        }

        let url = reqwest::Url::parse(&url).unwrap();

        let mut builder = client.request(self.method.clone(), url);

        if let Some(ref headers) = self.headers {
            builder = builder.headers(headers.clone());
        }

        if let Some(ref payload) = self.payload {
            builder = builder.body(payload.clone());
        }

        builder.build().unwrap()
    }
}

#[derive(Debug, Default, Clone)]
pub struct HttpResponse {
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: String,
    pub json_payload: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub http_request: HttpRequest,
}

impl HttpResponse {
    pub async fn from_response(response: reqwest::Response, http_request: HttpRequest) -> Self {
        let status = response.status();
        let headers = response.headers().clone();

        let bytes = response.bytes().await.unwrap();

        let json_payload = serde_json::from_slice::<serde_json::Value>(&bytes).ok();

        let body = String::from_utf8_lossy(&bytes).into_owned();

        Self {
            status,
            headers,
            body,
            json_payload,
            http_request,
            ..Default::default()
        }
    }
}
