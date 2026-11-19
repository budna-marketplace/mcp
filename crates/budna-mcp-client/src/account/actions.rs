use reqwest::{Method, header::AUTHORIZATION};
use serde::Deserialize;

use super::{
    AccountApiClient, AccountWatchlistActionStatus, ApiEnvelope, ClientError, DelegatedAccess,
    WATCHLIST_MANAGE_SCOPE, invalid_input, invalid_response, is_canonical_uuid,
};

pub(super) fn valid_action_text(value: &str, limit: usize) -> bool {
    !value.trim().is_empty() && value.chars().count() <= limit && !value.chars().any(|ch| {
        ch.is_control() || matches!(ch, '\u{061C}' | '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{E0000}'..='\u{E007F}')
    })
}

pub fn is_action_id(value: &str) -> bool {
    is_canonical_uuid(value) && value != "00000000-0000-0000-0000-000000000000"
}

pub fn is_watchlist_name(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 255 && !value.chars().any(char::is_control)
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AccountActionState {
    Refused,
    AwaitingConfirmation,
    Declined,
    Expired,
    Executing,
    Executed,
    Failed,
    Unknown,
}

#[derive(Debug, Deserialize)]
pub struct AccountActionStatus {
    #[serde(rename = "id")]
    pub action_id: String,
    pub status: AccountActionState,
    pub expires_at: Option<i64>,
}

#[derive(Clone, Copy, Debug)]
pub enum PrivateWatchlistChange<'a> {
    Create { name: &'a str },
    Rename { watchlist_id: i64, name: &'a str },
    Delete { watchlist_id: i64 },
}

#[derive(Debug, Deserialize)]
pub struct AccountPrivateWatchlistReceipt {
    pub action_id: String,
    pub watchlist_id: Option<i64>,
    pub status: AccountWatchlistActionStatus,
}

impl AccountApiClient {
    pub(super) fn action_request(
        &self,
        operation: &'static str,
        scope: &'static str,
        access: &DelegatedAccess,
        method: Method,
        path: &str,
        key: &str,
    ) -> Result<reqwest::RequestBuilder, ClientError> {
        self.require_scope(access, scope, operation)?;
        if !is_action_id(key) {
            return Err(invalid_input(operation, "Invalid request key"));
        }
        Ok(self
            .http
            .request(method, self.endpoint(operation, path)?)
            .header(AUTHORIZATION, access.credential.header(operation)?)
            .header("Idempotency-Key", key))
    }
    pub(super) async fn direct_receipt<T: serde::de::DeserializeOwned>(
        &self,
        operation: &'static str,
        request: reqwest::RequestBuilder,
        matches: impl FnOnce(&T) -> bool,
    ) -> Result<T, ClientError> {
        let envelope: ApiEnvelope<T> = self
            .send(operation, request, 4096)
            .await
            .map_err(ClientError::account_write)?;
        envelope
            .data
            .filter(|receipt| envelope.success && matches(receipt))
            .ok_or(ClientError::AccountWrite {
                operation,
                rejection_status: None,
            })
    }

    pub async fn action_status(
        &self,
        access: &DelegatedAccess,
        action_id: &str,
    ) -> Result<AccountActionStatus, ClientError> {
        const OPERATION: &str = "get_my_action_status";
        // The API checks the actual action scope and grant together without disclosing either.
        if !self
            .supported_write_scopes()
            .any(|scope| access.allows(scope))
        {
            return Err(ClientError::Api {
                operation: OPERATION,
                status: 403,
                code: None,
                retry_after: None,
            });
        }
        if !is_action_id(action_id) {
            return Err(invalid_input(OPERATION, "Invalid action reference"));
        }
        let status: AccountActionStatus = self
            .get(
                OPERATION,
                &format!("mcp/account/actions/{action_id}"),
                access,
                &[],
            )
            .await?;
        if status.action_id != action_id || status.expires_at.is_some_and(|time| time < 0) {
            return Err(invalid_response(OPERATION));
        }
        Ok(status)
    }

    pub async fn change_private_watchlist(
        &self,
        access: &DelegatedAccess,
        change: PrivateWatchlistChange<'_>,
        idempotency_key: &str,
    ) -> Result<AccountPrivateWatchlistReceipt, ClientError> {
        let (operation, method, id, name) = match change {
            PrivateWatchlistChange::Create { name } => (
                "create_my_private_watchlist",
                Method::POST,
                None,
                Some(name),
            ),
            PrivateWatchlistChange::Rename { watchlist_id, name } => (
                "rename_my_private_watchlist",
                Method::PUT,
                Some(watchlist_id),
                Some(name),
            ),
            PrivateWatchlistChange::Delete { watchlist_id } => (
                "delete_my_private_watchlist",
                Method::DELETE,
                Some(watchlist_id),
                None,
            ),
        };
        self.require_scope(access, WATCHLIST_MANAGE_SCOPE, operation)?;
        if !is_action_id(idempotency_key)
            || id.is_some_and(|id| id < 1)
            || name.is_some_and(|s| !is_watchlist_name(s))
        {
            return Err(invalid_input(
                operation,
                "Invalid watchlist, name or operation key",
            ));
        }
        let path = match id {
            Some(id) => format!("mcp/account/private-watchlists/{id}"),
            None => "mcp/account/private-watchlists".into(),
        };
        let mut request = self
            .http
            .request(method, self.endpoint(operation, &path)?)
            .header(AUTHORIZATION, access.credential.header(operation)?)
            .header("Idempotency-Key", idempotency_key);
        if let Some(name) = name {
            request = request.json(&serde_json::json!({"name":name}));
        }
        let envelope: ApiEnvelope<AccountPrivateWatchlistReceipt> = self
            .send(operation, request, 4096)
            .await
            .map_err(ClientError::account_write)?;
        envelope
            .data
            .filter(|receipt| {
                envelope.success
                    && is_action_id(&receipt.action_id)
                    && receipt.watchlist_id.is_none_or(|returned| returned > 0)
                    && id.is_none_or(|expected| receipt.watchlist_id == Some(expected))
                    && (receipt.status != AccountWatchlistActionStatus::Executed
                        || receipt.watchlist_id.is_some())
            })
            .ok_or(ClientError::AccountWrite {
                operation,
                rejection_status: None,
            })
    }
}
