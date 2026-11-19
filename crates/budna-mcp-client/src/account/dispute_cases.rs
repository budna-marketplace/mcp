use super::{actions::valid_outgoing_text, *};
use reqwest::Method;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseReason {
    ItemNotReceived,
    ItemNotAsDescribed,
    ItemDamaged,
    PaymentNotReceived,
    InappropriateRefundRequest,
    SellerUnresponsive,
    BuyerUnresponsive,
    Other,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseRemedy {
    Refund,
    Return,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseAudience {
    PartiesAndBudna,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseSide {
    Buyer,
    Seller,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum CaseCurrency {
    Sek,
    Nok,
    Eur,
}
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaseDiscount {
    #[schemars(
        length(min = 1, max = 16),
        regex(pattern = r"^(0|[1-9][0-9]*)(\.[0-9]{1,2})?$")
    )]
    pub amount: String,
    pub currency_code: CaseCurrency,
}
impl CaseDiscount {
    fn valid(&self) -> bool {
        let (whole, fraction) = self.amount.split_once('.').unwrap_or((&self.amount, ""));
        self.amount.len() <= 16
            && crate::is_exact_decimal(&self.amount)
            && (whole == "0" || !whole.starts_with('0'))
            && fraction.len() <= 2
            && !crate::decimal_is_less_than_or_equal(&self.amount, "0")
    }
}
#[derive(Debug, Deserialize)]
pub struct CaseOffer {
    pub offer_id: i64,
    pub amount: crate::Money,
    pub expires_at: i64,
}
#[derive(Debug, Deserialize)]
pub struct DisputeOptions {
    pub order_id: i64,
    pub listing_id: String,
    pub order_revision: i64,
    pub side: CaseSide,
    pub can_open: bool,
    pub payout_can_be_held: bool,
    pub current_dispute_id: Option<i64>,
    pub case_revision: Option<i64>,
    pub can_withdraw_claim: bool,
    pub can_propose: bool,
    pub can_accept: bool,
    pub can_decline: bool,
    pub can_withdraw_offer: bool,
    pub can_submit_statement: bool,
    pub current_offer: Option<CaseOffer>,
    pub maximum_discount: Option<crate::Money>,
}
#[derive(Debug, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum CaseChange {
    Open {
        expected_listing_id: String,
        expected_order_revision: i64,
        reason: CaseReason,
        title: String,
        description: String,
        requested_remedy: Option<CaseRemedy>,
        audience: CaseAudience,
    },
    Statement {
        dispute_id: i64,
        expected_case_revision: i64,
        body: String,
        audience: CaseAudience,
    },
    DeclineOffer {
        dispute_id: i64,
        offer_id: i64,
        expected_case_revision: i64,
    },
    WithdrawOffer {
        dispute_id: i64,
        offer_id: i64,
        expected_case_revision: i64,
    },
}
#[derive(Debug, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum CaseDecision {
    WithdrawClaim,
    Propose {
        amount: CaseDiscount,
        message: Option<String>,
    },
    Accept {
        offer_id: i64,
        amount: CaseDiscount,
        expires_at: i64,
    },
}
#[derive(Debug, Serialize)]
pub struct CaseTerms {
    pub order_id: i64,
    pub dispute_id: i64,
    pub expected_case_revision: i64,
    pub expected_order_revision: i64,
    pub decision: CaseDecision,
}
#[derive(Debug, Deserialize)]
pub struct CaseActionReceipt {
    pub action_id: String,
    pub order_id: i64,
    pub operation: String,
    pub status: AccountActionState,
    pub dispute_id: Option<i64>,
    pub statement_id: Option<i64>,
    pub offer_id: Option<i64>,
}
impl CaseActionReceipt {
    fn matches(&self, order: i64, verb: &str, case: Option<i64>, offer: Option<i64>) -> bool {
        self.order_id == order
            && self.operation == verb
            && is_action_id(&self.action_id)
            && [self.dispute_id, self.statement_id, self.offer_id]
                .into_iter()
                .flatten()
                .all(|id| id > 0)
            && (self.status != AccountActionState::Executed || self.dispute_id.is_some())
            && self
                .dispute_id
                .is_none_or(|id| case.is_none_or(|case| id == case))
            && self
                .offer_id
                .is_none_or(|id| offer.is_none_or(|offer| id == offer))
    }
}
fn valid_returned_amount(m: &crate::Money) -> bool {
    matches!(m.currency_code.as_str(), "SEK" | "NOK" | "EUR")
        && crate::money_is_valid(m, &m.currency_code)
        && m.amount.len() <= 16
        && m.amount.split_once('.').is_none_or(|(_, f)| f.len() <= 2)
        && !crate::decimal_is_less_than_or_equal(&m.amount, "0")
}
impl AccountApiClient {
    pub async fn case_options(
        &self,
        access: &DelegatedAccess,
        order: i64,
    ) -> Result<DisputeOptions, ClientError> {
        const OP: &str = "get_my_order_dispute_options";
        self.require_scope(access, ORDERS_READ_SCOPE, OP)?;
        if order <= 0
            || !(self.write_permissions().dispute_cases
                || self.write_permissions().dispute_settlements)
        {
            return Err(invalid_input(
                OP,
                "Invalid order or dispute options unavailable",
            ));
        }
        let r: DisputeOptions = self
            .get(
                OP,
                &format!("mcp/account/orders/{order}/dispute-options"),
                access,
                &[],
            )
            .await?;
        let open = r.current_dispute_id.is_some();
        let offer = r.current_offer.is_some();
        if r.order_id != order
            || !crate::is_listing_id(&r.listing_id)
            || r.order_revision < 0
            || (r.can_open && (open || r.order_revision == 0))
            || open != r.case_revision.is_some()
            || [r.case_revision, r.current_dispute_id]
                .into_iter()
                .flatten()
                .any(|id| id <= 0)
            || (!open
                && (offer
                    || r.can_withdraw_claim
                    || r.can_propose
                    || r.can_accept
                    || r.can_decline
                    || r.can_withdraw_offer
                    || r.can_submit_statement
                    || r.maximum_discount.is_some()))
            || ((r.can_accept || r.can_decline) && (!offer || r.side != CaseSide::Buyer))
            || (r.can_withdraw_offer && (!offer || r.side != CaseSide::Seller))
            || (r.can_propose && (r.side != CaseSide::Seller || r.maximum_discount.is_none()))
            || r.maximum_discount
                .as_ref()
                .is_some_and(|m| !valid_returned_amount(m))
            || r.current_offer.as_ref().is_some_and(|o| {
                o.offer_id <= 0 || o.expires_at <= 0 || !valid_returned_amount(&o.amount)
            })
        {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }
    pub async fn change_dispute(
        &self,
        access: &DelegatedAccess,
        order: i64,
        change: CaseChange,
        key: &str,
    ) -> Result<CaseActionReceipt, ClientError> {
        let (op, verb, scope, case, offer, valid) = match &change {
            CaseChange::Open {
                expected_listing_id,
                expected_order_revision,
                title,
                description,
                ..
            } => (
                "open_my_order_dispute",
                "open_order_dispute",
                DISPUTE_CASES_WRITE_SCOPE,
                None,
                None,
                crate::is_listing_id(expected_listing_id)
                    && *expected_order_revision > 0
                    && title.chars().count() >= 5
                    && valid_outgoing_text(title, 255, false)
                    && description.chars().count() >= 20
                    && valid_outgoing_text(description, 5000, true),
            ),
            CaseChange::Statement {
                dispute_id,
                expected_case_revision,
                body,
                ..
            } => (
                "add_my_dispute_statement",
                "submit_dispute_statement",
                DISPUTE_CASES_WRITE_SCOPE,
                Some(*dispute_id),
                None,
                *expected_case_revision > 0 && valid_outgoing_text(body, 2000, true),
            ),
            CaseChange::DeclineOffer {
                dispute_id,
                offer_id,
                expected_case_revision,
            } => (
                "decline_my_dispute_settlement",
                "decline_dispute_settlement",
                DISPUTE_SETTLEMENTS_WRITE_SCOPE,
                Some(*dispute_id),
                Some(*offer_id),
                *expected_case_revision > 0,
            ),
            CaseChange::WithdrawOffer {
                dispute_id,
                offer_id,
                expected_case_revision,
            } => (
                "withdraw_my_settlement_offer",
                "withdraw_dispute_settlement_offer",
                DISPUTE_SETTLEMENTS_WRITE_SCOPE,
                Some(*dispute_id),
                Some(*offer_id),
                *expected_case_revision > 0,
            ),
        };
        if !valid || order <= 0 || case.is_some_and(|i| i <= 0) || offer.is_some_and(|i| i <= 0) {
            return Err(invalid_input(
                op,
                "Invalid case, audience, revision or text",
            ));
        }
        let req = self
            .action_request(
                op,
                scope,
                access,
                Method::POST,
                &format!("mcp/account/orders/{order}/dispute-actions"),
                key,
            )?
            .json(&change);
        self.direct_receipt(op, req, |r: &CaseActionReceipt| {
            r.matches(order, verb, case, offer)
                && matches!(
                    r.status,
                    AccountActionState::Executed
                        | AccountActionState::Executing
                        | AccountActionState::Unknown
                )
                && (r.statement_id.is_none() || matches!(change, CaseChange::Statement { .. }))
                && (r.offer_id.is_none() || offer.is_some())
                && (r.status != AccountActionState::Executed
                    || (r.statement_id.is_some() == matches!(change, CaseChange::Statement { .. })
                        && r.offer_id == offer))
        })
        .await
    }
    pub async fn decide_dispute(
        &self,
        access: &DelegatedAccess,
        t: CaseTerms,
        key: &str,
    ) -> Result<CaseActionReceipt, ClientError> {
        let (op, verb, scope, offer, valid) = match &t.decision {
            CaseDecision::WithdrawClaim => (
                "request_withdraw_my_dispute",
                "withdraw_dispute_claim",
                DISPUTE_CASES_WRITE_SCOPE,
                None,
                true,
            ),
            CaseDecision::Propose { amount, message } => (
                "propose_my_dispute_settlement",
                "propose_dispute_settlement",
                DISPUTE_SETTLEMENTS_WRITE_SCOPE,
                None,
                amount.valid()
                    && message
                        .as_ref()
                        .is_none_or(|s| valid_outgoing_text(s, 1000, true)),
            ),
            CaseDecision::Accept {
                offer_id,
                amount,
                expires_at,
            } => (
                "accept_my_dispute_settlement",
                "accept_dispute_settlement",
                DISPUTE_SETTLEMENTS_WRITE_SCOPE,
                Some(*offer_id),
                *offer_id > 0 && *expires_at > 0 && amount.valid(),
            ),
        };
        if !valid
            || [
                t.order_id,
                t.dispute_id,
                t.expected_case_revision,
                t.expected_order_revision,
            ]
            .into_iter()
            .any(|i| i <= 0)
        {
            return Err(invalid_input(op, "Invalid reviewed dispute terms"));
        }
        let req = self
            .action_request(
                op,
                scope,
                access,
                Method::POST,
                &format!("mcp/account/orders/{}/dispute-decisions", t.order_id),
                key,
            )?
            .json(&t);
        let env: ApiEnvelope<CaseActionReceipt> = self
            .send(op, req, 8192)
            .await
            .map_err(ClientError::account_write)?;
        env.data
            .filter(|r| {
                env.success
                    && r.matches(t.order_id, verb, Some(t.dispute_id), offer)
                    && r.dispute_id == Some(t.dispute_id)
                    && r.statement_id.is_none()
                    && match t.decision {
                        CaseDecision::Accept { .. } => r.offer_id == offer,
                        CaseDecision::Propose { .. } => {
                            r.offer_id.is_some() == (r.status == AccountActionState::Executed)
                        }
                        CaseDecision::WithdrawClaim => r.offer_id.is_none(),
                    }
            })
            .ok_or_else(|| invalid_response(op))
    }
}
