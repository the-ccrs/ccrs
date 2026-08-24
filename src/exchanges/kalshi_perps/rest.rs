#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest
    for crate::exchanges::kalshi_perps::common::KalshiPerpsClient
{
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params = std::collections::HashMap::new();

        if !get_instrument_info_request.status.is_empty() {
            query_params.insert(
                "status".to_string(),
                get_instrument_info_request.status.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/trade-api/v2/margin/markets",
            None,
            Some(query_params),
            None,
        )
    }

    fn create_get_top_of_book_http_request(
        &self,
        _get_top_of_book_request: &crate::exchange_client::common::GetTopOfBookRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/trade-api/v2/margin/markets",
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
        let timestamp = now.timestamp_millis().to_string();
        let base_url_path = url::Url::parse(&http_request.base_url)
            .map(|url| url.path().trim_end_matches('/').to_string())
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
        let mut body = serde_json::Map::new();

        body.insert(
            "ticker".to_string(),
            serde_json::json!(place_order_request.symbol),
        );
        body.insert(
            "client_order_id".to_string(),
            serde_json::json!(place_order_request.client_order_id),
        );
        body.insert(
            "side".to_string(),
            serde_json::json!(self.convert_order_side_to_string(place_order_request.side)),
        );
        body.insert(
            "count".to_string(),
            serde_json::json!(place_order_request.quantity),
        );
        body.insert(
            "price".to_string(),
            serde_json::json!(place_order_request.price),
        );
        body.insert(
            "time_in_force".to_string(),
            serde_json::json!(
                if place_order_request.order_type == crate::types::OrderType::Market {
                    "fill_or_kill"
                } else {
                    "good_till_canceled"
                }
            ),
        );
        body.insert(
            "self_trade_prevention_type".to_string(),
            serde_json::json!("taker_at_cross"),
        );

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/trade-api/v2/margin/orders",
            None,
            None,
            Some(serde_json::Value::Object(body)),
        )
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let path = format!(
            "/trade-api/v2/margin/orders/{}",
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
        let mut query_params = std::collections::HashMap::new();

        if !get_open_order_request.symbol.is_empty() {
            query_params.insert("ticker".to_string(), get_open_order_request.symbol.clone());
        }
        query_params.insert("status".to_string(), "resting".to_string());
        if get_open_order_request.limit > 0 {
            query_params.insert(
                "limit".to_string(),
                get_open_order_request.limit.to_string(),
            );
        }
        if !get_open_order_request.next_page_cursor.is_empty() {
            query_params.insert(
                "cursor".to_string(),
                get_open_order_request.next_page_cursor.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/trade-api/v2/margin/orders",
            None,
            Some(query_params),
            None,
        )
    }

    fn create_get_position_http_request(
        &self,
        get_position_request: &crate::exchange_client::common::GetPositionRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params = std::collections::HashMap::new();

        if !get_position_request.symbol.is_empty() {
            query_params.insert("ticker".to_string(), get_position_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/trade-api/v2/margin/positions",
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
            "/trade-api/v2/margin/balance",
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
        let Some(markets) = json_payload
            .get("markets")
            .and_then(serde_json::Value::as_array)
        else {
            return crate::exchange_client::common::Response::GetInstrumentInfo(response);
        };

        response.data.reserve(markets.len());
        for market in markets {
            response.data.push(crate::types::InstrumentInfo {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::KalshiPerps,
                symbol: market["ticker"].as_str().unwrap_or("").to_string(),
                quote_asset: "USD".to_string(),
                order_price_increment: market["tick_size"].as_str().unwrap_or("").to_string(),
                contract_size: market["contract_size"].as_str().unwrap_or("").to_string(),
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
        let Some(markets) = json_payload
            .get("markets")
            .and_then(serde_json::Value::as_array)
        else {
            return crate::exchange_client::common::Response::GetTopOfBook(response);
        };

        response.data.reserve(markets.len());
        for market in markets {
            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::KalshiPerps,
                symbol: market["ticker"].as_str().unwrap_or("").to_string(),
                timestamp: chrono::DateTime::default(),
                bid_price: market["bid"].as_str().unwrap_or("").to_string(),
                bid_size: String::new(),
                ask_price: market["ask"].as_str().unwrap_or("").to_string(),
                ask_size: String::new(),
            });
        }

        crate::exchange_client::common::Response::GetTopOfBook(response)
    }

    fn create_place_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();
        crate::exchange_client::common::Response::PlaceOrder(
            crate::exchange_client::common::PlaceOrderResponse {
                order_id: json_payload["order_id"].as_str().unwrap_or("").to_string(),
            },
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

        let next_page_cursor = json_payload["cursor"].as_str().unwrap_or("").to_string();

        let data = json_payload
            .get("orders")
            .and_then(serde_json::Value::as_array)
            .map(|orders| {
                orders
                    .iter()
                    .map(|order| self.convert_json_value_to_order(order))
                    .collect()
            })
            .unwrap_or_default();

        let response = crate::exchange_client::common::GetOpenOrderResponse {
            next_page_cursor,
            data,
        };

        crate::exchange_client::common::Response::GetOpenOrder(response)
    }

    fn create_get_position_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();
        let mut response = crate::exchange_client::common::GetPositionResponse::default();

        if let Some(positions) = json_payload
            .get("positions")
            .and_then(serde_json::Value::as_array)
        {
            response.data = positions
                .iter()
                .map(|position| self.convert_json_value_to_position(position))
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
        let Some(json_payload) = http_response.json_payload.as_ref() else {
            return crate::exchange_client::common::Response::HttpErrorResponse(
                http_response.clone(),
            );
        };
        let mut response = http_response.clone();
        response.error_code = json_payload["code"].as_str().map(str::to_string);
        response.error_message = json_payload["message"].as_str().map(str::to_string);
        crate::exchange_client::common::Response::HttpErrorResponse(response)
    }
}
