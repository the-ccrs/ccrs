#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest for crate::exchanges::okx::common::OkxClient {
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        query_params.insert("instType".into(), self.inst_type_str.clone());

        if !get_instrument_info_request.symbol.is_empty() {
            query_params.insert("instId".into(), get_instrument_info_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/api/v5/public/instruments",
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

        query_params.insert("instType".into(), self.inst_type_str.clone());

        if !get_top_of_book_request.symbol.is_empty() {
            query_params.insert("instId".into(), get_top_of_book_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/api/v5/market/tickers",
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

        let timestamp = now.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        let path_with_query = if let Some(ref qs) = http_request.query_string {
            format!("{}?{}", http_request.path, qs)
        } else {
            http_request.path.clone()
        };

        let body = if http_request.method == reqwest::Method::POST {
            if let Some(ref payload) = http_request.payload {
                payload.as_str()
            } else {
                ""
            }
        } else {
            ""
        };

        let prehash = format!(
            "{}{}{}{}",
            timestamp,
            http_request.method.as_str(),
            path_with_query,
            body
        );

        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .unwrap();

        hmac::Mac::update(&mut mac, prehash.as_bytes());
        let signature_bytes = hmac::Mac::finalize(mac).into_bytes();
        let signature =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature_bytes);

        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);

        headers.insert(
            reqwest::header::HeaderName::from_static("ok-access-key"),
            reqwest::header::HeaderValue::from_str(&credential.api_key).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("ok-access-sign"),
            reqwest::header::HeaderValue::from_str(&signature).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("ok-access-timestamp"),
            reqwest::header::HeaderValue::from_str(&timestamp).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("ok-access-passphrase"),
            reqwest::header::HeaderValue::from_str(&credential.passphrase).unwrap(),
        );

        if self.use_demo_trading {
            headers.insert(
                reqwest::header::HeaderName::from_static("x-simulated-trading"),
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
            "instId".into(),
            serde_json::json!(place_order_request.symbol),
        );
        body_map.insert("tdMode".into(), serde_json::json!(self.td_mode));
        body_map.insert(
            "side".into(),
            serde_json::json!(self.convert_order_side_to_string(place_order_request.side)),
        );
        body_map.insert(
            "ordType".into(),
            serde_json::json!(self.convert_order_type_to_string(place_order_request.order_type)),
        );
        body_map.insert("sz".into(), serde_json::json!(place_order_request.quantity));

        if place_order_request.order_type == crate::types::OrderType::Limit {
            body_map.insert("px".into(), serde_json::json!(place_order_request.price));
        }

        if !place_order_request.client_order_id.is_empty() {
            body_map.insert(
                "clOrdId".into(),
                serde_json::json!(place_order_request.client_order_id),
            );
        }

        body_map.insert("tag".into(), serde_json::json!(self.api_broker_code));

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/api/v5/trade/order",
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
            "instId".into(),
            serde_json::json!(cancel_order_request.symbol),
        );

        if !cancel_order_request.order_id.is_empty() {
            body_map.insert(
                "ordId".into(),
                serde_json::json!(cancel_order_request.order_id),
            );
        } else if !cancel_order_request.client_order_id.is_empty() {
            body_map.insert(
                "clOrdId".into(),
                serde_json::json!(cancel_order_request.client_order_id),
            );
        }

        let body_value = serde_json::Value::Object(body_map);

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/api/v5/trade/cancel-order",
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

        query_params.insert("instType".into(), self.inst_type_str.clone());

        if !get_open_order_request.symbol.is_empty() {
            query_params.insert("instId".into(), get_open_order_request.symbol.clone());
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
            "/api/v5/trade/orders-pending",
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

        query_params.insert("instType".into(), self.inst_type_str.clone());

        if !get_position_request.symbol.is_empty() {
            query_params.insert("instId".into(), get_position_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/api/v5/account/positions",
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
            query_params.insert("ccy".into(), get_balance_request.asset.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/api/v5/account/balance",
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

        matches!(json.get("code"), Some(v) if v == "0")
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
            crate::types::OkxInstrumentType::Spot | crate::types::OkxInstrumentType::Margin => {
                for item in data_array {
                    let order_quote_quantity_max =
                        if let Some(s) = item.get("maxLmtAmt").and_then(|v| v.as_str()) {
                            s.to_string()
                        } else {
                            String::new()
                        };

                    response.data.push(crate::types::InstrumentInfo {
                        exchange_instrument_type: crate::types::ExchangeInstrumentType::Okx(
                            self.instrument_type,
                        ),
                        symbol: item["instId"].as_str().unwrap().to_string(),
                        base_asset: item["baseCcy"].as_str().unwrap().to_string(),
                        quote_asset: item["quoteCcy"].as_str().unwrap().to_string(),

                        order_price_increment: item["tickSz"].as_str().unwrap().to_string(),

                        order_quantity_increment: item["lotSz"].as_str().unwrap().to_string(),

                        order_quantity_min: item["minSz"].as_str().unwrap().to_string(),

                        order_quantity_max: item["maxLmtSz"].as_str().unwrap().to_string(),

                        order_quote_quantity_min: "0".to_string(),

                        order_quote_quantity_max,

                        ..Default::default()
                    });
                }
            }

            crate::types::OkxInstrumentType::Swap
            | crate::types::OkxInstrumentType::Futures
            | crate::types::OkxInstrumentType::Option => {
                for item in data_array {
                    let base_asset = if let Some(s) = item.get("baseCcy").and_then(|v| v.as_str()) {
                        s.to_string()
                    } else {
                        String::new()
                    };

                    let quote_asset = if let Some(s) = item.get("quoteCcy").and_then(|v| v.as_str())
                    {
                        s.to_string()
                    } else {
                        String::new()
                    };

                    let settle_asset =
                        if let Some(s) = item.get("settleCcy").and_then(|v| v.as_str()) {
                            s.to_string()
                        } else {
                            String::new()
                        };

                    let contract_size = if let Some(s) = item.get("ctVal").and_then(|v| v.as_str())
                    {
                        s.to_string()
                    } else {
                        String::new()
                    };

                    let contract_multiplier =
                        if let Some(s) = item.get("ctMult").and_then(|v| v.as_str()) {
                            s.to_string()
                        } else {
                            String::new()
                        };

                    let expiry_timestamp = {
                        let exp_time_str = item
                            .get("expTime")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        if exp_time_str.is_empty() || exp_time_str == "0" {
                            chrono::DateTime::<chrono::Utc>::default()
                        } else {
                            crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
                                exp_time_str.parse::<i64>().unwrap(),
                            )
                        }
                    };

                    response.data.push(crate::types::InstrumentInfo {
                        exchange_instrument_type: crate::types::ExchangeInstrumentType::Okx(
                            self.instrument_type,
                        ),
                        symbol: item["instId"].as_str().unwrap().to_string(),
                        base_asset,
                        quote_asset,
                        settle_asset,
                        contract_size,
                        contract_multiplier,

                        order_price_increment: item["tickSz"].as_str().unwrap().to_string(),

                        order_quantity_increment: item["lotSz"].as_str().unwrap().to_string(),

                        order_quantity_min: item["minSz"].as_str().unwrap().to_string(),

                        order_quantity_max: item["maxLmtSz"].as_str().unwrap().to_string(),

                        order_quote_quantity_min: "0".to_string(),

                        expiry_timestamp,

                        ..Default::default()
                    });
                }
            }

            crate::types::OkxInstrumentType::Unknown => panic!(),
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
            let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
                item["ts"].as_str().unwrap().parse::<i64>().unwrap(),
            );

            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Okx(
                    self.instrument_type,
                ),
                symbol: item["instId"].as_str().unwrap().to_string(),
                timestamp,
                bid_price: item["bidPx"].as_str().unwrap().to_string(),
                bid_size: item["bidSz"].as_str().unwrap().to_string(),
                ask_price: item["askPx"].as_str().unwrap().to_string(),
                ask_size: item["askSz"].as_str().unwrap().to_string(),
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
            order_id: json_payload["data"][0]["ordId"]
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

        if let Some(list) = json_payload.get("data").and_then(|v| v.as_array()) {
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

        let details = json_payload["data"].as_array().unwrap().first().unwrap()["details"]
            .as_array()
            .unwrap();

        response.data = details
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
