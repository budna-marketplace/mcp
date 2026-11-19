use super::{account::ActionStateOutput, common::MoneyOutput};
use budna_mcp_client::{AccountActionState, BidQuote, BidSubmissionReceipt};
use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct BidQuoteOutput {
    quote_id: String,
    listing_id: String,
    live_session_id: Option<String>,
    round_ref: String,
    bid: MoneyOutput,
    current_bid: Option<MoneyOutput>,
    minimum_bid: MoneyOutput,
    increment: MoneyOutput,
    buyer_protection_fee: MoneyOutput,
    buyer_protection_mandatory: bool,
    unpaid_fee: MoneyOutput,
    payment_due_hours: u32,
    payment_grace_hours: u32,
    delivery_at_checkout: bool,
    ends_at: i64,
    expires_at: i64,
}

impl From<BidQuote> for BidQuoteOutput {
    fn from(q: BidQuote) -> Self {
        Self {
            quote_id: q.quote_id,
            listing_id: q.listing_id,
            live_session_id: q.live_session_id,
            round_ref: q.round_ref,
            bid: q.bid.into(),
            current_bid: q.current_bid.map(Into::into),
            minimum_bid: q.minimum_bid.into(),
            increment: q.increment.into(),
            buyer_protection_fee: q.buyer_protection_fee.into(),
            buyer_protection_mandatory: q.buyer_protection_mandatory,
            unpaid_fee: q.unpaid_fee.into(),
            payment_due_hours: q.payment_due_hours,
            payment_grace_hours: q.payment_grace_hours,
            delivery_at_checkout: q.delivery_at_checkout,
            ends_at: q.ends_at,
            expires_at: q.expires_at,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct BidSubmissionOutput {
    action_id: String,
    status: ActionStateOutput,
    listing_id: String,
    bid: MoneyOutput,
    confirmation_url: Option<String>,
}

impl From<(BidSubmissionReceipt, &str)> for BidSubmissionOutput {
    fn from((r, origin): (BidSubmissionReceipt, &str)) -> Self {
        let confirmation_url = (r.status == AccountActionState::AwaitingConfirmation)
            .then(|| format!("{origin}/approvals/{}", r.action_id));
        Self {
            action_id: r.action_id,
            status: r.status.into(),
            listing_id: r.listing_id,
            bid: r.bid.into(),
            confirmation_url,
        }
    }
}
