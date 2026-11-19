use super::{
    AccountActionState, AccountApiClient, ApiEnvelope, BidCurrency, ClientError, DelegatedAccess,
    OFFERS_READ_SCOPE, invalid_input, invalid_response, is_action_id,
};
use crate::{
    Money, decimal_is_less_than_or_equal, is_exact_decimal, is_listing_id, money_is_valid,
};
pub use budna_mcp_core::write_permissions::OFFERS_WRITE_SCOPE;
use reqwest::{Method, header::AUTHORIZATION};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NegotiationMove {
    MakeOffer,
    RaiseOffer,
    CounterOffer,
    LowerOfferCounter,
    AcceptOffer,
    RejectOffer,
    WithdrawOffer,
    WithdrawOfferCounter,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NegotiationParty {
    Buyer,
    Seller,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OfferTotal {
    #[schemars(
        length(min = 1, max = 11),
        regex(pattern = r"^(0|[1-9][0-9]{0,7})(\.[0-9]{1,2})?$")
    )]
    pub amount: String,
    pub currency_code: BidCurrency,
}
fn offer_value(value: &str, allow_zero: bool) -> bool {
    let (whole, frac) = value.split_once('.').unwrap_or((value, ""));
    is_exact_decimal(value)
        && (whole == "0" || !whole.starts_with('0'))
        && whole.len() <= 8
        && frac.len() <= 2
        && (allow_zero || !decimal_is_less_than_or_equal(value, "0"))
}
fn valid_digest(v: &str) -> bool {
    v.len() == 43
        && v.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum NegotiationChange {
    MakeOffer {
        #[schemars(range(min = 1, max = 10000))]
        quantity: u16,
        total: OfferTotal,
    },
    RaiseOffer {
        #[schemars(range(min = 1))]
        offer_id: i64,
        total: OfferTotal,
    },
    CounterOffer {
        #[schemars(range(min = 1))]
        offer_id: i64,
        total: OfferTotal,
    },
    LowerOfferCounter {
        #[schemars(range(min = 1))]
        offer_id: i64,
        total: OfferTotal,
    },
    AcceptOffer {
        #[schemars(range(min = 1))]
        offer_id: i64,
    },
    RejectOffer {
        #[schemars(range(min = 1))]
        offer_id: i64,
    },
    WithdrawOffer {
        #[schemars(range(min = 1))]
        offer_id: i64,
    },
    WithdrawOfferCounter {
        #[schemars(range(min = 1))]
        offer_id: i64,
    },
}
impl NegotiationChange {
    fn parts(
        &self,
    ) -> (
        NegotiationMove,
        Option<i64>,
        Option<u16>,
        Option<&OfferTotal>,
    ) {
        use NegotiationChange::*;
        match self {
            MakeOffer { quantity, total } => (
                NegotiationMove::MakeOffer,
                None,
                Some(*quantity),
                Some(total),
            ),
            RaiseOffer { offer_id, total } => (
                NegotiationMove::RaiseOffer,
                Some(*offer_id),
                None,
                Some(total),
            ),
            CounterOffer { offer_id, total } => (
                NegotiationMove::CounterOffer,
                Some(*offer_id),
                None,
                Some(total),
            ),
            LowerOfferCounter { offer_id, total } => (
                NegotiationMove::LowerOfferCounter,
                Some(*offer_id),
                None,
                Some(total),
            ),
            AcceptOffer { offer_id } => (NegotiationMove::AcceptOffer, Some(*offer_id), None, None),
            RejectOffer { offer_id } => (NegotiationMove::RejectOffer, Some(*offer_id), None, None),
            WithdrawOffer { offer_id } => {
                (NegotiationMove::WithdrawOffer, Some(*offer_id), None, None)
            }
            WithdrawOfferCounter { offer_id } => (
                NegotiationMove::WithdrawOfferCounter,
                Some(*offer_id),
                None,
                None,
            ),
        }
    }
    fn valid(&self) -> bool {
        let (_, id, qty, m) = self.parts();
        id.is_none_or(|x| x > 0)
            && qty.is_none_or(|q| (1..=10000).contains(&q))
            && m.is_none_or(|m| offer_value(&m.amount, false))
    }
}
#[derive(Debug, Deserialize)]
pub struct WireOfferProposal {
    pub operation: NegotiationMove,
    pub offer_id: Option<i64>,
    pub quantity: Option<u16>,
    pub total: Option<Money>,
}
#[derive(Debug, Deserialize)]
pub struct NegotiationTerms {
    pub quote_id: String,
    pub listing_id: String,
    pub proposal: WireOfferProposal,
    pub revision: String,
    pub role: NegotiationParty,
    pub quantity: u16,
    pub total: Money,
    pub standing_buyer_total: Option<Money>,
    pub standing_seller_total: Option<Money>,
    pub remaining_buyer_total: Option<Money>,
    pub remaining_seller_total: Option<Money>,
    pub next_responder: Option<NegotiationParty>,
    pub current_offer_expires_at: Option<i64>,
    pub response_window_hours: Option<u32>,
    pub buyer_protection_fee: Money,
    pub buyer_protection_mandatory: bool,
    pub high_value_fee: Money,
    pub unpaid_fee: Money,
    pub payment_due_hours: u32,
    pub payment_grace_hours: u32,
    pub delivery_at_checkout: bool,
    pub reserves_stock: bool,
    pub expires_at: i64,
}
#[derive(Debug, Deserialize)]
pub struct NegotiationPreview {
    pub terms: NegotiationTerms,
    pub terms_digest: String,
}
#[derive(Debug, Deserialize)]
pub struct NegotiationReceipt {
    pub action_id: String,
    pub status: AccountActionState,
    pub listing_id: String,
    pub operation: NegotiationMove,
    pub offer_id: Option<i64>,
    pub order_id: Option<i64>,
}
impl NegotiationTerms {
    fn matches(&self, listing: &str, p: &NegotiationChange) -> bool {
        let (op, id, qty, input) = p.parts();
        let m = &self.proposal;
        let positive = |v: &Money| money_is_valid(v, "SEK") && offer_value(&v.amount, false);
        let fee = |v: &Money| money_is_valid(v, "SEK") && offer_value(&v.amount, true);
        let total_matches = match (input, &m.total) {
            (Some(i), Some(o)) => {
                positive(o)
                    && decimal_is_less_than_or_equal(&i.amount, &o.amount)
                    && decimal_is_less_than_or_equal(&o.amount, &i.amount)
                    && o == &self.total
            }
            (None, None) => true,
            _ => false,
        };
        is_action_id(&self.quote_id)
            && self.listing_id == listing
            && valid_digest(&self.revision)
            && op == m.operation
            && id == m.offer_id
            && qty == m.quantity
            && qty.is_none_or(|q| q == self.quantity)
            && (1..=10000).contains(&self.quantity)
            && positive(&self.total)
            && total_matches
            && [
                &self.standing_buyer_total,
                &self.standing_seller_total,
                &self.remaining_buyer_total,
                &self.remaining_seller_total,
            ]
            .into_iter()
            .flatten()
            .all(positive)
            && [
                &self.buyer_protection_fee,
                &self.high_value_fee,
                &self.unpaid_fee,
            ]
            .into_iter()
            .all(fee)
            && self.reserves_stock == (op == NegotiationMove::AcceptOffer)
            && self.delivery_at_checkout
            && self.payment_due_hours > 0
            && self.expires_at > 0
            && self
                .current_offer_expires_at
                .is_none_or(|t| self.expires_at <= t)
            && self.response_window_hours.is_none_or(|x| x > 0)
            && match self.next_responder {
                None => {
                    self.remaining_buyer_total.is_none() && self.remaining_seller_total.is_none()
                }
                Some(NegotiationParty::Buyer) => self.remaining_seller_total.is_some(),
                Some(NegotiationParty::Seller) => self.remaining_buyer_total.is_some(),
            }
    }
}
impl AccountApiClient {
    pub async fn prepare_offer_change(
        &self,
        a: &DelegatedAccess,
        listing: &str,
        p: &NegotiationChange,
    ) -> Result<NegotiationPreview, ClientError> {
        const OP: &str = "prepare_my_offer_change";
        self.require_scope(a, OFFERS_READ_SCOPE, OP)?;
        if !self.write_permissions().offers || !is_listing_id(listing) || !p.valid() {
            return Err(invalid_input(OP, "The negotiation input is invalid"));
        }
        let req = self
            .http
            .post(self.endpoint(
                OP,
                &format!("mcp/account/listings/{listing}/offer-action-quote"),
            )?)
            .header(AUTHORIZATION, a.credential.header(OP)?)
            .json(p);
        let result: ApiEnvelope<NegotiationPreview> = self.send(OP, req, 12000).await?;
        result
            .data
            .filter(|v| {
                result.success && valid_digest(&v.terms_digest) && v.terms.matches(listing, p)
            })
            .ok_or_else(|| invalid_response(OP))
    }
    pub async fn submit_offer_change(
        &self,
        a: &DelegatedAccess,
        listing: &str,
        quote: &str,
        review: &str,
        p: &NegotiationChange,
    ) -> Result<NegotiationReceipt, ClientError> {
        const OP: &str = "submit_my_quoted_offer_change";
        if !is_listing_id(listing) || !is_action_id(quote) || !valid_digest(review) || !p.valid() {
            return Err(invalid_input(OP, "The reviewed offer is invalid"));
        }
        let (op, id, _, _) = p.parts();
        let req = self
            .action_request(
                OP,
                OFFERS_WRITE_SCOPE,
                a,
                Method::POST,
                &format!("mcp/account/listings/{listing}/offer-actions"),
                quote,
            )?
            .json(&serde_json::json!({"quote_id":quote,"terms_digest":review,"proposal":p}));
        self.direct_receipt(OP, req, |r: &NegotiationReceipt| {
            is_action_id(&r.action_id)
                && r.listing_id == listing
                && r.operation == op
                && r.offer_id.is_none_or(|i| i > 0)
                && r.order_id.is_none_or(|i| i > 0)
                && id.is_none_or(|id| Some(id) == r.offer_id)
                && if r.status == AccountActionState::Executed {
                    r.offer_id.is_some()
                        && r.order_id.is_some() == (op == NegotiationMove::AcceptOffer)
                } else {
                    r.offer_id == id && r.order_id.is_none()
                }
        })
        .await
    }
}
