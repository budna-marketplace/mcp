use super::{account::ActionStateOutput, common::MoneyOutput};
use budna_mcp_client::{
    AccountActionState, CaseActionReceipt, CaseOffer, CaseSide, DisputeOptions,
};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct CurrentOffer {
    offer_id: i64,
    amount: MoneyOutput,
    expires_at: i64,
}
impl From<CaseOffer> for CurrentOffer {
    fn from(o: CaseOffer) -> Self {
        Self {
            offer_id: o.offer_id,
            amount: o.amount.into(),
            expires_at: o.expires_at,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DisputeOptionsOutput {
    order_id: i64,
    listing_id: String,
    order_revision: i64,
    side: CaseSide,
    can_open: bool,
    payout_can_be_held: bool,
    current_dispute_id: Option<i64>,
    case_revision: Option<i64>,
    can_withdraw_claim: bool,
    can_propose: bool,
    can_accept: bool,
    can_decline: bool,
    can_withdraw_offer: bool,
    can_submit_statement: bool,
    current_offer: Option<CurrentOffer>,
    maximum_discount: Option<MoneyOutput>,
}
impl From<DisputeOptions> for DisputeOptionsOutput {
    fn from(r: DisputeOptions) -> Self {
        Self {
            order_id: r.order_id,
            listing_id: r.listing_id,
            order_revision: r.order_revision,
            side: r.side,
            can_open: r.can_open,
            payout_can_be_held: r.payout_can_be_held,
            current_dispute_id: r.current_dispute_id,
            case_revision: r.case_revision,
            can_withdraw_claim: r.can_withdraw_claim,
            can_propose: r.can_propose,
            can_accept: r.can_accept,
            can_decline: r.can_decline,
            can_withdraw_offer: r.can_withdraw_offer,
            can_submit_statement: r.can_submit_statement,
            current_offer: r.current_offer.map(Into::into),
            maximum_discount: r.maximum_discount.map(Into::into),
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CaseReceiptOutput {
    action_id: String,
    order_id: i64,
    operation: String,
    status: ActionStateOutput,
    dispute_id: Option<i64>,
    statement_id: Option<i64>,
    offer_id: Option<i64>,
    confirmation_url: Option<String>,
}
impl From<(CaseActionReceipt, &str)> for CaseReceiptOutput {
    fn from((r, origin): (CaseActionReceipt, &str)) -> Self {
        let confirmation_url = (r.status == AccountActionState::AwaitingConfirmation)
            .then(|| format!("{origin}/approvals/{}", r.action_id));
        Self {
            action_id: r.action_id,
            order_id: r.order_id,
            operation: r.operation,
            status: r.status.into(),
            dispute_id: r.dispute_id,
            statement_id: r.statement_id,
            offer_id: r.offer_id,
            confirmation_url,
        }
    }
}
