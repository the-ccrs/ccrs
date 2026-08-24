#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest for crate::exchanges::bitstamp::common::BitstampClient {
    fn create_get_instrument_info_http_request(
        &self,
        _: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/api/v2/markets/",
            None,
            None,
            None,
        )
    }

    fn create_get_top_of_book_http_request(
        &self,
        request: &crate::exchange_client::common::GetTopOfBookRequest,
    ) -> crate::networking::http::HttpRequest {
        let path = if request.symbol.is_empty() {
            "/api/v2/ticker/".to_string()
        } else {
            format!("/api/v2/ticker/{}/", request.symbol)
        };
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &path,
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
        let nonce = uuid::Uuid::new_v4().to_string().to_lowercase();
        let timestamp = now.timestamp_millis().to_string();
        let version = "v2";
        let x_auth = format!("BITSTAMP {}", credential.api_key);
        let parsed_base_url = url::Url::parse(&self.rest_api_base_url).unwrap();
        let host = parsed_base_url.host_str().unwrap();
        let query = http_request.query_string.as_deref().unwrap_or("");
        let body = http_request.payload.as_deref().unwrap_or("");
        let content_type = if body.is_empty() {
            ""
        } else {
            "application/x-www-form-urlencoded"
        };
        let message = format!(
            "{}{}{}{}{}{}{}{}{}",
            x_auth,
            http_request.method.as_str(),
            host,
            http_request.path,
            query,
            content_type,
            nonce,
            timestamp,
            version
        ) + body;
        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .unwrap();
        <hmac::Hmac<sha2::Sha256> as hmac::Mac>::update(&mut mac, message.as_bytes());
        let signature =
            hex::encode(<hmac::Hmac<sha2::Sha256> as hmac::Mac>::finalize(mac).into_bytes());
        let headers = http_request
            .headers
            .get_or_insert_with(reqwest::header::HeaderMap::new);
        headers.insert(
            reqwest::header::HeaderName::from_static("x-auth"),
            reqwest::header::HeaderValue::from_str(&x_auth).unwrap(),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("x-auth-signature"),
            reqwest::header::HeaderValue::from_str(&signature).unwrap(),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("x-auth-nonce"),
            reqwest::header::HeaderValue::from_str(&nonce).unwrap(),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("x-auth-timestamp"),
            reqwest::header::HeaderValue::from_str(&timestamp).unwrap(),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("x-auth-version"),
            reqwest::header::HeaderValue::from_static(version),
        );
        if !body.is_empty() {
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        } else {
            headers.remove(reqwest::header::CONTENT_TYPE);
        }
    }

    fn create_place_order_http_request(
        &self,
        request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let side = match request.side {
            crate::types::OrderSide::Buy => "buy",
            crate::types::OrderSide::Sell => "sell",
            crate::types::OrderSide::Unknown => panic!("Invalid side"),
        };
        let path = match request.order_type {
            crate::types::OrderType::Limit => format!("/api/v2/{}/{}/", side, request.symbol),
            crate::types::OrderType::Market => {
                format!("/api/v2/{}/market/{}/", side, request.symbol)
            }
            crate::types::OrderType::Unknown => panic!("Invalid order type"),
        };
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        serializer.append_pair("amount", &request.quantity);
        if request.order_type == crate::types::OrderType::Limit {
            serializer.append_pair("price", &request.price);
        }
        if !request.client_order_id.is_empty() {
            serializer.append_pair("client_order_id", &request.client_order_id);
        }
        if !request.leverage.is_empty() {
            serializer.append_pair("margin_mode", "CROSS");
            serializer.append_pair("leverage", &request.leverage);
        }
        let mut http_request = crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            &path,
            None,
            None,
            None,
        );
        http_request.payload = Some(serializer.finish());
        http_request
    }

    fn create_cancel_order_http_request(
        &self,
        request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut serializer = url::form_urlencoded::Serializer::new(String::new());
        if !request.order_id.is_empty() {
            serializer.append_pair("id", &request.order_id);
        } else {
            serializer.append_pair("client_order_id", &request.client_order_id);
        }
        let mut http_request = crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/api/v2/cancel_order/",
            None,
            None,
            None,
        );
        http_request.payload = Some(serializer.finish());
        http_request
    }

    fn create_get_open_order_http_request(
        &self,
        request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let path = if request.symbol.is_empty() {
            "/api/v2/open_orders/".to_string()
        } else {
            format!("/api/v2/open_orders/{}/", request.symbol)
        };
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            &path,
            None,
            None,
            None,
        )
    }

    fn create_get_position_http_request(
        &self,
        request: &crate::exchange_client::common::GetPositionRequest,
    ) -> crate::networking::http::HttpRequest {
        let path = if request.symbol.is_empty() {
            "/api/v2/open_positions/".to_string()
        } else {
            format!("/api/v2/open_positions/{}/", request.symbol)
        };
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            &path,
            None,
            None,
            None,
        )
    }

    fn create_get_balance_http_request(
        &self,
        _: &crate::exchange_client::common::GetBalanceRequest,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/api/v2/account_balances/",
            None,
            None,
            None,
        )
    }

    fn is_http_response_success(
        &self,
        http_response: &crate::networking::http::HttpResponse,
    ) -> bool {
        http_response.status.is_success() && http_response.json_payload.is_some()
    }

    fn create_get_instrument_info_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut response = crate::exchange_client::common::GetInstrumentInfoResponse::default();
        if let Some(markets) = http_response
            .json_payload
            .as_ref()
            .and_then(|v| v.as_array())
        {
            response.data.reserve(markets.len());
            for market in markets {
                let decimals = market["base_decimals"].as_u64().unwrap_or(0) as usize;
                let quantity_increment = if decimals == 0 {
                    "1".to_string()
                } else {
                    format!("0.{:0>width$}", 1, width = decimals)
                };
                response.data.push(crate::types::InstrumentInfo {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitstamp,
                    symbol: market["market_symbol"].as_str().unwrap_or("").to_string(),
                    base_asset: market["base_currency"].as_str().unwrap_or("").to_string(),
                    quote_asset: market["counter_currency"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    order_price_increment: market["tick_size"].as_str().unwrap_or("").to_string(),
                    order_quantity_increment: quantity_increment,
                    order_quantity_min: market["minimum_order_amount"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    order_quantity_max: market["maximum_order_amount"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    order_quote_quantity_min: market["minimum_order_value"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    order_quote_quantity_max: market["maximum_order_value"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    settle_asset: market["counter_currency"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    underlying_symbol: market["underlying_asset"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    contract_size: market["contract_size"].as_str().unwrap_or("").to_string(),
                    ..Default::default()
                });
            }
        }
        crate::exchange_client::common::Response::GetInstrumentInfo(response)
    }

    fn create_get_top_of_book_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut response = crate::exchange_client::common::GetTopOfBookResponse::default();
        let json = http_response.json_payload.unwrap();
        let values: Vec<&serde_json::Value> = match json.as_array() {
            Some(values) => values.iter().collect(),
            None => vec![&json],
        };
        response.data.reserve(values.len());
        for value in values {
            let timestamp = value["timestamp"]
                .as_str()
                .and_then(|v| v.parse::<i64>().ok())
                .and_then(|v| chrono::DateTime::<chrono::Utc>::from_timestamp(v, 0))
                .unwrap_or_default();
            response.data.push(crate::types::TopOfBook {
                exchange_instrument_type: crate::types::ExchangeInstrumentType::Bitstamp,
                symbol: value["market"]
                    .as_str()
                    .or_else(|| value["pair"].as_str())
                    .unwrap_or("")
                    .to_string(),
                timestamp,
                bid_price: value["bid"].as_str().unwrap_or("").to_string(),
                bid_size: String::new(),
                ask_price: value["ask"].as_str().unwrap_or("").to_string(),
                ask_size: String::new(),
            });
        }
        crate::exchange_client::common::Response::GetTopOfBook(response)
    }

    fn create_place_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json = http_response.json_payload.unwrap();
        let order_id = json["id"].as_str().map(str::to_string).unwrap_or_else(|| {
            json["id"]
                .as_u64()
                .map(|v| v.to_string())
                .unwrap_or_default()
        });
        crate::exchange_client::common::Response::PlaceOrder(
            crate::exchange_client::common::PlaceOrderResponse { order_id },
        )
    }

    fn create_cancel_order_rest_response(
        &self,
        _: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        crate::exchange_client::common::Response::CancelOrder(
            crate::exchange_client::common::CancelOrderResponse::default(),
        )
    }

    fn create_get_open_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut response = crate::exchange_client::common::GetOpenOrderResponse::default();
        if let Some(orders) = http_response
            .json_payload
            .as_ref()
            .and_then(|v| v.as_array())
        {
            response.data = orders
                .iter()
                .map(|value| self.convert_json_value_to_order(value))
                .collect();
        }
        crate::exchange_client::common::Response::GetOpenOrder(response)
    }

    fn create_get_position_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut response = crate::exchange_client::common::GetPositionResponse::default();
        if let Some(positions) = http_response
            .json_payload
            .as_ref()
            .and_then(|v| v.as_array())
        {
            response.data = positions
                .iter()
                .map(|value| self.convert_json_value_to_position(value))
                .collect();
        }
        crate::exchange_client::common::Response::GetPosition(response)
    }

    fn create_get_balance_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut response = crate::exchange_client::common::GetBalanceResponse::default();
        if let Some(balances) = http_response
            .json_payload
            .as_ref()
            .and_then(|v| v.as_array())
        {
            response.data = balances
                .iter()
                .map(|value| crate::types::Balance {
                    exchange: crate::types::Exchange::Bitstamp,
                    asset: value["currency"].as_str().unwrap_or("").to_string(),
                    quantity: value["total"].as_str().unwrap_or("").to_string(),
                })
                .collect();
        }
        crate::exchange_client::common::Response::GetBalance(response)
    }

    fn create_http_error_response(
        &self,
        http_response: &crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut response = http_response.clone();
        if let Some(json) = response.json_payload.as_ref() {
            response.error_code = json["response_code"]
                .as_str()
                .map(str::to_string)
                .or_else(|| json["code"].as_str().map(str::to_string));
            response.error_message = json["response_explanation"]
                .as_str()
                .map(str::to_string)
                .or_else(|| json["reason"].as_str().map(str::to_string))
                .or_else(|| json["error"].as_str().map(str::to_string));
        }
        crate::exchange_client::common::Response::HttpErrorResponse(response)
    }
}
