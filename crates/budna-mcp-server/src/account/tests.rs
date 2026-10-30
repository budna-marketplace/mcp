use std::future;

use budna_mcp_client::{AccountSummary, ClientConfig};
use serde_json::json;

use super::*;

fn server(timeout: Duration) -> AccountMcpServer {
    let config = ClientConfig::new("https://api.example.test/api/v1")
        .unwrap_or_else(|error| panic!("synthetic account URL: {error}"));
    let client = AccountApiClient::new(
        config,
        "https://api.example.test",
        "https://mcp.example.test/account/mcp",
        "synthetic-service-credential-for-unit-tests".to_owned(),
    )
    .unwrap_or_else(|error| panic!("synthetic account client: {error}"));
    AccountMcpServer::new(client, timeout)
}

#[tokio::test]
async fn account_tool_admission_is_bounded_and_recovers_after_release() {
    let server = server(Duration::from_secs(1));
    let permits = server
        .slots
        .acquire_many(8)
        .await
        .unwrap_or_else(|error| panic!("hold account slots: {error}"));
    let Err(error) = server
        .execute::<AccountSummary, AccountOutput, _>(
            "get_my_account",
            future::pending::<Result<AccountSummary, ClientError>>(),
        )
        .await
    else {
        panic!("account admission should time out while every slot is held");
    };
    assert_eq!(error.is_error, Some(true));
    assert_eq!(
        error
            .structured_content
            .as_ref()
            .and_then(|payload| payload.pointer("/error/code")),
        Some(&json!("SERVER_BUSY"))
    );
    drop(permits);
    assert_eq!(server.slots.available_permits(), 8);
}

#[tokio::test]
async fn account_tool_timeout_cancels_work_and_releases_its_slot() {
    let server = server(Duration::from_millis(5));
    let Err(error) = server
        .execute::<AccountSummary, AccountOutput, _>(
            "get_my_account",
            future::pending::<Result<AccountSummary, ClientError>>(),
        )
        .await
    else {
        panic!("stalled account work should hit its operation timeout");
    };
    assert_eq!(error.is_error, Some(true));
    assert_eq!(
        error
            .structured_content
            .as_ref()
            .and_then(|payload| payload.pointer("/error/code")),
        Some(&json!("OPERATION_TIMEOUT"))
    );
    assert_eq!(server.slots.available_permits(), 8);
}

#[test]
fn every_account_tool_requires_a_delegated_permission() {
    let tools = AccountMcpServer::tool_router().list_all();
    assert_eq!(tools.len(), 18);
    for tool in tools {
        assert!(
            crate::http_transport::account_tool_scope(&tool.name).is_some(),
            "{} has no permission",
            tool.name
        );
    }
}
