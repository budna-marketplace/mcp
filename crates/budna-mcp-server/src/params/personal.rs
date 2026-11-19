use super::*;

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SettingParam {
    Pinned,
    Archived,
    Muted,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConversationIdParams {
    #[schemars(range(min = 1))]
    pub conversation_id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConversationSettingParams {
    #[schemars(range(min = 1))]
    pub conversation_id: i64,
    pub setting: SettingParam,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MessageCopyParams {
    #[schemars(range(min = 1))]
    pub message_id: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListingChatMuteParams {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    pub listing_id: String,
    pub muted: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EditProfileParams {
    #[schemars(range(min = 1))]
    pub expected_revision: i64,
    #[schemars(length(min = 1, max = 100))]
    #[serde(default, deserialize_with = "supplied_text")]
    pub display_name: Option<String>,
    #[schemars(length(max = 1000))]
    #[serde(default, deserialize_with = "supplied_text")]
    pub bio: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FollowingStateParams {
    #[schemars(length(min = 1, max = 64))]
    pub username: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SetFollowingParams {
    #[schemars(length(min = 1, max = 64))]
    pub username: String,
    #[schemars(length(min=41,max=41),regex(pattern=USER_ID_PATTERN))]
    pub expected_user_id: String,
    pub following: bool,
}

fn supplied_text<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    String::deserialize(d).map(Some)
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutgoingBlockParams {
    #[schemars(range(min = 1, max = 50))]
    pub limit: Option<u32>,
    #[schemars(length(min=41,max=41),regex(pattern=USER_ID_PATTERN))]
    pub after: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PersonalBlockParams {
    #[schemars(length(min=41,max=41),regex(pattern=USER_ID_PATTERN))]
    pub user_id: String,
    pub blocked: bool,
}
