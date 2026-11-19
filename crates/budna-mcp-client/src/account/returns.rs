use super::{
    AccountActionState, AccountApiClient, ClientError, DelegatedAccess, actions::valid_action_text,
    invalid_input, invalid_response, is_action_id, shipping::valid_revision,
};
use crate::{Money, is_listing_id, money_is_valid};
use budna_mcp_core::write_permissions::{RETURN_REVIEW_SCOPE, RETURNS_WRITE_SCOPE};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReturnCarrier {
    Postnord,
    Dsv,
    Dhl,
    Other,
}
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReturnChange {
    RequestWithdrawal {},
    AcceptAgreed {},
    ConfirmReceived {},
    SetTracking {
        carrier: ReturnCarrier,
        #[schemars(length(min = 1, max = 100))]
        tracking_number: String,
    },
    Cancel {},
}
impl ReturnChange {
    fn name(&self) -> &'static str {
        match self {
            Self::RequestWithdrawal {} => "request_statutory_withdrawal",
            Self::AcceptAgreed {} => "accept_agreed_return",
            Self::ConfirmReceived {} => "confirm_return_received",
            Self::SetTracking { .. } => "set_own_return_tracking",
            Self::Cancel {} => "cancel_my_return",
        }
    }
}
#[derive(Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReturnRole {
    Buyer,
    Seller,
}
#[derive(Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReturnKind {
    Agreed,
    Statutory,
    BuyerProtection,
}
#[derive(Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReturnState {
    Requested,
    ReturnShipped,
    ReturnReceived,
    RefundProcessing,
    Completed,
    Cancelled,
}
#[derive(Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RefundVia {
    Budna,
    ManualReview,
    Seller,
    Undetermined,
}
#[derive(Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReturnPostage {
    Buyer,
    Seller,
    None,
}
#[derive(Debug, Deserialize)]
pub struct ReturnTerms {
    pub revision: String,
    pub order_id: i64,
    pub listing_id: String,
    pub role: ReturnRole,
    pub return_kind: Option<ReturnKind>,
    pub status: Option<ReturnState>,
    pub consumer_buyer: bool,
    pub business_seller: bool,
    pub order_amount: Money,
    pub refundable_amount: Option<Money>,
    pub refund_via: RefundVia,
    pub pre_dispatch: bool,
    pub return_shipping_payer: ReturnPostage,
    pub withdrawal_deadline: Option<i64>,
    pub online_refund_deadline: Option<i64>,
    pub has_own_tracking: bool,
    pub budna_label_booked: bool,
    pub buyer_protection_label: bool,
    pub can_request_withdrawal: bool,
    pub can_accept_agreed: bool,
    pub can_confirm_received: bool,
    pub can_set_tracking: bool,
    pub can_cancel: bool,
}
#[derive(Debug, Deserialize)]
pub struct ReturnReceipt {
    pub order_id: i64,
    pub action_id: String,
    pub status: AccountActionState,
    pub action: String,
}
impl AccountApiClient {
    pub async fn return_terms(
        &self,
        access: &DelegatedAccess,
        order_id: i64,
    ) -> Result<ReturnTerms, ClientError> {
        const OP: &str = "get_my_return_terms";
        self.require_scope(access, RETURN_REVIEW_SCOPE, OP)?;
        if order_id <= 0 {
            return Err(invalid_input(OP, "Use a positive order ID"));
        }
        let q: ReturnTerms = self
            .get(
                OP,
                &format!("mcp/account/orders/{order_id}/return-actions"),
                access,
                &[],
            )
            .await?;
        if q.order_id != order_id
            || !is_listing_id(&q.listing_id)
            || !valid_revision(&q.revision)
            || !money_is_valid(&q.order_amount, &q.order_amount.currency_code)
            || q.order_amount.amount.starts_with('-')
            || q.refundable_amount.as_ref().is_some_and(|p| {
                p.amount.starts_with('-') || !money_is_valid(p, &q.order_amount.currency_code)
            })
            || q.withdrawal_deadline.is_some_and(|t| t <= 0)
            || q.online_refund_deadline.is_some_and(|t| t <= 0)
            || (q.role == ReturnRole::Buyer && (q.can_accept_agreed || q.can_confirm_received))
            || (q.role == ReturnRole::Seller
                && (q.can_set_tracking || q.can_cancel || q.can_request_withdrawal))
            || (q.can_request_withdrawal && (!q.business_seller || !q.consumer_buyer))
        {
            return Err(invalid_response(OP));
        }
        Ok(q)
    }
    pub async fn change_return(
        &self,
        access: &DelegatedAccess,
        order_id: i64,
        expected_revision: &str,
        change: &ReturnChange,
        key: &str,
    ) -> Result<ReturnReceipt, ClientError> {
        const OP: &str = "change_my_order_return";
        if order_id <= 0 || !valid_revision(expected_revision) {
            return Err(invalid_input(
                OP,
                "Use a positive order ID and current revision",
            ));
        }
        if let ReturnChange::SetTracking {
            tracking_number: s, ..
        } = change
            && (!valid_action_text(s, 100)
                || s.trim() != s
                || !s.chars().any(char::is_alphanumeric))
        {
            return Err(invalid_input(OP, "Invalid return tracking number"));
        }
        #[derive(Serialize)]
        struct Request<'a> {
            order_id: i64,
            expected_revision: &'a str,
            #[serde(flatten)]
            change: &'a ReturnChange,
        }
        let req = self
            .action_request(
                OP,
                RETURNS_WRITE_SCOPE,
                access,
                Method::POST,
                "mcp/account/returns/change",
                key,
            )?
            .json(&Request {
                order_id,
                expected_revision,
                change,
            });
        self.direct_receipt(OP, req, |r: &ReturnReceipt| {
            r.order_id == order_id
                && r.action == change.name()
                && is_action_id(&r.action_id)
                && (!matches!(
                    change,
                    ReturnChange::SetTracking { .. } | ReturnChange::Cancel {}
                ) || r.status != AccountActionState::AwaitingConfirmation)
        })
        .await
    }
}
