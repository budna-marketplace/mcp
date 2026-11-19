use super::{account::ActionStateOutput, common::MoneyOutput};
use budna_mcp_client::{
    AccountActionState, NegotiationMove, NegotiationParty, NegotiationPreview, NegotiationReceipt,
};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ReviewedOfferOutput {
    quote_id: String,
    listing_id: String,
    operation: NegotiationMove,
    offer_id: Option<i64>,
    role: NegotiationParty,
    terms_digest: String,
    revision: String,
    quantity: u16,
    total_for_all_units: MoneyOutput,
    standing_buyer_total: Option<MoneyOutput>,
    standing_seller_total: Option<MoneyOutput>,
    remaining_buyer_total: Option<MoneyOutput>,
    remaining_seller_total: Option<MoneyOutput>,
    next_responder: Option<NegotiationParty>,
    current_offer_expires_at: Option<i64>,
    response_window_hours: Option<u32>,
    buyer_protection_fee: MoneyOutput,
    buyer_protection_mandatory: bool,
    high_value_fee: MoneyOutput,
    unpaid_fee: MoneyOutput,
    payment_due_hours: u32,
    payment_grace_hours: u32,
    delivery_at_checkout: bool,
    reserves_stock: bool,
    expires_at: i64,
}
impl From<NegotiationPreview> for ReviewedOfferOutput {
    fn from(v: NegotiationPreview) -> Self {
        let q = v.terms;
        Self {
            quote_id: q.quote_id,
            listing_id: q.listing_id,
            operation: q.proposal.operation,
            offer_id: q.proposal.offer_id,
            role: q.role,
            terms_digest: v.terms_digest,
            revision: q.revision,
            quantity: q.quantity,
            total_for_all_units: q.total.into(),
            standing_buyer_total: q.standing_buyer_total.map(Into::into),
            standing_seller_total: q.standing_seller_total.map(Into::into),
            remaining_buyer_total: q.remaining_buyer_total.map(Into::into),
            remaining_seller_total: q.remaining_seller_total.map(Into::into),
            next_responder: q.next_responder,
            current_offer_expires_at: q.current_offer_expires_at,
            response_window_hours: q.response_window_hours,
            buyer_protection_fee: q.buyer_protection_fee.into(),
            buyer_protection_mandatory: q.buyer_protection_mandatory,
            high_value_fee: q.high_value_fee.into(),
            unpaid_fee: q.unpaid_fee.into(),
            payment_due_hours: q.payment_due_hours,
            payment_grace_hours: q.payment_grace_hours,
            delivery_at_checkout: q.delivery_at_checkout,
            reserves_stock: q.reserves_stock,
            expires_at: q.expires_at,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct OfferReceiptOutput {
    action_id: String,
    status: ActionStateOutput,
    operation: NegotiationMove,
    listing_id: String,
    offer_id: Option<i64>,
    order_id: Option<i64>,
    confirmation_url: Option<String>,
}
impl From<(NegotiationReceipt, &str)> for OfferReceiptOutput {
    fn from((r, origin): (NegotiationReceipt, &str)) -> Self {
        let confirmation_url = (r.status == AccountActionState::AwaitingConfirmation)
            .then(|| format!("{origin}/approvals/{}", r.action_id));
        Self {
            action_id: r.action_id,
            status: r.status.into(),
            operation: r.operation,
            listing_id: r.listing_id,
            offer_id: r.offer_id,
            order_id: r.order_id,
            confirmation_url,
        }
    }
}
