use super::*;

#[tokio::test]
async fn case_permissions_bind_party_amount_expiry_and_the_approval_destination() -> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "cases",
        "case-owner",
        USER_A,
        "orders:read disputecases:write disputesettlements:write",
    )
    .await;
    exchange(&api, "reader", "case-reader", USER_B, "orders:read").await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("disputecases:write disputesettlements:write"),
    )
    .await?;
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let amount = json!({"amount":"2.50","currency_code":"SEK"});
    let facts = json!({"order_id":7,"listing_id":LISTING_A,"order_revision":9,"side":"buyer","can_open":false,"payout_can_be_held":true,"current_dispute_id":11,"case_revision":13,"can_withdraw_claim":true,"can_propose":false,"can_accept":true,"can_decline":true,"can_withdraw_offer":false,"can_submit_statement":true,"current_offer":{"offer_id":17,"amount":amount,"expires_at":19},"maximum_discount":null});
    let mut wire = facts.clone();
    wire["current_offer"]["unlisted"] = json!(SENTINEL);
    wire["private_field"] = json!(SENTINEL);
    Mock::given(method("GET"))
        .and(path("/api/v1/mcp/account/orders/7/dispute-options"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":wire})))
        .expect(1)
        .mount(&api)
        .await;
    let response: Value = post(
        &client,
        &endpoint,
        "reader",
        call("get_my_order_dispute_options", json!({"order_id":7})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(response.pointer("/result/structuredContent"), Some(&facts));
    let base = json!({"order_id":7,"dispute_id":11,"expected_case_revision":13,"expected_order_revision":9});
    let mut accept = base.clone();
    accept["offer_id"] = json!(17);
    accept["amount"] = amount.clone();
    accept["expires_at"] = json!(19);
    assert_eq!(
        post(
            &client,
            &endpoint,
            "reader",
            write_call("accept_my_dispute_settlement", accept.clone())
        )
        .await?
        .status(),
        StatusCode::FORBIDDEN
    );
    let mut bad = accept.clone();
    bad["approved"] = json!(true);
    let r = post(
        &client,
        &endpoint,
        "cases",
        write_call("accept_my_dispute_settlement", bad),
    )
    .await?
    .json()
    .await?;
    assert_write_error(&r, "accept_my_dispute_settlement", "INVALID_INPUT")?;
    let mut propose = base.clone();
    propose["amount"] = amount.clone();
    propose["message"] = json!("Agreed discount");
    for (op, verb, args, body, commit, case, statement, offer) in [
        (
            "open_my_order_dispute",
            "open_order_dispute",
            json!({"order_id":7,"expected_listing_id":LISTING_A,"expected_order_revision":9,"reason":"item_damaged","title":"Item damaged","description":"The ceramic cup has a cracked handle.","requested_remedy":"refund","audience":"parties_and_budna"}),
            json!({"operation":"open","expected_listing_id":LISTING_A,"expected_order_revision":9,"reason":"item_damaged","title":"Item damaged","description":"The ceramic cup has a cracked handle.","requested_remedy":"refund","audience":"parties_and_budna"}),
            false,
            11,
            None,
            None,
        ),
        (
            "add_my_dispute_statement",
            "submit_dispute_statement",
            json!({"order_id":7,"dispute_id":11,"expected_case_revision":13,"body":"The handle arrived cracked.","audience":"parties_and_budna"}),
            json!({"operation":"statement","dispute_id":11,"expected_case_revision":13,"body":"The handle arrived cracked.","audience":"parties_and_budna"}),
            false,
            11,
            Some(8),
            None,
        ),
        (
            "decline_my_dispute_settlement",
            "decline_dispute_settlement",
            json!({"order_id":7,"dispute_id":11,"expected_case_revision":13,"offer_id":17}),
            json!({"operation":"decline_offer","dispute_id":11,"expected_case_revision":13,"offer_id":17}),
            false,
            11,
            None,
            Some(17),
        ),
        (
            "withdraw_my_settlement_offer",
            "withdraw_dispute_settlement_offer",
            json!({"order_id":7,"dispute_id":11,"expected_case_revision":13,"offer_id":17}),
            json!({"operation":"withdraw_offer","dispute_id":11,"expected_case_revision":13,"offer_id":17}),
            false,
            11,
            None,
            Some(17),
        ),
        (
            "request_withdraw_my_dispute",
            "withdraw_dispute_claim",
            base.clone(),
            json!({"operation":"withdraw_claim"}),
            true,
            11,
            None,
            None,
        ),
        (
            "propose_my_dispute_settlement",
            "propose_dispute_settlement",
            propose,
            json!({"operation":"propose","amount":amount,"message":"Agreed discount"}),
            true,
            11,
            None,
            None,
        ),
        (
            "accept_my_dispute_settlement",
            "accept_dispute_settlement",
            accept,
            json!({"operation":"accept","offer_id":17,"amount":amount,"expires_at":19}),
            true,
            11,
            None,
            Some(17),
        ),
    ] {
        let body = if commit {
            let mut b = base.clone();
            b["decision"] = body;
            b
        } else {
            body
        };
        let receipt = json!({"order_id":7,"dispute_id":case,"statement_id":statement,"offer_id":offer,"action_id":REQUEST_ID,"operation":verb,"status":if commit{"awaiting_confirmation"}else{"executed"}});
        let mut raw = receipt.clone();
        raw["confirmation_url"] = json!("https://example.invalid/spoof");
        let _mock = Mock::given(method("POST"))
            .and(path(format!(
                "/api/v1/mcp/account/orders/7/dispute-{}",
                if commit { "decisions" } else { "actions" }
            )))
            .and(matches_header("idempotency-key", REQUEST_ID))
            .and(body_json(body))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":raw})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let r: Value = post(&client, &endpoint, "cases", write_call(op, args))
            .await?
            .error_for_status()?
            .json()
            .await?;
        let mut expected = receipt;
        expected["confirmation_url"] =
            json!(commit.then(|| format!("https://connect.budna.se/approvals/{REQUEST_ID}")));
        assert_eq!(
            r.pointer("/result/structuredContent"),
            Some(&expected),
            "{op}"
        );
    }
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}

#[tokio::test]
async fn listing_owner_can_apply_only_a_quoted_typed_change() -> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "owner",
        "manage-listing",
        USER_A,
        "selling:read listingmanagement:write",
    )
    .await;
    exchange(&api, "reader", "read-listing", USER_B, "selling:read").await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("listingmanagement:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let quote = format!("quote_{}", "a".repeat(43));
    let preview = "prepare_my_listing_change";
    let apply = "apply_my_quoted_listing_change";
    let qs = format!("/api/v1/mcp/account/listings/{LISTING_A}/management-quote");
    let submit = format!("/api/v1/mcp/account/listings/{LISTING_A}/management");
    for change in [
        json!({"kind":"edit_content","title":"Blue lamp"}),
        json!({"kind":"edit_prices","seller_reserve":{"amount":"50","currency_code":"SEK"}}),
        json!({"kind":"extend","days":2}),
        json!({"kind":"relist","days":7}),
        json!({"kind":"delete"}),
    ] {
        let relist = change["kind"] == "relist";
        let facts = json!({"listing_id":LISTING_A,"revision":7,"kind":change["kind"],"ready":true,"blockers":[],"quote_id":quote,"starts_at":(!relist).then_some(10),"ends_at":(!relist).then_some(20),"additional_sale_fee":{"amount":"0","currency_code":"SEK"},"notifies_watchers":relist});
        let _q = Mock::given(method("POST"))
            .and(path(&qs))
            .and(body_json(json!({"operation":change})))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":facts})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let r: Value = post(
            &client,
            &endpoint,
            "reader",
            call(preview, json!({"listing_id":LISTING_A,"operation":change})),
        )
        .await?
        .error_for_status()?
        .json()
        .await?;
        assert_eq!(r.pointer("/result/structuredContent"), Some(&facts));
        let receipt = json!({"action_id":REQUEST_ID,"listing_id":LISTING_A,"status":"executed"});
        let _s = Mock::given(method("POST"))
            .and(path(&submit))
            .and(matches_header("idempotency-key", REQUEST_ID))
            .and(body_json(
                json!({"operation":change,"expected_revision":7,"quote_id":quote}),
            ))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":receipt})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let args = json!({"listing_id":LISTING_A,"operation":change,"expected_revision":7,"quote_id":quote});
        assert_eq!(
            post(
                &client,
                &endpoint,
                "reader",
                write_call(apply, args.clone())
            )
            .await?
            .status(),
            StatusCode::FORBIDDEN
        );
        let r: Value = post(&client, &endpoint, "owner", write_call(apply, args))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_eq!(r.pointer("/result/structuredContent"), Some(&receipt));
    }
    for operation in [
        json!({"kind":"delete","force":true}),
        json!({"kind":"extend","days":15}),
        json!({"kind":"edit_prices","starting_price":null}),
        json!({"kind":"edit_content"}),
    ] {
        let r = post(
            &client,
            &endpoint,
            "owner",
            call(
                preview,
                json!({"listing_id":LISTING_A,"operation":operation}),
            ),
        )
        .await?
        .json()
        .await?;
        assert_write_error(&r, preview, "INVALID_INPUT")?;
    }
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
