use serde::Deserialize;

use crate::{Pagination, is_listing_id};

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub account_type: String,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountListingSummary {
    pub id: String,
    pub title: Option<String>,
    pub status: String,
    pub listing_type: String,
    pub updated_at: i64,
}

impl AccountListingSummary {
    pub(super) fn is_valid(&self) -> bool {
        is_listing_id(&self.id)
            && self.title.as_ref().is_none_or(|title| title.len() <= 4096)
            && self.updated_at >= 0
            && matches!(
                self.status.as_str(),
                "draft" | "active" | "sold" | "expired" | "cancelled"
            )
            && matches!(
                self.listing_type.as_str(),
                "auction" | "fixed_price" | "auction_fixed_price" | "in_stock"
            )
    }
}

/// One bounded page of the connected account's own records.
#[derive(Clone, Debug, Deserialize)]
pub struct AccountPage<T> {
    pub items: Vec<T>,
    pub pagination: Pagination,
}

pub type AccountListingPage = AccountPage<AccountListingSummary>;

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
