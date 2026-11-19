use super::{account::ActionStateOutput, common::MoneyOutput};
use budna_mcp_client::{
    AccountActionState, FulfillmentOperation, FulfillmentReceipt, FulfillmentTerms, OutboundCarrier,
};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct Member {
    order_id: i64,
    listing_id: String,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub(crate) struct TermsOutput {
    order_id: i64,
    group_id: Option<i64>,
    operation: FulfillmentOperation,
    eligible: bool,
    revision: String,
    members: Vec<Member>,
    has_tracking: bool,
    carrier: Option<OutboundCarrier>,
    funded_balance: Option<MoneyOutput>,
    release_delay_hours: Option<i64>,
    handover_window_days: Option<i64>,
    closable_dispute_ids: Vec<i64>,
    budna_order_url: String,
}
impl From<(FulfillmentTerms, &str)> for TermsOutput {
    fn from((v, origin): (FulfillmentTerms, &str)) -> Self {
        Self {
            budna_order_url: format!("{origin}/orders/{}", v.order_id),
            order_id: v.order_id,
            group_id: v.group_id,
            operation: v.operation,
            eligible: v.eligible,
            revision: v.revision,
            members: v
                .members
                .into_iter()
                .map(|m| Member {
                    order_id: m.order_id,
                    listing_id: m.listing_id,
                })
                .collect(),
            has_tracking: v.has_tracking,
            carrier: v.carrier,
            funded_balance: v.funded_balance.map(Into::into),
            release_delay_hours: v.release_delay_hours,
            handover_window_days: v.handover_window_days,
            closable_dispute_ids: v.closable_dispute_ids,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub(crate) struct ReceiptOutput {
    order_id: i64,
    operation: FulfillmentOperation,
    status: ActionStateOutput,
    action_id: String,
    confirmation_url: Option<String>,
}
impl From<(FulfillmentReceipt, &str)> for ReceiptOutput {
    fn from((v, origin): (FulfillmentReceipt, &str)) -> Self {
        let confirmation_url = (v.status == AccountActionState::AwaitingConfirmation)
            .then(|| format!("{origin}/approvals/{}", v.action_id));
        Self {
            order_id: v.order_id,
            operation: v.operation,
            status: v.status.into(),
            action_id: v.action_id,
            confirmation_url,
        }
    }
}
