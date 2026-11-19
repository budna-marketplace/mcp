use super::*;

#[tokio::test]
async fn room_actions_honor_separate_grants_target_kind_and_room_boundaries() -> Result<()> {
    const LIVE: &str = "live_00000000-0000-4000-8000-000000000008";
    const ACTION: &str = "00000000-0000-4000-8000-000000000003";
    const PERMISSIONS: &str = "chatrooms:read roommoderation:read roomreactions:write livereports:write roommoderation:write";
    let api = MockServer::start().await;
    exchange(&api, "host", "host-grant", USER_A, PERMISSIONS).await;
    exchange(
        &api,
        "viewer",
        "viewer-grant",
        USER_B,
        "roomreactions:write",
    )
    .await;
    let mut server = start_http_server_with_scopes(
        &api.uri(),
        30,
        Some("account-actions"),
        Some(false),
        Some(PERMISSIONS),
    )
    .await?;
    let endpoint = format!("http://{}/account/mcp", server.address);
    let client = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    for (name, scope) in [
        ("list_my_room_reaction_targets", "chatrooms:read"),
        (
            "list_my_room_moderation_participants",
            "roommoderation:read",
        ),
        ("report_live_chat_post", "livereports:write"),
        ("hide_my_room_message", "roommoderation:write"),
    ] {
        let r = post(&client, &endpoint, "viewer", call(name, json!({}))).await?;
        assert_eq!(r.status(), StatusCode::FORBIDDEN);
        assert!(
            r.headers()[header::WWW_AUTHENTICATE]
                .to_str()?
                .contains(scope)
        );
    }
    for (name, room, suffix, output) in [
        (
            "list_my_room_reaction_targets",
            LISTING_A,
            "reaction-targets",
            json!({"room_id":LISTING_A,"messages":[{"id":9,"counts":[{"emoji":"👍","count":3,"mine":true}]}],"bids":[{"id":11,"counts":[]}]}),
        ),
        (
            "list_my_room_moderation_participants",
            LIVE,
            "moderation",
            json!({"room_id":LIVE,"ban_scope":"seller_live_chats","participants":[{"user_id":USER_B,"banned":true,"expires_at":null}]}),
        ),
    ] {
        let mut wire = output.clone();
        if let Some(first) = wire.pointer_mut("/bids/0") {
            first["extra"] = json!(SENTINEL);
        }
        wire["extra"] = json!(SENTINEL);
        let _guard = Mock::given(method("GET"))
            .and(path(format!(
                "/api/v1/mcp/account/chat-rooms/{room}/{suffix}"
            )))
            .and(matches_header("authorization", "Bearer host-grant"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":wire})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let r: Value = post(
            &client,
            &endpoint,
            "host",
            call(name, json!({"room_id":room})),
        )
        .await?
        .error_for_status()?
        .json()
        .await?;
        assert_eq!(r.pointer("/result/structuredContent"), Some(&output));
        assert!(!r.to_string().contains(SENTINEL));
    }
    for (name, room, suffix, action, kind, id, args, body) in [
        (
            "set_my_room_reaction",
            LISTING_A,
            "reactions",
            "set_public_room_reaction",
            "bid",
            "11",
            json!({"target_kind":"bid","target_id":11,"emoji":"👍","reacted":false}),
            json!({"target_kind":"bid","target_id":11,"emoji":"👍","reacted":false}),
        ),
        (
            "report_live_chat_post",
            LIVE,
            "reports",
            "report_live_chat_post",
            "message",
            "7",
            json!({"message_id":7}),
            json!({"message_id":7}),
        ),
        (
            "set_my_room_message_pinned",
            LISTING_A,
            "moderation",
            "set_room_message_pinned",
            "message",
            "7",
            json!({"message_id":7,"pinned":true}),
            json!({"operation":"set_pinned","message_id":7,"pinned":true}),
        ),
        (
            "hide_my_room_message",
            LISTING_A,
            "moderation",
            "hide_room_message",
            "message",
            "7",
            json!({"message_id":7}),
            json!({"operation":"hide_message","message_id":7}),
        ),
        (
            "report_my_room_participant",
            LISTING_A,
            "moderation",
            "report_room_participant",
            "participant",
            USER_B,
            json!({"user_id":USER_B,"message_id":7}),
            json!({"operation":"report_participant","user_id":USER_B,"message_id":7}),
        ),
        (
            "ban_my_room_participant",
            LIVE,
            "moderation",
            "ban_room_participant",
            "participant",
            USER_B,
            json!({"user_id":USER_B}),
            json!({"operation":"ban_participant","user_id":USER_B}),
        ),
        (
            "unban_my_room_participant",
            LIVE,
            "moderation",
            "unban_room_participant",
            "participant",
            USER_B,
            json!({"user_id":USER_B}),
            json!({"operation":"unban_participant","user_id":USER_B}),
        ),
    ] {
        let mut input = args;
        input["room_id"] = json!(room);
        let output = json!({"room_id":room,"action_id":ACTION,"action":action,"target_kind":kind,"target_id":id,"status":"executed"});
        let _guard = Mock::given(method(if suffix == "reactions" { "PUT" } else { "POST" }))
            .and(path(format!(
                "/api/v1/mcp/account/chat-rooms/{room}/{suffix}"
            )))
            .and(body_json(body))
            .and(matches_header("authorization", "Bearer host-grant"))
            .and(header_regex("idempotency-key", "^[0-9a-f-]{36}$"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"success":true,"data":output})),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let r: Value = post(&client, &endpoint, "host", write_call(name, input))
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
    for (name, input) in [
        (
            "unban_my_room_participant",
            json!({"room_id":LISTING_A,"user_id":USER_B}),
        ),
        (
            "report_my_room_participant",
            json!({"room_id":LISTING_A,"user_id":USER_B}),
        ),
        (
            "report_my_room_participant",
            json!({"room_id":LIVE,"user_id":USER_B,"message_id":7}),
        ),
        (
            "ban_my_room_participant",
            json!({"room_id":LIVE,"user_id":USER_B,"expires_at":777}),
        ),
        (
            "report_live_chat_post",
            json!({"room_id":LISTING_A,"message_id":7}),
        ),
    ] {
        let r = post(&client, &endpoint, "host", write_call(name, input))
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
