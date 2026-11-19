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

#[tokio::test]
async fn draft_edits_preserve_omitted_fields_and_flag_incomplete_attribute_previews() -> Result<()>
{
    let api = MockServer::start().await;
    exchange(
        &api,
        "owner",
        "draft-grant",
        USER_A,
        "draftauthoring:write listings:read",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("draftauthoring:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let path_base = format!("/api/v1/mcp/account/listings/{LISTING_A}/draft-authoring");
    let fields = json!({"title":"My draft","description":null,"currency":"SEK","starting_price":{"amount":"1.25","currency_code":"SEK","extra":SENTINEL},"extra":SENTINEL});
    let attr = json!({"filter_definition_id":2,"value":{"kind":"text","value":"old\u{202e}text"},"extra":SENTINEL});
    Mock::given(method("GET")).and(path(&path_base)).and(matches_header("authorization","Bearer draft-grant"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"listing_id":LISTING_A,"revision":2,"fields":fields,"attributes":[attr],"attributes_replaceable":true,"vat":null,"media_count":0,"extra":SENTINEL}}))).expect(1).mount(&api).await;
    let r: Value = post(
        &client,
        &endpoint,
        "owner",
        call("get_my_draft_authoring", json!({"listing_id":LISTING_A})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    let read = &r["result"]["structuredContent"];
    assert_eq!(read["attributes_replaceable"], false);
    assert_eq!(read["attributes"][0]["value"]["value"], "oldtext");
    assert_eq!(
        read["fields"]["starting_price"],
        json!({"amount":"1.25","currency_code":"SEK"})
    );
    assert!(!r.to_string().contains(SENTINEL));
    let target = json!({"listing_id":LISTING_A,"expected_revision":2,"request_id":KEY});
    let edit = "edit_my_listing_draft";
    for change in [
        json!({}),
        json!({"status":"active"}),
        json!({"quantity":3}),
        json!({"description":null}),
        json!({"package_size":"small"}),
        json!({"seller_reserve":{"amount":12.5,"currency_code":"SEK"}}),
        json!({"custom_shipping_price":{"amount":"12.50","currency_code":"SEK"},"clear_custom_shipping_price":true}),
    ] {
        let mut args = target.clone();
        args["changes"] = change;
        let r = post(&client, &endpoint, "owner", call(edit, args))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_write_error(&r, edit, "INVALID_INPUT")?;
    }
    let receipt = json!({"action_id":KEY,"listing_id":LISTING_A,"status":"executed"});
    for (name, http, suffix, extra) in [
        (
            edit,
            "PATCH",
            "",
            json!({"changes":{"seller_reserve":{"amount":"12.50","currency_code":"SEK"}}}),
        ),
        (
            "replace_my_draft_attributes",
            "PUT",
            "/attributes",
            json!({"attributes":[]}),
        ),
        (
            "set_my_draft_vat",
            "PUT",
            "/vat",
            json!({"vat_treatment":null}),
        ),
        ("discard_my_listing_draft", "DELETE", "", json!({})),
    ] {
        let mut args = target.clone();
        let mut wire = extra.clone();
        for (k, v) in extra.as_object().context("edit")? {
            args[k] = v.clone();
        }
        wire["expected_revision"] = json!(2);
        let _m = Mock::given(method(http))
            .and(path(format!("{path_base}{suffix}")))
            .and(matches_header("idempotency-key", KEY))
            .and(body_json(wire))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":receipt})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let r: Value = post(&client, &endpoint, "owner", call(name, args))
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
async fn inventory_requires_the_exact_reviewed_stock_and_outside_sale_effects() -> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "seller",
        "seller-inventory",
        USER_A,
        "inventory:write listings:read",
    )
    .await;
    exchange(&api, "reader", "reader-inventory", USER_B, "listings:read").await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("inventory:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let base = format!("/api/v1/mcp/account/listings/{LISTING_A}/inventory");
    let view = json!({"listing_id":LISTING_A,"revision":3,"available_quantity":4,"checkout_reserved_quantity":0,"is_sold_out":false,"can_set_stock":true,"can_record_external_sale":true,"pending_offers_affected":1});
    Mock::given(method("GET"))
        .and(path(&base))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":view})))
        .expect(1)
        .mount(&api)
        .await;
    let r: Value = post(
        &client,
        &endpoint,
        "seller",
        call("get_my_listing_inventory", json!({"listing_id":LISTING_A})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(r.pointer("/result/structuredContent"), Some(&view));
    let reviewed = json!({"listing_id":LISTING_A,"request_id":KEY,"expected_revision":3,"expected_quantity":4,"expected_reserved":0});
    let set = "set_my_listing_stock";
    let sale = "record_my_listing_sold_elsewhere";
    assert_eq!(
        post(
            &client,
            &endpoint,
            "reader",
            call(set, json!({"reviewed":reviewed,"quantity":1}))
        )
        .await?
        .status(),
        StatusCode::FORBIDDEN
    );
    for (op, bad) in [
        (set, json!({"reviewed":reviewed,"quantity":0})),
        (sale, json!({"reviewed":reviewed})),
        (
            sale,
            json!({"reviewed":reviewed,"expected_pending_offers":1,"order_id":7}),
        ),
    ] {
        let r = post(&client, &endpoint, "seller", call(op, bad))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_write_error(&r, op, "INVALID_INPUT")?;
    }
    for (op, args, delta, action) in [
        (
            set,
            json!({"reviewed":reviewed,"quantity":2}),
            json!({"kind":"set_available","quantity":2}),
            "set_listing_stock",
        ),
        (
            sale,
            json!({"reviewed":reviewed,"expected_pending_offers":1}),
            json!({"kind":"record_external_sale","expected_pending_offers":1}),
            "record_external_sale",
        ),
    ] {
        let receipt =
            json!({"action_id":KEY,"listing_id":LISTING_A,"status":"executed","operation":action});
        let _m=Mock::given(method("POST")).and(path(&base)).and(matches_header("idempotency-key",KEY)).and(body_json(json!({"change":delta,"expected_revision":3,"expected_quantity":4,"expected_reserved":0})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":receipt}))).expect(1).mount_as_scoped(&api).await;
        let r: Value = post(&client, &endpoint, "seller", call(op, args))
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
async fn publishing_preserves_the_quoted_price_and_refuses_future_start_and_verification_gaps()
-> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "seller",
        "publish-grant",
        USER_A,
        "selling:read listingpublication:write",
    )
    .await;
    exchange(&api, "reader", "selling-grant", USER_B, "selling:read").await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("listingpublication:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let base = format!("/api/v1/mcp/account/listings/{LISTING_A}/publication");
    let quote = format!("quote_{}", "A".repeat(43));
    let money = |s: &str| json!({"amount":s,"currency_code":"SEK"});
    let prepared = json!({"listing_id":LISTING_A,"revision":1,"ready":true,"blockers":[],"quote_id":quote,"starts_at":100,"ends_at":1000,"media_count":1,"upfront_due":money("0"),"assumed_sale_price":money("100.00"),"estimated_sale_fees":money("12.75"),"estimated_seller_proceeds":money("87.25"),"priority_tier":null,"priority_credit_expected":false});
    let name = "publish_my_quoted_draft";
    for blocked in [true, false] {
        let mut state = prepared.clone();
        if blocked {
            state["ready"] = json!(false);
            state["blockers"] = json!(["future_start_unsupported", "seller_verification_required"]);
            state["quote_id"] = Value::Null;
            for field in [
                "assumed_sale_price",
                "estimated_sale_fees",
                "estimated_seller_proceeds",
            ] {
                state[field] = Value::Null;
            }
        }
        let mut wire = state.clone();
        wire["extra"] = json!(SENTINEL);
        let _m = Mock::given(method("GET"))
            .and(path(&base))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":wire})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let r: Value = post(
            &client,
            &endpoint,
            "reader",
            call(
                "get_my_publication_readiness",
                json!({"listing_id":LISTING_A}),
            ),
        )
        .await?
        .error_for_status()?
        .json()
        .await?;
        assert_eq!(r.pointer("/result/structuredContent"), Some(&state));
    }
    let args =
        json!({"request_id":KEY,"listing_id":LISTING_A,"expected_revision":1,"quote_id":quote});
    assert_eq!(
        post(&client, &endpoint, "reader", call(name, args.clone()))
            .await?
            .status(),
        StatusCode::FORBIDDEN
    );
    let mut forge = args.clone();
    forge["override_upfront_fee"] = json!(true);
    let r = post(&client, &endpoint, "seller", call(name, forge))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_write_error(&r, name, "INVALID_INPUT")?;
    let route = || {
        Mock::given(method("POST"))
            .and(path(&base))
            .and(matches_header("idempotency-key", KEY))
            .and(body_json(json!({"quote_id":quote,"expected_revision":1})))
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
        let r = post(&client, &endpoint, "seller", call(name, args.clone()))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_write_error(&r, name, "SELLER_VERIFICATION_REQUIRED")?;
    }
    let receipt = json!({"action_id":KEY,"listing_id":LISTING_A,"status":"executed"});
    let _m = route()
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":receipt})),
        )
        .expect(1)
        .mount_as_scoped(&api)
        .await;
    let r: Value = post(&client, &endpoint, "seller", call(name, args))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(r.pointer("/result/structuredContent"), Some(&receipt));
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
