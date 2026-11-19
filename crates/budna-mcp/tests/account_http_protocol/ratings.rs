use super::*;

#[tokio::test]
async fn ratings_bind_an_owned_order_stars_and_the_customers_exact_comment_choice() -> Result<()> {
    let api = MockServer::start().await;
    exchange(
        &api,
        "owner",
        "owner-grant",
        USER_A,
        "orders:read ratings:write",
    )
    .await;
    exchange(&api, "reader", "read-grant", USER_B, "orders:read").await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("ratings:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let facts = json!({"order_id":7,"listing_id":LISTING_A,"counterparty_username":"seller_7","eligible":true,"audience":"marketplace"});
    let route = "/api/v1/mcp/account/orders/7/rating";
    let _preview = Mock::given(method("GET"))
        .and(path(route))
        .and(matches_header("authorization", "Bearer owner-grant"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":facts})),
        )
        .expect(1)
        .mount_as_scoped(&api)
        .await;
    let preview: Value = post(
        &client,
        &endpoint,
        "owner",
        call("get_my_order_rating_eligibility", json!({"order_id":7})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(preview.pointer("/result/structuredContent"), Some(&facts));
    let name = "rate_my_order_counterparty";
    assert_eq!(
        post(&client, &endpoint, "reader", write_call(name, json!({})))
            .await?
            .status(),
        StatusCode::FORBIDDEN
    );
    for bad in [
        json!({"order_id":7,"stars":4,"audience":"marketplace"}),
        json!({"order_id":7,"stars":4,"audience":"marketplace","comment":null,"ratee_id":USER_B}),
        json!({"order_id":7,"stars":6,"audience":"marketplace","comment":null}),
        json!({"order_id":7,"stars":2,"audience":"private","comment":"Great"}),
    ] {
        let r = post(&client, &endpoint, "owner", write_call(name, bad))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_write_error(&r, name, "INVALID_INPUT")?;
    }
    for comment in [Value::Null, json!("Accurately described")] {
        let receipt = json!({"order_id":7,"rating_id":12,"action_id":"00000000-0000-4000-8000-000000000003","status":"executed"});
        let mut wire = receipt.clone();
        wire["extra"] = json!(SENTINEL);
        let _submitted = Mock::given(method("POST"))
            .and(path(route))
            .and(body_json(
                json!({"stars":4,"comment":comment,"audience":"marketplace"}),
            ))
            .and(matches_header("authorization", "Bearer owner-grant"))
            .and(header_regex("idempotency-key", "^[0-9a-f-]{36}$"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":wire})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let response: Value = post(
            &client,
            &endpoint,
            "owner",
            write_call(
                name,
                json!({"order_id":7,"stars":4,"comment":comment,"audience":"marketplace"}),
            ),
        )
        .await?
        .error_for_status()?
        .json()
        .await?;
        assert_eq!(
            response.pointer("/result/structuredContent"),
            Some(&receipt)
        );
        assert!(!response.to_string().contains(SENTINEL));
    }
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
