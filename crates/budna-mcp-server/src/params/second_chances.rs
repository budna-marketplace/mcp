use budna_mcp_client::{SecondChanceAnswer, SecondChanceSale};
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct SaleStateParams {
    #[schemars(range(min = 1))]
    pub source_order_id: i64,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct InviteParams {
    #[schemars(range(min = 1))]
    pub offer_id: i64,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct SaleQuote {
    pub change: SecondChanceSale,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct AnswerQuote {
    pub change: SecondChanceAnswer,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct SubmitSale {
    pub change: SecondChanceSale,
    #[schemars(length(min = 36, max = 36))]
    pub quote_id: String,
    #[schemars(length(min = 43, max = 43))]
    pub terms_digest: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct SubmitAnswer {
    pub change: SecondChanceAnswer,
    #[schemars(length(min = 36, max = 36))]
    pub quote_id: String,
    #[schemars(length(min = 43, max = 43))]
    pub terms_digest: String,
}
