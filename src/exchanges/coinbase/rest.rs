#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest for crate::exchanges::coinbase::common::CoinbaseClient {
    fn create_get_instrument_info_http_request(
        &self,
        _get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/products",
            None,
            None,
            None,
        )
    }

    fn create_get_top_of_book_http_request(
        &self,
        get_top_of_book_request: &crate::exchange_client::common::GetTopOfBookRequest,
    ) -> crate::networking::http::HttpRequest {
        let path = format!("/products/{}/book", get_top_of_book_request.symbol);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &path,
            None,
            None,
            None,
        )
    }

    fn sign_http_request(
        &self,
        http_request: &mut crate::networking::http::HttpRequest,
        now: chrono::DateTime<chrono::Utc>,
    ) {
        let credential = self.credential.as_ref().unwrap();

        let timestamp = now.timestamp().to_string();

        let method = http_request.method.as_str();

        let request_path = if let Some(ref qs) = http_request.query_string {
            format!("{}?{}", http_request.path, qs)
        } else {
            http_request.path.clone()
        };

        let body = if http_request.method == reqwest::Method::POST
            || http_request.method == reqwest::Method::DELETE
        {
            if let Some(ref payload) = http_request.payload {
                payload.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let prehash = format!("{}{}{}{}", timestamp, method, request_path, body);

        let decoded_secret = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            credential.api_secret.as_bytes(),
        )
        .unwrap();

        let mut mac = hmac_sha256::HMAC::new(decoded_secret.as_slice());
        mac.update(prehash.as_bytes());
        let signature_bytes = mac.finalize();
        let signature =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature_bytes);

        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);

        headers.insert(
            reqwest::header::HeaderName::from_static("cb-access-key"),
            reqwest::header::HeaderValue::from_str(&credential.api_key).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("cb-access-timestamp"),
            reqwest::header::HeaderValue::from_str(&timestamp).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("cb-access-sign"),
            reqwest::header::HeaderValue::from_str(&signature).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("cb-access-passphrase"),
            reqwest::header::HeaderValue::from_str(&credential.api_passphrase).unwrap(),
        );
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        body_map.insert(
            "product_id".into(),
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
        body_map.insert(
            "size".into(),
            serde_json::json!(place_order_request.quantity),
        );

        if place_order_request.order_type == crate::types::OrderType::Limit {
            body_map.insert("price".into(), serde_json::json!(place_order_request.price));
        }

        if !place_order_request.client_order_id.is_empty() {
            body_map.insert(
                "client_oid".into(),
                serde_json::json!(place_order_request.client_order_id),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/orders",
            None,
            None,
            Some(body_value),
        )
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let order_id = if !cancel_order_request.order_id.is_empty() {
            cancel_order_request.order_id.clone()
        } else {
            format!("client:{}", cancel_order_request.client_order_id)
        };

        let path = format!("/orders/{}", order_id);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::DELETE,
            &path,
            None,
            None,
            None,
        )
    }

    fn create_get_open_order_http_request(
        &self,
        get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if !get_open_order_request.symbol.is_empty() {
            query_params.insert("product_id".into(), get_open_order_request.symbol.clone());
        }

        if get_open_order_request.limit > 0 {
            query_params.insert("limit".into(), get_open_order_request.limit.to_string());
        }

        if !get_open_order_request.next_page_cursor.is_empty() {
            query_params.insert(
                "after".into(),
                get_open_order_request.next_page_cursor.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/orders",
            None,
            Some(query_params),
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
            "/accounts",
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
            "/accounts",
            None,
            None,
            None,
        )
    }

    fn is_http_response_success(
        &self,
        http_response: &crate::networking::http::HttpResponse,
    ) -> bool {
        http_response.status.is_success()
    }

    fn create_get_instrument_info_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetInstrumentInfoResponse::default();

        let data_array = match json_payload.as_array() {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetInstrumentInfo(response),
        };

        response.data.reserve(data_array.len());

        for item in data_array {
            response.data.push(crate::types::InstrumentInfo {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
                symbol: item["id"].as_str().unwrap_or("").to_string(),
                base_asset: item["base_currency"].as_str().unwrap_or("").to_string(),
                quote_asset: item["quote_currency"].as_str().unwrap_or("").to_string(),
                order_price_increment: item["quote_increment"].as_str().unwrap_or("").to_string(),
                order_quantity_increment: item["base_increment"].as_str().unwrap_or("").to_string(),
                order_quote_quantity_min: item["min_market_funds"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
                ..Default::default()
            });
        }

        crate::exchange_client::common::Response::GetInstrumentInfo(response)
    }

    fn create_get_top_of_book_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let symbol = http_response
            .http_request
            .path
            .trim_start_matches("/products/")
            .trim_end_matches("/book")
            .to_string();

        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetTopOfBookResponse::default();

        let timestamp = json_payload["time"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let bid = &json_payload["bids"][0];
        let ask = &json_payload["asks"][0];

        response.data.push(crate::types::TopOfBook {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Coinbase,
            symbol,
            timestamp,
            bid_price: bid[0].as_str().unwrap_or("").to_string(),
            bid_size: bid[1].as_str().unwrap_or("").to_string(),
            ask_price: ask[0].as_str().unwrap_or("").to_string(),
            ask_size: ask[1].as_str().unwrap_or("").to_string(),
        });

        crate::exchange_client::common::Response::GetTopOfBook(response)
    }

    fn create_place_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let response = crate::exchange_client::common::PlaceOrderResponse {
            order_id: json_payload["id"].as_str().unwrap_or("").to_string(),
        };

        crate::exchange_client::common::Response::PlaceOrder(response)
    }

    fn create_cancel_order_rest_response(
        &self,
        _http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let response = crate::exchange_client::common::CancelOrderResponse::default();

        crate::exchange_client::common::Response::CancelOrder(response)
    }

    fn create_get_open_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetOpenOrderResponse::default();

        if let Some(list) = json_payload.as_array() {
            response.data = list
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

        if let Some(list) = json_payload.as_array() {
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

        new_http_response.error_message = json_payload
            .get("message")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
