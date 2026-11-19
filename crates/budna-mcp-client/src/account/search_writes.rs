use std::collections::BTreeMap;

use reqwest::{Method, header::AUTHORIZATION};
use serde::{Deserialize, Serialize};

use super::{
    AccountApiClient, AccountWatchlistActionStatus, ApiEnvelope, ClientError, DelegatedAccess,
    SAVED_SEARCH_WRITE_SCOPE, WATCHLIST_READ_SCOPE, invalid_input, invalid_response, is_action_id,
};
use crate::is_exact_decimal;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct SearchPrice {
    pub amount: String,
    pub currency_code: String,
}

impl SearchPrice {
    fn hundredths(&self) -> Option<u64> {
        if !matches!(self.currency_code.as_str(), "SEK" | "NOK") || !is_exact_decimal(&self.amount)
        {
            return None;
        }
        let (whole, fraction) = self.amount.split_once('.').unwrap_or((&self.amount, ""));
        if whole.len() > 13 || fraction.len() > 2 {
            return None;
        }
        let main = whole.parse::<u64>().ok()?.checked_mul(100)?;
        let part = match fraction.len() {
            0 => 0,
            1 => fraction.parse::<u64>().ok()? * 10,
            _ => fraction.parse().ok()?,
        };
        main.checked_add(part).filter(|v| *v <= 100_000_000_000_000)
    }
}

/// Missing fields are preserved. An explicitly absent value clears one criterion on update.
#[derive(Debug, Default, Eq, PartialEq, Serialize)]
pub struct SearchCriteriaPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<Option<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_price: Option<Option<SearchPrice>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<Option<SearchPrice>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_type: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<Option<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_region: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_municipality: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_pickup: Option<Option<bool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_mode: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<Option<String>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub filters: BTreeMap<String, Option<String>>,
}

use super::actions::valid_action_text;

impl SearchCriteriaPatch {
    fn valid(&self, create: bool) -> bool {
        let has_clear = self.text == Some(None)
            || self.category_id == Some(None)
            || self.market == Some(None)
            || matches!(self.min_price, Some(None))
            || matches!(self.max_price, Some(None))
            || self.condition == Some(None)
            || self.listing_type == Some(None)
            || self.location_id == Some(None)
            || self.location_region == Some(None)
            || self.location_municipality == Some(None)
            || self.allow_pickup == Some(None)
            || self.search_mode == Some(None)
            || self.sort_by == Some(None)
            || self.sort_order == Some(None);
        if create && (has_clear || self.filters.values().any(Option::is_none)) {
            return false;
        }
        let minimum = self.min_price.as_ref().and_then(Option::as_ref);
        let maximum = self.max_price.as_ref().and_then(Option::as_ref);
        // On edit an omitted market comes from the stored search and is checked by Budna.
        let currency = match self.market.as_ref().map(Option::as_deref) {
            Some(Some("norwegian")) => Some("NOK"),
            Some(None | Some("swedish")) => Some("SEK"),
            None if create => Some("SEK"),
            _ => None,
        };
        if [minimum, maximum]
            .into_iter()
            .flatten()
            .any(|p| currency.is_some_and(|expected| p.currency_code != expected))
            || minimum
                .zip(maximum)
                .is_some_and(|(a, b)| a.currency_code != b.currency_code)
        {
            return false;
        }
        self.text
            .as_ref()
            .and_then(Option::as_ref)
            .is_none_or(|s| valid_action_text(s, 256))
            && self.category_id.flatten().is_none_or(|id| id > 0)
            && self.location_id.flatten().is_none_or(|id| id > 0)
            && self
                .market
                .as_ref()
                .and_then(Option::as_deref)
                .is_none_or(|s| matches!(s, "swedish" | "norwegian"))
            && self
                .condition
                .as_ref()
                .and_then(Option::as_deref)
                .is_none_or(|s| {
                    matches!(s, "new" | "like_new" | "very_good" | "good" | "acceptable")
                })
            && self
                .listing_type
                .as_ref()
                .and_then(Option::as_deref)
                .is_none_or(|s| {
                    matches!(
                        s,
                        "auction" | "fixed_price" | "auction_fixed_price" | "in_stock"
                    )
                })
            && self
                .search_mode
                .as_ref()
                .and_then(Option::as_deref)
                .is_none_or(|s| matches!(s, "hybrid" | "semantic" | "keyword"))
            && self
                .sort_order
                .as_ref()
                .and_then(Option::as_deref)
                .is_none_or(|s| matches!(s, "asc" | "desc"))
            && self
                .sort_by
                .as_ref()
                .and_then(Option::as_deref)
                .is_none_or(|s| {
                    matches!(
                        s,
                        "relevance" | "price" | "created_at" | "end_time" | "popularity"
                    ) || s
                        .strip_prefix("attr_")
                        .is_some_and(|field| valid_action_text(field, 64))
                })
            && minimum.is_none_or(|v| v.hundredths().is_some())
            && maximum.is_none_or(|v| v.hundredths().is_some())
            && minimum
                .zip(maximum)
                .is_none_or(|(a, b)| a.hundredths() <= b.hundredths())
            && [&self.location_region, &self.location_municipality]
                .iter()
                .all(|p| {
                    p.as_ref()
                        .and_then(Option::as_deref)
                        .is_none_or(|s| valid_action_text(s, 100))
                })
            && self.filters.len() <= 20
            && self.filters.iter().all(|(k, v)| {
                valid_action_text(k, 64) && v.as_deref().is_none_or(|s| valid_action_text(s, 256))
            })
    }
}

#[derive(Debug, Default, Serialize)]
pub struct SavedSearchChanges {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<SearchCriteriaPatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_frequency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

#[derive(Debug)]
pub enum SavedSearchChange {
    Create(SavedSearchChanges),
    Update {
        search_id: i64,
        expected_updated_at: i64,
        changes: SavedSearchChanges,
    },
    Delete {
        search_id: i64,
        expected_updated_at: i64,
    },
}

#[derive(Debug, Deserialize)]
pub struct SavedSearchReceipt {
    pub action_id: String,
    pub status: AccountWatchlistActionStatus,
    pub search_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SavedSearchAuthoring {
    pub limit: i64,
    pub used: i64,
    pub notification_frequencies: Vec<String>,
}

impl AccountApiClient {
    pub async fn saved_search_authoring(
        &self,
        access: &DelegatedAccess,
    ) -> Result<SavedSearchAuthoring, ClientError> {
        const OP: &str = "get_my_saved_search_authoring";
        self.require_scope(access, WATCHLIST_READ_SCOPE, OP)?;
        if !self.write_permissions().saved_searches {
            return Err(invalid_input(OP, "Saved-search authoring is unavailable"));
        }
        let result: SavedSearchAuthoring = self
            .get(OP, "mcp/account/saved-search-authoring", access, &[])
            .await?;
        let choices = result
            .notification_frequencies
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        if result.limit < 0
            || result.used < 0
            || result.notification_frequencies.len() != 3
            || choices != std::collections::BTreeSet::from(["immediate", "daily", "weekly"])
        {
            return Err(invalid_response(OP));
        }
        Ok(result)
    }

    pub async fn change_saved_search(
        &self,
        access: &DelegatedAccess,
        change: &SavedSearchChange,
        key: &str,
    ) -> Result<SavedSearchReceipt, ClientError> {
        let (op, method, id, revision, fields) = match change {
            SavedSearchChange::Create(fields) => (
                "create_my_saved_search",
                Method::POST,
                None,
                None,
                Some(fields),
            ),
            SavedSearchChange::Update {
                search_id,
                expected_updated_at,
                changes,
            } => (
                "update_my_saved_search",
                Method::PATCH,
                Some(*search_id),
                Some(*expected_updated_at),
                Some(changes),
            ),
            SavedSearchChange::Delete {
                search_id,
                expected_updated_at,
            } => (
                "delete_my_saved_search",
                Method::DELETE,
                Some(*search_id),
                Some(*expected_updated_at),
                None,
            ),
        };
        self.require_scope(access, SAVED_SEARCH_WRITE_SCOPE, op)?;
        if !is_action_id(key) || id.is_some_and(|n| n < 1) || revision.is_some_and(|n| n < 1) {
            return Err(invalid_input(op, "Invalid search, revision or request key"));
        }
        if let Some(fields) = fields {
            let create = id.is_none();
            if fields
                .name
                .as_deref()
                .is_some_and(|s| !valid_action_text(s, 255))
                || fields
                    .notification_frequency
                    .as_deref()
                    .is_some_and(|s| !matches!(s, "immediate" | "daily" | "weekly"))
                || fields.criteria.as_ref().is_some_and(|c| !c.valid(create))
                || (create
                    && (fields.name.is_none()
                        || fields.criteria.is_none()
                        || fields.active.is_some()))
                || (!create
                    && fields.name.is_none()
                    && fields.notification_frequency.is_none()
                    && fields.active.is_none()
                    && fields
                        .criteria
                        .as_ref()
                        .is_none_or(|c| c == &SearchCriteriaPatch::default()))
            {
                return Err(invalid_input(
                    op,
                    "Invalid saved-search settings or empty update",
                ));
            }
        }
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            expected_updated_at: Option<i64>,
            #[serde(flatten)]
            fields: Option<&'a SavedSearchChanges>,
        }
        let path = id.map_or_else(
            || "mcp/account/saved-searches".to_owned(),
            |id| format!("mcp/account/saved-searches/{id}"),
        );
        let request = self
            .http
            .request(method, self.endpoint(op, &path)?)
            .header(AUTHORIZATION, access.credential.header(op)?)
            .header("Idempotency-Key", key)
            .json(&Body {
                expected_updated_at: revision,
                fields,
            });
        let envelope: ApiEnvelope<SavedSearchReceipt> = self
            .send(op, request, 4096)
            .await
            .map_err(ClientError::account_write)?;
        envelope
            .data
            .filter(|r| {
                envelope.success
                    && is_action_id(&r.action_id)
                    && r.search_id.is_none_or(|n| n > 0)
                    && id.is_none_or(|n| r.search_id == Some(n))
                    && (r.status != AccountWatchlistActionStatus::Executed || r.search_id.is_some())
            })
            .ok_or(ClientError::AccountWrite {
                operation: op,
                rejection_status: None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_matches_the_selected_or_default_market_without_conversion() {
        let mut criteria = SearchCriteriaPatch {
            min_price: Some(Some(SearchPrice {
                amount: "15.90".into(),
                currency_code: "NOK".into(),
            })),
            ..Default::default()
        };
        assert!(!criteria.valid(true));
        assert!(criteria.valid(false)); // The API rechecks the stored market on a partial edit.
        criteria.market = Some(Some("norwegian".into()));
        assert!(criteria.valid(true));
        criteria.market = Some(Some("swedish".into()));
        assert!(!criteria.valid(false));
    }
}
