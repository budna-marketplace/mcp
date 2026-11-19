use super::{
    AccountApiClient, AccountWatchlistActionStatus, ApiEnvelope, ClientError, DelegatedAccess,
    NOTIFICATION_STATE_SCOPE, NOTIFICATIONS_READ_SCOPE, invalid_input, invalid_response,
    is_action_id,
};
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Debug, Deserialize)]
pub struct NotificationReadBatch {
    pub notification_ids: Vec<i64>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
pub struct NotificationReadReceipt {
    pub action_id: String,
    pub status: AccountWatchlistActionStatus,
    pub requested_count: u32,
}

fn valid_ids(ids: &[i64]) -> bool {
    ids.len() <= 50
        && ids.iter().all(|id| *id > 0)
        && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()
}

impl AccountApiClient {
    pub async fn notification_read_batch(
        &self,
        access: &DelegatedAccess,
    ) -> Result<NotificationReadBatch, ClientError> {
        const OP: &str = "get_my_notification_read_batch";
        self.require_scope(access, NOTIFICATIONS_READ_SCOPE, OP)?;
        if !self.write_permissions().notification_state {
            return Err(invalid_input(
                OP,
                "Notification acknowledgement is unavailable",
            ));
        }
        let batch: NotificationReadBatch = self
            .get(OP, "mcp/account/notifications/read-batch", access, &[])
            .await?;
        if !valid_ids(&batch.notification_ids)
            || (batch.has_more && batch.notification_ids.len() != 50)
        {
            return Err(invalid_response(OP));
        }
        Ok(batch)
    }

    pub async fn mark_notifications_read(
        &self,
        access: &DelegatedAccess,
        ids: &[i64],
        key: &str,
    ) -> Result<NotificationReadReceipt, ClientError> {
        const OP: &str = "mark_my_notifications_read";
        self.require_scope(access, NOTIFICATION_STATE_SCOPE, OP)?;
        if !is_action_id(key) || ids.is_empty() || !valid_ids(ids) {
            return Err(invalid_input(
                OP,
                "Supply between one and fifty distinct positive notification IDs",
            ));
        }
        let request = self
            .http
            .put(self.endpoint(OP, "mcp/account/notifications/read-state")?)
            .header(AUTHORIZATION, access.credential.header(OP)?)
            .header("Idempotency-Key", key)
            .json(&serde_json::json!({"notification_ids":ids}));
        let envelope: ApiEnvelope<NotificationReadReceipt> = self
            .send(OP, request, 4096)
            .await
            .map_err(ClientError::account_write)?;
        envelope
            .data
            .filter(|r| {
                envelope.success
                    && is_action_id(&r.action_id)
                    && r.requested_count as usize == ids.len()
            })
            .ok_or(ClientError::AccountWrite {
                operation: OP,
                rejection_status: None,
            })
    }
}
