use serde_json::{Value, json};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

use super::*;
use crate::ClientConfig;

fn listing_id(number: i64) -> String {
    format!("listing_00000000-0000-4000-8000-{number:012x}")
}

fn user_id(number: i64) -> String {
    format!("user_00000000-0000-4000-8000-{number:012x}")
}

fn collection_id(number: i64) -> String {
    format!("showcase_00000000-0000-4000-8000-{number:012x}")
}

fn live_id(number: i64) -> String {
    format!("live_00000000-0000-4000-8000-{number:012x}")
}

fn client(server: &MockServer) -> PublicApiClient {
    let config = ClientConfig::new(server.uri())
        .unwrap_or_else(|error| panic!("mock URL should be valid: {error}"));
    PublicApiClient::new(config).unwrap_or_else(|error| panic!("client should build: {error}"))
}

fn client_error<T>(result: Result<T, ClientError>, context: &str) -> ClientError {
    match result {
        Ok(_) => panic!("{context}"),
        Err(error) => error,
    }
}

fn ok(data: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({"success": true, "data": data}))
}

fn listing(id: i64, status: &str, approved: bool) -> Value {
    json!({
        "id": listing_id(id),
        "seller": {"id": user_id(42), "name": "Public seller", "username": "seller42"},
        "title": "Camera",
        "description": "A public listing",
        "category_id": 12,
        "condition": "good",
        "listing_type": "auction",
        "currency": "SEK",
        "market": "swedish",
        "auction": {
            "starting_price": {"amount": "100.00", "currency_code": "SEK"},
            "bid_increment": {"amount": "10.00", "currency_code": "SEK"},
            "current_bid": {"amount": "140.00", "currency_code": "SEK"},
            "reserve": {"met": false},
            "buy_now_price": null,
            "bid_count": 4,
            "start_time": 1_700_000_000_000_i64,
            "end_time": 1_800_000_000_000_i64
        },
        "fixed_price": null,
        "quantity": 1,
        "status": status,
        "views_count": 20,
        "approved": approved,
        "featured": false,
        "tags": ["camera"],
        "images": [{"id": "123e4567-e89b-12d3-a456-426614174000"}],
        "created_at": 1_700_000_000_000_i64,
        "updated_at": 1_700_000_000_100_i64,
        "shipping": {
            "custom_price": null,
            "package_size": "small",
            "package_weight_grams": 500,
            "provider_codes": null,
            "location": null,
            "allow_pickup": false
        },
        "buyer_protection": null
    })
}

fn seller() -> Value {
    json!({"id": user_id(42), "username": "seller42", "display_name": "Public seller", "image_id": null})
}

fn shop_session(id: i64, status: &str) -> Value {
    json!({
        "id": live_id(id),
        "status": status,
        "listing_id": null,
        "showcase_id": null,
        "shop": true,
        "title": "Friday drop",
        "description": "Cameras, live",
        "notify_followers": true,
        "stage_listing_id": listing_id(7),
        "scheduled_start_at": 1_800_000_000_000_i64,
        "started_at": null,
        "ended_at": null,
        "viewer_count": 12,
        "viewer_peak": 30,
        "reminder_count": 4,
        "playback": null
    })
}

fn collection(id: i64, status: &str) -> Value {
    json!({
        "id": collection_id(id),
        "seller": seller(),
        "kind": "auction",
        "status": status,
        "title": "Autumn cameras",
        "description": "Ten cameras, one end time",
        "banner_image_id": null,
        "starts_at": 1_700_000_000_000_i64,
        "ends_at": 1_800_000_000_000_i64,
        "market": "Swedish",
        "chat_enabled": true,
        "spotlight_listing_id": listing_id(7),
        "spotlight_set_at": null,
        "public_item_count": 1,
        "preview_image_ids": [],
        "is_boosted": true,
        "is_owner": false,
        "live": null,
        "created_at": 1_700_000_000_000_i64,
        "updated_at": 1_700_000_000_000_i64
    })
}

fn bid(id: i64, bidder: i64, amount: &str, bid_time: i64) -> Value {
    json!({
        "id": id,
        "listing_id": listing_id(7),
        "bidder_id": user_id(bidder),
        "price": {"amount": amount, "currency_code": "SEK"},
        "bid_time": bid_time,
        "status": "outbid",
        "listing_title": "Camera",
        "listing_image_id": null,
        "auction_id": 3
    })
}

fn bid_page(bids: Vec<Value>, page: i64, total: i64) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "success": true,
        "data": bids,
        "pagination": {
            "page": page,
            "per_page": 100,
            "total": total,
            "total_pages": (total - 1) / 100 + 1,
            "has_next": false,
            "has_previous": page > 1
        }
    }))
}

async fn mount_public_listing(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/listings/{}", listing_id(7))))
        .respond_with(ok(listing(7, "active", true)))
        .mount(server)
        .await;
}

#[test]
fn collection_and_live_stream_ids_require_their_own_prefix() {
    assert!(is_collection_id(&collection_id(1)));
    assert!(is_live_stream_id(&live_id(1)));
    assert!(!is_collection_id(&live_id(1)));
    assert!(!is_live_stream_id(&listing_id(1)));
    assert!(!is_collection_id(
        &collection_id(1).replace("-4000-", "-1000-")
    ));
}

#[tokio::test]
async fn collection_feeds_decode_public_collections_and_hide_drafts() {
    let server = MockServer::start().await;
    let mut session = shop_session(9, "live");
    session["shop"] = json!(false);
    session["showcase_id"] = json!(collection_id(1));
    session["stage_listing_id"] = Value::Null;
    let mut live = collection(1, "active");
    live["live"] = session;
    Mock::given(method("GET"))
        .and(path("/api/v1/showcases"))
        .and(query_param("limit", "2"))
        .and(query_param("offset", "2"))
        .and(query_param("market", "swedish"))
        .and(query_param("live", "true"))
        .respond_with(ok(
            json!({"items": [live], "limit": 2, "offset": 2, "has_more": false}),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let page = client(&server)
        .collection_feed(Some("swedish"), true, 2, 2)
        .await
        .unwrap_or_else(|error| panic!("the feed should decode: {error}"));
    assert_eq!(page.items[0].market, "swedish");
    assert_eq!(
        page.items[0]
            .live
            .as_ref()
            .and_then(|live| live.showcase_id.clone()),
        Some(collection_id(1))
    );

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/users/{}/showcases", user_id(42))))
        .respond_with(ok(json!({
            "items": [collection(2, "draft")],
            "limit": 10,
            "offset": 0,
            "has_more": false
        })))
        .expect(1)
        .mount(&server)
        .await;
    let error = client_error(
        client(&server)
            .seller_collections(&user_id(42), 1, 10)
            .await,
        "a draft collection must never be shown",
    );
    assert_eq!(error.status(), Some(404));
    assert_eq!(error.public_code(), Some("COLLECTIONS_UNAVAILABLE"));
}

#[tokio::test]
async fn collection_detail_reads_one_page_of_public_items() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/showcases/{}", collection_id(1))))
        .respond_with(ok(json!({
            "showcase": collection(1, "ended"),
            "items": {"items": [], "total": 0, "limit": 50, "offset": 0}
        })))
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/showcases/{}/items", collection_id(1))))
        .and(query_param("limit", "5"))
        .and(query_param("offset", "5"))
        .respond_with(ok(json!({
            "items": [{"listing": listing(7, "sold", true), "position": 6, "lot_number": 7, "aligned": true}],
            "total": 6,
            "limit": 5,
            "offset": 5
        })))
        .expect(1)
        .mount(&server)
        .await;

    let detail = client(&server)
        .get_collection(&collection_id(1), 2, 5)
        .await
        .unwrap_or_else(|error| panic!("the collection should decode: {error}"));
    assert_eq!(detail.collection.status, "ended");
    assert_eq!(detail.total, 6);
    assert_eq!(detail.items[0].lot_number, Some(7));
    assert_eq!(detail.items[0].listing.id, listing_id(7));

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/showcases/{}/items", collection_id(1))))
        .and(query_param("offset", "0"))
        .respond_with(ok(json!({
            "items": [{"listing": listing(8, "active", false), "position": 0, "lot_number": null, "aligned": false}],
            "total": 1,
            "limit": 10,
            "offset": 0
        })))
        .expect(1)
        .mount(&server)
        .await;
    let error = client_error(
        client(&server)
            .get_collection(&collection_id(1), 1, 10)
            .await,
        "an unapproved item must fail closed",
    );
    assert_eq!(error.public_code(), Some("COLLECTION_NOT_FOUND"));
}

#[tokio::test]
async fn live_streams_leave_out_unlisted_items_and_check_rounds() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/live-sessions/upcoming"))
        .and(query_param("limit", "5"))
        .respond_with(ok(json!([{
            "session": shop_session(11, "scheduled"),
            "seller": seller(),
            "stage": null,
            "preview_items": [listing(8, "active", true), listing(10, "draft", false)]
        }])))
        .expect(1)
        .mount(&server)
        .await;
    let shops = client(&server)
        .live_streams(true, 5)
        .await
        .unwrap_or_else(|error| panic!("upcoming streams should decode: {error}"));
    assert_eq!(shops[0].preview_items.len(), 1);
    assert_eq!(shops[0].preview_items[0].id, listing_id(8));

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/live-sessions/{}", live_id(9))))
        .respond_with(ok(shop_session(9, "live")))
        .expect(2)
        .mount(&server)
        .await;
    let round = json!({
        "id": 3,
        "listing_id": listing_id(7),
        "state": "open",
        "format": "extend",
        "duration_seconds": 30,
        "extend_seconds": 5,
        "start_price": {"amount": "100", "currency_code": "SEK"},
        "opened_at": 1_800_000_000_000_i64,
        "ends_at": 1_800_000_030_000_i64,
        "closed_at": null,
        "final_price": null
    });
    let stage = |round: Value| {
        json!({
            "session": shop_session(9, "live"),
            "seller": seller(),
            "stage_revision": 4,
            "stage": listing(7, "active", true),
            "round": round,
            "items": [listing(8, "active", true), listing(10, "draft", false)],
            "recent": []
        })
    };
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/live-sessions/{}/stage", live_id(9))))
        .respond_with(ok(stage(round.clone())))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    let stream = client(&server)
        .get_live_stream(&live_id(9))
        .await
        .unwrap_or_else(|error| panic!("the stream should decode: {error}"));
    let live_stage = stream
        .stage
        .unwrap_or_else(|| panic!("a shop stream should have a stage"));
    assert_eq!(live_stage.items.len(), 1);
    assert_eq!(live_stage.items[0].id, listing_id(8));
    assert_eq!(
        live_stage.on_stage.map(|listing| listing.id),
        Some(listing_id(7))
    );
    assert_eq!(
        live_stage.round.map(|round| round.state),
        Some("open".to_owned())
    );

    let mut mixed_currency = round;
    mixed_currency["final_price"] = json!({"amount": "120", "currency_code": "NOK"});
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/live-sessions/{}/stage", live_id(9))))
        .respond_with(ok(stage(mixed_currency)))
        .expect(1)
        .mount(&server)
        .await;
    let error = client_error(
        client(&server).get_live_stream(&live_id(9)).await,
        "a round in two currencies must be rejected",
    );
    assert_eq!(error.public_code(), Some("BUDNA_API_INVALID_RESPONSE"));

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/live-sessions/{}", live_id(12))))
        .respond_with(ok(shop_session(12, "draft")))
        .expect(1)
        .mount(&server)
        .await;
    let error = client_error(
        client(&server).get_live_stream(&live_id(12)).await,
        "a draft stream must stay hidden",
    );
    assert_eq!(error.public_code(), Some("LIVE_STREAM_NOT_FOUND"));
}

#[tokio::test]
async fn bid_history_numbers_bidders_by_first_bid_and_marks_the_viewers_own() {
    let server = MockServer::start().await;
    mount_public_listing(&server).await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/listings/{}/auctions", listing_id(7))))
        .respond_with(ok(json!([{
            "id": 3,
            "listing_id": listing_id(7),
            "round": 2,
            "status": "active",
            "outcome": null,
            "start_time": 1_700_000_000_000_i64,
            "end_time": 1_800_000_000_000_i64,
            "bid_count": 4,
            "highest_amount": {"amount": "140", "currency_code": "SEK"},
            "closed_at": null
        }])))
        .mount(&server)
        .await;
    // Newest first, as the API pages them: 77 bid first, then 55, then both again.
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/bids/listing/{}", listing_id(7))))
        .and(query_param("page", "1"))
        .and(query_param("limit", "100"))
        .and(query_param("auction_id", "3"))
        .respond_with(bid_page(
            vec![
                bid(4, 55, "140", 4_000),
                bid(3, 77, "130", 3_000),
                bid(2, 55, "120", 2_000),
                bid(1, 77, "110", 1_000),
            ],
            1,
            4,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let viewer = user_id(55);
    let history = client(&server)
        .get_listing_bid_history(&listing_id(7), Some(3), Some(&viewer))
        .await
        .unwrap_or_else(|error| panic!("bid history should decode: {error}"));
    assert_eq!(
        history
            .bids
            .iter()
            .map(|bid| (bid.bidder_number, bid.own))
            .collect::<Vec<_>>(),
        [(2, true), (1, false), (2, true), (1, false)],
        "the viewer keeps their number, as in the app"
    );
    assert_eq!(history.bidder_count, 2);
    assert_eq!(history.bid_count, 4);
    assert!(history.complete);
    assert_eq!(history.rounds[0].round, 2);
    let rendered = format!("{history:?}");
    assert!(!rendered.contains("user_"));

    let error = client_error(
        client(&server)
            .get_listing_bid_history(&listing_id(7), Some(99), None)
            .await,
        "an unknown round must not be read",
    );
    assert_eq!(error.public_code(), Some("AUCTION_ROUND_NOT_FOUND"));
}

#[tokio::test]
async fn bid_history_pages_through_a_round_and_keeps_each_bid_once() {
    let server = MockServer::start().await;
    mount_public_listing(&server).await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/listings/{}/auctions", listing_id(7))))
        .respond_with(ok(json!([])))
        .mount(&server)
        .await;
    let first_page = (2..=101)
        .rev()
        .map(|id| bid(id, 1_000 + id % 3, "100", id * 1_000))
        .collect();
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/bids/listing/{}", listing_id(7))))
        .and(query_param("page", "1"))
        .respond_with(bid_page(first_page, 1, 101))
        .expect(1)
        .mount(&server)
        .await;
    // A bid placed between the two reads repeats bid 2 on the next page.
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/bids/listing/{}", listing_id(7))))
        .and(query_param("page", "2"))
        .respond_with(bid_page(
            vec![bid(2, 1_002, "100", 2_000), bid(1, 1_001, "100", 1_000)],
            2,
            101,
        ))
        .expect(1)
        .mount(&server)
        .await;

    let history = client(&server)
        .get_listing_bid_history(&listing_id(7), None, None)
        .await
        .unwrap_or_else(|error| panic!("both pages should decode: {error}"));
    assert_eq!(history.bids.len(), 101);
    assert!(history.complete);
    assert_eq!(history.bidder_count, 3);
    assert_eq!(
        history.bids.last().map(|bid| bid.bidder_number),
        Some(1),
        "the earliest bid's bidder is Bidder 1"
    );
    assert!(history.bids.iter().all(|bid| !bid.own));
}

#[tokio::test]
async fn bidders_who_bid_at_the_same_moment_keep_the_loaded_order() {
    let server = MockServer::start().await;
    mount_public_listing(&server).await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/listings/{}/auctions", listing_id(7))))
        .respond_with(ok(json!([])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/bids/listing/{}", listing_id(7))))
        .respond_with(bid_page(
            vec![bid(5, 900, "100", 1_000), bid(4, 901, "90", 1_000)],
            1,
            2,
        ))
        .mount(&server)
        .await;

    let history = client(&server)
        .get_listing_bid_history(&listing_id(7), None, None)
        .await
        .unwrap_or_else(|error| panic!("bid history should decode: {error}"));
    assert_eq!(
        history
            .bids
            .iter()
            .map(|bid| bid.bidder_number)
            .collect::<Vec<_>>(),
        [1, 2]
    );
}

fn statistics(user: &str, distribution: [i64; 5]) -> Value {
    json!({
        "user_id": user,
        "total_ratings_received": 3,
        "total_ratings_given": 1,
        "average_rating_received": 4.0,
        "average_rating_given": 5.0,
        "rating_distribution": distribution,
        "total_comments_received": 2,
        "total_comments_given": 0,
        "latest_rating_received_at": 1_700_000_000_000_i64,
        "latest_rating_given_at": null,
        "reputation_level": "Very Good",
        "most_common_rating_received": 3,
        "positive_rating_percentage": 66.67,
        "is_active_rater": false,
        "is_trusted_member": false,
        "has_ratings": true
    })
}

fn rating(value: i32, rater: i64) -> Value {
    json!({
        "value": value,
        "comment": "Smooth trade",
        "created_at": 1_700_000_000_000_i64,
        "verified_transaction": true,
        "rating_type": "buyer_to_seller",
        "rater_id": user_id(rater)
    })
}

#[tokio::test]
async fn seller_ratings_name_raters_only_by_public_username() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/ratings/user/{}/statistics",
            user_id(42)
        )))
        .respond_with(ok(statistics(&user_id(42), [0, 0, 1, 1, 1])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/ratings/public/users/{}",
            user_id(42)
        )))
        .and(query_param("page", "1"))
        .and(query_param("limit", "10"))
        .respond_with(ok(json!({
            "items": [rating(5, 7), rating(4, 7), rating(3, 8)],
            "pagination": {"page": 1, "limit": 10, "total": 3, "total_pages": 1}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/profiles/{}", user_id(7))))
        .respond_with(ok(
            json!({"user_id": user_id(7), "username": "buyer7", "display_name": "Private Name"}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/profiles/{}", user_id(8))))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&server)
        .await;

    let ratings = client(&server)
        .get_seller_ratings(&user_id(42), 1, 10)
        .await
        .unwrap_or_else(|error| panic!("ratings should decode: {error}"));
    assert_eq!(
        ratings
            .ratings
            .iter()
            .map(|rating| rating.rater_username.as_deref())
            .collect::<Vec<_>>(),
        [Some("buyer7"), Some("buyer7"), None]
    );
    assert_eq!(ratings.statistics.rating_distribution, [0, 0, 1, 1, 1]);
    let rendered = format!("{ratings:?}");
    assert!(!rendered.contains(&user_id(7)));
    assert!(!rendered.contains(&user_id(8)));
    assert!(!rendered.contains("Private Name"));

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/ratings/user/{}/statistics",
            user_id(42)
        )))
        .respond_with(ok(statistics(&user_id(42), [0, 0, 0, 1, 1])))
        .expect(1)
        .mount(&server)
        .await;
    let error = client_error(
        client(&server)
            .get_seller_ratings(&user_id(42), 1, 10)
            .await,
        "a distribution that disagrees with the total must be rejected",
    );
    assert_eq!(error.public_code(), Some("BUDNA_API_INVALID_RESPONSE"));
}

#[tokio::test]
async fn brands_suggestions_locations_and_shipping_decode_their_contracts() {
    let server = MockServer::start().await;
    let brand = |name: &str, slug: &str| {
        json!({
            "id": 1, "name": name, "slug": slug, "logo_url": null, "logo_url_light": null,
            "description": null, "website_url": null, "is_verified": false, "is_featured": true,
            "listing_count": 4, "created_at": 0, "updated_at": 0
        })
    };
    Mock::given(method("GET"))
        .and(path("/api/v1/brands"))
        .and(query_param("page", "1"))
        .and(query_param("limit", "20"))
        .and(query_param("featured", "true"))
        .respond_with(ok(json!([brand("Rolex", "rolex")])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/brands/slug/patek-philippe"))
        .respond_with(ok(
            json!({"brand": brand("Patek Philippe", "patek-philippe")}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/search/suggestions"))
        .and(query_param("q", "cam"))
        .and(query_param("limit", "6"))
        .and(query_param("market", "swedish"))
        .respond_with(ok(json!({
            "listings": [{
                "id": listing_id(7), "title": "Camera", "price": "120", "currency": "SEK",
                "image_id": "123e4567-e89b-12d3-a456-426614174000"
            }],
            "categories": [{"id": 12, "name": "Cameras", "listing_count": null, "translations": null}],
            "popular_searches": [{"term": "camera", "count": 9}]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/locations"))
        .and(query_param("country_code", "NO"))
        .and(query_param("category_id", "12"))
        .respond_with(ok(json!({"regions": [{
            "country_code": "NO", "region_name": "Oslo", "listing_count": 3,
            "cities": [{"location_id": 301, "municipality_name": "Oslo", "listing_count": 3}]
        }]})))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/shipping/package-presets"))
        .respond_with(ok(json!([{
            "id": 1, "size_code": "small", "label": "Small", "max_weight_grams": 2_000,
            "max_length_mm": 600, "max_width_mm": 400, "max_height_mm": 100, "sort_order": 1
        }])))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/shipping/domestic-rates"))
        .respond_with(ok(json!({
            "from_country": "SE",
            "to_country": "SE",
            "rates": [{"provider_code": "postnord", "package_size": "small", "amount": {"amount": "59.00", "currency_code": "SEK"}}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let api = client(&server);
    let brands = api
        .get_brands(true, 1, 20)
        .await
        .unwrap_or_else(|error| panic!("brands should decode: {error}"));
    assert_eq!(brands[0].name, "Rolex");
    let brand = api
        .get_brand("patek-philippe")
        .await
        .unwrap_or_else(|error| panic!("a brand should decode: {error}"));
    assert_eq!(brand.listing_count, 4);
    let suggestions = api
        .get_search_suggestions("cam", Some("swedish"), 6)
        .await
        .unwrap_or_else(|error| panic!("suggestions should decode: {error}"));
    assert_eq!(suggestions.listings[0].id, listing_id(7));
    assert_eq!(suggestions.categories[0].id, 12);
    let regions = api
        .get_locations("NO", Some(12))
        .await
        .unwrap_or_else(|error| panic!("locations should decode: {error}"));
    assert_eq!(regions[0].cities[0].location_id, 301);
    let shipping = api
        .get_shipping_rates()
        .await
        .unwrap_or_else(|error| panic!("shipping rates should decode: {error}"));
    assert_eq!(shipping.package_sizes[0].size_code, "small");
    assert_eq!(shipping.rates[0].amount.amount, "59.00");
}

#[tokio::test]
async fn listing_feeds_send_brand_names_as_one_encoded_segment() {
    let server = MockServer::start().await;
    let empty =
        json!({"items": [], "pagination": {"page": 1, "limit": 10, "total": 0, "total_pages": 1}});
    Mock::given(method("GET"))
        .and(path("/api/v1/listings/brand/AC%2FDC%20Gear"))
        .and(query_param("page", "1"))
        .and(query_param("limit", "10"))
        .respond_with(ok(empty.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/listings/featured"))
        .and(query_param("page", "1"))
        .and(query_param("limit", "10"))
        .respond_with(ok(empty))
        .expect(1)
        .mount(&server)
        .await;

    let api = client(&server);
    let brand = api
        .brand_listings("AC/DC Gear", 1, 10)
        .await
        .unwrap_or_else(|error| panic!("brand listings should decode: {error}"));
    assert!(brand.items.is_empty());
    let featured = api
        .featured_listings(1, 10)
        .await
        .unwrap_or_else(|error| panic!("featured listings should decode: {error}"));
    assert!(featured.items.is_empty());
}

fn message(id: i64, author: i64, owner: bool, body: &str) -> Value {
    json!({
        "id": id,
        "listing_id": listing_id(7),
        "showcase_id": null,
        "live_session_id": null,
        "user_id": user_id(author),
        "username": format!("member{author}"),
        "display_name": "Private Name",
        "image_id": null,
        "is_owner": owner,
        "pinned": false,
        "question": !owner,
        "reply_to": null,
        "image_ids": [],
        "body": body,
        "created_at": 1_700_000_000_000_i64,
        "reactions": [{"emoji": "+1", "count": 1, "user_ids": [user_id(99)]}]
    })
}

#[tokio::test]
async fn listing_questions_keep_public_usernames_and_drop_identities() {
    let server = MockServer::start().await;
    mount_public_listing(&server).await;
    let mut answer = message(3, 42, true, "Yes, it works");
    answer["reply_to"] = json!({
        "message_id": 2,
        "user_id": user_id(5),
        "username": "member5",
        "display_name": "Private Name",
        "body": "Does it work?"
    });
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/listings/{}/chat", listing_id(7))))
        .and(query_param("limit", "20"))
        .respond_with(ok(json!({
            "seller_id": user_id(42),
            "messages": [answer.clone(), message(2, 5, false, "Does it work?")],
            "pinned": [],
            "bids": [{"id": 9, "price": {"amount": "120", "currency_code": "SEK"}, "bid_time": 1, "reactions": [], "listing_id": null}],
            "questions": [{"question": message(2, 5, false, "Does it work?"), "answer": answer}],
            "viewer_muted": null,
            "viewer_capabilities": null,
            "next_before_id": 2,
            "send_cooldown_seconds": 5
        })))
        .expect(1)
        .mount(&server)
        .await;

    let questions = client(&server)
        .get_listing_questions(&listing_id(7), None, 20)
        .await
        .unwrap_or_else(|error| panic!("questions should decode: {error}"));
    assert_eq!(
        questions.questions[0]
            .answer
            .as_ref()
            .map(|answer| answer.from_seller),
        Some(true)
    );
    assert_eq!(
        questions.messages[0]
            .reply_to
            .as_ref()
            .and_then(|reply| reply.username.as_deref()),
        Some("member5")
    );
    assert_eq!(questions.next_before_id, Some(2));
    let rendered = format!("{questions:?}");
    assert!(!rendered.contains("user_"));
    assert!(!rendered.contains("Private Name"));
}

#[tokio::test]
async fn invalid_inputs_never_reach_the_network() {
    let server = MockServer::start().await;
    let api = client(&server);
    let errors = [
        client_error(api.get_collection("showcase_bad", 1, 10).await, "bad id"),
        client_error(
            api.get_collection(&collection_id(1), 1, 51).await,
            "oversized page",
        ),
        client_error(
            api.collection_feed(Some("danish"), false, 1, 10).await,
            "unknown market",
        ),
        client_error(api.get_live_stream(&listing_id(7)).await, "wrong id kind"),
        client_error(api.live_streams(false, 21).await, "oversized list"),
        client_error(api.get_brand("Rolex").await, "uppercase slug"),
        client_error(api.brand_listings("\u{0}", 1, 10).await, "control brand"),
        client_error(
            api.get_listing_bid_history(&listing_id(7), Some(0), None)
                .await,
            "zero round",
        ),
        client_error(
            api.get_seller_ratings(&listing_id(7), 1, 10).await,
            "wrong seller id kind",
        ),
        client_error(
            api.get_search_suggestions(&"x".repeat(201), None, 10).await,
            "oversized query",
        ),
        client_error(
            api.get_search_suggestions("  ", None, 10).await,
            "blank query",
        ),
        client_error(api.get_locations("DK", None).await, "unknown country"),
        client_error(
            api.get_listing_questions(&listing_id(7), Some(0), 10).await,
            "zero cursor",
        ),
    ];
    for error in errors {
        assert_eq!(error.public_code(), Some("INVALID_REQUEST"));
    }
    assert!(
        server
            .received_requests()
            .await
            .unwrap_or_else(|| panic!("requests should be retained"))
            .is_empty()
    );
}
