use super::*;
const KEY: &str = "50000000-0000-4000-8000-000000000075";

#[tokio::test]
async fn similar_draft_checks_its_own_grant_verification_source_and_intent() -> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "author",
        "owner-grant",
        USER_A,
        "profile:read draftauthoring:write",
    )
    .await;
    exchange(
        &api,
        "initial",
        "initial-grant",
        USER_B,
        "listingdrafts:write",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("listingdrafts:write draftauthoring:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let name = "create_my_similar_draft";
    let args = json!({"expected_revision":1234,"listing_id":LISTING_A,"request_id":KEY});
    assert_eq!(
        post(&client, &endpoint, "initial", call(name, args.clone()))
            .await?
            .status(),
        StatusCode::FORBIDDEN
    );
    let route = || {
        Mock::given(method("POST"))
            .and(path(format!(
                "/api/v1/mcp/account/listings/{LISTING_A}/similar-draft"
            )))
            .and(matches_header("authorization", "Bearer owner-grant"))
            .and(matches_header("idempotency-key", KEY))
            .and(body_json(json!({"expected_revision":1234})))
    };
    {
        let _m = route()
            .respond_with(
                ResponseTemplate::new(403)
                    .set_body_json(json!({"code":"SELLER_VERIFICATION_REQUIRED"})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let denied = post(&client, &endpoint, "author", call(name, args.clone()))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_write_error(&denied, name, "SELLER_VERIFICATION_REQUIRED")?;
    }
    let receipt = json!({"listing_id":LISTING_B,"action_id":"00000000-0000-4000-8000-000000000077","status":"executed"});
    let _m = route()
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":receipt})),
        )
        .expect(2)
        .mount_as_scoped(&api)
        .await;
    for _ in 0..2 {
        let r: Value = post(&client, &endpoint, "author", call(name, args.clone()))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_eq!(r.pointer("/result/structuredContent"), Some(&receipt));
    }
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}

#[tokio::test]
async fn shipping_edits_bind_one_delta_to_a_reviewed_revision_and_independent_grant() -> Result<()>
{
    let api = MockServer::start().await;
    exchange(
        &api,
        "rules",
        "rule-grant",
        USER_A,
        "shippingrules:read shippingrules:write",
    )
    .await;
    exchange(
        &api,
        "defaults",
        "default-grant",
        USER_B,
        "selling:read shippingdefaults:write",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("shippingrules:read shippingrules:write shippingdefaults:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let change = "change_my_shipping_rule";
    let revision = "A".repeat(43);
    let kinds = json!([{"id":9,"name":"Books","listing_count":1}]);
    let parcels =
        json!([{"id":2,"name":"","package_size":"small","kinds":[{"kind_id":9,"quantity":2}]}]);
    let base = "/api/v1/mcp/account/selling";
    Mock::given(method("GET")).and(path(format!("{base}/shipping-rules"))).and(matches_header("authorization","Bearer rule-grant")).respond_with(
        ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"revision":revision,"kinds":kinds,"parcels":parcels,"extra":SENTINEL}}))).expect(1).mount(&api).await;
    let r: Value = post(
        &client,
        &endpoint,
        "rules",
        call("get_my_shipping_rules", json!({})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    let plate = &r["result"]["structuredContent"];
    assert_eq!(plate["revision"], revision);
    assert_eq!(plate["kinds"], kinds);
    assert_eq!(plate["parcels"], parcels);
    assert!(plate["untrusted_content_notice"].is_string());
    assert_eq!(
        plate
            .as_object()
            .context("structured shipping rules")?
            .len(),
        4
    );
    let args = json!({"expected_revision":revision,"request_id":KEY,"change":{"operation":"set_parcel_kind","parcel_id":2,"kind_id":9,"quantity":3}});
    assert_eq!(
        post(&client, &endpoint, "defaults", call(change, args.clone()))
            .await?
            .status(),
        StatusCode::FORBIDDEN
    );
    for bad in [
        json!({"operation":"set_parcel_kind","parcel_id":2,"kind_id":9,"quantity":0}),
        json!({"operation":"set_parcel_kind","parcel_id":2,"kind_id":9,"quantity":3,"price":"4.50"}),
        json!({"operation":"create_kind","name":"Book","listing_id":"invalid"}),
    ] {
        let mut bad_args = args.clone();
        bad_args["change"] = bad;
        let r = post(&client, &endpoint, "rules", call(change, bad_args))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_write_error(&r, change, "INVALID_INPUT")?;
    }
    let receipt = json!({"kind_id":null,"parcel_id":2,"action_id":KEY,"operation":"set_parcel_kind","status":"executed"});
    Mock::given(method("POST")).and(path(format!("{base}/shipping-rules"))).and(matches_header("idempotency-key",KEY))
        .and(body_json(json!({"expected_revision":revision,"operation":"set_parcel_kind","parcel_id":2,"kind_id":9,"quantity":3})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":receipt}))).expect(1).mount(&api).await;
    let r: Value = post(&client, &endpoint, "rules", call(change, args))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(r.pointer("/result/structuredContent"), Some(&receipt));
    let mut defaults = json!({"expected_revision":revision,"request_id":KEY,"change":{"operation":"set_carriers","provider_codes":["dhl","dhl"]}});
    let name = "change_my_shipping_defaults";
    let bad = post(&client, &endpoint, "defaults", call(name, defaults.clone()))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_write_error(&bad, name, "INVALID_INPUT")?;
    defaults["change"] = json!({"operation":"reset_carriers"});
    let receipt = json!({"kind_id":null,"parcel_id":null,"action_id":KEY,"operation":"reset_carriers","status":"executed"});
    Mock::given(method("POST"))
        .and(path(format!("{base}/shipping-settings")))
        .and(matches_header("authorization", "Bearer default-grant"))
        .and(body_json(
            json!({"expected_revision":revision,"operation":"reset_carriers"}),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":receipt})),
        )
        .expect(1)
        .mount(&api)
        .await;
    let r: Value = post(&client, &endpoint, "defaults", call(name, defaults))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(r.pointer("/result/structuredContent"), Some(&receipt));
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
