use super::*;

#[tokio::test]
async fn private_text_and_reactions_bind_audience_target_and_independent_grants() -> Result<()> {
    const ACTION: &str = "00000000-0000-4000-8000-000000000008";
    let api = MockServer::start().await;
    exchange(
        &api,
        "writer",
        "text-token",
        USER_A,
        "messages:write messages:read",
    )
    .await;
    exchange(
        &api,
        "reactor",
        "reaction-token",
        USER_A,
        "messagereactions:write messages:read",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("messages:write messagereactions:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let list: Value = post(
        &client,
        &endpoint,
        "writer",
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    let tools = list["result"]["tools"].as_array().context("tools")?;
    let reaction_schema = &tools
        .iter()
        .find(|t| t["name"] == "set_my_message_reaction")
        .context("reaction tool")?["inputSchema"];
    assert!(
        reaction_schema["required"]
            .as_array()
            .context("required")?
            .contains(&json!("reaction"))
    );
    assert!(
        reaction_schema["properties"]["reaction"]
            .to_string()
            .contains("\"null\""),
        "clearing must be discoverable: {reaction_schema}"
    );
    for (name, token) in [
        ("send_my_message", "reactor"),
        ("set_my_message_reaction", "writer"),
    ] {
        assert_eq!(
            post(&client, &endpoint, token, call(name, json!({})))
                .await?
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    for (name, input, url, output) in [
        (
            "get_my_message_recipient",
            json!({"username":"maja"}),
            "message-recipients/maja",
            json!({"username":"maja","user_id":USER_B}),
        ),
        (
            "get_my_conversation_audience",
            json!({"conversation_id":7}),
            "conversations/7/audience",
            json!({"conversation_id":7,"recipient_username":"maja","recipient_user_id":USER_B,"listing_id":null}),
        ),
        (
            "get_my_message_authoring",
            json!({"message_id":13}),
            "messages/13/authoring",
            json!({"conversation_id":7,"message_id":13,"revision":123,"can_edit":true}),
        ),
        (
            "get_my_message_reaction",
            json!({"conversation_id":7,"message_id":13}),
            "conversations/7/messages/13/reaction",
            json!({"conversation_id":7,"message_id":13,"reaction":"❤️","counts":[{"emoji":"❤️","count":1}]}),
        ),
    ] {
        let mut wire = output.clone();
        wire["extra"] = json!(SENTINEL);
        let _guard = Mock::given(method("GET"))
            .and(path(format!("/api/v1/mcp/account/{url}")))
            .and(matches_header("authorization", "Bearer text-token"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":wire})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let r: Value = post(&client, &endpoint, "writer", call(name, input))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_eq!(
            r.pointer("/result/structuredContent"),
            Some(&output),
            "{name}"
        );
    }
    let base =
        json!({"recipient_username":"maja","recipient_user_id":USER_B,"text":"Line one\nLine two"});
    for (name, verb, url, extras, body_extras) in [
        (
            "start_my_conversation",
            "POST",
            "conversations/start",
            json!({"listing_id":LISTING_A}),
            json!({"listing_id":LISTING_A}),
        ),
        (
            "send_my_message",
            "POST",
            "conversations/7/messages",
            json!({"conversation_id":7}),
            json!({}),
        ),
        (
            "reply_to_my_message",
            "POST",
            "messages/12/replies",
            json!({"conversation_id":7,"message_id":12}),
            json!({"conversation_id":7}),
        ),
        (
            "edit_my_message",
            "PUT",
            "messages/13/text",
            json!({"conversation_id":7,"message_id":13,"expected_revision":123}),
            json!({"conversation_id":7,"expected_revision":123}),
        ),
    ] {
        let mut input = base.clone();
        input
            .as_object_mut()
            .context("input")?
            .extend(extras.as_object().context("keys")?.clone());
        let mut body = base.clone();
        body.as_object_mut()
            .context("body")?
            .extend(body_extras.as_object().context("keys")?.clone());
        let receipt =
            json!({"action_id":ACTION,"conversation_id":7,"message_id":13,"status":"executed"});
        let _guard = Mock::given(method(verb))
            .and(path(format!("/api/v1/mcp/account/{url}")))
            .and(body_json(body))
            .and(matches_header("authorization", "Bearer text-token"))
            .and(header_regex("Idempotency-Key", "^[0-9a-f-]{36}$"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":receipt})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let r: Value = post(&client, &endpoint, "writer", write_call(name, input))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_eq!(
            r.pointer("/result/structuredContent"),
            Some(&receipt),
            "{name}"
        );
    }
    for reaction in [json!("👍"), Value::Null] {
        let input = json!({"conversation_id":7,"message_id":13,"reaction":reaction});
        let result = json!({"action_id":ACTION,"conversation_id":7,"message_id":13,"reaction":reaction,"status":"executed"});
        let _guard = Mock::given(method("PUT"))
            .and(path(
                "/api/v1/mcp/account/conversations/7/messages/13/reaction",
            ))
            .and(body_json(json!({"reaction":reaction})))
            .and(matches_header("authorization", "Bearer reaction-token"))
            .and(header_regex("Idempotency-Key", "^[0-9a-f-]{36}$"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":result})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let r: Value = post(
            &client,
            &endpoint,
            "reactor",
            write_call("set_my_message_reaction", input),
        )
        .await?
        .error_for_status()?
        .json()
        .await?;
        assert_eq!(r.pointer("/result/structuredContent"), Some(&result));
    }
    for (name, token, input) in [
        (
            "set_my_message_reaction",
            "reactor",
            json!({"conversation_id":7,"message_id":13}),
        ),
        (
            "set_my_message_reaction",
            "reactor",
            json!({"conversation_id":7,"message_id":13,"reaction":"❤"}),
        ),
        (
            "start_my_conversation",
            "writer",
            json!({"recipient_username":"maja","recipient_user_id":USER_A,"text":"hi"}),
        ),
        (
            "start_my_conversation",
            "writer",
            json!({"recipient_username":"maja","recipient_user_id":USER_B,"text":"hi\u{202e}"}),
        ),
        (
            "start_my_conversation",
            "writer",
            json!({"recipient_username":"maja","recipient_user_id":USER_B,"text":"  "}),
        ),
    ] {
        let r: Value = post(&client, &endpoint, token, write_call(name, input))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_write_error(&r, name, "INVALID_INPUT")?;
    }
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}

#[tokio::test]
async fn public_post_requires_its_own_grant_and_an_explicit_public_audience() -> Result<()> {
    const ACTION: &str = "00000000-0000-4000-8000-000000000008";
    let api = MockServer::start().await;
    exchange(&api, "private", "dm-token", USER_A, "messages:write").await;
    exchange(
        &api,
        "room",
        "room-token",
        USER_A,
        "messages:read publicchat:write",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some("publicchat:write messages:write"),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let post_tool = "post_public_chat_message";
    let input = json!({"room_id":LISTING_A,"audience":"public","text":"Question about the lot","reply_to_message_id":9});
    let unauthorized = post(
        &client,
        &endpoint,
        "private",
        write_call(post_tool, input.clone()),
    )
    .await?;
    assert_eq!(unauthorized.status(), StatusCode::FORBIDDEN);
    assert!(
        unauthorized.headers()[header::WWW_AUTHENTICATE]
            .to_str()?
            .contains("publicchat:write")
    );
    for invalid in [
        json!({"room_id":LISTING_A,"text":"hi"}),
        json!({"room_id":LISTING_A,"audience":"private","text":"hi"}),
        json!({"room_id":USER_A,"audience":"public","text":"hi"}),
    ] {
        let r = post(&client, &endpoint, "room", write_call(post_tool, invalid))
            .await?
            .error_for_status()?
            .json()
            .await?;
        assert_write_error(&r, post_tool, "INVALID_INPUT")?;
    }
    let posting = json!({"room_id":LISTING_A,"audience":"public","role":"viewer","can_post":true,"blocker":null,"cooldown_seconds":5,"max_text_length":500,"replyable_message_ids":[9]});
    let _preview = Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/mcp/account/chat-rooms/{LISTING_A}/posting"
        )))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":posting})),
        )
        .expect(1)
        .mount_as_scoped(&api)
        .await;
    let r: Value = post(
        &client,
        &endpoint,
        "room",
        call("get_my_chat_posting", json!({"room_id":LISTING_A})),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(r.pointer("/result/structuredContent"), Some(&posting));
    let _read=Mock::given(method("GET")).and(path(format!("/api/v1/mcp/account/chat-rooms/{LISTING_A}/messages/9"))).respond_with(ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":{"room_id":LISTING_A,"message_id":9,"text":"a\u{202e}b","is_host":true,"reply_to_message_id":null,"extra":SENTINEL}}))).expect(1).mount_as_scoped(&api).await;
    let read: Value = post(
        &client,
        &endpoint,
        "room",
        call(
            "get_my_chat_message",
            json!({"room_id":LISTING_A,"message_id":9}),
        ),
    )
    .await?
    .error_for_status()?
    .json()
    .await?;
    assert_eq!(
        read.pointer("/result/structuredContent/text"),
        Some(&json!("ab"))
    );
    assert!(!read.to_string().contains(SENTINEL));
    let receipt =
        json!({"room_id":LISTING_A,"message_id":10,"action_id":ACTION,"status":"executed"});
    let _submit = Mock::given(method("POST"))
        .and(path(format!(
            "/api/v1/mcp/account/chat-rooms/{LISTING_A}/messages"
        )))
        .and(body_json(
            json!({"audience":"public","text":"Question about the lot","reply_to_message_id":9}),
        ))
        .and(matches_header("authorization", "Bearer room-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":receipt})),
        )
        .expect(1)
        .mount_as_scoped(&api)
        .await;
    let r: Value = post(&client, &endpoint, "room", write_call(post_tool, input))
        .await?
        .error_for_status()?
        .json()
        .await?;
    assert_eq!(r.pointer("/result/structuredContent"), Some(&receipt));
    server.child.kill().await?;
    server.child.wait().await?;
    Ok(())
}
