use super::{ProjectionBudget, UNTRUSTED_CONTENT_NOTICE, account::WatchlistActionStatusOutput};
use budna_mcp_client::{PlannedStreamStatus, StreamAuthoring, StreamPlanReceipt, StreamPlanTarget};
use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AuthoringOutput {
    revision: String,
    target: StreamPlanTarget,
    seller_verified: bool,
    seller_ready: bool,
    target_allows_scheduling: bool,
    target_ends_at: Option<i64>,
    session: Option<SessionOutput>,
    max_advance_days: u8,
    max_lineup: usize,
    untrusted_content_notice: &'static str,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct SessionOutput {
    stream_id: String,
    status: PlannedStreamStatus,
    scheduled_start_at: i64,
    title: Option<String>,
    description: Option<String>,
    notify_followers: bool,
    listing_ids: Vec<String>,
}
impl From<StreamAuthoring> for AuthoringOutput {
    fn from(a: StreamAuthoring) -> Self {
        let mut budget = ProjectionBudget::default();
        Self {
            revision: a.revision,
            target: a.target,
            seller_verified: a.seller_verified,
            seller_ready: a.seller_ready,
            target_allows_scheduling: a.target_allows_scheduling,
            target_ends_at: a.target_ends_at,
            session: a.session.map(|s| SessionOutput {
                stream_id: s.stream_id,
                status: s.status,
                scheduled_start_at: s.scheduled_start_at,
                title: s.title.map(|title| budget.text(title, 120)),
                description: s.description.map(|desc| budget.text(desc, 1000)),
                notify_followers: s.notify_followers,
                listing_ids: s.listing_ids,
            }),
            max_advance_days: a.max_advance_days,
            max_lineup: a.max_lineup,
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ReceiptOutput {
    action_id: String,
    action: String,
    stream_id: Option<String>,
    status: WatchlistActionStatusOutput,
}
impl From<StreamPlanReceipt> for ReceiptOutput {
    fn from(r: StreamPlanReceipt) -> Self {
        Self {
            action_id: r.action_id,
            action: r.action,
            stream_id: r.stream_id,
            status: r.status.into(),
        }
    }
}
