use budna_mcp_client::NegotiationChange;
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct PrepareOfferParams {
    #[schemars(length(min = 44, max = 44))]
    pub listing_id: String,
    pub proposal: NegotiationChange,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct SubmitOfferParams {
    #[schemars(length(min = 44, max = 44))]
    pub listing_id: String,
    #[schemars(length(min = 36, max = 36))]
    pub quote_id: String,
    #[schemars(length(min = 43, max = 43))]
    pub terms_digest: String,
    pub proposal: NegotiationChange,
}
