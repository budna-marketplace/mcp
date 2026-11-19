use super::{account::ActionStateOutput, common::MoneyOutput};
use budna_mcp_client::{
    AccountActionState, ClosedOrderState, ClosingOrigin, ClosingPayment, ClosingRefund,
    ClosingRole, OrderClosureMove, OrderClosureQuote, OrderClosureReceipt, OwnOrderClosure,
};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ClosureStateOutput {
    order_id: i64,
    listing_id: String,
    role: ClosingRole,
    order_status: ClosedOrderState,
    payment_status: ClosingPayment,
    payment_deadline: Option<i64>,
    closed_unpaid: bool,
    relisted_at: Option<i64>,
}
impl From<OwnOrderClosure> for ClosureStateOutput {
    fn from(r: OwnOrderClosure) -> Self {
        Self {
            order_id: r.order_id,
            listing_id: r.listing_id,
            role: r.role,
            order_status: r.order_status,
            payment_status: r.payment_status,
            payment_deadline: r.payment_deadline,
            closed_unpaid: r.closed_unpaid,
            relisted_at: r.relisted_at,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ClosureQuoteOutput {
    terms_digest: String,
    order_id: i64,
    listing_id: String,
    role: ClosingRole,
    order_status: ClosedOrderState,
    payment_status: ClosingPayment,
    payment_deadline: Option<i64>,
    quote_id: String,
    operation: OrderClosureMove,
    order_origin: ClosingOrigin,
    legacy_terms: bool,
    payment_due_at: Option<i64>,
    debt_payer: Option<ClosingRole>,
    refund_route: ClosingRefund,
    restores_stock: bool,
    ends_order: bool,
    starts_listing: bool,
    relist_duration_ms: Option<i64>,
    expires_at: i64,
    debt_amount: Option<MoneyOutput>,
    refund_amount: Option<MoneyOutput>,
    retained_partial: Option<MoneyOutput>,
    relist_starting_price: Option<MoneyOutput>,
    relist_buy_now_price: Option<MoneyOutput>,
}
impl From<OrderClosureQuote> for ClosureQuoteOutput {
    fn from(r: OrderClosureQuote) -> Self {
        let q = r.terms;
        Self {
            terms_digest: r.terms_digest,
            order_id: q.order_id,
            listing_id: q.listing_id,
            role: q.role,
            order_status: q.order_status,
            payment_status: q.payment_status,
            payment_deadline: q.payment_deadline,
            quote_id: q.quote_id,
            operation: q.operation,
            order_origin: q.order_origin,
            legacy_terms: q.legacy_terms,
            payment_due_at: q.payment_due_at,
            debt_payer: q.debt_payer,
            refund_route: q.refund_route,
            restores_stock: q.restores_stock,
            ends_order: q.ends_order,
            starts_listing: q.starts_listing,
            relist_duration_ms: q.relist_duration_ms,
            expires_at: q.expires_at,
            debt_amount: q.debt_amount.map(Into::into),
            refund_amount: q.refund_amount.map(Into::into),
            retained_partial: q.retained_partial.map(Into::into),
            relist_starting_price: q.relist_starting_price.map(Into::into),
            relist_buy_now_price: q.relist_buy_now_price.map(Into::into),
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ClosureReceiptOutput {
    action_id: String,
    status: ActionStateOutput,
    order_id: i64,
    operation: OrderClosureMove,
    confirmation_url: Option<String>,
}
impl From<(OrderClosureReceipt, &str)> for ClosureReceiptOutput {
    fn from((r, origin): (OrderClosureReceipt, &str)) -> Self {
        let url = (r.status == AccountActionState::AwaitingConfirmation)
            .then(|| format!("{origin}/approvals/{}", r.action_id));
        Self {
            action_id: r.action_id,
            status: r.status.into(),
            order_id: r.order_id,
            operation: r.operation,
            confirmation_url: url,
        }
    }
}
