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
    pub title: String,
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
                    title: budget.text(item.title, 512),
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
