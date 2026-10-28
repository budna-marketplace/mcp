use std::net::{Ipv4Addr, SocketAddr};

use anyhow::{Context, Result};
use axum::Router;
use budna_mcp_core::HttpServerSettings;
use budna_mcp_server::{BudnaMcpServer, public_http_router};
use tokio_util::sync::CancellationToken;

const MCP_PATH: &str = "/mcp";

pub async fn serve(server: BudnaMcpServer, settings: HttpServerSettings) -> Result<()> {
    let cancellation = CancellationToken::new();
    let router = public_http_router(server, &settings, cancellation.clone())?;
    serve_router(router, settings, cancellation, MCP_PATH).await
}

pub(crate) async fn serve_router(
    router: Router,
    settings: HttpServerSettings,
    cancellation: CancellationToken,
    path: &'static str,
) -> Result<()> {
    let bind_address = SocketAddr::from((Ipv4Addr::LOCALHOST, settings.port()));
    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .with_context(|| format!("failed to bind Budna MCP HTTP server to {bind_address}"))?;
    let local_address = listener
        .local_addr()
        .context("failed to read Budna MCP HTTP listener address")?;

    if settings.port() == 0 {
        tracing::warn!(
            bind_address = %local_address,
            path,
            "Budna MCP assigned an ephemeral Streamable HTTP port"
        );
    } else {
        tracing::info!(
            bind_address = %local_address,
            path,
            "Budna MCP Streamable HTTP server started"
        );
    }

    let signal_cancellation = cancellation.clone();
    let signal_task = tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => signal_cancellation.cancel(),
            Err(error) => {
                tracing::warn!(error = %error, "Failed to listen for the shutdown signal")
            }
        }
    });

    let result = axum::serve(listener, router)
        .with_graceful_shutdown(cancellation.clone().cancelled_owned())
        .await;

    cancellation.cancel();
    signal_task.abort();

    result.context("Budna MCP Streamable HTTP server failed")
}
