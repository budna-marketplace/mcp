use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess, DraftMarket,
    actions::valid_outgoing_text, invalid_input, invalid_response, is_action_id,
    shipping::valid_revision,
};
use crate::{COLLECTION_ID_PATTERN, LISTING_ID_PATTERN, is_collection_id, is_listing_id};
use budna_mcp_core::write_permissions::{SHOWCASE_AUTHORING_READ_SCOPE, SHOWCASES_WRITE_SCOPE};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const ROUTE: &str = "mcp/account/selling/showcase-authoring";
#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthoringShowcaseKind {
    Theme,
    Sale,
    Auction,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthoringShowcaseStatus {
    Draft,
    Active,
    Ended,
    Cancelled,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ShowcaseListingRefusal {
    ShowcaseClosed,
    ShowcaseOver,
    NotOwnListing,
    OtherMarket,
    NotListable,
    Suspended,
    AlreadyMember,
    InAnotherShowcase,
    AuctionHasBids,
    PeriodTooLong,
    NotMember,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShowcaseChange {
    CreateDraft {
        kind: AuthoringShowcaseKind,
        #[schemars(length(min = 1, max = 120))]
        title: String,
        #[schemars(length(max = 5000))]
        description: String,
        market: DraftMarket,
        #[schemars(range(min = 1))]
        ends_at: i64,
        chat_enabled: bool,
    },
    EditDraft {
        #[schemars(regex(pattern=COLLECTION_ID_PATTERN))]
        showcase_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        kind: Option<AuthoringShowcaseKind>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(length(min = 1, max = 120))]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(length(max = 5000))]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(range(min = 1))]
        ends_at: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        chat_enabled: Option<bool>,
    },
    DeleteDraft {
        #[schemars(regex(pattern=COLLECTION_ID_PATTERN))]
        showcase_id: String,
    },
    AddListing {
        #[schemars(regex(pattern=COLLECTION_ID_PATTERN))]
        showcase_id: String,
        #[schemars(regex(pattern=LISTING_ID_PATTERN))]
        listing_id: String,
    },
    Publish {
        #[schemars(regex(pattern=COLLECTION_ID_PATTERN))]
        showcase_id: String,
    },
    Cancel {
        #[schemars(regex(pattern=COLLECTION_ID_PATTERN))]
        showcase_id: String,
    },
}
impl ShowcaseChange {
    fn id(&self) -> Option<&str> {
        match self {
            Self::CreateDraft { .. } => None,
            Self::EditDraft { showcase_id, .. }
            | Self::DeleteDraft { showcase_id }
            | Self::AddListing { showcase_id, .. }
            | Self::Publish { showcase_id }
            | Self::Cancel { showcase_id } => Some(showcase_id),
        }
    }
    fn action(&self) -> Option<&'static str> {
        if self.id().is_some_and(|id| !is_collection_id(id)) {
            return None;
        }
        let title_ok = |s: &str| s.trim() == s && valid_outgoing_text(s, 120, false);
        let desc_ok = |s: &str| s.is_empty() || s.trim() == s && valid_outgoing_text(s, 5000, true);
        match self {
            Self::CreateDraft {
                title,
                description,
                ends_at,
                ..
            } if title_ok(title) && desc_ok(description) && *ends_at > 0 => {
                Some("create_draft_showcase")
            }
            Self::EditDraft {
                title,
                description,
                ends_at,
                chat_enabled,
                kind,
                ..
            } if (title.is_some()
                || description.is_some()
                || ends_at.is_some()
                || chat_enabled.is_some()
                || kind.is_some())
                && title.as_deref().is_none_or(title_ok)
                && description.as_deref().is_none_or(desc_ok)
                && ends_at.is_none_or(|t| t > 0) =>
            {
                Some("edit_draft_showcase")
            }
            Self::DeleteDraft { .. } => Some("delete_draft_showcase"),
            Self::AddListing { listing_id, .. } if is_listing_id(listing_id) => {
                Some("add_showcase_listing")
            }
            Self::Publish { .. } => Some("publish_showcase"),
            Self::Cancel { .. } => Some("cancel_showcase"),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ShowcaseAuthoring {
    pub revision: String,
    pub seller_verified: bool,
    pub open_count: u64,
    pub max_open: u64,
    pub max_items: u64,
    pub showcase: Option<ShowcaseAuthoringDetails>,
    pub candidate: Option<ShowcaseCandidateTerms>,
}
#[derive(Debug, Deserialize)]
pub struct ShowcaseAuthoringDetails {
    pub showcase_id: String,
    pub kind: AuthoringShowcaseKind,
    pub status: AuthoringShowcaseStatus,
    pub market: DraftMarket,
    pub title: String,
    pub description: String,
    pub chat_enabled: bool,
    pub ends_at: i64,
    pub has_banner: bool,
    pub can_publish: bool,
    pub public_item_count: u64,
    pub scheduled_stream_count: u64,
    pub live_stream_count: u64,
    pub members: Vec<ShowcaseMemberTerms>,
}
#[derive(Debug, Deserialize)]
pub struct ShowcaseMemberTerms {
    pub listing_id: String,
    pub ends_at: i64,
    pub aligned: bool,
    pub has_bid: bool,
}
#[derive(Debug, Deserialize)]
pub struct ShowcaseCandidateTerms {
    pub listing_id: String,
    pub current_ends_at: i64,
    pub resulting_ends_at: Option<i64>,
    pub refusal: Option<ShowcaseListingRefusal>,
}
#[derive(Debug, Deserialize)]
pub struct ShowcaseChangeReceipt {
    pub action_id: String,
    pub action: String,
    pub showcase_id: Option<String>,
    pub status: AccountWatchlistActionStatus,
}

impl AccountApiClient {
    pub async fn showcase_authoring(
        &self,
        access: &DelegatedAccess,
        showcase: Option<&str>,
        listing: Option<&str>,
    ) -> Result<ShowcaseAuthoring, ClientError> {
        const OP: &str = "get_my_showcase_authoring";
        self.require_scope(access, SHOWCASE_AUTHORING_READ_SCOPE, OP)?;
        if showcase.is_some_and(|s| !is_collection_id(s))
            || listing.is_some_and(|l| showcase.is_none() || !is_listing_id(l))
        {
            return Err(invalid_input(
                OP,
                "Select a valid showcase before previewing a listing",
            ));
        }
        let query = [("showcase_id", showcase), ("listing_id", listing)]
            .into_iter()
            .filter_map(|(k, v)| v.map(|s| (k, s.to_owned())))
            .collect::<Vec<_>>();
        let out: ShowcaseAuthoring = self.get(OP, ROUTE, access, &query).await?;
        if !valid_revision(&out.revision)
            || out.showcase.as_ref().map(|s| s.showcase_id.as_str()) != showcase
            || out.candidate.as_ref().map(|l| l.listing_id.as_str()) != listing
        {
            return Err(invalid_response(OP));
        }
        if let Some(s) = &out.showcase {
            let mut ids = HashSet::new();
            if s.members.len() > 500
                || s.ends_at < 1
                || s.members.iter().any(|m| {
                    !is_listing_id(&m.listing_id) || !ids.insert(&m.listing_id) || m.ends_at < 0
                })
                || s.can_publish
                    && (!out.seller_verified
                        || !s.has_banner
                        || s.public_item_count == 0
                        || s.status != AuthoringShowcaseStatus::Draft)
            {
                return Err(invalid_response(OP));
            }
        }
        if out.candidate.as_ref().is_some_and(|c| {
            c.current_ends_at < 0
                || c.resulting_ends_at.is_some_and(|time| time < 0)
                || c.resulting_ends_at.is_some() == c.refusal.is_some()
        }) {
            return Err(invalid_response(OP));
        }
        Ok(out)
    }
    pub async fn change_showcase(
        &self,
        access: &DelegatedAccess,
        revision: &str,
        change: &ShowcaseChange,
        key: &str,
    ) -> Result<ShowcaseChangeReceipt, ClientError> {
        const OP: &str = "change_my_showcase";
        let action = change
            .action()
            .filter(|_| valid_revision(revision))
            .ok_or_else(|| invalid_input(OP, "Invalid showcase revision or change"))?;
        #[derive(Serialize)]
        struct Body<'a> {
            expected_revision: &'a str,
            #[serde(flatten)]
            change: &'a ShowcaseChange,
        }
        let request = self
            .action_request(OP, SHOWCASES_WRITE_SCOPE, access, Method::POST, ROUTE, key)?
            .json(&Body {
                expected_revision: revision,
                change,
            });
        self.direct_receipt(OP, request, |r: &ShowcaseChangeReceipt| {
            is_action_id(&r.action_id)
                && r.action == action
                && r.showcase_id.as_deref().is_none_or(is_collection_id)
                && (r.status != AccountWatchlistActionStatus::Executed || r.showcase_id.is_some())
                && change
                    .id()
                    .is_none_or(|id| r.showcase_id.as_deref().is_none_or(|s| s == id))
        })
        .await
    }
}
