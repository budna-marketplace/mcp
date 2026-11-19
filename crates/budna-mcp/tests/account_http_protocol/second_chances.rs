use super::*;
#[tokio::test]
async fn a_second_chance_answer_requires_the_invited_buyers_separate_grant_and_reference()
-> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "buyer",
        "invited",
        USER_A,
        "offers:read secondchanceanswers:write",
    )
    .await;
    exchange(
        &api,
        "seller",
        "owner",
        USER_B,
        "orders:read secondchances:write",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("secondchances:write secondchanceanswers:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::new();
    let m = |n: &str| json!({"amount":n,"currency_code":"SEK"});
    let q = "00000000-0000-4000-8000-000000000031";
    let receipt = "00000000-0000-4000-8000-000000000032";
    let seal = "A".repeat(43);
    let revision = "B".repeat(43);
    let invitation = json!({"offer_id":38,"invitation_id":55,"listing_id":LISTING_A,"price":m("155"),"window_ends_at":2000000000000_i64});
    Mock::given(method("GET"))
        .and(path("/api/v1/mcp/account/second-chances/38/invitation"))
        .and(matches_header("authorization", "Bearer invited"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":invitation})),
        )
        .expect(1)
        .mount(&api)
        .await;
    let r: Value = post(
        &client,
        &endpoint,
        "buyer",
        call("get_my_second_chance_invitation", json!({"offer_id":38})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(r.pointer("/result/structuredContent"), Some(&invitation));
    let change = json!({"operation":"accept_second_chance","offer_id":38,"invitation_id":55});
    let terms = json!({"quote_id":q,"change":change,"listing_id":LISTING_A,"revision":revision,"price":m("155"),"maximum_seller_price":null,
        "current_window_ends_at":2000000000000_i64,"response_window_hours":24,"payment_due_hours":48,"payment_grace_hours":0,"buyer_protection_fee":m("10"),"buyer_protection_mandatory":true,
        "high_value_fee":m("0"),"unpaid_fee":m("30"),"delivery_at_checkout":true,"expires_at":1999999900000_i64});
    Mock::given(method("POST"))
        .and(path("/api/v1/mcp/account/second-chances/quote"))
        .and(body_json(&change))
        .and(matches_header("authorization", "Bearer invited"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"success":true,"data":{"terms":terms,"terms_digest":seal}})),
        )
        .expect(1)
        .mount(&api)
        .await;
    let r: Value = post(
        &client,
        &endpoint,
        "buyer",
        call("prepare_my_second_chance_answer", json!({"change":change})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        r.pointer("/result/structuredContent/quote_id"),
        Some(&json!(q))
    );
    let args = json!({"quote_id":q,"terms_digest":seal,"change":change});
    let denied = post(
        &client,
        &endpoint,
        "seller",
        call("submit_my_quoted_second_chance_answer", args.clone()),
    )
    .await?;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    let r = json!({"action_id":receipt,"status":"executed","operation":"accept_second_chance","source_order_id":null,"invitation_id":55,"offer_id":38,"order_id":99,"order_status":"pending"});
    Mock::given(method("POST"))
        .and(path("/api/v1/mcp/account/second-chances/actions"))
        .and(body_json(&args))
        .and(matches_header("authorization", "Bearer invited"))
        .and(matches_header("idempotency-key", q))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":r})))
        .expect(1)
        .mount(&api)
        .await;
    let r: Value = post(
        &client,
        &endpoint,
        "buyer",
        call("submit_my_quoted_second_chance_answer", args),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        r.pointer("/result/structuredContent/order_id"),
        Some(&json!(99))
    );
    assert_eq!(
        r.pointer("/result/structuredContent/order_status"),
        Some(&json!("pending"))
    );
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
