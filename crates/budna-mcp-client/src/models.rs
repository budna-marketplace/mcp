use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Money {
    pub amount: String,
    pub currency_code: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SearchListingsRequest {
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listing_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ending_soon: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,
    pub page: u32,
    #[serde(rename = "per_page")]
    pub limit: u32,
    pub include_facets: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_municipality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_pickup: Option<bool>,
    #[serde(flatten)]
    pub custom_filters: BTreeMap<String, String>,
}

impl Default for SearchListingsRequest {
    fn default() -> Self {
        Self {
            query: None,
            category_id: None,
            market: None,
            min_price: None,
            max_price: None,
            condition: None,
            listing_type: None,
            status: None,
            ending_soon: None,
            featured: None,
            sort_by: None,
            sort_order: None,
            page: 1,
            limit: 10,
            include_facets: false,
            search_mode: None,
            location_id: None,
            location_region: None,
            location_municipality: None,
            allow_pickup: None,
            custom_filters: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ListingSearchResult {
    pub hits: Vec<SearchListingHit>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub search_time_ms: u64,
    pub facets: Option<SearchFacets>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SearchListingHit {
    pub id: String,
    pub seller_id: String,
    pub title: Option<String>,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub category_breadcrumb: Option<String>,
    pub condition: String,
    pub listing_type: String,
    pub currency: String,
    pub market: String,
    pub starting_price: String,
    pub current_bid: Option<String>,
    pub buy_now_price: Option<String>,
    pub status: String,
    pub start_time: i64,
    pub end_time: i64,
    pub featured: bool,
    pub tags: Vec<String>,
    pub image_ids: Vec<String>,
    pub primary_image_id: Option<String>,
    pub ending_soon: bool,
    pub has_bids: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SearchFacets {
    pub categories: Vec<FacetCount>,
    pub conditions: Vec<FacetCount>,
    pub listing_types: Vec<FacetCount>,
    pub markets: Vec<FacetCount>,
    pub statuses: Vec<FacetCount>,
    pub regions: Vec<FacetCount>,
    pub cities: Vec<FacetCount>,
    pub allow_pickup: Vec<FacetCount>,
    pub price_stats: Option<PriceStats>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PriceStats {
    pub min: String,
    pub max: String,
    pub avg: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct FacetCount {
    pub value: String,
    pub count: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(try_from = "ListingWire")]
pub struct ListingResponse {
    pub id: String,
    pub seller_id: String,
    pub seller_name: Option<String>,
    pub seller_username: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub condition: String,
    pub listing_type: String,
    pub currency: String,
    pub market: String,
    pub starting_price: Money,
    pub bid_increment: Option<Money>,
    pub current_bid: Option<Money>,
    pub reserve_price_met: bool,
    pub buy_now_price: Option<Money>,
    pub shipping_cost: Option<Money>,
    pub quantity: i32,
    pub status: String,
    pub start_time: i64,
    pub end_time: i64,
    pub views_count: i32,
    pub bid_count: Option<i64>,
    pub featured: bool,
    pub tags: Vec<String>,
    pub image_ids: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub package_size: Option<String>,
    pub package_weight_grams: Option<i32>,
    pub shipping_provider_codes: Option<Vec<String>>,
    pub location: Option<ListingLocation>,
    pub allow_pickup: bool,
    pub buyer_protection_config: Option<BuyerProtectionConfig>,
}

#[derive(Debug, Deserialize)]
struct ListingWire {
    id: String,
    seller: ListingSellerWire,
    title: Option<String>,
    description: Option<String>,
    category_id: Option<i32>,
    condition: String,
    listing_type: String,
    currency: String,
    market: String,
    quantity: i32,
    status: String,
    auction: Option<ListingAuctionWire>,
    fixed_price: Option<ListingFixedPriceWire>,
    images: Vec<ListingImageWire>,
    shipping: ListingShippingWire,
    buyer_protection: Option<BuyerProtectionConfig>,
    views_count: i32,
    featured: bool,
    tags: Vec<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Deserialize)]
struct ListingSellerWire {
    id: String,
    name: Option<String>,
    username: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListingAuctionWire {
    start_time: i64,
    end_time: i64,
    starting_price: Money,
    buy_now_price: Option<Money>,
    current_bid: Option<Money>,
    bid_count: Option<i64>,
    bid_increment: Option<Money>,
    reserve: ListingReserveWire,
}

#[derive(Debug, Deserialize)]
struct ListingReserveWire {
    met: bool,
}

#[derive(Debug, Deserialize)]
struct ListingFixedPriceWire {
    price: Money,
    buy_now_price: Option<Money>,
    start_time: i64,
    end_time: i64,
}

#[derive(Debug, Deserialize)]
struct ListingImageWire {
    id: String,
}

#[derive(Debug, Deserialize)]
struct ListingShippingWire {
    custom_price: Option<Money>,
    package_size: Option<String>,
    package_weight_grams: Option<i32>,
    provider_codes: Option<Vec<String>>,
    location: Option<ListingLocation>,
    allow_pickup: bool,
}

impl TryFrom<ListingWire> for ListingResponse {
    type Error = &'static str;

    fn try_from(wire: ListingWire) -> Result<Self, Self::Error> {
        let (
            starting_price,
            buy_now_price,
            current_bid,
            bid_increment,
            bid_count,
            reserve_price_met,
            start_time,
            end_time,
        ) = match (wire.listing_type.as_str(), wire.auction, wire.fixed_price) {
            ("auction" | "auction_fixed_price", Some(auction), None) => (
                auction.starting_price,
                auction.buy_now_price,
                auction.current_bid,
                auction.bid_increment,
                auction.bid_count,
                auction.reserve.met,
                auction.start_time,
                auction.end_time,
            ),
            ("fixed_price" | "in_stock", None, Some(fixed)) => (
                fixed.price,
                fixed.buy_now_price,
                None,
                None,
                None,
                false,
                fixed.start_time,
                fixed.end_time,
            ),
            _ => return Err("listing type must match exactly one price section"),
        };
        Ok(Self {
            id: wire.id,
            seller_id: wire.seller.id,
            seller_name: wire.seller.name,
            seller_username: wire.seller.username,
            title: wire.title,
            description: wire.description,
            category_id: wire.category_id,
            condition: wire.condition,
            listing_type: wire.listing_type,
            currency: wire.currency,
            market: wire.market,
            starting_price,
            bid_increment,
            current_bid,
            reserve_price_met,
            buy_now_price,
            shipping_cost: wire.shipping.custom_price,
            quantity: wire.quantity,
            status: wire.status,
            start_time,
            end_time,
            views_count: wire.views_count,
            bid_count,
            featured: wire.featured,
            tags: wire.tags,
            image_ids: wire.images.into_iter().map(|image| image.id).collect(),
            created_at: wire.created_at,
            updated_at: wire.updated_at,
            package_size: wire.shipping.package_size,
            package_weight_grams: wire.shipping.package_weight_grams,
            shipping_provider_codes: wire.shipping.provider_codes,
            location: wire.shipping.location,
            allow_pickup: wire.shipping.allow_pickup,
            buyer_protection_config: wire.buyer_protection,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ListingPage {
    pub items: Vec<ListingResponse>,
    pub pagination: Pagination,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ListingLocation {
    pub city: String,
    pub region: Option<String>,
    pub country: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct BuyerProtectionConfig {
    pub rate_percent: String,
    pub flat_fee: String,
    pub mandatory_threshold: String,
    pub cap: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CategoryListRequest {
    pub page: i32,
    pub limit: i32,
    pub parent_id: Option<i32>,
    pub include_filters: bool,
    pub translations: bool,
}

impl Default for CategoryListRequest {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 100,
            parent_id: None,
            include_filters: false,
            translations: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CategoryPage {
    pub items: Vec<CategorySummary>,
    pub pagination: Pagination,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CategorySummary {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub listing_count: i64,
    pub translations: Option<CategoryTranslations>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CategoryWithFilters {
    pub id: i32,
    pub name: String,
    pub parent_id: Option<i32>,
    pub listing_count: i64,
    pub filters: Option<CategoryFilters>,
    pub translations: Option<CategoryTranslations>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct CategoryFilters {
    pub baseline_filters: Vec<FilterWithOptions>,
    pub category_filters: Vec<FilterWithOptions>,
    pub inherited_filters: Vec<FilterWithOptions>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FilterWithOptions {
    pub definition: FilterDefinition,
    pub options: Option<Vec<FilterOption>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FilterDefinition {
    pub id: i32,
    pub name: String,
    pub label: String,
    pub filter_type: String,
    pub is_baseline: bool,
    pub sortable: bool,
    pub configuration: FilterConfiguration,
    pub validation_rules: ValidationRules,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub translations: Option<FilterTranslations>,
    pub option_count: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct FilterConfiguration {
    pub placeholder: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_number_or_string")]
    pub min_value: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_number_or_string")]
    pub max_value: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_number_or_string")]
    pub step: Option<String>,
    pub unit: Option<String>,
    pub max_stars: Option<i32>,
    pub multiple: Option<bool>,
    pub required: Option<bool>,
    pub searchable: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ValidationRules {
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub pattern: Option<String>,
    pub required: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct FilterTranslations {
    pub label: Option<TranslationMap>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct FilterOptionTranslations {
    pub label: Option<TranslationMap>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FilterOption {
    pub id: i32,
    pub filter_id: i32,
    pub value: String,
    pub display_value: String,
    pub display_order: i32,
    pub is_active: bool,
    pub created_at: i64,
    pub is_suggested: bool,
    pub translations: Option<FilterOptionTranslations>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct FilterOptionList {
    pub options: Vec<FilterOption>,
    pub filter_id: i32,
    pub total: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CategoryTranslations {
    pub name: TranslationMap,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct TranslationMap {
    pub en: String,
    pub sv: String,
    pub no: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Pagination {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub total_pages: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SellerProfileSummary {
    pub id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub display_name: String,
    pub bio: Option<String>,
    pub language: String,
    pub currency: String,
    pub auction_history: PublicAuctionHistory,
    pub verification_status: PublicVerificationStatus,
    pub rating: String,
    pub total_ratings: i32,
    pub image_id: Option<String>,
    pub categories: Vec<String>,
    pub is_company: bool,
    pub created_at: i64,
    pub followers_count: Option<i64>,
    pub following_count: Option<i64>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub level: Option<i32>,
    pub level_name: Option<String>,
    pub unlocked_badges: Option<Vec<PublicBadge>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ListingAttributes {
    pub listing_id: String,
    pub attributes: Vec<ListingAttribute>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ListingAttribute {
    pub id: i64,
    pub listing_id: String,
    pub filter_definition_id: i32,
    pub filter_name: String,
    pub label: String,
    pub value: AttributeValue,
    pub display_value: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[non_exhaustive]
#[serde(tag = "type", content = "value")]
pub enum AttributeValue {
    String(String),
    Numeric(serde_json::Value),
    Boolean(bool),
    Date(i64),
    Json(serde_json::Value),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PublicAuctionHistory {
    pub won_auctions_count: u64,
    pub sold_items_count: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PublicVerificationStatus {
    pub id_verified: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PublicBadge {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub icon_url: Option<String>,
    pub unlocked_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ListingBidSummary {
    pub listing_id: String,
    pub bid_count: Option<i64>,
    pub current_bid: Option<Money>,
    pub reserve_price_met: bool,
    pub listing_status: String,
    pub end_time: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct RatingSummary {
    pub listing_id: String,
    pub total_ratings: i64,
    pub average_rating: f64,
    pub rating_distribution: [i64; 5],
    pub total_comments: i64,
    pub has_ratings: bool,
    pub has_comments: bool,
    pub most_common_rating: Option<i32>,
    pub positive_percentage: f64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PublicListingPageWire {
    pub items: Vec<PublicListingWire>,
    pub pagination: Pagination,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PublicListingWire {
    #[serde(flatten)]
    pub listing: ListingResponse,
    pub approved: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ApiEnvelope<T> {
    pub success: bool,
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProblemDetails {
    pub code: Option<String>,
}

fn deserialize_option_number_or_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(serde_json::Value::String(value)) => Ok(Some(value)),
        Some(serde_json::Value::Number(value)) => Ok(Some(value.to_string())),
        Some(_) => Err(serde::de::Error::custom("expected string or number")),
    }
}
