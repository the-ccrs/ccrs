#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest for crate::exchanges::cryptocom::common::CryptocomClient {
    fn create_get_instrument_info_http_request(
        &self,
        request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut params = std::collections::HashMap::<String, String>::new();
        if !request.symbol.is_empty() {
            params.insert("instrument_name".to_string(), request.symbol.clone());
        }
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/public/get-instruments",
            None,
            Some(params),
            None,
        )
    }

    fn create_get_top_of_book_http_request(
        &self,
        request: &crate::exchange_client::common::GetTopOfBookRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut params = std::collections::HashMap::<String, String>::new();
        if !request.symbol.is_empty() {
            params.insert("instrument_name".to_string(), request.symbol.clone());
        }
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::GET,
            "/public/get-tickers",
            None,
            Some(params),
            None,
        )
    }

    fn sign_http_request(
        &self,
        http_request: &mut crate::networking::http::HttpRequest,
        now: chrono::DateTime<chrono::Utc>,
    ) {
        let credential = self.credential.as_ref().unwrap();
        let nonce = now.timestamp_millis();
        let id = nonce;
        let method = http_request.path.trim_start_matches('/');
        let params = http_request
            .json_payload
            .as_ref()
            .and_then(|value| value.get("params"))
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let mut parameter_string = String::new();
        if let Some(object) = params.as_object() {
            let mut keys = object.keys().collect::<Vec<&String>>();
            keys.sort();
            for key in keys {
                parameter_string.push_str(key);
                parameter_string.push_str(&Self::json_value_to_string(&object[key]));
            }
        }
        let signature_payload = format!(
            "{}{}{}{}{}",
            method, id, credential.api_key, parameter_string, nonce
        );
        let mut mac = <hmac::Hmac<sha2::Sha256> as hmac::digest::KeyInit>::new_from_slice(
            credential.api_secret.as_bytes(),
        )
        .unwrap();
        <hmac::Hmac<sha2::Sha256> as hmac::Mac>::update(&mut mac, signature_payload.as_bytes());
        let signature =
            hex::encode(<hmac::Hmac<sha2::Sha256> as hmac::Mac>::finalize(mac).into_bytes());
        let body = serde_json::json!({
            "id": id,
            "method": method,
            "api_key": credential.api_key,
            "params": params,
            "nonce": nonce,
            "sig": signature
        });
        http_request.json_payload = Some(body.clone());
        http_request.payload = Some(serde_json::to_string(&body).unwrap());
    }

    fn create_place_order_http_request(
        &self,
        request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut params = serde_json::Map::<String, serde_json::Value>::new();
        params.insert(
            "instrument_name".to_string(),
            serde_json::json!(request.symbol),
        );
        params.insert(
            "side".to_string(),
            serde_json::json!(self.convert_order_side_to_string(request.side)),
        );
        params.insert(
            "type".to_string(),
            serde_json::json!(self.convert_order_type_to_string(request.order_type)),
        );
        params.insert("quantity".to_string(), serde_json::json!(request.quantity));
        if request.order_type == crate::types::OrderType::Limit {
            params.insert("price".to_string(), serde_json::json!(request.price));
        }
        if !request.client_order_id.is_empty() {
            params.insert(
                "client_oid".to_string(),
                serde_json::json!(request.client_order_id),
            );
        }
        Self::private_request(
            &self.rest_api_base_url,
            "/private/create-order",
            serde_json::Value::Object(params),
        )
    }

    fn create_cancel_order_http_request(
        &self,
        request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let mut params = serde_json::Map::<String, serde_json::Value>::new();
        if !request.order_id.is_empty() {
            params.insert("order_id".to_string(), serde_json::json!(request.order_id));
        } else if !request.client_order_id.is_empty() {
            params.insert(
                "client_oid".to_string(),
                serde_json::json!(request.client_order_id),
            );
        }
        Self::private_request(
            &self.rest_api_base_url,
            "/private/cancel-order",
            serde_json::Value::Object(params),
        )
    }

    fn create_get_open_order_http_request(
        &self,
        request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let params = if request.symbol.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::json!({"instrument_name": request.symbol})
        };
        Self::private_request(&self.rest_api_base_url, "/private/get-open-orders", params)
    }

    fn create_get_position_http_request(
        &self,
        request: &crate::exchange_client::common::GetPositionRequest,
    ) -> crate::networking::http::HttpRequest {
        let params = if request.symbol.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::json!({"instrument_name": request.symbol})
        };
        Self::private_request(&self.rest_api_base_url, "/private/get-positions", params)
    }

    fn create_get_balance_http_request(
        &self,
        _request: &crate::exchange_client::common::GetBalanceRequest,
    ) -> crate::networking::http::HttpRequest {
        Self::private_request(
            &self.rest_api_base_url,
            "/private/user-balance",
            serde_json::json!({}),
        )
    }

    fn is_http_response_success(&self, response: &crate::networking::http::HttpResponse) -> bool {
        response.status.is_success()
            && response
                .json_payload
                .as_ref()
                .and_then(|value| value["code"].as_i64())
                == Some(0)
    }

    fn create_get_instrument_info_rest_response(
        &self,
        response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut result = crate::exchange_client::common::GetInstrumentInfoResponse::default();
        if let Some(instruments) = response
            .json_payload
            .as_ref()
            .and_then(|value| value["result"]["data"].as_array())
        {
            result.data.reserve(instruments.len());
            for value in instruments {
                result.data.push(crate::types::InstrumentInfo {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Cryptocom,
                    symbol: value["symbol"].as_str().unwrap_or("").to_string(),
                    base_asset: value["base_ccy"].as_str().unwrap_or("").to_string(),
                    quote_asset: value["quote_ccy"].as_str().unwrap_or("").to_string(),
                    order_price_increment: Self::json_value_to_string(&value["price_tick_size"]),
                    order_quantity_increment: Self::json_value_to_string(&value["qty_tick_size"]),
                    order_quantity_min: Self::json_value_to_string(&value["qty_tick_size"]),
                    underlying_symbol: value["underlying_symbol"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    contract_size: Self::json_value_to_string(&value["contract_size"]),
                    ..Default::default()
                });
            }
        }
        crate::exchange_client::common::Response::GetInstrumentInfo(result)
    }

    fn create_get_top_of_book_rest_response(
        &self,
        response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut result = crate::exchange_client::common::GetTopOfBookResponse::default();
        if let Some(tickers) = response
            .json_payload
            .as_ref()
            .and_then(|value| value["result"]["data"].as_array())
        {
            result.data.reserve(tickers.len());
            for value in tickers {
                let timestamp = value["t"]
                    .as_i64()
                    .and_then(chrono::DateTime::<chrono::Utc>::from_timestamp_millis)
                    .unwrap_or_default();
                result.data.push(crate::types::TopOfBook {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Cryptocom,
                    symbol: value["i"].as_str().unwrap_or("").to_string(),
                    timestamp,
                    bid_price: Self::json_value_to_string(&value["b"]),
                    bid_size: Self::json_value_to_string(&value["bs"]),
                    ask_price: Self::json_value_to_string(&value["k"]),
                    ask_size: Self::json_value_to_string(&value["ks"]),
                });
            }
        }
        crate::exchange_client::common::Response::GetTopOfBook(result)
    }

    fn create_place_order_rest_response(
        &self,
        response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        crate::exchange_client::common::Response::PlaceOrder(
            crate::exchange_client::common::PlaceOrderResponse {
                order_id: response
                    .json_payload
                    .as_ref()
                    .map(|value| Self::json_value_to_string(&value["result"]["order_id"]))
                    .unwrap_or_default(),
            },
        )
    }

    fn create_cancel_order_rest_response(
        &self,
        _response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        crate::exchange_client::common::Response::CancelOrder(
            crate::exchange_client::common::CancelOrderResponse::default(),
        )
    }

    fn create_get_open_order_rest_response(
        &self,
        response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut result = crate::exchange_client::common::GetOpenOrderResponse::default();
        if let Some(orders) = response
            .json_payload
            .as_ref()
            .and_then(|value| value["result"]["data"].as_array())
        {
            result.data.reserve(orders.len());
            for value in orders {
                result.data.push(self.convert_json_value_to_order(value));
            }
        }
        crate::exchange_client::common::Response::GetOpenOrder(result)
    }

    fn create_get_position_rest_response(
        &self,
        response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut result = crate::exchange_client::common::GetPositionResponse::default();
        if let Some(positions) = response
            .json_payload
            .as_ref()
            .and_then(|value| value["result"]["data"].as_array())
        {
            result.data.reserve(positions.len());
            for value in positions {
                let quantity = Self::json_value_to_string(&value["quantity"]);
                let side = if quantity.starts_with('-') {
                    crate::types::PositionSide::Short
                } else if quantity.parse::<f64>().unwrap_or(0.0) > 0.0 {
                    crate::types::PositionSide::Long
                } else {
                    crate::types::PositionSide::Unknown
                };
                result.data.push(crate::types::Position {
                    exchange_instrument_type: crate::types::ExchangeInstrumentType::Cryptocom,
                    symbol: value["instrument_name"].as_str().unwrap_or("").to_string(),
                    side,
                    quantity,
                    position_asset: value["instrument_name"].as_str().unwrap_or("").to_string(),
                    ..Default::default()
                });
            }
        }
        crate::exchange_client::common::Response::GetPosition(result)
    }

    fn create_get_balance_rest_response(
        &self,
        response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut result = crate::exchange_client::common::GetBalanceResponse::default();
        if let Some(accounts) = response
            .json_payload
            .as_ref()
            .and_then(|value| value["result"]["data"].as_array())
        {
            for account in accounts {
                if let Some(balances) = account["position_balances"].as_array() {
                    for value in balances {
                        result.data.push(crate::types::Balance {
                            exchange: crate::types::Exchange::Cryptocom,
                            asset: value["instrument_name"].as_str().unwrap_or("").to_string(),
                            quantity: Self::json_value_to_string(&value["quantity"]),
                        });
                    }
                }
            }
        }
        crate::exchange_client::common::Response::GetBalance(result)
    }

    fn create_http_error_response(
        &self,
        response: &crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let mut result = response.clone();
        result.error_code = response
            .json_payload
            .as_ref()
            .and_then(|value| value["code"].as_i64())
            .map(|value| value.to_string());
        result.error_message = response
            .json_payload
            .as_ref()
            .and_then(|value| value["message"].as_str())
            .map(str::to_string);
        crate::exchange_client::common::Response::HttpErrorResponse(result)
    }
}

impl crate::exchanges::cryptocom::common::CryptocomClient {
    fn private_request(
        base_url: &str,
        path: &str,
        params: serde_json::Value,
    ) -> crate::networking::http::HttpRequest {
        crate::networking::http::HttpRequest::new(
            base_url,
            reqwest::Method::POST,
            path,
            None,
            None,
            Some(serde_json::json!({"params": params})),
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_get_position_rest_response_parses_positions() {
        let client = crate::exchanges::cryptocom::common::CryptocomClient::default();

        let response = crate::networking::http::HttpResponse {
            json_payload: Some(serde_json::json!({
                "id": "1",
                "method": "private/get-positions",
                "code": "0",
                "result": {
                    "data": [
                        {
                            "account_id": "858dbc8b-22fd-49fa-bff4-d342d98a8acb",
                            "quantity": "-0.1984",
                            "cost": "-10159.573500",
                            "open_position_pnl": "-497.743736",
                            "open_pos_cost": "-10159.352200",
                            "session_pnl": "2.236145",
                            "update_timestamp_ms": "1613552240770",
                            "instrument_name": "BTCUSD-PERP",
                            "type": "PERPETUAL_SWAP"
                        },
                        {
                            "account_id": "858dbc8b-22fd-49fa-bff4-d342d98a8acb",
                            "quantity": "-0.1984",
                            "cost": "-10159.573500",
                            "open_position_pnl": "-497.743736",
                            "open_pos_cost": "-10159.352200",
                            "session_pnl": "2.236145",
                            "update_timestamp_ms": "1613552240771",
                            "instrument_name": "BTCUSD-PERP",
                            "type": "PERPETUAL_SWAP",
                            "isolation_id": "19848526",
                            "isolation_type": "ISOLATED_MARGIN"
                        }
                    ]
                }
            })),
            ..Default::default()
        };

        let result =
            <crate::exchanges::cryptocom::common::CryptocomClient as crate::exchange_client::rest::Rest>::create_get_position_rest_response(
                &client, response,
            );

        match result {
            crate::exchange_client::common::Response::GetPosition(result) => {
                assert_eq!(result.data.len(), 2);

                let first = &result.data[0];
                assert_eq!(
                    first.exchange_instrument_type,
                    crate::types::ExchangeInstrumentType::Cryptocom
                );
                assert_eq!(first.symbol, "BTCUSD-PERP");
                assert_eq!(first.position_asset, "BTCUSD-PERP");
                assert_eq!(first.quantity, "-0.1984");
                assert_eq!(first.side, crate::types::PositionSide::Short);

                let second = &result.data[1];
                assert_eq!(
                    second.exchange_instrument_type,
                    crate::types::ExchangeInstrumentType::Cryptocom
                );
                assert_eq!(second.symbol, "BTCUSD-PERP");
                assert_eq!(second.position_asset, "BTCUSD-PERP");
                assert_eq!(second.quantity, "-0.1984");
                assert_eq!(second.side, crate::types::PositionSide::Short);
            }
            _ => panic!("expected Response::GetPosition variant"),
        }
    }
}
