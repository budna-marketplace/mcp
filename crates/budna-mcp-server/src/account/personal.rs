use super::*;
use crate::{output::personal::*, params::personal::*};
use budna_mcp_client::ConversationSetting;

#[tool_router(router=inbox_actions_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Check personal conversation settings",
        description = "Read the connected participant’s pin, archive and mute state with messages:read. This read leaves delivery and read receipts alone.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_conversation_settings(
        &self,
        Parameters(params): Parameters<SafeToolParams<ConversationIdParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ConversationSettingsOutput>, CallToolResult> {
        const OP: &str = "get_my_conversation_settings";
        let input = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .conversation_settings(&access, input.conversation_id),
        )
        .await
    }
    #[tool(
        title = "Change a personal conversation setting",
        description = "On explicit request, change one personal pin, archive or mute setting with inbox:write. Archiving hides the conversation for this account and can be reversed.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn set_my_conversation_setting(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<ConversationSettingParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ConversationSettingReceiptOutput>, CallToolResult> {
        const OP: &str = "set_my_conversation_setting";
        let WithRequestId {
            fields: input,
            request_id,
        } = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.set_conversation_setting(
                &access,
                input.conversation_id,
                match input.setting {
                    SettingParam::Pinned => ConversationSetting::Pinned,
                    SettingParam::Archived => ConversationSetting::Archived,
                    SettingParam::Muted => ConversationSetting::Muted,
                },
                input.enabled,
                &request_id,
            ),
        )
        .await
    }
    #[tool(
        title = "Hide my copy of one message",
        description = "On explicit request, hide one message for the connected participant with inbox:write. Other participants keep their copies.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn hide_my_message_copy(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<MessageCopyParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<MessageCopyReceiptOutput>, CallToolResult> {
        const OP: &str = "hide_my_message_copy";
        let WithRequestId {
            fields: input,
            request_id,
        } = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .hide_message_copy(&access, input.message_id, &request_id),
        )
        .await
    }
    #[tool(
        title = "Check my listing chat notifications",
        description = "Read the connected account’s notification setting for one eligible listing chat with messages:read. The read does not join the chat.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_listing_chat_notification_setting(
        &self,
        Parameters(params): Parameters<SafeToolParams<ListingIdParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ListingChatPreferenceOutput>, CallToolResult> {
        const OP: &str = "get_my_listing_chat_notification_setting";
        let input = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .listing_chat_preference(&access, &input.listing_id),
        )
        .await
    }
    #[tool(
        title = "Mute or unmute my listing chat notifications",
        description = "On explicit request, change personal notifications for one eligible listing chat with inbox:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn set_my_listing_chat_muted(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<ListingChatMuteParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ListingChatPreferenceReceiptOutput>, CallToolResult> {
        const OP: &str = "set_my_listing_chat_muted";
        let WithRequestId {
            fields: input,
            request_id,
        } = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.set_listing_chat_muted(
                &access,
                &input.listing_id,
                input.muted,
                &request_id,
            ),
        )
        .await
    }
}

#[tool_router(router=profile_actions_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Read editable profile fields",
        description = "Read the current display name, bio, public visibility and revision of the connected account with profile:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_profile_authoring(
        &self,
        Parameters(params): Parameters<SafeToolParams<EmptyAccountParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ProfileAuthoringOutput>, CallToolResult> {
        const OP: &str = "get_my_profile_authoring";
        params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, self.client.profile_authoring(&access))
            .await
    }
    #[tool(
        title = "Update my display name or bio",
        description = "On explicit request change only the supplied profile fields with profile:write. Use the observed revision. An empty bio clears it. The current public visibility applies.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn update_my_public_profile(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<EditProfileParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ProfileEditReceiptOutput>, CallToolResult> {
        const OP: &str = "update_my_public_profile";
        let WithRequestId {
            fields: input,
            request_id,
        } = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.edit_public_profile(
                &access,
                input.expected_revision,
                input.display_name.as_deref(),
                input.bio.as_deref(),
                &request_id,
            ),
        )
        .await
    }
}

#[tool_router(router=following_actions_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Check whether I follow a public account",
        description = "Resolve one exact public username, its stable public account ID and the current follow relationship with following:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_following_state(
        &self,
        Parameters(params): Parameters<SafeToolParams<FollowingStateParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<FollowStateOutput>, CallToolResult> {
        const OP: &str = "get_my_following_state";
        let input = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, self.client.following_state(&access, &input.username))
            .await
    }
    #[tool(
        title = "Follow or unfollow a public account",
        description = "On explicit request, follow or unfollow one public account with following:write. Supply the matching target ID. Following can notify the other account.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn set_my_following(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<SetFollowingParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<FollowReceiptOutput>, CallToolResult> {
        const OP: &str = "set_my_following";
        let WithRequestId {
            fields: input,
            request_id,
        } = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.set_following(
                &access,
                &input.username,
                &input.expected_user_id,
                input.following,
                &request_id,
            ),
        )
        .await
    }
}
