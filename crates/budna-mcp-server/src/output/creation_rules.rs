use budna_mcp_client::{
    CreationField, CreationFilterType, CreationOption, CreationOptions, CreationRules,
    CreationValueKind, DraftCondition, DraftListingType, DraftMarket,
};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CreationFieldOutput {
    filter_definition_id: i32,
    name: String,
    label: String,
    filter_type: CreationFilterType,
    value_kind: CreationValueKind,
    required_on_publish: bool,
    min_length: Option<i32>,
    max_length: Option<i32>,
    pattern: Option<String>,
    minimum: Option<String>,
    maximum: Option<String>,
    step: Option<String>,
    unit: Option<String>,
    max_fraction_digits: Option<u8>,
    maximum_exclusive_magnitude: Option<String>,
    max_choices: Option<usize>,
    active_option_count: usize,
    supported_in_mcp: bool,
}
impl From<CreationField> for CreationFieldOutput {
    fn from(v: CreationField) -> Self {
        Self {
            filter_definition_id: v.filter_definition_id,
            name: v.name,
            label: v.label,
            filter_type: v.filter_type,
            value_kind: v.value_kind,
            required_on_publish: v.required_on_publish,
            min_length: v.min_length,
            max_length: v.max_length,
            pattern: v.pattern,
            minimum: v.minimum,
            maximum: v.maximum,
            step: v.step,
            unit: v.unit,
            max_fraction_digits: v.max_fraction_digits,
            maximum_exclusive_magnitude: v.maximum_exclusive_magnitude,
            max_choices: v.max_choices,
            active_option_count: v.active_option_count,
            supported_in_mcp: v.supported_in_mcp,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CreationOptionOutput {
    option_id: i32,
    value: Option<String>,
    label: Option<String>,
    supported_in_mcp: bool,
}
impl From<CreationOption> for CreationOptionOutput {
    fn from(v: CreationOption) -> Self {
        Self {
            option_id: v.option_id,
            value: v.value,
            label: v.label,
            supported_in_mcp: v.supported_in_mcp,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CreationRulesOutput {
    category_id: i32,
    category_name: String,
    market: DraftMarket,
    mcp_price_currency: String,
    rules_revision: String,
    listing_types: Vec<DraftListingType>,
    conditions: Vec<DraftCondition>,
    attributes_max: usize,
    replacement: String,
    required_for: String,
    value_dependencies: String,
    total_fields: usize,
    fields: Vec<CreationFieldOutput>,
    next_after: Option<i32>,
}
impl From<CreationRules> for CreationRulesOutput {
    fn from(v: CreationRules) -> Self {
        Self {
            category_id: v.category_id,
            category_name: v.category_name,
            market: v.market,
            mcp_price_currency: v.mcp_price_currency,
            rules_revision: v.rules_revision,
            listing_types: v.listing_types,
            conditions: v.conditions,
            attributes_max: v.attributes_max,
            replacement: v.replacement,
            required_for: v.required_for,
            value_dependencies: v.value_dependencies,
            total_fields: v.total_fields,
            fields: v.fields.into_iter().map(Into::into).collect(),
            next_after: v.next_after,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CreationOptionsOutput {
    category_id: i32,
    market: DraftMarket,
    filter_definition_id: i32,
    rules_revision: String,
    total_matches: usize,
    options: Vec<CreationOptionOutput>,
    next_after: Option<i32>,
}
impl From<CreationOptions> for CreationOptionsOutput {
    fn from(v: CreationOptions) -> Self {
        Self {
            category_id: v.category_id,
            market: v.market,
            filter_definition_id: v.filter_definition_id,
            rules_revision: v.rules_revision,
            total_matches: v.total_matches,
            options: v.options.into_iter().map(Into::into).collect(),
            next_after: v.next_after,
        }
    }
}
