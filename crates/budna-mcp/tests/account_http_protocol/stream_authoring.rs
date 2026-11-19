use super::*;
const STREAM: &str = "live_00000000-0000-4000-8000-000000000091";
const KEY: &str = "00000000-0000-4000-8000-000000000092";

#[tokio::test]
async fn stream_planning_binds_reviewed_resource_and_separates_authoring_from_writing() -> Result<()>
{
    let api = MockServer::start().await;
    exchange(&api, "read", "read-stream", USER_A, "streamauthoring:read").await;
    exchange(&api, "write", "write-stream", USER_A, "streamplans:write").await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("streamauthoring:read streamplans:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let route = "/api/v1/mcp/account/selling/stream-authoring";
    let rev = "x".repeat(43);
    let session = json!({"stream_id":STREAM,"status":"scheduled","scheduled_start_at":4000,"title":"Synthetic schedule","description":"Untrusted description","notify_followers":true,"listing_ids":[LISTING_A]});
    let mut wire = session.clone();
    wire["extra"] = json!(SENTINEL);
    Mock::given(method("GET")).and(path(route)).and(wiremock::matchers::query_param("stream_id",STREAM)).and(matches_header("authorization","Bearer read-stream"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"revision":rev,"target":{"kind":"shop"},"seller_verified":true,"seller_ready":true,"target_allows_scheduling":true,"target_ends_at":null,"session":wire,"max_advance_days":30,"max_lineup":200,"extra":SENTINEL}}))).expect(1).mount(&api).await;
    let data: Value = post(
        &client,
        &endpoint,
        "read",
        call("get_my_stream_authoring", json!({"stream_id":STREAM})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    let out = &data["result"]["structuredContent"];
    assert_eq!(out["session"], session);
    assert_eq!(out["target"], json!({"kind":"shop"}));
    assert_eq!(out["revision"], rev);
    assert!(out["untrusted_content_notice"].is_string());
    assert_eq!(out.as_object().context("authoring projection")?.len(), 10);
    let op = "change_my_stream_plan";
    let change = json!({"operation":"set_announced","stream_id":STREAM,"listing_id":LISTING_A,"announced":false});
    let input = json!({"expected_revision":rev,"request_id":KEY,"change":change});
    assert_eq!(
        post(&client, &endpoint, "read", call(op, input.clone()))
            .await?
            .status(),
        StatusCode::FORBIDDEN
    );
    let bad = json!({"expected_revision":rev,"request_id":KEY,"change":{"operation":"set_announced","stream_id":STREAM,"listing_id":LISTING_A,"announced":false,"seller_id":2}});
    let rejected: Value = post(&client, &endpoint, "write", call(op, bad))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(rejected["result"]["isError"], true);
    let receipt = json!({"action_id":KEY,"action":"change_stream_lineup","stream_id":STREAM,"status":"executed"});
    Mock::given(method("POST")).and(path(route)).and(matches_header("authorization","Bearer write-stream")).and(matches_header("idempotency-key",KEY)).and(body_json(json!({"expected_revision":rev,"operation":"set_announced","stream_id":STREAM,"listing_id":LISTING_A,"announced":false})))
      .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":receipt}))).expect(1).mount(&api).await;
    let changed: Value = post(&client, &endpoint, "write", call(op, input))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(changed["result"]["structuredContent"], receipt);
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
