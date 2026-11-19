use super::InputError;
use budna_mcp_client::{SavedSearchChanges, SearchCriteriaPatch, SearchPrice};
use rmcp::schemars;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchPriceParam {
    #[schemars(length(min = 1, max = 128))]
    amount: String,
    currency_code: SearchCurrency,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
enum SearchCurrency {
    #[serde(rename = "SEK")]
    Sek,
}

impl From<SearchPriceParam> for SearchPrice {
    fn from(value: SearchPriceParam) -> Self {
        let currency_code = match value.currency_code {
            SearchCurrency::Sek => "SEK",
        }
        .to_owned();
        Self {
            amount: value.amount,
            currency_code,
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct FilterChange {
    #[schemars(length(min = 1, max = 64))]
    field: String,
    /// A string sets this one filter. Null removes the named filter on an edit.
    #[serde(deserialize_with = "Option::deserialize")]
    value: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchCriteriaParams {
    #[serde(default, deserialize_with = "present")]
    text: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    category_id: Option<Option<i32>>,
    #[serde(default, deserialize_with = "present")]
    market: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    min_price: Option<Option<SearchPriceParam>>,
    #[serde(default, deserialize_with = "present")]
    max_price: Option<Option<SearchPriceParam>>,
    #[serde(default, deserialize_with = "present")]
    condition: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    listing_type: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    location_id: Option<Option<i32>>,
    #[serde(default, deserialize_with = "present")]
    location_region: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    location_municipality: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    allow_pickup: Option<Option<bool>>,
    #[serde(default, deserialize_with = "present")]
    search_mode: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    sort_by: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    sort_order: Option<Option<String>>,
    #[serde(default, deserialize_with = "present")]
    #[schemars(length(max = 20))]
    filters: Option<Vec<FilterChange>>,
}

impl SearchCriteriaParams {
    pub(super) fn into_patch(self) -> Result<SearchCriteriaPatch, InputError> {
        let mut filters = BTreeMap::new();
        for entry in self.filters.unwrap_or_default() {
            if filters.insert(entry.field, entry.value).is_some() {
                return Err(InputError::new("Each search filter can appear only once"));
            }
        }
        Ok(SearchCriteriaPatch {
            text: self.text,
            category_id: self.category_id,
            market: self.market,
            min_price: self.min_price.map(|v| v.map(Into::into)),
            max_price: self.max_price.map(|v| v.map(Into::into)),
            condition: self.condition,
            listing_type: self.listing_type,
            location_id: self.location_id,
            location_region: self.location_region,
            location_municipality: self.location_municipality,
            allow_pickup: self.allow_pickup,
            search_mode: self.search_mode,
            sort_by: self.sort_by,
            sort_order: self.sort_order,
            filters,
        })
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateSearchParams {
    #[schemars(length(min = 1, max = 255))]
    pub name: String,
    /// Search criteria to save. Use public taxonomy reads for filter values.
    pub criteria: SearchCriteriaParams,
    #[serde(default, deserialize_with = "present")]
    pub notification_frequency: Option<Frequency>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Frequency {
    Immediate,
    Daily,
    Weekly,
}

impl Frequency {
    fn wire(self) -> String {
        match self {
            Self::Immediate => "immediate",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
        }
        .into()
    }
}

impl CreateSearchParams {
    pub fn changes(self) -> Result<SavedSearchChanges, InputError> {
        Ok(SavedSearchChanges {
            name: Some(self.name),
            criteria: Some(self.criteria.into_patch()?),
            notification_frequency: self.notification_frequency.map(Frequency::wire),
            active: None,
        })
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateSearchParams {
    #[schemars(range(min = 1))]
    pub search_id: i64,
    #[schemars(range(min = 1))]
    pub expected_updated_at: i64,
    #[serde(default, deserialize_with = "present")]
    #[schemars(length(min = 1, max = 255))]
    pub name: Option<String>,
    /// Omit a criterion to keep it. Null clears one scalar. Filter changes address individual names.
    #[serde(default, deserialize_with = "present")]
    pub criteria: Option<SearchCriteriaParams>,
    #[serde(default, deserialize_with = "present")]
    pub notification_frequency: Option<Frequency>,
    #[serde(default, deserialize_with = "present")]
    pub active: Option<bool>,
}

impl UpdateSearchParams {
    pub fn changes(self) -> Result<SavedSearchChanges, InputError> {
        Ok(SavedSearchChanges {
            name: self.name,
            criteria: self
                .criteria
                .map(SearchCriteriaParams::into_patch)
                .transpose()?,
            notification_frequency: self.notification_frequency.map(Frequency::wire),
            active: self.active,
        })
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeleteSearchParams {
    #[schemars(range(min = 1))]
    pub search_id: i64,
    #[schemars(range(min = 1))]
    pub expected_updated_at: i64,
}
