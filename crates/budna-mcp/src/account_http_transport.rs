use anyhow::Result;
use budna_mcp_client::AccountApiClient;
use budna_mcp_core::{ACCOUNT_MCP_PATH, AccountSettings, HttpServerSettings};
use budna_mcp_server::{AccountMcpServer, account_http_router};
use tokio_util::sync::CancellationToken;

pub async fn serve(
    server: AccountMcpServer,
    client: AccountApiClient,
    settings: HttpServerSettings,
    account: &AccountSettings,
) -> Result<()> {
    let cancellation = CancellationToken::new();
    let router = account_http_router(server, client, &settings, account, cancellation.clone())?;
    crate::http_transport::serve_router(router, settings, cancellation, ACCOUNT_MCP_PATH).await
}
