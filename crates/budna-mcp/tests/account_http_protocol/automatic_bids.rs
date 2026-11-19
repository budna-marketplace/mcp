use super::*;

#[tokio::test]
async fn standing_authority_has_its_own_scope_quote_exact_amount_and_revision_bound_pause()
-> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "owner",
        "owner-grant",
        USER_A,
        "bids:read autobids:write",
    )
    .await;
    exchange(
        &api,
        "once",
        "one-bid-grant",
        USER_B,
        "bids:read bids:write",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("bids:write autobids:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let money = |n: &str| json!({"amount":n,"currency_code":"SEK"});
    let rev = "A".repeat(43);
    let key = "00000000-0000-4000-8000-000000000085";
    let act = "00000000-0000-4000-8000-000000000086";
    let base = format!("/api/v1/mcp/account/listings/{LISTING_A}/automatic-bid");
    let own =
        json!({"listing_id":LISTING_A,"revision":rev,"status":"active","maximum":money("125")});
    Mock::given(method("GET"))
        .and(path(&base))
        .and(matches_header("authorization", "Bearer owner-grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":own})))
        .expect(1)
        .mount(&api)
        .await;
    let view: Value = post(
        &client,
        &endpoint,
        "owner",
        call(
            "get_my_automatic_bid_state",
            json!({"listing_id":LISTING_A}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(view.pointer("/result/structuredContent"), Some(&own));
    let q = json!({"quote_id":key,"listing_id":LISTING_A,"live_session_id":null,"round_ref":format!("round_{}","B".repeat(43)),"bid":money("150"),"current_bid":money("95"),"minimum_bid":money("100"),"increment":money("5"),"buyer_protection_fee":money("5"),"buyer_protection_mandatory":true,"unpaid_fee":money("25"),"payment_due_hours":72,"payment_grace_hours":0,"delivery_at_checkout":true,"ends_at":1900001000000i64,"expires_at":1900000000000i64});
    Mock::given(method("POST")).and(path(format!("{base}-quote"))).and(matches_header("authorization","Bearer owner-grant"))
        .and(body_json(json!({"maximum":money("150.00")}))).respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"auction":q,"existing":own,"can_place_bid_immediately":true}}))).expect(1).mount(&api).await;
    let v: Value = post(
        &client,
        &endpoint,
        "owner",
        call(
            "prepare_my_automatic_bid_maximum",
            json!({"listing_id":LISTING_A,"maximum":money("150.00")}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        v.pointer("/result/structuredContent/maximum"),
        Some(&money("150"))
    );
    assert_eq!(v.pointer("/result/structuredContent/existing"), Some(&own));
    assert_eq!(
        v.pointer("/result/structuredContent/can_place_bid_immediately"),
        Some(&json!(true))
    );
    let args = json!({"listing_id":LISTING_A,"quote_id":key,"maximum":money("150.00")});
    let denied = post(
        &client,
        &endpoint,
        "once",
        call("set_my_quoted_automatic_bid_maximum", args.clone()),
    )
    .await?;
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    Mock::given(method("PUT")).and(path(&base)).and(matches_header("authorization","Bearer owner-grant")).and(matches_header("idempotency-key",key))
        .and(body_json(json!({"quote_id":key,"maximum":money("150.00")}))).respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"action_id":act,"status":"awaiting_confirmation","maximum":money("150"),"listing_id":LISTING_A}}))).expect(1).mount(&api).await;
    let v: Value = post(
        &client,
        &endpoint,
        "owner",
        call("set_my_quoted_automatic_bid_maximum", args),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        v.pointer("/result/structuredContent/confirmation_url"),
        Some(&json!(format!("https://connect.budna.se/approvals/{act}")))
    );
    let r = json!({"action_id":act,"status":"executed","listing_id":LISTING_A});
    Mock::given(method("POST"))
        .and(path(format!("{base}/pause")))
        .and(matches_header("idempotency-key", key))
        .and(body_json(json!({"expected_revision":rev})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":r})))
        .expect(1)
        .mount(&api)
        .await;
    let v: Value = post(
        &client,
        &endpoint,
        "owner",
        call(
            "pause_my_automatic_bid",
            json!({"listing_id":LISTING_A,"expected_revision":rev,"idempotency_key":key}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(v.pointer("/result/structuredContent"), Some(&r));
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}

#[tokio::test]
async fn a_historical_maximum_keeps_its_currency_and_an_unrepresentable_one_is_never_relabeled()
-> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "owner",
        "owner-grant",
        USER_A,
        "bids:read autobids:write",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("bids:write autobids:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let rev = "C".repeat(43);
    let key = "00000000-0000-4000-8000-000000000087";
    let act = "00000000-0000-4000-8000-000000000088";
    let base = format!("/api/v1/mcp/account/listings/{LISTING_A}/automatic-bid");
    let reply =
        |data: Value| ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":data}));
    let state = |status: &str, maximum: Value| json!({"listing_id":LISTING_A,"revision":rev,"status":status,"maximum":maximum});
    let read = |listing: &str| call("get_my_automatic_bid_state", json!({"listing_id":listing}));
    let nok = json!({"amount":"150.5","currency_code":"NOK"});
    let active = state("active", nok.clone());
    let paused = state("paused", nok);
    let unrepresentable = state("active", json!({"amount":"0","currency_code":"NOK"}));
    for (wire, shown) in [(&active, true), (&unrepresentable, false)] {
        let guard = Mock::given(method("GET"))
            .and(path(&base))
            .respond_with(reply(wire.clone()))
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let v: Value = post(&client, &endpoint, "owner", read(LISTING_A))
            .await?
            .error_for_status()?
            .json()
            .await?;
        if shown {
            assert_eq!(v.pointer("/result/structuredContent"), Some(wire));
        } else {
            assert_write_error(
                &v,
                "get_my_automatic_bid_state",
                "BUDNA_API_INVALID_RESPONSE",
            )?;
        }
        drop(guard);
    }
    let receipt = json!({"action_id":act,"status":"executed","listing_id":LISTING_A});
    Mock::given(method("POST"))
        .and(path(format!("{base}/pause")))
        .and(matches_header("idempotency-key", key))
        .and(body_json(json!({"expected_revision":rev})))
        .respond_with(reply(receipt.clone()))
        .expect(1)
        .mount(&api)
        .await;
    let v: Value = post(
        &client,
        &endpoint,
        "owner",
        call(
            "pause_my_automatic_bid",
            json!({"listing_id":LISTING_A,"expected_revision":rev,"idempotency_key":key}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(v.pointer("/result/structuredContent"), Some(&receipt));
    let guard = Mock::given(method("GET"))
        .and(path(&base))
        .respond_with(reply(paused.clone()))
        .expect(1)
        .mount_as_scoped(&api)
        .await;
    let v: Value = post(&client, &endpoint, "owner", read(LISTING_A))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(v.pointer("/result/structuredContent"), Some(&paused));
    drop(guard);
    for (listing, status, code) in [
        (LISTING_A, 409, "API_CONFLICT"),
        (LISTING_B, 404, "API_NOT_FOUND"),
    ] {
        let guard = Mock::given(method("GET"))
            .and(path(format!(
                "/api/v1/mcp/account/listings/{listing}/automatic-bid"
            )))
            .respond_with(ResponseTemplate::new(status))
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let v: Value = post(&client, &endpoint, "owner", read(listing))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_write_error(&v, "get_my_automatic_bid_state", code)?;
        drop(guard);
    }
    let money = |n: &str| json!({"amount":n,"currency_code":"SEK"});
    let q = json!({"quote_id":key,"listing_id":LISTING_A,"live_session_id":null,"round_ref":format!("round_{}","B".repeat(43)),"bid":money("200"),"current_bid":null,"minimum_bid":money("100"),"increment":money("5"),"buyer_protection_fee":money("5"),"buyer_protection_mandatory":true,"unpaid_fee":money("25"),"payment_due_hours":72,"payment_grace_hours":0,"delivery_at_checkout":true,"ends_at":1900001000000i64,"expires_at":1900000000000i64});
    Mock::given(method("POST"))
        .and(path(format!("{base}-quote")))
        .respond_with(reply(
            json!({"auction":q,"existing":active,"can_place_bid_immediately":true}),
        ))
        .expect(1)
        .mount(&api)
        .await;
    let v: Value = post(
        &client,
        &endpoint,
        "owner",
        call(
            "prepare_my_automatic_bid_maximum",
            json!({"listing_id":LISTING_A,"maximum":money("200")}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_write_error(
        &v,
        "prepare_my_automatic_bid_maximum",
        "BUDNA_API_INVALID_RESPONSE",
    )?;
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
