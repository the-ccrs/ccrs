#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest for crate::exchanges::kalshi::common::KalshiClient {
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if !get_instrument_info_request.symbol.is_empty() {
            query_params.insert("tickers".into(), get_instrument_info_request.symbol.clone());
        }

        if get_instrument_info_request.limit > 0 {
            query_params.insert(
                "limit".into(),
                get_instrument_info_request.limit.to_string(),
            );
        }

        if !get_instrument_info_request.next_page_cursor.is_empty() {
            query_params.insert(
                "cursor".into(),
                get_instrument_info_request.next_page_cursor.clone(),
            );
        }

        if !get_instrument_info_request.status.is_empty() {
            query_params.insert("status".into(), get_instrument_info_request.status.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/trade-api/v2/markets",
            None,
            Some(query_params),
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
            query_params.insert("tickers".into(), get_top_of_book_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/trade-api/v2/markets",
            None,
            Some(query_params),
            None,
        )
    }

    fn sign_http_request(
        &self,
        http_request: &mut crate::networking::http::HttpRequest,
        now: chrono::DateTime<chrono::Utc>,
    ) {
        let credential = self.credential.as_ref().unwrap();

        let timestamp = now.timestamp_millis().to_string();

        let base_url_path = url::Url::parse(&http_request.base_url)
            .map(|u| u.path().trim_end_matches('/').to_string())
            .unwrap_or_default();

        let sign_path = format!("{}{}", base_url_path, http_request.path);

        let signature = self.build_signature(&timestamp, http_request.method.as_str(), &sign_path);

        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);

        headers.insert(
            reqwest::header::HeaderName::from_static("kalshi-access-key"),
            reqwest::header::HeaderValue::from_str(&credential.api_key).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("kalshi-access-timestamp"),
            reqwest::header::HeaderValue::from_str(&timestamp).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("kalshi-access-signature"),
            reqwest::header::HeaderValue::from_str(&signature).unwrap(),
        );
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        body_map.insert(
            "ticker".into(),
            serde_json::json!(place_order_request.symbol),
        );
        body_map.insert(
            "side".into(),
            serde_json::json!(self.convert_order_side_to_string(place_order_request.side)),
        );
        body_map.insert(
            "count".into(),
            serde_json::json!(place_order_request.quantity),
        );
        body_map.insert("price".into(), serde_json::json!(place_order_request.price));
        body_map.insert(
            "time_in_force".into(),
            serde_json::json!(
                if place_order_request.order_type == crate::types::OrderType::Market {
                    "fill_or_kill"
                } else {
                    "good_till_canceled"
                }
            ),
        );
        body_map.insert(
            "self_trade_prevention_type".into(),
            serde_json::json!("taker_at_cross"),
        );

        if !place_order_request.client_order_id.is_empty() {
            body_map.insert(
                "client_order_id".into(),
                serde_json::json!(place_order_request.client_order_id),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/trade-api/v2/portfolio/events/orders",
            None,
            None,
            Some(body_value),
        )
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let path = format!(
            "/trade-api/v2/portfolio/events/orders/{}",
            cancel_order_request.order_id
        );

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
            query_params.insert("ticker".into(), get_open_order_request.symbol.clone());
        }

        query_params.insert("status".into(), "resting".into());

        if get_open_order_request.limit > 0 {
            query_params.insert("limit".into(), get_open_order_request.limit.to_string());
        }

        if !get_open_order_request.next_page_cursor.is_empty() {
            query_params.insert(
                "cursor".into(),
                get_open_order_request.next_page_cursor.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/trade-api/v2/portfolio/orders",
            None,
            Some(query_params),
            None,
        )
    }

    fn create_get_position_http_request(
        &self,
        get_position_request: &crate::exchange_client::common::GetPositionRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if !get_position_request.symbol.is_empty() {
            query_params.insert("ticker".into(), get_position_request.symbol.clone());
        }

        if get_position_request.limit > 0 {
            query_params.insert("limit".into(), get_position_request.limit.to_string());
        }

        if !get_position_request.next_page_cursor.is_empty() {
            query_params.insert(
                "cursor".into(),
                get_position_request.next_page_cursor.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/trade-api/v2/portfolio/positions",
            None,
            Some(query_params),
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
            "/trade-api/v2/portfolio/balance",
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

        let markets_array = match json_payload.get("markets").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetInstrumentInfo(response),
        };

        if let Some(cursor) = json_payload.get("cursor").and_then(|v| v.as_str())
            && !cursor.is_empty()
        {
            response.next_page_cursor = cursor.to_string();
        }

        response.data.reserve(markets_array.len());

        for item in markets_array {
            let order_price_increment = item
                .get("price_ranges")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|first| first.get("step"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or("0.01")
                .to_string();

            let expiry_timestamp = item
                .get("close_time")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_default();

            response.data.push(crate::types::InstrumentInfo {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Kalshi,
                symbol: item["ticker"].as_str().unwrap_or("").to_string(),
                base_asset: String::new(),
                quote_asset: "USD".to_string(),
                order_price_increment,
                order_quantity_increment: "1".to_string(),
                order_quantity_min: "1".to_string(),
                order_quantity_max: String::new(),
                order_quote_quantity_min: "0".to_string(),
                order_quote_quantity_max: String::new(),
                expiry_timestamp,
                ..Default::default()
            });
        }

        crate::exchange_client::common::Response::GetInstrumentInfo(response)
    }

    fn create_get_top_of_book_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetTopOfBookResponse::default();

        let markets_array = match json_payload.get("markets").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetTopOfBook(response),
        };

        response.data.reserve(markets_array.len());

        for item in markets_array {
            let timestamp = item
                .get("updated_time")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_default();

            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Kalshi,
                symbol: item["ticker"].as_str().unwrap_or("").to_string(),
                timestamp,
                bid_price: item["yes_bid_dollars"].as_str().unwrap_or("").to_string(),
                bid_size: item["yes_bid_size_fp"].as_str().unwrap_or("").to_string(),
                ask_price: item["yes_ask_dollars"].as_str().unwrap_or("").to_string(),
                ask_size: item["yes_ask_size_fp"].as_str().unwrap_or("").to_string(),
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
            order_id: json_payload["order_id"].as_str().unwrap_or("").to_string(),
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

        if let Some(cursor) = json_payload.get("cursor").and_then(|v| v.as_str())
            && !cursor.is_empty()
        {
            response.next_page_cursor = cursor.to_string();
        }

        if let Some(list) = json_payload.get("orders").and_then(|v| v.as_array()) {
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

        if let Some(cursor) = json_payload.get("cursor").and_then(|v| v.as_str())
            && !cursor.is_empty()
        {
            response.next_page_cursor = cursor.to_string();
        }

        if let Some(list) = json_payload
            .get("market_positions")
            .and_then(|v| v.as_array())
        {
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

        response
            .data
            .push(self.convert_json_value_to_balance(&json_payload));

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
            .get("code")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        new_http_response.error_message = json_payload
            .get("message")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
