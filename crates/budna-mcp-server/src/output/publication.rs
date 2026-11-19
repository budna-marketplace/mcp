use super::common::MoneyOutput;
use budna_mcp_client::{PublicationBlocker, PublicationPriority, PublicationReadiness};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PublicationReadinessOutput {
    listing_id: String,
    revision: i64,
    ready: bool,
    blockers: Vec<PublicationBlocker>,
    quote_id: Option<String>,
    starts_at: i64,
    ends_at: i64,
    media_count: u32,
    upfront_due: Option<MoneyOutput>,
    assumed_sale_price: Option<MoneyOutput>,
    estimated_sale_fees: Option<MoneyOutput>,
    estimated_seller_proceeds: Option<MoneyOutput>,
    priority_tier: Option<PublicationPriority>,
    priority_credit_expected: bool,
}
impl From<PublicationReadiness> for PublicationReadinessOutput {
    fn from(r: PublicationReadiness) -> Self {
        Self {
            listing_id: r.listing_id,
            revision: r.revision,
            ready: r.ready,
            blockers: r.blockers,
            quote_id: r.quote_id,
            starts_at: r.starts_at,
            ends_at: r.ends_at,
            media_count: r.media_count,
            upfront_due: r.upfront_due.map(Into::into),
            assumed_sale_price: r.assumed_sale_price.map(Into::into),
            estimated_sale_fees: r.estimated_sale_fees.map(Into::into),
            estimated_seller_proceeds: r.estimated_seller_proceeds.map(Into::into),
            priority_tier: r.priority_tier,
            priority_credit_expected: r.priority_credit_expected,
        }
    }
}
