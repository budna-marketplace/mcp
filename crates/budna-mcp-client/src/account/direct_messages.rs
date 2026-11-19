use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess,
    MESSAGES_READ_SCOPE, MESSAGES_WRITE_SCOPE, actions::valid_outgoing_text, invalid_input,
    invalid_response, is_action_id, is_listing_id, is_user_id, social::is_account_username,
};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct MessageRecipient {
    pub username: String,
    pub user_id: String,
}
#[derive(Debug, Deserialize)]
pub struct ConversationAudience {
    pub conversation_id: i64,
    pub recipient_username: String,
    pub recipient_user_id: String,
    pub listing_id: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct MessageAuthoring {
    pub conversation_id: i64,
    pub message_id: i64,
    pub revision: i64,
    pub can_edit: bool,
}
#[derive(Debug, Deserialize)]
pub struct MessageActionReceipt {
    pub action_id: String,
    pub status: AccountWatchlistActionStatus,
    pub conversation_id: Option<i64>,
    pub message_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct AddressedText<'a> {
    pub recipient_username: &'a str,
    pub recipient_user_id: &'a str,
    pub text: &'a str,
}
pub enum MessageAction<'a> {
    Start {
        listing_id: Option<&'a str>,
    },
    Send {
        conversation_id: i64,
    },
    Reply {
        conversation_id: i64,
        parent_id: i64,
    },
    Edit {
        conversation_id: i64,
        message_id: i64,
        expected_revision: i64,
    },
}

impl AccountApiClient {
    fn require_message_read(
        &self,
        access: &DelegatedAccess,
        op: &'static str,
    ) -> Result<(), ClientError> {
        self.require_scope(access, MESSAGES_READ_SCOPE, op)?;
        if !self.write_permissions().messages {
            return Err(invalid_input(op, "Message authoring is unavailable"));
        }
        Ok(())
    }
    pub async fn message_recipient(
        &self,
        access: &DelegatedAccess,
        username: &str,
    ) -> Result<MessageRecipient, ClientError> {
        const OP: &str = "get_my_message_recipient";
        self.require_message_read(access, OP)?;
        if !is_account_username(username) {
            return Err(invalid_input(OP, "Invalid recipient handle"));
        }
        let r: MessageRecipient = self
            .get(
                OP,
                &format!("mcp/account/message-recipients/{username}"),
                access,
                &[],
            )
            .await?;
        if r.username != username || !is_user_id(&r.user_id) || r.user_id == access.subject {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }
    pub async fn conversation_audience(
        &self,
        access: &DelegatedAccess,
        id: i64,
    ) -> Result<ConversationAudience, ClientError> {
        const OP: &str = "get_my_conversation_audience";
        self.require_message_read(access, OP)?;
        if id <= 0 {
            return Err(invalid_input(OP, "Invalid conversation"));
        }
        let r: ConversationAudience = self
            .get(
                OP,
                &format!("mcp/account/conversations/{id}/audience"),
                access,
                &[],
            )
            .await?;
        if r.conversation_id != id
            || !is_account_username(&r.recipient_username)
            || !is_user_id(&r.recipient_user_id)
            || r.recipient_user_id == access.subject
            || r.listing_id
                .as_deref()
                .is_some_and(|listing| !is_listing_id(listing))
        {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }
    pub async fn message_authoring(
        &self,
        access: &DelegatedAccess,
        id: i64,
    ) -> Result<MessageAuthoring, ClientError> {
        const OP: &str = "get_my_message_authoring";
        self.require_message_read(access, OP)?;
        if id <= 0 {
            return Err(invalid_input(OP, "Invalid message"));
        }
        let r: MessageAuthoring = self
            .get(
                OP,
                &format!("mcp/account/messages/{id}/authoring"),
                access,
                &[],
            )
            .await?;
        if r.message_id != id || r.conversation_id <= 0 || r.revision <= 0 {
            return Err(invalid_response(OP));
        }
        Ok(r)
    }
    pub async fn write_message(
        &self,
        access: &DelegatedAccess,
        action: MessageAction<'_>,
        content: AddressedText<'_>,
        key: &str,
    ) -> Result<MessageActionReceipt, ClientError> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(flatten)]
            content: AddressedText<'a>,
            #[serde(skip_serializing_if = "Option::is_none")]
            listing_id: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            conversation_id: Option<i64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            expected_revision: Option<i64>,
        }
        let (op, method, path, conversation, message) = match action {
            MessageAction::Start { listing_id } => {
                const OP: &str = "start_my_conversation";
                if listing_id.is_some_and(|s| !is_listing_id(s)) {
                    return Err(invalid_input(OP, "Invalid listing"));
                }
                (
                    OP,
                    Method::POST,
                    "mcp/account/conversations/start".to_owned(),
                    None,
                    None,
                )
            }
            MessageAction::Send { conversation_id } => (
                "send_my_message",
                Method::POST,
                format!("mcp/account/conversations/{conversation_id}/messages"),
                Some(conversation_id),
                None,
            ),
            MessageAction::Reply {
                conversation_id,
                parent_id,
            } => (
                "reply_to_my_message",
                Method::POST,
                format!("mcp/account/messages/{parent_id}/replies"),
                Some(conversation_id),
                Some(parent_id),
            ),
            MessageAction::Edit {
                conversation_id,
                message_id,
                expected_revision,
            } => {
                if expected_revision < 1 {
                    return Err(invalid_input("edit_my_message", "Invalid revision"));
                }
                (
                    "edit_my_message",
                    Method::PUT,
                    format!("mcp/account/messages/{message_id}/text"),
                    Some(conversation_id),
                    Some(message_id),
                )
            }
        };
        if conversation.is_some_and(|id| id < 1)
            || message.is_some_and(|id| id < 1)
            || !is_account_username(content.recipient_username)
            || !is_user_id(content.recipient_user_id)
            || content.recipient_user_id == access.subject
            || !valid_outgoing_text(content.text, 10_000, true)
        {
            return Err(invalid_input(op, "Invalid audience, message or text"));
        }
        let mut body = Body {
            content,
            conversation_id: None,
            listing_id: None,
            expected_revision: None,
        };
        match action {
            MessageAction::Start { listing_id } => body.listing_id = listing_id,
            MessageAction::Send { .. } => {}
            MessageAction::Reply {
                conversation_id, ..
            } => body.conversation_id = Some(conversation_id),
            MessageAction::Edit {
                conversation_id,
                expected_revision,
                ..
            } => {
                body.conversation_id = Some(conversation_id);
                body.expected_revision = Some(expected_revision);
            }
        }
        let request = self
            .action_request(op, MESSAGES_WRITE_SCOPE, access, method, &path, key)?
            .json(&body);
        self.direct_receipt(op, request, |r: &MessageActionReceipt| {
            if !is_action_id(&r.action_id) {
                return false;
            }
            match (r.conversation_id, r.message_id) {
                (None, None) => r.status != AccountWatchlistActionStatus::Executed,
                (Some(cid), Some(mid)) => {
                    cid > 0
                        && mid > 0
                        && conversation.is_none_or(|expected| cid == expected)
                        && match action {
                            MessageAction::Edit { message_id, .. } => mid == message_id,
                            MessageAction::Reply { parent_id, .. } => mid != parent_id,
                            _ => true,
                        }
                }
                _ => false,
            }
        })
        .await
    }
}
