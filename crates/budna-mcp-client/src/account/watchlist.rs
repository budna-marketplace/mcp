use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

use super::{
    AccountApiClient, ApiEnvelope, ClientError, DelegatedAccess, WATCHLIST_WRITE_SCOPE,
    invalid_input, is_action_id, is_listing_id,
};

/// State of the original request. Replaying a receipt does not check current membership.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccountWatchlistActionStatus {
    Executed,
    Executing,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct AccountWatchlistMembershipReceipt {
    pub action_id: String,
    pub watchlist_id: i64,
    pub listing_id: String,
    /// The desired state supplied in the original request.
    pub saved: bool,
    pub status: AccountWatchlistActionStatus,
}

impl AccountApiClient {
    /// Sets one membership on an owned private list. The authoritative write checks
    /// list visibility and ownership at mutation time. Use one key per logical call.
    pub async fn set_my_watchlist_membership(
        &self,
        access: &DelegatedAccess,
        watchlist_id: i64,
        listing_id: &str,
        saved: bool,
        idempotency_key: &str,
    ) -> Result<AccountWatchlistMembershipReceipt, ClientError> {
        const OPERATION: &str = "set_my_watchlist_membership";
        self.require_scope(access, WATCHLIST_WRITE_SCOPE, OPERATION)?;
        if watchlist_id < 1 || !is_listing_id(listing_id) || !is_action_id(idempotency_key) {
            return Err(invalid_input(
                OPERATION,
                "Invalid watchlist, listing or operation key",
            ));
        }
        let path = format!("mcp/account/watchlists/{watchlist_id}/listings/{listing_id}");
        let request = self
            .http
            .put(self.endpoint(OPERATION, &path)?)
            .header(AUTHORIZATION, access.credential.header(OPERATION)?)
            .header("Idempotency-Key", idempotency_key)
            .json(&serde_json::json!({"saved": saved}));
        let envelope: ApiEnvelope<AccountWatchlistMembershipReceipt> = self
            .send(OPERATION, request, 4096)
            .await
            .map_err(|error| ClientError::AccountWrite {
                operation: OPERATION,
                rejection_status: error
                    .status()
                    .filter(|status| matches!(status, 400 | 401 | 403 | 404 | 409 | 422 | 429)),
            })?;
        envelope
            .data
            .filter(|receipt| {
                envelope.success
                    && is_action_id(&receipt.action_id)
                    && receipt.watchlist_id == watchlist_id
                    && receipt.listing_id == listing_id
                    && receipt.saved == saved
            })
            .ok_or(ClientError::AccountWrite {
                operation: OPERATION,
                rejection_status: None,
            })
    }
}
