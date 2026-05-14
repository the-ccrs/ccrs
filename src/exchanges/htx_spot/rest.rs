#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest for crate::exchanges::htx_spot::common::HtxSpotClient {
    fn create_get_instrument_info_http_request(
        &self,
        _get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/v1/common/symbols",
            None,
            None,
            None,
        )
    }

    fn create_get_top_of_book_http_request(
        &self,
        get_top_of_book_request: &crate::exchange_client::common::GetTopOfBookRequest,
    ) -> crate::networking::http::HttpRequest {
        if !get_top_of_book_request.symbol.is_empty() {
            let mut query_params: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();

            query_params.insert("symbol".into(), get_top_of_book_request.symbol.clone());

            crate::networking::http::HttpRequest::new(
                &self.rest_api_base_url,
                reqwest::Method::GET,
                "/market/detail/merged",
                None,
                Some(query_params),
                None,
            )
        } else {
            crate::networking::http::HttpRequest::new(
                &self.rest_api_base_url,
                reqwest::Method::GET,
                "/market/tickers",
                None,
                None,
                None,
            )
        }
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
                    crate::exchanges::htx_spot::common::HtxSpotClient::percent_encode_htx(k),
                    crate::exchanges::htx_spot::common::HtxSpotClient::percent_encode_htx(v),
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
            crate::exchanges::htx_spot::common::HtxSpotClient::percent_encode_htx(&signature_b64);

        http_request.query_string =
            Some(format!("{}&Signature={}", encoded_query, signature_encoded));
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        body_map.insert("account-id".into(), serde_json::json!(self.account_id));
        body_map.insert(
            "symbol".into(),
            serde_json::json!(place_order_request.symbol),
        );
        body_map.insert(
            "type".into(),
            serde_json::json!(self.convert_order_side_and_type_to_string(
                place_order_request.side,
                place_order_request.order_type,
            )),
        );
        body_map.insert(
            "amount".into(),
            serde_json::json!(place_order_request.quantity),
        );

        if place_order_request.order_type == crate::types::OrderType::Limit {
            body_map.insert("price".into(), serde_json::json!(place_order_request.price));
        }

        if !place_order_request.client_order_id.is_empty() {
            body_map.insert(
                "client-order-id".into(),
                serde_json::json!(place_order_request.client_order_id),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/v1/order/orders/place",
            Some(reqwest::header::HeaderMap::new()),
            None,
            Some(body_value),
        )
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        if !cancel_order_request.order_id.is_empty() {
            crate::networking::http::HttpRequest::new(
                &self.rest_api_base_url,
                reqwest::Method::POST,
                &format!(
                    "/v1/order/orders/{}/submitcancel",
                    cancel_order_request.order_id
                ),
                Some(reqwest::header::HeaderMap::new()),
                None,
                None,
            )
        } else {
            let body_value = serde_json::json!({
                "client-order-id": cancel_order_request.client_order_id
            });

            crate::networking::http::HttpRequest::new(
                &self.rest_api_base_url,
                reqwest::Method::POST,
                "/v1/order/orders/submitCancelClientOrder",
                Some(reqwest::header::HeaderMap::new()),
                None,
                Some(body_value),
            )
        }
    }

    fn create_get_open_order_http_request(
        &self,
        get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if !self.account_id.is_empty() {
            query_params.insert("account-id".into(), self.account_id.clone());
        }

        if !get_open_order_request.symbol.is_empty() {
            query_params.insert("symbol".into(), get_open_order_request.symbol.clone());
        }

        if get_open_order_request.limit > 0 {
            query_params.insert("size".into(), get_open_order_request.limit.to_string());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/v1/order/openOrders",
            None,
            if query_params.is_empty() {
                None
            } else {
                Some(query_params)
            },
            None,
        )
    }

    fn create_get_position_http_request(
        &self,
        _get_position_request: &crate::exchange_client::common::GetPositionRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/v1/account/accounts",
            None,
            None,
            None,
        )
    }

    fn create_get_balance_http_request(
        &self,
        _get_balance_request: &crate::exchange_client::common::GetBalanceRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &format!("/v1/account/accounts/{}/balance", self.account_id),
            None,
            None,
            None,
        )
    }

    fn create_get_account_info_http_request(
        &self,
        _get_account_info_request: &crate::exchange_client::common::GetAccountInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/v1/account/accounts",
            None,
            None,
            None,
        )
    }

    fn is_http_response_success(
        &self,
        http_response: &crate::networking::http::HttpResponse,
    ) -> bool {
        if let Some(json) = &http_response.json_payload {
            json["status"].as_str() == Some("ok")
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

        if let Some(data_array) = json_payload["data"].as_array() {
            response.data.reserve(data_array.len());
            for item in data_array {
                response
                    .data
                    .push(self.convert_json_value_to_instrument_info(item));
            }
        }

        crate::exchange_client::common::Response::GetInstrumentInfo(response)
    }

    fn create_get_top_of_book_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetTopOfBookResponse::default();

        let timestamp = chrono::Utc::now();

        if let Some(data_array) = json_payload["data"].as_array() {
            response.data.reserve(data_array.len());
            for item in data_array {
                let bid_price = item["bid"]
                    .as_f64()
                    .map(|v| format!("{}", v))
                    .unwrap_or_default();
                let bid_size = item["bidSize"]
                    .as_f64()
                    .map(|v| format!("{}", v))
                    .unwrap_or_default();
                let ask_price = item["ask"]
                    .as_f64()
                    .map(|v| format!("{}", v))
                    .unwrap_or_default();
                let ask_size = item["askSize"]
                    .as_f64()
                    .map(|v| format!("{}", v))
                    .unwrap_or_default();

                response.data.push(crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxSpot,
                    symbol: item["symbol"].as_str().unwrap_or("").to_string(),
                    timestamp,
                    bid_price,
                    bid_size,
                    ask_price,
                    ask_size,
                });
            }
        } else if json_payload["tick"].is_object() {
            let tick = &json_payload["tick"];
            let symbol = json_payload["ch"]
                .as_str()
                .unwrap_or("")
                .split('.')
                .nth(1)
                .unwrap_or("")
                .to_string();

            let bid_price = tick["bid"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_f64())
                .map(|v| format!("{}", v))
                .unwrap_or_default();
            let bid_size = tick["bid"]
                .as_array()
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_f64())
                .map(|v| format!("{}", v))
                .unwrap_or_default();
            let ask_price = tick["ask"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_f64())
                .map(|v| format!("{}", v))
                .unwrap_or_default();
            let ask_size = tick["ask"]
                .as_array()
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_f64())
                .map(|v| format!("{}", v))
                .unwrap_or_default();

            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::HtxSpot,
                symbol,
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

        let order_id = json_payload["data"].as_str().unwrap_or("").to_string();

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

        if let Some(data_array) = json_payload["data"].as_array() {
            response.data = data_array
                .iter()
                .map(|item| self.convert_json_value_to_order(item))
                .collect();
        }

        crate::exchange_client::common::Response::GetOpenOrder(response)
    }

    fn create_get_position_rest_response(
        &self,
        _http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        crate::exchange_client::common::Response::GetPosition(
            crate::exchange_client::common::GetPositionResponse::default(),
        )
    }

    fn create_get_balance_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetBalanceResponse::default();

        if let Some(list) = json_payload["data"]["list"].as_array() {
            response.data = list
                .iter()
                .filter(|item| item["type"].as_str() == Some("trade"))
                .map(|item| self.convert_json_value_to_balance(item))
                .collect();
        }

        crate::exchange_client::common::Response::GetBalance(response)
    }

    fn create_get_account_info_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetAccountInfoResponse::default();

        if let Some(list) = json_payload["data"].as_array() {
            response.data = list
                .iter()
                .filter(|item| item["type"].as_str() == Some("spot"))
                .map(|item| crate::types::AccountInfo {
                    exchange: crate::types::Exchange::HtxSpot,
                    id: item["id"].to_string(),
                    account_type: item["type"].as_str().unwrap_or_default().to_string(),
                })
                .collect();
        }

        crate::exchange_client::common::Response::GetAccountInfo(response)
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

        new_http_response.error_code = json_payload
            .get("err-code")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        new_http_response.error_message = json_payload
            .get("err-msg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
