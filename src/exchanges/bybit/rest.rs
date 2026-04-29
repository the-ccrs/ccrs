#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest for crate::exchanges::bybit::common::BybitClient {
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        query_params.insert("category".into(), self.category.clone());

        query_params.insert("symbol".into(), get_instrument_info_request.symbol.clone());

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

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/v5/market/instruments-info",
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

        query_params.insert("category".into(), self.category.clone());

        query_params.insert("symbol".into(), get_top_of_book_request.symbol.clone());

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/v5/market/tickers",
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

        let params_str = if http_request.method == reqwest::Method::POST {
            if let Some(ref payload) = http_request.payload {
                payload.clone()
            } else {
                String::new()
            }
        } else if http_request.method == reqwest::Method::GET {
            if let Some(ref query_string) = http_request.query_string {
                query_string.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let prehash = format!(
            "{}{}{}{}",
            timestamp, credential.api_key, self.api_receive_window_milliseconds, params_str
        );

        let mut mac = hmac_sha256::HMAC::new(credential.api_secret.as_bytes());
        mac.update(prehash.as_bytes());
        let signature_bytes = mac.finalize();
        let signature_hex = hex::encode(signature_bytes);

        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);

        headers.insert(
            reqwest::header::HeaderName::from_static("x-bapi-api-key"),
            reqwest::header::HeaderValue::from_str(&credential.api_key).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("x-bapi-timestamp"),
            reqwest::header::HeaderValue::from_str(&timestamp).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("x-bapi-recv-window"),
            reqwest::header::HeaderValue::from_str(
                &self.api_receive_window_milliseconds.to_string(),
            )
            .unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("x-referer"),
            reqwest::header::HeaderValue::from_str(&self.api_broker_id).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("x-bapi-sign"),
            reqwest::header::HeaderValue::from_str(&signature_hex).unwrap(),
        );
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        body_map.insert("category".into(), serde_json::json!(self.category));
        body_map.insert(
            "symbol".into(),
            serde_json::json!(place_order_request.symbol),
        );
        body_map.insert(
            "side".into(),
            serde_json::json!(self.convert_order_side_to_string(place_order_request.side)),
        );
        body_map.insert(
            "orderType".into(),
            serde_json::json!(self.convert_order_type_to_string(place_order_request.order_type)),
        );
        body_map.insert(
            "qty".into(),
            serde_json::json!(place_order_request.quantity),
        );

        if place_order_request.order_type == crate::types::OrderType::Limit {
            body_map.insert("price".into(), serde_json::json!(place_order_request.price));
        }

        if !place_order_request.client_order_id.is_empty() {
            body_map.insert(
                "orderLinkId".into(),
                serde_json::json!(place_order_request.client_order_id),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/v5/order/create",
            None,
            None,
            Some(body_value),
        )
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        body_map.insert("category".into(), serde_json::json!(self.category));
        body_map.insert(
            "symbol".into(),
            serde_json::json!(cancel_order_request.symbol),
        );

        if !cancel_order_request.order_id.is_empty() {
            body_map.insert(
                "orderId".into(),
                serde_json::json!(cancel_order_request.order_id),
            );
        } else if !cancel_order_request.client_order_id.is_empty() {
            body_map.insert(
                "orderLinkId".into(),
                serde_json::json!(cancel_order_request.client_order_id),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/v5/order/cancel",
            None,
            None,
            Some(body_value),
        )
    }

    fn create_get_open_order_http_request(
        &self,
        get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        query_params.insert("category".into(), self.category.clone());

        if !get_open_order_request.symbol.is_empty() {
            query_params.insert("symbol".into(), get_open_order_request.symbol.clone());
        }

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
            "/v5/order/realtime",
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

        query_params.insert("category".into(), self.category.clone());

        if !get_position_request.symbol.is_empty() {
            query_params.insert("symbol".into(), get_position_request.symbol.clone());
        }

        if !get_position_request.settle_asset.is_empty() {
            query_params.insert(
                "settleCoin".into(),
                get_position_request.settle_asset.clone(),
            );
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
            "/v5/position/list",
            None,
            Some(query_params),
            None,
        )
    }

    fn create_get_balance_http_request(
        &self,
        get_balance_request: &crate::exchange_client::common::GetBalanceRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        query_params.insert("accountType".into(), "UNIFIED".into());

        if !get_balance_request.asset.is_empty() {
            query_params.insert("coin".into(), get_balance_request.asset.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/v5/account/wallet-balance",
            None,
            Some(query_params),
            None,
        )
    }

    fn is_http_response_success(
        &self,
        http_response: &crate::networking::http::HttpResponse,
    ) -> bool {
        if !http_response.status.is_success() {
            return false;
        }

        let Some(json) = &http_response.json_payload else {
            return false;
        };

        matches!(json.get("retCode"), Some(v) if v == 0)
    }

    fn create_get_instrument_info_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetInstrumentInfoResponse::default();

        let result = &json_payload["result"];
        let data_array = match result.get("list").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetInstrumentInfo(response),
        };

        response.data.reserve(data_array.len());

        match self.instrument_type {
            crate::types::BybitInstrumentType::Spot => {
                for item in data_array {
                    response.data.push(crate::types::InstrumentInfo {
                        exchange_instrument_type: crate::types::ExchangeInstrumentType::Bybit(
                            self.instrument_type,
                        ),
                        symbol: item["symbol"].as_str().unwrap().to_string(),
                        base_asset: item["baseCoin"].as_str().unwrap().to_string(),
                        quote_asset: item["quoteCoin"].as_str().unwrap().to_string(),

                        order_price_increment: item["priceFilter"]["tickSize"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        order_quantity_increment: item["lotSizeFilter"]["basePrecision"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        order_quantity_min: item["lotSizeFilter"]["minOrderQty"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        order_quantity_max: item["lotSizeFilter"]["maxOrderQty"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        order_quote_quantity_min: item["lotSizeFilter"]["minOrderAmt"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        order_quote_quantity_max: item["lotSizeFilter"]["maxOrderAmt"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        ..Default::default()
                    });
                }
            }

            crate::types::BybitInstrumentType::Linear
            | crate::types::BybitInstrumentType::Inverse => {
                for item in data_array {
                    response.data.push(crate::types::InstrumentInfo {
                        exchange_instrument_type: crate::types::ExchangeInstrumentType::Bybit(
                            self.instrument_type,
                        ),
                        symbol: item["symbol"].as_str().unwrap().to_string(),
                        base_asset: item["baseCoin"].as_str().unwrap().to_string(),
                        quote_asset: item["quoteCoin"].as_str().unwrap().to_string(),

                        order_price_increment: item["priceFilter"]["tickSize"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        order_quantity_increment: item["lotSizeFilter"]["qtyStep"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        order_quantity_min: item["lotSizeFilter"]["minOrderQty"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        order_quantity_max: item["lotSizeFilter"]["maxOrderQty"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        order_quote_quantity_min: item["lotSizeFilter"]["minNotionalValue"]
                            .as_str()
                            .unwrap()
                            .to_string(),

                        settle_asset: item["settleCoin"].as_str().unwrap().to_string(),

                        expiry_timestamp:
                            crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
                                item["deliveryTime"]
                                    .as_str()
                                    .unwrap()
                                    .parse::<i64>()
                                    .unwrap(),
                            ),

                        ..Default::default()
                    });
                }

                response.next_page_cursor = result["nextPageCursor"].as_str().unwrap().to_string();
            }

            _ => panic!(),
        }

        crate::exchange_client::common::Response::GetInstrumentInfo(response)
    }

    fn create_get_top_of_book_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetTopOfBookResponse::default();

        let result = &json_payload["result"];
        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
            json_payload["time"].as_i64().unwrap(),
        );
        let data_array = match result.get("list").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetTopOfBook(response),
        };

        response.data.reserve(data_array.len());

        for item in data_array {
            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Bybit(
                    self.instrument_type,
                ),
                symbol: item["symbol"].as_str().unwrap().to_string(),
                timestamp,
                bid_price: item["bid1Price"].as_str().unwrap().to_string(),
                bid_size: item["bid1Size"].as_str().unwrap().to_string(),
                ask_price: item["ask1Price"].as_str().unwrap().to_string(),
                ask_size: item["ask1Size"].as_str().unwrap().to_string(),
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
            order_id: json_payload["result"]["orderId"]
                .as_str()
                .unwrap()
                .to_string(),
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

        let result = &json_payload["result"];

        response.next_page_cursor = result["nextPageCursor"].as_str().unwrap().to_string();

        if let Some(list) = result["list"].as_array() {
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

        let result = &json_payload["result"];

        response.next_page_cursor = result["nextPageCursor"].as_str().unwrap().to_string();

        if let Some(list) = result["list"].as_array() {
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

        let coins = json_payload["result"]["list"]
            .as_array()
            .unwrap()
            .first()
            .unwrap()["coin"]
            .as_array()
            .unwrap();

        response.data = coins
            .iter()
            .map(|item| self.convert_json_value_to_balance(item))
            .collect();

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
            .get("retCode")
            .and_then(|v| v.as_i64())
            .map(|v| v.to_string());

        new_http_response.error_message = json_payload
            .get("retMsg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
