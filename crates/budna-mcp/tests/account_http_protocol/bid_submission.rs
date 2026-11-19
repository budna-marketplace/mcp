use super::*;

#[tokio::test]
async fn quote_and_commit_keep_the_amount_round_and_approval_separate() -> Result<()> {
    let api = MockServer::start().await;
    exchange(&api, "owner", "owner-grant", USER_A, "bids:read bids:write").await;
    exchange(&api, "reader", "reader-grant", USER_B, "bids:read").await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("bids:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let amount = |value: &str| json!({"amount":value,"currency_code":"SEK"});
    let quote_id = "00000000-0000-4000-8000-000000000075";
    let action_id = "00000000-0000-4000-8000-000000000076";
    let prepare = "prepare_my_bid";
    let submit = "submit_my_quoted_bid";
    let quote = json!({"quote_id":quote_id,"listing_id":LISTING_A,"live_session_id":null,"round_ref":format!("round_{}","A".repeat(43)),"bid":amount("100.1"),"current_bid":null,"minimum_bid":amount("95"),"increment":amount("5"),"buyer_protection_fee":amount("4.5"),"buyer_protection_mandatory":false,"unpaid_fee":amount("5"),"payment_due_hours":48,"payment_grace_hours":0,"delivery_at_checkout":true,"ends_at":2000000000000_i64,"expires_at":1900000000000_i64});
    let mut wire = quote.clone();
    wire["extra"] = json!(SENTINEL);
    Mock::given(method("POST"))
        .and(path(format!(
            "/api/v1/mcp/account/listings/{LISTING_A}/bid-quote"
        )))
        .and(matches_header("authorization", "Bearer reader-grant"))
        .and(body_json(json!({"bid":amount("100.10")})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":wire})))
        .expect(1)
        .mount(&api)
        .await;
    let response: Value = post(
        &client,
        &endpoint,
        "reader",
        call(
            prepare,
            json!({"listing_id":LISTING_A,"bid":amount("100.10")}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(response.pointer("/result/structuredContent"), Some(&quote));
    let submit_args = json!({"listing_id":LISTING_A,"quote_id":quote_id,"bid":amount("100.10")});
    assert_eq!(
        post(
            &client,
            &endpoint,
            "reader",
            call(submit, submit_args.clone())
        )
        .await?
        .status(),
        StatusCode::FORBIDDEN
    );
    let mut forged = submit_args.clone();
    forged["client_approved"] = json!(true);
    let invalid = post(&client, &endpoint, "owner", call(submit, forged))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_write_error(&invalid, submit, "INVALID_INPUT")?;
    let mut invalid_cash = amount("12.005");
    invalid_cash["currency_code"] = json!("NOK");
    let invalid = post(
        &client,
        &endpoint,
        "owner",
        call(prepare, json!({"listing_id":LISTING_A,"bid":invalid_cash})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_write_error(&invalid, prepare, "INVALID_INPUT")?;
    let receipt = json!({"action_id":action_id,"status":"awaiting_confirmation","listing_id":LISTING_A,"bid":amount("100.1")});
    let mut wire = receipt.clone();
    wire["confirmation_url"] = json!("https://untrusted.example.test/route");
    let _submit = Mock::given(method("POST"))
        .and(path(format!(
            "/api/v1/mcp/account/listings/{LISTING_A}/bids"
        )))
        .and(matches_header("authorization", "Bearer owner-grant"))
        .and(matches_header("idempotency-key", quote_id))
        .and(body_json(
            json!({"quote_id":quote_id,"bid":amount("100.10")}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":wire})))
        .expect(2)
        .mount_as_scoped(&api)
        .await;
    for _ in 0..2 {
        let r: Value = post(
            &client,
            &endpoint,
            "owner",
            call(submit, submit_args.clone()),
        )
        .await?
        .error_for_status()?
        .json()
        .await?;
        let mut expected = receipt.clone();
        expected["confirmation_url"] =
            json!(format!("https://connect.budna.se/approvals/{action_id}"));
        assert_eq!(r.pointer("/result/structuredContent"), Some(&expected));
    }
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
