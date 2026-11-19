use super::{account::ActionStateOutput, common::MoneyOutput};
use budna_mcp_client::{
    AccountActionState, CombinedParcelReceipt, CombinedParcelTerms, PackageSize, ParcelRole,
    ParcelStatus,
};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TermsOutput {
    revision: String,
    reviewed_at: Option<i64>,
    expires_at: Option<i64>,
    group_id: Option<i64>,
    role: ParcelRole,
    seller_id: String,
    status: ParcelStatus,
    members: Vec<MemberOutput>,
    payment_deadline: i64,
    seller_answer_deadline: i64,
    package_size: Option<PackageSize>,
    package_weight_grams: Option<i32>,
    custom_shipping_price: Option<MoneyOutput>,
    shipping_cost: Option<MoneyOutput>,
    matched_seller_rule: bool,
    can_quote: bool,
    can_decline: bool,
    can_dissolve: bool,
    payment_available_on_budna: bool,
    budna_order_url: Option<String>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct MemberOutput {
    order_id: i64,
    listing_id: String,
    quantity: i32,
    item_price: MoneyOutput,
    buyer_protection_fee: Option<MoneyOutput>,
    high_value_fee: MoneyOutput,
    current_payment_deadline: Option<i64>,
    resulting_payment_deadline: Option<i64>,
}
impl From<(CombinedParcelTerms, &str)> for TermsOutput {
    fn from((t, origin): (CombinedParcelTerms, &str)) -> Self {
        let budna_order_url = t.group_id.and_then(|_| {
            t.members
                .first()
                .map(|m| format!("{origin}/orders/{}", m.order_id))
        });
        Self {
            revision: t.revision,
            reviewed_at: t.reviewed_at,
            expires_at: t.expires_at,
            group_id: t.group_id,
            role: t.role,
            seller_id: t.seller_id,
            status: t.status,
            members: t
                .members
                .into_iter()
                .map(|m| MemberOutput {
                    order_id: m.order_id,
                    listing_id: m.listing_id,
                    quantity: m.quantity,
                    item_price: m.item_price.into(),
                    buyer_protection_fee: m.buyer_protection_fee.map(Into::into),
                    high_value_fee: m.high_value_fee.into(),
                    current_payment_deadline: m.current_payment_deadline,
                    resulting_payment_deadline: m.resulting_payment_deadline,
                })
                .collect(),
            payment_deadline: t.payment_deadline,
            seller_answer_deadline: t.seller_answer_deadline,
            package_size: t.package_size,
            package_weight_grams: t.package_weight_grams,
            custom_shipping_price: t.custom_shipping_price.map(Into::into),
            shipping_cost: t.shipping_cost.map(Into::into),
            matched_seller_rule: t.matched_seller_rule,
            can_quote: t.can_quote,
            can_decline: t.can_decline,
            can_dissolve: t.can_dissolve,
            payment_available_on_budna: t.payment_available_on_budna,
            budna_order_url,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ReceiptOutput {
    action_id: String,
    action: String,
    status: ActionStateOutput,
    group_id: Option<i64>,
    confirmation_url: Option<String>,
}
impl From<(CombinedParcelReceipt, &str)> for ReceiptOutput {
    fn from((r, origin): (CombinedParcelReceipt, &str)) -> Self {
        let confirmation_url = (r.status == AccountActionState::AwaitingConfirmation
            && r.action == "request_combined_parcel")
            .then(|| format!("{origin}/approvals/{}", r.action_id));
        Self {
            action_id: r.action_id,
            action: r.action,
            status: r.status.into(),
            group_id: r.group_id,
            confirmation_url,
        }
    }
}
