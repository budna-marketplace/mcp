use super::*;
const KEY: &str = "00000000-0000-4000-8000-000000000101";
#[tokio::test]
async fn parcel_submission_requires_budna_approval_and_a_separate_participant_review_scope()
-> Result<()> {
    let api = MockServer::start().await;
    exchange(&api, "parcel-read", "p-read", USER_A, "parcelreview:read").await;
    exchange(
        &api,
        "parcel-write",
        "p-write",
        USER_A,
        "parcelrequests:write",
    )
    .await;
    exchange(
        &api,
        "parcel-manage",
        "p-manage",
        USER_A,
        "parcelmanagement:write",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("parcelreview:read parcelrequests:write parcelmanagement:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let rev = "p".repeat(43);
    let price = |amount: &str| json!({"currency_code":"SEK","amount":amount});
    let member = |id: i64| json!({"order_id":id,"listing_id":LISTING_A,"quantity":1,"item_price":price("50.50"),"buyer_protection_fee":price("5"),"high_value_fee":price("0"),"current_payment_deadline":5000,"resulting_payment_deadline":6000});
    let members = json!([member(17), member(19)]);
    let sel = json!({"kind":"orders","order_ids":[17,19]});
    let mut terms = json!({"revision":rev,"reviewed_at":2000,"expires_at":3000,"group_id":null,"role":"buyer","seller_id":USER_B,"status":"awaiting_seller","members":members,
        "payment_deadline":6000,"seller_answer_deadline":4000,"package_size":null,"package_weight_grams":null,"custom_shipping_price":null,"shipping_cost":null,"matched_seller_rule":false,"can_quote":false,"can_decline":false,"can_dissolve":false,"payment_available_on_budna":false});
    terms["internal"] = json!(SENTINEL);
    Mock::given(method("POST"))
        .and(path("/api/v1/mcp/account/parcels/review"))
        .and(body_json(&sel))
        .and(matches_header("authorization", "Bearer p-read"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":terms})),
        )
        .expect(1)
        .mount(&api)
        .await;
    let data: Value = post(
        &client,
        &endpoint,
        "parcel-read",
        call("get_my_combined_parcel_terms", json!({"selection":sel})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(data["result"]["structuredContent"]["members"], members);
    assert!(!data.to_string().contains(SENTINEL));
    let input =
        json!({"request_id":KEY,"expected_revision":rev,"reviewed_at":2000,"order_ids":[17,19]});
    assert_eq!(
        post(
            &client,
            &endpoint,
            "parcel-read",
            call("request_my_combined_parcel", input.clone())
        )
        .await?
        .status(),
        StatusCode::FORBIDDEN
    );
    let mut invalid = input.clone();
    invalid["order_ids"] = json!([17, 17]);
    let bad: Value = post(
        &client,
        &endpoint,
        "parcel-write",
        call("request_my_combined_parcel", invalid),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(bad["result"]["isError"], true);
    Mock::given(method("POST")).and(path("/api/v1/mcp/account/parcels/request")).and(body_json(json!({"operation":"request","expected_revision":rev,"reviewed_at":2000,"order_ids":[17,19]}))).and(matches_header("authorization","Bearer p-write")).and(matches_header("idempotency-key",KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"action_id":KEY,"status":"awaiting_confirmation","action":"request_combined_parcel","group_id":null}}))).expect(1).mount(&api).await;
    let pending: Value = post(
        &client,
        &endpoint,
        "parcel-write",
        call("request_my_combined_parcel", input),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        pending["result"]["structuredContent"]["status"],
        "awaiting_confirmation"
    );
    assert!(
        pending["result"]["structuredContent"]["confirmation_url"]
            .as_str()
            .is_some_and(|url| url.ends_with(&format!("/approvals/{KEY}")))
    );
    let change = json!({"operation":"quote","group_id":7,"package_size":"small","package_weight_grams":650,"custom_shipping_price":price("19.50")});
    let req = json!({"request_id":KEY,"expected_revision":rev,"change":change});
    assert_eq!(
        post(
            &client,
            &endpoint,
            "parcel-write",
            call("change_my_combined_parcel", req.clone())
        )
        .await?
        .status(),
        StatusCode::FORBIDDEN
    );
    let mut wire = change;
    wire["expected_revision"] = json!(rev);
    Mock::given(method("POST")).and(path("/api/v1/mcp/account/parcels/manage")).and(body_json(wire)).and(matches_header("authorization","Bearer p-manage")).and(matches_header("idempotency-key",KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"action_id":KEY,"status":"executed","action":"quote_combined_parcel","group_id":7}}))).expect(1).mount(&api).await;
    let managed: Value = post(
        &client,
        &endpoint,
        "parcel-manage",
        call("change_my_combined_parcel", req),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        managed["result"]["structuredContent"],
        json!({"action_id":KEY,"action":"quote_combined_parcel","status":"executed","group_id":7,"confirmation_url":null})
    );
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
