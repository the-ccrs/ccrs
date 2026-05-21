#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest for crate::exchanges::bitget::common::BitgetClient {
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        query_params.insert("category".into(), self.category.clone());

        if !get_instrument_info_request.symbol.is_empty() {
            query_params.insert("symbol".into(), get_instrument_info_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/api/v3/market/instruments",
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
            "/api/v3/market/tickers",
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

        let method_str = http_request.method.as_str();

        let path_with_query = if http_request.method == reqwest::Method::GET {
            if let Some(ref qs) = http_request.query_string {
                format!("{}?{}", http_request.path, qs)
            } else {
                http_request.path.clone()
            }
        } else {
            http_request.path.clone()
        };

        let body = if http_request.method == reqwest::Method::POST {
            if let Some(ref payload) = http_request.payload {
                payload.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let prehash = format!("{}{}{}{}", timestamp, method_str, path_with_query, body);

        let mut mac = hmac_sha256::HMAC::new(credential.api_secret.as_bytes());
        mac.update(prehash.as_bytes());
        let signature_bytes = mac.finalize();
        let signature_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature_bytes);

        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);

        headers.insert(
            reqwest::header::HeaderName::from_static("content-type"),
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("access-key"),
            reqwest::header::HeaderValue::from_str(&credential.api_key).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("access-sign"),
            reqwest::header::HeaderValue::from_str(&signature_b64).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("access-timestamp"),
            reqwest::header::HeaderValue::from_str(&timestamp).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("access-passphrase"),
            reqwest::header::HeaderValue::from_str(&credential.passphrase).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("x-channel-api-code"),
            reqwest::header::HeaderValue::from_str(&self.api_channel_api_code).unwrap(),
        );

        if self.use_demo_trading {
            headers.insert(
                reqwest::header::HeaderName::from_static("paptrading"),
                reqwest::header::HeaderValue::from_static("1"),
            );
        }
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut body_map = serde_json::Map::new();

        body_map.insert(
            "symbol".into(),
            serde_json::json!(place_order_request.symbol),
        );
        body_map.insert("category".into(), serde_json::json!(self.category));
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
                "clientOid".into(),
                serde_json::json!(place_order_request.client_order_id),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/api/v3/trade/place-order",
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

        body_map.insert(
            "symbol".into(),
            serde_json::json!(cancel_order_request.symbol),
        );
        body_map.insert("category".into(), serde_json::json!(self.category));

        if !cancel_order_request.order_id.is_empty() {
            body_map.insert(
                "orderId".into(),
                serde_json::json!(cancel_order_request.order_id),
            );
        } else if !cancel_order_request.client_order_id.is_empty() {
            body_map.insert(
                "clientOid".into(),
                serde_json::json!(cancel_order_request.client_order_id),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/api/v3/trade/cancel-order",
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
            "/api/v3/trade/unfilled-orders",
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

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/api/v3/position/current-position",
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

        if !get_balance_request.asset.is_empty() {
            query_params.insert("coin".into(), get_balance_request.asset.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/api/v3/account/assets",
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

        matches!(json.get("code"), Some(v) if v == "00000")
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

        match self.instrument_type {
            crate::types::BitgetInstrumentType::Spot => {
                for item in data_array {
                    let price_precision: usize = item["pricePrecision"]
                        .as_str()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                    let qty_precision: usize = item["quantityPrecision"]
                        .as_str()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);

                    let order_price_increment = format!(
                        "{:.prec$}",
                        10f64.powi(-(price_precision as i32)),
                        prec = price_precision
                    );
                    let order_quantity_increment = format!(
                        "{:.prec$}",
                        10f64.powi(-(qty_precision as i32)),
                        prec = qty_precision
                    );

                    response.data.push(crate::types::InstrumentInfo {
                        exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitget(
                            self.instrument_type,
                        ),
                        symbol: item["symbol"].as_str().unwrap_or("").to_string(),
                        base_asset: item["baseCoin"].as_str().unwrap_or("").to_string(),
                        quote_asset: item["quoteCoin"].as_str().unwrap_or("").to_string(),
                        order_price_increment,
                        order_quantity_increment,
                        order_quantity_min: item["minOrderQty"].as_str().unwrap_or("").to_string(),
                        order_quantity_max: item["maxOrderQty"].as_str().unwrap_or("").to_string(),
                        order_quote_quantity_min: item["minOrderAmount"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        ..Default::default()
                    });
                }
            }

            crate::types::BitgetInstrumentType::UsdtFutures
            | crate::types::BitgetInstrumentType::CoinFutures => {
                for item in data_array {
                    let price_precision: usize = item["pricePrecision"]
                        .as_str()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                    let price_multiplier: f64 = item["priceMultiplier"]
                        .as_str()
                        .unwrap_or("1")
                        .parse()
                        .unwrap_or(1.0);

                    let order_price_increment =
                        format!("{:.prec$}", price_multiplier, prec = price_precision);

                    let delivery_time_str = item["deliveryTime"].as_str().unwrap_or("0");
                    let expiry_timestamp = if delivery_time_str.is_empty()
                        || delivery_time_str == "0"
                        || delivery_time_str == "-1"
                    {
                        chrono::DateTime::<chrono::Utc>::default()
                    } else {
                        crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
                            delivery_time_str.parse::<i64>().unwrap_or(0),
                        )
                    };

                    response.data.push(crate::types::InstrumentInfo {
                        exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitget(
                            self.instrument_type,
                        ),
                        symbol: item["symbol"].as_str().unwrap_or("").to_string(),
                        base_asset: item["baseCoin"].as_str().unwrap_or("").to_string(),
                        quote_asset: item["quoteCoin"].as_str().unwrap_or("").to_string(),
                        order_price_increment,
                        order_quantity_increment: item["quantityMultiplier"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        order_quantity_min: item["minOrderQty"].as_str().unwrap_or("").to_string(),
                        order_quantity_max: item["maxOrderQty"].as_str().unwrap_or("").to_string(),
                        order_quote_quantity_min: item["minOrderAmount"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        settle_asset: item["quoteCoin"].as_str().unwrap_or("").to_string(),
                        expiry_timestamp,
                        ..Default::default()
                    });
                }

                if let Some(cursor) = json_payload
                    .get("data")
                    .and_then(|d| d.as_object())
                    .and_then(|o| o.get("nextCursor"))
                    .and_then(|v| v.as_str())
                {
                    response.next_page_cursor = cursor.to_string();
                }
            }

            crate::types::BitgetInstrumentType::Unknown => panic!(),
        }

        crate::exchange_client::common::Response::GetInstrumentInfo(response)
    }

    fn create_get_top_of_book_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetTopOfBookResponse::default();

        let data_array = match json_payload.get("data").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetTopOfBook(response),
        };

        response.data.reserve(data_array.len());

        for item in data_array {
            let ts_str = item["ts"].as_str().unwrap_or("0");
            let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
                ts_str.parse::<i64>().unwrap_or(0),
            );

            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitget(
                    self.instrument_type,
                ),
                symbol: item["symbol"].as_str().unwrap_or("").to_string(),
                timestamp,
                bid_price: item["bid1Price"].as_str().unwrap_or("").to_string(),
                bid_size: item["bid1Size"].as_str().unwrap_or("").to_string(),
                ask_price: item["ask1Price"].as_str().unwrap_or("").to_string(),
                ask_size: item["ask1Size"].as_str().unwrap_or("").to_string(),
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
            order_id: json_payload["data"]["orderId"]
                .as_str()
                .unwrap_or("")
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

        if let Some(list) = json_payload["data"]["list"].as_array() {
            response.data = list
                .iter()
                .map(|item| self.convert_json_value_to_order(item))
                .collect();
        }

        if let Some(cursor) = json_payload["data"]["cursor"].as_str() {
            response.next_page_cursor = cursor.to_string();
        }

        crate::exchange_client::common::Response::GetOpenOrder(response)
    }

    fn create_get_position_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetPositionResponse::default();

        if let Some(list) = json_payload["data"]["list"].as_array() {
            response.data = list
                .iter()
                .map(|item| self.convert_json_value_to_position(item))
                .collect();
        }

        if let Some(cursor) = json_payload.get("nextCursor").and_then(|v| v.as_str()) {
            response.next_page_cursor = cursor.to_string();
        }

        crate::exchange_client::common::Response::GetPosition(response)
    }

    fn create_get_balance_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetBalanceResponse::default();

        let coins = if let Some(arr) = json_payload["data"].as_array() {
            arr.iter()
                .map(|item| self.convert_json_value_to_balance(item))
                .collect()
        } else if let Some(arr) = json_payload["data"]["assets"].as_array() {
            arr.iter()
                .map(|item| self.convert_json_value_to_balance(item))
                .collect()
        } else {
            Vec::new()
        };

        response.data = coins;

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
            .get("msg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
