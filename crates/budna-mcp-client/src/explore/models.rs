use serde::Deserialize;

use crate::{CategoryTranslations, ListingResponse, Money, Pagination, models::PublicListingWire};

/// A seller's listings under one banner, as the public sees it.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Collection {
    pub id: String,
    pub seller: PublicSeller,
    pub kind: String,
    pub status: String,
    pub title: String,
    pub description: String,
    pub starts_at: Option<i64>,
    pub ends_at: i64,
    pub market: String,
    pub spotlight_listing_id: Option<String>,
    pub public_item_count: i64,
    pub is_boosted: bool,
    pub live: Option<LiveSession>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PublicSeller {
    pub id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CollectionPage {
    pub items: Vec<Collection>,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollectionDetail {
    pub collection: Collection,
    pub items: Vec<CollectionItem>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollectionItem {
    pub listing: ListingResponse,
    pub position: i32,
    pub lot_number: Option<i32>,
    pub aligned: bool,
}

/// A live stream on a listing, a collection or a seller's whole shop.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct LiveSession {
    pub id: String,
    pub status: String,
    pub listing_id: Option<String>,
    pub showcase_id: Option<String>,
    pub shop: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub stage_listing_id: Option<String>,
    pub scheduled_start_at: i64,
    pub started_at: Option<i64>,
    pub ended_at: Option<i64>,
    pub viewer_count: i64,
    pub reminder_count: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveShop {
    pub session: LiveSession,
    pub seller: PublicSeller,
    pub stage: Option<ListingResponse>,
    pub preview_items: Vec<ListingResponse>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveStream {
    pub session: LiveSession,
    /// Present for a shop stream, which has a stage.
    pub stage: Option<LiveStage>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LiveStage {
    pub seller: PublicSeller,
    pub on_stage: Option<ListingResponse>,
    pub round: Option<LiveRound>,
    pub items: Vec<ListingResponse>,
    pub recent: Vec<LiveRound>,
}

/// A live auction on one listing during a stream.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct LiveRound {
    pub listing_id: String,
    pub state: String,
    pub format: String,
    pub duration_seconds: i32,
    pub extend_seconds: Option<i32>,
    pub start_price: Money,
    pub opened_at: i64,
    pub ends_at: i64,
    pub closed_at: Option<i64>,
    pub final_price: Option<Money>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Brand {
    pub name: String,
    pub slug: String,
    pub is_featured: bool,
    pub listing_count: i64,
}

/// One sales attempt of an auction listing.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct AuctionRound {
    pub id: i64,
    pub listing_id: String,
    pub round: i32,
    pub status: String,
    pub outcome: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub bid_count: i64,
    pub highest_amount: Option<Money>,
    pub closed_at: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListingBidHistory {
    pub listing_id: String,
    pub rounds: Vec<AuctionRound>,
    /// The round the bids belong to; `None` is the listing's current round.
    pub round_id: Option<i64>,
    pub bid_count: i64,
    /// Newest first.
    pub bids: Vec<PublicBid>,
    pub bidder_count: usize,
    /// Every bid of the round was loaded.
    pub complete: bool,
}

/// A bid without its bidder: only the bidder's number on this listing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicBid {
    pub bidder_number: usize,
    /// The viewer placed this bid.
    pub own: bool,
    pub amount: Money,
    pub bid_time: i64,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RatingStatistics {
    pub user_id: String,
    pub total_ratings_received: i64,
    pub total_ratings_given: i64,
    pub average_rating_received: f64,
    pub rating_distribution: [i64; 5],
    pub total_comments_received: i64,
    pub latest_rating_received_at: Option<i64>,
    pub reputation_level: String,
    pub most_common_rating_received: Option<i32>,
    pub positive_rating_percentage: f64,
    pub is_trusted_member: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SellerRatings {
    pub seller_id: String,
    pub statistics: RatingStatistics,
    pub ratings: Vec<PublicRating>,
    pub pagination: Pagination,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublicRating {
    pub value: i32,
    pub comment: Option<String>,
    pub created_at: i64,
    pub verified_transaction: bool,
    pub rating_type: Option<String>,
    pub rater_username: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SearchSuggestions {
    pub listings: Vec<ListingSuggestion>,
    pub categories: Vec<CategorySuggestion>,
}

/// The suggestion's price is left out: it is not reliably in the listing's currency.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ListingSuggestion {
    pub id: String,
    pub title: String,
    pub image_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CategorySuggestion {
    pub id: i32,
    pub name: String,
    pub translations: Option<CategoryTranslations>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct LocationRegion {
    pub country_code: String,
    pub region_name: String,
    pub listing_count: i64,
    pub cities: Vec<LocationCity>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct LocationCity {
    pub location_id: i32,
    pub municipality_name: String,
    pub listing_count: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShippingRates {
    pub package_sizes: Vec<PackagePreset>,
    pub from_country: String,
    pub to_country: String,
    pub rates: Vec<ShippingRate>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PackagePreset {
    pub size_code: String,
    pub label: String,
    pub max_weight_grams: i32,
    pub max_length_mm: i32,
    pub max_width_mm: i32,
    pub max_height_mm: i32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ShippingRate {
    pub provider_code: String,
    pub package_size: String,
    pub amount: Money,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListingQuestions {
    pub listing_id: String,
    pub questions: Vec<ListingQuestion>,
    pub pinned: Vec<ChatMessage>,
    /// Newest first.
    pub messages: Vec<ChatMessage>,
    pub next_before_id: Option<i64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListingQuestion {
    pub question: ChatMessage,
    pub answer: Option<ChatMessage>,
}

/// A public chat message, its author named only by public username.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatMessage {
    pub username: Option<String>,
    pub from_seller: bool,
    pub body: String,
    pub created_at: i64,
    pub reply_to: Option<ChatReply>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatReply {
    pub username: Option<String>,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CollectionDetailWire {
    pub showcase: Collection,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CollectionItemsWire {
    pub items: Vec<CollectionItemWire>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CollectionItemWire {
    pub listing: PublicListingWire,
    pub position: i32,
    pub lot_number: Option<i32>,
    pub aligned: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LiveShopWire {
    pub session: LiveSession,
    pub seller: PublicSeller,
    pub stage: Option<PublicListingWire>,
    pub preview_items: Vec<PublicListingWire>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LiveStageWire {
    pub session: LiveSession,
    pub seller: PublicSeller,
    pub stage: Option<PublicListingWire>,
    pub round: Option<LiveRound>,
    pub items: Vec<PublicListingWire>,
    pub recent: Vec<LiveRound>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BrandWire {
    pub brand: Brand,
}

/// A paginated response that carries its page metadata beside `data`.
#[derive(Debug, Deserialize)]
pub(crate) struct PagedEnvelope<T> {
    pub success: bool,
    pub data: Option<Vec<T>>,
    pub pagination: Option<PageMeta>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PageMeta {
    pub page: i64,
    pub per_page: i64,
    pub total: i64,
    pub total_pages: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BidWire {
    pub id: i64,
    pub listing_id: String,
    pub bidder_id: String,
    pub price: Money,
    pub bid_time: i64,
    pub status: String,
    pub auction_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RatingFeedWire {
    pub items: Vec<RatingWire>,
    pub pagination: Pagination,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RatingWire {
    pub value: i32,
    pub comment: Option<String>,
    pub created_at: i64,
    pub verified_transaction: bool,
    pub rating_type: Option<String>,
    pub rater_id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProfileNameWire {
    pub user_id: String,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LocationsWire {
    pub regions: Vec<LocationRegion>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DomesticRatesWire {
    pub from_country: String,
    pub to_country: String,
    pub rates: Vec<ShippingRate>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatFeedWire {
    pub seller_id: String,
    pub messages: Vec<ChatMessageWire>,
    pub pinned: Vec<ChatMessageWire>,
    pub questions: Vec<ChatQuestionWire>,
    pub next_before_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatQuestionWire {
    pub question: ChatMessageWire,
    pub answer: Option<ChatMessageWire>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatMessageWire {
    pub id: i64,
    pub listing_id: Option<String>,
    pub user_id: String,
    pub username: Option<String>,
    pub is_owner: bool,
    pub reply_to: Option<ChatReplyWire>,
    pub body: String,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ChatReplyWire {
    pub message_id: i64,
    pub user_id: String,
    pub username: Option<String>,
    pub body: String,
}
