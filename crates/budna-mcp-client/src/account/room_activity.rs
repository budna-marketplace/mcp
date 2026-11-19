use super::{
    AccountApiClient, AccountWatchlistActionStatus, CHAT_ROOMS_READ_SCOPE, ClientError,
    DelegatedAccess, LIVE_REPORTS_WRITE_SCOPE, ROOM_MODERATION_READ_SCOPE,
    ROOM_MODERATION_WRITE_SCOPE, ROOM_REACTIONS_WRITE_SCOPE, ReactionEmoji, invalid_input,
    invalid_response, is_action_id, is_user_id, public_chat::valid_room,
};
use crate::is_live_stream_id;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Debug, Clone, Copy, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomTargetKind {
    Message,
    Bid,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomReceiptTarget {
    Message,
    Bid,
    Participant,
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoomBanScope {
    Room,
    SellerLiveChats,
}
#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoomActionName {
    SetPublicRoomReaction,
    ReportLiveChatPost,
    SetRoomMessagePinned,
    HideRoomMessage,
    ReportRoomParticipant,
    BanRoomParticipant,
    UnbanRoomParticipant,
}
#[derive(Debug, Deserialize)]
pub struct RoomTally {
    pub emoji: ReactionEmoji,
    pub count: i64,
    pub mine: bool,
}
#[derive(Debug, Deserialize)]
pub struct RoomReactionTarget {
    pub id: i64,
    pub counts: Vec<RoomTally>,
}
#[derive(Debug, Deserialize)]
pub struct RoomTargets {
    pub room_id: String,
    pub messages: Vec<RoomReactionTarget>,
    pub bids: Vec<RoomReactionTarget>,
}
#[derive(Debug, Deserialize)]
pub struct RoomParticipant {
    pub user_id: String,
    pub banned: bool,
    pub expires_at: Option<i64>,
}
#[derive(Debug, Deserialize)]
pub struct RoomModerationView {
    pub room_id: String,
    pub ban_scope: RoomBanScope,
    pub participants: Vec<RoomParticipant>,
}
#[derive(Debug, Deserialize)]
pub struct RoomReceipt {
    pub action_id: String,
    pub room_id: String,
    pub action: RoomActionName,
    pub target_kind: RoomReceiptTarget,
    pub target_id: String,
    pub status: AccountWatchlistActionStatus,
}

#[derive(Debug)]
pub enum RoomOperation<'a> {
    Reaction {
        target_kind: RoomTargetKind,
        target_id: i64,
        emoji: ReactionEmoji,
        reacted: bool,
    },
    ReportPost {
        message_id: i64,
    },
    Pin {
        message_id: i64,
        pinned: bool,
    },
    Hide {
        message_id: i64,
    },
    ReportParticipant {
        user_id: &'a str,
        message_id: i64,
    },
    Ban {
        user_id: &'a str,
    },
    Unban {
        user_id: &'a str,
    },
}
impl AccountApiClient {
    pub async fn room_targets(
        &self,
        access: &DelegatedAccess,
        room: &str,
        limit: u32,
        before_message: Option<i64>,
        before_bid: Option<i64>,
    ) -> Result<RoomTargets, ClientError> {
        const OP: &str = "list_my_room_reaction_targets";
        self.require_scope(access, CHAT_ROOMS_READ_SCOPE, OP)?;
        if !valid_room(room)
            || !(1..=50).contains(&limit)
            || [before_message, before_bid]
                .into_iter()
                .flatten()
                .any(|id| id < 1)
        {
            return Err(invalid_input(OP, "Invalid room, limit or cursor"));
        }
        let mut query = vec![("limit", limit.to_string())];
        if let Some(id) = before_message {
            query.push(("before_message_id", id.to_string()));
        }
        if let Some(id) = before_bid {
            query.push(("before_bid_id", id.to_string()));
        }
        let result: RoomTargets = self
            .get(
                OP,
                &format!("mcp/account/chat-rooms/{room}/reaction-targets"),
                access,
                &query,
            )
            .await?;
        if result.room_id != room {
            return Err(invalid_response(OP));
        }
        for (rows, before) in [
            (&result.messages, before_message),
            (&result.bids, before_bid),
        ] {
            let mut ids = BTreeSet::new();
            if rows.len() > limit as usize
                || rows.iter().any(|row| {
                    let mut emojis = BTreeSet::new();
                    row.id < 1
                        || before.is_some_and(|cur| row.id >= cur)
                        || !ids.insert(row.id)
                        || row.counts.len() > 7
                        || row
                            .counts
                            .iter()
                            .any(|t| t.count < 1 || !emojis.insert(t.emoji))
                })
            {
                return Err(invalid_response(OP));
            }
        }
        Ok(result)
    }
    pub async fn room_moderation_state(
        &self,
        access: &DelegatedAccess,
        room: &str,
        limit: u32,
        after: Option<&str>,
    ) -> Result<RoomModerationView, ClientError> {
        const OP: &str = "list_my_room_moderation_participants";
        self.require_scope(access, ROOM_MODERATION_READ_SCOPE, OP)?;
        if !valid_room(room) || !(1..=50).contains(&limit) || after.is_some_and(|s| !is_user_id(s))
        {
            return Err(invalid_input(OP, "Invalid room, limit or cursor"));
        }
        let mut query = vec![("limit", limit.to_string())];
        if let Some(id) = after {
            query.push(("after_user_id", id.to_owned()));
        }
        let result: RoomModerationView = self
            .get(
                OP,
                &format!("mcp/account/chat-rooms/{room}/moderation"),
                access,
                &query,
            )
            .await?;
        let mut ids = BTreeSet::new();
        if result.room_id != room
            || (result.ban_scope == RoomBanScope::SellerLiveChats) != is_live_stream_id(room)
            || result.participants.len() > limit as usize
            || result.participants.iter().any(|p| {
                !is_user_id(&p.user_id)
                    || Some(p.user_id.as_str()) == after
                    || !ids.insert(&p.user_id)
                    || p.expires_at.is_some_and(|t| t <= 0)
            })
        {
            return Err(invalid_response(OP));
        }
        Ok(result)
    }
    pub async fn change_room_activity(
        &self,
        access: &DelegatedAccess,
        room: &str,
        operation: RoomOperation<'_>,
        key: &str,
    ) -> Result<RoomReceipt, ClientError> {
        use RoomActionName as Name;
        use RoomReceiptTarget as Target;
        use serde_json::json;
        let (op, action, kind, id, body) = match operation {
            RoomOperation::Reaction {
                target_kind,
                target_id,
                emoji,
                reacted,
            } => (
                "set_my_room_reaction",
                Name::SetPublicRoomReaction,
                match target_kind {
                    RoomTargetKind::Message => Target::Message,
                    RoomTargetKind::Bid => Target::Bid,
                },
                target_id.to_string(),
                json!({ "target_kind":target_kind,"target_id":target_id,"emoji":emoji,"reacted":reacted }),
            ),
            RoomOperation::ReportPost { message_id } => (
                "report_live_chat_post",
                Name::ReportLiveChatPost,
                Target::Message,
                message_id.to_string(),
                json!({"message_id":message_id}),
            ),
            RoomOperation::Pin { message_id, pinned } => (
                "set_my_room_message_pinned",
                Name::SetRoomMessagePinned,
                Target::Message,
                message_id.to_string(),
                json!({"operation":"set_pinned","message_id":message_id,"pinned":pinned}),
            ),
            RoomOperation::Hide { message_id } => (
                "hide_my_room_message",
                Name::HideRoomMessage,
                Target::Message,
                message_id.to_string(),
                json!({"operation":"hide_message","message_id":message_id}),
            ),
            RoomOperation::ReportParticipant {
                user_id,
                message_id,
            } => {
                const OP: &str = "report_my_room_participant";
                if message_id < 1 || is_live_stream_id(room) {
                    return Err(invalid_input(
                        OP,
                        "Select a post and participant in a listing or showcase room",
                    ));
                }
                (
                    OP,
                    Name::ReportRoomParticipant,
                    Target::Participant,
                    user_id.to_owned(),
                    json!({"operation":"report_participant","user_id":user_id,"message_id":message_id}),
                )
            }
            RoomOperation::Ban { user_id } => (
                "ban_my_room_participant",
                Name::BanRoomParticipant,
                Target::Participant,
                user_id.to_owned(),
                json!({"operation":"ban_participant","user_id":user_id}),
            ),
            RoomOperation::Unban { user_id } => (
                "unban_my_room_participant",
                Name::UnbanRoomParticipant,
                Target::Participant,
                user_id.to_owned(),
                json!({"operation":"unban_participant","user_id":user_id}),
            ),
        };
        if !valid_room(room)
            || (matches!(
                action,
                Name::ReportLiveChatPost | Name::UnbanRoomParticipant
            ) && !is_live_stream_id(room))
            || match kind {
                Target::Participant => !is_user_id(&id) || id == access.subject,
                Target::Message | Target::Bid => id.parse::<i64>().map_or(true, |n| n < 1),
            }
        {
            return Err(invalid_input(op, "Invalid room or target"));
        }
        let (scope, method, suffix) = match action {
            Name::SetPublicRoomReaction => (ROOM_REACTIONS_WRITE_SCOPE, Method::PUT, "reactions"),
            Name::ReportLiveChatPost => (LIVE_REPORTS_WRITE_SCOPE, Method::POST, "reports"),
            _ => (ROOM_MODERATION_WRITE_SCOPE, Method::POST, "moderation"),
        };
        let request = self
            .action_request(
                op,
                scope,
                access,
                method,
                &format!("mcp/account/chat-rooms/{room}/{suffix}"),
                key,
            )?
            .json(&body);
        self.direct_receipt(op, request, |r: &RoomReceipt| {
            is_action_id(&r.action_id)
                && r.room_id == room
                && r.action == action
                && r.target_kind == kind
                && r.target_id == id
        })
        .await
    }
}
