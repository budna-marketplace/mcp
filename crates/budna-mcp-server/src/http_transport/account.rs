use std::{io, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use budna_mcp_client::CHAT_ROOMS_READ_SCOPE;
use budna_mcp_client::LIVE_REPORTS_WRITE_SCOPE;
use budna_mcp_client::PUBLIC_CHAT_WRITE_SCOPE;
use budna_mcp_client::RATINGS_WRITE_SCOPE;
use budna_mcp_client::ROOM_MODERATION_READ_SCOPE;
use budna_mcp_client::ROOM_MODERATION_WRITE_SCOPE;
use budna_mcp_client::ROOM_REACTIONS_WRITE_SCOPE;
use budna_mcp_client::{
    ACCOUNT_READ_SCOPE, AccountApiClient, BIDS_READ_SCOPE, COMPANY_READ_SCOPE, ClientError,
    DRAFT_CREATE_SCOPE, FOLLOWING_READ_SCOPE, FOLLOWING_WRITE_SCOPE, INBOX_WRITE_SCOPE,
    LISTINGS_READ_SCOPE, LIVE_REMINDER_WRITE_SCOPE, MESSAGE_REACTIONS_WRITE_SCOPE,
    MESSAGES_READ_SCOPE, MESSAGES_WRITE_SCOPE, NOTIFICATION_STATE_SCOPE, NOTIFICATIONS_READ_SCOPE,
    OFFERS_READ_SCOPE, ORDERS_READ_SCOPE, PAYMENTS_READ_SCOPE, PERSONAL_BLOCKS_READ_SCOPE,
    PERSONAL_BLOCKS_WRITE_SCOPE, PROFILE_READ_SCOPE, PROFILE_WRITE_SCOPE, RATINGS_READ_SCOPE,
    SAVED_SEARCH_WRITE_SCOPE, SELLING_READ_SCOPE, WATCHLIST_MANAGE_SCOPE, WATCHLIST_READ_SCOPE,
    WATCHLIST_WRITE_SCOPE, is_account_access_token,
};
use budna_mcp_core::{ACCOUNT_MCP_PATH, AccountSettings, HttpServerSettings};
use http::{
    HeaderValue, StatusCode,
    header::{AUTHORIZATION, CACHE_CONTROL, COOKIE, WWW_AUTHENTICATE},
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::{
    AccountMcpServer,
    account::{
        ACTION_STATUS_TOOL, DRAFT_CREATE_TOOL, DRAFT_ELIGIBILITY_TOOL, NOTIFICATION_BATCH_TOOL,
        NOTIFICATION_READ_TOOL, REMINDER_TOOL, SEARCH_AUTHORING_TOOL, SEARCH_WRITE_TOOLS,
        WATCHLIST_MANAGEMENT_TOOLS, WATCHLIST_MEMBERSHIP_TOOL,
    },
};

#[derive(Clone)]
struct AccountAuth {
    client: AccountApiClient,
    enabled_tools: Arc<std::collections::HashSet<std::borrow::Cow<'static, str>>>,
    challenge: HeaderValue,
    metadata_url: Arc<str>,
    slots: Arc<Semaphore>,
}

/// Builds the stateless account router with per-request authentication and HTTP controls.
/// The caller owns TLS, listener binding and cancellation during shutdown.
pub fn account_http_router(
    mut server: AccountMcpServer,
    client: AccountApiClient,
    settings: &HttpServerSettings,
    account: &AccountSettings,
    cancellation: CancellationToken,
) -> Result<Router, http::header::InvalidHeaderValue> {
    let configured = account
        .policy()
        .write_permissions()
        .intersection(client.write_permissions());
    let permissions = server.limit_permissions(configured);
    let client = client.with_write_permissions(permissions);
    let scopes: Vec<_> = client.supported_scopes().collect();
    server.apply_account_profile(account.policy());
    let enabled_tools = Arc::new(server.enabled_account_tools());
    let service = StreamableHttpService::new(
        move || Ok::<_, io::Error>(server.clone()),
        Arc::new(NeverSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true)
            .with_allowed_hosts(settings.allowed_hosts().iter().cloned())
            .with_allowed_origins(settings.allowed_origins().iter().cloned())
            .with_cancellation_token(cancellation.child_token()),
    );
    let metadata_url = account.metadata_url();
    let challenge = HeaderValue::from_str(&format!(
        "Bearer resource_metadata=\"{metadata_url}\", scope=\"{ACCOUNT_READ_SCOPE}\""
    ))?;
    let has_writes = client.supported_write_scopes().next().is_some();
    let auth = AccountAuth {
        client,
        enabled_tools,
        challenge,
        metadata_url: metadata_url.into(),
        slots: Arc::new(Semaphore::new(8)),
    };
    let metadata = serde_json::json!({
        "resource": account.resource(),
        "authorization_servers": [account.issuer()],
        "scopes_supported": scopes,
        "bearer_methods_supported": ["header"],
        "resource_name": if has_writes { "Budna account actions" } else { "Budna account reads" }
    });
    let router = Router::new()
        .nest_service(ACCOUNT_MCP_PATH, service)
        .route_layer(middleware::from_fn_with_state(auth, authenticate))
        .route(
            "/.well-known/oauth-protected-resource/account/mcp",
            get(move || async move { Json(metadata.clone()) }),
        );
    super::secure_router(router, settings, true)
}

async fn authenticate(
    State(auth): State<AccountAuth>,
    mut request: Request,
    next: Next,
) -> Response {
    let result = async {
        let mut credentials = request.headers().get_all(AUTHORIZATION).iter();
        let token = credentials
            .next()
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.split_once(' '))
            .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("Bearer"))
            .map(|(_, token)| token);
        let Some(token) = token.filter(|token| credentials.next().is_none() && is_account_access_token(token)) else {
            return unauthorized(&auth);
        };
        let Ok(_permit) = auth.slots.try_acquire() else {
            return StatusCode::TOO_MANY_REQUESTS.into_response();
        };
        match auth.client.exchange(token).await {
            Ok(access) => {
                if request.method() == http::Method::POST {
                    let (parts, body) = request.into_parts();
                    let body = match tokio::time::timeout(
                        Duration::from_secs(5),
                        to_bytes(body, 64 * 1024),
                    )
                    .await
                    {
                        Ok(Ok(body)) => body,
                        Ok(Err(_)) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
                        Err(_) => return StatusCode::REQUEST_TIMEOUT.into_response(),
                    };
                    if let Ok(message) = serde_json::from_slice::<serde_json::Value>(&body) {
                        let scope = if message.get("method").and_then(|value| value.as_str())
                            == Some("tools/call")
                        {
                            message
                                .pointer("/params/name")
                                .and_then(|value| value.as_str())
                                .filter(|name| auth.enabled_tools.contains(*name))
                                .and_then(|name| {
                                    if name == ACTION_STATUS_TOOL {
                                        // The API checks the specific grant and scope of the stored receipt.
                                        if auth.client.supported_write_scopes().any(|scope| access.allows(scope)) {
                                            None
                                        } else {
                                            auth.client.supported_write_scopes().next()
                                        }
                                    } else {
                                        account_tool_scope(name)
                                    }
                                })
                        } else {
                            None
                        };
                        if let Some(required) = scope.filter(|scope| !access.allows(scope)) {
                            let challenge = format!(
                                "Bearer resource_metadata=\"{}\", scope=\"{required}\", error=\"insufficient_scope\"",
                                auth.metadata_url
                            );
                            return (StatusCode::FORBIDDEN, [(WWW_AUTHENTICATE, challenge)])
                                .into_response();
                        }
                    }
                    request = Request::from_parts(parts, Body::from(body));
                }
                request.headers_mut().remove(AUTHORIZATION);
                request.headers_mut().remove(COOKIE);
                request.extensions_mut().insert(access);
                next.run(request).await
            }
            Err(ClientError::Api {
                status: 401 | 403, ..
            }) => unauthorized(&auth),
            Err(ClientError::Api { status: 429, .. }) => {
                StatusCode::TOO_MANY_REQUESTS.into_response()
            }
            Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        }
    }
    .await;
    let mut result = result;
    result
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    result
}

/// The permission each account tool needs, checked before the call reaches the tool.
pub(crate) fn account_tool_scope(name: &str) -> Option<&'static str> {
    Some(match name {
        "get_my_message_recipient"
        | "get_my_conversation_audience"
        | "get_my_message_authoring"
        | "get_my_message_reaction" => MESSAGES_READ_SCOPE,
        "start_my_conversation" | "send_my_message" | "reply_to_my_message" | "edit_my_message" => {
            MESSAGES_WRITE_SCOPE
        }
        "set_my_message_reaction" => MESSAGE_REACTIONS_WRITE_SCOPE,
        "get_my_chat_message" | "get_my_chat_posting" => MESSAGES_READ_SCOPE,
        "post_public_chat_message" => PUBLIC_CHAT_WRITE_SCOPE,
        "get_my_order_rating_eligibility" => ORDERS_READ_SCOPE,
        "rate_my_order_counterparty" => RATINGS_WRITE_SCOPE,
        "list_my_room_reaction_targets" => CHAT_ROOMS_READ_SCOPE,
        "list_my_room_moderation_participants" => ROOM_MODERATION_READ_SCOPE,
        "set_my_room_reaction" => ROOM_REACTIONS_WRITE_SCOPE,
        "report_live_chat_post" => LIVE_REPORTS_WRITE_SCOPE,
        "set_my_room_message_pinned"
        | "hide_my_room_message"
        | "report_my_room_participant"
        | "ban_my_room_participant"
        | "unban_my_room_participant" => ROOM_MODERATION_WRITE_SCOPE,
        "get_my_conversation_settings" | "get_my_listing_chat_notification_setting" => {
            MESSAGES_READ_SCOPE
        }
        "set_my_conversation_setting" | "hide_my_message_copy" | "set_my_listing_chat_muted" => {
            INBOX_WRITE_SCOPE
        }
        "get_my_profile_authoring" => PROFILE_READ_SCOPE,
        "update_my_public_profile" => PROFILE_WRITE_SCOPE,
        "get_my_following_state" => FOLLOWING_READ_SCOPE,
        "set_my_following" => FOLLOWING_WRITE_SCOPE,
        "list_my_personal_blocks" => PERSONAL_BLOCKS_READ_SCOPE,
        "set_my_personal_block" => PERSONAL_BLOCKS_WRITE_SCOPE,
        SEARCH_AUTHORING_TOOL => WATCHLIST_READ_SCOPE,
        name if SEARCH_WRITE_TOOLS.contains(&name) => SAVED_SEARCH_WRITE_SCOPE,
        NOTIFICATION_BATCH_TOOL => NOTIFICATIONS_READ_SCOPE,
        NOTIFICATION_READ_TOOL => NOTIFICATION_STATE_SCOPE,
        REMINDER_TOOL => LIVE_REMINDER_WRITE_SCOPE,
        DRAFT_CREATE_TOOL => DRAFT_CREATE_SCOPE,
        DRAFT_ELIGIBILITY_TOOL => PROFILE_READ_SCOPE,
        WATCHLIST_MEMBERSHIP_TOOL => WATCHLIST_WRITE_SCOPE,
        name if WATCHLIST_MANAGEMENT_TOOLS.contains(&name) => WATCHLIST_MANAGE_SCOPE,
        "get_my_account" => ACCOUNT_READ_SCOPE,
        "get_my_profile" | "get_my_achievements" | "get_my_referral" | "get_my_legal_documents" => {
            PROFILE_READ_SCOPE
        }
        "get_my_seller_insights"
        | "list_my_boosts"
        | "get_my_listing_promotion"
        | "get_my_shipping_settings"
        | "list_my_showcases"
        | "list_my_live_sessions"
        | "list_my_listing_tags" => SELLING_READ_SCOPE,
        "list_my_following" | "list_my_followers" => FOLLOWING_READ_SCOPE,
        "list_my_companies" | "get_my_company" => COMPANY_READ_SCOPE,
        "list_my_listings" | "get_my_listing" => LISTINGS_READ_SCOPE,
        "list_my_bids" | "list_my_auto_bids" => BIDS_READ_SCOPE,
        "list_my_offers" | "list_my_second_chance_offers" | "get_my_offer_history" => {
            OFFERS_READ_SCOPE
        }
        "list_my_watchlists"
        | "list_my_watchlist_listings"
        | "list_my_saved_searches"
        | "list_my_saved_search_matches"
        | "list_my_live_reminders" => WATCHLIST_READ_SCOPE,
        "list_my_orders"
        | "get_my_order"
        | "list_my_disputes"
        | "get_my_dispute"
        | "list_my_order_groups"
        | "get_my_order_group" => ORDERS_READ_SCOPE,
        "list_my_notifications" | "get_my_notification_preferences" => NOTIFICATIONS_READ_SCOPE,
        "list_my_ratings" => RATINGS_READ_SCOPE,
        "list_my_conversations"
        | "get_my_conversation"
        | "list_my_listing_chats"
        | "search_my_messages" => MESSAGES_READ_SCOPE,
        "get_my_money_summary"
        | "list_my_payment_activity"
        | "list_my_payouts"
        | "list_my_debts"
        | "get_my_credits"
        | "get_my_payout_setup" => PAYMENTS_READ_SCOPE,
        _ => return None,
    })
}

fn unauthorized(auth: &AccountAuth) -> Response {
    let mut response = StatusCode::UNAUTHORIZED.into_response();
    response
        .headers_mut()
        .insert(WWW_AUTHENTICATE, auth.challenge.clone());
    response
}

#[cfg(test)]
mod tests {
    use budna_mcp_client::{ClientConfig, PublicApiClient};
    use serde_json::{Value, json};
    use tower::ServiceExt;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_json, header, method, path},
    };

    use super::*;
    use crate::BudnaMcpServer;

    async fn request(
        router: &Router,
        uri: &str,
        body: Option<Value>,
        token: Option<&str>,
    ) -> Response {
        let mut request = Request::builder()
            .uri(uri)
            .header(http::header::HOST, "mcp.example.test")
            .header(http::header::ACCEPT, "application/json, text/event-stream");
        if body.is_some() {
            request = request
                .method(http::Method::POST)
                .header(http::header::CONTENT_TYPE, "application/json");
        }
        if let Some(token) = token {
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        router
            .clone()
            .oneshot(
                request
                    .body(Body::from(
                        body.map_or_else(String::new, |body| body.to_string()),
                    ))
                    .unwrap_or_else(|error| panic!("synthetic request should build: {error}")),
            )
            .await
            .unwrap_or_else(|error| panic!("router request should be infallible: {error}"))
    }

    #[tokio::test]
    async fn account_discovery_is_scoped_without_advertising_public_authentication()
    -> Result<(), Box<dyn std::error::Error>> {
        let api = MockServer::start().await;
        let resource = "https://mcp.example.test/account/mcp";
        let service_secret = "synthetic-unit-service-credential";
        Mock::given(method("POST"))
            .and(path("/api/v1/mcp/oauth/exchange"))
            .and(header("authorization", format!("Bearer {service_secret}")))
            .and(body_json(
                json!({"subject_token":"invalid","resource":resource}),
            ))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&api)
            .await;
        let config = ClientConfig::new(format!("{}/api/v1", api.uri()))?;
        let account = AccountSettings::new(&api.uri(), resource)?;
        let client = AccountApiClient::new(
            config.clone(),
            &api.uri(),
            resource,
            service_secret.to_owned(),
        )?
        .with_private_watchlist_writes(true); // Account Read must limit even a pilot-capable client.
        let public = BudnaMcpServer::new(PublicApiClient::new(config)?);
        let server = AccountMcpServer::new(client.clone(), Duration::from_secs(10))
            .with_public(public.clone());
        let settings = HttpServerSettings::new(
            0,
            vec!["mcp.example.test".to_owned()],
            vec!["https://app.example.test".to_owned()],
        )?;
        let cancellation = CancellationToken::new();
        let router =
            account_http_router(server, client, &settings, &account, cancellation.clone())?.merge(
                super::super::public_http_router(public, &settings, cancellation)?,
            );

        for uri in [
            "/.well-known/oauth-protected-resource",
            "/.well-known/oauth-protected-resource/mcp",
        ] {
            let response = request(&router, uri, None, None).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND);
            assert!(response.headers().get(WWW_AUTHENTICATE).is_none());
            assert!(to_bytes(response.into_body(), 1024).await?.is_empty());
        }

        let response = request(
            &router,
            "/.well-known/oauth-protected-resource/account/mcp",
            None,
            None,
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get(WWW_AUTHENTICATE).is_none());
        let metadata: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 16 * 1024).await?)?;
        assert_eq!(
            metadata,
            json!({
                "resource": resource,
                "authorization_servers": [api.uri()],
                "scopes_supported": budna_mcp_client::ACCOUNT_SCOPES,
                "bearer_methods_supported": ["header"],
                "resource_name": "Budna account reads"
            })
        );

        for body in [
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
                "protocolVersion":"2025-11-25","capabilities":{},
                "clientInfo":{"name":"synthetic-unit-client","version":"1"}
            }}),
            json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        ] {
            let response = request(&router, "/mcp", Some(body), None).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert!(response.headers().get(WWW_AUTHENTICATE).is_none());
            let result: Value =
                serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await?)?;
            assert!(result.get("error").is_none());
            assert!(result.get("result").is_some());
        }

        let challenge = format!(
            "Bearer resource_metadata=\"https://mcp.example.test/.well-known/oauth-protected-resource/account/mcp\", scope=\"{ACCOUNT_READ_SCOPE}\""
        );
        for token in [None, Some("invalid")] {
            let response = request(
                &router,
                "/account/mcp",
                Some(json!({"jsonrpc":"2.0","id":3,"method":"tools/list"})),
                token,
            )
            .await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(response.headers()[WWW_AUTHENTICATE], challenge);
            assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
        }
        Mock::given(method("POST"))
            .and(path("/api/v1/mcp/oauth/exchange"))
            .and(header("authorization", format!("Bearer {service_secret}")))
            .and(body_json(
                json!({"subject_token":"account-only","resource":resource}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token":"synthetic-api-token","token_type":"Bearer","expires_in":60,
                "scope":ACCOUNT_READ_SCOPE,"subject":"user_00000000-0000-4000-8000-000000000001",
                "grant_id":"synthetic-grant","client_id":"synthetic-client","resource":resource
            })))
            .expect(2)
            .mount(&api)
            .await;
        for (tool, scope) in [
            ("list_my_watchlists", WATCHLIST_READ_SCOPE),
            ("list_my_orders", ORDERS_READ_SCOPE),
        ] {
            let response = request(
                &router,
                ACCOUNT_MCP_PATH,
                Some(json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":tool,"arguments":{}}})),
                Some("account-only"),
            )
            .await;
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert_eq!(
                response.headers()[WWW_AUTHENTICATE],
                format!(
                    "Bearer resource_metadata=\"{}\", scope=\"{scope}\", error=\"insufficient_scope\"",
                    account.metadata_url()
                )
            );
            assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
        }
        api.verify().await;
        Ok(())
    }
}
