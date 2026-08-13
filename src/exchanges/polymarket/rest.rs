#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest for crate::exchanges::polymarket::common::PolymarketClient {
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params = std::collections::HashMap::new();
        if !get_instrument_info_request.symbol.is_empty() {
            query_params.insert(
                "clob_token_ids".to_string(),
                get_instrument_info_request.symbol.clone(),
            );
        }
        if get_instrument_info_request.limit > 0 {
            query_params.insert(
                "limit".to_string(),
                get_instrument_info_request.limit.to_string(),
            );
        }
        if !get_instrument_info_request.next_page_cursor.is_empty() {
            query_params.insert(
                "after_cursor".to_string(),
                get_instrument_info_request.next_page_cursor.clone(),
            );
        }
        crate::networking::http::HttpRequest::new(
            &self.gamma_api_base_url,
            reqwest::Method::GET,
            "/markets/keyset",
            None,
            Some(query_params),
            None,
        )
    }

    fn create_get_top_of_book_http_request(
        &self,
        get_top_of_book_request: &crate::exchange_client::common::GetTopOfBookRequest,
    ) -> crate::networking::http::HttpRequest {
        let body = serde_json::json!([{
            "token_id": get_top_of_book_request.symbol
        }]);
        crate::networking::http::HttpRequest::new(
            &self.clob_api_base_url,
            reqwest::Method::POST,
            "/books",
            None,
            None,
            Some(body),
        )
    }

    fn sign_http_request(
        &self,
        http_request: &mut crate::networking::http::HttpRequest,
        now: chrono::DateTime<chrono::Utc>,
    ) {
        let credential = match self.credential.as_ref() {
            Some(c) => c,
            None => return,
        };

        if !http_request.base_url.contains("clob.polymarket.com") {
            return;
        }

        let timestamp = now.timestamp().to_string();
        let method = http_request.method.as_str().to_uppercase();

        let request_path =
            if let Some(qs) = http_request.query_string.as_ref().filter(|s| !s.is_empty()) {
                format!("{}?{}", http_request.path, qs)
            } else {
                http_request.path.clone()
            };

        let body = http_request.payload.as_deref().unwrap_or("");

        let signature =
            self.compute_l2_hmac_signature(&timestamp, &method, &request_path, body, credential);

        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);

        headers.insert(
            reqwest::header::HeaderName::from_static("poly_address"),
            reqwest::header::HeaderValue::from_str(&self.signer_address).unwrap(),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("poly_signature"),
            reqwest::header::HeaderValue::from_str(&signature).unwrap(),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("poly_timestamp"),
            reqwest::header::HeaderValue::from_str(&timestamp).unwrap(),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("poly_api_key"),
            reqwest::header::HeaderValue::from_str(&credential.api_key).unwrap(),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("poly_passphrase"),
            reqwest::header::HeaderValue::from_str(&credential.api_passphrase).unwrap(),
        );
    }

    async fn prepare_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
        now: chrono::DateTime<chrono::Utc>,
    ) -> crate::networking::http::HttpRequest {
        let sdk_client = self
            .sdk_client
            .as_ref()
            .expect("initialize_sdk_client must be called before placing orders");

        let credential = self
            .credential
            .as_ref()
            .expect("Credential not initialized");

        let signer = super::common::private_key_to_sdk_signer(&credential.signing_key)
            .expect("invalid private key");

        let token_id: alloy::primitives::U256 = place_order_request
            .symbol
            .parse()
            .expect("invalid token id");
        let size = place_order_request
            .quantity
            .parse()
            .expect("invalid quantity");

        let side = match place_order_request.side {
            crate::types::OrderSide::Buy => polymarket_client_sdk_v2::clob::types::Side::Buy,
            crate::types::OrderSide::Sell => polymarket_client_sdk_v2::clob::types::Side::Sell,
            _ => panic!("unsupported side"),
        };

        let signable_order = match place_order_request.order_type {
            crate::types::OrderType::Limit => {
                let price = place_order_request.price.parse().expect("invalid price");
                sdk_client
                    .limit_order()
                    .token_id(token_id)
                    .price(price)
                    .size(size)
                    .side(side)
                    .build()
                    .await
                    .expect("failed to build order")
            }

            crate::types::OrderType::Market => sdk_client
                .market_order()
                .token_id(token_id)
                .amount(
                    polymarket_client_sdk_v2::clob::types::Amount::usdc(
                        <polymarket_client_sdk_v2::types::Decimal as std::str::FromStr>::from_str(
                            &place_order_request.quantity,
                        )
                        .unwrap(),
                    )
                    .unwrap(),
                )
                .side(side)
                .order_type(polymarket_client_sdk_v2::clob::types::OrderType::FOK)
                .build()
                .await
                .expect("failed to build market order"),

            crate::types::OrderType::Unknown => {
                panic!("unsupported order type")
            }
        };

        let signed_order = sdk_client
            .sign(&signer, signable_order)
            .await
            .expect("failed to sign order");

        let body = serde_json::to_value(&signed_order).expect("failed to serialize order");

        let mut http_request = crate::networking::http::HttpRequest::new(
            &self.clob_api_base_url,
            reqwest::Method::POST,
            "/order",
            None,
            None,
            Some(body),
        );

        self.sign_http_request(&mut http_request, now);

        http_request
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

        let body = serde_json::json!({
            "orderID": order_id
        });

        crate::networking::http::HttpRequest::new(
            &self.clob_api_base_url,
            reqwest::Method::DELETE,
            "/order",
            None,
            None,
            Some(body),
        )
    }

    fn create_get_open_order_http_request(
        &self,
        get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params = std::collections::HashMap::new();
        if !get_open_order_request.symbol.is_empty() {
            query_params.insert(
                "asset_id".to_string(),
                get_open_order_request.symbol.clone(),
            );
        }
        if !get_open_order_request.next_page_cursor.is_empty() {
            query_params.insert(
                "next_cursor".to_string(),
                get_open_order_request.next_page_cursor.clone(),
            );
        }
        crate::networking::http::HttpRequest::new(
            &self.clob_api_base_url,
            reqwest::Method::GET,
            "/data/orders",
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
        query_params.insert("user".to_string(), self.funder_address.clone());
        if get_position_request.limit > 0 {
            query_params.insert("limit".to_string(), get_position_request.limit.to_string());
        }
        if get_position_request.offset > 0 {
            query_params.insert(
                "offset".to_string(),
                get_position_request.offset.to_string(),
            );
        }
        crate::networking::http::HttpRequest::new(
            &self.data_api_base_url,
            reqwest::Method::GET,
            "/positions",
            None,
            Some(query_params),
            None,
        )
    }

    fn create_get_balance_http_request(
        &self,
        _get_balance_request: &crate::exchange_client::common::GetBalanceRequest,
    ) -> crate::networking::http::HttpRequest {
        let pusd_address = "0xC011a7E12a19f7B1f670d46F03B03f3342E82DFB";
        let function_selector = "70a08231";

        let addr = self.funder_address.trim_start_matches("0x");
        let padded_addr = format!("{:0>64}", addr);
        let call_data = format!("0x{}{}", function_selector, padded_addr);

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": chrono::Utc::now().timestamp_millis() as u64,
            "method": "eth_call",
            "params": [
                {
                    "to": pusd_address,
                    "data": call_data
                },
                "latest"
            ]
        });

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        crate::networking::http::HttpRequest::new(
            "https://polygon-bor-rpc.publicnode.com",
            reqwest::Method::POST,
            "/",
            None,
            None,
            Some(body),
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
            return true;
        };
        if let Some(error) = json.get("error").and_then(|v| v.as_str())
            && !error.is_empty()
        {
            return false;
        }
        if let Some(success) = json.get("success").and_then(|v| v.as_bool())
            && !success
        {
            return false;
        }
        true
    }

    fn create_get_instrument_info_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();
        let mut response = crate::exchange_client::common::GetInstrumentInfoResponse::default();

        let markets = if let Some(arr) = json_payload.get("markets").and_then(|v| v.as_array()) {
            arr.clone()
        } else {
            vec![]
        };

        let next_cursor = json_payload
            .get("next_cursor")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        response.data.reserve(markets.len());
        for item in markets.iter() {
            let negative_risk = item
                .get("events")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|event| event.get("negRisk"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if let Some(clob_token_ids) = item["clobTokenIds"]
                .as_str()
                .and_then(|s| serde_json::from_str::<Vec<String>>(s).ok())
            {
                for clob_token_id in clob_token_ids.iter() {
                    let order_price_increment = item["orderPriceMinTickSize"]
                        .as_f64()
                        .map(|v| v.to_string())
                        .or_else(|| {
                            item["orderPriceMinTickSize"]
                                .as_str()
                                .map(|s| s.to_string())
                        })
                        .unwrap_or_default();

                    let order_quantity_min = item["orderMinSize"]
                        .as_f64()
                        .map(|v| v.to_string())
                        .or_else(|| item["orderMinSize"].as_str().map(|s| s.to_string()))
                        .unwrap_or_default();

                    let expiry_timestamp = item
                        .get("endDate")
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_default();

                    response.data.push(crate::types::InstrumentInfo {
                        exchange_instrument_type: crate::types::ExchangeInstrumentType::Polymarket,
                        symbol: clob_token_id.clone(),
                        order_price_increment,
                        order_quantity_min,
                        exchange_specific: crate::types::ExchangeSpecificInstrumentInfo::Polymarket(
                            crate::types::PolymarketSpecificInstrumentInfo { negative_risk },
                        ),
                        expiry_timestamp,
                        ..Default::default()
                    });
                }
            }
        }

        response.next_page_cursor = next_cursor;
        crate::exchange_client::common::Response::GetInstrumentInfo(response)
    }

    fn create_get_top_of_book_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response
            .json_payload
            .unwrap_or(serde_json::Value::Array(vec![]));
        let mut response = crate::exchange_client::common::GetTopOfBookResponse::default();

        let books = if let Some(arr) = json_payload.as_array() {
            arr.clone()
        } else {
            vec![json_payload]
        };

        for book in books.iter() {
            let asset_id = book["asset_id"].as_str().unwrap_or("").to_string();
            let timestamp_str = book["timestamp"].as_str().unwrap_or("0");
            let ts_ms: i64 = timestamp_str.parse().unwrap_or(0);
            let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(ts_ms);

            let bids = book.get("bids").and_then(|v| v.as_array());
            let asks = book.get("asks").and_then(|v| v.as_array());

            let best_bid = bids.and_then(|b| b.last());
            let best_ask = asks.and_then(|a| a.last());

            let bid_price = best_bid
                .and_then(|b| b["price"].as_str())
                .unwrap_or("")
                .to_string();
            let bid_size = best_bid
                .and_then(|b| b["size"].as_str())
                .unwrap_or("")
                .to_string();
            let ask_price = best_ask
                .and_then(|a| a["price"].as_str())
                .unwrap_or("")
                .to_string();
            let ask_size = best_ask
                .and_then(|a| a["size"].as_str())
                .unwrap_or("")
                .to_string();

            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Polymarket,
                symbol: asset_id,
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
        let json_payload = http_response.json_payload.unwrap_or_default();
        let order_id = json_payload["orderID"].as_str().unwrap_or("").to_string();
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
        let json_payload = http_response.json_payload.unwrap_or_default();
        let mut response = crate::exchange_client::common::GetOpenOrderResponse::default();

        let next_cursor = json_payload
            .get("next_cursor")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if let Some(list) = json_payload.get("data").and_then(|v| v.as_array()) {
            response.data = list
                .iter()
                .map(|item| self.convert_json_value_to_order(item))
                .collect();
        }

        response.next_page_cursor = next_cursor;
        crate::exchange_client::common::Response::GetOpenOrder(response)
    }

    fn create_get_position_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap_or_default();
        let mut response = crate::exchange_client::common::GetPositionResponse::default();

        let positions = if let Some(arr) = json_payload.as_array() {
            arr.clone()
        } else if let Some(arr) = json_payload.get("data").and_then(|v| v.as_array()) {
            arr.clone()
        } else {
            vec![]
        };

        response.data = positions
            .iter()
            .filter(|item| {
                let size = item["size"].as_f64().unwrap_or(0.0);
                size != 0.0
            })
            .map(|item| self.convert_json_value_to_position(item))
            .collect();

        crate::exchange_client::common::Response::GetPosition(response)
    }

    fn create_get_balance_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap_or_default();
        let mut response = crate::exchange_client::common::GetBalanceResponse::default();

        if let Some(result_hex) = json_payload.get("result").and_then(|v| v.as_str()) {
            let hex = result_hex.trim_start_matches("0x");
            if let Ok(balance) = u64::from_str_radix(hex, 16) {
                let balance_str = crate::utils::scale_decimal(balance, 6);
                response.data = vec![crate::types::Balance {
                    exchange: crate::types::Exchange::Polymarket,
                    asset: "pUSD".to_string(),
                    quantity: balance_str,
                }];
            }
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
            .get("error")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| {
                json_payload
                    .get("errorMsg")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            });
        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
