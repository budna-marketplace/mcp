use super::{ProjectionBudget, UNTRUSTED_CONTENT_NOTICE, account::WatchlistActionStatusOutput};
use budna_mcp_client::{
    AuthoringShowcaseKind, AuthoringShowcaseStatus, DraftMarket, ShowcaseAuthoring,
    ShowcaseChangeReceipt, ShowcaseListingRefusal,
};
use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AuthoringOutput {
    revision: String,
    seller_verified: bool,
    open_count: u64,
    max_open: u64,
    max_items: u64,
    showcase: Option<ShowcaseOutput>,
    candidate: Option<CandidateOutput>,
    untrusted_content_notice: &'static str,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ShowcaseOutput {
    showcase_id: String,
    kind: AuthoringShowcaseKind,
    status: AuthoringShowcaseStatus,
    market: DraftMarket,
    title: String,
    description: String,
    chat_enabled: bool,
    ends_at: i64,
    has_banner: bool,
    can_publish: bool,
    public_item_count: u64,
    scheduled_stream_count: u64,
    live_stream_count: u64,
    members: Vec<MemberOutput>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct MemberOutput {
    listing_id: String,
    ends_at: i64,
    aligned: bool,
    has_bid: bool,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct CandidateOutput {
    listing_id: String,
    current_ends_at: i64,
    resulting_ends_at: Option<i64>,
    refusal: Option<ShowcaseListingRefusal>,
}
impl From<ShowcaseAuthoring> for AuthoringOutput {
    fn from(a: ShowcaseAuthoring) -> Self {
        let mut budget = ProjectionBudget::default();
        Self {
            revision: a.revision,
            seller_verified: a.seller_verified,
            open_count: a.open_count,
            max_open: a.max_open,
            max_items: a.max_items,
            showcase: a.showcase.map(|s| ShowcaseOutput {
                showcase_id: s.showcase_id,
                kind: s.kind,
                status: s.status,
                market: s.market,
                title: budget.text(s.title, 120),
                description: budget.text(s.description, 5000),
                chat_enabled: s.chat_enabled,
                ends_at: s.ends_at,
                has_banner: s.has_banner,
                can_publish: s.can_publish,
                public_item_count: s.public_item_count,
                scheduled_stream_count: s.scheduled_stream_count,
                live_stream_count: s.live_stream_count,
                members: s
                    .members
                    .into_iter()
                    .map(|m| MemberOutput {
                        listing_id: m.listing_id,
                        ends_at: m.ends_at,
                        aligned: m.aligned,
                        has_bid: m.has_bid,
                    })
                    .collect(),
            }),
            candidate: a.candidate.map(|c| CandidateOutput {
                listing_id: c.listing_id,
                current_ends_at: c.current_ends_at,
                resulting_ends_at: c.resulting_ends_at,
                refusal: c.refusal,
            }),
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ReceiptOutput {
    action_id: String,
    action: String,
    showcase_id: Option<String>,
    status: WatchlistActionStatusOutput,
}
impl From<ShowcaseChangeReceipt> for ReceiptOutput {
    fn from(r: ShowcaseChangeReceipt) -> Self {
        Self {
            action_id: r.action_id,
            action: r.action,
            showcase_id: r.showcase_id,
            status: r.status.into(),
        }
    }
}
