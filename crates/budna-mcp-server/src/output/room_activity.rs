use super::account::WatchlistActionStatusOutput;
use budna_mcp_client::{ReactionEmoji, RoomActionName, RoomBanScope, RoomReceiptTarget};
use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct RoomReceiptOutput {
    pub room_id: String,
    pub action_id: String,
    pub action: RoomActionName,
    pub target_kind: RoomReceiptTarget,
    pub target_id: String,
    pub status: WatchlistActionStatusOutput,
}
impl From<budna_mcp_client::RoomReceipt> for RoomReceiptOutput {
    fn from(r: budna_mcp_client::RoomReceipt) -> Self {
        Self {
            room_id: r.room_id,
            action_id: r.action_id,
            action: r.action,
            target_kind: r.target_kind,
            target_id: r.target_id,
            status: r.status.into(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct RoomTallyOutput {
    pub emoji: ReactionEmoji,
    pub count: i64,
    pub mine: bool,
}
impl From<budna_mcp_client::RoomTally> for RoomTallyOutput {
    fn from(r: budna_mcp_client::RoomTally) -> Self {
        Self {
            emoji: r.emoji,
            count: r.count,
            mine: r.mine,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct RoomReactionTargetOutput {
    pub id: i64,
    #[schemars(length(max = 7))]
    pub counts: Vec<RoomTallyOutput>,
}
impl From<budna_mcp_client::RoomReactionTarget> for RoomReactionTargetOutput {
    fn from(r: budna_mcp_client::RoomReactionTarget) -> Self {
        Self {
            id: r.id,
            counts: r.counts.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct RoomTargetsOutput {
    pub room_id: String,
    #[schemars(length(max = 50))]
    pub messages: Vec<RoomReactionTargetOutput>,
    #[schemars(length(max = 50))]
    pub bids: Vec<RoomReactionTargetOutput>,
}
impl From<budna_mcp_client::RoomTargets> for RoomTargetsOutput {
    fn from(r: budna_mcp_client::RoomTargets) -> Self {
        Self {
            room_id: r.room_id,
            messages: r.messages.into_iter().map(Into::into).collect(),
            bids: r.bids.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct RoomParticipantOutput {
    pub user_id: String,
    pub banned: bool,
    pub expires_at: Option<i64>,
}
impl From<budna_mcp_client::RoomParticipant> for RoomParticipantOutput {
    fn from(r: budna_mcp_client::RoomParticipant) -> Self {
        Self {
            user_id: r.user_id,
            banned: r.banned,
            expires_at: r.expires_at,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct RoomModerationViewOutput {
    pub room_id: String,
    pub ban_scope: RoomBanScope,
    #[schemars(length(max = 50))]
    pub participants: Vec<RoomParticipantOutput>,
}
impl From<budna_mcp_client::RoomModerationView> for RoomModerationViewOutput {
    fn from(r: budna_mcp_client::RoomModerationView) -> Self {
        Self {
            room_id: r.room_id,
            ban_scope: r.ban_scope,
            participants: r.participants.into_iter().map(Into::into).collect(),
        }
    }
}
