use serde::Deserialize;

use crate::Pagination;

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub account_type: String,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountListingSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub listing_type: String,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountListingPage {
    pub items: Vec<AccountListingSummary>,
    pub pagination: Pagination,
}

use super::Credential;

#[derive(Deserialize)]
pub(super) struct ExchangeResponse {
    pub(super) access_token: Credential,
    pub(super) token_type: String,
    pub(super) expires_in: u64,
    pub(super) scope: String,
    pub(super) subject: String,
    pub(super) grant_id: String,
    pub(super) client_id: String,
    pub(super) resource: String,
}
