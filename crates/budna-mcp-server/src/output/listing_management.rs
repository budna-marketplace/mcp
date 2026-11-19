use super::common::MoneyOutput;
use budna_mcp_client::{ListingChangeKind, ListingManagementBlocker, ListingManagementReview};
use rmcp::schemars;
use serde::Serialize;
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListingManagementOutput {
    listing_id: String,
    revision: i64,
    kind: ListingChangeKind,
    ready: bool,
    blockers: Vec<ListingManagementBlocker>,
    quote_id: Option<String>,
    starts_at: Option<i64>,
    ends_at: Option<i64>,
    additional_sale_fee: Option<MoneyOutput>,
    notifies_watchers: bool,
}
impl From<ListingManagementReview> for ListingManagementOutput {
    fn from(r: ListingManagementReview) -> Self {
        Self {
            listing_id: r.listing_id,
            revision: r.revision,
            kind: r.kind,
            ready: r.ready,
            blockers: r.blockers,
            quote_id: r.quote_id,
            starts_at: r.starts_at,
            ends_at: r.ends_at,
            additional_sale_fee: r.additional_sale_fee.map(Into::into),
            notifies_watchers: r.notifies_watchers,
        }
    }
}
