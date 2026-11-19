use super::*;

const LISTING: &str = "listing_00000000-0000-4000-8000-000000000004";
const KEY: &str = "00000000-0000-4000-8000-000000000005";
const ACTION: &str = "00000000-0000-4000-8000-000000000006";

fn membership_request(saved: bool) -> wiremock::MockBuilder {
    Mock::given(method("PUT"))
        .and(path(format!(
            "/api/v1/mcp/account/watchlists/7/listings/{LISTING}"
        )))
        .and(header("authorization", "Bearer delegated"))
        .and(header("idempotency-key", KEY))
        .and(body_json(json!({"saved": saved})))
}

fn membership_receipt(saved: bool, status: &str) -> serde_json::Value {
    json!({"success":true,"data":{
        "action_id": ACTION, "watchlist_id":7, "listing_id":LISTING, "saved":saved,
        "status": status, "extra_field":"ignored"
    }})
}

#[tokio::test]
async fn watchlist_write_grant_is_limited_to_the_enabled_profile() -> Result<(), ClientError> {
    let api = MockServer::start().await;
    let read = client(&api);
    let pilot = read.clone().with_private_watchlist_writes(true);
    for (i, scope, allowed) in [
        (0, WATCHLIST_WRITE_SCOPE.to_owned(), true),
        (
            1,
            format!("{} {WATCHLIST_WRITE_SCOPE}", ACCOUNT_SCOPES.join(" ")),
            true,
        ),
        (
            2,
            format!("{WATCHLIST_WRITE_SCOPE} {WATCHLIST_WRITE_SCOPE}"),
            false,
        ),
        (3, format!("{WATCHLIST_WRITE_SCOPE} offers:write"), false),
    ] {
        let token = format!("incoming-{i}");
        exchange(&api, &token, "delegated", USER_A, &scope).await;
        assert!(read.exchange(&token).await.is_err());
        assert_eq!(pilot.exchange(&token).await.is_ok(), allowed);
    }
    exchange(&api, "read-only", "delegated", USER_A, WATCHLIST_READ_SCOPE).await;
    let access = pilot.exchange("read-only").await?;
    let denied = pilot
        .set_my_watchlist_membership(&access, 7, LISTING, true, KEY)
        .await;
    assert!(matches!(denied, Err(ClientError::Api { status: 403, .. })));

    let access = pilot.exchange("incoming-0").await?;
    let disabled = read
        .set_my_watchlist_membership(&access, 7, LISTING, true, KEY)
        .await;
    assert!(matches!(
        disabled,
        Err(ClientError::Api { status: 403, .. })
    ));
    for (id, listing, key) in [
        (0, LISTING, KEY),
        (7, "not-a-listing", KEY),
        (7, LISTING, "bad key"),
        (7, LISTING, "00000000-0000-0000-0000-000000000000"),
    ] {
        assert!(matches!(
            pilot
                .set_my_watchlist_membership(&access, id, listing, true, key)
                .await,
            Err(ClientError::InvalidInput { .. })
        ));
    }
    assert!(
        api.received_requests()
            .await
            .unwrap_or_default()
            .iter()
            .all(|r| r.method == "POST")
    );
    Ok(())
}

#[tokio::test]
async fn watchlist_membership_receipt_is_bound_to_the_requested_change() -> Result<(), ClientError>
{
    let api = MockServer::start().await;
    exchange(&api, "incoming", "delegated", USER_A, WATCHLIST_WRITE_SCOPE).await;
    let pilot = client(&api).with_private_watchlist_writes(true);
    let access = pilot.exchange("incoming").await?;
    for (saved, status, expected) in [
        (true, "executed", AccountWatchlistActionStatus::Executed),
        (false, "executing", AccountWatchlistActionStatus::Executing),
        (false, "unknown", AccountWatchlistActionStatus::Unknown),
    ] {
        let _route = membership_request(saved)
            .respond_with(
                ResponseTemplate::new(200).set_body_json(membership_receipt(saved, status)),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let result = pilot
            .set_my_watchlist_membership(&access, 7, LISTING, saved, KEY)
            .await?;
        assert_eq!(result.action_id, ACTION);
        assert_eq!(result.watchlist_id, 7);
        assert_eq!(result.listing_id, LISTING);
        assert_eq!(result.saved, saved);
        assert_eq!(result.status, expected);
    }
    for (field, value) in [
        ("action_id", json!("bad-reference")),
        ("watchlist_id", json!(8)),
        (
            "listing_id",
            json!("listing_00000000-0000-4000-8000-000000000098"),
        ),
        ("saved", json!(false)),
        ("status", json!("future_result")),
    ] {
        let mut body = membership_receipt(true, "executed");
        body["data"][field] = value;
        let _route = membership_request(true)
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let error = pilot
            .set_my_watchlist_membership(&access, 7, LISTING, true, KEY)
            .await
            .err()
            .unwrap_or_else(|| panic!("invalid receipt was accepted"));
        assert_eq!(error.public_code(), Some("ACTION_OUTCOME_UNKNOWN"));
        assert!(!error.retryable());
    }
    Ok(())
}

#[tokio::test]
async fn watchlist_write_failures_are_not_retried_or_recommended_for_retry()
-> Result<(), ClientError> {
    let api = MockServer::start().await;
    exchange(&api, "incoming", "delegated", USER_A, WATCHLIST_WRITE_SCOPE).await;
    let pilot = client(&api).with_private_watchlist_writes(true);
    let access = pilot.exchange("incoming").await?;
    for (status, code) in [
        (404, "API_NOT_FOUND"),
        (409, "API_CONFLICT"),
        (429, "API_RATE_LIMITED"),
        (408, "ACTION_OUTCOME_UNKNOWN"),
        (499, "ACTION_OUTCOME_UNKNOWN"),
        (503, "ACTION_OUTCOME_UNKNOWN"),
    ] {
        let _route = membership_request(true)
            .respond_with(
                ResponseTemplate::new(status).set_body_string("untrusted-backend-sentinel"),
            )
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        let error = pilot
            .set_my_watchlist_membership(&access, 7, LISTING, true, KEY)
            .await
            .err()
            .unwrap_or_else(|| panic!("write failure reported as success"));
        assert_eq!(error.public_code(), Some(code));
        assert!(!error.retryable());
        assert!(!error.public_message().contains("sentinel"));
    }
    Ok(())
}

#[tokio::test]
async fn watchlist_reads_keep_unknown_visibility_distinct_from_private_and_public()
-> Result<(), ClientError> {
    let api = MockServer::start().await;
    exchange(&api, "incoming", "delegated", USER_A, WATCHLIST_READ_SCOPE).await;
    let read = client(&api);
    let access = read.exchange("incoming").await?;
    for visibility in [None, Some(true), Some(false)] {
        let mut list = json!({"id":7,"name":"Sample","is_default":false,"item_count":0,"created_at":0,"updated_at":0});
        if let Some(value) = visibility {
            list["is_private"] = json!(value);
        }
        let _route = Mock::given(method("GET"))
            .and(path("/api/v1/mcp/account/watchlists"))
            .respond_with(ResponseTemplate::new(200).set_body_json(activity_page(json!([list]))))
            .expect(1)
            .mount_as_scoped(&api)
            .await;
        assert_eq!(
            read.watchlists(&access, 1, 10).await?.items[0].is_private,
            visibility
        );
    }
    Ok(())
}
