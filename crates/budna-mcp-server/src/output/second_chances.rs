use super::{account::ActionStateOutput, common::MoneyOutput};
use budna_mcp_client::{
    AccountActionState, ChanceOperation, ChanceOrderStatus, SecondChanceInvitation,
    SecondChanceQuote, SecondChanceReceipt, SecondChanceSaleState,
};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SellerStateOutput {
    source_order_id: i64,
    listing_id: String,
    maximum_price: Option<MoneyOutput>,
    active_offer_id: Option<i64>,
    can_start: bool,
    can_cancel: bool,
}
impl From<SecondChanceSaleState> for SellerStateOutput {
    fn from(s: SecondChanceSaleState) -> Self {
        Self {
            source_order_id: s.source_order_id,
            listing_id: s.listing_id,
            maximum_price: s.maximum_price.map(Into::into),
            active_offer_id: s.active_offer_id,
            can_start: s.can_start,
            can_cancel: s.can_cancel,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct InvitationOutput {
    offer_id: i64,
    invitation_id: i64,
    listing_id: String,
    price: MoneyOutput,
    window_ends_at: i64,
}
impl From<SecondChanceInvitation> for InvitationOutput {
    fn from(i: SecondChanceInvitation) -> Self {
        Self {
            offer_id: i.offer_id,
            invitation_id: i.invitation_id,
            listing_id: i.listing_id,
            price: i.price.into(),
            window_ends_at: i.window_ends_at,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct QuoteOutput {
    quote_id: String,
    terms_digest: String,
    operation: ChanceOperation,
    listing_id: String,
    source_order_id: Option<i64>,
    offer_id: Option<i64>,
    invitation_id: Option<i64>,
    revision: String,
    price: MoneyOutput,
    maximum_seller_price: Option<MoneyOutput>,
    current_window_ends_at: Option<i64>,
    response_window_hours: u32,
    payment_due_hours: u32,
    payment_grace_hours: u32,
    buyer_protection_fee: MoneyOutput,
    buyer_protection_mandatory: bool,
    high_value_fee: MoneyOutput,
    unpaid_fee: MoneyOutput,
    delivery_at_checkout: bool,
    expires_at: i64,
}
impl From<SecondChanceQuote> for QuoteOutput {
    fn from(v: SecondChanceQuote) -> Self {
        let q = v.terms;
        Self {
            quote_id: q.quote_id,
            terms_digest: v.terms_digest,
            operation: q.change.operation,
            listing_id: q.listing_id,
            source_order_id: q.change.source_order_id,
            offer_id: q.change.offer_id,
            invitation_id: q.change.invitation_id,
            revision: q.revision,
            price: q.price.into(),
            maximum_seller_price: q.maximum_seller_price.map(Into::into),
            current_window_ends_at: q.current_window_ends_at,
            response_window_hours: q.response_window_hours,
            payment_due_hours: q.payment_due_hours,
            payment_grace_hours: q.payment_grace_hours,
            buyer_protection_fee: q.buyer_protection_fee.into(),
            buyer_protection_mandatory: q.buyer_protection_mandatory,
            high_value_fee: q.high_value_fee.into(),
            unpaid_fee: q.unpaid_fee.into(),
            delivery_at_checkout: q.delivery_at_checkout,
            expires_at: q.expires_at,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ChanceReceiptOutput {
    action_id: String,
    status: ActionStateOutput,
    operation: ChanceOperation,
    source_order_id: Option<i64>,
    offer_id: Option<i64>,
    invitation_id: Option<i64>,
    order_id: Option<i64>,
    order_status: Option<ChanceOrderStatus>,
    confirmation_url: Option<String>,
}
impl From<(SecondChanceReceipt, &str)> for ChanceReceiptOutput {
    fn from((r, origin): (SecondChanceReceipt, &str)) -> Self {
        let confirmation_url = (r.status == AccountActionState::AwaitingConfirmation)
            .then(|| format!("{origin}/approvals/{}", r.action_id));
        Self {
            action_id: r.action_id,
            status: r.status.into(),
            operation: r.operation,
            source_order_id: r.source_order_id,
            offer_id: r.offer_id,
            invitation_id: r.invitation_id,
            order_id: r.order_id,
            order_status: r.order_status,
            confirmation_url,
        }
    }
}
