use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess,
    actions::valid_outgoing_text, invalid_input, invalid_response, is_action_id,
    shipping::valid_revision,
};
use crate::{
    COLLECTION_ID_PATTERN, LISTING_ID_PATTERN, LIVE_STREAM_ID_PATTERN, is_collection_id,
    is_listing_id, is_live_stream_id,
};
use budna_mcp_core::write_permissions::{STREAM_AUTHORING_READ_SCOPE, STREAM_PLANS_WRITE_SCOPE};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const ROUTE: &str = "mcp/account/selling/stream-authoring";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum StreamPlanTarget {
    Shop {},
    Listing {
        #[schemars(regex(pattern=LISTING_ID_PATTERN))]
        listing_id: String,
    },
    Showcase {
        #[schemars(regex(pattern=COLLECTION_ID_PATTERN))]
        showcase_id: String,
    },
}
impl StreamPlanTarget {
    fn valid(&self) -> bool {
        match self {
            Self::Shop {} => true,
            Self::Listing { listing_id } => is_listing_id(listing_id),
            Self::Showcase { showcase_id } => is_collection_id(showcase_id),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum StreamPlanChange {
    Schedule {
        target: StreamPlanTarget,
        #[schemars(range(min = 1))]
        scheduled_start_at: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(length(max = 120))]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(length(max = 1000))]
        description: Option<String>,
        notify_followers: bool,
    },
    Reschedule {
        #[schemars(regex(pattern=LIVE_STREAM_ID_PATTERN))]
        stream_id: String,
        #[schemars(range(min = 1))]
        scheduled_start_at: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(length(max = 120))]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(length(max = 1000))]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        notify_followers: Option<bool>,
    },
    Cancel {
        #[schemars(regex(pattern=LIVE_STREAM_ID_PATTERN))]
        stream_id: String,
    },
    SetAnnounced {
        #[schemars(regex(pattern=LIVE_STREAM_ID_PATTERN))]
        stream_id: String,
        #[schemars(regex(pattern=LISTING_ID_PATTERN))]
        listing_id: String,
        announced: bool,
    },
    MoveAnnounced {
        #[schemars(regex(pattern=LIVE_STREAM_ID_PATTERN))]
        stream_id: String,
        #[schemars(regex(pattern=LISTING_ID_PATTERN))]
        listing_id: String,
        /// Omit to move the selected listing to the end.
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(regex(pattern=LISTING_ID_PATTERN))]
        before_listing_id: Option<String>,
    },
}
impl StreamPlanChange {
    fn stream(&self) -> Option<&str> {
        match self {
            Self::Schedule { .. } => None,
            Self::Reschedule { stream_id, .. }
            | Self::Cancel { stream_id }
            | Self::SetAnnounced { stream_id, .. }
            | Self::MoveAnnounced { stream_id, .. } => Some(stream_id),
        }
    }
    fn action(&self) -> Option<&'static str> {
        let text_ok = |s: &str, max, lines| {
            s.trim() == s && (s.is_empty() || valid_outgoing_text(s, max, lines))
        };
        if self.stream().is_some_and(|s| !is_live_stream_id(s)) {
            return None;
        }
        match self {
            Self::Schedule {
                target,
                scheduled_start_at,
                title,
                description,
                ..
            } if target.valid()
                && *scheduled_start_at > 0
                && title.as_ref().is_none_or(|t| text_ok(t, 120, false))
                && description.as_ref().is_none_or(|t| text_ok(t, 1000, true))
                && (matches!(target, StreamPlanTarget::Shop {})
                    && title.as_ref().is_some_and(|t| !t.is_empty())
                    || !matches!(target, StreamPlanTarget::Shop {}) && description.is_none()) =>
            {
                Some("schedule_seller_stream")
            }
            Self::Reschedule {
                scheduled_start_at,
                title,
                description,
                ..
            } if *scheduled_start_at > 0
                && title.as_ref().is_none_or(|t| text_ok(t, 120, false))
                && description.as_ref().is_none_or(|t| text_ok(t, 1000, true)) =>
            {
                Some("reschedule_seller_stream")
            }
            Self::Cancel { .. } => Some("cancel_seller_stream"),
            Self::SetAnnounced { listing_id, .. } if is_listing_id(listing_id) => {
                Some("change_stream_lineup")
            }
            Self::MoveAnnounced {
                listing_id,
                before_listing_id,
                ..
            } if is_listing_id(listing_id)
                && before_listing_id
                    .as_deref()
                    .is_none_or(|before| is_listing_id(before) && before != listing_id) =>
            {
                Some("change_stream_lineup")
            }
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlannedStreamStatus {
    Draft,
    Scheduled,
    Live,
    Ended,
    Cancelled,
}
#[derive(Debug, Deserialize)]
pub struct StreamAuthoring {
    pub revision: String,
    pub target: StreamPlanTarget,
    pub seller_verified: bool,
    pub seller_ready: bool,
    pub target_allows_scheduling: bool,
    pub target_ends_at: Option<i64>,
    pub session: Option<StreamPlanDetails>,
    pub max_advance_days: u8,
    pub max_lineup: usize,
}
#[derive(Debug, Deserialize)]
pub struct StreamPlanDetails {
    pub stream_id: String,
    pub status: PlannedStreamStatus,
    pub scheduled_start_at: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub notify_followers: bool,
    pub listing_ids: Vec<String>,
}
#[derive(Debug, Deserialize)]
pub struct StreamPlanReceipt {
    pub action_id: String,
    pub action: String,
    pub stream_id: Option<String>,
    pub status: AccountWatchlistActionStatus,
}

impl AccountApiClient {
    pub async fn stream_authoring(
        &self,
        access: &DelegatedAccess,
        target: Option<&StreamPlanTarget>,
        stream: Option<&str>,
    ) -> Result<StreamAuthoring, ClientError> {
        const OP: &str = "get_my_stream_authoring";
        self.require_scope(access, STREAM_AUTHORING_READ_SCOPE, OP)?;
        let (key, value) = match (target, stream) {
            (Some(t), None) if t.valid() => match t {
                StreamPlanTarget::Shop {} => ("shop", "true"),
                StreamPlanTarget::Listing { listing_id } => ("listing_id", listing_id.as_str()),
                StreamPlanTarget::Showcase { showcase_id } => ("showcase_id", showcase_id.as_str()),
            },
            (None, Some(s)) if is_live_stream_id(s) => ("stream_id", s),
            _ => return Err(invalid_input(OP, "Select one owned stream or target")),
        };
        let out: StreamAuthoring = self
            .get(OP, ROUTE, access, &[(key, value.to_owned())])
            .await?;
        if !valid_revision(&out.revision)
            || !out.target.valid()
            || target.is_some_and(|t| t != &out.target)
            || out.session.as_ref().map(|s| s.stream_id.as_str()) != stream
            || out.max_advance_days != 30
            || out.max_lineup != 200
            || out.target_ends_at.is_some_and(|t| t <= 0)
            || out.target_ends_at.is_none() != matches!(out.target, StreamPlanTarget::Shop {})
        {
            return Err(invalid_response(OP));
        }
        if let Some(s) = &out.session {
            let mut ids = HashSet::new();
            if s.listing_ids.len() > out.max_lineup
                || s.listing_ids
                    .iter()
                    .any(|id| !is_listing_id(id) || !ids.insert(id))
            {
                return Err(invalid_response(OP));
            }
        }
        Ok(out)
    }
    pub async fn change_stream_plan(
        &self,
        access: &DelegatedAccess,
        revision: &str,
        change: &StreamPlanChange,
        key: &str,
    ) -> Result<StreamPlanReceipt, ClientError> {
        const OP: &str = "change_my_stream_plan";
        let action = change
            .action()
            .filter(|_| valid_revision(revision))
            .ok_or_else(|| invalid_input(OP, "Invalid stream revision or change"))?;
        #[derive(Serialize)]
        struct Body<'a> {
            expected_revision: &'a str,
            #[serde(flatten)]
            change: &'a StreamPlanChange,
        }
        let request = self
            .action_request(
                OP,
                STREAM_PLANS_WRITE_SCOPE,
                access,
                Method::POST,
                ROUTE,
                key,
            )?
            .json(&Body {
                expected_revision: revision,
                change,
            });
        self.direct_receipt(OP, request, |r: &StreamPlanReceipt| {
            is_action_id(&r.action_id)
                && r.action == action
                && r.stream_id.as_deref().is_none_or(is_live_stream_id)
                && (r.status != AccountWatchlistActionStatus::Executed || r.stream_id.is_some())
                && change
                    .stream()
                    .is_none_or(|s| r.stream_id.as_deref().is_none_or(|id| id == s))
        })
        .await
    }
}
