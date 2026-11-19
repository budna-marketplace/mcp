use super::*;
const KEY: &str = "00000000-0000-4000-8000-000000000107";

#[tokio::test]
async fn delivery_review_booking_dispatch_and_owner_receipt_are_independent() -> Result<()> {
    let api = MockServer::start().await;
    for (token, scope) in [
        ("review", "fulfillmentreview:read"),
        ("seller", "sellerfulfillment:write"),
        ("book", "carrierbookings:write"),
        ("buyer", "buyerreceipts:write"),
    ] {
        exchange(&api, token, &format!("fulfillment-{token}"), USER_A, scope).await;
    }
    let mut server=start_http_server_with_scopes(&api.uri(),30,Some("account-actions"),Some(false),Some("fulfillmentreview:read sellerfulfillment:write carrierbookings:write buyerreceipts:write")).await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let revision = "f".repeat(43);
    let items =
        json!([{"order_id":17,"listing_id":LISTING_A},{"order_id":19,"listing_id":LISTING_A}]);
    Mock::given(method("POST")).and(path("/api/v1/mcp/account/orders/17/fulfillment-review")).and(matches_header("authorization","Bearer fulfillment-review")).and(body_json(json!({"operation":"confirm_meetup_receipt"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"order_id":17,"group_id":4,"operation":"confirm_meetup_receipt","eligible":true,"revision":revision,"members":items,
        "has_tracking":false,"carrier":null,"funded_balance":{"amount":"100.50","currency_code":"SEK"},"release_delay_hours":0,"handover_window_days":null,"closable_dispute_ids":[12],"internal":SENTINEL}}))).expect(1).mount(&api).await;
    let output: Value = post(
        &client,
        &endpoint,
        "review",
        call(
            "get_my_fulfillment_terms",
            json!({"order_id":17,"operation":"confirm_meetup_receipt"}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    let data = &output["result"]["structuredContent"];
    assert_eq!(data["release_delay_hours"], 0);
    assert_eq!(data["closable_dispute_ids"], json!([12]));
    assert_eq!(data["members"], items);
    assert!(!data.to_string().contains(SENTINEL));
    for (credential, tool, fields) in [
        (
            "review",
            "confirm_my_order_receipt",
            json!({"operation":"confirm_meetup_receipt"}),
        ),
        (
            "book",
            "update_my_seller_fulfillment",
            json!({"change":{"operation":"mark_order_dispatched"}}),
        ),
        ("seller", "book_my_order_shipment", json!({})),
    ] {
        let mut args = json!({"order_id":17,"expected_revision":revision,"request_id":KEY});
        args.as_object_mut()
            .context("synthetic tool arguments")?
            .extend(fields.as_object().context("synthetic fields")?.clone());
        assert_eq!(
            post(&client, &endpoint, credential, call(tool, args))
                .await?
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    for (credential, tool, op, params, state) in [
        (
            "book",
            "book_my_order_shipment",
            "book_order_shipment",
            json!({}),
            "executed",
        ),
        (
            "buyer",
            "confirm_my_order_receipt",
            "confirm_meetup_receipt",
            json!({"operation":"confirm_meetup_receipt"}),
            "awaiting_confirmation",
        ),
        (
            "seller",
            "update_my_seller_fulfillment",
            "mark_order_dispatched",
            json!({"change":{"operation":"mark_order_dispatched"}}),
            "executed",
        ),
    ] {
        Mock::given(method("POST")).and(path("/api/v1/mcp/account/orders/17/fulfillment-actions")).and(matches_header("idempotency-key",KEY))
            .and(matches_header("authorization",&format!("Bearer fulfillment-{credential}"))).and(body_json(json!({"expected_revision":revision,"operation":op,"tracking":null})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"order_id":17,"operation":op,"action_id":KEY,"status":state,"provider_payload":SENTINEL}}))).expect(1).mount(&api).await;
        let mut args = json!({"order_id":17,"expected_revision":revision,"request_id":KEY});
        args.as_object_mut()
            .context("synthetic tool arguments")?
            .extend(params.as_object().context("synthetic fields")?.clone());
        let output: Value = post(&client, &endpoint, credential, call(tool, args))
            .await?
            .error_for_status()?
            .json()
            .await?;
        let receipt = &output["result"]["structuredContent"];
        assert_eq!(receipt["status"], state);
        assert_eq!(receipt["operation"], op);
        assert!(!receipt.to_string().contains(SENTINEL));
        assert_eq!(
            receipt["confirmation_url"].is_string(),
            credential == "buyer"
        );
    }
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
