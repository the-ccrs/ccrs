#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest
    for crate::exchanges::gateio_spot_and_margin::common::GateioSpotAndMarginClient
{
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let path = if !get_instrument_info_request.symbol.is_empty() {
            format!(
                "{}/spot/currency_pairs/{}",
                Self::REST_API_PREFIX,
                get_instrument_info_request.symbol
            )
        } else {
            format!("{}/spot/currency_pairs", Self::REST_API_PREFIX)
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
            query_params.insert(
                "currency_pair".into(),
                get_top_of_book_request.symbol.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &format!("{}/spot/tickers", Self::REST_API_PREFIX),
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
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        body_map.insert(
            "currency_pair".into(),
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
            "amount".into(),
            serde_json::json!(place_order_request.quantity),
        );
        body_map.insert("account".into(), serde_json::json!(self.account));

        match place_order_request.order_type {
            crate::types::OrderType::Limit => {
                body_map.insert("price".into(), serde_json::json!(place_order_request.price));
                body_map.insert("time_in_force".into(), serde_json::json!("gtc"));
            }
            crate::types::OrderType::Market => {
                body_map.insert("time_in_force".into(), serde_json::json!("ioc"));
            }
            crate::types::OrderType::Unknown => {
                panic!()
            }
        }

        if !place_order_request.client_order_id.is_empty() {
            body_map.insert(
                "text".into(),
                serde_json::json!(place_order_request.client_order_id.clone()),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-gate-channel-id", self.api_channel_id.parse().unwrap());

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            &format!("{}/spot/orders", Self::REST_API_PREFIX),
            Some(headers),
            None,
            Some(body_value),
        )
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        query_params.insert("currency_pair".into(), cancel_order_request.symbol.clone());

        let order_id = if !cancel_order_request.order_id.is_empty() {
            cancel_order_request.order_id.clone()
        } else {
            cancel_order_request.client_order_id.clone()
        };

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::DELETE,
            &format!("{}/spot/orders/{}", Self::REST_API_PREFIX, order_id),
            None,
            Some(query_params),
            None,
        )
    }

    fn create_get_open_order_http_request(
        &self,
        get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if get_open_order_request.limit > 0 {
            query_params.insert("limit".into(), get_open_order_request.limit.to_string());
        }

        if !get_open_order_request.next_page_cursor.is_empty()
            && let Ok(page) = get_open_order_request.next_page_cursor.parse::<u32>()
        {
            query_params.insert("page".into(), page.to_string());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &format!("{}/spot/open_orders", Self::REST_API_PREFIX),
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
            &format!("{}/spot/accounts", Self::REST_API_PREFIX),
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
            &format!("{}/spot/accounts", Self::REST_API_PREFIX),
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
                exchange_instrument_type: crate::types::ExchangeInstrumentType::GateioSpotAndMargin(
                    self.instrument_type,
                ),
                symbol: item["currency_pair"].as_str().unwrap_or("").to_string(),
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

        let response = crate::exchange_client::common::PlaceOrderResponse {
            order_id: json_payload["id"].as_str().unwrap_or("").to_string(),
        };

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

        if let Some(list) = json_payload.as_array() {
            response.data = list
                .iter()
                .flat_map(|item| {
                    item["orders"]
                        .as_array()
                        .map(|orders| {
                            orders
                                .iter()
                                .map(|order| self.convert_json_value_to_order(order))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                })
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

        if let Some(accounts) = json_payload.as_array() {
            response.data = accounts
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
