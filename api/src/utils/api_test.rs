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

#[macro_export]
macro_rules! test_register {
    ( $service:expr,$app:expr) => {{
            let payload = json!({
                "deviceId":  $app.device.id,
                "contact": $app.user.contact,
                "kind": "register"
            });
            let res: BackendRespond<String> = test_service_call!(
                $service,
                "post",
                "/accountManager/getCaptcha",
                Some(payload.to_string()),
                None::<String>
            );

            let payload = json!({
                "deviceId":  $app.device.id,
                "deviceBrand": $app.device.brand,
                "email": $app.user.contact,
                "captcha": $app.device.id,
                "password": $app.user.password    
            });

            let res: BackendRespond<String> = test_service_call!(
                $service,
                "post",
                "/accountManager/registerByEmail",
                Some(payload.to_string()),
                None::<String>
            );
            $app.user.token = Some(res.data);
    }};
}


#[macro_export]
macro_rules! test_login {
    ($service:expr, $app:expr) => {{
            let payload = json!({
                "deviceId":  $app.device.id,
                "deviceBrand": $app.device.brand,
                "contact": $app.user.contact,
                "password": $app.user.password    
            });
            let res: BackendRespond<String> = test_service_call!(
                $service,
                "post",
                "/accountManager/login",
                Some(payload.to_string()),
                None::<String>
            );
            $app.user.token = Some(res.data);     
    }};
}

#[macro_export]
macro_rules! test_create_main_account{
    ($service:expr, $app:expr) => {{
        let payload = json!({
            "masterPubkey":  $app.wallet.main_account,
            "masterPrikeyEncryptedByPwd": $app.wallet.prikey,
            "masterPrikeyEncryptedByAnswer": $app.wallet.prikey,
            "subaccountPubkey":  $app.wallet.subaccount.first().unwrap(),
            "subaccountPrikeyEncrypedByPwd": $app.wallet.sub_prikey.as_ref().unwrap().first().unwrap(),
            "subaccountPrikeyEncrypedByAnswer": $app.wallet.sub_prikey.unwrap().first().unwrap(),
            "signPwdHash": ""
        });
        let res: BackendRespond<String> = test_service_call!(
            $service,
            "post",
            "/wallet/createMainAccount",
            Some(payload.to_string()),
            Some(&$app.user.token.unwrap())
        );
    }};
}


#[macro_export]
macro_rules! test_search_message{
    ($service:expr, $app:expr) => {{
        let url = format!("/wallet/searchMessage");
        let res: BackendRespond<Vec<AccountMessage>> = test_service_call!(
            $service,
            "get",
            &url,
            None::<String>,
            Some($app.user.token.as_ref().unwrap())
        );
        res
    }};
}


#[macro_export]
macro_rules! test_get_strategy{
    ($service:expr, $app:expr) => {{
        let url = format!("/wallet/getStrategy?accountId={}", $app.wallet.main_account);
        let res: BackendRespond<StrategyDataTmp> = test_service_call!(
            $service,
            "get",
            &url,
            None::<String>,
            Some($app.user.token.as_ref().unwrap())
        );
        res
    }};
}

