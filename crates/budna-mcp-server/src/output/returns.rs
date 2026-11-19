use super::{account::ActionStateOutput, common::MoneyOutput};
use budna_mcp_client::{
    AccountActionState, RefundVia, ReturnKind, ReturnPostage, ReturnReceipt, ReturnRole,
    ReturnState, ReturnTerms,
};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TermsOutput {
    revision: String,
    order_id: i64,
    listing_id: String,
    role: ReturnRole,
    return_kind: Option<ReturnKind>,
    status: Option<ReturnState>,
    consumer_buyer: bool,
    business_seller: bool,
    order_amount: MoneyOutput,
    refundable_amount: Option<MoneyOutput>,
    refund_via: RefundVia,
    pre_dispatch: bool,
    return_shipping_payer: ReturnPostage,
    withdrawal_deadline: Option<i64>,
    online_refund_deadline: Option<i64>,
    has_own_tracking: bool,
    budna_label_booked: bool,
    buyer_protection_label: bool,
    can_request_withdrawal: bool,
    can_accept_agreed: bool,
    can_confirm_received: bool,
    can_set_tracking: bool,
    can_cancel: bool,
    budna_order_url: String,
}
impl From<(ReturnTerms, &str)> for TermsOutput {
    fn from((t, origin): (ReturnTerms, &str)) -> Self {
        let budna_order_url = format!("{origin}/orders/{}", t.order_id);
        Self {
            budna_order_url,
            revision: t.revision,
            order_id: t.order_id,
            listing_id: t.listing_id,
            role: t.role,
            return_kind: t.return_kind,
            status: t.status,
            consumer_buyer: t.consumer_buyer,
            business_seller: t.business_seller,
            order_amount: t.order_amount.into(),
            refundable_amount: t.refundable_amount.map(Into::into),
            refund_via: t.refund_via,
            pre_dispatch: t.pre_dispatch,
            return_shipping_payer: t.return_shipping_payer,
            withdrawal_deadline: t.withdrawal_deadline,
            online_refund_deadline: t.online_refund_deadline,
            has_own_tracking: t.has_own_tracking,
            budna_label_booked: t.budna_label_booked,
            buyer_protection_label: t.buyer_protection_label,
            can_request_withdrawal: t.can_request_withdrawal,
            can_accept_agreed: t.can_accept_agreed,
            can_confirm_received: t.can_confirm_received,
            can_set_tracking: t.can_set_tracking,
            can_cancel: t.can_cancel,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ReceiptOutput {
    order_id: i64,
    action_id: String,
    action: String,
    status: ActionStateOutput,
    confirmation_url: Option<String>,
}
impl From<(ReturnReceipt, &str)> for ReceiptOutput {
    fn from((r, origin): (ReturnReceipt, &str)) -> Self {
        let confirmation_url = (r.status == AccountActionState::AwaitingConfirmation)
            .then(|| format!("{origin}/approvals/{}", r.action_id));
        Self {
            action_id: r.action_id,
            action: r.action,
            status: r.status.into(),
            order_id: r.order_id,
            confirmation_url,
        }
    }
}
