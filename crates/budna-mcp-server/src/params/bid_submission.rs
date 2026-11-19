use super::*;
use budna_mcp_client::BidAmount;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PrepareBidParams {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    pub listing_id: String,
    pub bid: BidAmount,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubmitBidParams {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    pub listing_id: String,
    #[schemars(length(min = 36, max = 36))]
    pub quote_id: String,
    pub bid: BidAmount,
}
