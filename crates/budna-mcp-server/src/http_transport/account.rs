use std::{io, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use budna_mcp_client::{
    ACCOUNT_READ_SCOPE, ACCOUNT_SCOPES, AccountApiClient, BIDS_READ_SCOPE, ClientError,
    LISTINGS_READ_SCOPE, MESSAGES_READ_SCOPE, NOTIFICATIONS_READ_SCOPE, OFFERS_READ_SCOPE,
    ORDERS_READ_SCOPE, PAYMENTS_READ_SCOPE, PROFILE_READ_SCOPE, RATINGS_READ_SCOPE,
    WATCHLIST_READ_SCOPE,
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

use crate::AccountMcpServer;

#[derive(Clone)]
struct AccountAuth {
    client: AccountApiClient,
    challenge: HeaderValue,
    slots: Arc<Semaphore>,
}

/// Builds the stateless account router with per-request authentication and HTTP controls.
/// The caller owns TLS, listener binding and cancellation during shutdown.
pub fn account_http_router(
    server: AccountMcpServer,
    client: AccountApiClient,
    settings: &HttpServerSettings,
    account: &AccountSettings,
    cancellation: CancellationToken,
) -> Result<Router, http::header::InvalidHeaderValue> {
    let service = StreamableHttpService::new(
        move || Ok::<_, io::Error>(server.clone()),
        Arc::new(NeverSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_stateful_mode(false)
            .with_json_response(true)
            .with_allowed_hosts(settings.allowed_hosts().iter().cloned())
            .with_allowed_origins(settings.allowed_origins().iter().cloned())
            .with_cancellation_token(cancellation.child_token()),
    );
    let challenge = HeaderValue::from_str(&format!(
        "Bearer resource_metadata=\"{}\", scope=\"{}\"",
        account.metadata_url(),
        ACCOUNT_SCOPES.join(" ")
    ))?;
    let auth = AccountAuth {
        client,
        challenge,
        slots: Arc::new(Semaphore::new(8)),
    };
    let metadata = serde_json::json!({
        "resource": account.resource(),
        "authorization_servers": [account.issuer()],
        "scopes_supported": ACCOUNT_SCOPES,
        "bearer_methods_supported": ["header"],
        "resource_name": "Budna account reads"
    });
    let router = Router::new()
        .nest_service(ACCOUNT_MCP_PATH, service)
        .route_layer(middleware::from_fn_with_state(auth, authenticate))
        .route(
            "/.well-known/oauth-protected-resource/account/mcp",
            get({
                let metadata = metadata.clone();
                move || async move { Json(metadata.clone()) }
            }),
        )
        .route(
            "/.well-known/oauth-protected-resource",
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
        let Some(token) = token.filter(|_| credentials.next().is_none()) else {
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
                                .and_then(account_tool_scope)
                        } else {
                            None
                        };
                        if scope.is_some_and(|scope| !access.allows(scope)) {
                            let mut response = StatusCode::FORBIDDEN.into_response();
                            let challenge = format!(
                                "{}, error=\"insufficient_scope\"",
                                auth.challenge.to_str().unwrap_or("Bearer")
                            );
                            if let Ok(value) = HeaderValue::from_str(&challenge) {
                                response.headers_mut().insert(WWW_AUTHENTICATE, value);
                            }
                            return response;
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
        "get_my_account" => ACCOUNT_READ_SCOPE,
        "get_my_profile" => PROFILE_READ_SCOPE,
        "list_my_listings" | "get_my_listing" => LISTINGS_READ_SCOPE,
        "list_my_bids" | "list_my_auto_bids" => BIDS_READ_SCOPE,
        "list_my_offers" | "list_my_second_chance_offers" => OFFERS_READ_SCOPE,
        "list_my_watchlists" | "list_my_watchlist_listings" | "list_my_saved_searches" => {
            WATCHLIST_READ_SCOPE
        }
        "list_my_orders" | "get_my_order" => ORDERS_READ_SCOPE,
        "list_my_notifications" => NOTIFICATIONS_READ_SCOPE,
        "list_my_ratings" => RATINGS_READ_SCOPE,
        "list_my_conversations" | "get_my_conversation" => MESSAGES_READ_SCOPE,
        "get_my_money_summary"
        | "list_my_payment_activity"
        | "list_my_payouts"
        | "list_my_debts" => PAYMENTS_READ_SCOPE,
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
