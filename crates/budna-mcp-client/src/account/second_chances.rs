use super::{
    AccountActionState, AccountApiClient, ApiEnvelope, BidCurrency, ClientError, DelegatedAccess,
    OFFERS_READ_SCOPE, ORDERS_READ_SCOPE, invalid_input, invalid_response, is_action_id,
};
use crate::{
    Money, decimal_is_less_than_or_equal, is_exact_decimal, is_listing_id, money_is_valid,
};
pub use budna_mcp_core::write_permissions::{
    SECOND_CHANCE_ANSWERS_WRITE_SCOPE, SECOND_CHANCES_WRITE_SCOPE,
};
use reqwest::{Method, header::AUTHORIZATION};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SecondChancePrice {
    #[schemars(
        length(min = 1, max = 13),
        regex(pattern = r"^(0|[1-9][0-9]{0,9})(\.[0-9]{1,2})?$")
    )]
    pub amount: String,
    pub currency_code: BidCurrency,
}
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum SecondChanceSale {
    StartSecondChance {
        #[schemars(range(min = 1))]
        source_order_id: i64,
        price: SecondChancePrice,
    },
    CancelSecondChance {
        #[schemars(range(min = 1))]
        source_order_id: i64,
        #[schemars(range(min = 1))]
        offer_id: i64,
    },
}
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum SecondChanceAnswer {
    AcceptSecondChance {
        #[schemars(range(min = 1))]
        offer_id: i64,
        #[schemars(range(min = 1))]
        invitation_id: i64,
    },
    DeclineSecondChance {
        #[schemars(range(min = 1))]
        offer_id: i64,
        #[schemars(range(min = 1))]
        invitation_id: i64,
    },
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChanceOperation {
    StartSecondChance,
    CancelSecondChance,
    AcceptSecondChance,
    DeclineSecondChance,
}
#[derive(Debug, Deserialize)]
pub struct SecondChanceSaleState {
    pub source_order_id: i64,
    pub listing_id: String,
    pub maximum_price: Option<Money>,
    pub active_offer_id: Option<i64>,
    pub can_start: bool,
    pub can_cancel: bool,
}
#[derive(Debug, Deserialize)]
pub struct SecondChanceInvitation {
    pub offer_id: i64,
    pub invitation_id: i64,
    pub listing_id: String,
    pub price: Money,
    pub window_ends_at: i64,
}
#[derive(Debug, Deserialize)]
pub struct ChanceWireChange {
    pub operation: ChanceOperation,
    pub source_order_id: Option<i64>,
    pub offer_id: Option<i64>,
    pub invitation_id: Option<i64>,
    pub price: Option<Money>,
}
#[derive(Debug, Deserialize)]
pub struct SecondChanceReview {
    pub quote_id: String,
    pub change: ChanceWireChange,
    pub listing_id: String,
    pub revision: String,
    pub price: Money,
    pub maximum_seller_price: Option<Money>,
    pub current_window_ends_at: Option<i64>,
    pub response_window_hours: u32,
    pub payment_due_hours: u32,
    pub payment_grace_hours: u32,
    pub buyer_protection_fee: Money,
    pub buyer_protection_mandatory: bool,
    pub high_value_fee: Money,
    pub unpaid_fee: Money,
    pub delivery_at_checkout: bool,
    pub expires_at: i64,
}
#[derive(Debug, Deserialize)]
pub struct SecondChanceQuote {
    pub terms: SecondChanceReview,
    pub terms_digest: String,
}
#[derive(Debug, Deserialize)]
pub struct SecondChanceReceipt {
    pub action_id: String,
    pub status: AccountActionState,
    pub operation: ChanceOperation,
    pub source_order_id: Option<i64>,
    pub invitation_id: Option<i64>,
    pub offer_id: Option<i64>,
    pub order_id: Option<i64>,
    pub order_status: Option<ChanceOrderStatus>,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ChanceOrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Completed,
    Cancelled,
    Refunded,
}
fn digest(v: &str) -> bool {
    v.len() == 43
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
}
fn amount(v: &str, zero: bool) -> bool {
    let (w, f) = v.split_once('.').unwrap_or((v, ""));
    is_exact_decimal(v)
        && w.len() <= 10
        && (w == "0" || !w.starts_with('0'))
        && f.len() <= 2
        && (zero || !decimal_is_less_than_or_equal(v, "0"))
}
fn money(v: &Money, zero: bool) -> bool {
    money_is_valid(v, "SEK") && amount(&v.amount, zero)
}
pub enum ChanceIntent<'a> {
    Seller(&'a SecondChanceSale),
    Buyer(&'a SecondChanceAnswer),
}
struct Identity<'a> {
    operation: ChanceOperation,
    source: Option<i64>,
    offer: Option<i64>,
    invite: Option<i64>,
    price: Option<&'a str>,
}
impl ChanceIntent<'_> {
    fn fields(&self) -> Identity<'_> {
        use ChanceOperation::*;
        let (op, src, off, inv, price) = match self {
            Self::Seller(SecondChanceSale::StartSecondChance {
                source_order_id,
                price,
            }) => (
                StartSecondChance,
                Some(*source_order_id),
                None,
                None,
                Some(price.amount.as_str()),
            ),
            Self::Seller(SecondChanceSale::CancelSecondChance {
                source_order_id,
                offer_id,
            }) => (
                CancelSecondChance,
                Some(*source_order_id),
                Some(*offer_id),
                None,
                None,
            ),
            Self::Buyer(SecondChanceAnswer::AcceptSecondChance {
                offer_id,
                invitation_id,
            }) => (
                AcceptSecondChance,
                None,
                Some(*offer_id),
                Some(*invitation_id),
                None,
            ),
            Self::Buyer(SecondChanceAnswer::DeclineSecondChance {
                offer_id,
                invitation_id,
            }) => (
                DeclineSecondChance,
                None,
                Some(*offer_id),
                Some(*invitation_id),
                None,
            ),
        };
        Identity {
            operation: op,
            source: src,
            offer: off,
            invite: inv,
            price,
        }
    }
    fn valid(&self) -> bool {
        let f = self.fields();
        [f.source, f.offer, f.invite]
            .into_iter()
            .flatten()
            .all(|id| id > 0)
            && f.price.is_none_or(|s| amount(s, false))
    }
    fn body(&self) -> serde_json::Value {
        match self {
            Self::Seller(s) => serde_json::json!(s),
            Self::Buyer(b) => serde_json::json!(b),
        }
    }
    fn seller(&self) -> bool {
        matches!(self, Self::Seller(_))
    }
}
impl SecondChanceQuote {
    fn matches(&self, c: &ChanceIntent<'_>) -> bool {
        let q = &self.terms;
        let f = c.fields();
        let w = &q.change;
        let exact = |x: &str, y: &str| {
            decimal_is_less_than_or_equal(x, y) && decimal_is_less_than_or_equal(y, x)
        };
        digest(&self.terms_digest)
            && is_action_id(&q.quote_id)
            && digest(&q.revision)
            && is_listing_id(&q.listing_id)
            && f.operation == w.operation
            && f.source == w.source_order_id
            && f.offer == w.offer_id
            && f.invite == w.invitation_id
            && match (f.price, &w.price) {
                (Some(total), Some(v)) => {
                    money(v, false) && exact(total, &v.amount) && exact(total, &q.price.amount)
                }
                (None, None) => true,
                _ => false,
            }
            && money(&q.price, false)
            && c.seller() == q.maximum_seller_price.is_some()
            && q.maximum_seller_price.as_ref().is_none_or(|m| {
                money(m, false) && decimal_is_less_than_or_equal(&q.price.amount, &m.amount)
            })
            && [&q.buyer_protection_fee, &q.high_value_fee, &q.unpaid_fee]
                .into_iter()
                .all(|m| money(m, true))
            && q.response_window_hours > 0
            && q.payment_due_hours > 0
            && q.payment_grace_hours == 0
            && q.delivery_at_checkout
            && q.expires_at > 0
            && (c.seller() || q.current_window_ends_at.is_some_and(|t| t >= q.expires_at))
    }
}
impl AccountApiClient {
    pub async fn second_chance_sale_state(
        &self,
        a: &DelegatedAccess,
        src: i64,
    ) -> Result<SecondChanceSaleState, ClientError> {
        const OP: &str = "get_my_second_chance_sale";
        self.require_scope(a, ORDERS_READ_SCOPE, OP)?;
        if !self.write_permissions().second_chances || src <= 0 {
            return Err(invalid_input(OP, "Invalid source order"));
        }
        let req = self
            .http
            .get(self.endpoint(OP, &format!("mcp/account/orders/{src}/second-chance"))?)
            .header(AUTHORIZATION, a.credential.header(OP)?);
        let env: ApiEnvelope<SecondChanceSaleState> = self.send(OP, req, 4096).await?;
        env.data
            .filter(|v| {
                env.success
                    && v.source_order_id == src
                    && is_listing_id(&v.listing_id)
                    && v.maximum_price.as_ref().is_none_or(|m| money(m, false))
                    && (!v.can_start || v.maximum_price.is_some())
                    && v.can_cancel == v.active_offer_id.is_some()
                    && v.active_offer_id.is_none_or(|id| id > 0)
                    && !(v.can_cancel && v.can_start)
            })
            .ok_or_else(|| invalid_response(OP))
    }
    pub async fn second_chance_invitation(
        &self,
        a: &DelegatedAccess,
        offer: i64,
    ) -> Result<SecondChanceInvitation, ClientError> {
        const OP: &str = "get_my_second_chance_invitation";
        self.require_scope(a, OFFERS_READ_SCOPE, OP)?;
        if !self.write_permissions().second_chance_answers || offer <= 0 {
            return Err(invalid_input(OP, "Invalid invitation offer"));
        }
        let req = self
            .http
            .get(self.endpoint(
                OP,
                &format!("mcp/account/second-chances/{offer}/invitation"),
            )?)
            .header(AUTHORIZATION, a.credential.header(OP)?);
        let env: ApiEnvelope<SecondChanceInvitation> = self.send(OP, req, 4096).await?;
        env.data
            .filter(|v| {
                env.success
                    && v.offer_id == offer
                    && v.invitation_id > 0
                    && is_listing_id(&v.listing_id)
                    && v.window_ends_at > 0
                    && money(&v.price, false)
            })
            .ok_or_else(|| invalid_response(OP))
    }
    pub async fn prepare_second_chance(
        &self,
        a: &DelegatedAccess,
        c: ChanceIntent<'_>,
    ) -> Result<SecondChanceQuote, ClientError> {
        let (s, op, enabled) = if c.seller() {
            (
                ORDERS_READ_SCOPE,
                "prepare_my_second_chance_sale",
                self.write_permissions().second_chances,
            )
        } else {
            (
                OFFERS_READ_SCOPE,
                "prepare_my_second_chance_answer",
                self.write_permissions().second_chance_answers,
            )
        };
        self.require_scope(a, s, op)?;
        if !enabled || !c.valid() {
            return Err(invalid_input(op, "Invalid second chance"));
        }
        let req = self
            .http
            .post(self.endpoint(op, "mcp/account/second-chances/quote")?)
            .header(AUTHORIZATION, a.credential.header(op)?)
            .json(&c.body());
        let env: ApiEnvelope<SecondChanceQuote> = self.send(op, req, 8192).await?;
        env.data
            .filter(|v| env.success && v.matches(&c))
            .ok_or_else(|| invalid_response(op))
    }
    pub async fn submit_second_chance(
        &self,
        a: &DelegatedAccess,
        c: ChanceIntent<'_>,
        q: &str,
        seal: &str,
    ) -> Result<SecondChanceReceipt, ClientError> {
        let (s, op) = if c.seller() {
            (
                SECOND_CHANCES_WRITE_SCOPE,
                "submit_my_quoted_second_chance_sale",
            )
        } else {
            (
                SECOND_CHANCE_ANSWERS_WRITE_SCOPE,
                "submit_my_quoted_second_chance_answer",
            )
        };
        if !c.valid() || !is_action_id(q) || !digest(seal) {
            return Err(invalid_input(op, "Invalid reviewed second chance"));
        }
        let req = self
            .action_request(
                op,
                s,
                a,
                Method::POST,
                "mcp/account/second-chances/actions",
                q,
            )?
            .json(&serde_json::json!({"quote_id":q,"terms_digest":seal,"change":c.body()}));
        let f = c.fields();
        self.direct_receipt(op, req, |r: &SecondChanceReceipt| {
            is_action_id(&r.action_id)
                && r.operation == f.operation
                && r.invitation_id == f.invite
                && r.source_order_id == f.source
                && r.offer_id.is_none_or(|id| id > 0)
                && f.offer.is_none_or(|v| Some(v) == r.offer_id)
                && r.order_id.is_none_or(|id| id > 0)
                && r.order_status.is_some() == r.order_id.is_some()
                && if r.status == AccountActionState::Executed {
                    r.offer_id.is_some()
                        && r.order_id.is_some()
                            == (f.operation == ChanceOperation::AcceptSecondChance)
                } else {
                    r.offer_id == f.offer && r.order_id.is_none()
                }
        })
        .await
    }
}
