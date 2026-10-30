use budna_mcp_client::{
    AccountBoost, AccountListingPromotion, AccountLiveSession, AccountOwnBoost, AccountPage,
    AccountSellerInsights, AccountShippingSettings, AccountShowcase,
};
use rmcp::schemars;
use serde::Serialize;

use super::{
    MAX_DISPLAY_TEXT_BYTES, ProjectionBudget, UNTRUSTED_CONTENT_NOTICE,
    activity::{AccountPageOutput, project},
    common::MoneyOutput,
};

const MAX_SERIES_DAYS: usize = 100;
const MAX_TOP_LISTINGS: usize = 10;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountSellerInsightsOutput {
    /// warming_up while history is still being collected, fresh, or stale when the latest refresh is late.
    #[schemars(length(max = 64))]
    pub status: String,
    pub coverage_started_at: Option<i64>,
    pub generated_at: Option<i64>,
    pub complete_through: Option<i64>,
    pub live: AccountEngagementLiveOutput,
    pub period: AccountEngagementPeriodOutput,
    #[schemars(length(max = 100))]
    pub series: Vec<AccountEngagementDayOutput>,
    #[schemars(length(max = 10))]
    pub top_listings: Vec<AccountTopListingOutput>,
    pub priority_credits: AccountPriorityCreditsOutput,
    pub truncated: bool,
    pub untrusted_content_notice: &'static str,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountEngagementLiveOutput {
    pub counted_views: i64,
    pub current_external_saves: i64,
    pub as_of: Option<i64>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountEngagementPeriodOutput {
    pub counted_views: i64,
    pub new_saves: i64,
    pub user_unsaves: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountEngagementDayOutput {
    /// UTC calendar day, YYYY-MM-DD.
    #[schemars(length(min = 10, max = 10))]
    pub date: String,
    pub counted_views: i64,
    pub new_saves: i64,
    pub user_unsaves: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountTopListingOutput {
    #[schemars(length(max = 64))]
    pub listing_id: String,
    #[schemars(length(max = 512))]
    pub title: String,
    pub counted_views: i64,
    pub new_saves: i64,
    pub current_external_saves: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountPriorityCreditsOutput {
    pub granted: i64,
    pub consumed: i64,
    pub available: i64,
    pub next_expiry_at: Option<i64>,
}

impl From<AccountSellerInsights> for AccountSellerInsightsOutput {
    fn from(insights: AccountSellerInsights) -> Self {
        let mut budget = ProjectionBudget::default();
        let top_listings = budget
            .objects(insights.top_listings, MAX_TOP_LISTINGS)
            .into_iter()
            .map(|listing| AccountTopListingOutput {
                listing_id: listing.listing_id,
                title: budget.text(listing.title, MAX_DISPLAY_TEXT_BYTES),
                counted_views: listing.counted_views,
                new_saves: listing.new_saves,
                current_external_saves: listing.current_external_saves,
            })
            .collect();
        let series = budget
            .objects(insights.series, MAX_SERIES_DAYS)
            .into_iter()
            .map(|day| AccountEngagementDayOutput {
                date: day.date,
                counted_views: day.counted_views,
                new_saves: day.new_saves,
                user_unsaves: day.user_unsaves,
            })
            .collect();
        let credits = insights.priority_credits;
        Self {
            status: insights.status,
            coverage_started_at: insights.coverage_started_at,
            generated_at: insights.generated_at,
            complete_through: insights.complete_through,
            live: AccountEngagementLiveOutput {
                counted_views: insights.live.counted_views,
                current_external_saves: insights.live.current_external_saves,
                as_of: insights.live.as_of,
            },
            period: AccountEngagementPeriodOutput {
                counted_views: insights.period.counted_views,
                new_saves: insights.period.new_saves,
                user_unsaves: insights.period.user_unsaves,
            },
            series,
            top_listings,
            priority_credits: AccountPriorityCreditsOutput {
                granted: credits.granted,
                consumed: credits.consumed,
                available: credits.available,
                next_expiry_at: credits.next_expiry_at,
            },
            truncated: budget.truncated(),
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountBoostOutput {
    /// standard or rocket; older boosts may say super.
    #[schemars(length(max = 64))]
    pub boost_type: String,
    #[schemars(length(max = 64))]
    pub status: String,
    pub price: MoneyOutput,
    pub starts_at: i64,
    pub expires_at: i64,
}

impl From<AccountBoost> for AccountBoostOutput {
    fn from(boost: AccountBoost) -> Self {
        Self {
            boost_type: boost.boost_type,
            status: boost.status,
            price: boost.price.into(),
            starts_at: boost.starts_at,
            expires_at: boost.expires_at,
        }
    }
}

/// A boost the account bought; exactly one of listing_id and showcase_id is set.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountOwnBoostOutput {
    #[schemars(length(max = 64))]
    pub listing_id: Option<String>,
    #[schemars(length(max = 64))]
    pub showcase_id: Option<String>,
    #[schemars(length(max = 512))]
    pub title: String,
    #[schemars(length(max = 64))]
    pub boost_type: String,
    #[schemars(length(max = 64))]
    pub status: String,
    pub price: MoneyOutput,
    pub starts_at: i64,
    pub expires_at: i64,
}

impl From<AccountPage<AccountOwnBoost>> for AccountPageOutput<AccountOwnBoostOutput> {
    fn from(page: AccountPage<AccountOwnBoost>) -> Self {
        project(page, |owned, budget| AccountOwnBoostOutput {
            listing_id: owned.listing_id,
            showcase_id: owned.showcase_id,
            title: budget.text(owned.title, MAX_DISPLAY_TEXT_BYTES),
            boost_type: owned.boost.boost_type,
            status: owned.boost.status,
            price: owned.boost.price.into(),
            starts_at: owned.boost.starts_at,
            expires_at: owned.boost.expires_at,
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountListingPromotionOutput {
    pub current_boost: Option<AccountBoostOutput>,
    #[schemars(length(max = 20))]
    pub queued_boosts: Vec<AccountBoostOutput>,
    #[schemars(length(max = 20))]
    pub boost_history: Vec<AccountBoostOutput>,
    pub search_priority: Option<AccountSearchPriorityOutput>,
    pub video: Option<AccountListingVideoOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountSearchPriorityOutput {
    /// standard, extra or max.
    #[schemars(length(max = 64))]
    pub tier: String,
    pub quoted_price: MoneyOutput,
    #[schemars(length(max = 64))]
    pub fee_schedule_version: String,
    pub selected_at: i64,
}

/// Processing state only; video links are never included.
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountListingVideoOutput {
    /// True once the listing's video choice is locked.
    pub frozen: bool,
    #[schemars(length(max = 50))]
    pub videos: Vec<AccountVideoOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountVideoOutput {
    #[schemars(length(max = 64))]
    pub status: String,
    pub selected: bool,
    pub pending: bool,
    pub duration_millis: Option<i64>,
    pub updated_at: i64,
}

impl From<AccountListingPromotion> for AccountListingPromotionOutput {
    fn from(promotion: AccountListingPromotion) -> Self {
        Self {
            current_boost: promotion.current_boost.map(Into::into),
            queued_boosts: promotion
                .queued_boosts
                .into_iter()
                .map(Into::into)
                .collect(),
            boost_history: promotion
                .boost_history
                .into_iter()
                .map(Into::into)
                .collect(),
            search_priority: promotion.search_priority.map(|priority| {
                AccountSearchPriorityOutput {
                    tier: priority.tier,
                    quoted_price: priority.quoted_price.into(),
                    fee_schedule_version: priority.fee_schedule_version,
                    selected_at: priority.selected_at,
                }
            }),
            video: promotion.video.map(|video| AccountListingVideoOutput {
                frozen: video.frozen,
                videos: video
                    .videos
                    .into_iter()
                    .map(|asset| AccountVideoOutput {
                        status: asset.status,
                        selected: asset.selected,
                        pending: asset.pending,
                        duration_millis: asset.duration_millis,
                        updated_at: asset.updated_at,
                    })
                    .collect(),
            }),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountShippingSettingsOutput {
    /// Default carriers; null when the seller has not chosen any.
    #[schemars(length(max = 10), inner(length(max = 64)))]
    pub provider_codes: Option<Vec<String>>,
    pub combined_enabled: Option<bool>,
    pub updated_at: Option<i64>,
}

impl From<AccountShippingSettings> for AccountShippingSettingsOutput {
    fn from(settings: AccountShippingSettings) -> Self {
        Self {
            provider_codes: settings.provider_codes,
            combined_enabled: settings.combined_enabled,
            updated_at: settings.updated_at,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountShowcaseOutput {
    #[schemars(length(max = 64))]
    pub id: String,
    #[schemars(length(max = 512))]
    pub title: String,
    #[schemars(length(max = 64))]
    pub kind: String,
    #[schemars(length(max = 64))]
    pub status: String,
    pub starts_at: Option<i64>,
    pub ends_at: i64,
    pub public_item_count: i64,
    pub is_boosted: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<AccountPage<AccountShowcase>> for AccountPageOutput<AccountShowcaseOutput> {
    fn from(page: AccountPage<AccountShowcase>) -> Self {
        project(page, |showcase, budget| AccountShowcaseOutput {
            id: showcase.id,
            title: budget.text(showcase.title, MAX_DISPLAY_TEXT_BYTES),
            kind: showcase.kind,
            status: showcase.status,
            starts_at: showcase.starts_at,
            ends_at: showcase.ends_at,
            public_item_count: showcase.public_item_count,
            is_boosted: showcase.is_boosted,
            created_at: showcase.created_at,
            updated_at: showcase.updated_at,
        })
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountLiveSessionOutput {
    #[schemars(length(max = 64))]
    pub id: String,
    #[schemars(length(max = 64))]
    pub status: String,
    #[schemars(length(max = 512))]
    pub title: Option<String>,
    pub scheduled_start_at: i64,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub viewer_peak: i32,
    pub reminder_count: i64,
    pub item_count: i64,
    pub sold: i64,
    pub revenue: MoneyOutput,
}

impl From<AccountPage<AccountLiveSession>> for AccountPageOutput<AccountLiveSessionOutput> {
    fn from(page: AccountPage<AccountLiveSession>) -> Self {
        project(page, |session, budget| AccountLiveSessionOutput {
            id: session.id,
            status: session.status,
            title: budget.optional_text(session.title, MAX_DISPLAY_TEXT_BYTES),
            scheduled_start_at: session.scheduled_start_at,
            started_at: session.started_at,
            ended_at: session.ended_at,
            viewer_peak: session.viewer_peak,
            reminder_count: session.reminder_count,
            item_count: session.item_count,
            sold: session.sold,
            revenue: session.revenue.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use budna_mcp_client::{
        AccountEngagementDay, AccountEngagementLive, AccountEngagementPeriod, AccountListingVideo,
        AccountPriorityCredits, AccountSearchPriority, AccountTopListing, AccountVideo, Money,
        Pagination,
    };

    use super::*;
    use crate::output::assert_within_mcp_result_budget;

    const LISTING: &str = "listing_00000000-0000-4000-8000-000000000007";

    fn sek(amount: &str) -> Money {
        Money {
            amount: amount.to_owned(),
            currency_code: "SEK".to_owned(),
        }
    }

    fn boost(status: &str) -> AccountBoost {
        AccountBoost {
            boost_type: "rocket".to_owned(),
            status: status.to_owned(),
            price: sek("49.00"),
            starts_at: 1,
            expires_at: 2,
        }
    }

    #[test]
    fn maximum_insights_stay_within_the_tool_result_budget() {
        let insights = AccountSellerInsights {
            status: "fresh".to_owned(),
            coverage_started_at: Some(1),
            generated_at: Some(2),
            complete_through: Some(3),
            live: AccountEngagementLive {
                counted_views: 10,
                current_external_saves: 2,
                as_of: Some(4),
            },
            period: AccountEngagementPeriod {
                counted_views: 100,
                new_saves: 5,
                user_unsaves: 1,
            },
            series: (0..120)
                .map(|_| AccountEngagementDay {
                    date: "2026-10-30".to_owned(),
                    counted_views: 1,
                    new_saves: 0,
                    user_unsaves: 0,
                })
                .collect(),
            top_listings: (0..12)
                .map(|_| AccountTopListing {
                    listing_id: LISTING.to_owned(),
                    title: "\u{0000}".repeat(4096),
                    counted_views: 9,
                    new_saves: 1,
                    current_external_saves: 1,
                })
                .collect(),
            priority_credits: AccountPriorityCredits {
                granted: 3,
                consumed: 1,
                available: 2,
                next_expiry_at: None,
            },
        };
        let output = AccountSellerInsightsOutput::from(insights);
        assert_eq!(output.series.len(), MAX_SERIES_DAYS);
        assert_eq!(output.top_listings.len(), MAX_TOP_LISTINGS);
        assert!(output.truncated);
        assert_within_mcp_result_budget(&output);
    }

    #[test]
    fn promotion_and_boost_pages_carry_state_without_links() {
        let promotion = AccountListingPromotion {
            current_boost: Some(boost("active")),
            queued_boosts: vec![boost("queued")],
            boost_history: (0..20).map(|_| boost("expired")).collect(),
            search_priority: Some(AccountSearchPriority {
                tier: "max".to_owned(),
                quoted_price: sek("29.00"),
                fee_schedule_version: "2026-10".to_owned(),
                selected_at: 1,
            }),
            video: Some(AccountListingVideo {
                frozen: true,
                videos: vec![AccountVideo {
                    status: "published".to_owned(),
                    selected: true,
                    pending: false,
                    duration_millis: Some(30_000),
                    updated_at: 1,
                }],
            }),
        };
        let value = serde_json::to_value(AccountListingPromotionOutput::from(promotion))
            .unwrap_or_else(|error| panic!("promotion should serialize: {error}"));
        assert_eq!(
            value["video"]["videos"][0]
                .as_object()
                .map(|video| video.keys().map(String::as_str).collect::<BTreeSet<_>>()),
            Some(BTreeSet::from([
                "duration_millis",
                "pending",
                "selected",
                "status",
                "updated_at"
            ]))
        );
        let page = AccountPage {
            items: (0..60)
                .map(|_| AccountOwnBoost {
                    listing_id: Some(LISTING.to_owned()),
                    showcase_id: None,
                    title: "t".repeat(4096),
                    boost: boost("active"),
                })
                .collect(),
            pagination: Pagination {
                page: 1,
                limit: 50,
                total: 60,
                total_pages: 2,
            },
        };
        let output = AccountPageOutput::<AccountOwnBoostOutput>::from(page);
        assert_eq!(output.items.len(), 50);
        assert_eq!(output.items[0].boost_type, "rocket");
        assert_within_mcp_result_budget(&output);
    }
}
