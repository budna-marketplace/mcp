use super::{account::ActionStateOutput, common::MoneyOutput};
use budna_mcp_client::{
    AccountActionState, BindingCheckoutQuote, BindingCheckoutReceipt, CheckoutDelivery,
    PersonalCheckoutPhase, PersonalCheckoutState,
};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CheckoutStateOutput {
    listing_id: String,
    session_id: Option<String>,
    state: PersonalCheckoutPhase,
    order_id: Option<i64>,
    payment_deadline: Option<i64>,
}
impl From<PersonalCheckoutState> for CheckoutStateOutput {
    fn from(r: PersonalCheckoutState) -> Self {
        Self {
            listing_id: r.listing_id,
            session_id: r.session_id,
            state: r.state,
            order_id: r.order_id,
            payment_deadline: r.payment_deadline,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CheckoutQuoteOutput {
    terms_digest: String,
    quote_id: String,
    session_id: String,
    listing_id: String,
    quantity: i32,
    fulfillment_mode: CheckoutDelivery,
    buyer_protection_applied: bool,
    quoted_at: i64,
    payment_deadline: i64,
    ends_auction: bool,
    expires_at: i64,
    item_price: MoneyOutput,
    shipping_cost: MoneyOutput,
    buyer_protection_fee: MoneyOutput,
    high_value_fee: MoneyOutput,
    total: MoneyOutput,
    unpaid_fee: MoneyOutput,
}
impl From<BindingCheckoutQuote> for CheckoutQuoteOutput {
    fn from(r: BindingCheckoutQuote) -> Self {
        let q = r.terms;
        Self {
            terms_digest: r.terms_digest,
            quote_id: q.quote_id,
            session_id: q.session_id,
            listing_id: q.listing_id,
            quantity: q.quantity,
            fulfillment_mode: q.fulfillment_mode,
            buyer_protection_applied: q.buyer_protection_applied,
            quoted_at: q.quoted_at,
            payment_deadline: q.payment_deadline,
            ends_auction: q.ends_auction,
            expires_at: q.expires_at,
            item_price: q.item_price.into(),
            shipping_cost: q.shipping_cost.into(),
            buyer_protection_fee: q.buyer_protection_fee.into(),
            high_value_fee: q.high_value_fee.into(),
            total: q.total.into(),
            unpaid_fee: q.unpaid_fee.into(),
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CheckoutReceiptOutput {
    action_id: String,
    status: ActionStateOutput,
    listing_id: String,
    session_id: String,
    order_id: Option<i64>,
    payment_deadline: Option<i64>,
    confirmation_url: Option<String>,
}
impl From<(BindingCheckoutReceipt, &str)> for CheckoutReceiptOutput {
    fn from((r, origin): (BindingCheckoutReceipt, &str)) -> Self {
        let confirmation_url = (r.status == AccountActionState::AwaitingConfirmation)
            .then(|| format!("{origin}/approvals/{}", r.action_id));
        Self {
            action_id: r.action_id,
            status: r.status.into(),
            listing_id: r.listing_id,
            session_id: r.session_id,
            order_id: r.order_id,
            payment_deadline: r.payment_deadline,
            confirmation_url: confirmation_url,
        }
    }
}
