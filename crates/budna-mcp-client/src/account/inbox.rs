use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess,
    INBOX_WRITE_SCOPE, MESSAGES_READ_SCOPE, invalid_input, invalid_response, is_action_id,
    is_listing_id,
};

#[derive(Debug, Deserialize)]
pub struct ConversationSettings {
    pub conversation_id: i64,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub is_muted: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConversationSetting {
    Pinned,
    Archived,
    Muted,
}

#[derive(Debug, Deserialize)]
pub struct ConversationSettingReceipt {
    pub action_id: String,
    pub status: AccountWatchlistActionStatus,
    pub conversation_id: i64,
    pub setting: ConversationSetting,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct MessageCopyReceipt {
    pub action_id: String,
    pub status: AccountWatchlistActionStatus,
    pub message_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ListingChatPreference {
    pub listing_id: String,
    pub muted: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListingChatPreferenceReceipt {
    pub action_id: String,
    pub status: AccountWatchlistActionStatus,
    pub listing_id: String,
    pub muted: bool,
}

impl AccountApiClient {
    fn require_inbox_read(
        &self,
        access: &DelegatedAccess,
        op: &'static str,
    ) -> Result<(), ClientError> {
        self.require_scope(access, MESSAGES_READ_SCOPE, op)?;
        if !self.write_permissions().inbox {
            return Err(invalid_input(op, "Inbox organization is unavailable"));
        }
        Ok(())
    }

    pub async fn conversation_settings(
        &self,
        access: &DelegatedAccess,
        id: i64,
    ) -> Result<ConversationSettings, ClientError> {
        const OP: &str = "get_my_conversation_settings";
        self.require_inbox_read(access, OP)?;
        if id < 1 {
            return Err(invalid_input(OP, "Invalid conversation ID"));
        }
        let result: ConversationSettings = self
            .get(
                OP,
                &format!("mcp/account/conversations/{id}/settings"),
                access,
                &[],
            )
            .await?;
        if result.conversation_id != id {
            return Err(invalid_response(OP));
        }
        Ok(result)
    }

    pub async fn set_conversation_setting(
        &self,
        access: &DelegatedAccess,
        id: i64,
        setting: ConversationSetting,
        enabled: bool,
        key: &str,
    ) -> Result<ConversationSettingReceipt, ClientError> {
        const OP: &str = "set_my_conversation_setting";
        if id < 1 {
            return Err(invalid_input(OP, "Invalid conversation ID"));
        }
        let request = self
            .action_request(
                OP,
                INBOX_WRITE_SCOPE,
                access,
                Method::PUT,
                &format!("mcp/account/conversations/{id}/settings"),
                key,
            )?
            .json(&serde_json::json!({ "setting":setting,"enabled":enabled }));
        self.direct_receipt(OP, request, |r: &ConversationSettingReceipt| {
            is_action_id(&r.action_id)
                && r.conversation_id == id
                && r.setting == setting
                && r.enabled == enabled
        })
        .await
    }

    pub async fn hide_message_copy(
        &self,
        access: &DelegatedAccess,
        id: i64,
        key: &str,
    ) -> Result<MessageCopyReceipt, ClientError> {
        const OP: &str = "hide_my_message_copy";
        if id < 1 {
            return Err(invalid_input(OP, "Invalid message ID"));
        }
        let request = self.action_request(
            OP,
            INBOX_WRITE_SCOPE,
            access,
            Method::DELETE,
            &format!("mcp/account/messages/{id}/copy"),
            key,
        )?;
        self.direct_receipt(OP, request, |r: &MessageCopyReceipt| {
            is_action_id(&r.action_id) && r.message_id == id
        })
        .await
    }

    pub async fn listing_chat_preference(
        &self,
        access: &DelegatedAccess,
        id: &str,
    ) -> Result<ListingChatPreference, ClientError> {
        const OP: &str = "get_my_listing_chat_notification_setting";
        self.require_inbox_read(access, OP)?;
        if !is_listing_id(id) {
            return Err(invalid_input(OP, "Invalid listing ID"));
        }
        let result: ListingChatPreference = self
            .get(
                OP,
                &format!("mcp/account/listing-chats/{id}/notifications"),
                access,
                &[],
            )
            .await?;
        if result.listing_id != id {
            return Err(invalid_response(OP));
        }
        Ok(result)
    }

    pub async fn set_listing_chat_muted(
        &self,
        access: &DelegatedAccess,
        id: &str,
        muted: bool,
        key: &str,
    ) -> Result<ListingChatPreferenceReceipt, ClientError> {
        const OP: &str = "set_my_listing_chat_muted";
        if !is_listing_id(id) {
            return Err(invalid_input(OP, "Invalid listing ID"));
        }
        let request = self
            .action_request(
                OP,
                INBOX_WRITE_SCOPE,
                access,
                Method::PUT,
                &format!("mcp/account/listing-chats/{id}/notifications"),
                key,
            )?
            .json(&serde_json::json!({"muted":muted}));
        self.direct_receipt(OP, request, |r: &ListingChatPreferenceReceipt| {
            is_action_id(&r.action_id) && r.listing_id == id && r.muted == muted
        })
        .await
    }
}
