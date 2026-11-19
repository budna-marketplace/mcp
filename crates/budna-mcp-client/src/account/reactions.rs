use super::{
    AccountApiClient, AccountWatchlistActionStatus, ClientError, DelegatedAccess,
    MESSAGE_REACTIONS_WRITE_SCOPE, MESSAGES_READ_SCOPE, invalid_input, invalid_response,
    is_action_id,
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Deserialize, Serialize, schemars::JsonSchema,
)]
pub enum ReactionEmoji {
    #[serde(rename = "👍")]
    ThumbsUp,
    #[serde(rename = "👎")]
    ThumbsDown,
    #[serde(rename = "❤️")]
    Heart,
    #[serde(rename = "😂")]
    Laugh,
    #[serde(rename = "😮")]
    Surprised,
    #[serde(rename = "😢")]
    Sad,
    #[serde(rename = "🎉")]
    Celebrate,
}
#[derive(Debug, Deserialize)]
pub struct MessageReactionCount {
    pub emoji: ReactionEmoji,
    pub count: i64,
}
#[derive(Debug, Deserialize)]
pub struct MessageReactionView {
    pub conversation_id: i64,
    pub message_id: i64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub reaction: Option<ReactionEmoji>,
    pub counts: Vec<MessageReactionCount>,
}
#[derive(Debug, Deserialize)]
pub struct MessageReactionReceipt {
    pub action_id: String,
    pub conversation_id: i64,
    pub message_id: i64,
    #[serde(deserialize_with = "Option::deserialize")]
    pub reaction: Option<ReactionEmoji>,
    pub status: AccountWatchlistActionStatus,
}

fn reaction_path(op: &'static str, conversation: i64, message: i64) -> Result<String, ClientError> {
    if conversation < 1 || message < 1 {
        return Err(invalid_input(op, "Invalid conversation or message"));
    }
    Ok(format!(
        "mcp/account/conversations/{conversation}/messages/{message}/reaction"
    ))
}

impl AccountApiClient {
    pub async fn message_reaction(
        &self,
        access: &DelegatedAccess,
        conversation: i64,
        message: i64,
    ) -> Result<MessageReactionView, ClientError> {
        const OP: &str = "get_my_message_reaction";
        self.require_scope(access, MESSAGES_READ_SCOPE, OP)?;
        if !self.write_permissions().message_reactions {
            return Err(invalid_input(OP, "Message reactions are unavailable"));
        }
        let view: MessageReactionView = self
            .get(OP, &reaction_path(OP, conversation, message)?, access, &[])
            .await?;
        let mut seen = BTreeSet::new();
        if view.conversation_id != conversation
            || view.message_id != message
            || view.counts.len() > 7
            || view
                .counts
                .iter()
                .any(|item| item.count <= 0 || !seen.insert(item.emoji))
            || view.reaction.is_some_and(|emoji| !seen.contains(&emoji))
        {
            return Err(invalid_response(OP));
        }
        Ok(view)
    }
    pub async fn set_message_reaction(
        &self,
        access: &DelegatedAccess,
        conversation: i64,
        message: i64,
        reaction: Option<ReactionEmoji>,
        key: &str,
    ) -> Result<MessageReactionReceipt, ClientError> {
        const OP: &str = "set_my_message_reaction";
        let request = self
            .action_request(
                OP,
                MESSAGE_REACTIONS_WRITE_SCOPE,
                access,
                Method::PUT,
                &reaction_path(OP, conversation, message)?,
                key,
            )?
            .json(&serde_json::json!({"reaction":reaction}));
        self.direct_receipt(OP, request, |r: &MessageReactionReceipt| {
            is_action_id(&r.action_id)
                && r.message_id == message
                && r.conversation_id == conversation
                && r.reaction == reaction
        })
        .await
    }
}
