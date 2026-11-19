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

impl From<budna_mcp_client::AccountActionState> for ActionStateOutput {
    fn from(state: budna_mcp_client::AccountActionState) -> Self {
        use budna_mcp_client::AccountActionState as S;
        match state {
            S::Refused => ActionStateOutput::Refused,
            S::AwaitingConfirmation => ActionStateOutput::AwaitingConfirmation,
            S::Declined => ActionStateOutput::Declined,
            S::Expired => ActionStateOutput::Expired,
            S::Executing => ActionStateOutput::Executing,
            S::Executed => ActionStateOutput::Executed,
            S::Failed => ActionStateOutput::Failed,
            S::Unknown => ActionStateOutput::Unknown,
        }
    }
}

impl From<budna_mcp_client::AccountActionStatus> for ActionStatusOutput {
    fn from(action: budna_mcp_client::AccountActionStatus) -> Self {
        Self {
            action_id: action.action_id,
            status: action.status.into(),
            expires_at: action.expires_at,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DraftVerificationOutput {
    Satisfied,
    VerifyIdentityOnBudna,
    VerifyCompanyOnBudna,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DraftEligibilityOutput {
    pub seller_verification: DraftVerificationOutput,
    pub verification_url: Option<String>,
}

impl DraftEligibilityOutput {
    pub fn new(
        eligibility: budna_mcp_client::AccountDraftEligibility,
        verification_url: Option<String>,
    ) -> Self {
        use budna_mcp_client::DraftVerification;
        Self {
            seller_verification: match eligibility.seller_verification {
                DraftVerification::Satisfied => DraftVerificationOutput::Satisfied,
                DraftVerification::VerifyIdentityOnBudna => {
                    DraftVerificationOutput::VerifyIdentityOnBudna
                }
                DraftVerification::VerifyCompanyOnBudna => {
                    DraftVerificationOutput::VerifyCompanyOnBudna
                }
            },
            verification_url,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DraftReceiptOutput {
    pub action_id: String,
    pub listing_id: Option<String>,
    pub status: WatchlistActionStatusOutput,
}

impl From<budna_mcp_client::DraftReceipt> for DraftReceiptOutput {
    fn from(receipt: budna_mcp_client::DraftReceipt) -> Self {
        Self {
            action_id: receipt.action_id,
            listing_id: receipt.listing_id,
            status: receipt.status.into(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LiveReminderReceiptOutput {
    pub action_id: String,
    pub session_id: String,
    pub reminded: bool,
    pub status: WatchlistActionStatusOutput,
}

impl From<budna_mcp_client::LiveReminderReceipt> for LiveReminderReceiptOutput {
    fn from(receipt: budna_mcp_client::LiveReminderReceipt) -> Self {
        Self {
            action_id: receipt.action_id,
            session_id: receipt.session_id,
            reminded: receipt.reminded,
            status: receipt.status.into(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct NotificationReadBatchOutput {
    #[schemars(length(max = 50))]
    pub notification_ids: Vec<i64>,
    pub has_more: bool,
}
impl From<budna_mcp_client::NotificationReadBatch> for NotificationReadBatchOutput {
    fn from(batch: budna_mcp_client::NotificationReadBatch) -> Self {
        Self {
            notification_ids: batch.notification_ids,
            has_more: batch.has_more,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct NotificationReadReceiptOutput {
    pub action_id: String,
    pub status: WatchlistActionStatusOutput,
    pub requested_count: u32,
}
impl From<budna_mcp_client::NotificationReadReceipt> for NotificationReadReceiptOutput {
    fn from(receipt: budna_mcp_client::NotificationReadReceipt) -> Self {
        Self {
            action_id: receipt.action_id,
            status: receipt.status.into(),
            requested_count: receipt.requested_count,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SavedSearchAuthoringOutput {
    pub used: i64,
    pub limit: i64,
    #[schemars(length(min = 3, max = 3))]
    pub notification_frequencies: Vec<String>,
}

impl From<budna_mcp_client::SavedSearchAuthoring> for SavedSearchAuthoringOutput {
    fn from(value: budna_mcp_client::SavedSearchAuthoring) -> Self {
        Self {
            used: value.used,
            limit: value.limit,
            notification_frequencies: value.notification_frequencies,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SavedSearchReceiptOutput {
    pub action_id: String,
    pub search_id: Option<i64>,
    pub status: WatchlistActionStatusOutput,
}
impl From<budna_mcp_client::SavedSearchReceipt> for SavedSearchReceiptOutput {
    fn from(value: budna_mcp_client::SavedSearchReceipt) -> Self {
        Self {
            action_id: value.action_id,
            search_id: value.search_id,
            status: value.status.into(),
        }
    }
}
