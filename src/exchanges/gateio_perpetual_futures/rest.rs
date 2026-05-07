#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest
    for crate::exchanges::gateio_perpetual_futures::common::GateioPerpetualFuturesClient
{
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let path = if !get_instrument_info_request.symbol.is_empty() {
            format!(
                "{}/futures/{}/contracts/{}",
                Self::REST_API_PREFIX,
                self.settle,
                get_instrument_info_request.symbol
            )
        } else {
            format!(
                "{}/futures/{}/contracts",
                Self::REST_API_PREFIX,
                self.settle
            )
        };

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &path,
            None,
            None,
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
            query_params.insert("contract".into(), get_top_of_book_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &format!("{}/futures/{}/tickers", Self::REST_API_PREFIX, self.settle),
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

        let timestamp = now.timestamp().to_string();

        let base_url_parsed = url::Url::parse(&self.rest_api_base_url).unwrap();
        let base_path = base_url_parsed.path();
        let sign_path = format!("{}{}", base_path.trim_end_matches('/'), http_request.path);

        let query_string = http_request
            .query_string
            .as_deref()
            .unwrap_or("")
            .to_string();

        let body = if http_request.method == reqwest::Method::POST {
            http_request.payload.as_deref().unwrap_or("").to_string()
        } else {
            String::new()
        };

        let mut body_hasher = sha2::Sha512::default();
        <sha2::Sha512 as sha2::Digest>::update(&mut body_hasher, body.as_bytes());
        let body_hash_bytes = <sha2::Sha512 as sha2::Digest>::finalize(body_hasher);
        let body_hash = hex::encode(body_hash_bytes);

        let prehash = format!(
            "{}\n{}\n{}\n{}\n{}",
            http_request.method.as_str(),
            sign_path,
            query_string,
            body_hash,
            timestamp
        );

        let mut mac = <hmac::Hmac<sha2::Sha512> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .unwrap();
        <hmac::Hmac<sha2::Sha512> as hmac::Mac>::update(&mut mac, prehash.as_bytes());
        let signature_bytes = <hmac::Hmac<sha2::Sha512> as hmac::Mac>::finalize(mac).into_bytes();
        let signature = hex::encode(signature_bytes);

        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);

        headers.insert(
            reqwest::header::HeaderName::from_static("key"),
            reqwest::header::HeaderValue::from_str(&credential.api_key).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("timestamp"),
            reqwest::header::HeaderValue::from_str(&timestamp).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("sign"),
            reqwest::header::HeaderValue::from_str(&signature).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("x-gate-channel-id"),
            reqwest::header::HeaderValue::from_str(&self.api_channel_id).unwrap(),
        );
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let qty = place_order_request.quantity.parse::<f64>().unwrap_or(0.0) as i64;
        let size = self.convert_order_side_to_signed_size(place_order_request.side, qty);
        let tif = self.convert_order_type_to_tif_string(place_order_request.order_type);

        let mut body_map = serde_json::Map::new();

        body_map.insert(
            "contract".into(),
            serde_json::json!(place_order_request.symbol),
        );
        body_map.insert("size".into(), serde_json::json!(size));
        body_map.insert("tif".into(), serde_json::json!(tif));

        match place_order_request.order_type {
            crate::types::OrderType::Limit => {
                body_map.insert("price".into(), serde_json::json!(place_order_request.price));
            }
            crate::types::OrderType::Market => {
                body_map.insert("price".into(), serde_json::json!("0"));
            }
            crate::types::OrderType::Unknown => panic!("Invalid order type"),
        }

        if !place_order_request.client_order_id.is_empty() {
            body_map.insert(
                "text".into(),
                serde_json::json!(place_order_request.client_order_id),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            &format!("{}/futures/{}/orders", Self::REST_API_PREFIX, self.settle),
            Some(reqwest::header::HeaderMap::new()),
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
            cancel_order_request.client_order_id.clone()
        };

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::DELETE,
            &format!(
                "{}/futures/{}/orders/{}",
                Self::REST_API_PREFIX,
                self.settle,
                order_id
            ),
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

        query_params.insert("status".into(), "open".into());

        if !get_open_order_request.symbol.is_empty() {
            query_params.insert("contract".into(), get_open_order_request.symbol.clone());
        }

        if get_open_order_request.limit > 0 {
            query_params.insert("limit".into(), get_open_order_request.limit.to_string());
        }

        if !get_open_order_request.next_page_cursor.is_empty() {
            query_params.insert(
                "last_id".into(),
                get_open_order_request.next_page_cursor.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &format!("{}/futures/{}/orders", Self::REST_API_PREFIX, self.settle),
            None,
            Some(query_params),
            None,
        )
    }

    fn create_get_position_http_request(
        &self,
        get_position_request: &crate::exchange_client::common::GetPositionRequest,
    ) -> crate::networking::http::HttpRequest {
        let path = if !get_position_request.symbol.is_empty() {
            format!(
                "{}/futures/{}/positions/{}",
                Self::REST_API_PREFIX,
                self.settle,
                get_position_request.symbol
            )
        } else {
            format!(
                "{}/futures/{}/positions",
                Self::REST_API_PREFIX,
                self.settle
            )
        };

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &path,
            None,
            None,
            None,
        )
    }

    fn create_get_balance_http_request(
        &self,
        get_balance_request: &crate::exchange_client::common::GetBalanceRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if !get_balance_request.asset.is_empty() {
            query_params.insert("currency".into(), get_balance_request.asset.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &format!("{}/unified/accounts", Self::REST_API_PREFIX),
            None,
            if query_params.is_empty() {
                None
            } else {
                Some(query_params)
            },
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

        if let Some(data_array) = json_payload.as_array() {
            response.data.reserve(data_array.len());
            for item in data_array {
                response
                    .data
                    .push(self.convert_json_value_to_instrument_info(item));
            }
        } else if json_payload.is_object() {
            response
                .data
                .push(self.convert_json_value_to_instrument_info(&json_payload));
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

        let data_array = match json_payload.as_array() {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetTopOfBook(response),
        };

        response.data.reserve(data_array.len());

        for item in data_array {
            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type:
                    crate::types::ExchangeInstrumentType::GateioPerpetualFutures,
                symbol: item["contract"].as_str().unwrap_or("").to_string(),
                timestamp,
                bid_price: item["highest_bid"].as_str().unwrap_or("").to_string(),
                bid_size: item["highest_size"].as_str().unwrap_or("").to_string(),
                ask_price: item["lowest_ask"].as_str().unwrap_or("").to_string(),
                ask_size: item["lowest_size"].as_str().unwrap_or("").to_string(),
            });
        }

        crate::exchange_client::common::Response::GetTopOfBook(response)
    }

    fn create_place_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let order_id = if let Some(id) = json_payload.get("id") {
            if let Some(n) = id.as_i64() {
                n.to_string()
            } else if let Some(s) = id.as_str() {
                s.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        crate::exchange_client::common::Response::PlaceOrder(
            crate::exchange_client::common::PlaceOrderResponse { order_id },
        )
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
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetPositionResponse::default();

        if let Some(list) = json_payload.as_array() {
            response.data = list
                .iter()
                .map(|item| self.convert_json_value_to_position(item))
                .collect();
        } else if json_payload.is_object() {
            response
                .data
                .push(self.convert_json_value_to_position(&json_payload));
        }

        crate::exchange_client::common::Response::GetPosition(response)
    }

    fn create_get_balance_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetBalanceResponse::default();

        if let Some(balances) = json_payload.get("balances").and_then(|v| v.as_object()) {
            response.data = balances
                .iter()
                .map(|(currency, balance_json)| {
                    self.convert_balance_entry_to_balance(currency, balance_json)
                })
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

        new_http_response.error_code = json_payload
            .get("label")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        new_http_response.error_message = json_payload
            .get("message")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
