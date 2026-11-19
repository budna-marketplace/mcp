use super::*;
use budna_mcp_client::ReactionEmoji;

// A submitted null clears a reaction. An omitted field is invalid.
fn nullable_reaction(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    generator.subschema_for::<Option<ReactionEmoji>>()
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReactionTargetParams {
    #[schemars(range(min = 1))]
    pub conversation_id: i64,
    #[schemars(range(min = 1))]
    pub message_id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetReactionParams {
    #[schemars(range(min = 1))]
    pub conversation_id: i64,
    #[schemars(range(min = 1))]
    pub message_id: i64,
    #[serde(deserialize_with = "Option::deserialize")]
    #[schemars(schema_with = "nullable_reaction")]
    pub reaction: Option<ReactionEmoji>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StartConversationParams {
    #[schemars(length(min = 1, max = 64))]
    pub recipient_username: String,
    #[schemars(length(min=41,max=41),regex(pattern=USER_ID_PATTERN))]
    pub recipient_user_id: String,
    #[schemars(length(min = 1, max = 10000))]
    pub text: String,
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    pub listing_id: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SendMessageParams {
    #[schemars(length(min = 1, max = 64))]
    pub recipient_username: String,
    #[schemars(length(min=41,max=41),regex(pattern=USER_ID_PATTERN))]
    pub recipient_user_id: String,
    #[schemars(length(min = 1, max = 10000))]
    pub text: String,
    #[schemars(range(min = 1))]
    pub conversation_id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplyMessageParams {
    #[schemars(length(min = 1, max = 64))]
    pub recipient_username: String,
    #[schemars(length(min=41,max=41),regex(pattern=USER_ID_PATTERN))]
    pub recipient_user_id: String,
    #[schemars(length(min = 1, max = 10000))]
    pub text: String,
    #[schemars(range(min = 1))]
    pub conversation_id: i64,
    #[schemars(range(min = 1))]
    pub message_id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EditMessageParams {
    #[schemars(length(min = 1, max = 64))]
    pub recipient_username: String,
    #[schemars(length(min=41,max=41),regex(pattern=USER_ID_PATTERN))]
    pub recipient_user_id: String,
    #[schemars(length(min = 1, max = 10000))]
    pub text: String,
    #[schemars(range(min = 1))]
    pub conversation_id: i64,
    #[schemars(range(min = 1))]
    pub message_id: i64,
    #[schemars(range(min = 1))]
    pub expected_revision: i64,
}

const ROOM_PATTERN: &str =
    "^(listing|showcase|live)_[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$";
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RoomParams {
    #[schemars(regex(pattern=ROOM_PATTERN))]
    pub room_id: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PostRoomParams {
    #[schemars(regex(pattern=ROOM_PATTERN))]
    pub room_id: String,
    pub audience: budna_mcp_client::PublicChatAudience,
    #[schemars(length(min = 1, max = 500))]
    pub text: String,
    #[schemars(range(min = 1))]
    pub reply_to_message_id: Option<i64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RoomMessageParams {
    #[schemars(regex(pattern=ROOM_PATTERN))]
    pub room_id: String,
    #[schemars(range(min = 1))]
    pub message_id: i64,
}
