#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest
    for crate::exchanges::kraken_spot::common::KrakenSpotClient
{
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if !get_instrument_info_request.symbol.is_empty() {
            query_params.insert("pair".into(), get_instrument_info_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/0/public/AssetPairs",
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
            query_params.insert("pair".into(), get_top_of_book_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/0/public/Ticker",
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

        let existing = http_request.payload.take().unwrap_or_default();
        let post_data = if existing.is_empty() {
            format!("nonce={}", nonce_str)
        } else {
            format!("nonce={}&{}", nonce_str, existing)
        };
        http_request.payload = Some(post_data.clone());

        let sha_input = format!("{}{}", nonce_str, post_data);
        let mut sha256_hasher = sha2::Sha256::default();
        <sha2::Sha256 as sha2::Digest>::update(&mut sha256_hasher, sha_input.as_bytes());
        let sha256_hash = <sha2::Sha256 as sha2::Digest>::finalize(sha256_hasher);

        let mut hmac_message = Vec::<u8>::new();
        hmac_message.extend_from_slice(http_request.path.as_bytes());
        hmac_message.extend_from_slice(&sha256_hash);

        let decoded_secret = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            credential.api_secret.as_bytes(),
        )
        .unwrap();

        let mut mac =
            <hmac::Hmac<sha2::Sha512> as hmac::digest::KeyInit>::new_from_slice(&decoded_secret)
                .unwrap();
        <hmac::Hmac<sha2::Sha512> as hmac::Mac>::update(&mut mac, &hmac_message);
        let signature_bytes = <hmac::Hmac<sha2::Sha512> as hmac::Mac>::finalize(mac).into_bytes();

        let signature =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, signature_bytes);

        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);

        headers.insert(
            reqwest::header::HeaderName::from_static("api-key"),
            reqwest::header::HeaderValue::from_str(&credential.api_key).unwrap(),
        );

        headers.insert(
            reqwest::header::HeaderName::from_static("api-sign"),
            reqwest::header::HeaderValue::from_str(&signature).unwrap(),
        );
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut parts: Vec<String> = Vec::new();

        parts.push(format!("pair={}", place_order_request.symbol));
        parts.push(format!(
            "type={}",
            self.convert_order_side_to_string(place_order_request.side)
        ));
        parts.push(format!(
            "ordertype={}",
            self.convert_order_type_to_string(place_order_request.order_type)
        ));
        parts.push(format!("volume={}", place_order_request.quantity));

        if place_order_request.order_type == crate::types::OrderType::Limit {
            parts.push(format!("price={}", place_order_request.price));
        }

        if !place_order_request.client_order_id.is_empty() {
            parts.push(format!("cl_ord_id={}", place_order_request.client_order_id));
        }

        let form_body = parts.join("&");

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let mut request = crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/0/private/AddOrder",
            Some(headers),
            None,
            None,
        );
        request.payload = Some(form_body);
        request
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut parts: Vec<String> = Vec::new();

        if !cancel_order_request.order_id.is_empty() {
            parts.push(format!("txid={}", cancel_order_request.order_id));
        } else if !cancel_order_request.client_order_id.is_empty() {
            parts.push(format!(
                "cl_ord_id={}",
                cancel_order_request.client_order_id
            ));
        }

        let form_body = parts.join("&");

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let mut request = crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/0/private/CancelOrder",
            Some(headers),
            None,
            None,
        );
        request.payload = Some(form_body);
        request
    }

    fn create_get_open_order_http_request(
        &self,
        _get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/0/private/OpenOrders",
            Some(headers),
            None,
            None,
        )
    }

    fn create_get_balance_http_request(
        &self,
        _get_balance_request: &crate::exchange_client::common::GetBalanceRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/0/private/Balance",
            Some(headers),
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

        match json.get("error").and_then(|v| v.as_array()) {
            Some(errors) => errors.is_empty(),
            None => false,
        }
    }

    fn create_get_instrument_info_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetInstrumentInfoResponse::default();

        let result = match json_payload.get("result").and_then(|v| v.as_object()) {
            Some(obj) => obj,
            None => return crate::exchange_client::common::Response::GetInstrumentInfo(response),
        };

        response.data.reserve(result.len());

        for (pair_name, item) in result {
            let lot_dec = item["lot_decimals"].as_i64().unwrap_or(8);
            let qty_inc = if lot_dec == 0 {
                "1".to_string()
            } else {
                format!("0.{:0>width$}", 1, width = lot_dec as usize)
            };

            response.data.push(crate::types::InstrumentInfo {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenSpot,
                symbol: pair_name.clone(),
                base_asset: item["base"].as_str().unwrap_or("").to_string(),
                quote_asset: item["quote"].as_str().unwrap_or("").to_string(),
                order_price_increment: item["tick_size"].as_str().unwrap_or("").to_string(),
                order_quantity_increment: qty_inc,
                order_quantity_min: item["ordermin"].as_str().unwrap_or("").to_string(),
                order_quote_quantity_min: item["costmin"].as_str().unwrap_or("").to_string(),
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

        let result = match json_payload.get("result").and_then(|v| v.as_object()) {
            Some(obj) => obj,
            None => return crate::exchange_client::common::Response::GetTopOfBook(response),
        };

        let timestamp = chrono::Utc::now();

        response.data.reserve(result.len());

        for (pair_name, item) in result {
            let bid_price = item["b"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let bid_size = item["b"]
                .as_array()
                .and_then(|a| a.get(2))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let ask_price = item["a"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let ask_size = item["a"]
                .as_array()
                .and_then(|a| a.get(2))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::KrakenSpot,
                symbol: pair_name.clone(),
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

        let order_id = json_payload["result"]["txid"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
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

        let open_orders = match json_payload["result"]["open"].as_object() {
            Some(obj) => obj,
            None => return crate::exchange_client::common::Response::GetOpenOrder(response),
        };

        response.data.reserve(open_orders.len());

        for (txid, order_value) in open_orders {
            response
                .data
                .push(self.convert_json_value_to_order(txid, order_value));
        }

        crate::exchange_client::common::Response::GetOpenOrder(response)
    }

    fn create_get_balance_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let mut response = crate::exchange_client::common::GetBalanceResponse::default();

        let result = match json_payload.get("result").and_then(|v| v.as_object()) {
            Some(obj) => obj,
            None => return crate::exchange_client::common::Response::GetBalance(response),
        };

        response.data.reserve(result.len());

        for (asset, balance_value) in result {
            let quantity = balance_value.as_str().unwrap_or("");
            response
                .data
                .push(self.convert_entry_to_balance(asset, quantity));
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
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
