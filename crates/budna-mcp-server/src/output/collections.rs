use budna_mcp_client::{
    COLLECTION_ID_PATTERN, Collection, CollectionDetail, CollectionPage, LISTING_ID_PATTERN,
    LIVE_STREAM_ID_PATTERN, LiveRound, LiveSession, LiveShop, LiveStream, PublicSeller,
    USER_ID_PATTERN,
};
use budna_mcp_core::PublicUrlSettings;
use rmcp::schemars;
use serde::Serialize;

use super::{
    MAX_CODE_BYTES, MAX_DISPLAY_TEXT_BYTES, MAX_LONG_TEXT_BYTES, MAX_NAME_BYTES, ProjectionBudget,
    UNTRUSTED_CONTENT_NOTICE, common::MoneyOutput, listings::ListingSummaryOutput,
};

const MAX_COLLECTION_RESULTS: usize = 50;
const MAX_LIVE_STREAM_RESULTS: usize = 20;
const MAX_PREVIEW_ITEMS: usize = 3;
const MAX_STAGE_ITEMS: usize = 50;
const MAX_RECENT_ROUNDS: usize = 20;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SellerOutput {
    #[schemars(length(min = 41, max = 41), regex(pattern = USER_ID_PATTERN))]
    pub id: String,
    #[schemars(length(max = 256))]
    pub username: Option<String>,
    #[schemars(length(max = 256))]
    pub display_name: Option<String>,
}

impl SellerOutput {
    fn project(seller: PublicSeller, budget: &mut ProjectionBudget) -> Self {
        Self {
            id: seller.id,
            username: budget.optional_text(seller.username, MAX_NAME_BYTES),
            display_name: budget.optional_text(seller.display_name, MAX_NAME_BYTES),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LiveSessionOutput {
    #[schemars(length(min = 41, max = 41), regex(pattern = LIVE_STREAM_ID_PATTERN))]
    pub id: String,
    /// scheduled, live, ended or cancelled.
    #[schemars(length(max = 128))]
    pub status: String,
    /// A stream over the seller's whole shop, with a stage, rather than one listing or collection.
    pub shop: bool,
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub listing_id: Option<String>,
    #[schemars(length(min = 45, max = 45), regex(pattern = COLLECTION_ID_PATTERN))]
    pub collection_id: Option<String>,
    #[schemars(length(max = 256))]
    pub title: Option<String>,
    #[schemars(length(max = 4096))]
    pub description: Option<String>,
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub stage_listing_id: Option<String>,
    pub scheduled_start_at: i64,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub viewer_count: i64,
    /// Viewers waiting to hear when it starts.
    pub reminder_count: i64,
}

impl LiveSessionOutput {
    fn project(
        session: LiveSession,
        description_limit: usize,
        budget: &mut ProjectionBudget,
    ) -> Self {
        Self {
            id: session.id,
            status: budget.text(session.status, MAX_CODE_BYTES),
            shop: session.shop,
            listing_id: session.listing_id,
            collection_id: session.showcase_id,
            title: budget.optional_text(session.title, MAX_NAME_BYTES),
            description: budget.optional_text(session.description, description_limit),
            stage_listing_id: session.stage_listing_id,
            scheduled_start_at: session.scheduled_start_at,
            started_at: session.started_at,
            ended_at: session.ended_at,
            viewer_count: session.viewer_count,
            reminder_count: session.reminder_count,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CollectionOutput {
    #[schemars(length(min = 45, max = 45), regex(pattern = COLLECTION_ID_PATTERN))]
    pub id: String,
    pub seller: SellerOutput,
    /// theme keeps each listing's own end time; sale and auction end every item with the collection.
    #[schemars(length(max = 128))]
    pub kind: String,
    /// active or ended.
    #[schemars(length(max = 128))]
    pub status: String,
    #[schemars(length(max = 256))]
    pub title: String,
    #[schemars(length(max = 4096))]
    pub description: String,
    pub starts_at: Option<i64>,
    pub ends_at: i64,
    #[schemars(length(max = 128))]
    pub market: String,
    /// The listing shown as "now showing".
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub spotlight_listing_id: Option<String>,
    pub item_count: i64,
    /// A paid boost is active on this collection.
    pub boosted: bool,
    /// Its scheduled or running live stream.
    pub live_stream: Option<LiveSessionOutput>,
}

impl CollectionOutput {
    fn project(
        collection: Collection,
        description_limit: usize,
        budget: &mut ProjectionBudget,
    ) -> Self {
        Self {
            id: collection.id,
            seller: SellerOutput::project(collection.seller, budget),
            kind: budget.text(collection.kind, MAX_CODE_BYTES),
            status: budget.text(collection.status, MAX_CODE_BYTES),
            title: budget.text(collection.title, MAX_NAME_BYTES),
            description: budget.text(collection.description, description_limit),
            starts_at: collection.starts_at,
            ends_at: collection.ends_at,
            market: budget.text(collection.market, MAX_CODE_BYTES),
            spotlight_listing_id: collection.spotlight_listing_id,
            item_count: collection.public_item_count,
            boosted: collection.is_boosted,
            live_stream: collection
                .live
                .map(|live| LiveSessionOutput::project(live, MAX_DISPLAY_TEXT_BYTES, budget)),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CollectionsOutput {
    pub content_notice: String,
    pub truncated: bool,
    #[schemars(length(max = 50))]
    pub collections: Vec<CollectionOutput>,
    pub page: i64,
    pub limit: i64,
    /// Another page may follow.
    pub has_more: bool,
}

impl From<CollectionPage> for CollectionsOutput {
    fn from(page: CollectionPage) -> Self {
        let mut budget = ProjectionBudget::default();
        let collections = budget
            .objects(page.items, MAX_COLLECTION_RESULTS)
            .into_iter()
            .map(|collection| {
                CollectionOutput::project(collection, MAX_DISPLAY_TEXT_BYTES, &mut budget)
            })
            .collect();
        let truncated = budget.truncated();

        Self {
            content_notice: UNTRUSTED_CONTENT_NOTICE.to_owned(),
            truncated,
            collections,
            page: page.offset / page.limit.max(1) + 1,
            limit: page.limit,
            has_more: page.has_more,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CollectionItemOutput {
    pub position: i32,
    pub lot_number: Option<i32>,
    /// The listing ends with the collection.
    pub ends_with_collection: bool,
    pub listing: ListingSummaryOutput,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CollectionDetailOutput {
    pub content_notice: String,
    pub truncated: bool,
    pub collection: CollectionOutput,
    #[schemars(length(max = 50))]
    pub items: Vec<CollectionItemOutput>,
    pub items_total: i64,
    pub page: i64,
    pub limit: i64,
}

impl CollectionDetailOutput {
    pub fn from_with_public_urls(
        detail: CollectionDetail,
        public_urls: &PublicUrlSettings,
    ) -> Self {
        let mut budget = ProjectionBudget::default();
        let collection =
            CollectionOutput::project(detail.collection, MAX_LONG_TEXT_BYTES, &mut budget);
        let items = budget
            .objects(detail.items, MAX_COLLECTION_RESULTS)
            .into_iter()
            .map(|item| CollectionItemOutput {
                position: item.position,
                lot_number: item.lot_number,
                ends_with_collection: item.aligned,
                listing: ListingSummaryOutput::project(item.listing, public_urls, &mut budget),
            })
            .collect();
        let truncated = budget.truncated();

        Self {
            content_notice: UNTRUSTED_CONTENT_NOTICE.to_owned(),
            truncated,
            collection,
            items,
            items_total: detail.total,
            page: detail.offset / detail.limit.max(1) + 1,
            limit: detail.limit,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LiveShopOutput {
    pub stream: LiveSessionOutput,
    pub seller: SellerOutput,
    pub on_stage: Option<ListingSummaryOutput>,
    /// The first items announced for the stream.
    #[schemars(length(max = 3))]
    pub preview_items: Vec<ListingSummaryOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LiveStreamsOutput {
    pub content_notice: String,
    pub truncated: bool,
    #[schemars(length(max = 20))]
    pub live_streams: Vec<LiveShopOutput>,
}

impl LiveStreamsOutput {
    pub fn from_with_public_urls(shops: Vec<LiveShop>, public_urls: &PublicUrlSettings) -> Self {
        let mut budget = ProjectionBudget::default();
        let live_streams = budget
            .objects(shops, MAX_LIVE_STREAM_RESULTS)
            .into_iter()
            .map(|shop| {
                let preview_items = budget.objects(shop.preview_items, MAX_PREVIEW_ITEMS);
                LiveShopOutput {
                    stream: LiveSessionOutput::project(
                        shop.session,
                        MAX_DISPLAY_TEXT_BYTES,
                        &mut budget,
                    ),
                    seller: SellerOutput::project(shop.seller, &mut budget),
                    on_stage: shop.stage.map(|listing| {
                        ListingSummaryOutput::project(listing, public_urls, &mut budget)
                    }),
                    preview_items: preview_items
                        .into_iter()
                        .map(|listing| {
                            ListingSummaryOutput::project(listing, public_urls, &mut budget)
                        })
                        .collect(),
                }
            })
            .collect();
        let truncated = budget.truncated();

        Self {
            content_notice: UNTRUSTED_CONTENT_NOTICE.to_owned(),
            truncated,
            live_streams,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LiveRoundOutput {
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub listing_id: String,
    /// open, sold or passed.
    #[schemars(length(max = 128))]
    pub state: String,
    /// extend puts the clock back after a late bid; sudden_death never does.
    #[schemars(length(max = 128))]
    pub format: String,
    pub duration_seconds: i32,
    pub extend_seconds: Option<i32>,
    pub start_price: MoneyOutput,
    pub opened_at: i64,
    pub ends_at: i64,
    pub closed_at: Option<i64>,
    pub final_price: Option<MoneyOutput>,
}

impl LiveRoundOutput {
    fn project(round: LiveRound, budget: &mut ProjectionBudget) -> Self {
        Self {
            listing_id: round.listing_id,
            state: budget.text(round.state, MAX_CODE_BYTES),
            format: budget.text(round.format, MAX_CODE_BYTES),
            duration_seconds: round.duration_seconds,
            extend_seconds: round.extend_seconds,
            start_price: MoneyOutput::from(round.start_price),
            opened_at: round.opened_at,
            ends_at: round.ends_at,
            closed_at: round.closed_at,
            final_price: round.final_price.map(MoneyOutput::from),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LiveStreamOutput {
    pub content_notice: String,
    pub truncated: bool,
    pub stream: LiveSessionOutput,
    /// The fields below are set for shop streams, which have a stage.
    pub seller: Option<SellerOutput>,
    pub on_stage: Option<ListingSummaryOutput>,
    /// The live auction running now.
    pub round: Option<LiveRoundOutput>,
    /// Items still to come, in order.
    #[schemars(length(max = 50))]
    pub items: Vec<ListingSummaryOutput>,
    /// Finished live auctions, newest first.
    #[schemars(length(max = 20))]
    pub recent_rounds: Vec<LiveRoundOutput>,
}

impl LiveStreamOutput {
    pub fn from_with_public_urls(stream: LiveStream, public_urls: &PublicUrlSettings) -> Self {
        let mut budget = ProjectionBudget::default();
        let mut output = Self {
            content_notice: UNTRUSTED_CONTENT_NOTICE.to_owned(),
            truncated: false,
            stream: LiveSessionOutput::project(stream.session, MAX_LONG_TEXT_BYTES, &mut budget),
            seller: None,
            on_stage: None,
            round: None,
            items: Vec::new(),
            recent_rounds: Vec::new(),
        };
        if let Some(stage) = stream.stage {
            output.seller = Some(SellerOutput::project(stage.seller, &mut budget));
            output.on_stage = stage
                .on_stage
                .map(|listing| ListingSummaryOutput::project(listing, public_urls, &mut budget));
            output.round = stage
                .round
                .map(|round| LiveRoundOutput::project(round, &mut budget));
            output.items = budget
                .objects(stage.items, MAX_STAGE_ITEMS)
                .into_iter()
                .map(|listing| ListingSummaryOutput::project(listing, public_urls, &mut budget))
                .collect();
            output.recent_rounds = budget
                .objects(stage.recent, MAX_RECENT_ROUNDS)
                .into_iter()
                .map(|round| LiveRoundOutput::project(round, &mut budget))
                .collect();
        }
        output.truncated = budget.truncated();
        output
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use budna_mcp_client::{CollectionItem, LiveStage, Money};

    use super::*;
    use crate::output::{assert_within_mcp_result_budget, listings::tests::listing_fixture};

    fn keys(value: &serde_json::Value) -> BTreeSet<&str> {
        value
            .as_object()
            .map(|object| object.keys().map(String::as_str).collect())
            .unwrap_or_else(|| panic!("expected an object"))
    }

    fn public_id(prefix: &str, number: usize) -> String {
        format!("{prefix}00000000-0000-4000-8000-{number:012x}")
    }

    fn session(number: usize, text: &str) -> LiveSession {
        LiveSession {
            id: public_id("live_", number),
            status: "live".to_owned(),
            listing_id: None,
            showcase_id: None,
            shop: true,
            title: Some(text.to_owned()),
            description: Some(text.repeat(8)),
            stage_listing_id: Some(public_id("listing_", 7)),
            scheduled_start_at: 1_800_000_000_000,
            started_at: Some(1_800_000_000_000),
            ended_at: None,
            viewer_count: 12,
            reminder_count: 4,
        }
    }

    fn seller(text: &str) -> PublicSeller {
        PublicSeller {
            id: public_id("user_", 42),
            username: Some(text.to_owned()),
            display_name: Some(text.to_owned()),
        }
    }

    fn collection(number: usize, text: &str) -> Collection {
        let mut live = session(number, text);
        live.shop = false;
        live.showcase_id = Some(public_id("showcase_", number));
        Collection {
            id: public_id("showcase_", number),
            seller: seller(text),
            kind: "auction".to_owned(),
            status: "active".to_owned(),
            title: text.to_owned(),
            description: text.repeat(16),
            starts_at: Some(1_700_000_000_000),
            ends_at: 1_800_000_000_000,
            market: "swedish".to_owned(),
            spotlight_listing_id: Some(public_id("listing_", 7)),
            public_item_count: 3,
            is_boosted: true,
            live: Some(live),
        }
    }

    fn round(text: &str) -> LiveRound {
        let money = |amount: &str| Money {
            amount: amount.to_owned(),
            currency_code: "SEK".to_owned(),
        };
        LiveRound {
            listing_id: public_id("listing_", 7),
            state: text.to_owned(),
            format: "extend".to_owned(),
            duration_seconds: 30,
            extend_seconds: Some(5),
            start_price: money("100"),
            opened_at: 1_800_000_000_000,
            ends_at: 1_800_000_030_000,
            closed_at: None,
            final_price: Some(money("140")),
        }
    }

    #[test]
    fn collection_shapes_are_exact_and_bounded() {
        let output = CollectionsOutput::from(CollectionPage {
            items: (0..60)
                .map(|number| collection(number, &"💡".repeat(400)))
                .collect(),
            limit: 50,
            offset: 50,
            has_more: true,
        });
        assert!(output.truncated);
        assert_eq!(output.collections.len(), MAX_COLLECTION_RESULTS);
        assert_within_mcp_result_budget(&output);

        let value = serde_json::to_value(&output)
            .unwrap_or_else(|error| panic!("collections should serialize: {error}"));
        assert_eq!(value.pointer("/page"), Some(&serde_json::json!(2)));
        assert_eq!(
            keys(&value),
            BTreeSet::from([
                "collections",
                "content_notice",
                "has_more",
                "limit",
                "page",
                "truncated"
            ])
        );
        let collection = value
            .pointer("/collections/0")
            .unwrap_or_else(|| panic!("missing collection"));
        assert_eq!(
            keys(collection),
            BTreeSet::from([
                "boosted",
                "description",
                "ends_at",
                "id",
                "item_count",
                "kind",
                "live_stream",
                "market",
                "seller",
                "spotlight_listing_id",
                "starts_at",
                "status",
                "title",
            ])
        );
        assert_eq!(
            keys(&collection["seller"]),
            BTreeSet::from(["display_name", "id", "username"])
        );
        assert_eq!(
            keys(&collection["live_stream"]),
            BTreeSet::from([
                "collection_id",
                "description",
                "ended_at",
                "id",
                "listing_id",
                "reminder_count",
                "scheduled_start_at",
                "shop",
                "stage_listing_id",
                "started_at",
                "status",
                "title",
                "viewer_count",
            ])
        );
    }

    #[test]
    fn collection_detail_keeps_lots_and_stays_within_budget() {
        let items = (0..MAX_COLLECTION_RESULTS)
            .map(|number| {
                let mut listing = listing_fixture(number as i64 + 1);
                listing.description = Some("💡".repeat(1_024));
                CollectionItem {
                    listing,
                    position: number as i32,
                    lot_number: Some(number as i32 + 1),
                    aligned: true,
                }
            })
            .collect();
        let output = CollectionDetailOutput::from_with_public_urls(
            CollectionDetail {
                collection: collection(1, &"💡".repeat(2_000)),
                items,
                total: 120,
                limit: 50,
                offset: 0,
            },
            &PublicUrlSettings::default(),
        );
        assert_within_mcp_result_budget(&output);
        let value = serde_json::to_value(&output)
            .unwrap_or_else(|error| panic!("collection should serialize: {error}"));
        assert_eq!(
            keys(&value),
            BTreeSet::from([
                "collection",
                "content_notice",
                "items",
                "items_total",
                "limit",
                "page",
                "truncated"
            ])
        );
        assert_eq!(
            keys(
                value
                    .pointer("/items/0")
                    .unwrap_or_else(|| panic!("missing item"))
            ),
            BTreeSet::from(["ends_with_collection", "listing", "lot_number", "position"])
        );
        assert_eq!(
            value.pointer("/items/1/lot_number"),
            Some(&serde_json::json!(2))
        );
    }

    #[test]
    fn live_stream_shapes_are_exact_and_bounded() {
        let shops = (0..30)
            .map(|number| LiveShop {
                session: session(number, &"💡".repeat(300)),
                seller: seller("seller42"),
                stage: Some(listing_fixture(7)),
                preview_items: (1..=5).map(listing_fixture).collect(),
            })
            .collect();
        let output = LiveStreamsOutput::from_with_public_urls(shops, &PublicUrlSettings::default());
        assert!(output.truncated);
        assert_eq!(output.live_streams.len(), MAX_LIVE_STREAM_RESULTS);
        assert!(
            output
                .live_streams
                .iter()
                .all(|shop| shop.preview_items.len() <= MAX_PREVIEW_ITEMS)
        );
        assert_within_mcp_result_budget(&output);
        let value = serde_json::to_value(&output)
            .unwrap_or_else(|error| panic!("streams should serialize: {error}"));
        assert_eq!(
            keys(
                value
                    .pointer("/live_streams/0")
                    .unwrap_or_else(|| panic!("missing stream"))
            ),
            BTreeSet::from(["on_stage", "preview_items", "seller", "stream"])
        );

        let detail = LiveStreamOutput::from_with_public_urls(
            LiveStream {
                session: session(1, "Friday drop"),
                stage: Some(LiveStage {
                    seller: seller("seller42"),
                    on_stage: Some(listing_fixture(7)),
                    round: Some(round("open")),
                    items: (1..=60).map(listing_fixture).collect(),
                    recent: (0..30).map(|_| round("sold")).collect(),
                }),
            },
            &PublicUrlSettings::default(),
        );
        assert!(detail.truncated);
        assert_eq!(detail.items.len(), MAX_STAGE_ITEMS);
        assert_eq!(detail.recent_rounds.len(), MAX_RECENT_ROUNDS);
        assert_within_mcp_result_budget(&detail);
        let value = serde_json::to_value(&detail)
            .unwrap_or_else(|error| panic!("stream should serialize: {error}"));
        assert_eq!(
            keys(&value),
            BTreeSet::from([
                "content_notice",
                "items",
                "on_stage",
                "recent_rounds",
                "round",
                "seller",
                "stream",
                "truncated"
            ])
        );
        assert_eq!(
            keys(&value["round"]),
            BTreeSet::from([
                "closed_at",
                "duration_seconds",
                "ends_at",
                "extend_seconds",
                "final_price",
                "format",
                "listing_id",
                "opened_at",
                "start_price",
                "state",
            ])
        );
    }
}
