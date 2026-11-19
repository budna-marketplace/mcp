use budna_mcp_client::{AccountListingPage, AccountSummary};
use rmcp::schemars;
use serde::Serialize;

use super::{ProjectionBudget, UNTRUSTED_CONTENT_NOTICE, common::PaginationOutput};

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountOutput {
    #[schemars(length(max = 64))]
    pub id: String,
    #[schemars(length(max = 16))]
    pub account_type: String,
    #[schemars(length(max = 512))]
    pub display_name: Option<String>,
    pub untrusted_content_notice: &'static str,
}

impl From<AccountSummary> for AccountOutput {
    fn from(account: AccountSummary) -> Self {
        Self {
            id: account.id,
            account_type: account.account_type,
            display_name: ProjectionBudget::default().optional_text(account.display_name, 512),
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountListingOutput {
    #[schemars(length(max = 64))]
    pub id: String,
    #[schemars(length(max = 512))]
    pub title: Option<String>,
    #[schemars(length(max = 16))]
    pub status: String,
    #[schemars(length(max = 32))]
    pub listing_type: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountListingsOutput {
    #[schemars(length(max = 50))]
    pub items: Vec<AccountListingOutput>,
    pub pagination: PaginationOutput,
    pub untrusted_content_notice: &'static str,
}

impl From<AccountListingPage> for AccountListingsOutput {
    fn from(page: AccountListingPage) -> Self {
        let mut budget = ProjectionBudget::default();
        Self {
            items: page
                .items
                .into_iter()
                .map(|item| AccountListingOutput {
                    id: item.id,
                    title: budget.optional_text(item.title, 512),
                    status: item.status,
                    listing_type: item.listing_type,
                    updated_at: item.updated_at,
                })
                .collect(),
            pagination: page.pagination.into(),
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}

/// A receipt for the requested state, including actions whose outcome is still unknown.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct WatchlistMembershipOutput {
    pub action_id: String,
    pub watchlist_id: i64,
    pub listing_id: String,
    /// The state requested in the original action. Only an executed receipt confirms completion.
    pub saved: bool,
    pub status: WatchlistActionStatusOutput,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WatchlistActionStatusOutput {
    Executed,
    Executing,
    Unknown,
}

impl From<budna_mcp_client::AccountWatchlistMembershipReceipt> for WatchlistMembershipOutput {
    fn from(receipt: budna_mcp_client::AccountWatchlistMembershipReceipt) -> Self {
        Self {
            action_id: receipt.action_id,
            watchlist_id: receipt.watchlist_id,
            listing_id: receipt.listing_id,
            saved: receipt.saved,
            status: receipt.status.into(),
        }
    }
}

impl From<budna_mcp_client::AccountWatchlistActionStatus> for WatchlistActionStatusOutput {
    fn from(status: budna_mcp_client::AccountWatchlistActionStatus) -> Self {
        use budna_mcp_client::AccountWatchlistActionStatus;
        match status {
            AccountWatchlistActionStatus::Executed => WatchlistActionStatusOutput::Executed,
            AccountWatchlistActionStatus::Executing => WatchlistActionStatusOutput::Executing,
            AccountWatchlistActionStatus::Unknown => WatchlistActionStatusOutput::Unknown,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PrivateWatchlistOutput {
    pub action_id: String,
    pub watchlist_id: Option<i64>,
    pub status: WatchlistActionStatusOutput,
}

impl From<budna_mcp_client::AccountPrivateWatchlistReceipt> for PrivateWatchlistOutput {
    fn from(receipt: budna_mcp_client::AccountPrivateWatchlistReceipt) -> Self {
        Self {
            action_id: receipt.action_id,
            watchlist_id: receipt.watchlist_id,
            status: receipt.status.into(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionStateOutput {
    Refused,
    AwaitingConfirmation,
    Declined,
    Expired,
    Executing,
    Executed,
    Failed,
    Unknown,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ActionStatusOutput {
    pub action_id: String,
    pub status: ActionStateOutput,
    /// Unix epoch milliseconds, when applicable.
    pub expires_at: Option<i64>,
}

impl From<budna_mcp_client::AccountActionStatus> for ActionStatusOutput {
    fn from(action: budna_mcp_client::AccountActionStatus) -> Self {
        use budna_mcp_client::AccountActionState as S;
        let status = match action.status {
            S::Refused => ActionStateOutput::Refused,
            S::AwaitingConfirmation => ActionStateOutput::AwaitingConfirmation,
            S::Declined => ActionStateOutput::Declined,
            S::Expired => ActionStateOutput::Expired,
            S::Executing => ActionStateOutput::Executing,
            S::Executed => ActionStateOutput::Executed,
            S::Failed => ActionStateOutput::Failed,
            S::Unknown => ActionStateOutput::Unknown,
        };
        Self {
            action_id: action.action_id,
            status,
            expires_at: action.expires_at,
        }
    }
}
