use super::*;
#[tokio::test]
async fn negotiation_quotes_preserve_unit_totals_and_require_offer_consent_for_exact_submission()
-> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "trader",
        "exact-grant",
        USER_A,
        "offers:read offers:write",
    )
    .await;
    exchange(&api, "reader", "read-grant", USER_B, "offers:read").await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("offers:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::new();
    let q = "00000000-0000-4000-8000-000000000081";
    let act = "00000000-0000-4000-8000-000000000082";
    let rev = "a".repeat(43);
    let digest = "b".repeat(43);
    let m = |v: &str| json!({"amount":v,"currency_code":"SEK"});
    let p = json!({"operation":"make_offer","quantity":2,"total":m("130.00")});
    let terms = json!({"quote_id":q,"listing_id":LISTING_A,"revision":rev,"proposal":{"operation":"make_offer","quantity":2,"offer_id":null,"total":m("130")},"role":"buyer","quantity":2,"total":m("130"),"standing_buyer_total":null,"standing_seller_total":null,"remaining_buyer_total":m("130"),"remaining_seller_total":null,"next_responder":"seller","current_offer_expires_at":null,"response_window_hours":72,"buyer_protection_fee":m("10"),"buyer_protection_mandatory":true,"high_value_fee":m("0"),"unpaid_fee":m("30"),"payment_due_hours":48,"payment_grace_hours":24,"delivery_at_checkout":true,"reserves_stock":false,"expires_at":2000000000000_i64});
    let base = format!("/api/v1/mcp/account/listings/{LISTING_A}/offer-action");
    Mock::given(method("POST"))
        .and(path(format!("{base}-quote")))
        .and(body_json(&p))
        .and(matches_header("authorization", "Bearer exact-grant"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"success":true,"data":{"terms":terms,"terms_digest":digest}}),
            ),
        )
        .expect(1)
        .mount(&api)
        .await;
    let view: Value = post(
        &client,
        &endpoint,
        "trader",
        call(
            "prepare_my_offer_change",
            json!({"listing_id":LISTING_A,"proposal":p}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        view.pointer("/result/structuredContent/total_for_all_units"),
        Some(&m("130"))
    );
    assert_eq!(
        view.pointer("/result/structuredContent/reserves_stock"),
        Some(&json!(false))
    );
    let args = json!({"listing_id":LISTING_A,"quote_id":q,"terms_digest":digest,"proposal":p});
    let denied = post(
        &client,
        &endpoint,
        "reader",
        call("submit_my_quoted_offer_change", args.clone()),
    )
    .await?;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    Mock::given(method("POST")).and(path(format!("{base}s"))).and(matches_header("authorization","Bearer exact-grant")).and(matches_header("idempotency-key",q))
        .and(body_json(json!({"quote_id":q,"terms_digest":digest,"proposal":p}))).respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"action_id":act,"operation":"make_offer","status":"awaiting_confirmation","listing_id":LISTING_A,"offer_id":null,"order_id":null}}))).expect(1).mount(&api).await;
    let result: Value = post(
        &client,
        &endpoint,
        "trader",
        call("submit_my_quoted_offer_change", args),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        result.pointer("/result/structuredContent/confirmation_url"),
        Some(&json!(format!("https://connect.budna.se/approvals/{act}")))
    );
    let bad:Value=post(&client,&endpoint,"trader",call("prepare_my_offer_change",json!({"listing_id":LISTING_A,"proposal":{"operation":"accept_offer","offer_id":10,"total":m("125")}}))).await?.error_for_status()?.json().await?;
    assert_eq!(bad.pointer("/result/isError"), Some(&json!(true)));
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
