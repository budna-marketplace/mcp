use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{self, Write},
    marker::PhantomData,
};

use budna_mcp_client::{
    COLLECTION_ID_PATTERN, CategoryListRequest, LISTING_ID_PATTERN, LIVE_STREAM_ID_PATTERN,
    SearchListingsRequest, USER_ID_PATTERN, is_collection_id, is_listing_id, is_live_stream_id,
    is_user_id,
};
use rmcp::schemars;
use serde::{Deserialize, Deserializer, de::DeserializeOwned};

const MAX_SEARCH_RESULTS: u32 = 50;
const MAX_SEARCH_PAGE: u32 = 10_000;
const MAX_SEARCH_PRICE_MAJOR_UNITS: &str = "1000000000000";
const MAX_CATEGORY_RESULTS: i32 = 200;
const MAX_CATEGORY_PAGE: i32 = 10_000;
const MAX_LISTING_PAGE_RESULTS: u32 = 50;
const MAX_LISTING_PAGE: u32 = 10_000;
const MAX_CUSTOM_FILTERS: usize = 20;
const MAX_TOOL_ARGUMENT_BYTES: usize = 64 * 1024;
const MAX_LIVE_STREAM_RESULTS: u32 = 20;
const MAX_BRAND_RESULTS: u32 = 100;
const MAX_RATING_RESULTS: u32 = 20;
const MAX_SUGGESTION_RESULTS: u32 = 20;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EmptyAccountParams {}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountListingsParams {
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountListingsParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        validate_listing_page(self.page, self.limit)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountPageParams {
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountPageParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        validate_listing_page(self.page, self.limit)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountBidsParams {
    /// Return only bids that currently lead their auction.
    pub winning_only: Option<bool>,
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountBidsParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        validate_listing_page(self.page, self.limit)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OfferRoleParam {
    Sent,
    Received,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountOffersParams {
    /// sent: offers you made on other listings; received: offers on your listings.
    pub role: OfferRoleParam,
    /// Return only offers that still await a response.
    pub open_only: Option<bool>,
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountOffersParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        validate_listing_page(self.page, self.limit)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountWatchlistListingsParams {
    /// Watchlist ID returned by list_my_watchlists.
    #[schemars(range(min = 1))]
    pub watchlist_id: i64,
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountWatchlistListingsParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        if self.watchlist_id < 1 {
            return Err(InputError::new("watchlist_id must be at least 1"));
        }
        validate_listing_page(self.page, self.limit)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountConversationsParams {
    /// Return only conversations with unread messages.
    pub unread_only: Option<bool>,
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountConversationsParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        validate_listing_page(self.page, self.limit)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountConversationParams {
    /// Conversation ID returned by list_my_conversations.
    #[schemars(range(min = 1))]
    pub conversation_id: i64,
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountConversationParams {
    pub fn validate(&self) -> Result<(i64, u32, u32), InputError> {
        if self.conversation_id < 1 {
            return Err(InputError::new("conversation_id must be at least 1"));
        }
        let (page, limit) = validate_listing_page(self.page, self.limit)?;
        Ok((self.conversation_id, page, limit))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountCursorParams {
    /// Cursor returned by the previous page; omit for the newest entries.
    #[schemars(length(min = 1, max = 128))]
    pub cursor: Option<String>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountCursorParams {
    pub fn validate(&self) -> Result<(Option<&str>, u32), InputError> {
        if self
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.is_empty() || cursor.len() > 128)
        {
            return Err(InputError::new("cursor must be 1..128 characters"));
        }
        Ok((
            self.cursor.as_deref(),
            validate_listing_page(None, self.limit)?.1,
        ))
    }
}

/// The period seller insights cover.
#[derive(Clone, Copy, Debug, Default, Deserialize, schemars::JsonSchema)]
pub enum InsightsRangeParam {
    #[serde(rename = "7d")]
    Week,
    #[default]
    #[serde(rename = "30d")]
    Month,
    #[serde(rename = "90d")]
    Quarter,
}

impl InsightsRangeParam {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Week => "7d",
            Self::Month => "30d",
            Self::Quarter => "90d",
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountInsightsParams {
    /// 7d, 30d or 90d; defaults to 30d.
    pub range: Option<InsightsRangeParam>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountCompanyIdParams {
    /// Company ID returned by list_my_companies.
    #[schemars(range(min = 1))]
    pub company_id: i64,
}

impl AccountCompanyIdParams {
    pub fn validate(&self) -> Result<i64, InputError> {
        if self.company_id < 1 {
            return Err(InputError::new("company_id must be at least 1"));
        }
        Ok(self.company_id)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountLimitParams {
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountLimitParams {
    pub fn validate(&self) -> Result<u32, InputError> {
        Ok(validate_listing_page(None, self.limit)?.1)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountOrderIdParams {
    /// Order ID returned by list_my_orders or a notification.
    #[schemars(range(min = 1))]
    pub order_id: i64,
}

impl AccountOrderIdParams {
    pub fn validate(&self) -> Result<i64, InputError> {
        if self.order_id < 1 {
            return Err(InputError::new("order_id must be at least 1"));
        }
        Ok(self.order_id)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderRoleParam {
    Buyer,
    Seller,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountOrdersParams {
    /// buyer: your purchases; seller: your sales.
    pub role: OrderRoleParam,
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountOrdersParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        validate_listing_page(self.page, self.limit)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountNotificationsParams {
    /// Return only notifications the account has not read.
    pub unread_only: Option<bool>,
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountNotificationsParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        validate_listing_page(self.page, self.limit)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RatingDirectionParam {
    Received,
    Given,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountRatingsParams {
    /// received: ratings others gave you; given: ratings you gave others.
    pub direction: RatingDirectionParam,
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountRatingsParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        validate_listing_page(self.page, self.limit)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountDisputeIdParams {
    /// Dispute ID returned by list_my_disputes.
    #[schemars(range(min = 1))]
    pub dispute_id: i64,
}

impl AccountDisputeIdParams {
    pub fn validate(&self) -> Result<i64, InputError> {
        positive_id("dispute_id", self.dispute_id)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountOrderGroupIdParams {
    /// Combined-shipping group ID returned by list_my_order_groups.
    #[schemars(range(min = 1))]
    pub group_id: i64,
}

impl AccountOrderGroupIdParams {
    pub fn validate(&self) -> Result<i64, InputError> {
        positive_id("group_id", self.group_id)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountOfferIdParams {
    /// Offer ID returned by list_my_offers or a notification.
    #[schemars(range(min = 1))]
    pub offer_id: i64,
}

impl AccountOfferIdParams {
    pub fn validate(&self) -> Result<i64, InputError> {
        positive_id("offer_id", self.offer_id)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountSavedSearchMatchesParams {
    /// Saved search ID returned by list_my_saved_searches.
    #[schemars(range(min = 1))]
    pub saved_search_id: i64,
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountSavedSearchMatchesParams {
    pub fn validate(&self) -> Result<(i64, u32, u32), InputError> {
        let saved_search_id = positive_id("saved_search_id", self.saved_search_id)?;
        let (page, limit) = validate_listing_page(self.page, self.limit)?;
        Ok((saved_search_id, page, limit))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AccountMessageSearchParams {
    /// Text to find in message bodies, 3 to 100 characters after trimming.
    #[schemars(length(min = 3, max = 100))]
    pub query: String,
    /// next_page from the previous result; defaults to 1.
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl AccountMessageSearchParams {
    pub fn validate(&self) -> Result<(&str, u32, u32), InputError> {
        let query = self.query.trim();
        if !(3..=100).contains(&query.chars().count()) {
            return Err(InputError::new(
                "query must contain 3 to 100 characters after trimming",
            ));
        }
        let (page, limit) = validate_listing_page(self.page, self.limit)?;
        Ok((query, page, limit))
    }
}

fn positive_id(field: &str, value: i64) -> Result<i64, InputError> {
    if value < 1 {
        return Err(InputError::new(format!("{field} must be at least 1")));
    }
    Ok(value)
}

struct BoundedSizeSink {
    remaining: usize,
}

impl Write for BoundedSizeSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.remaining {
            return Err(io::Error::other("serialized argument limit exceeded"));
        }
        self.remaining -= bytes.len();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn tool_arguments_oversized(raw: &serde_json::Value) -> bool {
    let mut sink = BoundedSizeSink {
        remaining: MAX_TOOL_ARGUMENT_BYTES,
    };
    serde_json::to_writer(&mut sink, raw).is_err()
}

#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct CustomFiltersParam(BTreeMap<String, String>);

impl From<BTreeMap<String, String>> for CustomFiltersParam {
    fn from(filters: BTreeMap<String, String>) -> Self {
        Self(filters)
    }
}

impl schemars::JsonSchema for CustomFiltersParam {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("BudnaCustomFilters")
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "maxProperties": 20,
            "propertyNames": {
                "type": "string",
                "pattern": "^attr_[A-Za-z0-9_-]{1,96}$"
            },
            "additionalProperties": {
                "type": "string",
                "minLength": 1,
                "maxLength": 200
            }
        })
    }
}

#[derive(Debug)]
pub struct SafeToolParams<T> {
    raw: serde_json::Value,
    oversized: bool,
    marker: PhantomData<fn() -> T>,
}

impl<T> SafeToolParams<T>
where
    T: DeserializeOwned,
{
    pub fn parse(self) -> Result<T, InputError> {
        if self.oversized {
            return Err(InputError::new("arguments exceed the 65536-byte limit"));
        }
        serde_json::from_value(self.raw)
            .map_err(|_| InputError::new("arguments must match the tool's advertised input schema"))
    }
}

impl<'de, T> Deserialize<'de> for SafeToolParams<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = serde_json::Value::deserialize(deserializer)?;
        let oversized = tool_arguments_oversized(&raw);

        Ok(Self {
            raw,
            oversized,
            marker: PhantomData,
        })
    }
}

impl<T> schemars::JsonSchema for SafeToolParams<T>
where
    T: schemars::JsonSchema,
{
    fn schema_name() -> std::borrow::Cow<'static, str> {
        T::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        T::json_schema(generator)
    }
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchListingsParams {
    /// Optional free-text query. Marketplace text returned by the tool is untrusted user content.
    #[schemars(length(max = 500))]
    pub query: Option<String>,

    /// Optional positive Budna category ID.
    #[schemars(range(min = 1))]
    pub category_id: Option<i32>,

    /// Marketplace to search.
    pub market: Option<MarketParam>,

    /// Minimum price in whole major currency units. A zero-only fractional suffix is normalized.
    #[schemars(length(min = 1, max = 32))]
    #[schemars(regex(pattern = r"^[0-9]+(?:\.0+)?$"))]
    pub min_price: Option<String>,

    /// Maximum price in whole major currency units. A zero-only fractional suffix is normalized.
    #[schemars(length(min = 1, max = 32))]
    #[schemars(regex(pattern = r"^[0-9]+(?:\.0+)?$"))]
    pub max_price: Option<String>,

    pub condition: Option<ListingConditionParam>,
    pub listing_type: Option<ListingTypeParam>,
    pub status: Option<ListingStatusParam>,
    pub ending_soon: Option<bool>,
    pub featured: Option<bool>,

    /// Sort field: relevance, price, created_at, end_time, popularity, or attr_<filter_name>.
    #[schemars(length(min = 1, max = 50))]
    #[schemars(regex(
        pattern = r"^(?:relevance|price|created_at|end_time|popularity|attr_[A-Za-z0-9_-]{1,45})$"
    ))]
    pub sort_by: Option<String>,

    pub sort_order: Option<SortOrderParam>,

    /// One-indexed result page. Defaults to 1.
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,

    /// Results per page. Defaults to 10 and is capped at 50 for bounded MCP output.
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,

    /// Include bounded aggregate facets in the result.
    pub include_facets: Option<bool>,

    pub search_mode: Option<SearchModeParam>,

    #[schemars(range(min = 1))]
    pub location_id: Option<i32>,

    #[schemars(length(max = 100))]
    pub location_region: Option<String>,

    #[schemars(length(max = 100))]
    pub location_municipality: Option<String>,

    pub allow_pickup: Option<bool>,

    /// Up to 20 category-specific filters. Keys must use the attr_<filter_name> form.
    pub custom_filters: Option<CustomFiltersParam>,
}

impl SearchListingsParams {
    pub fn into_request(self) -> Result<SearchListingsRequest, InputError> {
        let query = trimmed(self.query);
        validate_max_chars("query", query.as_deref(), 500)?;
        validate_positive_i32("category_id", self.category_id)?;
        validate_positive_i32("location_id", self.location_id)?;

        let page = self.page.unwrap_or(1);
        if !(1..=MAX_SEARCH_PAGE).contains(&page) {
            return Err(InputError::new("page must be between 1 and 10000"));
        }

        let limit = self.limit.unwrap_or(10);
        if !(1..=MAX_SEARCH_RESULTS).contains(&limit) {
            return Err(InputError::new("limit must be between 1 and 50"));
        }

        let min_price = normalize_search_price("min_price", self.min_price)?;
        let max_price = normalize_search_price("max_price", self.max_price)?;
        if let (Some(min_price), Some(max_price)) = (&min_price, &max_price)
            && search_price_whole_units(min_price)? > search_price_whole_units(max_price)?
        {
            return Err(InputError::new(
                "min_price must not be greater than max_price",
            ));
        }

        let sort_by = trimmed(self.sort_by);
        validate_max_chars("sort_by", sort_by.as_deref(), 50)?;
        if let Some(value) = sort_by.as_deref()
            && !is_valid_sort_field(value)
        {
            return Err(InputError::new(
                "sort_by must be relevance, price, created_at, end_time, popularity, or attr_<filter_name>",
            ));
        }

        let location_region = trimmed(self.location_region);
        let location_municipality = trimmed(self.location_municipality);
        validate_max_chars("location_region", location_region.as_deref(), 100)?;
        validate_max_chars(
            "location_municipality",
            location_municipality.as_deref(),
            100,
        )?;
        validate_filter_literal("location_region", location_region.as_deref())?;
        validate_filter_literal("location_municipality", location_municipality.as_deref())?;

        let custom_filters = normalize_custom_filters(self.custom_filters.unwrap_or_default().0)?;

        Ok(SearchListingsRequest {
            query,
            category_id: self.category_id,
            market: self.market.map(MarketParam::as_str).map(str::to_owned),
            min_price,
            max_price,
            condition: self
                .condition
                .map(ListingConditionParam::as_str)
                .map(str::to_owned),
            listing_type: self
                .listing_type
                .map(ListingTypeParam::as_str)
                .map(str::to_owned),
            status: self
                .status
                .map(ListingStatusParam::as_str)
                .map(str::to_owned),
            ending_soon: self.ending_soon,
            featured: self.featured,
            sort_by,
            sort_order: self
                .sort_order
                .map(SortOrderParam::as_str)
                .map(str::to_owned),
            page,
            limit,
            include_facets: self.include_facets.unwrap_or(false),
            search_mode: self
                .search_mode
                .map(SearchModeParam::as_str)
                .map(str::to_owned),
            location_id: self.location_id,
            location_region,
            location_municipality,
            allow_pickup: self.allow_pickup,
            custom_filters,
        })
    }
}

#[derive(Debug, Default, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GetCategoriesParams {
    /// One-indexed result page. Defaults to 1.
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<i32>,

    /// Categories per page. Defaults to 100 and is capped at 200 for bounded MCP output.
    #[schemars(range(min = 1, max = 200))]
    pub limit: Option<i32>,

    /// Optional positive parent category ID.
    #[schemars(range(min = 1))]
    pub parent_id: Option<i32>,

    /// Include localized category names. Defaults to true.
    pub translations: Option<bool>,
}

impl GetCategoriesParams {
    pub fn into_request(self) -> Result<CategoryListRequest, InputError> {
        let page = self.page.unwrap_or(1);
        if !(1..=MAX_CATEGORY_PAGE).contains(&page) {
            return Err(InputError::new("page must be between 1 and 10000"));
        }

        let limit = self.limit.unwrap_or(100);
        if !(1..=MAX_CATEGORY_RESULTS).contains(&limit) {
            return Err(InputError::new("limit must be between 1 and 200"));
        }
        validate_positive_i32("parent_id", self.parent_id)?;

        Ok(CategoryListRequest {
            page,
            limit,
            parent_id: self.parent_id,
            include_filters: false,
            translations: self.translations.unwrap_or(true),
        })
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListingIdParams {
    /// Public Budna listing ID, prefixed with listing_.
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub listing_id: String,
}

impl ListingIdParams {
    pub fn validate(&self) -> Result<(), InputError> {
        validate_listing_id(&self.listing_id)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListingIdPageParams {
    /// Public Budna listing ID, prefixed with listing_.
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub listing_id: String,

    /// One-indexed result page. Defaults to 1.
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,

    /// Results per page. Defaults to 10 and is capped at 50 for bounded MCP output.
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl ListingIdPageParams {
    pub fn validate(&self) -> Result<(&str, u32, u32), InputError> {
        validate_listing_id(&self.listing_id)?;
        let (page, limit) = validate_listing_page(self.page, self.limit)?;
        Ok((&self.listing_id, page, limit))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SellerIdParams {
    /// Public Budna seller user ID, prefixed with user_.
    #[schemars(length(min = 41, max = 41), regex(pattern = USER_ID_PATTERN))]
    pub seller_id: String,
}

impl SellerIdParams {
    pub fn validate(&self) -> Result<(), InputError> {
        validate_seller_id(&self.seller_id)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SellerListingsParams {
    /// Public Budna seller user ID, prefixed with user_.
    #[schemars(length(min = 41, max = 41), regex(pattern = USER_ID_PATTERN))]
    pub seller_id: String,

    /// One-indexed result page. Defaults to 1.
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,

    /// Results per page. Defaults to 10 and is capped at 50 for bounded MCP output.
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl SellerListingsParams {
    pub fn validate(&self) -> Result<(&str, u32, u32), InputError> {
        validate_seller_id(&self.seller_id)?;
        let (page, limit) = validate_listing_page(self.page, self.limit)?;
        Ok((&self.seller_id, page, limit))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CategoryFiltersParams {
    /// Positive Budna category ID.
    #[schemars(range(min = 1))]
    pub category_id: i32,

    /// Include localized filter labels. Defaults to true.
    pub translations: Option<bool>,
}

impl CategoryFiltersParams {
    pub fn validate(&self) -> Result<(i32, bool), InputError> {
        validate_positive_i32("category_id", Some(self.category_id))?;
        Ok((self.category_id, self.translations.unwrap_or(true)))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FilterOptionsParams {
    /// Positive Budna filter definition ID.
    #[schemars(range(min = 1))]
    pub filter_id: i32,

    /// Include localized option labels. Defaults to true.
    pub translations: Option<bool>,
}

impl FilterOptionsParams {
    pub fn validate(&self) -> Result<(i32, bool), InputError> {
        validate_positive_i32("filter_id", Some(self.filter_id))?;
        Ok((self.filter_id, self.translations.unwrap_or(true)))
    }
}

/// No arguments. The empty `properties` keeps the schema a closed object that every client reads.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("properties" = {}))]
pub struct EmptyParams {}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BrowseCollectionsParams {
    /// Only this seller's live and ended collections. Without it, the feed of live collections.
    #[schemars(length(min = 41, max = 41), regex(pattern = USER_ID_PATTERN))]
    pub seller_id: Option<String>,

    /// Feed only: the marketplace to browse.
    pub market: Option<MarketParam>,

    /// Feed only: just collections streaming live right now.
    pub live_only: Option<bool>,

    /// One-indexed result page. Defaults to 1.
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,

    /// Results per page. Defaults to 10 and is capped at 50.
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl BrowseCollectionsParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        if let Some(seller_id) = &self.seller_id {
            validate_seller_id(seller_id)?;
            if self.market.is_some() || self.live_only.is_some() {
                return Err(InputError::new(
                    "market and live_only apply only to the feed, without seller_id",
                ));
            }
        }
        validate_listing_page(self.page, self.limit)
    }

    pub fn market(&self) -> Option<&'static str> {
        self.market.map(MarketParam::as_str)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CollectionParams {
    /// Public Budna collection ID, prefixed with showcase_.
    #[schemars(length(min = 45, max = 45), regex(pattern = COLLECTION_ID_PATTERN))]
    pub collection_id: String,

    /// One-indexed page of the collection's items. Defaults to 1.
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,

    /// Items per page. Defaults to 10 and is capped at 50.
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl CollectionParams {
    pub fn validate(&self) -> Result<(&str, u32, u32), InputError> {
        if !is_collection_id(&self.collection_id) {
            return Err(InputError::new(
                "collection_id must be a canonical showcase_ UUIDv4 ID",
            ));
        }
        let (page, limit) = validate_listing_page(self.page, self.limit)?;
        Ok((&self.collection_id, page, limit))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LiveStreamStatusParam {
    Live,
    Upcoming,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LiveStreamsParams {
    /// live (default): on air now; upcoming: scheduled, soonest first.
    pub status: Option<LiveStreamStatusParam>,

    /// Streams to return. Defaults to 10 and is capped at 20.
    #[schemars(range(min = 1, max = 20))]
    pub limit: Option<u32>,
}

impl LiveStreamsParams {
    /// Whether to list upcoming streams, and how many.
    pub fn validate(&self) -> Result<(bool, u32), InputError> {
        let limit = self.limit.unwrap_or(10);
        if !(1..=MAX_LIVE_STREAM_RESULTS).contains(&limit) {
            return Err(InputError::new("limit must be between 1 and 20"));
        }
        Ok((
            matches!(self.status, Some(LiveStreamStatusParam::Upcoming)),
            limit,
        ))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LiveStreamParams {
    /// Public Budna live stream ID, prefixed with live_.
    #[schemars(length(min = 41, max = 41), regex(pattern = LIVE_STREAM_ID_PATTERN))]
    pub live_stream_id: String,
}

impl LiveStreamParams {
    pub fn validate(&self) -> Result<(), InputError> {
        is_live_stream_id(&self.live_stream_id)
            .then_some(())
            .ok_or_else(|| InputError::new("live_stream_id must be a canonical live_ UUIDv4 ID"))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BrandsParams {
    /// One brand by the slug get_brands returns. Cannot be combined with the other fields.
    #[schemars(length(min = 1, max = 100), regex(pattern = "^[a-z0-9-]+$"))]
    pub slug: Option<String>,

    /// Only brands Budna features.
    pub featured_only: Option<bool>,

    /// One-indexed result page. Defaults to 1.
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,

    /// Brands per page. Defaults to 50 and is capped at 100.
    #[schemars(range(min = 1, max = 100))]
    pub limit: Option<u32>,
}

impl BrandsParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        if let Some(slug) = &self.slug {
            if slug.is_empty()
                || slug.len() > 100
                || !slug
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            {
                return Err(InputError::new(
                    "slug must use lowercase letters, digits and hyphens",
                ));
            }
            if self.featured_only.is_some() || self.page.is_some() || self.limit.is_some() {
                return Err(InputError::new(
                    "slug cannot be combined with featured_only, page or limit",
                ));
            }
        }
        let page = self.page.unwrap_or(1);
        if !(1..=MAX_LISTING_PAGE).contains(&page) {
            return Err(InputError::new("page must be between 1 and 10000"));
        }
        let limit = self.limit.unwrap_or(50);
        if !(1..=MAX_BRAND_RESULTS).contains(&limit) {
            return Err(InputError::new("limit must be between 1 and 100"));
        }
        Ok((page, limit))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ListingFeedParam {
    Featured,
    Recent,
    Brand,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BrowseListingsParams {
    /// featured: listings Budna features; recent: newest first; brand: one brand's listings.
    pub feed: ListingFeedParam,

    /// The brand name, as get_brands returns it. Required for the brand feed only.
    #[schemars(length(min = 1, max = 100))]
    pub brand: Option<String>,

    /// One-indexed result page. Defaults to 1.
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,

    /// Results per page. Defaults to 10 and is capped at 50.
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl BrowseListingsParams {
    pub fn validate(&self) -> Result<(u32, u32), InputError> {
        let has_brand = self
            .brand
            .as_deref()
            .is_some_and(|brand| !brand.trim().is_empty() && brand.chars().count() <= 100);
        match (self.feed, &self.brand) {
            (ListingFeedParam::Brand, _) if !has_brand => {
                return Err(InputError::new(
                    "the brand feed needs a brand name of at most 100 characters",
                ));
            }
            (ListingFeedParam::Featured | ListingFeedParam::Recent, Some(_)) => {
                return Err(InputError::new("brand applies only to the brand feed"));
            }
            _ => {}
        }
        validate_listing_page(self.page, self.limit)
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BidHistoryParams {
    /// Public Budna listing ID, prefixed with listing_.
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub listing_id: String,

    /// An auction round from the returned rounds. Defaults to the current round.
    #[schemars(range(min = 1))]
    pub round_id: Option<i64>,
}

impl BidHistoryParams {
    pub fn validate(&self) -> Result<(), InputError> {
        validate_listing_id(&self.listing_id)?;
        if self.round_id.is_some_and(|round_id| round_id < 1) {
            return Err(InputError::new("round_id must be at least 1"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SellerRatingsParams {
    /// Public Budna seller user ID, prefixed with user_.
    #[schemars(length(min = 41, max = 41), regex(pattern = USER_ID_PATTERN))]
    pub seller_id: String,

    /// One-indexed page of ratings. Defaults to 1.
    #[schemars(range(min = 1, max = 10000))]
    pub page: Option<u32>,

    /// Ratings per page. Defaults to 10 and is capped at 20.
    #[schemars(range(min = 1, max = 20))]
    pub limit: Option<u32>,
}

impl SellerRatingsParams {
    pub fn validate(&self) -> Result<(&str, u32, u32), InputError> {
        validate_seller_id(&self.seller_id)?;
        let page = self.page.unwrap_or(1);
        if !(1..=MAX_LISTING_PAGE).contains(&page) {
            return Err(InputError::new("page must be between 1 and 10000"));
        }
        let limit = self.limit.unwrap_or(10);
        if !(1..=MAX_RATING_RESULTS).contains(&limit) {
            return Err(InputError::new("limit must be between 1 and 20"));
        }
        Ok((&self.seller_id, page, limit))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchSuggestionsParams {
    /// What the buyer has typed so far.
    #[schemars(length(min = 1, max = 200))]
    pub query: String,

    pub market: Option<MarketParam>,

    /// Defaults to 10 and is capped at 20.
    #[schemars(range(min = 1, max = 20))]
    pub limit: Option<u32>,
}

impl SearchSuggestionsParams {
    pub fn validate(&self) -> Result<(String, Option<&'static str>, u32), InputError> {
        let query = trimmed(Some(self.query.clone()))
            .ok_or_else(|| InputError::new("query must not be empty"))?;
        validate_max_chars("query", Some(query.as_str()), 200)?;
        let limit = self.limit.unwrap_or(10);
        if !(1..=MAX_SUGGESTION_RESULTS).contains(&limit) {
            return Err(InputError::new("limit must be between 1 and 20"));
        }
        Ok((query, self.market.map(MarketParam::as_str), limit))
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum CountryParam {
    #[default]
    Se,
    No,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LocationsParams {
    /// SE (default) for Sweden or NO for Norway.
    pub country: Option<CountryParam>,

    /// Count only listings in this category and its subcategories.
    #[schemars(range(min = 1))]
    pub category_id: Option<i32>,
}

impl LocationsParams {
    pub fn validate(&self) -> Result<(&'static str, Option<i32>), InputError> {
        validate_positive_i32("category_id", self.category_id)?;
        let country = match self.country.unwrap_or_default() {
            CountryParam::Se => "SE",
            CountryParam::No => "NO",
        };
        Ok((country, self.category_id))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListingQuestionsParams {
    /// Public Budna listing ID, prefixed with listing_.
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub listing_id: String,

    /// next_before_id from the previous page, to read older messages.
    #[schemars(range(min = 1))]
    pub before_id: Option<i64>,

    /// Messages per page. Defaults to 20 and is capped at 50.
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
}

impl ListingQuestionsParams {
    pub fn validate(&self) -> Result<(&str, Option<i64>, u32), InputError> {
        validate_listing_id(&self.listing_id)?;
        if self.before_id.is_some_and(|before_id| before_id < 1) {
            return Err(InputError::new("before_id must be at least 1"));
        }
        let (_, limit) = validate_listing_page(None, Some(self.limit.unwrap_or(20)))?;
        Ok((&self.listing_id, self.before_id, limit))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MarketParam {
    Norwegian,
    Swedish,
}

impl MarketParam {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Norwegian => "norwegian",
            Self::Swedish => "swedish",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ListingConditionParam {
    New,
    LikeNew,
    VeryGood,
    Good,
    Acceptable,
}

impl ListingConditionParam {
    const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::LikeNew => "like_new",
            Self::VeryGood => "very_good",
            Self::Good => "good",
            Self::Acceptable => "acceptable",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ListingTypeParam {
    Auction,
    FixedPrice,
    AuctionFixedPrice,
}

impl ListingTypeParam {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Auction => "auction",
            Self::FixedPrice => "fixed_price",
            Self::AuctionFixedPrice => "auction_fixed_price",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ListingStatusParam {
    Active,
    Sold,
    Expired,
}

impl ListingStatusParam {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Sold => "sold",
            Self::Expired => "expired",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SortOrderParam {
    Asc,
    Desc,
}

impl SortOrderParam {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchModeParam {
    Keyword,
    Semantic,
    Hybrid,
}

impl SearchModeParam {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Keyword => "keyword",
            Self::Semantic => "semantic",
            Self::Hybrid => "hybrid",
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct InputError {
    message: String,
}

impl InputError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

fn trimmed(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_owned())
    })
}

fn validate_positive_i32(field: &str, value: Option<i32>) -> Result<(), InputError> {
    if value.is_some_and(|value| value < 1) {
        return Err(InputError::new(format!("{field} must be at least 1")));
    }
    Ok(())
}

fn validate_listing_id(value: &str) -> Result<(), InputError> {
    is_listing_id(value)
        .then_some(())
        .ok_or_else(|| InputError::new("listing_id must be a canonical listing_ UUIDv4 ID"))
}

fn validate_seller_id(value: &str) -> Result<(), InputError> {
    is_user_id(value)
        .then_some(())
        .ok_or_else(|| InputError::new("seller_id must be a canonical user_ UUIDv4 ID"))
}

fn validate_listing_page(page: Option<u32>, limit: Option<u32>) -> Result<(u32, u32), InputError> {
    let page = page.unwrap_or(1);
    if !(1..=MAX_LISTING_PAGE).contains(&page) {
        return Err(InputError::new("page must be between 1 and 10000"));
    }

    let limit = limit.unwrap_or(10);
    if !(1..=MAX_LISTING_PAGE_RESULTS).contains(&limit) {
        return Err(InputError::new("limit must be between 1 and 50"));
    }

    Ok((page, limit))
}

fn validate_max_chars(field: &str, value: Option<&str>, max: usize) -> Result<(), InputError> {
    if value.is_some_and(|value| value.chars().count() > max) {
        return Err(InputError::new(format!(
            "{field} must contain at most {max} characters"
        )));
    }
    Ok(())
}

fn normalize_search_price(
    field: &str,
    value: Option<String>,
) -> Result<Option<String>, InputError> {
    let Some(value) = trimmed(value) else {
        return Ok(None);
    };

    if value.len() > 32 {
        return Err(search_price_error(field));
    }

    let (whole, fraction) = match value.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (value.as_str(), None),
    };
    if whole.is_empty() || !whole.chars().all(|character| character.is_ascii_digit()) {
        return Err(search_price_error(field));
    }
    if let Some(fraction) = fraction
        && (fraction.is_empty()
            || !fraction
                .chars()
                .all(|character| character.is_ascii_digit() && character == '0'))
    {
        return Err(search_price_error(field));
    }

    let normalized = whole.trim_start_matches('0');
    let normalized = if normalized.is_empty() {
        "0".to_owned()
    } else {
        normalized.to_owned()
    };
    if normalized.len() > MAX_SEARCH_PRICE_MAJOR_UNITS.len()
        || (normalized.len() == MAX_SEARCH_PRICE_MAJOR_UNITS.len()
            && normalized.as_str() > MAX_SEARCH_PRICE_MAJOR_UNITS)
    {
        return Err(search_price_error(field));
    }
    Ok(Some(normalized))
}

fn search_price_whole_units(value: &str) -> Result<u64, InputError> {
    value
        .parse::<u64>()
        .map_err(|_| search_price_error("price"))
}

fn search_price_error(field: &str) -> InputError {
    InputError::new(format!(
        "{field} must use whole major currency units no greater than {MAX_SEARCH_PRICE_MAJOR_UNITS}"
    ))
}

fn is_valid_sort_field(value: &str) -> bool {
    matches!(
        value,
        "relevance" | "price" | "created_at" | "end_time" | "popularity"
    ) || value
        .strip_prefix("attr_")
        .is_some_and(is_valid_filter_name)
}

fn normalize_custom_filters(
    filters: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, InputError> {
    if filters.len() > MAX_CUSTOM_FILTERS {
        return Err(InputError::new(
            "custom_filters supports at most 20 entries",
        ));
    }

    let mut normalized = BTreeMap::new();
    for (key, value) in filters {
        let Some(name) = key.strip_prefix("attr_") else {
            return Err(InputError::new("custom filter keys must start with attr_"));
        };
        if !is_valid_filter_name(name) {
            return Err(InputError::new(
                "custom filter names may contain only letters, numbers, underscores, and hyphens",
            ));
        }
        let value = value.trim();
        if value.is_empty() || value.chars().count() > 200 {
            return Err(InputError::new(
                "custom filter values must contain between 1 and 200 characters",
            ));
        }
        if key.ends_with("-min") || key.ends_with("-max") {
            let range_value =
                normalize_search_price("custom filter range", Some(value.to_owned()))?
                    .ok_or_else(|| InputError::new("custom filter range cannot be empty"))?;
            normalized.insert(key, range_value);
            continue;
        }

        let mut values = Vec::new();
        for part in value.split(',') {
            let part = part.trim();
            if part.is_empty() || !is_safe_filter_literal(part) {
                return Err(InputError::new(
                    "custom filter values contain unsupported search syntax",
                ));
            }
            values.push(part);
        }
        normalized.insert(key, values.join(","));
    }
    Ok(normalized)
}

fn validate_filter_literal(field: &str, value: Option<&str>) -> Result<(), InputError> {
    if value.is_some_and(|value| !is_safe_filter_literal(value)) {
        return Err(InputError::new(format!(
            "{field} contains unsupported search syntax"
        )));
    }
    Ok(())
}

fn is_safe_filter_literal(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_alphanumeric()
                || character.is_whitespace()
                || matches!(character, '_' | '-' | '.' | '/' | '+' | '\'')
        })
}

fn is_valid_filter_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_message_search_trims_and_bounds_its_query() {
        let params = |query: &str| AccountMessageSearchParams {
            query: query.to_owned(),
            page: None,
            limit: None,
        };
        assert!(matches!(
            params("  leica  ").validate(),
            Ok(("leica", 1, 10))
        ));
        assert!(params("  ab  ").validate().is_err());
        assert!(params(&"å".repeat(101)).validate().is_err());
        assert!(params(&"å".repeat(100)).validate().is_ok());
    }

    #[test]
    fn account_record_ids_must_be_positive() {
        assert!(
            AccountSavedSearchMatchesParams {
                saved_search_id: 0,
                page: None,
                limit: None,
            }
            .validate()
            .is_err()
        );
        assert!(
            AccountDisputeIdParams { dispute_id: -1 }
                .validate()
                .is_err()
        );
        assert_eq!(
            AccountOfferIdParams { offer_id: 11 }
                .validate()
                .unwrap_or_else(|error| panic!("a positive ID is valid: {}", error.message())),
            11
        );
    }

    #[test]
    fn search_defaults_are_bounded() {
        let request = SearchListingsParams::default()
            .into_request()
            .unwrap_or_else(|error| panic!("defaults should validate: {}", error.message()));

        assert_eq!(request.page, 1);
        assert_eq!(request.limit, 10);
        assert!(!request.include_facets);
    }

    #[test]
    fn invalid_ids_limits_and_filters_are_rejected() {
        let invalid_limit = SearchListingsParams {
            limit: Some(51),
            ..SearchListingsParams::default()
        };
        assert!(invalid_limit.into_request().is_err());

        let invalid_category = SearchListingsParams {
            category_id: Some(0),
            ..SearchListingsParams::default()
        };
        assert!(invalid_category.into_request().is_err());

        let invalid_filter = SearchListingsParams {
            custom_filters: Some(
                BTreeMap::from([("category_id".to_owned(), "1".to_owned())]).into(),
            ),
            ..SearchListingsParams::default()
        };
        assert!(invalid_filter.into_request().is_err());
    }

    #[test]
    fn category_pages_are_bounded() {
        let invalid_page = GetCategoriesParams {
            page: Some(MAX_CATEGORY_PAGE + 1),
            ..GetCategoriesParams::default()
        };

        let error = match invalid_page.into_request() {
            Ok(_) => panic!("oversized category page should fail"),
            Err(error) => error,
        };
        assert_eq!(error.message(), "page must be between 1 and 10000");
    }

    #[test]
    fn listing_and_seller_ids_require_their_public_string_contracts() {
        let listing = "listing_00000000-0000-4000-8000-000000000007";
        let seller = "user_00000000-0000-4000-8000-00000000002a";
        assert!(
            ListingIdParams {
                listing_id: listing.to_owned()
            }
            .validate()
            .is_ok()
        );
        assert!(
            SellerIdParams {
                seller_id: seller.to_owned()
            }
            .validate()
            .is_ok()
        );
        assert!(
            ListingIdParams {
                listing_id: seller.to_owned()
            }
            .validate()
            .is_err()
        );
        assert!(
            SellerIdParams {
                seller_id: listing.to_owned()
            }
            .validate()
            .is_err()
        );
        assert!(
            serde_json::from_value::<ListingIdParams>(serde_json::json!({"listing_id": 7}))
                .is_err()
        );
        let schema = serde_json::to_value(schemars::schema_for!(ListingIdParams))
            .unwrap_or_else(|error| panic!("input schema should serialize: {error}"));
        assert_eq!(
            schema.pointer("/properties/listing_id/type"),
            Some(&serde_json::json!("string"))
        );
        assert_eq!(
            schema.pointer("/properties/listing_id/pattern"),
            Some(&serde_json::json!(LISTING_ID_PATTERN))
        );
    }

    #[test]
    fn listing_pages_and_public_filter_ids_are_bounded() {
        let params = ListingIdPageParams {
            listing_id: "listing_00000000-0000-4000-8000-000000000007".to_owned(),
            page: None,
            limit: None,
        };
        let related = params
            .validate()
            .unwrap_or_else(|error| panic!("defaults should validate: {}", error.message()));
        assert_eq!(
            related,
            ("listing_00000000-0000-4000-8000-000000000007", 1, 10)
        );

        let invalid_related = ListingIdPageParams {
            listing_id: "listing_00000000-0000-4000-8000-000000000007".to_owned(),
            page: Some(MAX_LISTING_PAGE + 1),
            limit: None,
        };
        assert!(invalid_related.validate().is_err());

        let invalid_seller = SellerListingsParams {
            seller_id: "user_00000000-0000-0000-8000-000000000000".to_owned(),
            page: None,
            limit: Some(1),
        };
        assert!(invalid_seller.validate().is_err());

        let invalid_filter = FilterOptionsParams {
            filter_id: 0,
            translations: None,
        };
        assert!(invalid_filter.validate().is_err());
    }

    #[test]
    fn whole_price_filters_are_normalized() {
        let params = SearchListingsParams {
            min_price: Some("0010.00".to_owned()),
            max_price: Some("2000.00".to_owned()),
            ..SearchListingsParams::default()
        };

        let request = params
            .into_request()
            .unwrap_or_else(|error| panic!("whole prices should validate: {}", error.message()));
        assert_eq!(request.min_price.as_deref(), Some("10"));
        assert_eq!(request.max_price.as_deref(), Some("2000"));
    }

    #[test]
    fn invalid_price_ranges_and_filter_syntax_are_rejected() {
        let fractional = SearchListingsParams {
            min_price: Some("10.50".to_owned()),
            ..SearchListingsParams::default()
        };
        assert!(fractional.into_request().is_err());

        let overflowing = SearchListingsParams {
            min_price: Some("1000000000001".to_owned()),
            ..SearchListingsParams::default()
        };
        assert!(overflowing.into_request().is_err());

        let reversed = SearchListingsParams {
            min_price: Some("21".to_owned()),
            max_price: Some("20".to_owned()),
            ..SearchListingsParams::default()
        };
        assert!(reversed.into_request().is_err());

        let injected = SearchListingsParams {
            custom_filters: Some(
                BTreeMap::from([(
                    "attr_color".to_owned(),
                    "red || unexpected:=true".to_owned(),
                )])
                .into(),
            ),
            ..SearchListingsParams::default()
        };
        assert!(injected.into_request().is_err());
    }

    #[test]
    fn search_schema_advertises_dynamic_filter_and_sort_constraints() {
        let schema = serde_json::to_value(schemars::schema_for!(SearchListingsParams))
            .unwrap_or_else(|error| panic!("schema should serialize: {error}"));

        assert_eq!(
            schema.pointer("/properties/custom_filters/maxProperties"),
            Some(&serde_json::json!(20))
        );
        assert_eq!(
            schema.pointer("/properties/custom_filters/propertyNames/pattern"),
            Some(&serde_json::json!("^attr_[A-Za-z0-9_-]{1,96}$"))
        );
        assert_eq!(
            schema.pointer("/properties/custom_filters/additionalProperties/maxLength"),
            Some(&serde_json::json!(200))
        );
        assert!(
            schema
                .pointer("/properties/sort_by/pattern")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|pattern| pattern.contains("attr_"))
        );
    }

    #[test]
    fn safe_tool_params_bound_stdio_argument_size_before_typed_parsing() {
        let oversized = serde_json::json!({
            "query": "x".repeat(MAX_TOOL_ARGUMENT_BYTES + 1)
        });

        let params = serde_json::from_value::<SafeToolParams<SearchListingsParams>>(oversized)
            .unwrap_or_else(|error| panic!("raw tool arguments should deserialize: {error}"));

        let Err(error) = params.parse() else {
            panic!("oversized tool arguments should be rejected");
        };
        assert_eq!(error.message(), "arguments exceed the 65536-byte limit");
    }
}
