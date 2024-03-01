#[macro_export]
macro_rules! test_service_call {
    ( $service:expr,$method:expr,$api:expr,$payload:expr,$token:expr) => {{
        let mut parameters = if $method == "post" {
            test::TestRequest::post()
                .uri($api)
                .insert_header(header::ContentType::json())
        } else {
            test::TestRequest::get().uri($api)
        };

        if let Some(data) = $payload {
            parameters = parameters.set_payload(data);
        };

        if let Some(data) = $token {
            parameters =
                parameters.insert_header((header::AUTHORIZATION, format!("bearer {}", data)));
        };

        let req = parameters.to_request();
        let body = test::call_and_read_body(&$service, req)
            .await
            .try_into_bytes()
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        println!("body_str {}", body_str);
        serde_json::from_str::<_>(&body_str).unwrap()
    }};
}
