#[async_trait::async_trait]
pub trait Rest {
    async fn create_http_client(
        &self,
        http_config: crate::networking::http::HttpConfig,
    ) -> anyhow::Result<crate::networking::http::HttpClient> {
        let http_client = crate::networking::http::HttpClient::builder(http_config).build()?;
        crate::finer!("Created http_client: {:#?}", http_client);
        Ok(http_client)
    }

    fn create_get_instrument_info_http_request(
        &self,
        get_instrument_info_request: &crate::exchange_client::common::GetInstrumentInfoRequest,
    ) -> crate::networking::http::HttpRequest;

    fn create_get_top_of_book_http_request(
        &self,
        get_top_of_book_request: &crate::exchange_client::common::GetTopOfBookRequest,
    ) -> crate::networking::http::HttpRequest;

    fn sign_http_request(
        &self,
        http_request: &mut crate::networking::http::HttpRequest,
        now: chrono::DateTime<chrono::Utc>,
    );

    fn create_place_order_http_request(
        &self,
        place_order_request: &crate::exchange_client::common::PlaceOrderRequest,
    ) -> crate::networking::http::HttpRequest;

    fn create_cancel_order_http_request(
        &self,
        cancel_order_request: &crate::exchange_client::common::CancelOrderRequest,
    ) -> crate::networking::http::HttpRequest;

    fn create_get_open_order_http_request(
        &self,
        get_open_order_request: &crate::exchange_client::common::GetOpenOrderRequest,
    ) -> crate::networking::http::HttpRequest;

    fn create_get_position_http_request(
        &self,
        get_position_order_request: &crate::exchange_client::common::GetPositionRequest,
    ) -> crate::networking::http::HttpRequest;

    fn create_get_balance_http_request(
        &self,
        get_balance_request: &crate::exchange_client::common::GetBalanceRequest,
    ) -> crate::networking::http::HttpRequest;

    fn is_http_response_success(
        &self,
        http_response: &crate::networking::http::HttpResponse,
    ) -> bool;

    fn create_get_instrument_info_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response;

    fn create_get_top_of_book_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response;

    fn create_place_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response;

    fn create_cancel_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response;

    fn create_get_open_order_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response;

    fn create_get_position_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response;

    fn create_get_balance_rest_response(
        &self,
        http_response: crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response;

    fn create_http_error_response(
        &self,
        http_response: &crate::networking::http::HttpResponse,
    ) -> crate::exchange_client::common::Response;

    async fn send_http_request(
        &self,
        http_client: &crate::networking::http::HttpClient,
        request: crate::exchange_client::common::Request,
    ) -> crate::exchange_client::common::Response {
        let now = chrono::Utc::now();
        let http_request = match &request {
            crate::exchange_client::common::Request::GetInstrumentInfo(
                get_instrument_info_request,
            ) => self.create_get_instrument_info_http_request(get_instrument_info_request),
            crate::exchange_client::common::Request::GetTopOfBook(get_top_of_book_request) => {
                self.create_get_top_of_book_http_request(get_top_of_book_request)
            }
            crate::exchange_client::common::Request::PlaceOrder(place_order_request) => {
                let mut http_request = self.create_place_order_http_request(place_order_request);
                self.sign_http_request(&mut http_request, now);
                http_request
            }
            crate::exchange_client::common::Request::CancelOrder(cancel_order_request) => {
                let mut http_request = self.create_cancel_order_http_request(cancel_order_request);
                self.sign_http_request(&mut http_request, now);
                http_request
            }
            crate::exchange_client::common::Request::GetOpenOrder(get_open_order_request) => {
                let mut http_request =
                    self.create_get_open_order_http_request(get_open_order_request);
                self.sign_http_request(&mut http_request, now);
                http_request
            }
            crate::exchange_client::common::Request::GetPosition(get_position_request) => {
                let mut http_request = self.create_get_position_http_request(get_position_request);
                self.sign_http_request(&mut http_request, now);
                http_request
            }
            crate::exchange_client::common::Request::GetBalance(get_balance_request) => {
                let mut http_request = self.create_get_balance_http_request(get_balance_request);
                self.sign_http_request(&mut http_request, now);
                http_request
            }
            _ => panic!(),
        };

        let http_response = match execute_http_request(http_client, http_request).await {
            Ok(http_response) => http_response,
            Err(err) => return crate::exchange_client::common::Response::HttpRequestError(err),
        };

        if !self.is_http_response_success(&http_response) {
            return self.create_http_error_response(&http_response);
        }

        match request {
            crate::exchange_client::common::Request::GetInstrumentInfo(
                _get_instrument_info_request,
            ) => self.create_get_instrument_info_rest_response(http_response),
            crate::exchange_client::common::Request::GetTopOfBook(_) => {
                self.create_get_top_of_book_rest_response(http_response)
            }
            crate::exchange_client::common::Request::PlaceOrder(_) => {
                self.create_place_order_rest_response(http_response)
            }
            crate::exchange_client::common::Request::CancelOrder(_) => {
                self.create_cancel_order_rest_response(http_response)
            }
            crate::exchange_client::common::Request::GetOpenOrder(_) => {
                self.create_get_open_order_rest_response(http_response)
            }
            crate::exchange_client::common::Request::GetPosition(_) => {
                self.create_get_position_rest_response(http_response)
            }
            crate::exchange_client::common::Request::GetBalance(_) => {
                self.create_get_balance_rest_response(http_response)
            }

            _ => panic!(),
        }
    }
}

pub async fn execute_http_request(
    http_client: &crate::networking::http::HttpClient,
    http_request: crate::networking::http::HttpRequest,
) -> anyhow::Result<crate::networking::http::HttpResponse> {
    let reqwest_request = http_request.to_request(http_client.client());

    crate::fine!("=== HTTP REQUEST ===");
    crate::fine!(
        "{} {} {}",
        http_request.method,
        http_request.base_url,
        http_request.path
    );
    crate::fine!("Headers: {:#?}", http_request.headers);
    if let Some(query_params) = http_request.query_params.as_ref() {
        crate::fine!("Query params: {:#?}", query_params);
    }
    if let Some(query_string) = http_request.query_string.as_ref() {
        crate::fine!("Query string: {:#?}", query_string);
    }
    if let Some(json_payload) = http_request.json_payload.as_ref() {
        crate::fine!("Json payload: {:#?}", json_payload);
    }
    if let Some(payload) = http_request.payload.as_ref() {
        crate::fine!("Payload: {:#?}", payload);
    }

    let reqwest_response = http_client.client().execute(reqwest_request).await?;

    let http_response =
        crate::networking::http::HttpResponse::from_response(reqwest_response, http_request).await;

    crate::fine!("=== HTTP RESPONSE ===");
    crate::fine!("Status: {}", http_response.status);
    crate::fine!("Headers: {:#?}", http_response.headers);
    crate::fine!("Body: {}", http_response.body);
    if let Some(json_payload) = http_response.json_payload.as_ref() {
        crate::fine!("Json payload: {:#?}", json_payload);
    }

    Ok(http_response)
}
