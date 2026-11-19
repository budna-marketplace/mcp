use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess,
    ORDERS_READ_SCOPE, RATINGS_WRITE_SCOPE, actions::valid_outgoing_text, invalid_input,
    invalid_response, is_action_id, is_listing_id,
};
use crate::is_public_username;
use reqwest::Method;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAudience {
    Marketplace,
}
#[derive(Debug, Deserialize)]
pub struct OrderReviewFacts {
    pub order_id: i64,
    pub listing_id: String,
    pub counterparty_username: Option<String>,
    pub eligible: bool,
    pub audience: ReviewAudience,
}
#[derive(Debug, Deserialize)]
pub struct OrderReviewReceipt {
    pub order_id: i64,
    pub action_id: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub rating_id: Option<i64>,
    pub status: AccountWatchlistActionStatus,
}

impl AccountApiClient {
    pub async fn order_review_facts(
        &self,
        access: &DelegatedAccess,
        order: i64,
    ) -> Result<OrderReviewFacts, ClientError> {
        const OP: &str = "get_my_order_rating_eligibility";
        self.require_scope(access, ORDERS_READ_SCOPE, OP)?;
        if order < 1 || !self.write_permissions().ratings {
            return Err(invalid_input(OP, "Invalid order or ratings unavailable"));
        }
        let f: OrderReviewFacts = self
            .get(
                OP,
                &format!("mcp/account/orders/{order}/rating"),
                access,
                &[],
            )
            .await?;
        if f.order_id != order
            || !is_listing_id(&f.listing_id)
            || f.counterparty_username
                .as_deref()
                .is_some_and(|s| !is_public_username(s))
        {
            return Err(invalid_response(OP));
        }
        Ok(f)
    }
    pub async fn rate_order(
        &self,
        access: &DelegatedAccess,
        order: i64,
        stars: i32,
        comment: Option<&str>,
        audience: ReviewAudience,
        key: &str,
    ) -> Result<OrderReviewReceipt, ClientError> {
        const OP: &str = "rate_my_order_counterparty";
        if order < 1
            || !(1..=5).contains(&stars)
            || comment.is_some_and(|s| !valid_outgoing_text(s, 1000, true))
        {
            return Err(invalid_input(OP, "Invalid order, stars or comment"));
        }
        let request = self
            .action_request(
                OP,
                RATINGS_WRITE_SCOPE,
                access,
                Method::POST,
                &format!("mcp/account/orders/{order}/rating"),
                key,
            )?
            .json(&serde_json::json!({"stars":stars,"comment":comment,"audience":audience}));
        self.direct_receipt(OP, request, |r: &OrderReviewReceipt| {
            r.order_id == order
                && is_action_id(&r.action_id)
                && match r.rating_id {
                    Some(id) => id > 0,
                    None => r.status != AccountWatchlistActionStatus::Executed,
                }
        })
        .await
    }
}
