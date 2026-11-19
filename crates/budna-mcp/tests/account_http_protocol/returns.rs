use super::*;
const KEY: &str = "00000000-0000-4000-8000-000000000101";
#[tokio::test]
async fn return_rights_use_scoped_order_terms_and_approval_or_tracking_receipts() -> Result<()> {
    let api = MockServer::start().await;
    exchange(&api, "return-read", "r-read", USER_A, "returnreview:read").await;
    exchange(&api, "return-change", "r-write", USER_A, "returns:write").await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("returnreview:read returns:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let rev = "r".repeat(43);
    let money = json!({"amount":"100.25","currency_code":"SEK"});
    let terms = json!({"order_id":7,"revision":rev,"listing_id":LISTING_A,"role":"buyer","return_kind":"statutory","status":"requested",
        "consumer_buyer":true,"business_seller":true,"order_amount":money,"refundable_amount":money,"refund_via":"budna","pre_dispatch":false,"return_shipping_payer":"buyer",
        "withdrawal_deadline":8000,"online_refund_deadline":7000,"has_own_tracking":false,"budna_label_booked":false,"buyer_protection_label":false,
        "can_request_withdrawal":false,"can_accept_agreed":false,"can_confirm_received":false,"can_cancel":true,"can_set_tracking":true});
    let mut wire = terms.clone();
    wire["extra"] = json!(SENTINEL);
    Mock::given(method("GET"))
        .and(path("/api/v1/mcp/account/orders/7/return-actions"))
        .and(matches_header("authorization", "Bearer r-read"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":wire})))
        .expect(1)
        .mount(&api)
        .await;
    let result: Value = post(
        &client,
        &endpoint,
        "return-read",
        call("get_my_return_terms", json!({"order_id":7})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    let mut expected = terms.clone();
    expected["budna_order_url"] = json!("https://budna.se/orders/7");
    let out = &result["result"]["structuredContent"];
    assert!(
        out["budna_order_url"]
            .as_str()
            .is_some_and(|url| url.ends_with("/orders/7"))
    );
    expected["budna_order_url"] = out["budna_order_url"].clone();
    assert_eq!(out, &expected);
    let op = "change_my_order_return";
    let input = |change: Value| json!({"order_id":7,"expected_revision":rev,"request_id":KEY,"change":change});
    assert_eq!(
        post(
            &client,
            &endpoint,
            "return-read",
            call(op, input(json!({"operation":"cancel"})))
        )
        .await?
        .status(),
        StatusCode::FORBIDDEN
    );
    for bad in [
        json!({"operation":"cancel","order_id":8}),
        json!({"operation":"set_tracking","carrier":"other","tracking_number":" "}),
    ] {
        let r: Value = post(&client, &endpoint, "return-change", call(op, input(bad)))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_eq!(r["result"]["isError"], true);
    }
    for (change, action, status) in [
        (
            json!({"operation":"request_withdrawal"}),
            "request_statutory_withdrawal",
            "awaiting_confirmation",
        ),
        (
            json!({"operation":"set_tracking","tracking_number":"TEST-123","carrier":"other"}),
            "set_own_return_tracking",
            "executed",
        ),
    ] {
        let mut payload = change.clone();
        payload["order_id"] = json!(7);
        payload["expected_revision"] = json!(rev);
        Mock::given(method("POST")).and(path("/api/v1/mcp/account/returns/change")).and(body_json(payload)).and(matches_header("authorization","Bearer r-write")).and(matches_header("idempotency-key",KEY)).respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"order_id":7,"action_id":KEY,"action":action,"status":status,"internal":SENTINEL}}))).expect(1).mount(&api).await;
        let result: Value = post(&client, &endpoint, "return-change", call(op, input(change)))
            .await?
            .error_for_status()?
            .json()
            .await?;
        let out = &result["result"]["structuredContent"];
        assert_eq!(out["status"], status);
        assert_eq!(out["action"], action);
        assert_eq!(out["order_id"], 7);
        assert_eq!(
            out["confirmation_url"].is_string(),
            status == "awaiting_confirmation"
        );
        assert!(!result.to_string().contains(SENTINEL));
        assert!(!result.to_string().contains("TEST-123"));
    }
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
