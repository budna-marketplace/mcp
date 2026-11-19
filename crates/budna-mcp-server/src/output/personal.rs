use super::account::WatchlistActionStatusOutput;
use super::{ProjectionBudget, UNTRUSTED_CONTENT_NOTICE};
use rmcp::schemars;
use serde::Serialize;

#[derive(Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SettingOutput {
    Pinned,
    Archived,
    Muted,
}

impl From<budna_mcp_client::ConversationSetting> for SettingOutput {
    fn from(s: budna_mcp_client::ConversationSetting) -> Self {
        use budna_mcp_client::ConversationSetting;
        match s {
            ConversationSetting::Pinned => Self::Pinned,
            ConversationSetting::Archived => Self::Archived,
            ConversationSetting::Muted => Self::Muted,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ConversationSettingsOutput {
    pub conversation_id: i64,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub is_muted: bool,
}
impl From<budna_mcp_client::ConversationSettings> for ConversationSettingsOutput {
    fn from(value: budna_mcp_client::ConversationSettings) -> Self {
        Self {
            conversation_id: value.conversation_id,
            is_pinned: value.is_pinned,
            is_archived: value.is_archived,
            is_muted: value.is_muted,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ConversationSettingReceiptOutput {
    pub action_id: String,
    pub status: WatchlistActionStatusOutput,
    pub conversation_id: i64,
    pub setting: SettingOutput,
    pub enabled: bool,
}
impl From<budna_mcp_client::ConversationSettingReceipt> for ConversationSettingReceiptOutput {
    fn from(value: budna_mcp_client::ConversationSettingReceipt) -> Self {
        Self {
            action_id: value.action_id,
            status: value.status.into(),
            conversation_id: value.conversation_id,
            setting: value.setting.into(),
            enabled: value.enabled,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct MessageCopyReceiptOutput {
    pub action_id: String,
    pub status: WatchlistActionStatusOutput,
    pub message_id: i64,
}
impl From<budna_mcp_client::MessageCopyReceipt> for MessageCopyReceiptOutput {
    fn from(value: budna_mcp_client::MessageCopyReceipt) -> Self {
        Self {
            action_id: value.action_id,
            status: value.status.into(),
            message_id: value.message_id,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListingChatPreferenceOutput {
    pub listing_id: String,
    pub muted: bool,
}
impl From<budna_mcp_client::ListingChatPreference> for ListingChatPreferenceOutput {
    fn from(value: budna_mcp_client::ListingChatPreference) -> Self {
        Self {
            listing_id: value.listing_id,
            muted: value.muted,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListingChatPreferenceReceiptOutput {
    pub action_id: String,
    pub status: WatchlistActionStatusOutput,
    pub listing_id: String,
    pub muted: bool,
}
impl From<budna_mcp_client::ListingChatPreferenceReceipt> for ListingChatPreferenceReceiptOutput {
    fn from(value: budna_mcp_client::ListingChatPreferenceReceipt) -> Self {
        Self {
            action_id: value.action_id,
            status: value.status.into(),
            listing_id: value.listing_id,
            muted: value.muted,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ProfileEditReceiptOutput {
    pub action_id: String,
    pub status: WatchlistActionStatusOutput,
}
impl From<budna_mcp_client::ProfileEditReceipt> for ProfileEditReceiptOutput {
    fn from(value: budna_mcp_client::ProfileEditReceipt) -> Self {
        Self {
            action_id: value.action_id,
            status: value.status.into(),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct FollowStateOutput {
    pub user_id: String,
    pub username: String,
    pub following: bool,
}
impl From<budna_mcp_client::FollowState> for FollowStateOutput {
    fn from(value: budna_mcp_client::FollowState) -> Self {
        Self {
            user_id: value.user_id,
            username: value.username,
            following: value.following,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct FollowReceiptOutput {
    pub action_id: String,
    pub status: WatchlistActionStatusOutput,
    pub following: bool,
}
impl From<budna_mcp_client::FollowReceipt> for FollowReceiptOutput {
    fn from(value: budna_mcp_client::FollowReceipt) -> Self {
        Self {
            action_id: value.action_id,
            status: value.status.into(),
            following: value.following,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ProfileAuthoringOutput {
    pub display_name: String,
    pub bio: Option<String>,
    pub public_profile: bool,
    pub revision: i64,
    pub truncated: bool,
    pub untrusted_content_notice: &'static str,
}
impl From<budna_mcp_client::ProfileAuthoring> for ProfileAuthoringOutput {
    fn from(value: budna_mcp_client::ProfileAuthoring) -> Self {
        let mut budget = ProjectionBudget::default();
        Self {
            display_name: budget.text(value.display_name, 4096),
            bio: budget.optional_text(value.bio, 8192),
            public_profile: value.public_profile,
            revision: value.revision,
            truncated: budget.truncated,
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct OutgoingBlockOutput {
    pub user_id: String,
    pub username: String,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct OutgoingBlocksOutput {
    #[schemars(length(max = 50))]
    pub items: Vec<OutgoingBlockOutput>,
    pub next_after: Option<String>,
    pub untrusted_content_notice: &'static str,
}
impl From<budna_mcp_client::OutgoingBlocks> for OutgoingBlocksOutput {
    fn from(value: budna_mcp_client::OutgoingBlocks) -> Self {
        let mut budget = ProjectionBudget::default();
        Self {
            items: value
                .items
                .into_iter()
                .map(|i| OutgoingBlockOutput {
                    user_id: i.user_id,
                    username: budget.text(i.username, 256),
                })
                .collect(),
            next_after: value.next_after,
            untrusted_content_notice: UNTRUSTED_CONTENT_NOTICE,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct PersonalBlockReceiptOutput {
    pub action_id: String,
    pub status: WatchlistActionStatusOutput,
    pub user_id: String,
    pub blocked: bool,
}
impl From<budna_mcp_client::PersonalBlockReceipt> for PersonalBlockReceiptOutput {
    fn from(value: budna_mcp_client::PersonalBlockReceipt) -> Self {
        Self {
            action_id: value.action_id,
            status: value.status.into(),
            user_id: value.user_id,
            blocked: value.blocked,
        }
    }
}
