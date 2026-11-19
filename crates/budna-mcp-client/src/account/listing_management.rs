use super::{actions::valid_outgoing_text, draft_fields::supplied, publication::valid_quote, *};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ListingChangeKind {
    EditContent,
    EditPrices,
    Extend,
    Relist,
    Delete,
}
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ListingManagementChange {
    EditContent {
        #[serde(
            default,
            deserialize_with = "supplied",
            skip_serializing_if = "Option::is_none"
        )]
        #[schemars(length(min = 1, max = 200))]
        title: Option<String>,
        #[serde(
            default,
            deserialize_with = "supplied",
            skip_serializing_if = "Option::is_none"
        )]
        #[schemars(length(min = 1, max = 10000))]
        description: Option<String>,
        #[serde(
            default,
            deserialize_with = "supplied",
            skip_serializing_if = "Option::is_none"
        )]
        condition: Option<DraftCondition>,
    },
    EditPrices {
        #[serde(
            default,
            deserialize_with = "supplied",
            skip_serializing_if = "Option::is_none"
        )]
        starting_price: Option<ListingPrice>,
        #[serde(
            default,
            deserialize_with = "supplied",
            skip_serializing_if = "Option::is_none"
        )]
        seller_reserve: Option<ListingPrice>,
        #[serde(
            default,
            deserialize_with = "supplied",
            skip_serializing_if = "Option::is_none"
        )]
        buy_now_price: Option<ListingPrice>,
        #[serde(
            default,
            deserialize_with = "supplied",
            skip_serializing_if = "Option::is_none"
        )]
        bid_increment: Option<ListingPrice>,
    },
    Extend {
        #[schemars(range(min = 1, max = 14))]
        days: u8,
    },
    Relist {
        #[schemars(range(min = 1, max = 14))]
        days: u8,
    },
    Delete {},
}
impl ListingManagementChange {
    fn kind(&self) -> ListingChangeKind {
        match self {
            Self::EditContent { .. } => ListingChangeKind::EditContent,
            Self::EditPrices { .. } => ListingChangeKind::EditPrices,
            Self::Extend { .. } => ListingChangeKind::Extend,
            Self::Relist { .. } => ListingChangeKind::Relist,
            Self::Delete {} => ListingChangeKind::Delete,
        }
    }
    fn valid(&self) -> bool {
        match self {
            Self::EditContent {
                title,
                description,
                condition,
            } => {
                (title.is_some() || description.is_some() || condition.is_some())
                    && title
                        .as_ref()
                        .is_none_or(|s| valid_outgoing_text(s, 200, false))
                    && description
                        .as_ref()
                        .is_none_or(|s| valid_outgoing_text(s, 10000, true))
            }
            Self::EditPrices {
                starting_price,
                seller_reserve,
                buy_now_price,
                bid_increment,
            } => {
                let prices = [starting_price, seller_reserve, buy_now_price, bid_increment];
                prices.iter().any(|p| p.is_some())
                    && prices.into_iter().flatten().all(ListingPrice::valid)
                    && bid_increment
                        .as_ref()
                        .is_none_or(|p| !crate::decimal_is_less_than_or_equal(&p.amount, "0"))
            }
            Self::Extend { days } | Self::Relist { days } => (1..=14).contains(days),
            Self::Delete {} => true,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ListingManagementBlocker {
    UnsupportedCurrency,
    StreamManaged,
    SecondChancePending,
    SellerVerificationRequired,
    OutstandingDebt,
    TermsRequired,
    PayoutSetupRequired,
    UpfrontPaymentRequired,
    SellerLimitReached,
    IneligibleChange,
    FeeUnavailable,
}
#[derive(Debug, Deserialize)]
pub struct ListingManagementReview {
    pub listing_id: String,
    pub revision: i64,
    pub kind: ListingChangeKind,
    pub ready: bool,
    pub blockers: Vec<ListingManagementBlocker>,
    pub quote_id: Option<String>,
    pub starts_at: Option<i64>,
    pub ends_at: Option<i64>,
    pub additional_sale_fee: Option<crate::Money>,
    pub notifies_watchers: bool,
}
impl AccountApiClient {
    pub async fn prepare_listing_change(
        &self,
        access: &DelegatedAccess,
        listing: &str,
        change: &ListingManagementChange,
    ) -> Result<ListingManagementReview, ClientError> {
        const OP: &str = "prepare_my_listing_change";
        self.require_scope(access, SELLING_READ_SCOPE, OP)?;
        if !self.write_permissions().listing_management
            || !crate::is_listing_id(listing)
            || !change.valid()
        {
            return Err(invalid_input(OP, "Invalid listing change"));
        }
        let req = self
            .http
            .post(self.endpoint(
                OP,
                &format!("mcp/account/listings/{listing}/management-quote"),
            )?)
            .header(
                reqwest::header::AUTHORIZATION,
                access.credential.header(OP)?,
            )
            .json(&serde_json::json!({"operation":change}));
        let envelope: ApiEnvelope<ListingManagementReview> = self.send(OP, req, 8192).await?;
        let r = envelope.data.ok_or_else(|| invalid_response(OP))?;
        if !envelope.success
            || r.listing_id != listing
            || r.revision <= 0
            || r.kind != change.kind()
            || r.ready != r.blockers.is_empty()
            || r.ready != r.quote_id.is_some()
            || r.quote_id.as_deref().is_some_and(|s| !valid_quote(s))
            || r.notifies_watchers != (r.kind == ListingChangeKind::Relist)
            || r.blockers
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                != r.blockers.len()
            || r.additional_sale_fee
                .as_ref()
                .is_some_and(|m| !crate::money_is_valid(m, "SEK"))
            || if r.kind == ListingChangeKind::Relist {
                r.starts_at.is_some() || r.ends_at.is_some()
            } else {
                !matches!((r.starts_at,r.ends_at),(Some(a),Some(b)) if a>0&&b>=a)
            }
        {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }
    pub async fn apply_listing_change(
        &self,
        access: &DelegatedAccess,
        listing: &str,
        change: &ListingManagementChange,
        revision: i64,
        quote: &str,
        key: &str,
    ) -> Result<DraftReceipt, ClientError> {
        const OP: &str = "apply_my_quoted_listing_change";
        if !crate::is_listing_id(listing) || !valid_quote(quote) || revision <= 0 || !change.valid()
        {
            return Err(invalid_input(
                OP,
                "Invalid listing, change or reviewed quote",
            ));
        }
        let req=self.action_request(OP,LISTING_MANAGEMENT_WRITE_SCOPE,access,Method::POST,&format!("mcp/account/listings/{listing}/management"),key)?.json(&serde_json::json!({"operation":change,"expected_revision":revision,"quote_id":quote}));
        self.direct_receipt(OP, req, |r: &DraftReceipt| {
            is_action_id(&r.action_id)
                && r.listing_id.as_deref().is_none_or(|id| id == listing)
                && (r.status != AccountWatchlistActionStatus::Executed || r.listing_id.is_some())
        })
        .await
    }
}
