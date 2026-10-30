use std::{io, sync::Arc};

use axum::{
    Router,
    extract::{Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use budna_mcp_core::HttpServerSettings;
use http::{
    HeaderMap, HeaderName, HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HOST, ORIGIN, WWW_AUTHENTICATE},
    uri::Authority,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::never::NeverSessionManager,
};
use tokio_util::sync::CancellationToken;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
};

use crate::BudnaMcpServer;

mod account;

pub use account::account_http_router;
#[cfg(test)]
pub(crate) use account::account_tool_scope;

/// Explicit alias for the stateless public Explore router.
pub use public_http_router as stateless_public_http_router;

const MCP_PATH: &str = "/mcp";
const HTTP_REQUEST_BODY_LIMIT_BYTES: usize = 64 * 1024;
const LAST_EVENT_ID: HeaderName = HeaderName::from_static("last-event-id");
const MCP_SESSION_ID: HeaderName = HeaderName::from_static("mcp-session-id");
const MCP_PROTOCOL_VERSION: HeaderName = HeaderName::from_static("mcp-protocol-version");

#[derive(Clone)]
struct HttpAccessPolicy {
    allowed_hosts: Arc<[String]>,
    allowed_origins: Arc<[String]>,
}

/// Builds the stateless public Explore router with body, Host, Origin and CORS controls.
/// Requests receive JSON responses without MCP sessions or resumable SSE streams.
/// The caller owns the listener and cancels `cancellation` during shutdown.
pub fn public_http_router(
    server: BudnaMcpServer,
    settings: &HttpServerSettings,
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

    let router = Router::new().nest_service(MCP_PATH, service);
    secure_router(router, settings, false)
}

fn secure_router(
    router: Router,
    settings: &HttpServerSettings,
    account: bool,
) -> Result<Router, http::header::InvalidHeaderValue> {
    let cors = cors_layer(settings, account)?;
    let access_policy = HttpAccessPolicy {
        allowed_hosts: Arc::from(settings.allowed_hosts().to_vec()),
        allowed_origins: Arc::from(settings.allowed_origins().to_vec()),
    };
    Ok(router
        .layer(RequestBodyLimitLayer::new(HTTP_REQUEST_BODY_LIMIT_BYTES))
        .layer(cors)
        .layer(middleware::from_fn_with_state(
            access_policy,
            validate_http_access,
        )))
}

fn cors_layer(
    settings: &HttpServerSettings,
    account: bool,
) -> Result<CorsLayer, http::header::InvalidHeaderValue> {
    let allowed_origins = settings
        .allowed_origins()
        .iter()
        .map(|origin| HeaderValue::from_str(origin))
        .collect::<Result<Vec<_>, _>>()?;

    let mut allowed_headers = vec![
        ACCEPT,
        CONTENT_TYPE,
        LAST_EVENT_ID,
        MCP_SESSION_ID,
        MCP_PROTOCOL_VERSION,
    ];
    let mut exposed_headers = vec![MCP_SESSION_ID, MCP_PROTOCOL_VERSION];
    if account {
        allowed_headers.push(AUTHORIZATION);
        exposed_headers.push(WWW_AUTHENTICATE);
    }
    Ok(CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_headers(allowed_headers)
        .expose_headers(exposed_headers))
}

async fn validate_http_access(
    State(policy): State<HttpAccessPolicy>,
    request: Request,
    next: Next,
) -> Response {
    if !has_allowed_host(request.headers(), &policy.allowed_hosts)
        || !has_allowed_origin(request.headers(), &policy.allowed_origins)
    {
        return http::StatusCode::FORBIDDEN.into_response();
    }

    next.run(request).await
}

fn has_allowed_host(headers: &HeaderMap, allowed_hosts: &[String]) -> bool {
    let Some(host) = single_header_value(headers, HOST) else {
        return false;
    };
    let Ok(host) = host.to_str() else {
        return false;
    };
    let Ok(host) = Authority::try_from(host) else {
        return false;
    };
    let normalized_host = host.as_str().to_ascii_lowercase();

    allowed_hosts.iter().any(|allowed| {
        if allowed == &normalized_host {
            return true;
        }

        let Ok(allowed) = Authority::try_from(allowed.as_str()) else {
            return false;
        };
        allowed.port().is_none() && allowed.host().eq_ignore_ascii_case(host.host())
    })
}

fn has_allowed_origin(headers: &HeaderMap, allowed_origins: &[String]) -> bool {
    let Some(origin) = single_header_value(headers, ORIGIN) else {
        return headers.get_all(ORIGIN).iter().next().is_none();
    };

    allowed_origins
        .iter()
        .any(|allowed| origin.as_bytes() == allowed.as_bytes())
}

fn single_header_value(headers: &HeaderMap, name: HeaderName) -> Option<&HeaderValue> {
    let mut values = headers.get_all(name).iter();
    let value = values.next()?;
    values.next().is_none().then_some(value)
}
