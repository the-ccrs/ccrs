#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest
    for crate::exchanges::aster_futures::common::AsterFuturesClient
{
    fn create_get_instrument_info_http_request(
        &self,
        _get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/fapi/v3/exchangeInfo",
            None,
            None,
            None,
        )
    }

    fn create_get_top_of_book_http_request(
        &self,
        get_top_of_book_request: &crate::exchange_client::common::GetTopOfBookRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params = std::collections::HashMap::new();
        if !get_top_of_book_request.symbol.is_empty() {
            query_params.insert("symbol".to_string(), get_top_of_book_request.symbol.clone());
        }
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/fapi/v3/ticker/bookTicker",
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
        let mut parameters = http_request.query_params.clone().unwrap_or_default();
        parameters.insert("nonce".to_string(), now.timestamp_micros().to_string());
        parameters.insert(
            "signer".to_string(),
            credential.signing_key.address().to_string(),
        );
        let mut parameters = parameters.into_iter().collect::<Vec<_>>();
        parameters.sort_by(|left, right| left.0.cmp(&right.0));
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        for (key, value) in &parameters {
            serializer.append_pair(key, value);
        }
        let payload = serializer.finish();
        let signing_hash = Self::compute_signing_hash(&payload);
        let signature = <alloy::signers::local::PrivateKeySigner as alloy::signers::SignerSync>::sign_hash_sync(
            &credential.signing_key,
            &signing_hash,
        )
        .unwrap();
        http_request.query_string = Some(format!(
            "{}&signature=0x{}",
            payload,
            hex::encode(signature.as_bytes())
        ));
        if http_request.method != reqwest::Method::GET {
            http_request
                .headers
                .get_or_insert_with(reqwest::header::HeaderMap::new)
                .insert(
                    reqwest::header::CONTENT_TYPE,
                    reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"),
                );
        }
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params = std::collections::HashMap::new();
        query_params.insert("symbol".to_string(), place_order_request.symbol.clone());
        query_params.insert(
            "side".to_string(),
            self.convert_order_side_to_string(place_order_request.side)
                .to_string(),
        );
        query_params.insert(
            "type".to_string(),
            self.convert_order_type_to_string(place_order_request.order_type)
                .to_string(),
        );
        query_params.insert("quantity".to_string(), place_order_request.quantity.clone());
        if place_order_request.order_type == crate::types::OrderType::Limit {
            query_params.insert("price".to_string(), place_order_request.price.clone());
            query_params.insert("timeInForce".to_string(), "GTC".to_string());
        }
        if !place_order_request.client_order_id.is_empty() {
            query_params.insert(
                "newClientOrderId".to_string(),
                place_order_request.client_order_id.clone(),
            );
        }
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/fapi/v3/order",
            None,
            Some(query_params),
            None,
        )
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params = std::collections::HashMap::new();
        query_params.insert("symbol".to_string(), cancel_order_request.symbol.clone());
        if !cancel_order_request.order_id.is_empty() {
            query_params.insert("orderId".to_string(), cancel_order_request.order_id.clone());
        } else if !cancel_order_request.client_order_id.is_empty() {
            query_params.insert(
                "origClientOrderId".to_string(),
                cancel_order_request.client_order_id.clone(),
            );
        }
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::DELETE,
            "/fapi/v3/order",
            None,
            Some(query_params),
            None,
        )
    }

    fn create_get_open_order_http_request(
        &self,
        get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut query_params = std::collections::HashMap::new();
        if !get_open_order_request.symbol.is_empty() {
            query_params.insert("symbol".to_string(), get_open_order_request.symbol.clone());
        }
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/fapi/v3/openOrders",
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
            query_params.insert("symbol".to_string(), get_position_request.symbol.clone());
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
        let Some(symbols) = json_payload
            .get("symbols")
            .and_then(serde_json::Value::as_array)
        else {
            return crate::exchange_client::common::Response::GetInstrumentInfo(response);
        };
        response.data.reserve(symbols.len());
        for item in symbols {
            let Some(filters) = item.get("filters").and_then(serde_json::Value::as_array) else {
                continue;
            };
            let price_filter = filters
                .iter()
                .find(|filter| filter["filterType"].as_str() == Some("PRICE_FILTER"));
            let lot_size = filters
                .iter()
                .find(|filter| filter["filterType"].as_str() == Some("LOT_SIZE"));
            let min_notional = filters
                .iter()
                .find(|filter| filter["filterType"].as_str() == Some("MIN_NOTIONAL"));
            response.data.push(crate::types::InstrumentInfo {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::AsterFutures,
                symbol: item["symbol"].as_str().unwrap_or_default().to_string(),
                base_asset: item["baseAsset"].as_str().unwrap_or_default().to_string(),
                quote_asset: item["quoteAsset"].as_str().unwrap_or_default().to_string(),
                settle_asset: item["marginAsset"].as_str().unwrap_or_default().to_string(),
                order_price_increment: price_filter
                    .and_then(|filter| filter["tickSize"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                order_quantity_increment: lot_size
                    .and_then(|filter| filter["stepSize"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                order_quantity_min: lot_size
                    .and_then(|filter| filter["minQty"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                order_quantity_max: lot_size
                    .and_then(|filter| filter["maxQty"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                order_quote_quantity_min: min_notional
                    .and_then(|filter| filter["notional"].as_str())
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
        let values = if let Some(values) = json_payload.as_array() {
            values.iter().collect::<Vec<_>>()
        } else {
            vec![&json_payload]
        };
        response.data.reserve(values.len());
        for item in values {
            let timestamp = item["time"]
                .as_i64()
                .map(crate::utils::convert_unix_timestamp_milliseconds_to_timestamp)
                .unwrap_or_else(chrono::Utc::now);
            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::AsterFutures,
                symbol: item["symbol"].as_str().unwrap_or_default().to_string(),
                timestamp,
                bid_price: item["bidPrice"].as_str().unwrap_or_default().to_string(),
                bid_size: item["bidQty"].as_str().unwrap_or_default().to_string(),
                ask_price: item["askPrice"].as_str().unwrap_or_default().to_string(),
                ask_size: item["askQty"].as_str().unwrap_or_default().to_string(),
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
                order_id: json_payload["orderId"]
                    .as_i64()
                    .unwrap_or_default()
                    .to_string(),
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
        let mut response = crate::exchange_client::common::GetOpenOrderResponse::default();
        if let Some(orders) = json_payload.as_array() {
            response.data = orders
                .iter()
                .map(|order| self.convert_rest_json_to_order(order))
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
        if let Some(positions) = json_payload.as_array() {
            response.data = positions
                .iter()
                .map(|position| self.convert_rest_json_to_position(position))
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
        if let Some(balances) = json_payload.as_array() {
            response.data = balances
                .iter()
                .map(|balance| self.convert_json_value_to_balance(balance))
                .collect();
        }
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
        let mut new_http_response = http_response.clone();
        new_http_response.error_code = json_payload
            .get("code")
            .and_then(serde_json::Value::as_i64)
            .map(|value| value.to_string());
        new_http_response.error_message = json_payload
            .get("msg")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);
        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
