use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};

use super::{
    AccountApiClient, AccountWatchlistActionStatus, ApiEnvelope, ClientError, DRAFT_CREATE_SCOPE,
    DelegatedAccess, PROFILE_READ_SCOPE, invalid_input, is_action_id, is_listing_id,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DraftVerification {
    Satisfied,
    VerifyIdentityOnBudna,
    VerifyCompanyOnBudna,
}

#[derive(Debug, Deserialize)]
pub struct AccountDraftEligibility {
    pub seller_verification: DraftVerification,
}

#[derive(Debug, Serialize)]
pub struct DraftInput<'a> {
    pub market: &'a str,
    pub currency: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
pub struct DraftReceipt {
    pub action_id: String,
    pub listing_id: Option<String>,
    pub status: AccountWatchlistActionStatus,
}

impl AccountApiClient {
    pub async fn draft_eligibility(
        &self,
        access: &DelegatedAccess,
    ) -> Result<AccountDraftEligibility, ClientError> {
        const OPERATION: &str = "get_my_listing_draft_eligibility";
        self.require_scope(access, PROFILE_READ_SCOPE, OPERATION)?;
        if !self.listing_draft_creation_enabled() {
            return Err(invalid_input(
                OPERATION,
                "Draft creation is unavailable on this connection",
            ));
        }
        self.get(
            OPERATION,
            "mcp/account/listing-draft-eligibility",
            access,
            &[],
        )
        .await
    }

    pub async fn create_draft(
        &self,
        access: &DelegatedAccess,
        input: &DraftInput<'_>,
        key: &str,
    ) -> Result<DraftReceipt, ClientError> {
        const OPERATION: &str = "create_my_listing_draft";
        self.require_scope(access, DRAFT_CREATE_SCOPE, OPERATION)?;
        if !is_action_id(key)
            || !matches!(input.market, "swedish" | "norwegian")
            || input.currency != "SEK"
            || input
                .title
                .is_some_and(|text| !super::actions::valid_action_text(text, 200))
            || input
                .description
                .is_some_and(|text| !super::actions::valid_action_text(text, 10_000))
        {
            return Err(invalid_input(
                OPERATION,
                "Invalid draft market, currency, text or request key",
            ));
        }
        let request = self
            .http
            .post(self.endpoint(OPERATION, "mcp/account/listing-drafts")?)
            .header(AUTHORIZATION, access.credential.header(OPERATION)?)
            .header("Idempotency-Key", key)
            .json(input);
        let envelope: ApiEnvelope<DraftReceipt> = self
            .send(OPERATION, request, 4096)
            .await
            .map_err(ClientError::account_write)?;
        envelope
            .data
            .filter(|receipt| {
                envelope.success
                    && is_action_id(&receipt.action_id)
                    && receipt.listing_id.as_deref().is_none_or(is_listing_id)
                    && (receipt.status != AccountWatchlistActionStatus::Executed
                        || receipt.listing_id.is_some())
            })
            .ok_or(ClientError::AccountWrite {
                operation: OPERATION,
                rejection_status: None,
            })
    }
}
