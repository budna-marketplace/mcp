use super::*;
use budna_mcp_client::BidAmount;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StandingParams {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    pub listing_id: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrepareMaximumParams {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    pub listing_id: String,
    pub maximum: BidAmount,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetMaximumParams {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    pub listing_id: String,
    #[schemars(length(min = 36, max = 36))]
    pub quote_id: String,
    pub maximum: BidAmount,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PauseMaximumParams {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    pub listing_id: String,
    #[schemars(length(min = 43, max = 43), regex(pattern = r"^[A-Za-z0-9_-]{43}$"))]
    pub expected_revision: String,
    #[schemars(length(min = 36, max = 36))]
    pub idempotency_key: String,
}
