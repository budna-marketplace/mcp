use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess,
    MESSAGES_READ_SCOPE, PUBLIC_CHAT_WRITE_SCOPE, actions::valid_outgoing_text, invalid_input,
    invalid_response, is_action_id,
};
use crate::{is_collection_id, is_listing_id, is_live_stream_id};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PublicChatAudience {
    Public,
}
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PublicChatRole {
    Host,
    Viewer,
}
#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PublicChatBlocker {
    Closed,
    Banned,
    Frozen,
    VerificationRequired,
}

#[derive(Debug, Deserialize)]
pub struct PublicChatPosting {
    pub room_id: String,
    pub audience: PublicChatAudience,
    pub role: PublicChatRole,
    pub can_post: bool,
    #[serde(deserialize_with = "Option::deserialize")]
    pub blocker: Option<PublicChatBlocker>,
    pub cooldown_seconds: u32,
    pub max_text_length: u32,
    pub replyable_message_ids: Vec<i64>,
}
#[derive(Debug, Deserialize)]
pub struct PublicRoomMessage {
    pub room_id: String,
    pub message_id: i64,
    pub text: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub reply_to_message_id: Option<i64>,
    pub is_host: bool,
}
#[derive(Debug, Deserialize)]
pub struct PublicChatReceipt {
    pub action_id: String,
    pub room_id: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub message_id: Option<i64>,
    pub status: AccountWatchlistActionStatus,
}
fn valid_room(id: &str) -> bool {
    is_listing_id(id) || is_collection_id(id) || is_live_stream_id(id)
}

impl AccountApiClient {
    pub async fn chat_posting(
        &self,
        access: &DelegatedAccess,
        id: &str,
    ) -> Result<PublicChatPosting, ClientError> {
        const OP: &str = "get_my_chat_posting";
        self.require_scope(access, MESSAGES_READ_SCOPE, OP)?;
        if !self.write_permissions().public_chat || !valid_room(id) {
            return Err(invalid_input(OP, "Room unavailable or invalid"));
        }
        let r: PublicChatPosting = self
            .get(
                OP,
                &format!("mcp/account/chat-rooms/{id}/posting"),
                access,
                &[],
            )
            .await?;
        let mut ids = BTreeSet::new();
        if r.room_id != id
            || r.can_post != r.blocker.is_none()
            || r.replyable_message_ids.len() > 50
            || r.replyable_message_ids
                .iter()
                .any(|id| *id < 1 || !ids.insert(id))
            || r.max_text_length == 0
            || r.max_text_length > 500
        {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }
    pub async fn public_room_message(
        &self,
        access: &DelegatedAccess,
        room: &str,
        message: i64,
    ) -> Result<PublicRoomMessage, ClientError> {
        const OP: &str = "get_my_chat_message";
        self.require_scope(access, MESSAGES_READ_SCOPE, OP)?;
        if !self.write_permissions().public_chat || !valid_room(room) || message < 1 {
            return Err(invalid_input(OP, "Invalid room or message"));
        }
        let r: PublicRoomMessage = self
            .get(
                OP,
                &format!("mcp/account/chat-rooms/{room}/messages/{message}"),
                access,
                &[],
            )
            .await?;
        if r.room_id != room
            || r.message_id != message
            || r.text.len() > 8192
            || r.reply_to_message_id
                .is_some_and(|id| id < 1 || id == message)
        {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }

    pub async fn post_public_chat(
        &self,
        access: &DelegatedAccess,
        room: &str,
        audience: PublicChatAudience,
        text: &str,
        parent: Option<i64>,
        key: &str,
    ) -> Result<PublicChatReceipt, ClientError> {
        const OP: &str = "post_public_chat_message";
        if !valid_room(room)
            || !valid_outgoing_text(text, 500, true)
            || parent.is_some_and(|id| id < 1)
        {
            return Err(invalid_input(OP, "Invalid room, reply target or text"));
        }
        #[derive(Serialize)]
        struct Post<'a> {
            audience: PublicChatAudience,
            text: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            reply_to_message_id: Option<i64>,
        }
        let request = self
            .action_request(
                OP,
                PUBLIC_CHAT_WRITE_SCOPE,
                access,
                Method::POST,
                &format!("mcp/account/chat-rooms/{room}/messages"),
                key,
            )?
            .json(&Post {
                audience,
                text,
                reply_to_message_id: parent,
            });
        self.direct_receipt(OP, request, |r: &PublicChatReceipt| {
            is_action_id(&r.action_id)
                && r.room_id == room
                && match r.message_id {
                    Some(id) => id > 0 && Some(id) != parent,
                    None => r.status != AccountWatchlistActionStatus::Executed,
                }
        })
        .await
    }
}
