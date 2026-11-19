use super::{account::ActionStateOutput, common::MoneyOutput};
use budna_mcp_client::{
    AccountActionState, PauseMaximumReceipt, SetMaximumReceipt, StandingBidQuote, StandingBidState,
    StandingBidStatus,
};
use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct StandingBidStateOutput {
    listing_id: String,
    revision: String,
    status: Option<StandingBidStatus>,
    maximum: Option<MoneyOutput>,
}
impl From<StandingBidState> for StandingBidStateOutput {
    fn from(v: StandingBidState) -> Self {
        Self {
            listing_id: v.listing_id,
            revision: v.revision,
            status: v.status,
            maximum: v.maximum.map(Into::into),
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct StandingBidQuoteOutput {
    quote_id: String,
    listing_id: String,
    live_session_id: Option<String>,
    round_ref: String,
    maximum: MoneyOutput,
    current_bid: Option<MoneyOutput>,
    minimum_bid: MoneyOutput,
    increment: MoneyOutput,
    buyer_protection_fee_at_maximum: MoneyOutput,
    buyer_protection_mandatory: bool,
    unpaid_fee_at_maximum: MoneyOutput,
    payment_due_hours: u32,
    payment_grace_hours: u32,
    delivery_at_checkout: bool,
    ends_at: i64,
    expires_at: i64,
    existing: StandingBidStateOutput,
    can_place_bid_immediately: bool,
}
impl From<StandingBidQuote> for StandingBidQuoteOutput {
    fn from(v: StandingBidQuote) -> Self {
        let q = v.auction;
        Self {
            quote_id: q.quote_id,
            listing_id: q.listing_id,
            live_session_id: q.live_session_id,
            round_ref: q.round_ref,
            maximum: q.bid.into(),
            current_bid: q.current_bid.map(Into::into),
            minimum_bid: q.minimum_bid.into(),
            increment: q.increment.into(),
            buyer_protection_fee_at_maximum: q.buyer_protection_fee.into(),
            buyer_protection_mandatory: q.buyer_protection_mandatory,
            unpaid_fee_at_maximum: q.unpaid_fee.into(),
            payment_due_hours: q.payment_due_hours,
            payment_grace_hours: q.payment_grace_hours,
            delivery_at_checkout: q.delivery_at_checkout,
            ends_at: q.ends_at,
            expires_at: q.expires_at,
            existing: v.existing.into(),
            can_place_bid_immediately: v.can_place_bid_immediately,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SetMaximumOutput {
    action_id: String,
    status: ActionStateOutput,
    listing_id: String,
    maximum: MoneyOutput,
    confirmation_url: Option<String>,
}
impl From<(SetMaximumReceipt, &str)> for SetMaximumOutput {
    fn from((v, origin): (SetMaximumReceipt, &str)) -> Self {
        let confirmation_url = (v.status == AccountActionState::AwaitingConfirmation)
            .then(|| format!("{origin}/approvals/{}", v.action_id));
        Self {
            action_id: v.action_id,
            status: v.status.into(),
            listing_id: v.listing_id,
            maximum: v.maximum.into(),
            confirmation_url,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PauseMaximumOutput {
    action_id: String,
    status: ActionStateOutput,
    listing_id: String,
}
impl From<PauseMaximumReceipt> for PauseMaximumOutput {
    fn from(v: PauseMaximumReceipt) -> Self {
        Self {
            action_id: v.action_id,
            status: v.status.into(),
            listing_id: v.listing_id,
        }
    }
}
