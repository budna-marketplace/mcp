use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

use super::{
    AccountApiClient, AccountWatchlistActionStatus, ApiEnvelope, ClientError, DelegatedAccess,
    LIVE_REMINDER_WRITE_SCOPE, invalid_input, is_action_id,
};
use crate::is_live_stream_id;

#[derive(Debug, Deserialize)]
pub struct LiveReminderReceipt {
    pub action_id: String,
    pub session_id: String,
    pub reminded: bool,
    pub status: AccountWatchlistActionStatus,
}

impl AccountApiClient {
    pub async fn set_live_reminder(
        &self,
        access: &DelegatedAccess,
        session_id: &str,
        reminded: bool,
        key: &str,
    ) -> Result<LiveReminderReceipt, ClientError> {
        const OPERATION: &str = "set_my_live_reminder";
        self.require_scope(access, LIVE_REMINDER_WRITE_SCOPE, OPERATION)?;
        if !is_live_stream_id(session_id) || !is_action_id(key) {
            return Err(invalid_input(
                OPERATION,
                "Invalid session reference or request key",
            ));
        }
        let url = self.endpoint(
            OPERATION,
            &format!("mcp/account/live-reminders/{session_id}"),
        )?;
        let request = self
            .http
            .put(url)
            .header(AUTHORIZATION, access.credential.header(OPERATION)?)
            .header("Idempotency-Key", key)
            .json(&serde_json::json!({"reminded":reminded}));
        let envelope: ApiEnvelope<LiveReminderReceipt> = self
            .send(OPERATION, request, 4096)
            .await
            .map_err(ClientError::account_write)?;
        envelope
            .data
            .filter(|r| {
                envelope.success
                    && is_action_id(&r.action_id)
                    && r.session_id == session_id
                    && r.reminded == reminded
            })
            .ok_or(ClientError::AccountWrite {
                operation: OPERATION,
                rejection_status: None,
            })
    }
}
