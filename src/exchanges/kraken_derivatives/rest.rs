#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest
    for crate::exchanges::kraken_derivatives::common::KrakenDerivativesClient
{
    fn create_get_instrument_info_http_request(
        &self,
        _get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/derivatives/api/v3/instruments",
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
            query_params.insert("symbol".into(), get_top_of_book_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/derivatives/api/v3/tickers",
            None,
            Some(query_params),
            None,
        )
    }

    fn sign_http_request(
        &self,
        http_request: &mut crate::networking::http::HttpRequest,
        _now: chrono::DateTime<chrono::Utc>,
    ) {
        let credential = self.credential.as_ref().unwrap();

        let nonce = crate::exchange_client::common::Common::generate_next_nonce(self);
        let nonce_str = nonce.to_string();

        let post_data = http_request.query_string.as_deref().unwrap_or("");
        let path_stripped = http_request.path.trim_start_matches("/derivatives");
        let nonce_path = format!("{}{}", nonce_str, path_stripped);

        let mut sha256_hasher = sha2::Sha256::default();
        <sha2::Sha256 as sha2::Digest>::update(&mut sha256_hasher, post_data.as_bytes());
        <sha2::Sha256 as sha2::Digest>::update(&mut sha256_hasher, nonce_path.as_bytes());
        let sha256_digest = <sha2::Sha256 as sha2::Digest>::finalize(sha256_hasher);

        let decoded_secret = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            credential.api_secret.as_bytes(),
        )
        .unwrap();

        let mut mac =
            <hmac::Hmac<sha2::Sha512> as hmac::digest::KeyInit>::new_from_slice(&decoded_secret)
                .unwrap();
        <hmac::Hmac<sha2::Sha512> as hmac::Mac>::update(&mut mac, &sha256_digest);
        let signature_bytes = <hmac::Hmac<sha2::Sha512> as hmac::Mac>::finalize(mac).into_bytes();

        let signature =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature_bytes);

        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);

        headers.insert(
            reqwest::header::HeaderName::from_static("apikey"),
            reqwest::header::HeaderValue::from_str(&credential.api_key).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("authent"),
            reqwest::header::HeaderValue::from_str(&signature).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("nonce"),
            reqwest::header::HeaderValue::from_str(&nonce_str).unwrap(),
        );
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        query_params.insert("symbol".into(), place_order_request.symbol.clone());
        query_params.insert(
            "side".into(),
            self.convert_order_side_to_string(place_order_request.side)
                .to_string(),
        );
        query_params.insert(
            "orderType".into(),
            self.convert_order_type_to_string(place_order_request.order_type)
                .to_string(),
        );
        query_params.insert("size".into(), place_order_request.quantity.clone());

        if place_order_request.order_type == crate::types::OrderType::Limit {
            query_params.insert("limitPrice".into(), place_order_request.price.clone());
        }

        if !place_order_request.client_order_id.is_empty() {
            query_params.insert(
                "cliOrdId".into(),
                place_order_request.client_order_id.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/derivatives/api/v3/sendorder",
            None,
            Some(query_params),
            None,
        )
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if !cancel_order_request.order_id.is_empty() {
            query_params.insert("order_id".into(), cancel_order_request.order_id.clone());
        } else if !cancel_order_request.client_order_id.is_empty() {
            query_params.insert(
                "cliOrdId".into(),
                cancel_order_request.client_order_id.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/derivatives/api/v3/cancelorder",
            None,
            Some(query_params),
            None,
        )
    }

    fn create_get_open_order_http_request(
        &self,
        _get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/derivatives/api/v3/openorders",
            None,
            None,
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
            "/derivatives/api/v3/openpositions",
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
            "/derivatives/api/v3/accounts",
            None,
            None,
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

        json.get("result").and_then(|v| v.as_str()) == Some("success")
    }

    fn create_get_instrument_info_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetInstrumentInfoResponse::default();

        let instruments = match json_payload.get("instruments").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetInstrumentInfo(response),
        };

        response.data.reserve(instruments.len());

        for item in instruments {
            let tick_size = item["tickSize"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default();

            let contract_size = item["contractSize"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default();

            response.data.push(crate::types::InstrumentInfo {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenDerivatives,
                symbol: item["symbol"].as_str().unwrap_or("").to_string(),
                base_asset: item["base"].as_str().unwrap_or("").to_string(),
                quote_asset: item["quote"].as_str().unwrap_or("").to_string(),
                order_price_increment: tick_size,
                contract_size,
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

        let tickers = match json_payload.get("tickers").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetTopOfBook(response),
        };

        let timestamp = chrono::Utc::now();

        response.data.reserve(tickers.len());

        for item in tickers {
            let bid_price = item["bid"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default();

            let bid_size = item["bidSize"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default();

            let ask_price = item["ask"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default();

            let ask_size = item["askSize"]
                .as_f64()
                .map(|f| f.to_string())
                .unwrap_or_default();

            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenDerivatives,
                symbol: item["symbol"].as_str().unwrap_or("").to_string(),
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

        let order_id = json_payload["sendStatus"]["order_id"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let response = crate::exchange_client::common::PlaceOrderResponse { order_id };

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

        let open_orders = match json_payload.get("openOrders").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetOpenOrder(response),
        };

        response.data.reserve(open_orders.len());

        for order_value in open_orders {
            response
                .data
                .push(self.convert_json_value_to_order(order_value));
        }

        crate::exchange_client::common::Response::GetOpenOrder(response)
    }

    fn create_get_position_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetPositionResponse::default();

        let open_positions = match json_payload.get("openPositions").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetPosition(response),
        };

        response.data.reserve(open_positions.len());

        for position_value in open_positions {
            response
                .data
                .push(self.convert_json_value_to_position(position_value));
        }

        crate::exchange_client::common::Response::GetPosition(response)
    }

    fn create_get_balance_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetBalanceResponse::default();

        let accounts = match json_payload.get("accounts").and_then(|v| v.as_object()) {
            Some(obj) => obj,
            None => return crate::exchange_client::common::Response::GetBalance(response),
        };

        for (_, account) in accounts {
            if let Some(currencies) = account.get("currencies").and_then(|v| v.as_object()) {
                for (asset, entry) in currencies {
                    let Some(qty) = entry.get("quantity") else {
                        continue;
                    };
                    let quantity = match qty {
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        _ => continue,
                    };
                    response
                        .data
                        .push(self.convert_entry_to_balance(asset, &quantity));
                }
                continue;
            }

            if let Some(balances) = account.get("balances").and_then(|v| v.as_object()) {
                for (asset, quantity_val) in balances {
                    let quantity = match quantity_val {
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        _ => continue,
                    };
                    response
                        .data
                        .push(self.convert_entry_to_balance(asset, &quantity));
                }
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
            .map(str::to_string);

        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
