use super::account::WatchlistActionStatusOutput;
use budna_mcp_client::ReactionEmoji;
use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ReactionCountOutput {
    pub emoji: ReactionEmoji,
    #[schemars(range(min = 1))]
    pub count: i64,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct MessageRecipientOutput {
    pub username: String,
    pub user_id: String,
}
impl From<budna_mcp_client::MessageRecipient> for MessageRecipientOutput {
    fn from(r: budna_mcp_client::MessageRecipient) -> Self {
        Self {
            username: r.username,
            user_id: r.user_id,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ConversationAudienceOutput {
    pub conversation_id: i64,
    pub recipient_username: String,
    pub recipient_user_id: String,
    pub listing_id: Option<String>,
}
impl From<budna_mcp_client::ConversationAudience> for ConversationAudienceOutput {
    fn from(r: budna_mcp_client::ConversationAudience) -> Self {
        Self {
            conversation_id: r.conversation_id,
            recipient_username: r.recipient_username,
            recipient_user_id: r.recipient_user_id,
            listing_id: r.listing_id,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct MessageAuthoringOutput {
    pub conversation_id: i64,
    pub message_id: i64,
    pub revision: i64,
    pub can_edit: bool,
}
impl From<budna_mcp_client::MessageAuthoring> for MessageAuthoringOutput {
    fn from(r: budna_mcp_client::MessageAuthoring) -> Self {
        Self {
            conversation_id: r.conversation_id,
            message_id: r.message_id,
            revision: r.revision,
            can_edit: r.can_edit,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct MessageActionReceiptOutput {
    pub action_id: String,
    pub conversation_id: Option<i64>,
    pub message_id: Option<i64>,
    pub status: WatchlistActionStatusOutput,
}
impl From<budna_mcp_client::MessageActionReceipt> for MessageActionReceiptOutput {
    fn from(r: budna_mcp_client::MessageActionReceipt) -> Self {
        Self {
            action_id: r.action_id,
            conversation_id: r.conversation_id,
            message_id: r.message_id,
            status: r.status.into(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct MessageReactionReceiptOutput {
    pub action_id: String,
    pub conversation_id: i64,
    pub message_id: i64,
    pub reaction: Option<ReactionEmoji>,
    pub status: WatchlistActionStatusOutput,
}
impl From<budna_mcp_client::MessageReactionReceipt> for MessageReactionReceiptOutput {
    fn from(r: budna_mcp_client::MessageReactionReceipt) -> Self {
        Self {
            action_id: r.action_id,
            conversation_id: r.conversation_id,
            message_id: r.message_id,
            reaction: r.reaction,
            status: r.status.into(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct MessageReactionViewOutput {
    pub conversation_id: i64,
    pub message_id: i64,
    pub reaction: Option<ReactionEmoji>,
    #[schemars(length(max = 7))]
    pub counts: Vec<ReactionCountOutput>,
}
impl From<budna_mcp_client::MessageReactionView> for MessageReactionViewOutput {
    fn from(r: budna_mcp_client::MessageReactionView) -> Self {
        Self {
            conversation_id: r.conversation_id,
            message_id: r.message_id,
            reaction: r.reaction,
            counts: r
                .counts
                .into_iter()
                .map(|i| ReactionCountOutput {
                    emoji: i.emoji,
                    count: i.count,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PublicChatPostingOutput {
    pub room_id: String,
    pub audience: budna_mcp_client::PublicChatAudience,
    pub role: budna_mcp_client::PublicChatRole,
    pub can_post: bool,
    pub blocker: Option<budna_mcp_client::PublicChatBlocker>,
    pub cooldown_seconds: u32,
    pub max_text_length: u32,
    #[schemars(length(max = 50))]
    pub replyable_message_ids: Vec<i64>,
}
impl From<budna_mcp_client::PublicChatPosting> for PublicChatPostingOutput {
    fn from(r: budna_mcp_client::PublicChatPosting) -> Self {
        Self {
            room_id: r.room_id,
            audience: r.audience,
            role: r.role,
            can_post: r.can_post,
            blocker: r.blocker,
            cooldown_seconds: r.cooldown_seconds,
            max_text_length: r.max_text_length,
            replyable_message_ids: r.replyable_message_ids,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PublicChatReceiptOutput {
    pub room_id: String,
    pub action_id: String,
    pub status: WatchlistActionStatusOutput,
    pub message_id: Option<i64>,
}
impl From<budna_mcp_client::PublicChatReceipt> for PublicChatReceiptOutput {
    fn from(r: budna_mcp_client::PublicChatReceipt) -> Self {
        Self {
            room_id: r.room_id,
            action_id: r.action_id,
            status: r.status.into(),
            message_id: r.message_id,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PublicRoomMessageOutput {
    pub room_id: String,
    pub message_id: i64,
    pub text: String,
    pub reply_to_message_id: Option<i64>,
    pub is_host: bool,
    pub truncated: bool,
    pub untrusted_content_notice: &'static str,
}
impl From<budna_mcp_client::PublicRoomMessage> for PublicRoomMessageOutput {
    fn from(r: budna_mcp_client::PublicRoomMessage) -> Self {
        let mut budget = super::ProjectionBudget::default();
        Self {
            room_id: r.room_id,
            message_id: r.message_id,
            text: budget.text(r.text, 2048),
            reply_to_message_id: r.reply_to_message_id,
            is_host: r.is_host,
            truncated: budget.truncated,
            untrusted_content_notice: super::UNTRUSTED_CONTENT_NOTICE,
        }
    }
}
