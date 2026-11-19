use super::{
    AccountActionState, AccountApiClient, ApiEnvelope, BIDS_READ_SCOPE, BidAmount, BidQuote,
    ClientError, DelegatedAccess, invalid_input, invalid_response, is_action_id,
};
use crate::{Money, decimal_is_less_than_or_equal, is_listing_id, money_is_valid};
use reqwest::{Method, header::AUTHORIZATION};
use serde::Deserialize;

pub use budna_mcp_core::write_permissions::AUTOBIDS_WRITE_SCOPE;

#[derive(
    Clone, Copy, Debug, Deserialize, serde::Serialize, schemars::JsonSchema, PartialEq, Eq,
)]
#[serde(rename_all = "snake_case")]
pub enum StandingBidStatus {
    Active,
    Paused,
    Expired,
    Completed,
    MaxReached,
}

#[derive(Debug, Deserialize)]
pub struct StandingBidState {
    pub listing_id: String,
    pub revision: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub status: Option<StandingBidStatus>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub maximum: Option<Money>,
}
fn revision(value: &str) -> bool {
    value.len() == 43
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}
impl StandingBidState {
    fn matches_listing(&self, listing: &str) -> bool {
        self.listing_id == listing
            && revision(&self.revision)
            && self.status.is_some() == self.maximum.is_some()
            && self.maximum.as_ref().is_none_or(|m| {
                money_is_valid(m, "SEK")
                    && !decimal_is_less_than_or_equal(&m.amount, "0")
                    && decimal_is_less_than_or_equal(&m.amount, "1000000")
                    && m.amount
                        .split_once('.')
                        .is_none_or(|(_, fraction)| fraction.len() <= 2)
            })
    }
}
#[derive(Debug, Deserialize)]
pub struct StandingBidQuote {
    pub auction: BidQuote,
    pub existing: StandingBidState,
    pub can_place_bid_immediately: bool,
}
#[derive(Debug, Deserialize)]
pub struct SetMaximumReceipt {
    pub action_id: String,
    pub status: AccountActionState,
    pub listing_id: String,
    pub maximum: Money,
}
#[derive(Debug, Deserialize)]
pub struct PauseMaximumReceipt {
    pub action_id: String,
    pub status: AccountActionState,
    pub listing_id: String,
}
impl AccountApiClient {
    pub async fn standing_bid(
        &self,
        access: &DelegatedAccess,
        listing: &str,
    ) -> Result<StandingBidState, ClientError> {
        const OP: &str = "get_my_automatic_bid_state";
        self.require_scope(access, BIDS_READ_SCOPE, OP)?;
        if !self.write_permissions().automatic_bids || !is_listing_id(listing) {
            return Err(invalid_input(
                OP,
                "Invalid listing or unavailable automatic bidding",
            ));
        }
        let request = self
            .http
            .get(self.endpoint(OP, &format!("mcp/account/listings/{listing}/automatic-bid"))?)
            .header(AUTHORIZATION, access.credential.header(OP)?);
        let result: ApiEnvelope<StandingBidState> = self.send(OP, request, 4096).await?;
        result
            .data
            .filter(|s| result.success && s.matches_listing(listing))
            .ok_or_else(|| invalid_response(OP))
    }
    pub async fn prepare_maximum(
        &self,
        access: &DelegatedAccess,
        listing: &str,
        maximum: &BidAmount,
    ) -> Result<StandingBidQuote, ClientError> {
        const OP: &str = "prepare_my_automatic_bid_maximum";
        self.require_scope(access, BIDS_READ_SCOPE, OP)?;
        if !self.write_permissions().automatic_bids || !is_listing_id(listing) || !maximum.valid() {
            return Err(invalid_input(OP, "Invalid listing or maximum"));
        }
        let request = self
            .http
            .post(self.endpoint(
                OP,
                &format!("mcp/account/listings/{listing}/automatic-bid-quote"),
            )?)
            .header(AUTHORIZATION, access.credential.header(OP)?)
            .json(&serde_json::json!({"maximum":maximum}));
        let result: ApiEnvelope<StandingBidQuote> = self.send(OP, request, 12288).await?;
        result
            .data
            .filter(|q| {
                result.success
                    && q.can_place_bid_immediately
                    && q.auction.matches_request(listing, maximum)
                    && q.existing.matches_listing(listing)
                    && (q.existing.status != Some(StandingBidStatus::Active)
                        || q.existing.maximum.as_ref().is_some_and(|prior| {
                            !decimal_is_less_than_or_equal(&maximum.amount, &prior.amount)
                        }))
            })
            .ok_or_else(|| invalid_response(OP))
    }
    pub async fn set_maximum(
        &self,
        access: &DelegatedAccess,
        listing: &str,
        quote: &str,
        maximum: &BidAmount,
    ) -> Result<SetMaximumReceipt, ClientError> {
        const OP: &str = "set_my_quoted_automatic_bid_maximum";
        if !is_listing_id(listing) || !is_action_id(quote) || !maximum.valid() {
            return Err(invalid_input(OP, "Invalid quoted maximum"));
        }
        let request = self
            .action_request(
                OP,
                AUTOBIDS_WRITE_SCOPE,
                access,
                Method::PUT,
                &format!("mcp/account/listings/{listing}/automatic-bid"),
                quote,
            )?
            .json(&serde_json::json!({"quote_id":quote,"maximum":maximum}));
        self.direct_receipt(OP, request, |r: &SetMaximumReceipt| {
            r.listing_id == listing && is_action_id(&r.action_id) && maximum.matches(&r.maximum)
        })
        .await
    }
    pub async fn pause_maximum(
        &self,
        access: &DelegatedAccess,
        listing: &str,
        expected: &str,
        key: &str,
    ) -> Result<PauseMaximumReceipt, ClientError> {
        const OP: &str = "pause_my_automatic_bid";
        if !is_listing_id(listing) || !revision(expected) || !is_action_id(key) {
            return Err(invalid_input(
                OP,
                "Invalid standing instruction or idempotency key",
            ));
        }
        let request = self
            .action_request(
                OP,
                AUTOBIDS_WRITE_SCOPE,
                access,
                Method::POST,
                &format!("mcp/account/listings/{listing}/automatic-bid/pause"),
                key,
            )?
            .json(&serde_json::json!({"expected_revision":expected}));
        self.direct_receipt(OP, request, |r: &PauseMaximumReceipt| {
            r.listing_id == listing && is_action_id(&r.action_id)
        })
        .await
    }
}
