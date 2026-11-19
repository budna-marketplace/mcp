use super::{BidCurrency, PackageSize, ShippingCarrier, actions::valid_outgoing_text};
use crate::{decimal_is_less_than_or_equal, is_exact_decimal};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DraftCondition {
    New,
    LikeNew,
    VeryGood,
    Good,
    Acceptable,
}
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DraftListingType {
    Auction,
    FixedPrice,
    AuctionFixedPrice,
    InStock,
}
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DraftMarket {
    Swedish,
    Norwegian,
}
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DraftVatChoice {
    VmbUsedGoods,
    #[serde(rename = "standard_25")]
    Standard25,
    #[serde(rename = "reduced_12")]
    Reduced12,
    #[serde(rename = "reduced_6")]
    Reduced6,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListingPrice {
    #[schemars(
        length(min = 1, max = 11),
        regex(pattern = r"^(0|[1-9][0-9]{0,7})(\.[0-9]{1,2})?$")
    )]
    pub amount: String,
    pub currency_code: BidCurrency,
}
impl ListingPrice {
    pub(super) fn valid(&self) -> bool {
        let (whole, fraction) = self.amount.split_once('.').unwrap_or((&self.amount, ""));
        is_exact_decimal(&self.amount)
            && (whole == "0" || !whole.starts_with('0'))
            && fraction.len() <= 2
            && decimal_is_less_than_or_equal(&self.amount, "99999999.99")
    }
}
pub(super) fn supplied<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
#[derive(Debug, Default, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DraftFieldChanges {
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(length(min = 1, max = 200))]
    pub title: Option<String>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(length(min = 1, max = 10000))]
    pub description: Option<String>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(range(min = 1))]
    pub category_id: Option<i32>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub condition: Option<DraftCondition>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub listing_type: Option<DraftListingType>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub market: Option<DraftMarket>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub currency: Option<BidCurrency>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub starting_price: Option<ListingPrice>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub bid_increment: Option<ListingPrice>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub seller_reserve: Option<ListingPrice>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub buy_now_price: Option<ListingPrice>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub custom_shipping_price: Option<ListingPrice>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(length(max = 20))]
    pub tags: Option<Vec<String>>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(range(min = 1))]
    pub start_time: Option<i64>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(range(min = 1))]
    pub end_time: Option<i64>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub package_size: Option<PackageSize>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(range(min = 1))]
    pub package_weight_grams: Option<i32>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(length(min = 1, max = 3))]
    pub shipping_provider_codes: Option<Vec<ShippingCarrier>>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub combined_shipping_enabled: Option<bool>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub allow_pickup: Option<bool>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(range(min = 1))]
    pub location_id: Option<i32>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub clear_custom_shipping_price: Option<bool>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub clear_carrier_package: Option<bool>,
    #[serde(
        default,
        deserialize_with = "supplied",
        skip_serializing_if = "Option::is_none"
    )]
    pub clear_shipping_provider_codes: Option<bool>,
}
#[derive(Debug, Deserialize)]
pub struct DraftFieldState {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category_id: Option<i32>,
    pub condition: Option<DraftCondition>,
    pub listing_type: Option<DraftListingType>,
    pub market: Option<DraftMarket>,
    pub currency: Option<BidCurrency>,
    pub starting_price: Option<crate::Money>,
    pub bid_increment: Option<crate::Money>,
    pub seller_reserve: Option<crate::Money>,
    pub buy_now_price: Option<crate::Money>,
    pub custom_shipping_price: Option<crate::Money>,
    pub tags: Option<Vec<String>>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub package_size: Option<PackageSize>,
    pub package_weight_grams: Option<i32>,
    pub shipping_provider_codes: Option<Vec<ShippingCarrier>>,
    pub combined_shipping_enabled: Option<bool>,
    pub allow_pickup: Option<bool>,
    pub location_id: Option<i32>,
}
impl DraftFieldChanges {
    pub(super) fn valid(&self) -> bool {
        self != &Self::default()
            && self
                .title
                .as_deref()
                .is_none_or(|s| valid_outgoing_text(s, 200, false))
            && self
                .description
                .as_deref()
                .is_none_or(|s| valid_outgoing_text(s, 10000, true))
            && self.category_id.is_none_or(|v| v > 0)
            && self.location_id.is_none_or(|v| v > 0)
            && self.start_time.is_none_or(|v| v > 0)
            && self.end_time.is_none_or(|v| v > 0)
            && self.tags.as_ref().is_none_or(|v| {
                v.len() <= 20
                    && v.iter().all(|s| valid_outgoing_text(s, 64, false))
                    && v.iter().collect::<HashSet<_>>().len() == v.len()
            })
            && self.package_weight_grams.is_none_or(|v| v > 0)
            && self.package_size.is_some() == self.package_weight_grams.is_some()
            && self.shipping_provider_codes.as_ref().is_none_or(|v| {
                (1..=3).contains(&v.len()) && v.iter().collect::<HashSet<_>>().len() == v.len()
            })
            && self
                .clear_shipping_provider_codes
                .is_none_or(|flag| flag && self.shipping_provider_codes.is_none())
            && self
                .clear_carrier_package
                .is_none_or(|flag| flag && self.package_size.is_none())
            && self
                .clear_custom_shipping_price
                .is_none_or(|flag| flag && self.custom_shipping_price.is_none())
            && [
                self.starting_price.as_ref(),
                self.bid_increment.as_ref(),
                self.seller_reserve.as_ref(),
                self.buy_now_price.as_ref(),
                self.custom_shipping_price.as_ref(),
            ]
            .into_iter()
            .flatten()
            .all(ListingPrice::valid)
    }
}
impl DraftFieldState {
    pub(super) fn valid(&self) -> bool {
        self.title.as_ref().is_none_or(|s| s.chars().count() <= 200)
            && self
                .description
                .as_ref()
                .is_none_or(|s| s.chars().count() <= 10000)
            && self
                .tags
                .as_ref()
                .is_none_or(|v| v.len() <= 20 && v.iter().all(|s| s.chars().count() <= 64))
            && self
                .shipping_provider_codes
                .as_ref()
                .is_none_or(|v| v.len() <= 3)
            && [
                self.starting_price.as_ref(),
                self.bid_increment.as_ref(),
                self.seller_reserve.as_ref(),
                self.buy_now_price.as_ref(),
                self.custom_shipping_price.as_ref(),
            ]
            .into_iter()
            .flatten()
            .all(|m| crate::money_is_valid(m, "SEK"))
    }
}
