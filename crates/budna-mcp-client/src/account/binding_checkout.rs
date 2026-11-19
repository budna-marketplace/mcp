use super::{
    AccountActionState, AccountApiClient, ApiEnvelope, ClientError, DelegatedAccess,
    ORDERS_READ_SCOPE, invalid_input, invalid_response, is_action_id,
};
use crate::{Money, is_exact_decimal, is_listing_id, money_is_valid};
pub use budna_mcp_core::write_permissions::CHECKOUT_ORDERS_WRITE_SCOPE;
use reqwest::{Method, header::AUTHORIZATION};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PersonalCheckoutPhase {
    NeedsCheckout,
    NeedsSelection,
    Ready,
    BindingOrder,
    ReservedOrder,
    Unavailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CheckoutDelivery {
    CarrierTracked,
    ManualUntracked,
    Meetup,
}
#[derive(Debug, Deserialize)]
pub struct PersonalCheckoutState {
    pub listing_id: String,
    pub session_id: Option<String>,
    pub state: PersonalCheckoutPhase,
    pub order_id: Option<i64>,
    pub payment_deadline: Option<i64>,
}
#[derive(Debug, Deserialize)]
pub struct BindingCheckoutTerms {
    pub quote_id: String,
    pub session_id: String,
    pub listing_id: String,
    pub revision: String,
    pub quantity: i32,
    pub item_price: Money,
    pub fulfillment_mode: CheckoutDelivery,
    pub shipping_cost: Money,
    pub buyer_protection_applied: bool,
    pub buyer_protection_fee: Money,
    pub high_value_fee: Money,
    pub total: Money,
    pub unpaid_fee: Money,
    pub quoted_at: i64,
    pub payment_deadline: i64,
    pub ends_auction: bool,
    pub expires_at: i64,
}
#[derive(Debug, Deserialize)]
pub struct BindingCheckoutQuote {
    pub terms: BindingCheckoutTerms,
    pub terms_digest: String,
}
#[derive(Debug, Deserialize)]
pub struct BindingCheckoutReceipt {
    pub action_id: String,
    pub status: AccountActionState,
    pub listing_id: String,
    pub session_id: String,
    pub order_id: Option<i64>,
    pub payment_deadline: Option<i64>,
}
fn seal(v: &str) -> bool {
    v.len() == 43
        && v.bytes()
            .all(|x| x.is_ascii_alphanumeric() || matches!(x, b'_' | b'-'))
}
// All components share the documented SEK two-decimal contract. Validate the exact sum without a float.
fn cents(m: &Money) -> Option<u64> {
    if !money_is_valid(m, "SEK") || m.amount.len() > 14 || !is_exact_decimal(&m.amount) {
        return None;
    }
    let (whole, part) = m.amount.split_once('.').unwrap_or((&m.amount, ""));
    if part.len() > 2 || (whole.len() > 1 && whole.starts_with('0')) {
        return None;
    }
    let minor = match part.len() {
        0 => 0,
        1 => part.parse::<u64>().ok()? * 10,
        _ => part.parse::<u64>().ok()?,
    };
    whole
        .parse::<u64>()
        .ok()?
        .checked_mul(100)?
        .checked_add(minor)
}
impl AccountApiClient {
    pub async fn personal_checkout(
        &self,
        a: &DelegatedAccess,
        listing: &str,
    ) -> Result<PersonalCheckoutState, ClientError> {
        const OP: &str = "get_my_binding_checkout";
        self.require_scope(a, ORDERS_READ_SCOPE, OP)?;
        if !self.write_permissions().checkout_orders || !is_listing_id(listing) {
            return Err(invalid_input(OP, "Invalid listing"));
        }
        let req = self
            .http
            .get(self.endpoint(
                OP,
                &format!("mcp/account/listings/{listing}/binding-checkout"),
            )?)
            .header(AUTHORIZATION, a.credential.header(OP)?);
        let env: ApiEnvelope<PersonalCheckoutState> = self.send(OP, req, 4096).await?;
        env.data
            .filter(|r| {
                env.success
                    && r.listing_id == listing
                    && r.session_id.as_ref().is_none_or(|v| is_action_id(v))
                    && (r.state == PersonalCheckoutPhase::NeedsCheckout) == r.session_id.is_none()
                    && r.order_id.is_none_or(|id| id > 0)
                    && r.order_id.is_some()
                        == matches!(
                            r.state,
                            PersonalCheckoutPhase::BindingOrder
                                | PersonalCheckoutPhase::ReservedOrder
                        )
                    && r.payment_deadline
                        .is_none_or(|t| r.order_id.is_some() && t > 0)
            })
            .ok_or_else(|| invalid_response(OP))
    }
    pub async fn quote_binding_checkout(
        &self,
        a: &DelegatedAccess,
        listing: &str,
        session: &str,
    ) -> Result<BindingCheckoutQuote, ClientError> {
        const OP: &str = "prepare_my_binding_checkout_order";
        self.require_scope(a, ORDERS_READ_SCOPE, OP)?;
        if !self.write_permissions().checkout_orders
            || !is_listing_id(listing)
            || !is_action_id(session)
        {
            return Err(invalid_input(OP, "Invalid checkout"));
        }
        let req = self
            .http
            .post(self.endpoint(
                OP,
                &format!("mcp/account/listings/{listing}/binding-checkout/quote"),
            )?)
            .header(AUTHORIZATION, a.credential.header(OP)?)
            .json(&serde_json::json!({"session_id":session}));
        let env: ApiEnvelope<BindingCheckoutQuote> = self.send(OP, req, 8192).await?;
        env.data
            .filter(|r| {
                let q = &r.terms;
                let sum = [
                    &q.item_price,
                    &q.shipping_cost,
                    &q.buyer_protection_fee,
                    &q.high_value_fee,
                ]
                .into_iter()
                .try_fold(0_u64, |v, m| v.checked_add(cents(m)?));
                env.success
                    && is_action_id(&q.quote_id)
                    && q.session_id == session
                    && q.listing_id == listing
                    && seal(&r.terms_digest)
                    && seal(&q.revision)
                    && q.quantity > 0
                    && cents(&q.item_price).is_some_and(|v| v > 0)
                    && sum.is_some()
                    && sum == cents(&q.total)
                    && cents(&q.unpaid_fee).is_some()
                    && (q.buyer_protection_applied || cents(&q.buyer_protection_fee) == Some(0))
                    && q.quoted_at > 0
                    && q.quoted_at < q.expires_at
                    && q.expires_at < q.payment_deadline
            })
            .ok_or_else(|| invalid_response(OP))
    }
    pub async fn place_binding_checkout(
        &self,
        a: &DelegatedAccess,
        listing: &str,
        session: &str,
        quote: &str,
        digest: &str,
    ) -> Result<BindingCheckoutReceipt, ClientError> {
        const OP: &str = "submit_my_quoted_checkout_order";
        if !is_listing_id(listing)
            || !is_action_id(session)
            || !is_action_id(quote)
            || !seal(digest)
        {
            return Err(invalid_input(OP, "Invalid reviewed checkout"));
        }
        let req = self
            .action_request(
                OP,
                CHECKOUT_ORDERS_WRITE_SCOPE,
                a,
                Method::POST,
                &format!("mcp/account/listings/{listing}/binding-checkout/order"),
                quote,
            )?
            .json(
                &serde_json::json!({"session_id":session,"quote_id":quote,"terms_digest":digest}),
            );
        self.direct_receipt(OP, req, |r: &BindingCheckoutReceipt| {
            is_action_id(&r.action_id)
                && r.session_id == session
                && r.listing_id == listing
                && r.order_id.is_some() == (r.status == AccountActionState::Executed)
                && r.order_id.is_none_or(|id| id > 0)
                && r.payment_deadline.is_some() == r.order_id.is_some()
                && r.payment_deadline.is_none_or(|t| t > 0)
        })
        .await
    }
}
