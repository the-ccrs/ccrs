#[async_trait::async_trait]
impl crate::exchange_client::rest::Rest
    for crate::exchanges::hyperliquid::common::HyperliquidClient
{
    fn create_get_instrument_info_http_request(
        &self,
        _get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest {
        let body = match self.instrument_type {
            crate::types::HyperliquidInstrumentType::Spot => {
                serde_json::json!({ "type": "spotMeta" })
            }
            crate::types::HyperliquidInstrumentType::Perpetuals => {
                serde_json::json!({ "type": "meta" })
            }
            crate::types::HyperliquidInstrumentType::Unknown => {
                panic!("HyperliquidInstrumentType::Unknown is not allowed here")
            }
        };
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/info",
            None,
            None,
            Some(body),
        )
    }

    fn create_get_top_of_book_http_request(
        &self,
        get_top_of_book_request: &crate::exchange_client::common::GetTopOfBookRequest,
    ) -> crate::networking::http::HttpRequest {
        let body = serde_json::json!({
            "type": "l2Book",
            "coin": get_top_of_book_request.symbol,
        });
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/info",
            None,
            None,
            Some(body),
        )
    }

    fn sign_http_request(
        &self,
        http_request: &mut crate::networking::http::HttpRequest,
        _: chrono::DateTime<chrono::Utc>,
    ) {
        let json = match http_request.json_payload.as_ref() {
            Some(j) => j,
            None => return,
        };
        let action = match json.get("action") {
            Some(a) => a.clone(),
            None => return,
        };
        let nonce = crate::exchange_client::common::Common::generate_next_nonce(self);
        let signature = self.sign_action(&action, nonce);
        let mut new_json = serde_json::Map::new();
        new_json.insert("action".to_string(), action);
        new_json.insert("nonce".to_string(), serde_json::json!(nonce));
        new_json.insert("signature".to_string(), signature);
        let new_json_value = serde_json::Value::Object(new_json);
        let new_payload = serde_json::to_string(&new_json_value).unwrap();
        http_request.json_payload = Some(new_json_value);
        http_request.payload = Some(new_payload);
    }

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let asset_index = place_order_request.symbol.parse::<u32>().unwrap();

        let is_buy = self.convert_order_side_to_bool(place_order_request.side);
        let tif = match place_order_request.order_type {
            crate::types::OrderType::Market => "Ioc",
            crate::types::OrderType::Limit => "Gtc",
            crate::types::OrderType::Unknown => panic!("Invalid order type"),
        };

        let price = if place_order_request.price.is_empty() {
            panic!("Price must not be empty");
        } else {
            crate::utils::remove_trailing_zeros(&place_order_request.price)
        };

        let quantity = crate::utils::remove_trailing_zeros(&place_order_request.quantity);

        let mut order_map = serde_json::Map::new();
        order_map.insert("a".to_string(), serde_json::json!(asset_index));
        order_map.insert("b".to_string(), serde_json::json!(is_buy));
        order_map.insert("p".to_string(), serde_json::json!(price));
        order_map.insert("s".to_string(), serde_json::json!(quantity));
        order_map.insert("r".to_string(), serde_json::json!(false));
        order_map.insert(
            "t".to_string(),
            serde_json::json!({ "limit": { "tif": tif } }),
        );
        if !place_order_request.client_order_id.is_empty() {
            order_map.insert(
                "c".to_string(),
                serde_json::json!(place_order_request.client_order_id),
            );
        }

        let mut action_map = serde_json::Map::new();
        action_map.insert("type".to_string(), serde_json::json!("order"));
        action_map.insert(
            "orders".to_string(),
            serde_json::Value::Array(vec![serde_json::Value::Object(order_map)]),
        );
        action_map.insert("grouping".to_string(), serde_json::json!("na"));

        let body = serde_json::json!({
            "action": serde_json::Value::Object(action_map)
        });

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/exchange",
            None,
            None,
            Some(body),
        )
    }

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let asset_index = cancel_order_request.symbol.parse::<u32>().unwrap();

        let (action_type, cancel_entry) = if !cancel_order_request.order_id.is_empty() {
            let oid: u64 = cancel_order_request
                .order_id
                .parse()
                .expect("order_id must be a numeric oid for Hyperliquid");
            let mut entry = serde_json::Map::new();
            entry.insert("a".to_string(), serde_json::json!(asset_index));
            entry.insert("o".to_string(), serde_json::json!(oid));
            ("cancel", serde_json::Value::Object(entry))
        } else {
            let mut entry = serde_json::Map::new();
            entry.insert("asset".to_string(), serde_json::json!(asset_index));
            entry.insert(
                "cloid".to_string(),
                serde_json::json!(cancel_order_request.client_order_id),
            );
            ("cancelByCloid", serde_json::Value::Object(entry))
        };

        let mut action_map = serde_json::Map::new();
        action_map.insert("type".to_string(), serde_json::json!(action_type));
        action_map.insert(
            "cancels".to_string(),
            serde_json::Value::Array(vec![cancel_entry]),
        );

        let body = serde_json::json!({
            "action": serde_json::Value::Object(action_map)
        });

        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/exchange",
            None,
            None,
            Some(body),
        )
    }

    fn create_get_open_order_http_request(
        &self,
        _get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest {
        let user = &self.wallet_address;
        let body = serde_json::json!({
            "type": "frontendOpenOrders",
            "user": user
        });
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/info",
            None,
            None,
            Some(body),
        )
    }

    fn create_get_position_http_request(
        &self,
        _get_position_request: &crate::exchange_client::common::GetPositionRequest,
    ) -> crate::networking::http::HttpRequest {
        let user = &self.wallet_address;
        let body = serde_json::json!({ "type": "clearinghouseState", "user": user });
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/info",
            None,
            None,
            Some(body),
        )
    }

    fn create_get_balance_http_request(
        &self,
        _get_balance_request: &crate::exchange_client::common::GetBalanceRequest,
    ) -> crate::networking::http::HttpRequest {
        let body =
            serde_json::json!({ "type": "spotClearinghouseState", "user": self.wallet_address });
        crate::networking::http::HttpRequest::new(
            &self.rest_api_base_url,
            reqwest::Method::POST,
            "/info",
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
            return false;
        };
        if json.get("status").and_then(|s| s.as_str()) == Some("err") {
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

        match self.instrument_type {
            crate::types::HyperliquidInstrumentType::Spot => {
                let tokens = match json_payload.get("tokens").and_then(|v| v.as_array()) {
                    Some(t) => t.clone(),
                    None => {
                        return crate::exchange_client::common::Response::GetInstrumentInfo(
                            response,
                        );
                    }
                };
                let universe = match json_payload.get("universe").and_then(|v| v.as_array()) {
                    Some(u) => u,
                    None => {
                        return crate::exchange_client::common::Response::GetInstrumentInfo(
                            response,
                        );
                    }
                };

                response.data.reserve(universe.len());
                for item in universe.iter() {
                    let name = item["name"].as_str().unwrap_or("").to_string();
                    let pair_tokens = match item["tokens"].as_array() {
                        Some(t) => t,
                        None => continue,
                    };
                    let base_idx =
                        pair_tokens.first().and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let quote_idx =
                        pair_tokens.get(1).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    let base_token = tokens.get(base_idx);
                    let quote_token = tokens.get(quote_idx);
                    let base_asset = base_token
                        .and_then(|t| t["name"].as_str())
                        .unwrap_or("")
                        .to_string();
                    let quote_asset = quote_token
                        .and_then(|t| t["name"].as_str())
                        .unwrap_or("")
                        .to_string();
                    let sz_decimals = base_token
                        .and_then(|t| t["szDecimals"].as_u64())
                        .unwrap_or(0);
                    let qty_increment = format!("1e-{}", sz_decimals);
                    response.data.push(crate::types::InstrumentInfo {
                        exchange_instrument_type: crate::types::ExchangeInstrumentType::Hyperliquid(
                            self.instrument_type,
                        ),
                        symbol: name,
                        base_asset,
                        quote_asset,
                        order_quantity_increment: qty_increment,
                        ..Default::default()
                    });
                }
            }

            crate::types::HyperliquidInstrumentType::Perpetuals => {
                let universe = match json_payload.get("universe").and_then(|v| v.as_array()) {
                    Some(u) => u,
                    None => {
                        return crate::exchange_client::common::Response::GetInstrumentInfo(
                            response,
                        );
                    }
                };

                response.data.reserve(universe.len());
                for item in universe.iter() {
                    let name = item["name"].as_str().unwrap_or("").to_string();
                    let sz_decimals = item["szDecimals"].as_u64().unwrap_or(0);
                    let qty_increment = format!("1e-{}", sz_decimals);
                    response.data.push(crate::types::InstrumentInfo {
                        exchange_instrument_type: crate::types::ExchangeInstrumentType::Hyperliquid(
                            self.instrument_type,
                        ),
                        symbol: name.clone(),
                        base_asset: name,
                        quote_asset: "USDC".to_string(),
                        order_quantity_increment: qty_increment,
                        ..Default::default()
                    });
                }
            }

            crate::types::HyperliquidInstrumentType::Unknown => {
                panic!("HyperliquidInstrumentType::Unknown is not allowed here")
            }
        }

        crate::exchange_client::common::Response::GetInstrumentInfo(response)
    }

    fn create_get_top_of_book_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();
        let mut response = crate::exchange_client::common::GetTopOfBookResponse::default();

        let coin = json_payload["coin"].as_str().unwrap_or("").to_string();
        let timestamp = crate::utils::convert_unix_timestamp_milliseconds_to_timestamp(
            json_payload["time"].as_i64().unwrap_or(0),
        );

        let levels = match json_payload.get("levels").and_then(|v| v.as_array()) {
            Some(l) => l,
            None => return crate::exchange_client::common::Response::GetTopOfBook(response),
        };

        let bids = levels.first().and_then(|v| v.as_array());
        let asks = levels.get(1).and_then(|v| v.as_array());

        let best_bid = bids.and_then(|b| b.first());
        let best_ask = asks.and_then(|a| a.first());

        let bid_price = best_bid
            .and_then(|b| b["px"].as_str())
            .unwrap_or("")
            .to_string();
        let bid_size = best_bid
            .and_then(|b| b["sz"].as_str())
            .unwrap_or("")
            .to_string();
        let ask_price = best_ask
            .and_then(|a| a["px"].as_str())
            .unwrap_or("")
            .to_string();
        let ask_size = best_ask
            .and_then(|a| a["sz"].as_str())
            .unwrap_or("")
            .to_string();

        response.data.push(crate::types::TopOfBook {
            exchange_instrument_type: crate::types::ExchangeInstrumentType::Hyperliquid(
                self.convert_symbol_to_instrument_type(&coin),
            ),
            symbol: coin,
            timestamp,
            bid_price,
            bid_size,
            ask_price,
            ask_size,
        });

        crate::exchange_client::common::Response::GetTopOfBook(response)
    }

    fn create_place_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response {
        let json_payload = http_response.json_payload.unwrap();
        let statuses = json_payload["response"]["data"]["statuses"]
            .as_array()
            .and_then(|a| a.first());
        let order_id = statuses
            .and_then(|s| s.get("resting"))
            .and_then(|r| r["oid"].as_u64())
            .or_else(|| {
                statuses
                    .and_then(|s| s.get("filled"))
                    .and_then(|f| f["oid"].as_u64())
            })
            .unwrap_or(0)
            .to_string();
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
        let json_payload = http_response.json_payload.unwrap();
        let mut response = crate::exchange_client::common::GetOpenOrderResponse::default();
        if let Some(list) = json_payload.as_array() {
            response.data = list
                .iter()
                .map(|item| self.convert_json_value_to_order(item, crate::types::OrderStatus::Open))
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
        if let Some(positions) = json_payload["assetPositions"].as_array() {
            response.data = positions
                .iter()
                .filter(|item| {
                    let szi = item["position"]["szi"].as_str().unwrap_or("0");
                    let v: f64 = szi.parse().unwrap_or(0.0);
                    v != 0.0
                })
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
        if let Some(balances) = json_payload["balances"].as_array() {
            response.data = balances
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
        new_http_response.error_message = json_payload
            .get("response")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| {
                json_payload
                    .get("error")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            });
        crate::exchange_client::common::Response::HttpErrorResponse(new_http_response)
    }
}
