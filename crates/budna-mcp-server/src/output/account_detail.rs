use budna_mcp_client::{AccountListingAttributeValue, AccountListingDetail, LISTING_ID_PATTERN};
use rmcp::schemars;
use serde::Serialize;

use super::{
    ProjectionBudget, UNTRUSTED_CONTENT_NOTICE, common::MoneyOutput, listings::PublicLocationOutput,
};

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountListingDetailOutput {
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub id: String,
    #[schemars(length(max = 4096))]
    pub title: Option<String>,
    #[schemars(length(max = 40000))]
    pub description: Option<String>,
    pub category_id: Option<i32>,
    #[schemars(length(max = 32))]
    pub condition: String,
    #[schemars(length(max = 32))]
    pub listing_type: String,
    #[schemars(length(min = 3, max = 3), regex(pattern = "^[A-Z]{3}$"))]
    pub currency: String,
    #[schemars(length(max = 32))]
    pub market: String,
    #[schemars(length(max = 16))]
    pub status: String,
    pub quantity: i32,
    pub auction: Option<AccountAuctionOutput>,
    pub fixed_price: Option<AccountFixedPriceOutput>,
    #[schemars(length(max = 50))]
    pub images: Vec<AccountImageOutput>,
    pub shipping: AccountShippingOutput,
    pub buyer_protection: Option<AccountBuyerProtectionOutput>,
    pub views_count: i32,
    pub approved: bool,
    pub featured: bool,
    pub is_boosted: bool,
    pub has_search_priority: bool,
    #[schemars(length(max = 50), inner(length(max = 512)))]
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    #[schemars(length(max = 100))]
    pub attributes: Vec<AccountAttributeOutput>,
    pub showcase: Option<AccountShowcaseOutput>,
    pub live: Option<AccountLiveOutput>,
    pub live_stream: Option<AccountLiveStreamOutput>,
    #[schemars(length(max = 41))]
    pub live_session_id: Option<String>,
    pub truncated: bool,
    pub untrusted_content_notice: &'static str,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountAuctionOutput {
    pub round: Option<AccountRoundOutput>,
    pub start_time: i64,
    pub end_time: i64,
    pub starting_price: MoneyOutput,
    pub buy_now_price: Option<MoneyOutput>,
    pub current_bid: Option<MoneyOutput>,
    pub bid_count: Option<i64>,
    pub bid_increment: Option<MoneyOutput>,
    pub next_min_bid: Option<MoneyOutput>,
    pub reserve: AccountReserveOutput,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountRoundOutput {
    pub number: i32,
    #[schemars(length(max = 16))]
    pub status: String,
    #[schemars(length(max = 32))]
    pub outcome: Option<String>,
    pub closed_at: Option<i64>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountReserveOutput {
    pub exists: bool,
    pub met: bool,
    pub price: Option<MoneyOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountFixedPriceOutput {
    pub price: MoneyOutput,
    pub buy_now_price: Option<MoneyOutput>,
    pub start_time: i64,
    pub end_time: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountImageOutput {
    #[schemars(length(min = 36, max = 36))]
    pub id: String,
    #[schemars(length(max = 256))]
    pub thumb_hash: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountShippingOutput {
    pub custom_price: Option<MoneyOutput>,
    #[schemars(length(max = 64))]
    pub package_size: Option<String>,
    pub package_weight_grams: Option<i32>,
    #[schemars(length(max = 20), inner(length(max = 32)))]
    pub provider_codes: Option<Vec<String>>,
    pub combined_enabled: bool,
    pub location_id: Option<i32>,
    pub location: Option<PublicLocationOutput>,
    pub allow_pickup: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountBuyerProtectionOutput {
    #[schemars(length(max = 128))]
    pub rate_percent: String,
    pub flat_fee: MoneyOutput,
    pub percentage_threshold: MoneyOutput,
    pub mandatory_threshold: MoneyOutput,
    pub cap: MoneyOutput,
    pub enabled: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountAttributeOutput {
    pub filter_definition_id: i32,
    #[schemars(length(max = 256))]
    pub filter_name: String,
    #[schemars(length(max = 512))]
    pub label: String,
    pub value: AccountAttributeValueOutput,
    #[schemars(length(max = 4096))]
    pub display_value: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum AccountAttributeValueOutput {
    String(#[schemars(length(max = 4096))] String),
    Numeric(#[schemars(length(max = 128))] String),
    Boolean(bool),
    Date(i64),
    JsonDisplayOnly,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountShowcaseOutput {
    #[schemars(length(max = 45))]
    pub id: String,
    #[schemars(length(max = 4096))]
    pub title: String,
    pub ends_at: i64,
    pub is_live: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountLiveOutput {
    #[schemars(length(max = 41))]
    pub id: String,
    #[schemars(length(max = 16))]
    pub status: String,
    #[schemars(length(max = 4096))]
    pub title: Option<String>,
    #[schemars(length(max = 40000))]
    pub description: Option<String>,
    pub scheduled_start_at: i64,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub viewer_count: i64,
    pub viewer_peak: i32,
    pub reminder_count: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountLiveStreamOutput {
    #[schemars(length(max = 41))]
    pub session_id: String,
    #[schemars(length(max = 16))]
    pub status: String,
    #[schemars(length(max = 4096))]
    pub title: Option<String>,
    pub scheduled_start_at: i64,
    pub on_stage: bool,
    pub round: Option<AccountLiveRoundOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AccountLiveRoundOutput {
    #[schemars(length(max = 16))]
    pub state: String,
    #[schemars(length(max = 16))]
    pub format: String,
    pub duration_seconds: i32,
    pub extend_seconds: Option<i32>,
    pub start_price: MoneyOutput,
    pub opened_at: i64,
    pub ends_at: i64,
    pub closed_at: Option<i64>,
    pub final_price: Option<MoneyOutput>,
}

impl From<AccountListingDetail> for AccountListingDetailOutput {
    fn from(listing: AccountListingDetail) -> Self {
        let mut budget = ProjectionBudget::default();
        let title = budget.optional_text(listing.title, 4096);
        let description = budget.optional_text(listing.description, 40000);
        let images = budget
            .objects(listing.images, 50)
            .into_iter()
            .map(|image| AccountImageOutput {
                id: image.id,
                thumb_hash: budget.optional_text(image.thumb_hash, 256),
            })
            .collect();
        let tags = budget.strings(listing.tags, 50, 512);
        let attributes = budget
            .objects(listing.attributes, 100)
            .into_iter()
            .map(|attribute| AccountAttributeOutput {
                filter_definition_id: attribute.filter_definition_id,
                filter_name: budget.text(attribute.filter_name, 256),
                label: budget.text(attribute.label, 512),
                value: match attribute.value {
                    AccountListingAttributeValue::String(value) => {
                        AccountAttributeValueOutput::String(budget.text(value, 4096))
                    }
                    AccountListingAttributeValue::Numeric(value) => {
                        AccountAttributeValueOutput::Numeric(value)
                    }
                    AccountListingAttributeValue::Boolean(value) => {
                        AccountAttributeValueOutput::Boolean(value)
                    }
                    AccountListingAttributeValue::Date(value) => {
                        AccountAttributeValueOutput::Date(value)
                    }
                    AccountListingAttributeValue::JsonDisplayOnly => {
                        AccountAttributeValueOutput::JsonDisplayOnly
                    }
                },
                display_value: budget.optional_text(attribute.display_value, 4096),
                created_at: attribute.created_at,
                updated_at: attribute.updated_at,
            })
            .collect();
        let showcase = listing.showcase.map(|showcase| AccountShowcaseOutput {
            id: showcase.id,
            title: budget.text(showcase.title, 4096),
            ends_at: showcase.ends_at,
            is_live: showcase.is_live,
        });
        let live = listing.live.map(|live| AccountLiveOutput {
            id: live.id,
            status: live.status,
            title: budget.optional_text(live.title, 4096),
            description: budget.optional_text(live.description, 40000),
            scheduled_start_at: live.scheduled_start_at,
            started_at: live.started_at,
            ended_at: live.ended_at,
            viewer_count: live.viewer_count,
            viewer_peak: live.viewer_peak,
            reminder_count: live.reminder_count,
        });
        let live_stream = listing.live_stream.map(|stream| AccountLiveStreamOutput {
            session_id: stream.session_id,
            status: stream.status,
            title: budget.optional_text(stream.title, 4096),
            scheduled_start_at: stream.scheduled_start_at,
            on_stage: stream.on_stage,
            round: stream.round.map(|round| AccountLiveRoundOutput {
                state: round.state,
                format: round.format,
                duration_seconds: round.duration_seconds,
                extend_seconds: round.extend_seconds,
                start_price: round.start_price.into(),
                opened_at: round.opened_at,
                ends_at: round.ends_at,
                closed_at: round.closed_at,
                final_price: round.final_price.map(Into::into),
            }),
        });
        Self {
            id: listing.id,
            title,
            description,
            category_id: listing.category_id,
            condition: listing.condition,
            listing_type: listing.listing_type,
            currency: listing.currency.clone(),
            market: listing.market,
            status: listing.status,
            quantity: listing.quantity,
            auction: listing.auction.map(|auction| AccountAuctionOutput {
                round: auction.round.map(|round| AccountRoundOutput {
                    number: round.number,
                    status: round.status,
                    outcome: round.outcome,
                    closed_at: round.closed_at,
                }),
                start_time: auction.start_time,
                end_time: auction.end_time,
                starting_price: auction.starting_price.into(),
                buy_now_price: auction.buy_now_price.map(Into::into),
                current_bid: auction.current_bid.map(Into::into),
                bid_count: auction.bid_count,
                bid_increment: auction.bid_increment.map(Into::into),
                next_min_bid: auction.next_min_bid.map(Into::into),
                reserve: AccountReserveOutput {
                    exists: auction.reserve.exists,
                    met: auction.reserve.met,
                    price: auction.reserve.price.map(Into::into),
                },
            }),
            fixed_price: listing.fixed_price.map(|fixed| AccountFixedPriceOutput {
                price: fixed.price.into(),
                buy_now_price: fixed.buy_now_price.map(Into::into),
                start_time: fixed.start_time,
                end_time: fixed.end_time,
            }),
            images,
            shipping: AccountShippingOutput {
                custom_price: listing.shipping.custom_price.map(Into::into),
                package_size: listing.shipping.package_size,
                package_weight_grams: listing.shipping.package_weight_grams,
                provider_codes: listing.shipping.provider_codes,
                combined_enabled: listing.shipping.combined_enabled,
                location_id: listing.shipping.location_id,
                location: listing
                    .shipping
                    .location
                    .map(|location| PublicLocationOutput::project(location, &mut budget)),
                allow_pickup: listing.shipping.allow_pickup,
            },
            buyer_protection: listing.buyer_protection.map(|protection| {
                AccountBuyerProtectionOutput {
                    rate_percent: protection.rate_percent,
                    flat_fee: MoneyOutput::from_amount(protection.flat_fee, &listing.currency),
                    percentage_threshold: MoneyOutput::from_amount(
                        protection.percentage_threshold,
                        &listing.currency,
                    ),
                    mandatory_threshold: MoneyOutput::from_amount(
                        protection.mandatory_threshold,
                        &listing.currency,
                    ),
                    cap: MoneyOutput::from_amount(protection.cap, &listing.currency),
                    enabled: protection.enabled,
                }
            }),
            views_count: listing.views_count,
            approved: listing.approved,
            featured: listing.featured,
            is_boosted: listing.is_boosted,
            has_search_priority: listing.has_search_priority,
            tags,
            created_at: listing.created_at,
            updated_at: listing.updated_at,
            attributes,
            showcase,
            live,
            live_stream,
            live_session_id: listing.live_session_id,
            truncated: budget.truncated(),
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}

#[cfg(test)]
mod tests;
