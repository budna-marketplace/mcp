use budna_mcp_client::{
    Brand, LISTING_ID_PATTERN, LocationRegion, SearchSuggestions, ShippingRates,
};
use budna_mcp_core::PublicUrlSettings;
use rmcp::schemars;
use serde::Serialize;

use super::{
    MAX_CODE_BYTES, MAX_NAME_BYTES, ProjectionBudget, UNTRUSTED_CONTENT_NOTICE,
    common::MoneyOutput,
    listings::{image_url, listing_url},
    taxonomy::CategoryTranslationsOutput,
};

const MAX_BRAND_RESULTS: usize = 100;
const MAX_SUGGESTED_LISTINGS: usize = 50;
const MAX_SUGGESTED_CATEGORIES: usize = 20;
const MAX_REGIONS: usize = 50;
const MAX_PACKAGE_SIZES: usize = 20;
const MAX_SHIPPING_RATES: usize = 50;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct BrandOutput {
    /// Pass as brand to browse_listings.
    #[schemars(length(max = 256))]
    pub name: String,
    #[schemars(length(max = 128))]
    pub slug: String,
    pub featured: bool,
    /// Active public listings of the brand.
    pub listing_count: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct BrandsOutput {
    pub content_notice: String,
    pub truncated: bool,
    #[schemars(length(max = 100))]
    pub brands: Vec<BrandOutput>,
    pub page: u32,
    pub limit: u32,
}

impl BrandsOutput {
    pub fn project(brands: Vec<Brand>, page: u32, limit: u32) -> Self {
        let mut budget = ProjectionBudget::default();
        let brands = budget
            .objects(brands, MAX_BRAND_RESULTS)
            .into_iter()
            .map(|brand| BrandOutput {
                name: budget.text(brand.name, MAX_NAME_BYTES),
                slug: budget.text(brand.slug, MAX_CODE_BYTES),
                featured: brand.is_featured,
                listing_count: brand.listing_count,
            })
            .collect();
        let truncated = budget.truncated();

        Self {
            content_notice: UNTRUSTED_CONTENT_NOTICE.to_owned(),
            truncated,
            brands,
            page,
            limit,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListingSuggestionOutput {
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub id: String,
    #[schemars(length(max = 544), regex(pattern = r"^https://"))]
    pub listing_url: String,
    #[schemars(length(max = 256))]
    pub title: String,
    #[schemars(length(max = 640), regex(pattern = r"^https://"))]
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CategorySuggestionOutput {
    pub id: i32,
    #[schemars(length(max = 256))]
    pub name: String,
    pub translations: Option<CategoryTranslationsOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct SearchSuggestionsOutput {
    pub content_notice: String,
    pub truncated: bool,
    #[schemars(length(max = 50))]
    pub listings: Vec<ListingSuggestionOutput>,
    #[schemars(length(max = 20))]
    pub categories: Vec<CategorySuggestionOutput>,
}

impl SearchSuggestionsOutput {
    pub fn from_with_public_urls(
        suggestions: SearchSuggestions,
        public_urls: &PublicUrlSettings,
    ) -> Self {
        let mut budget = ProjectionBudget::default();
        let listings = budget
            .objects(suggestions.listings, MAX_SUGGESTED_LISTINGS)
            .into_iter()
            .map(|listing| ListingSuggestionOutput {
                listing_url: listing_url(public_urls, &listing.id),
                image_url: listing
                    .image_id
                    .as_deref()
                    .and_then(|image_id| image_url(public_urls, &listing.id, image_id)),
                title: budget.text(listing.title, MAX_NAME_BYTES),
                id: listing.id,
            })
            .collect();
        let categories = budget
            .objects(suggestions.categories, MAX_SUGGESTED_CATEGORIES)
            .into_iter()
            .map(|category| CategorySuggestionOutput {
                id: category.id,
                name: budget.text(category.name, MAX_NAME_BYTES),
                translations: category.translations.map(|translations| {
                    CategoryTranslationsOutput::project(translations, &mut budget)
                }),
            })
            .collect();
        let truncated = budget.truncated();

        Self {
            content_notice: UNTRUSTED_CONTENT_NOTICE.to_owned(),
            truncated,
            listings,
            categories,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct MunicipalityOutput {
    /// Pass as location_id to search_listings.
    pub location_id: i32,
    /// Pass as location_municipality to search_listings.
    #[schemars(length(max = 256))]
    pub name: String,
    pub listing_count: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct RegionOutput {
    /// Pass as location_region to search_listings.
    #[schemars(length(max = 256))]
    pub name: String,
    pub listing_count: i64,
    #[schemars(length(max = 400))]
    pub municipalities: Vec<MunicipalityOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct LocationsOutput {
    pub content_notice: String,
    pub truncated: bool,
    #[schemars(length(min = 2, max = 2), regex(pattern = "^[A-Z]{2}$"))]
    pub country: String,
    #[schemars(length(max = 50))]
    pub regions: Vec<RegionOutput>,
}

impl LocationsOutput {
    pub fn project(country: &str, regions: Vec<LocationRegion>) -> Self {
        // A country has a few hundred municipalities; the object budget is sized for them.
        let mut budget = ProjectionBudget {
            remaining_objects: 512,
            ..ProjectionBudget::default()
        };
        let regions = budget
            .objects(regions, MAX_REGIONS)
            .into_iter()
            .map(|region| RegionOutput {
                name: budget.text(region.region_name, MAX_NAME_BYTES),
                listing_count: region.listing_count,
                municipalities: budget
                    .objects(region.cities, usize::MAX)
                    .into_iter()
                    .map(|city| MunicipalityOutput {
                        location_id: city.location_id,
                        name: budget.text(city.municipality_name, MAX_NAME_BYTES),
                        listing_count: city.listing_count,
                    })
                    .collect(),
            })
            .collect();
        let truncated = budget.truncated();

        Self {
            content_notice: UNTRUSTED_CONTENT_NOTICE.to_owned(),
            truncated,
            country: country.to_owned(),
            regions,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PackageSizeOutput {
    /// small, medium, large or extra_large, as listings name their package size.
    #[schemars(length(max = 128))]
    pub size: String,
    #[schemars(length(max = 256))]
    pub label: String,
    pub max_weight_grams: i32,
    pub max_length_mm: i32,
    pub max_width_mm: i32,
    pub max_height_mm: i32,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ShippingRateOutput {
    #[schemars(length(max = 128))]
    pub carrier: String,
    #[schemars(length(max = 128))]
    pub package_size: String,
    /// What the buyer pays.
    pub price: MoneyOutput,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ShippingRatesOutput {
    pub content_notice: String,
    pub truncated: bool,
    #[schemars(length(max = 20))]
    pub package_sizes: Vec<PackageSizeOutput>,
    #[schemars(length(min = 2, max = 2), regex(pattern = "^[A-Z]{2}$"))]
    pub from_country: String,
    #[schemars(length(min = 2, max = 2), regex(pattern = "^[A-Z]{2}$"))]
    pub to_country: String,
    #[schemars(length(max = 50))]
    pub rates: Vec<ShippingRateOutput>,
}

impl From<ShippingRates> for ShippingRatesOutput {
    fn from(shipping: ShippingRates) -> Self {
        let mut budget = ProjectionBudget::default();
        let package_sizes = budget
            .objects(shipping.package_sizes, MAX_PACKAGE_SIZES)
            .into_iter()
            .map(|size| PackageSizeOutput {
                size: budget.text(size.size_code, MAX_CODE_BYTES),
                label: budget.text(size.label, MAX_NAME_BYTES),
                max_weight_grams: size.max_weight_grams,
                max_length_mm: size.max_length_mm,
                max_width_mm: size.max_width_mm,
                max_height_mm: size.max_height_mm,
            })
            .collect();
        let rates = budget
            .objects(shipping.rates, MAX_SHIPPING_RATES)
            .into_iter()
            .map(|rate| ShippingRateOutput {
                carrier: budget.text(rate.provider_code, MAX_CODE_BYTES),
                package_size: budget.text(rate.package_size, MAX_CODE_BYTES),
                price: MoneyOutput::from(rate.amount),
            })
            .collect();
        let truncated = budget.truncated();

        Self {
            content_notice: UNTRUSTED_CONTENT_NOTICE.to_owned(),
            truncated,
            package_sizes,
            from_country: shipping.from_country,
            to_country: shipping.to_country,
            rates,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use budna_mcp_client::{
        CategorySuggestion, CategoryTranslations, ListingSuggestion, LocationCity, Money,
        PackagePreset, ShippingRate, TranslationMap,
    };

    use super::*;
    use crate::output::assert_within_mcp_result_budget;

    fn keys(value: &serde_json::Value) -> BTreeSet<&str> {
        value
            .as_object()
            .map(|object| object.keys().map(String::as_str).collect())
            .unwrap_or_else(|| panic!("expected an object"))
    }

    fn first<'a>(value: &'a serde_json::Value, pointer: &str) -> &'a serde_json::Value {
        value
            .pointer(pointer)
            .unwrap_or_else(|| panic!("missing {pointer}"))
    }

    #[test]
    fn brand_shapes_are_exact_and_bounded() {
        let brands = (0..150)
            .map(|number| Brand {
                name: format!("Brand {number} {}", "💡".repeat(100)),
                slug: format!("brand-{number}"),
                is_featured: number % 2 == 0,
                listing_count: number,
            })
            .collect();
        let output = BrandsOutput::project(brands, 1, 100);
        assert!(output.truncated);
        assert_eq!(output.brands.len(), MAX_BRAND_RESULTS);
        assert_within_mcp_result_budget(&output);
        let value = serde_json::to_value(&output)
            .unwrap_or_else(|error| panic!("brands should serialize: {error}"));
        assert_eq!(
            keys(&value),
            BTreeSet::from(["brands", "content_notice", "limit", "page", "truncated"])
        );
        assert_eq!(
            keys(first(&value, "/brands/0")),
            BTreeSet::from(["featured", "listing_count", "name", "slug"])
        );
    }

    #[test]
    fn suggestion_shapes_are_exact_and_link_only_valid_images() {
        let text = "💡".repeat(300);
        let translations = || CategoryTranslations {
            name: TranslationMap {
                en: text.clone(),
                sv: text.clone(),
                no: text.clone(),
            },
        };
        let suggestions = SearchSuggestions {
            listings: (0..60)
                .map(|number| ListingSuggestion {
                    id: format!("listing_00000000-0000-4000-8000-{number:012x}"),
                    title: text.clone(),
                    image_id: Some(if number == 0 {
                        "not-an-image".to_owned()
                    } else {
                        "123e4567-e89b-12d3-a456-426614174000".to_owned()
                    }),
                })
                .collect(),
            categories: (1..=30)
                .map(|id| CategorySuggestion {
                    id,
                    name: text.clone(),
                    translations: Some(translations()),
                })
                .collect(),
        };
        let output = SearchSuggestionsOutput::from_with_public_urls(
            suggestions,
            &PublicUrlSettings::default(),
        );
        assert!(output.truncated);
        assert_eq!(output.listings.len(), MAX_SUGGESTED_LISTINGS);
        assert!(output.listings[0].image_url.is_none());
        assert!(output.listings[1].image_url.is_some());
        assert_within_mcp_result_budget(&output);
        let value = serde_json::to_value(&output)
            .unwrap_or_else(|error| panic!("suggestions should serialize: {error}"));
        assert_eq!(
            keys(&value),
            BTreeSet::from(["categories", "content_notice", "listings", "truncated"])
        );
        assert_eq!(
            keys(first(&value, "/listings/0")),
            BTreeSet::from(["id", "image_url", "listing_url", "title"])
        );
        assert_eq!(
            keys(first(&value, "/categories/0")),
            BTreeSet::from(["id", "name", "translations"])
        );
    }

    #[test]
    fn a_whole_country_of_locations_fits_the_budget() {
        let regions = (0..21)
            .map(|region| LocationRegion {
                country_code: "NO".to_owned(),
                region_name: format!("Region {region}"),
                listing_count: 20,
                cities: (0..20)
                    .map(|city| LocationCity {
                        location_id: region * 100 + city + 1,
                        municipality_name: format!("Municipality {region}-{city}"),
                        listing_count: 1,
                    })
                    .collect(),
            })
            .collect();
        let output = LocationsOutput::project("NO", regions);
        assert!(!output.truncated);
        assert_eq!(
            output
                .regions
                .iter()
                .map(|region| region.municipalities.len())
                .sum::<usize>(),
            420
        );
        assert_within_mcp_result_budget(&output);
        let value = serde_json::to_value(&output)
            .unwrap_or_else(|error| panic!("locations should serialize: {error}"));
        assert_eq!(
            keys(&value),
            BTreeSet::from(["content_notice", "country", "regions", "truncated"])
        );
        assert_eq!(
            keys(first(&value, "/regions/0")),
            BTreeSet::from(["listing_count", "municipalities", "name"])
        );
        assert_eq!(
            keys(first(&value, "/regions/0/municipalities/0")),
            BTreeSet::from(["listing_count", "location_id", "name"])
        );
    }

    #[test]
    fn shipping_shapes_are_exact_and_bounded() {
        let output = ShippingRatesOutput::from(ShippingRates {
            package_sizes: (0..30)
                .map(|_| PackagePreset {
                    size_code: "small".to_owned(),
                    label: "Small".to_owned(),
                    max_weight_grams: 2_000,
                    max_length_mm: 600,
                    max_width_mm: 400,
                    max_height_mm: 100,
                })
                .collect(),
            from_country: "SE".to_owned(),
            to_country: "SE".to_owned(),
            rates: (0..60)
                .map(|_| ShippingRate {
                    provider_code: "postnord".to_owned(),
                    package_size: "small".to_owned(),
                    amount: Money {
                        amount: "59.00".to_owned(),
                        currency_code: "SEK".to_owned(),
                    },
                })
                .collect(),
        });
        assert!(output.truncated);
        assert_eq!(output.package_sizes.len(), MAX_PACKAGE_SIZES);
        assert_eq!(output.rates.len(), MAX_SHIPPING_RATES);
        assert_within_mcp_result_budget(&output);
        let value = serde_json::to_value(&output)
            .unwrap_or_else(|error| panic!("shipping should serialize: {error}"));
        assert_eq!(
            keys(&value),
            BTreeSet::from([
                "content_notice",
                "from_country",
                "package_sizes",
                "rates",
                "to_country",
                "truncated"
            ])
        );
        assert_eq!(
            keys(first(&value, "/package_sizes/0")),
            BTreeSet::from([
                "label",
                "max_height_mm",
                "max_length_mm",
                "max_weight_grams",
                "max_width_mm",
                "size"
            ])
        );
        assert_eq!(
            keys(first(&value, "/rates/0")),
            BTreeSet::from(["carrier", "package_size", "price"])
        );
    }
}
