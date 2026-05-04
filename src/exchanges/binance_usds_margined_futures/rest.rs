#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest
    for crate::exchanges::binance_usds_margined_futures::common::BinanceUsdsMarginedFuturesClient
{
    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        if !get_instrument_info_request.symbol.is_empty() {
            query_params.insert("symbol".into(), get_instrument_info_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/fapi/v1/exchangeInfo",
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

        query_params.insert("symbol".into(), get_top_of_book_request.symbol.clone());

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/fapi/v1/ticker/bookTicker",
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

        let existing = http_request.query_string.clone().unwrap_or_default();

        let params_str = if existing.is_empty() {
            format!("recvWindow={}&timestamp={}", self.recv_window, timestamp)
        } else {
            format!(
                "{}&recvWindow={}&timestamp={}",
                existing, self.recv_window, timestamp
            )
        };

        let signature = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            ed25519_dalek::Signer::sign(&credential.signing_key, params_str.as_bytes()).to_bytes(),
        );

        http_request.query_string = Some(format!("{}&signature={}", params_str, signature));

        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);

        headers.insert(
            reqwest::header::HeaderName::from_static("x-mbx-apikey"),
            reqwest::header::HeaderValue::from_str(&credential.api_key).unwrap(),
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
            "type".into(),
            self.convert_order_type_to_string(place_order_request.order_type)
                .to_string(),
        );
        query_params.insert("quantity".into(), place_order_request.quantity.clone());

        if place_order_request.order_type == crate::types::OrderType::Limit {
            query_params.insert("price".into(), place_order_request.price.clone());
            query_params.insert("timeInForce".into(), "GTC".into());
        }

        if !place_order_request.client_order_id.is_empty() {
            query_params.insert(
                "newClientOrderId".into(),
                place_order_request.client_order_id.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/fapi/v1/order",
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

        query_params.insert("symbol".into(), cancel_order_request.symbol.clone());

        if !cancel_order_request.order_id.is_empty() {
            query_params.insert("orderId".into(), cancel_order_request.order_id.clone());
        } else if !cancel_order_request.client_order_id.is_empty() {
            query_params.insert(
                "origClientOrderId".into(),
                cancel_order_request.client_order_id.clone(),
            );
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::DELETE,
            "/fapi/v1/order",
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

        if !get_open_order_request.symbol.is_empty() {
            query_params.insert("symbol".into(), get_open_order_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/fapi/v1/openOrders",
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
            query_params.insert("symbol".into(), get_position_request.symbol.clone());
        }

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/fapi/v3/positionRisk",
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
            "/fapi/v3/balance",
            None,
            Some(std::collections::HashMap::new()),
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

        let symbols = match json_payload.get("symbols").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return crate::exchange_client::common::Response::GetInstrumentInfo(response),
        };

        response.data.reserve(symbols.len());

        for item in symbols {
            let filters = match item.get("filters").and_then(|v| v.as_array()) {
                Some(f) => f,
                None => continue,
            };

            let price_filter = filters
                .iter()
                .find(|f| f["filterType"].as_str() == Some("PRICE_FILTER"));

            let lot_size = filters
                .iter()
                .find(|f| f["filterType"].as_str() == Some("LOT_SIZE"));

            let min_notional = filters
                .iter()
                .find(|f| f["filterType"].as_str() == Some("MIN_NOTIONAL"));

            response.data.push(crate::types::InstrumentInfo {
                exchange_instrument_type:
                    crate::types::ExchangeInstrumentType::BinanceUsdsMarginedFutures,
                symbol: item["symbol"].as_str().unwrap().to_string(),
                base_asset: item["baseAsset"].as_str().unwrap().to_string(),
                quote_asset: item["quoteAsset"].as_str().unwrap().to_string(),
                settle_asset: item["marginAsset"].as_str().unwrap_or_default().to_string(),
                order_price_increment: price_filter
                    .and_then(|f| f["tickSize"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                order_quantity_increment: lot_size
                    .and_then(|f| f["stepSize"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                order_quantity_min: lot_size
                    .and_then(|f| f["minQty"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                order_quantity_max: lot_size
                    .and_then(|f| f["maxQty"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                order_quote_quantity_min: min_notional
                    .and_then(|f| f["notional"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                order_quote_quantity_max: String::new(),
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

        let timestamp = chrono::Utc::now();

        if json_payload.is_object() {
            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type:
                    crate::types::ExchangeInstrumentType::BinanceUsdsMarginedFutures,
                symbol: json_payload["symbol"].as_str().unwrap().to_string(),
                timestamp,
                bid_price: json_payload["bidPrice"].as_str().unwrap().to_string(),
                bid_size: json_payload["bidQty"].as_str().unwrap().to_string(),
                ask_price: json_payload["askPrice"].as_str().unwrap().to_string(),
                ask_size: json_payload["askQty"].as_str().unwrap().to_string(),
            });
        } else if let Some(arr) = json_payload.as_array() {
            response.data.reserve(arr.len());
            for item in arr {
                response.data.push(crate::types::TopOfBook {
                    exchange_instrument_type:
                        crate::types::ExchangeInstrumentType::BinanceUsdsMarginedFutures,
                    symbol: item["symbol"].as_str().unwrap().to_string(),
                    timestamp,
                    bid_price: item["bidPrice"].as_str().unwrap().to_string(),
                    bid_size: item["bidQty"].as_str().unwrap().to_string(),
                    ask_price: item["askPrice"].as_str().unwrap().to_string(),
                    ask_size: item["askQty"].as_str().unwrap().to_string(),
                });
            }
        }

        crate::exchange_client::common::Response::GetTopOfBook(response)
    }

    fn create_place_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();

        let response = crate::exchange_client::common::PlaceOrderResponse {
            order_id: json_payload["orderId"].as_i64().unwrap().to_string(),
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

        if let Some(list) = json_payload.as_array() {
            response.data = list
                .iter()
                .map(|item| self.convert_rest_json_to_order(item))
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
                .map(|item| self.convert_rest_json_to_position(item))
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

        if let Some(list) = json_payload.as_array() {
            response.data = list
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
            .get("code")
            .and_then(|v| v.as_i64())
            .map(|v| v.to_string());

        new_http_response.error_message = json_payload
            .get("msg")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
