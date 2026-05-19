#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest
    for crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient
{
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if !get_instrument_info_request.symbol.is_empty() {
            query_params.insert(
                "contract_code".into(),
                get_instrument_info_request.symbol.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/linear-swap-api/v1/swap_contract_info",
            None,
            if query_params.is_empty() {
                None
            } else {
                Some(query_params)
            },
            None,
        )
    }

    fn create_get_top_of_book_http_request(
        &self,
        get_top_of_book_request: &crate::exchange_client::common::GetTopOfBookRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if !get_top_of_book_request.symbol.is_empty() {
            query_params.insert(
                "contract_code".into(),
                get_top_of_book_request.symbol.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/linear-swap-ex/market/bbo",
            None,
            if query_params.is_empty() {
                None
            } else {
                Some(query_params)
            },
            None,
        )
    }

    fn sign_http_request(
        &self,
        http_request: &mut crate::networking::http::HttpRequest,
        now: chrono::DateTime<chrono::Utc>,
    ) {
        let credential = self.credential.as_ref().unwrap();

        let timestamp = now.format("%Y-%m-%dT%H:%M:%S").to_string();

        let base_url_parsed = url::Url::parse(&self.rest_api_base_url).unwrap();
        let host = base_url_parsed.host_str().unwrap_or("").to_string();

        let query_params = http_request
            .query_params
            .get_or_insert_with(std::collections::HashMap::new);

        query_params.insert("AccessKeyId".to_string(), credential.api_key.clone());
        query_params.insert("SignatureMethod".to_string(), "HmacSHA256".to_string());
        query_params.insert("SignatureVersion".to_string(), "2".to_string());
        query_params.insert("Timestamp".to_string(), timestamp);

        let mut pairs: Vec<(String, String)> = query_params
            .iter()
            .map(|(k, v)| {
                (
                    crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::percent_encode_htx(k),
                    crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::percent_encode_htx(v),
                )
            })
            .collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));

        let encoded_query: String = pairs
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let prehash = format!(
            "{}\n{}\n{}\n{}",
            http_request.method.as_str(),
            host,
            http_request.path,
            encoded_query
        );

        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .unwrap();
        <hmac::Hmac<sha2::Sha256> as hmac::Mac>::update(&mut mac, prehash.as_bytes());
        let signature_bytes = <hmac::Hmac<sha2::Sha256> as hmac::Mac>::finalize(mac).into_bytes();

        let signature_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature_bytes);
        let signature_encoded =
            crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::percent_encode_htx(&signature_b64);

        http_request.query_string =
            Some(format!("{}&Signature={}", encoded_query, signature_encoded));
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        body_map.insert("margin_mode".into(), serde_json::json!("cross"));

        body_map.insert(
            "contract_code".into(),
            serde_json::json!(place_order_request.symbol),
        );

        body_map.insert(
            "side".into(),
            serde_json::json!(self.convert_order_side_to_string(place_order_request.side)),
        );

        body_map.insert(
            "type".into(),
            serde_json::json!(self.convert_order_type_to_string(place_order_request.order_type)),
        );

        let volume = place_order_request
            .quantity
            .parse::<f64>()
            .unwrap_or(0.0)
            .round() as i64;

        body_map.insert("volume".into(), serde_json::json!(volume));

        if place_order_request.order_type == crate::types::OrderType::Limit {
            let price = place_order_request.price.parse::<f64>().unwrap_or(0.0);

            body_map.insert("price".into(), serde_json::json!(price));
        }

        if !place_order_request.client_order_id.is_empty() {
            body_map.insert(
                "client_order_id".into(),
                serde_json::json!(place_order_request.client_order_id),
            );
        }

        body_map.insert("channel_code".into(), serde_json::json!(self.api_broker_id));

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/v5/trade/order",
            Some(reqwest::header::HeaderMap::new()),
            None,
            Some(body_value),
        )
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        if !cancel_order_request.symbol.is_empty() {
            body_map.insert(
                "contract_code".into(),
                serde_json::json!(cancel_order_request.symbol),
            );
        }

        if !cancel_order_request.order_id.is_empty() {
            body_map.insert(
                "order_id".into(),
                serde_json::json!(cancel_order_request.order_id),
            );
        } else if !cancel_order_request.client_order_id.is_empty() {
            if let Ok(cid) = cancel_order_request.client_order_id.parse::<i64>() {
                body_map.insert("client_order_id".into(), serde_json::json!(cid));
            } else {
                body_map.insert(
                    "client_order_id".into(),
                    serde_json::json!(cancel_order_request.client_order_id),
                );
            }
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/v5/trade/cancel_order",
            Some(reqwest::header::HeaderMap::new()),
            None,
            Some(body_value),
        )
    }

    fn create_get_open_order_http_request(
        &self,
        get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        if !get_open_order_request.symbol.is_empty() {
            body_map.insert(
                "contract_code".into(),
                serde_json::json!(get_open_order_request.symbol),
            );
        }

        if get_open_order_request.limit > 0 {
            body_map.insert(
                "limit".into(),
                serde_json::json!(get_open_order_request.limit),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/v5/trade/order/opens",
            Some(reqwest::header::HeaderMap::new()),
            None,
            Some(body_value),
        )
    }

    fn create_get_position_http_request(
        &self,
        get_position_request: &crate::exchange_client::common::GetPositionRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        if !get_position_request.symbol.is_empty() {
            body_map.insert(
                "contract_code".into(),
                serde_json::json!(get_position_request.symbol),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/v5/trade/position/opens",
            Some(reqwest::header::HeaderMap::new()),
            None,
            Some(body_value),
        )
    }

    fn create_get_balance_http_request(
        &self,
        _get_balance_request: &crate::exchange_client::common::GetBalanceRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/v5/account/balance",
            Some(reqwest::header::HeaderMap::new()),
            None,
            None,
        )
    }

    fn is_http_response_success(
        &self,
        http_response: &crate::networking::http::HttpResponse,
    ) -> bool {
        if let Some(json) = &http_response.json_payload {
            json["status"].as_str() == Some("ok") || json["code"].as_i64() == Some(200)
        } else {
            http_response.status.is_success()
        }
    }

    fn create_get_instrument_info_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetInstrumentInfoResponse::default();

        let data_array = match json_payload.get("data").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetInstrumentInfo(response),
        };

        response.data.reserve(data_array.len());

        for item in data_array {
            response
                .data
                .push(self.convert_json_value_to_instrument_info(item));
        }

        crate::exchange_client::common::Response::GetInstrumentInfo(response)
    }

    fn create_get_top_of_book_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetTopOfBookResponse::default();

        let ticks = match json_payload.get("ticks").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetTopOfBook(response),
        };

        response.data.reserve(ticks.len());

        for item in ticks {
            let timestamp = {
                let ms = item["mtime"].as_i64().unwrap_or(0);
                if ms > 0 {
                    crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ms)
                } else {
                    chrono::Utc::now()
                }
            };

            let (bid_price, bid_size) = if let Some(arr) = item["bid"].as_array() {
                (
                    crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                        arr.first().unwrap_or(&serde_json::Value::Null),
                    ),
                    crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                        arr.get(1).unwrap_or(&serde_json::Value::Null),
                    ),
                )
            } else {
                (
                    crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                        &item["bid"],
                    ),
                    crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                        &item["bidSize"],
                    ),
                )
            };

            let (ask_price, ask_size) = if let Some(arr) = item["ask"].as_array() {
                (
                    crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                        arr.first().unwrap_or(&serde_json::Value::Null),
                    ),
                    crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                        arr.get(1).unwrap_or(&serde_json::Value::Null),
                    ),
                )
            } else {
                (
                    crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                        &item["ask"],
                    ),
                    crate::exchanges::htx_usdt_margined_futures::common::HtxUsdtMarginedFuturesClient::json_number_to_string(
                        &item["askSize"],
                    ),
                )
            };

            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type:
                    crate::types::ExchangeInstrumentType::HtxUsdtMarginedFutures,
                symbol: item["contract_code"].as_str().unwrap_or("").to_string(),
                timestamp,
                bid_price,
                bid_size,
                ask_price,
                ask_size,
            });
        }

        crate::exchange_client::common::Response::GetTopOfBook(response)
    }

    fn create_place_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let order_id = json_payload["data"]["order_id"]
            .as_str()
            .map(str::to_string)
            .unwrap();

        let response = crate::exchange_client::common::PlaceOrderResponse { order_id };

        crate::exchange_client::common::Response::PlaceOrder(response)
    }

    fn create_cancel_order_rest_response(
        &self,
        _http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        crate::exchange_client::common::Response::CancelOrder(
            crate::exchange_client::common::CancelOrderResponse::default(),
        )
    }

    fn create_get_open_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetOpenOrderResponse::default();

        if let Some(orders) = json_payload["data"].as_array() {
            response.data = orders
                .iter()
                .map(|item| self.convert_json_value_to_order(item))
                .collect();
        }

        crate::exchange_client::common::Response::GetOpenOrder(response)
    }

    fn create_get_position_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetPositionResponse::default();

        if let Some(list) = json_payload.get("data").and_then(|v| v.as_array()) {
            response.data = list
                .iter()
                .map(|item| self.convert_json_value_to_position(item))
                .collect();
        }

        crate::exchange_client::common::Response::GetPosition(response)
    }

    fn create_get_balance_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetBalanceResponse::default();

        if let Some(list) = json_payload
            .get("data")
            .unwrap()
            .get("details")
            .and_then(|v| v.as_array())
        {
            response.data = list
                .iter()
                .map(|item| self.convert_json_value_to_balance(item))
                .collect();
        }

        crate::exchange_client::common::Response::GetBalance(response)
    }

    fn create_http_error_response(
        &self,
        http_response: &crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = match http_response.json_payload.as_ref() {
            Some(payload) => payload,
            None => {
                return crate::exchange_client::common::Response::HttpErrorResponse(
                    http_response.clone(),
                );
            }
        };

        let mut new_http_response = http_response.clone();

        new_http_response.error_code = json_payload.get("err_code").and_then(|v| {
            v.as_i64()
                .map(|n| n.to_string())
                .or_else(|| v.as_str().map(str::to_string))
        });

        new_http_response.error_message = json_payload
            .get("err_msg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
