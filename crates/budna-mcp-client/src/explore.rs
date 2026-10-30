//! Public reads around listings: collections, live streams, brands, listing
//! feeds, bid history, seller ratings, search suggestions, locations, shipping
//! rates and listing questions. Every read is an anonymous `GET`.

mod models;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet, hash_map::Entry};

use reqwest::Url;

pub use models::{
    AuctionRound, Brand, CategorySuggestion, ChatMessage, ChatReply, Collection, CollectionDetail,
    CollectionItem, CollectionPage, ListingBidHistory, ListingQuestion, ListingQuestions,
    ListingSuggestion, LiveRound, LiveSession, LiveShop, LiveStage, LiveStream, LocationCity,
    LocationRegion, PackagePreset, PopularSearch, PublicBid, PublicRating, PublicSeller,
    RatingStatistics, SearchSuggestions, SellerRatings, ShippingRate, ShippingRates,
};
use models::{
    BidWire, BrandWire, ChatFeedWire, ChatMessageWire, CollectionDetailWire, CollectionItemsWire,
    DomesticRatesWire, LiveShopWire, LiveStageWire, LocationsWire, PagedEnvelope, ProfileNameWire,
    RatingFeedWire,
};

use crate::{
    ClientError, ListingPage, ListingResponse, MAX_LISTING_PAGE_RESULTS, Money, PublicApiClient,
    invalid_input, invalid_response, is_collection_id, is_listing_id, is_live_stream_id,
    is_public_listing_status, is_user_id, models::PublicListingWire, money_is_valid,
    normalize_not_found, optional_value_is_allowed, public_resource_unavailable, validate_listing,
    validate_page_input, validate_pagination, validate_public_id_input,
};

const BROWSE_COLLECTIONS: &str = "browse_collections";
const BROWSE_LISTINGS: &str = "browse_listings";
const MAX_COLLECTION_RESULTS: u32 = 50;
const MAX_LIVE_STREAM_RESULTS: u32 = 20;
const MAX_BRAND_RESULTS: u32 = 100;
const MAX_RATING_RESULTS: u32 = 20;
const MAX_SUGGESTION_RESULTS: u32 = 20;
const MAX_QUESTION_RESULTS: u32 = 50;
const MAX_SUGGESTION_QUERY_CHARS: usize = 200;
const MAX_BRAND_NAME_CHARS: usize = 100;
const MAX_BRAND_SLUG_BYTES: usize = 100;
const BID_PAGE_SIZE: u32 = 100;
const MAX_BID_PAGES: u32 = 5;
const MARKETS: [&str; 2] = ["norwegian", "swedish"];

impl PublicApiClient {
    /// The public feed of live collections, boosted ones first.
    pub async fn collection_feed(
        &self,
        market: Option<&str>,
        live_only: bool,
        page: u32,
        limit: u32,
    ) -> Result<CollectionPage, ClientError> {
        validate_page_input(BROWSE_COLLECTIONS, page, limit, MAX_COLLECTION_RESULTS)?;
        if !optional_value_is_allowed(market, &MARKETS) {
            return Err(invalid_input(
                BROWSE_COLLECTIONS,
                "market must be norwegian or swedish",
            ));
        }
        let offset = (page - 1) * limit;
        let mut request = self
            .http
            .get(self.endpoint(BROWSE_COLLECTIONS, "showcases")?)
            .query(&[("limit", limit), ("offset", offset)]);
        if let Some(market) = market {
            request = request.query(&[("market", market)]);
        }
        if live_only {
            request = request.query(&[("live", true)]);
        }
        let collections: CollectionPage = self.get_required(BROWSE_COLLECTIONS, request).await?;
        validate_collection_page(collections, limit, offset, None)
    }

    /// One seller's live and ended collections.
    pub async fn seller_collections(
        &self,
        seller_id: &str,
        page: u32,
        limit: u32,
    ) -> Result<CollectionPage, ClientError> {
        validate_public_id_input(
            BROWSE_COLLECTIONS,
            seller_id,
            is_user_id,
            "seller_id must be a public user ID",
        )?;
        validate_page_input(BROWSE_COLLECTIONS, page, limit, MAX_COLLECTION_RESULTS)?;
        let offset = (page - 1) * limit;
        let url = self.endpoint(BROWSE_COLLECTIONS, &format!("users/{seller_id}/showcases"))?;
        let request = self
            .http
            .get(url)
            .query(&[("limit", limit), ("offset", offset)]);
        let collections: CollectionPage = self
            .get_required_or_unavailable(
                BROWSE_COLLECTIONS,
                request,
                "SELLER_NOT_FOUND",
                "Seller not found",
            )
            .await?;
        validate_collection_page(collections, limit, offset, Some(seller_id))
    }

    /// One public collection and a page of its public items.
    pub async fn get_collection(
        &self,
        collection_id: &str,
        page: u32,
        limit: u32,
    ) -> Result<CollectionDetail, ClientError> {
        const OPERATION: &str = "get_collection";
        validate_public_id_input(
            OPERATION,
            collection_id,
            is_collection_id,
            "collection_id must be a public collection ID",
        )?;
        validate_page_input(OPERATION, page, limit, MAX_COLLECTION_RESULTS)?;
        let url = self.endpoint(OPERATION, &format!("showcases/{collection_id}"))?;
        let detail: CollectionDetailWire = self
            .get_required_or_unavailable(
                OPERATION,
                self.http.get(url),
                "COLLECTION_NOT_FOUND",
                "Collection not found",
            )
            .await?;
        let mut collection = detail.showcase;
        if collection.id != collection_id || !collection_is_public(&collection) {
            return Err(collection_not_found(OPERATION));
        }
        validate_collection(OPERATION, &mut collection)?;

        let offset = (page - 1) * limit;
        let url = self.endpoint(OPERATION, &format!("showcases/{collection_id}/items"))?;
        let request = self
            .http
            .get(url)
            .query(&[("limit", limit), ("offset", offset)]);
        let items_page: CollectionItemsWire = self
            .get_required_or_unavailable(
                OPERATION,
                request,
                "COLLECTION_NOT_FOUND",
                "Collection not found",
            )
            .await?;
        if items_page.limit != i64::from(limit)
            || items_page.offset != i64::from(offset)
            || items_page.items.len() > limit as usize
            || items_page.total < items_page.items.len() as i64
        {
            return Err(invalid_response(OPERATION));
        }
        let mut items = Vec::with_capacity(items_page.items.len());
        for item in items_page.items {
            if !is_public_card(&item.listing) {
                return Err(collection_not_found(OPERATION));
            }
            validate_listing(OPERATION, &item.listing.listing)?;
            if item.position < 0 || item.lot_number.is_some_and(|lot| lot < 0) {
                return Err(invalid_response(OPERATION));
            }
            items.push(CollectionItem {
                listing: item.listing.listing,
                position: item.position,
                lot_number: item.lot_number,
                aligned: item.aligned,
            });
        }
        Ok(CollectionDetail {
            collection,
            items,
            total: items_page.total,
            limit: items_page.limit,
            offset: items_page.offset,
        })
    }

    /// Shop streams on air now, or the upcoming ones soonest first.
    pub async fn live_streams(
        &self,
        upcoming: bool,
        limit: u32,
    ) -> Result<Vec<LiveShop>, ClientError> {
        const OPERATION: &str = "browse_live_streams";
        if !(1..=MAX_LIVE_STREAM_RESULTS).contains(&limit) {
            return Err(invalid_input(
                OPERATION,
                "limit is outside the allowed range",
            ));
        }
        let path = if upcoming {
            "live-sessions/upcoming"
        } else {
            "live-sessions"
        };
        let request = self
            .http
            .get(self.endpoint(OPERATION, path)?)
            .query(&[("limit", limit)]);
        let shops: Vec<LiveShopWire> = self.get_required(OPERATION, request).await?;
        if shops.len() > limit as usize {
            return Err(invalid_response(OPERATION));
        }
        shops
            .into_iter()
            .map(|shop| {
                validate_live_session(OPERATION, &shop.session)?;
                if !shop.session.shop || !is_user_id(&shop.seller.id) {
                    return Err(invalid_response(OPERATION));
                }
                Ok(LiveShop {
                    session: shop.session,
                    seller: shop.seller,
                    stage: public_cards(OPERATION, shop.stage.into_iter().collect())?.pop(),
                    preview_items: public_cards(OPERATION, shop.preview_items)?,
                })
            })
            .collect()
    }

    /// One live stream and, for a shop stream, its stage.
    pub async fn get_live_stream(&self, live_stream_id: &str) -> Result<LiveStream, ClientError> {
        const OPERATION: &str = "get_live_stream";
        validate_public_id_input(
            OPERATION,
            live_stream_id,
            is_live_stream_id,
            "live_stream_id must be a public live stream ID",
        )?;
        let url = self.endpoint(OPERATION, &format!("live-sessions/{live_stream_id}"))?;
        let session: LiveSession = self
            .get_required_or_unavailable(
                OPERATION,
                self.http.get(url),
                "LIVE_STREAM_NOT_FOUND",
                "Live stream not found",
            )
            .await?;
        if session.id != live_stream_id {
            return Err(live_stream_not_found(OPERATION));
        }
        validate_live_session(OPERATION, &session)?;
        if !session.shop {
            return Ok(LiveStream {
                session,
                stage: None,
            });
        }

        let url = self.endpoint(OPERATION, &format!("live-sessions/{live_stream_id}/stage"))?;
        let stage: LiveStageWire = self
            .get_required_or_unavailable(
                OPERATION,
                self.http.get(url),
                "LIVE_STREAM_NOT_FOUND",
                "Live stream not found",
            )
            .await?;
        if stage.session.id != live_stream_id || !stage.session.shop {
            return Err(live_stream_not_found(OPERATION));
        }
        validate_live_session(OPERATION, &stage.session)?;
        if !is_user_id(&stage.seller.id)
            || !stage
                .round
                .iter()
                .chain(&stage.recent)
                .all(live_round_is_valid)
        {
            return Err(invalid_response(OPERATION));
        }
        Ok(LiveStream {
            session: stage.session,
            stage: Some(LiveStage {
                seller: stage.seller,
                on_stage: public_cards(OPERATION, stage.stage.into_iter().collect())?.pop(),
                round: stage.round,
                items: public_cards(OPERATION, stage.items)?,
                recent: stage.recent,
            }),
        })
    }

    /// Brands with active listing counts, featured ones first.
    pub async fn get_brands(
        &self,
        featured_only: bool,
        page: u32,
        limit: u32,
    ) -> Result<Vec<Brand>, ClientError> {
        const OPERATION: &str = "get_brands";
        validate_page_input(OPERATION, page, limit, MAX_BRAND_RESULTS)?;
        let mut request = self
            .http
            .get(self.endpoint(OPERATION, "brands")?)
            .query(&[("page", page), ("limit", limit)]);
        if featured_only {
            request = request.query(&[("featured", true)]);
        }
        let brands: Vec<Brand> = self.get_required(OPERATION, request).await?;
        if brands.len() > limit as usize
            || brands
                .iter()
                .any(|brand| !brand_is_valid(brand) || (featured_only && !brand.is_featured))
        {
            return Err(invalid_response(OPERATION));
        }
        Ok(brands)
    }

    pub async fn get_brand(&self, slug: &str) -> Result<Brand, ClientError> {
        const OPERATION: &str = "get_brands";
        if slug.is_empty() || !is_brand_slug(slug) {
            return Err(invalid_input(
                OPERATION,
                "slug must use lowercase letters, digits and hyphens",
            ));
        }
        let url = self.endpoint(OPERATION, &format!("brands/slug/{slug}"))?;
        let wire: BrandWire = self
            .get_required_or_unavailable(
                OPERATION,
                self.http.get(url),
                "BRAND_NOT_FOUND",
                "Brand not found",
            )
            .await?;
        if wire.brand.slug != slug {
            return Err(public_resource_unavailable(
                OPERATION,
                "BRAND_NOT_FOUND",
                "Brand not found",
            ));
        }
        if !brand_is_valid(&wire.brand) {
            return Err(invalid_response(OPERATION));
        }
        Ok(wire.brand)
    }

    pub async fn featured_listings(
        &self,
        page: u32,
        limit: u32,
    ) -> Result<ListingPage, ClientError> {
        let url = self.endpoint(BROWSE_LISTINGS, "listings/featured")?;
        self.listing_feed(url, page, limit).await
    }

    pub async fn recent_listings(&self, page: u32, limit: u32) -> Result<ListingPage, ClientError> {
        let url = self.endpoint(BROWSE_LISTINGS, "listings/recent")?;
        self.listing_feed(url, page, limit).await
    }

    /// Active public listings whose brand attribute is exactly `brand`.
    pub async fn brand_listings(
        &self,
        brand: &str,
        page: u32,
        limit: u32,
    ) -> Result<ListingPage, ClientError> {
        if brand.chars().count() > MAX_BRAND_NAME_CHARS
            || !brand.chars().any(char::is_alphanumeric)
            || brand.chars().any(char::is_control)
        {
            return Err(invalid_input(
                BROWSE_LISTINGS,
                "brand must be a brand name of at most 100 characters",
            ));
        }
        let mut url = self.endpoint(BROWSE_LISTINGS, "listings/brand")?;
        // Pushed as one encoded segment, so a name cannot add path segments or a query.
        url.path_segments_mut()
            .map_err(|()| invalid_input(BROWSE_LISTINGS, "brand cannot be addressed"))?
            .push(brand);
        self.listing_feed(url, page, limit).await
    }

    async fn listing_feed(
        &self,
        url: Url,
        page: u32,
        limit: u32,
    ) -> Result<ListingPage, ClientError> {
        validate_page_input(BROWSE_LISTINGS, page, limit, MAX_LISTING_PAGE_RESULTS)?;
        let request = self
            .http
            .get(url)
            .query(&[("page", page), ("limit", limit)]);
        self.get_public_listing_page(BROWSE_LISTINGS, request, page, limit, None)
            .await
    }

    /// The listing's auction rounds and the bids of one round, bidders numbered.
    pub async fn get_listing_bid_history(
        &self,
        listing_id: &str,
        round_id: Option<i64>,
    ) -> Result<ListingBidHistory, ClientError> {
        const OPERATION: &str = "get_listing_bid_history";
        if round_id.is_some_and(|round_id| round_id < 1) {
            return Err(invalid_input(OPERATION, "round_id must be at least 1"));
        }
        let listing = self.get_public_listing(OPERATION, listing_id).await?;
        let url = self.endpoint(OPERATION, &format!("listings/{listing_id}/auctions"))?;
        let rounds: Vec<AuctionRound> = self
            .get_required_or_unavailable(
                OPERATION,
                self.http.get(url),
                "LISTING_NOT_FOUND",
                "Listing not found",
            )
            .await?;
        if !rounds.iter().all(|round| round_is_valid(round, listing_id)) {
            return Err(invalid_response(OPERATION));
        }
        if round_id.is_some_and(|round_id| !rounds.iter().any(|round| round.id == round_id)) {
            return Err(public_resource_unavailable(
                OPERATION,
                "AUCTION_ROUND_NOT_FOUND",
                "Auction round not found",
            ));
        }

        let mut bids = Vec::new();
        let mut seen = HashSet::new();
        let mut total = 0;
        for page in 1..=MAX_BID_PAGES {
            let url = self.endpoint(OPERATION, &format!("bids/listing/{listing_id}"))?;
            let mut request = self
                .http
                .get(url)
                .query(&[("page", page), ("limit", BID_PAGE_SIZE)]);
            if let Some(round_id) = round_id {
                request = request.query(&[("auction_id", round_id)]);
            }
            let envelope: PagedEnvelope<BidWire> =
                self.get_body(OPERATION, request).await.map_err(|error| {
                    normalize_not_found(error, OPERATION, "LISTING_NOT_FOUND", "Listing not found")
                })?;
            if !envelope.success {
                return Err(ClientError::UnsuccessfulEnvelope {
                    operation: OPERATION,
                });
            }
            let (Some(items), Some(meta)) = (envelope.data, envelope.pagination) else {
                return Err(ClientError::MissingData {
                    operation: OPERATION,
                });
            };
            let page_size = i64::from(BID_PAGE_SIZE);
            let total_pages = if meta.total == 0 {
                1
            } else {
                (meta.total - 1) / page_size + 1
            };
            if meta.page != i64::from(page)
                || meta.per_page != page_size
                || meta.total < 0
                || meta.total_pages != total_pages
                || items.len() > BID_PAGE_SIZE as usize
            {
                return Err(invalid_response(OPERATION));
            }
            total = meta.total;
            let full_page = items.len() == BID_PAGE_SIZE as usize;
            for bid in items {
                if !bid_is_valid(&bid, &listing, round_id) {
                    return Err(invalid_response(OPERATION));
                }
                // A bid placed while paging shifts the pages; keep each bid once.
                if seen.insert(bid.id) {
                    bids.push(bid);
                }
            }
            if !full_page || bids.len() as i64 >= total {
                break;
            }
        }

        let complete = bids.len() as i64 >= total;
        let (bids, bidder_count) = number_bidders(bids);
        Ok(ListingBidHistory {
            listing_id: listing.id,
            rounds,
            round_id,
            bid_count: total,
            bids,
            bidder_count,
            complete,
        })
    }

    /// A seller's rating statistics and a page of the ratings they received.
    pub async fn get_seller_ratings(
        &self,
        seller_id: &str,
        page: u32,
        limit: u32,
    ) -> Result<SellerRatings, ClientError> {
        const OPERATION: &str = "get_seller_ratings";
        validate_public_id_input(
            OPERATION,
            seller_id,
            is_user_id,
            "seller_id must be a public user ID",
        )?;
        validate_page_input(OPERATION, page, limit, MAX_RATING_RESULTS)?;
        let url = self.endpoint(OPERATION, &format!("ratings/user/{seller_id}/statistics"))?;
        let statistics: RatingStatistics = self
            .get_required_or_unavailable(
                OPERATION,
                self.http.get(url),
                "SELLER_NOT_FOUND",
                "Seller not found",
            )
            .await?;
        if statistics.user_id != seller_id || !statistics_are_valid(&statistics) {
            return Err(invalid_response(OPERATION));
        }

        let url = self.endpoint(OPERATION, &format!("ratings/public/users/{seller_id}"))?;
        let request = self
            .http
            .get(url)
            .query(&[("page", page), ("limit", limit)]);
        let feed: RatingFeedWire = self
            .get_required_or_unavailable(OPERATION, request, "SELLER_NOT_FOUND", "Seller not found")
            .await?;
        if feed.items.len() > limit as usize {
            return Err(invalid_response(OPERATION));
        }
        validate_pagination(
            OPERATION,
            &feed.pagination,
            i64::from(page),
            i64::from(limit),
            feed.items.len(),
        )?;
        for rating in &feed.items {
            if !(1..=5).contains(&rating.value)
                || rating.created_at < 0
                || rating
                    .rating_type
                    .as_deref()
                    .is_some_and(|value| !code(value))
                || !is_user_id(&rating.rater_id)
            {
                return Err(invalid_response(OPERATION));
            }
        }

        // Raters are named by their public username, looked up once each.
        let mut usernames: HashMap<String, Option<String>> = HashMap::new();
        for rating in &feed.items {
            if let Entry::Vacant(entry) = usernames.entry(rating.rater_id.clone()) {
                entry.insert(self.public_username(OPERATION, &rating.rater_id).await?);
            }
        }
        let ratings = feed
            .items
            .into_iter()
            .map(|rating| PublicRating {
                rater_username: usernames.get(&rating.rater_id).cloned().flatten(),
                value: rating.value,
                comment: rating.comment,
                created_at: rating.created_at,
                verified_transaction: rating.verified_transaction,
                rating_type: rating.rating_type,
            })
            .collect();
        Ok(SellerRatings {
            seller_id: statistics.user_id.clone(),
            statistics,
            ratings,
            pagination: feed.pagination,
        })
    }

    async fn public_username(
        &self,
        operation: &'static str,
        user_id: &str,
    ) -> Result<Option<String>, ClientError> {
        let url = self.endpoint(operation, &format!("profiles/{user_id}"))?;
        match self
            .get_required::<ProfileNameWire>(operation, self.http.get(url))
            .await
        {
            Ok(profile) if profile.user_id == user_id => Ok(profile.username),
            Ok(_) => Err(invalid_response(operation)),
            Err(ClientError::Api { status: 404, .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub async fn get_search_suggestions(
        &self,
        query: &str,
        market: Option<&str>,
        limit: u32,
    ) -> Result<SearchSuggestions, ClientError> {
        const OPERATION: &str = "get_search_suggestions";
        if query.chars().count() > MAX_SUGGESTION_QUERY_CHARS
            || !(1..=MAX_SUGGESTION_RESULTS).contains(&limit)
            || !optional_value_is_allowed(market, &MARKETS)
        {
            return Err(invalid_input(
                OPERATION,
                "query, market or limit is outside the allowed range",
            ));
        }
        let mut request = self
            .http
            .get(self.endpoint(OPERATION, "search/suggestions")?)
            .query(&[("q", query)])
            .query(&[("limit", limit)]);
        if let Some(market) = market {
            request = request.query(&[("market", market)]);
        }
        let suggestions: SearchSuggestions = self.get_required(OPERATION, request).await?;
        if suggestions.listings.len() > 50
            || suggestions.categories.len() > 50
            || suggestions.popular_searches.len() > 50
            || !suggestions
                .listings
                .iter()
                .all(|listing| is_listing_id(&listing.id))
            || !suggestions
                .categories
                .iter()
                .all(|category| category.id >= 1)
            || !suggestions
                .popular_searches
                .iter()
                .all(|search| search.count >= 0)
        {
            return Err(invalid_response(OPERATION));
        }
        Ok(suggestions)
    }

    /// Regions and municipalities of one country, with active listing counts.
    pub async fn get_locations(
        &self,
        country_code: &str,
        category_id: Option<i32>,
    ) -> Result<Vec<LocationRegion>, ClientError> {
        const OPERATION: &str = "get_locations";
        if !matches!(country_code, "SE" | "NO") || category_id.is_some_and(|value| value < 1) {
            return Err(invalid_input(
                OPERATION,
                "country must be SE or NO and category_id at least 1",
            ));
        }
        let mut request = self
            .http
            .get(self.endpoint(OPERATION, "locations")?)
            .query(&[("country_code", country_code)]);
        if let Some(category_id) = category_id {
            request = request.query(&[("category_id", category_id)]);
        }
        let locations: LocationsWire = self.get_required(OPERATION, request).await?;
        if !locations.regions.iter().all(|region| {
            region.country_code == country_code
                && !region.region_name.is_empty()
                && region.listing_count >= 0
                && region.cities.iter().all(|city| {
                    city.location_id >= 1
                        && !city.municipality_name.is_empty()
                        && city.listing_count >= 0
                })
        }) {
            return Err(invalid_response(OPERATION));
        }
        Ok(locations.regions)
    }

    /// Package sizes and the domestic price per carrier and size, as a buyer pays it.
    pub async fn get_shipping_rates(&self) -> Result<ShippingRates, ClientError> {
        const OPERATION: &str = "get_shipping_rates";
        let url = self.endpoint(OPERATION, "shipping/package-presets")?;
        let package_sizes: Vec<PackagePreset> =
            self.get_required(OPERATION, self.http.get(url)).await?;
        let url = self.endpoint(OPERATION, "shipping/domestic-rates")?;
        let rates: DomesticRatesWire = self.get_required(OPERATION, self.http.get(url)).await?;
        if package_sizes.len() > 20
            || rates.rates.len() > 50
            || !is_country_code(&rates.from_country)
            || !is_country_code(&rates.to_country)
            || !package_sizes.iter().all(|size| {
                code(&size.size_code)
                    && size.max_weight_grams >= 0
                    && size.max_length_mm >= 0
                    && size.max_width_mm >= 0
                    && size.max_height_mm >= 0
            })
            || !rates.rates.iter().all(|rate| {
                code(&rate.provider_code) && code(&rate.package_size) && money(&rate.amount)
            })
        {
            return Err(invalid_response(OPERATION));
        }
        Ok(ShippingRates {
            package_sizes,
            from_country: rates.from_country,
            to_country: rates.to_country,
            rates: rates.rates,
        })
    }

    /// A public listing's seller-answered questions, pins and newest chat messages.
    pub async fn get_listing_questions(
        &self,
        listing_id: &str,
        before_id: Option<i64>,
        limit: u32,
    ) -> Result<ListingQuestions, ClientError> {
        const OPERATION: &str = "get_listing_questions";
        if before_id.is_some_and(|value| value < 1) || !(1..=MAX_QUESTION_RESULTS).contains(&limit)
        {
            return Err(invalid_input(
                OPERATION,
                "before_id must be at least 1 and limit between 1 and 50",
            ));
        }
        let listing = self.get_public_listing(OPERATION, listing_id).await?;
        let url = self.endpoint(OPERATION, &format!("listings/{listing_id}/chat"))?;
        let mut request = self.http.get(url).query(&[("limit", limit)]);
        if let Some(before_id) = before_id {
            request = request.query(&[("before_id", before_id)]);
        }
        let feed: ChatFeedWire = self
            .get_required_or_unavailable(
                OPERATION,
                request,
                "LISTING_NOT_FOUND",
                "Listing not found",
            )
            .await?;
        let messages_are_valid = feed
            .messages
            .iter()
            .chain(&feed.pinned)
            .chain(
                feed.questions
                    .iter()
                    .flat_map(|pair| std::iter::once(&pair.question).chain(pair.answer.as_ref())),
            )
            .all(|message| chat_message_is_valid(message, listing_id));
        if feed.seller_id != listing.seller_id
            || feed.messages.len() > limit as usize
            || feed.next_before_id.is_some_and(|value| value < 1)
            || !messages_are_valid
        {
            return Err(invalid_response(OPERATION));
        }
        Ok(ListingQuestions {
            listing_id: listing.id,
            questions: feed
                .questions
                .into_iter()
                .map(|pair| ListingQuestion {
                    question: public_message(pair.question),
                    answer: pair.answer.map(public_message),
                })
                .collect(),
            pinned: feed.pinned.into_iter().map(public_message).collect(),
            messages: feed.messages.into_iter().map(public_message).collect(),
            next_before_id: feed.next_before_id,
        })
    }
}

/// Descriptive values are backend vocabulary that may grow, so they are checked by shape.
fn code(value: &str) -> bool {
    let mut bytes = value.bytes();
    value.len() <= 64
        && bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn optional_time(value: Option<i64>) -> bool {
    value.is_none_or(|value| value >= 0)
}

fn money(value: &Money) -> bool {
    money_is_valid(value, &value.currency_code)
}

fn is_country_code(value: &str) -> bool {
    value.len() == 2 && value.bytes().all(|byte| byte.is_ascii_uppercase())
}

fn is_brand_slug(value: &str) -> bool {
    value.len() <= MAX_BRAND_SLUG_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn brand_is_valid(brand: &Brand) -> bool {
    !brand.name.is_empty() && is_brand_slug(&brand.slug) && brand.listing_count >= 0
}

fn collection_not_found(operation: &'static str) -> ClientError {
    public_resource_unavailable(operation, "COLLECTION_NOT_FOUND", "Collection not found")
}

fn live_stream_not_found(operation: &'static str) -> ClientError {
    public_resource_unavailable(operation, "LIVE_STREAM_NOT_FOUND", "Live stream not found")
}

fn collection_is_public(collection: &Collection) -> bool {
    matches!(collection.status.as_str(), "active" | "ended")
}

fn validate_collection_page(
    mut page: CollectionPage,
    limit: u32,
    offset: u32,
    seller_id: Option<&str>,
) -> Result<CollectionPage, ClientError> {
    if page.limit != i64::from(limit)
        || page.offset != i64::from(offset)
        || page.items.len() > limit as usize
    {
        return Err(invalid_response(BROWSE_COLLECTIONS));
    }
    for collection in &mut page.items {
        if !collection_is_public(collection)
            || seller_id.is_some_and(|seller_id| collection.seller.id != seller_id)
        {
            return Err(public_resource_unavailable(
                BROWSE_COLLECTIONS,
                "COLLECTIONS_UNAVAILABLE",
                "Collections unavailable",
            ));
        }
        validate_collection(BROWSE_COLLECTIONS, collection)?;
    }
    Ok(page)
}

fn validate_collection(
    operation: &'static str,
    collection: &mut Collection,
) -> Result<(), ClientError> {
    // A collection names its market for display ("Swedish"); listings use the lowercase code.
    collection.market.make_ascii_lowercase();
    if !is_collection_id(&collection.id)
        || !is_user_id(&collection.seller.id)
        || !code(&collection.kind)
        || !MARKETS.contains(&collection.market.as_str())
        || !optional_time(collection.starts_at)
        || collection.ends_at < 0
        || collection
            .spotlight_listing_id
            .as_deref()
            .is_some_and(|listing_id| !is_listing_id(listing_id))
        || collection.public_item_count < 0
    {
        return Err(invalid_response(operation));
    }
    if let Some(live) = &collection.live {
        validate_live_session(operation, live)?;
        if live.showcase_id.as_deref() != Some(collection.id.as_str()) {
            return Err(invalid_response(operation));
        }
    }
    Ok(())
}

fn validate_live_session(
    operation: &'static str,
    session: &LiveSession,
) -> Result<(), ClientError> {
    // A draft stream is its seller's alone.
    if !matches!(
        session.status.as_str(),
        "scheduled" | "live" | "ended" | "cancelled"
    ) {
        return Err(live_stream_not_found(operation));
    }
    let target_is_valid = match (
        session.listing_id.as_deref(),
        session.showcase_id.as_deref(),
    ) {
        (None, None) => session.shop,
        (Some(listing_id), None) => !session.shop && is_listing_id(listing_id),
        (None, Some(collection_id)) => !session.shop && is_collection_id(collection_id),
        (Some(_), Some(_)) => false,
    };
    if !is_live_stream_id(&session.id)
        || !target_is_valid
        || session
            .stage_listing_id
            .as_deref()
            .is_some_and(|listing_id| !is_listing_id(listing_id))
        || session.scheduled_start_at < 0
        || !optional_time(session.started_at)
        || !optional_time(session.ended_at)
        || session.viewer_count < 0
        || session.reminder_count < 0
    {
        return Err(invalid_response(operation));
    }
    Ok(())
}

fn live_round_is_valid(round: &LiveRound) -> bool {
    is_listing_id(&round.listing_id)
        && code(&round.state)
        && code(&round.format)
        && round.duration_seconds >= 1
        && round.extend_seconds.is_none_or(|seconds| seconds >= 1)
        && money(&round.start_price)
        && round.opened_at >= 0
        && round.ends_at >= round.opened_at
        && optional_time(round.closed_at)
        && round
            .final_price
            .as_ref()
            .is_none_or(|price| money_is_valid(price, &round.start_price.currency_code))
}

fn is_public_card(card: &PublicListingWire) -> bool {
    card.approved && is_public_listing_status(&card.listing.status)
}

/// Keeps the listings anyone may see. A stream can show an item added during
/// the broadcast before it is listed; such a draft stays out, as everywhere
/// else in this profile.
fn public_cards(
    operation: &'static str,
    cards: Vec<PublicListingWire>,
) -> Result<Vec<ListingResponse>, ClientError> {
    let mut listings = Vec::with_capacity(cards.len());
    for card in cards.into_iter().filter(is_public_card) {
        validate_listing(operation, &card.listing)?;
        listings.push(card.listing);
    }
    Ok(listings)
}

fn round_is_valid(round: &AuctionRound, listing_id: &str) -> bool {
    round.id >= 1
        && round.listing_id == listing_id
        && round.round >= 1
        && code(&round.status)
        && round.outcome.as_deref().is_none_or(code)
        && round.start_time >= 0
        && round.end_time >= 0
        && round.bid_count >= 0
        && round.highest_amount.as_ref().is_none_or(money)
        && optional_time(round.closed_at)
}

fn bid_is_valid(bid: &BidWire, listing: &ListingResponse, round_id: Option<i64>) -> bool {
    bid.id >= 1
        && bid.listing_id == listing.id
        && is_user_id(&bid.bidder_id)
        && money_is_valid(&bid.price, &listing.currency)
        && bid.bid_time >= 0
        && code(&bid.status)
        && bid.auction_id.is_none_or(|auction_id| auction_id >= 1)
        && round_id.is_none_or(|round_id| bid.auction_id == Some(round_id))
}

/// Numbers bidders "Bidder 1", "Bidder 2"… in the order of each one's first
/// bid among the bids loaded, as the Budna app does. Bidder identities end here.
fn number_bidders(bids: Vec<BidWire>) -> (Vec<PublicBid>, usize) {
    let mut by_time: Vec<&BidWire> = bids.iter().collect();
    by_time.sort_by_key(|bid| (bid.bid_time, bid.id));
    let mut numbers: HashMap<&str, usize> = HashMap::new();
    for bid in by_time {
        let next = numbers.len() + 1;
        numbers.entry(bid.bidder_id.as_str()).or_insert(next);
    }
    let public = bids
        .iter()
        .map(|bid| PublicBid {
            bidder_number: numbers
                .get(bid.bidder_id.as_str())
                .copied()
                .unwrap_or_default(),
            amount: bid.price.clone(),
            bid_time: bid.bid_time,
            status: bid.status.clone(),
        })
        .collect();
    (public, numbers.len())
}

fn statistics_are_valid(statistics: &RatingStatistics) -> bool {
    let distribution_total = statistics
        .rating_distribution
        .iter()
        .try_fold(0_i64, |sum, count| {
            if *count < 0 {
                None
            } else {
                sum.checked_add(*count)
            }
        });
    statistics.total_ratings_received >= 0
        && statistics.total_ratings_given >= 0
        && (0..=statistics.total_ratings_received).contains(&statistics.total_comments_received)
        && statistics.average_rating_received.is_finite()
        && (0.0..=5.0).contains(&statistics.average_rating_received)
        && distribution_total == Some(statistics.total_ratings_received)
        && optional_time(statistics.latest_rating_received_at)
        && statistics
            .most_common_rating_received
            .is_none_or(|value| (1..=5).contains(&value))
        && statistics.positive_rating_percentage.is_finite()
        && (0.0..=100.0).contains(&statistics.positive_rating_percentage)
        && !statistics.reputation_level.is_empty()
        && statistics.reputation_level.len() <= 64
        && statistics
            .reputation_level
            .chars()
            .all(|character| character.is_ascii_alphabetic() || character == ' ')
}

fn chat_message_is_valid(message: &ChatMessageWire, listing_id: &str) -> bool {
    message.id >= 1
        && message.listing_id.as_deref() == Some(listing_id)
        && is_user_id(&message.user_id)
        && message.created_at >= 0
        && message
            .reply_to
            .as_ref()
            .is_none_or(|reply| reply.message_id >= 1 && is_user_id(&reply.user_id))
}

fn public_message(message: ChatMessageWire) -> ChatMessage {
    ChatMessage {
        username: message.username,
        from_seller: message.is_owner,
        body: message.body,
        created_at: message.created_at,
        reply_to: message.reply_to.map(|reply| ChatReply {
            username: reply.username,
            body: reply.body,
        }),
    }
}
