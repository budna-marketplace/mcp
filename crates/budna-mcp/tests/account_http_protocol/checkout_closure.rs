use super::*;
#[tokio::test]
async fn checkout_and_closure_enforce_distinct_grants_and_exact_reviewed_amounts() -> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "buy",
        "buyer",
        USER_A,
        "orders:read checkoutorders:write",
    )
    .await;
    exchange(
        &api,
        "close",
        "closer",
        USER_B,
        "orders:read orderclosures:write",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("checkoutorders:write orderclosures:write ordersrelist:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::new();
    let m = |amount: &str| json!({"amount":amount,"currency_code":"SEK"});
    let q = "00000000-0000-4000-8000-000000000044";
    let sid = "00000000-0000-4000-8000-000000000045";
    let id = "00000000-0000-4000-8000-000000000046";
    let seal = "P".repeat(43);
    let data = json!({"listing_id":LISTING_A,"session_id":sid,"state":"needs_selection","order_id":null,"payment_deadline":null});
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/mcp/account/listings/{LISTING_A}/binding-checkout"
        )))
        .and(matches_header("authorization", "Bearer buyer"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":data})))
        .expect(1)
        .mount(&api)
        .await;
    let response: Value = post(
        &client,
        &endpoint,
        "buy",
        call("get_my_binding_checkout", json!({"listing_id":LISTING_A})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(response.pointer("/result/structuredContent"), Some(&data));
    let reviewed = json!({"quote_id":q,"listing_id":LISTING_A,"session_id":sid,"revision":"R".repeat(43),"quantity":1,"item_price":m("129.95"),"fulfillment_mode":"carrier_tracked","shipping_cost":m("30"),"buyer_protection_applied":true,"buyer_protection_fee":m("5.05"),"high_value_fee":m("0"),"total":m("165"),"unpaid_fee":m("25"),"quoted_at":1_900_000_000_000_i64,"payment_deadline":1_900_200_000_000_i64,"ends_auction":true,"expires_at":1_900_000_100_000_i64});
    Mock::given(method("POST"))
        .and(path(format!(
            "/api/v1/mcp/account/listings/{LISTING_A}/binding-checkout/quote"
        )))
        .and(body_json(json!({"session_id":sid})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"success":true,"data":{"terms":reviewed,"terms_digest":seal}}),
            ),
        )
        .expect(1)
        .mount(&api)
        .await;
    let response: Value = post(
        &client,
        &endpoint,
        "buy",
        call(
            "prepare_my_binding_checkout_order",
            json!({"listing_id":LISTING_A,"session_id":sid}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        response.pointer("/result/structuredContent/total"),
        Some(&m("165"))
    );
    let args = json!({"listing_id":LISTING_A,"quote_id":q,"session_id":sid,"terms_digest":seal});
    let denied = post(
        &client,
        &endpoint,
        "close",
        call("submit_my_quoted_checkout_order", args.clone()),
    )
    .await?;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    Mock::given(method("POST")).and(path(format!("/api/v1/mcp/account/listings/{LISTING_A}/binding-checkout/order"))).and(matches_header("idempotency-key",q)).and(matches_header("authorization","Bearer buyer")).and(body_json(json!({"session_id":sid,"quote_id":q,"terms_digest":seal}))).respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"action_id":id,"status":"awaiting_confirmation","listing_id":LISTING_A,"session_id":sid,"order_id":null,"payment_deadline":null}}))).expect(1).mount(&api).await;
    let response: Value = post(
        &client,
        &endpoint,
        "buy",
        call("submit_my_quoted_checkout_order", args),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        response.pointer("/result/structuredContent/status"),
        Some(&json!("awaiting_confirmation"))
    );
    assert!(
        response
            .pointer("/result/structuredContent/confirmation_url")
            .and_then(Value::as_str)
            .is_some_and(|s| s.ends_with(id))
    );
    let closure = json!({"order_id":77,"listing_id":LISTING_A,"operation":"close_overdue_order","quote_id":q,"revision":"P".repeat(43),"role":"seller","order_origin":"auction_win","order_status":"pending","payment_status":"pending","legacy_terms":false,"payment_due_at":1_800_000_000_000_i64,"payment_deadline":1_800_020_000_000_i64,"debt_payer":"buyer","debt_amount":m("35"),"refund_route":"none","refund_amount":null,"retained_partial":null,"restores_stock":false,"ends_order":true,"starts_listing":false,"expires_at":1_900_000_090_000_i64});
    Mock::given(method("POST"))
        .and(path("/api/v1/mcp/account/orders/77/closure/quote"))
        .and(body_json(json!({"operation":"close_overdue_order"})))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"success":true,"data":{"terms":closure,"terms_digest":seal}}),
            ),
        )
        .expect(1)
        .mount(&api)
        .await;
    let response: Value = post(
        &client,
        &endpoint,
        "close",
        call(
            "prepare_my_order_closure",
            json!({"order_id":77,"operation":"close_overdue_order"}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        response.pointer("/result/structuredContent/debt_amount"),
        Some(&m("35"))
    );
    assert_eq!(
        response.pointer("/result/structuredContent/debt_payer"),
        Some(&json!("buyer"))
    );
    let args =
        json!({"order_id":77,"operation":"close_overdue_order","quote_id":q,"terms_digest":seal});
    let denied = post(
        &client,
        &endpoint,
        "buy",
        call("submit_my_quoted_order_closure", args.clone()),
    )
    .await?;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    Mock::given(method("POST")).and(path("/api/v1/mcp/account/orders/77/closure/actions")).and(matches_header("idempotency-key",q)).and(matches_header("authorization","Bearer closer")).and(body_json(json!({"operation":"close_overdue_order","quote_id":q,"terms_digest":seal}))).respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"action_id":id,"status":"awaiting_confirmation","order_id":77,"operation":"close_overdue_order"}}))).expect(1).mount(&api).await;
    let response: Value = post(
        &client,
        &endpoint,
        "close",
        call("submit_my_quoted_order_closure", args),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        response.pointer("/result/structuredContent/status"),
        Some(&json!("awaiting_confirmation"))
    );
    let denied = post(
        &client,
        &endpoint,
        "close",
        call(
            "relist_my_quoted_closed_order",
            json!({"order_id":77,"quote_id":q,"terms_digest":seal}),
        ),
    )
    .await?;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
