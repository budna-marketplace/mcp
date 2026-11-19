use super::{ProjectionBudget, UNTRUSTED_CONTENT_NOTICE, common::MoneyOutput};
use budna_mcp_client::{
    BidCurrency, DraftAttributeValue, DraftAuthoringState, DraftCondition, DraftFieldState,
    DraftListingType, DraftMarket, PackageSize, ShippingCarrier,
};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DraftFieldsOutput {
    title: Option<String>,
    description: Option<String>,
    category_id: Option<i32>,
    condition: Option<DraftCondition>,
    listing_type: Option<DraftListingType>,
    market: Option<DraftMarket>,
    currency: Option<BidCurrency>,
    starting_price: Option<MoneyOutput>,
    bid_increment: Option<MoneyOutput>,
    seller_reserve: Option<MoneyOutput>,
    buy_now_price: Option<MoneyOutput>,
    custom_shipping_price: Option<MoneyOutput>,
    tags: Option<Vec<String>>,
    start_time: Option<i64>,
    end_time: Option<i64>,
    package_size: Option<PackageSize>,
    package_weight_grams: Option<i32>,
    shipping_provider_codes: Option<Vec<ShippingCarrier>>,
    combined_shipping_enabled: Option<bool>,
    allow_pickup: Option<bool>,
    location_id: Option<i32>,
}
impl DraftFieldsOutput {
    fn project(f: DraftFieldState, b: &mut ProjectionBudget) -> Self {
        Self {
            title: b.optional_text(f.title, 800),
            description: b.optional_text(f.description, 40000),
            category_id: f.category_id,
            condition: f.condition,
            listing_type: f.listing_type,
            market: f.market,
            currency: f.currency,
            starting_price: f.starting_price.map(Into::into),
            bid_increment: f.bid_increment.map(Into::into),
            seller_reserve: f.seller_reserve.map(Into::into),
            buy_now_price: f.buy_now_price.map(Into::into),
            custom_shipping_price: f.custom_shipping_price.map(Into::into),
            tags: f
                .tags
                .map(|v| v.into_iter().map(|s| b.text(s, 256)).collect()),
            start_time: f.start_time,
            end_time: f.end_time,
            package_size: f.package_size,
            package_weight_grams: f.package_weight_grams,
            shipping_provider_codes: f.shipping_provider_codes,
            combined_shipping_enabled: f.combined_shipping_enabled,
            allow_pickup: f.allow_pickup,
            location_id: f.location_id,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct DraftAttributeOutput {
    filter_definition_id: i32,
    value: DraftAttributeValue,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct VatOutput {
    vat_registered: bool,
    vat_treatment: Option<String>,
    default_vat_treatment: Option<String>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct DraftAuthoringOutput {
    listing_id: String,
    revision: i64,
    fields: DraftFieldsOutput,
    attributes: Vec<DraftAttributeOutput>,
    attributes_replaceable: bool,
    vat: Option<VatOutput>,
    media_count: u32,
    untrusted_content_notice: &'static str,
}
impl From<DraftAuthoringState> for DraftAuthoringOutput {
    fn from(s: DraftAuthoringState) -> Self {
        let mut budget = ProjectionBudget::default();
        let fields = DraftFieldsOutput::project(s.fields, &mut budget);
        let mut attributes_intact = true;
        let mut attribute_text = |s: String| {
            let original = s.len();
            let result = budget.text(s, 4000);
            attributes_intact &= result.len() == original;
            result
        };
        let attributes = s
            .attributes
            .into_iter()
            .map(|a| {
                let value = match a.value {
                    DraftAttributeValue::Text(v) => DraftAttributeValue::Text(attribute_text(v)),
                    DraftAttributeValue::Options(v) => DraftAttributeValue::Options(
                        v.into_iter().map(&mut attribute_text).collect(),
                    ),
                    v => v,
                };
                DraftAttributeOutput {
                    filter_definition_id: a.filter_definition_id,
                    value,
                }
            })
            .collect();
        Self {
            listing_id: s.listing_id,
            revision: s.revision,
            fields,
            attributes,
            attributes_replaceable: s.attributes_replaceable && attributes_intact,
            vat: s.vat.map(|v| VatOutput {
                vat_registered: v.vat_registered,
                vat_treatment: v.vat_treatment,
                default_vat_treatment: v.default_vat_treatment,
            }),
            media_count: s.media_count,
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}
