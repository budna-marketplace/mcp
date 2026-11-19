use super::{
    AccountActionState, AccountApiClient, ApiEnvelope, ClientError, DelegatedAccess,
    ORDERS_READ_SCOPE, invalid_input, invalid_response, is_action_id,
};
use crate::{Money, is_exact_decimal, is_listing_id, money_is_valid};
pub use budna_mcp_core::write_permissions::{
    ORDER_CLOSURES_WRITE_SCOPE, ORDERS_RELIST_WRITE_SCOPE,
};
use reqwest::{Method, header::AUTHORIZATION};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CloseOrderChoice {
    CancelOwnedOrder,
    CloseOverdueOrder,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderClosureMove {
    CancelOwnedOrder,
    CloseOverdueOrder,
    RelistClosedOrder,
}
impl From<CloseOrderChoice> for OrderClosureMove {
    fn from(o: CloseOrderChoice) -> Self {
        match o {
            CloseOrderChoice::CancelOwnedOrder => Self::CancelOwnedOrder,
            CloseOrderChoice::CloseOverdueOrder => Self::CloseOverdueOrder,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClosingRole {
    Buyer,
    Seller,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClosingRefund {
    None,
    RefundRequested,
    ManualReview,
    PartialPaymentRetained,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClosingOrigin {
    BuyNow,
    AuctionWin,
    OfferAccepted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClosedOrderState {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Completed,
    Refunded,
    Cancelled,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClosingPayment {
    Pending,
    Paid,
    Failed,
    Refunded,
    PartiallyRefunded,
}
#[derive(Debug, Deserialize)]
pub struct OwnOrderClosure {
    pub order_id: i64,
    pub listing_id: String,
    pub role: ClosingRole,
    pub order_status: ClosedOrderState,
    pub payment_status: ClosingPayment,
    pub payment_deadline: Option<i64>,
    pub closed_unpaid: bool,
    pub relisted_at: Option<i64>,
}
#[derive(Debug, Deserialize)]
pub struct OrderClosureTerms {
    pub quote_id: String,
    pub order_id: i64,
    pub listing_id: String,
    pub operation: OrderClosureMove,
    pub revision: String,
    pub role: ClosingRole,
    pub order_origin: ClosingOrigin,
    pub order_status: ClosedOrderState,
    pub payment_status: ClosingPayment,
    pub legacy_terms: bool,
    pub payment_due_at: Option<i64>,
    pub payment_deadline: Option<i64>,
    pub debt_payer: Option<ClosingRole>,
    pub debt_amount: Option<Money>,
    pub refund_amount: Option<Money>,
    pub refund_route: ClosingRefund,
    pub retained_partial: Option<Money>,
    pub restores_stock: bool,
    pub ends_order: bool,
    pub starts_listing: bool,
    pub relist_starting_price: Option<Money>,
    pub relist_buy_now_price: Option<Money>,
    pub relist_duration_ms: Option<i64>,
    pub expires_at: i64,
}
#[derive(Debug, Deserialize)]
pub struct OrderClosureQuote {
    pub terms: OrderClosureTerms,
    pub terms_digest: String,
}
#[derive(Debug, Deserialize)]
pub struct OrderClosureReceipt {
    pub action_id: String,
    pub status: AccountActionState,
    pub order_id: i64,
    pub operation: OrderClosureMove,
}
fn seal(s: &str) -> bool {
    s.len() == 43
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}
fn valid_optional_date(t: Option<i64>) -> bool {
    t.is_none_or(|n| n > 0)
}
impl AccountApiClient {
    pub async fn order_closure_state(
        &self,
        a: &DelegatedAccess,
        id: i64,
    ) -> Result<OwnOrderClosure, ClientError> {
        const OP: &str = "get_my_order_closure";
        self.require_scope(a, ORDERS_READ_SCOPE, OP)?;
        if id <= 0
            || !(self.write_permissions().order_closures || self.write_permissions().orders_relist)
        {
            return Err(invalid_input(OP, "Invalid order"));
        }
        let req = self
            .http
            .get(self.endpoint(OP, &format!("mcp/account/orders/{id}/closure"))?)
            .header(AUTHORIZATION, a.credential.header(OP)?);
        let env: ApiEnvelope<OwnOrderClosure> = self.send(OP, req, 4096).await?;
        env.data
            .filter(|r| {
                env.success
                    && r.order_id == id
                    && is_listing_id(&r.listing_id)
                    && valid_optional_date(r.payment_deadline)
                    && valid_optional_date(r.relisted_at)
                    && (!r.closed_unpaid || r.order_status == ClosedOrderState::Cancelled)
                    && (r.relisted_at.is_none() || r.closed_unpaid)
            })
            .ok_or_else(|| invalid_response(OP))
    }
    pub async fn quote_order_closure(
        &self,
        a: &DelegatedAccess,
        id: i64,
        operation: OrderClosureMove,
    ) -> Result<OrderClosureQuote, ClientError> {
        let relist = operation == OrderClosureMove::RelistClosedOrder;
        let (name, enabled) = if relist {
            (
                "prepare_my_closed_order_relist",
                self.write_permissions().orders_relist,
            )
        } else {
            (
                "prepare_my_order_closure",
                self.write_permissions().order_closures,
            )
        };
        self.require_scope(a, ORDERS_READ_SCOPE, name)?;
        if !enabled || id <= 0 {
            return Err(invalid_input(name, "Invalid order"));
        }
        let req = self
            .http
            .post(self.endpoint(name, &format!("mcp/account/orders/{id}/closure/quote"))?)
            .header(AUTHORIZATION, a.credential.header(name)?)
            .json(&serde_json::json!({"operation":operation}));
        let env: ApiEnvelope<OrderClosureQuote> = self.send(name, req, 8192).await?;
        env.data
            .filter(|r| {
                let t = &r.terms;
                let amounts = [
                    &t.debt_amount,
                    &t.refund_amount,
                    &t.retained_partial,
                    &t.relist_starting_price,
                    &t.relist_buy_now_price,
                ];
                let refund = match t.refund_route {
                    ClosingRefund::None => {
                        t.refund_amount.is_none() && t.retained_partial.is_none()
                    }
                    ClosingRefund::PartialPaymentRetained => t.retained_partial.is_some(),
                    ClosingRefund::ManualReview | ClosingRefund::RefundRequested => {
                        t.refund_amount.is_some() && t.retained_partial.is_none()
                    }
                };
                env.success
                    && t.operation == operation
                    && t.order_id == id
                    && seal(&r.terms_digest)
                    && seal(&t.revision)
                    && is_action_id(&t.quote_id)
                    && is_listing_id(&t.listing_id)
                    && t.expires_at > 0
                    && t.debt_payer.is_some() == t.debt_amount.is_some()
                    && amounts.into_iter().flatten().all(|m| {
                        money_is_valid(m, "SEK")
                            && is_exact_decimal(&m.amount)
                            && m.amount.split('.').nth(1).is_none_or(|f| f.len() <= 2)
                            && m.amount.len() <= 16
                    })
                    && valid_optional_date(t.payment_due_at)
                    && valid_optional_date(t.payment_deadline)
                    && refund
                    && t.ends_order != relist
                    && (!relist
                        || t.role == ClosingRole::Seller
                            && t.order_status == ClosedOrderState::Cancelled
                            && t.debt_amount.is_none()
                            && t.refund_route == ClosingRefund::None)
                    && (!t.starts_listing || relist && t.relist_starting_price.is_some())
                    && (t
                        .relist_duration_ms
                        .is_none_or(|ms| ms > 0 && ms <= 30 * 86_400_000))
                    && (operation != OrderClosureMove::CloseOverdueOrder
                        || t.role == ClosingRole::Seller)
            })
            .ok_or_else(|| invalid_response(name))
    }
    pub async fn submit_order_closure(
        &self,
        a: &DelegatedAccess,
        id: i64,
        op: OrderClosureMove,
        q: &str,
        digest: &str,
    ) -> Result<OrderClosureReceipt, ClientError> {
        let (name, scope) = if op == OrderClosureMove::RelistClosedOrder {
            ("relist_my_quoted_closed_order", ORDERS_RELIST_WRITE_SCOPE)
        } else {
            ("submit_my_quoted_order_closure", ORDER_CLOSURES_WRITE_SCOPE)
        };
        if id <= 0 || !is_action_id(q) || !seal(digest) {
            return Err(invalid_input(name, "Invalid reviewed order"));
        }
        let req = self
            .action_request(
                name,
                scope,
                a,
                Method::POST,
                &format!("mcp/account/orders/{id}/closure/actions"),
                q,
            )?
            .json(&serde_json::json!({"operation":op,"quote_id":q,"terms_digest":digest}));
        self.direct_receipt(name, req, |r: &OrderClosureReceipt| {
            is_action_id(&r.action_id) && r.order_id == id && r.operation == op
        })
        .await
    }
}
