use super::{
    AccountApiClient, ClientError, DelegatedAccess, DraftCondition, DraftListingType, DraftMarket,
    LISTINGS_READ_SCOPE, actions::valid_action_text, invalid_input, invalid_response,
    shipping::valid_revision,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CreationFilterType {
    Text,
    Select,
    CountrySelect,
    MultiSelect,
    RangeSingle,
    RangeDual,
    Boolean,
    DateRange,
    RatingStars,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CreationValueKind {
    Text,
    Number,
    Options,
    Boolean,
    Date,
}
#[derive(Debug, Deserialize)]
pub struct CreationField {
    pub filter_definition_id: i32,
    pub name: String,
    pub label: String,
    pub filter_type: CreationFilterType,
    pub value_kind: CreationValueKind,
    pub required_on_publish: bool,
    pub min_length: Option<i32>,
    pub max_length: Option<i32>,
    pub pattern: Option<String>,
    pub minimum: Option<String>,
    pub maximum: Option<String>,
    pub step: Option<String>,
    pub unit: Option<String>,
    pub max_fraction_digits: Option<u8>,
    pub maximum_exclusive_magnitude: Option<String>,
    pub max_choices: Option<usize>,
    pub active_option_count: usize,
    pub supported_in_mcp: bool,
}
#[derive(Debug, Deserialize)]
pub struct CreationRules {
    pub category_id: i32,
    pub category_name: String,
    pub market: DraftMarket,
    pub mcp_price_currency: String,
    pub rules_revision: String,
    pub listing_types: Vec<DraftListingType>,
    pub conditions: Vec<DraftCondition>,
    pub attributes_max: usize,
    pub replacement: String,
    pub required_for: String,
    pub value_dependencies: String,
    pub total_fields: usize,
    pub fields: Vec<CreationField>,
    pub next_after: Option<i32>,
}
#[derive(Debug, Deserialize)]
pub struct CreationOption {
    pub option_id: i32,
    pub value: Option<String>,
    pub label: Option<String>,
    pub supported_in_mcp: bool,
}
#[derive(Debug, Deserialize)]
pub struct CreationOptions {
    pub category_id: i32,
    pub market: DraftMarket,
    pub filter_definition_id: i32,
    pub rules_revision: String,
    pub total_matches: usize,
    pub options: Vec<CreationOption>,
    pub next_after: Option<i32>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreationRulesQuery {
    #[schemars(range(min = 1))]
    pub category_id: i32,
    pub market: DraftMarket,
    #[schemars(range(min = 1))]
    pub after: Option<i32>,
    #[schemars(range(min = 1, max = 25))]
    pub limit: Option<u8>,
    #[schemars(length(min = 43, max = 43))]
    pub expected_rules_revision: Option<String>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CreationOptionsQuery {
    #[schemars(range(min = 1))]
    pub category_id: i32,
    pub market: DraftMarket,
    #[schemars(range(min = 1))]
    pub filter_definition_id: i32,
    #[schemars(range(min = 1))]
    pub after: Option<i32>,
    #[schemars(range(min = 1, max = 25))]
    pub limit: Option<u8>,
    #[schemars(length(min = 43, max = 43))]
    pub expected_rules_revision: String,
    #[schemars(length(min = 1, max = 80))]
    pub search: Option<String>,
}
fn page(
    op: &'static str,
    category: i32,
    market: DraftMarket,
    after: Option<i32>,
    limit: Option<u8>,
    rev: Option<&str>,
) -> Result<Vec<(&'static str, String)>, ClientError> {
    if category <= 0
        || after.is_some_and(|n| n <= 0)
        || limit.is_some_and(|n| !(1..=25).contains(&n))
        || after.is_some() && rev.is_none()
        || rev.is_some_and(|v| !valid_revision(v))
    {
        return Err(invalid_input(
            op,
            "Invalid category, page or rules revision",
        ));
    }
    let mut out = vec![
        ("category_id", category.to_string()),
        (
            "market",
            match market {
                DraftMarket::Swedish => "swedish",
                DraftMarket::Norwegian => "norwegian",
            }
            .into(),
        ),
    ];
    if let Some(n) = after {
        out.push(("after", n.to_string()));
    }
    if let Some(n) = limit {
        out.push(("limit", n.to_string()));
    }
    if let Some(s) = rev {
        out.push(("expected_rules_revision", s.into()));
    }
    Ok(out)
}
fn valid_page(
    ids: impl Iterator<Item = i32>,
    after: Option<i32>,
    next: Option<i32>,
    limit: Option<u8>,
    total: usize,
) -> bool {
    let ids: Vec<_> = ids.collect();
    ids.len() <= usize::from(limit.unwrap_or(15))
        && ids.len() <= total
        && ids.first().is_none_or(|n| *n > after.unwrap_or(0))
        && ids.windows(2).all(|w| w[0] < w[1])
        && next.is_none_or(|id| ids.last() == Some(&id))
}
impl AccountApiClient {
    pub async fn creation_rules(
        &self,
        access: &DelegatedAccess,
        q: &CreationRulesQuery,
    ) -> Result<CreationRules, ClientError> {
        const OP: &str = "get_listing_creation_rules";
        self.require_scope(access, LISTINGS_READ_SCOPE, OP)?;
        if !self.write_permissions().draft_authoring {
            return Err(invalid_input(OP, "Draft authoring is unavailable"));
        }
        let args = page(
            OP,
            q.category_id,
            q.market,
            q.after,
            q.limit,
            q.expected_rules_revision.as_deref(),
        )?;
        let r: CreationRules = self
            .get(OP, "mcp/account/listings/creation-rules", access, &args)
            .await?;
        if r.category_id != q.category_id
            || r.market != q.market
            || !valid_revision(&r.rules_revision)
            || q.expected_rules_revision
                .as_ref()
                .is_some_and(|v| v != &r.rules_revision)
            || !valid_action_text(&r.category_name, 256)
            || r.mcp_price_currency != "SEK"
            || r.attributes_max != 50
            || r.replacement != "full_set"
            || r.required_for != "publication"
            || r.value_dependencies != "category"
            || r.listing_types
                != [
                    DraftListingType::Auction,
                    DraftListingType::AuctionFixedPrice,
                    DraftListingType::FixedPrice,
                    DraftListingType::InStock,
                ]
            || r.conditions
                != [
                    DraftCondition::New,
                    DraftCondition::LikeNew,
                    DraftCondition::VeryGood,
                    DraftCondition::Good,
                    DraftCondition::Acceptable,
                ]
            || !valid_page(
                r.fields.iter().map(|f| f.filter_definition_id),
                q.after,
                r.next_after,
                q.limit,
                r.total_fields,
            )
            || r.fields.iter().any(|f| {
                f.supported_in_mcp
                    && (!valid_action_text(&f.name, 128) || !valid_action_text(&f.label, 256))
                    || (!f.name.is_empty() && !valid_action_text(&f.name, 128))
                    || (!f.label.is_empty() && !valid_action_text(&f.label, 256))
                    || f.value_kind
                        != match f.filter_type {
                            CreationFilterType::Text
                            | CreationFilterType::Select
                            | CreationFilterType::CountrySelect => CreationValueKind::Text,
                            CreationFilterType::MultiSelect => CreationValueKind::Options,
                            CreationFilterType::RangeSingle
                            | CreationFilterType::RangeDual
                            | CreationFilterType::RatingStars => CreationValueKind::Number,
                            CreationFilterType::Boolean => CreationValueKind::Boolean,
                            CreationFilterType::DateRange => CreationValueKind::Date,
                        }
                    || f.pattern
                        .as_ref()
                        .is_some_and(|p| !valid_action_text(p, 512))
                    || f.unit.as_ref().is_some_and(|s| !valid_action_text(s, 64))
                    || [
                        &f.minimum,
                        &f.maximum,
                        &f.step,
                        &f.maximum_exclusive_magnitude,
                    ]
                    .into_iter()
                    .flatten()
                    .any(|v| v.len() > 40 || !crate::is_signed_exact_decimal(v))
            })
        {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }
    pub async fn creation_options(
        &self,
        access: &DelegatedAccess,
        q: &CreationOptionsQuery,
    ) -> Result<CreationOptions, ClientError> {
        const OP: &str = "list_listing_creation_options";
        self.require_scope(access, LISTINGS_READ_SCOPE, OP)?;
        if !self.write_permissions().draft_authoring
            || q.filter_definition_id <= 0
            || q.search
                .as_ref()
                .is_some_and(|s| s.trim() != s || !valid_action_text(s, 80))
        {
            return Err(invalid_input(OP, "Invalid authoring field or search"));
        }
        let mut args = page(
            OP,
            q.category_id,
            q.market,
            q.after,
            q.limit,
            Some(&q.expected_rules_revision),
        )?;
        args.push(("filter_definition_id", q.filter_definition_id.to_string()));
        if let Some(s) = &q.search {
            args.push(("search", s.clone()));
        }
        let r: CreationOptions = self
            .get(OP, "mcp/account/listings/creation-options", access, &args)
            .await?;
        if r.category_id != q.category_id
            || r.market != q.market
            || r.filter_definition_id != q.filter_definition_id
            || r.rules_revision != q.expected_rules_revision
            || !valid_page(
                r.options.iter().map(|o| o.option_id),
                q.after,
                r.next_after,
                q.limit,
                r.total_matches,
            )
            || r.options.iter().any(|o| {
                o.supported_in_mcp != o.value.is_some()
                    || o.value
                        .as_ref()
                        .is_some_and(|v| !valid_action_text(v, 1000))
                    || o.label.as_ref().is_some_and(|v| !valid_action_text(v, 256))
            })
        {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }
}
