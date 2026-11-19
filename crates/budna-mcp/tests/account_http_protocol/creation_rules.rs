use super::*;
use wiremock::matchers::query_param;

#[tokio::test]
async fn category_rules_and_options_share_a_snapshot_with_the_attached_draft_write() -> Result<()> {
    let api = MockServer::start().await;
    exchange(&api, "read", "seller-read", USER_A, "listings:read").await;
    exchange(
        &api,
        "write",
        "seller-write",
        USER_B,
        "draftauthoring:write",
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
    let revision = "a".repeat(43);
    let query = json!({"category_id":4,"market":"swedish","limit":1});
    let name = "get_listing_creation_rules";
    let choices = "list_listing_creation_options";
    let replace = "replace_my_draft_attributes";
    let fields = json!([{"filter_definition_id":2,"name":"colors","label":"Colors","filter_type":"multi_select","value_kind":"options","required_on_publish":true,"min_length":null,"max_length":null,"pattern":null,"minimum":null,"maximum":null,"step":null,"unit":null,"max_fraction_digits":null,"maximum_exclusive_magnitude":null,"max_choices":50,"active_option_count":30,"supported_in_mcp":true}]);
    let rules = json!({"category_id":4,"category_name":"Lamps","market":"swedish","mcp_price_currency":"SEK","rules_revision":revision,"listing_types":["auction","auction_fixed_price","fixed_price","in_stock"],"conditions":["new","like_new","very_good","good","acceptable"],"attributes_max":50,"replacement":"full_set","required_for":"publication","value_dependencies":"category","total_fields":2,"fields":fields,"next_after":2});
    let mut wire = rules.clone();
    wire["extra"] = json!(SENTINEL);
    wire["fields"][0]["private_note"] = json!(SENTINEL);
    Mock::given(method("GET"))
        .and(path("/api/v1/mcp/account/listings/creation-rules"))
        .and(query_param("category_id", "4"))
        .and(query_param("market", "swedish"))
        .and(query_param("limit", "1"))
        .and(matches_header("authorization", "Bearer seller-read"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":wire})))
        .expect(1)
        .mount(&api)
        .await;
    assert_eq!(
        post(&client, &endpoint, "write", call(name, query.clone()))
            .await?
            .status(),
        StatusCode::FORBIDDEN
    );
    let read: Value = post(&client, &endpoint, "read", call(name, query.clone()))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(read.pointer("/result/structuredContent"), Some(&rules));
    let mut bad = query;
    bad["after"] = json!(2);
    let invalid: Value = post(&client, &endpoint, "read", call(name, bad))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_write_error(&invalid, name, "INVALID_INPUT")?;
    let options = json!({"category_id":4,"market":"swedish","filter_definition_id":2,"rules_revision":revision,"total_matches":2,"options":[{"option_id":13,"value":"blue","label":"Blue","supported_in_mcp":true},{"option_id":19,"value":null,"label":"Other blue","supported_in_mcp":false}],"next_after":null});
    let mut wire = options.clone();
    wire["options"][1]["private_note"] = json!(SENTINEL);
    let opts = json!({"category_id":4,"market":"swedish","filter_definition_id":2,"expected_rules_revision":revision,"limit":2,"search":"blue"});
    Mock::given(method("GET"))
        .and(path("/api/v1/mcp/account/listings/creation-options"))
        .and(query_param("category_id", "4"))
        .and(query_param("market", "swedish"))
        .and(query_param("filter_definition_id", "2"))
        .and(query_param("expected_rules_revision", revision.as_str()))
        .and(query_param("limit", "2"))
        .and(query_param("search", "blue"))
        .and(matches_header("authorization", "Bearer seller-read"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":wire})))
        .expect(1)
        .mount(&api)
        .await;
    let read: Value = post(&client, &endpoint, "read", call(choices, opts))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(read.pointer("/result/structuredContent"), Some(&options));
    let attrs = json!([{"filter_definition_id":2,"value":{"kind":"options","value":["blue"]}}]);
    let wire =
        json!({"expected_revision":30,"expected_rules_revision":revision,"attributes":attrs});
    let mut args = wire.clone();
    args["listing_id"] = json!(LISTING_A);
    args["request_id"] = json!(REQUEST_ID);
    Mock::given(method("PUT"))
        .and(path(format!(
            "/api/v1/mcp/account/listings/{LISTING_A}/draft-authoring/attributes"
        )))
        .and(matches_header("authorization", "Bearer seller-write"))
        .and(matches_header("idempotency-key", REQUEST_ID))
        .and(body_json(wire))
        .respond_with(ResponseTemplate::new(409).set_body_json(json!({"code":"STATE_CONFLICT"})))
        .expect(1)
        .mount(&api)
        .await;
    assert_eq!(
        post(&client, &endpoint, "read", call(replace, args.clone()))
            .await?
            .status(),
        StatusCode::FORBIDDEN
    );
    for number in ["2.50", "-0", "1.00001", "100000000000"] {
        let mut invalid = args.clone();
        invalid["attributes"][0]["value"] = json!({"kind":"number","value":number});
        let denied: Value = post(&client, &endpoint, "write", call(replace, invalid))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_write_error(&denied, replace, "INVALID_INPUT")?;
    }
    let denied: Value = post(&client, &endpoint, "write", call(replace, args))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(denied.pointer("/result/isError"), Some(&json!(true)));
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
