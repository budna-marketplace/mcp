use super::messages::ROOM_PATTERN;
use super::*;
use budna_mcp_client::{ReactionEmoji, RoomTargetKind};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RoomTargetsParams {
    #[schemars(regex(pattern=ROOM_PATTERN))]
    pub room_id: String,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
    #[schemars(range(min = 1))]
    pub before_message_id: Option<i64>,
    #[schemars(range(min = 1))]
    pub before_bid_id: Option<i64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RoomModerationParticipantsParams {
    #[schemars(regex(pattern=ROOM_PATTERN))]
    pub room_id: String,
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
    #[schemars(regex(pattern=USER_ID_PATTERN))]
    pub after_user_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RoomReactParams {
    #[schemars(regex(pattern=ROOM_PATTERN))]
    pub room_id: String,

    pub target_kind: RoomTargetKind,
    #[schemars(range(min = 1))]
    pub target_id: i64,

    pub emoji: ReactionEmoji,

    pub reacted: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RoomPinParams {
    #[schemars(regex(pattern=ROOM_PATTERN))]
    pub room_id: String,
    #[schemars(range(min = 1))]
    pub message_id: i64,

    pub pinned: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RoomParticipantReportParams {
    #[schemars(regex(pattern=ROOM_PATTERN))]
    pub room_id: String,
    #[schemars(regex(pattern=USER_ID_PATTERN))]
    pub user_id: String,
    #[schemars(range(min = 1))]
    pub message_id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RoomMemberParams {
    #[schemars(regex(pattern=ROOM_PATTERN))]
    pub room_id: String,
    #[schemars(regex(pattern=USER_ID_PATTERN))]
    pub user_id: String,
}
