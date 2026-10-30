use serde::Deserialize;

use super::activity::{code, money, optional_text, optional_time};
use crate::{Money, is_listing_id, is_public_id};

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSellerInsights {
    pub status: String,
    pub coverage_started_at: Option<i64>,
    pub generated_at: Option<i64>,
    pub complete_through: Option<i64>,
    pub live: AccountEngagementLive,
    pub period: AccountEngagementPeriod,
    pub series: Vec<AccountEngagementDay>,
    pub top_listings: Vec<AccountTopListing>,
    pub priority_credits: AccountPriorityCredits,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountEngagementLive {
    pub counted_views: i64,
    pub current_external_saves: i64,
    pub as_of: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountEngagementPeriod {
    pub counted_views: i64,
    pub new_saves: i64,
    pub user_unsaves: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountEngagementDay {
    pub date: String,
    pub counted_views: i64,
    pub new_saves: i64,
    pub user_unsaves: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountTopListing {
    pub listing_id: String,
    pub title: String,
    pub counted_views: i64,
    pub new_saves: i64,
    pub current_external_saves: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountPriorityCredits {
    pub granted: i64,
    pub consumed: i64,
    pub available: i64,
    pub next_expiry_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountBoost {
    pub boost_type: String,
    pub status: String,
    pub price: Money,
    pub starts_at: i64,
    pub expires_at: i64,
}

/// A boost the account bought, for one listing or one collection.
#[derive(Clone, Debug, Deserialize)]
pub struct AccountOwnBoost {
    pub listing_id: Option<String>,
    pub showcase_id: Option<String>,
    pub title: String,
    #[serde(flatten)]
    pub boost: AccountBoost,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountListingPromotion {
    pub current_boost: Option<AccountBoost>,
    pub queued_boosts: Vec<AccountBoost>,
    pub boost_history: Vec<AccountBoost>,
    pub search_priority: Option<AccountSearchPriority>,
    pub video: Option<AccountListingVideo>,
    /// Only a company's listing carries a VAT treatment.
    pub vat: Option<AccountListingVat>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountListingVat {
    pub vat_registered: bool,
    pub vat_treatment: Option<String>,
    pub default_vat_treatment: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountSearchPriority {
    pub tier: String,
    pub quoted_price: Money,
    pub fee_schedule_version: String,
    pub selected_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountListingVideo {
    pub frozen: bool,
    pub videos: Vec<AccountVideo>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountVideo {
    pub status: String,
    pub selected: bool,
    pub pending: bool,
    pub duration_millis: Option<i64>,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountShippingSettings {
    pub provider_codes: Option<Vec<String>>,
    pub combined_enabled: Option<bool>,
    pub updated_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountShowcase {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub status: String,
    pub starts_at: Option<i64>,
    pub ends_at: i64,
    pub public_item_count: i64,
    pub is_boosted: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// An in-store tag's activity; the printed tag code stays private.
#[derive(Clone, Debug, Deserialize)]
pub struct AccountListingTag {
    pub listing_id: String,
    pub listing_title: Option<String>,
    pub scans: i64,
    pub scans_last_7_days: i64,
    pub last_scanned_on: Option<String>,
    pub saves: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountLiveSession {
    pub id: String,
    pub status: String,
    pub title: Option<String>,
    pub scheduled_start_at: i64,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub viewer_peak: i32,
    pub reminder_count: i64,
    pub item_count: i64,
    pub sold: i64,
    pub revenue: Money,
}

const MAX_TITLE_BYTES: usize = 4096;

impl AccountSellerInsights {
    pub(super) fn is_valid(&self) -> bool {
        code(&self.status)
            && optional_time(self.coverage_started_at)
            && optional_time(self.generated_at)
            && optional_time(self.complete_through)
            && self.live.counted_views >= 0
            && self.live.current_external_saves >= 0
            && optional_time(self.live.as_of)
            && [
                self.period.counted_views,
                self.period.new_saves,
                self.period.user_unsaves,
            ]
            .iter()
            .all(|count| *count >= 0)
            && self.series.len() <= 100
            && self.series.iter().all(|day| {
                is_calendar_day(&day.date)
                    && day.counted_views >= 0
                    && day.new_saves >= 0
                    && day.user_unsaves >= 0
            })
            && self.top_listings.len() <= 10
            && self.top_listings.iter().all(|listing| {
                is_listing_id(&listing.listing_id)
                    && listing.title.len() <= MAX_TITLE_BYTES
                    && listing.counted_views >= 0
                    && listing.new_saves >= 0
                    && listing.current_external_saves >= 0
            })
            && self.priority_credits.granted >= 0
            && self.priority_credits.consumed >= 0
            && self.priority_credits.available >= 0
            && optional_time(self.priority_credits.next_expiry_at)
    }
}

impl AccountBoost {
    fn is_valid(&self) -> bool {
        code(&self.boost_type)
            && code(&self.status)
            && money(&self.price)
            && self.starts_at >= 0
            && self.expires_at >= 0
    }
}

impl AccountOwnBoost {
    pub(super) fn is_valid(&self) -> bool {
        let target = match (&self.listing_id, &self.showcase_id) {
            (Some(listing_id), None) => is_listing_id(listing_id),
            (None, Some(showcase_id)) => is_public_id(showcase_id, "showcase_"),
            _ => false,
        };
        target && self.title.len() <= MAX_TITLE_BYTES && self.boost.is_valid()
    }
}

impl AccountListingPromotion {
    pub(super) fn is_valid(&self) -> bool {
        self.current_boost
            .as_ref()
            .is_none_or(AccountBoost::is_valid)
            && self.queued_boosts.len() <= 20
            && self.boost_history.len() <= 20
            && self
                .queued_boosts
                .iter()
                .chain(&self.boost_history)
                .all(AccountBoost::is_valid)
            && self.search_priority.as_ref().is_none_or(|priority| {
                code(&priority.tier)
                    && money(&priority.quoted_price)
                    && !priority.fee_schedule_version.is_empty()
                    && priority.fee_schedule_version.len() <= 64
                    && priority.selected_at >= 0
            })
            && self.video.as_ref().is_none_or(|video| {
                video.videos.len() <= 50
                    && video.videos.iter().all(|asset| {
                        code(&asset.status)
                            && optional_time(asset.duration_millis)
                            && asset.updated_at >= 0
                    })
            })
            && self.vat.as_ref().is_none_or(|vat| {
                vat.vat_treatment.as_deref().is_none_or(code)
                    && vat.default_vat_treatment.as_deref().is_none_or(code)
            })
    }
}

impl AccountShippingSettings {
    pub(super) fn is_valid(&self) -> bool {
        self.provider_codes
            .as_ref()
            .is_none_or(|codes| codes.len() <= 10 && codes.iter().all(|value| code(value)))
            && optional_time(self.updated_at)
    }
}

impl AccountShowcase {
    pub(super) fn is_valid(&self) -> bool {
        is_public_id(&self.id, "showcase_")
            && self.title.len() <= MAX_TITLE_BYTES
            && code(&self.kind)
            && code(&self.status)
            && optional_time(self.starts_at)
            && self.ends_at >= 0
            && self.public_item_count >= 0
            && self.created_at >= 0
            && self.updated_at >= 0
    }
}

impl AccountLiveSession {
    pub(super) fn is_valid(&self) -> bool {
        is_public_id(&self.id, "live_")
            && code(&self.status)
            && optional_text(self.title.as_deref(), MAX_TITLE_BYTES)
            && self.scheduled_start_at >= 0
            && optional_time(self.started_at)
            && optional_time(self.ended_at)
            && self.viewer_peak >= 0
            && self.reminder_count >= 0
            && self.item_count >= 0
            && self.sold >= 0
            && money(&self.revenue)
    }
}

impl AccountListingTag {
    pub(super) fn is_valid(&self) -> bool {
        is_listing_id(&self.listing_id)
            && optional_text(self.listing_title.as_deref(), MAX_TITLE_BYTES)
            && self.scans >= 0
            && self.scans_last_7_days >= 0
            && self.last_scanned_on.as_deref().is_none_or(is_calendar_day)
            && self.saves >= 0
    }
}

/// `YYYY-MM-DD`, as the engagement series labels its UTC days.
fn is_calendar_day(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes.iter().enumerate().all(|(index, byte)| {
            if matches!(index, 4 | 7) {
                *byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        })
}
