use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess, DraftReceipt,
    PUBLICATION_WRITE_SCOPE, SELLING_READ_SCOPE, invalid_input, invalid_response, is_action_id,
};
use crate::{Money, is_listing_id, money_is_valid};
use reqwest::Method;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PublicationBlocker {
    SellerVerificationRequired,
    OutstandingDebt,
    TermsRequired,
    PayoutSetupRequired,
    UpfrontPaymentRequired,
    FutureStartUnsupported,
    InvalidDraft,
    SellerLimitReached,
    MissingMedia,
    InvalidAttributes,
    PublicationUnavailable,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PublicationPriority {
    Standard,
    Extra,
    Max,
}
#[derive(Debug, Deserialize)]
pub struct PublicationReadiness {
    pub listing_id: String,
    pub revision: i64,
    pub ready: bool,
    pub blockers: Vec<PublicationBlocker>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub quote_id: Option<String>,
    pub starts_at: i64,
    pub ends_at: i64,
    pub media_count: u32,
    #[serde(deserialize_with = "Option::deserialize")]
    pub upfront_due: Option<Money>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub assumed_sale_price: Option<Money>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub estimated_sale_fees: Option<Money>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub estimated_seller_proceeds: Option<Money>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub priority_tier: Option<PublicationPriority>,
    pub priority_credit_expected: bool,
}
impl PublicationReadiness {
    fn financial_terms_valid(&self) -> bool {
        let upfront_ok = match &self.upfront_due {
            None => {
                !self.ready
                    && self
                        .blockers
                        .contains(&PublicationBlocker::UpfrontPaymentRequired)
            }
            Some(m) => {
                money_is_valid(m, "SEK")
                    && (!self.ready || crate::decimal_is_less_than_or_equal(&m.amount, "0"))
            }
        };
        upfront_ok
            && match (
                &self.assumed_sale_price,
                &self.estimated_sale_fees,
                &self.estimated_seller_proceeds,
            ) {
                (Some(price), Some(fees), Some(proceeds)) => {
                    money_is_valid(price, "SEK")
                        && money_is_valid(fees, "SEK")
                        && proceeds.currency_code == "SEK"
                        && crate::is_signed_exact_decimal(&proceeds.amount)
                }
                (None, None, None) => !self.ready,
                _ => false,
            }
    }
}
fn valid_quote(id: &str) -> bool {
    id.strip_prefix("quote_")
        .is_some_and(super::shipping::valid_revision)
}
impl AccountApiClient {
    pub async fn publication_readiness(
        &self,
        access: &DelegatedAccess,
        listing: &str,
    ) -> Result<PublicationReadiness, ClientError> {
        const OP: &str = "get_my_publication_readiness";
        self.require_scope(access, SELLING_READ_SCOPE, OP)?;
        if !self.write_permissions().publication || !is_listing_id(listing) {
            return Err(invalid_input(
                OP,
                "Invalid listing or publication unavailable",
            ));
        }
        let r: PublicationReadiness = self
            .get(
                OP,
                &format!("mcp/account/listings/{listing}/publication"),
                access,
                &[],
            )
            .await?;
        if r.listing_id != listing
            || r.revision <= 0
            || r.media_count > 30
            || r.starts_at < 0
            || r.ends_at < 0
            || r.blockers.len() > 11
            || r.blockers
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != r.blockers.len()
            || r.ready != r.blockers.is_empty()
            || r.ready != r.quote_id.is_some()
            || r.quote_id.as_deref().is_some_and(|id| !valid_quote(id))
            || !r.financial_terms_valid()
        {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }
    pub async fn publish_draft(
        &self,
        access: &DelegatedAccess,
        id: &str,
        revision: i64,
        quote: &str,
        key: &str,
    ) -> Result<DraftReceipt, ClientError> {
        const OP: &str = "publish_my_quoted_draft";
        if !is_listing_id(id) || revision <= 0 || !valid_quote(quote) {
            return Err(invalid_input(
                OP,
                "Invalid listing, revision or publication quote",
            ));
        }
        let request = self
            .action_request(
                OP,
                PUBLICATION_WRITE_SCOPE,
                access,
                Method::POST,
                &format!("mcp/account/listings/{id}/publication"),
                key,
            )?
            .json(&serde_json::json!({"expected_revision":revision,"quote_id":quote}));
        self.direct_receipt(OP, request, |r: &DraftReceipt| {
            is_action_id(&r.action_id)
                && r.listing_id.as_deref().is_none_or(|target| target == id)
                && (r.status != AccountWatchlistActionStatus::Executed || r.listing_id.is_some())
        })
        .await
    }
}
