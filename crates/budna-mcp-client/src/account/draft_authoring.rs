use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DRAFT_AUTHORING_SCOPE,
    DelegatedAccess, DraftReceipt, invalid_input, is_action_id,
};
use crate::is_listing_id;
use reqwest::Method;
impl AccountApiClient {
    pub async fn create_similar_draft(
        &self,
        access: &DelegatedAccess,
        source: &str,
        expected_revision: i64,
        request_id: &str,
    ) -> Result<DraftReceipt, ClientError> {
        const OP: &str = "create_my_similar_draft";
        if !is_listing_id(source) || expected_revision < 1 {
            return Err(invalid_input(OP, "Invalid source listing or revision"));
        }
        let request = self
            .action_request(
                OP,
                DRAFT_AUTHORING_SCOPE,
                access,
                Method::POST,
                &format!("mcp/account/listings/{source}/similar-draft"),
                request_id,
            )?
            .json(&serde_json::json!({"expected_revision":expected_revision}));
        self.direct_receipt(OP, request, |r: &DraftReceipt| {
            is_action_id(&r.action_id)
                && r.listing_id
                    .as_deref()
                    .is_none_or(|id| is_listing_id(id) && id != source)
                && (r.status != AccountWatchlistActionStatus::Executed || r.listing_id.is_some())
        })
        .await
    }
}
