use super::*;
use crate::{
    output::messages::*,
    params::{
        messages::*,
        personal::{ConversationIdParams, FollowingStateParams, MessageCopyParams},
    },
};
use budna_mcp_client::{AddressedText, MessageAction};
#[tool_router(router=direct_messages_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Resolve a private message recipient",
        description = "Resolve one exact public handle to its stable public user ID with messages:read before starting a private conversation.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_message_recipient(
        &self,
        Parameters(input): Parameters<SafeToolParams<FollowingStateParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<MessageRecipientOutput>, CallToolResult> {
        const OP: &str = "get_my_message_recipient";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, self.client.message_recipient(&access, &input.username))
            .await
    }

    #[tool(
        title = "Check who will receive my message",
        description = "Read the current other participant and optional listing in one owned conversation with messages:read. This read has no effect on read receipts.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_conversation_audience(
        &self,
        Parameters(input): Parameters<SafeToolParams<ConversationIdParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ConversationAudienceOutput>, CallToolResult> {
        const OP: &str = "get_my_conversation_audience";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .conversation_audience(&access, input.conversation_id),
        )
        .await
    }

    #[tool(
        title = "Check whether I can edit a message",
        description = "Read the revision and edit eligibility for one message visible to the connected account with messages:read. Preserve this revision when editing.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_message_authoring(
        &self,
        Parameters(input): Parameters<SafeToolParams<MessageCopyParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<MessageAuthoringOutput>, CallToolResult> {
        const OP: &str = "get_my_message_authoring";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, self.client.message_authoring(&access, input.message_id))
            .await
    }

    #[tool(
        title = "Start a private conversation",
        description = "Send the exact requested first message to one resolved username and public ID. May notify the recipient. Include an optional listing only when relevant. Requires messages:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn start_my_conversation(
        &self,
        Parameters(input): Parameters<SafeToolParams<WithRequestId<StartConversationParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<MessageActionReceiptOutput>, CallToolResult> {
        const OP: &str = "start_my_conversation";
        let WithRequestId {
            fields: input,
            request_id,
        } = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.write_message(
                &access,
                MessageAction::Start {
                    listing_id: input.listing_id.as_deref(),
                },
                AddressedText {
                    recipient_username: &input.recipient_username,
                    recipient_user_id: &input.recipient_user_id,
                    text: &input.text,
                },
                &request_id,
            ),
        )
        .await
    }

    #[tool(
        title = "Send a private message",
        description = "Send the exact requested text to the confirmed participant in one existing private thread. May unarchive the conversation and notify the recipient. Requires messages:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn send_my_message(
        &self,
        Parameters(input): Parameters<SafeToolParams<WithRequestId<SendMessageParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<MessageActionReceiptOutput>, CallToolResult> {
        const OP: &str = "send_my_message";
        let WithRequestId {
            fields: input,
            request_id,
        } = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.write_message(
                &access,
                MessageAction::Send {
                    conversation_id: input.conversation_id,
                },
                AddressedText {
                    recipient_username: &input.recipient_username,
                    recipient_user_id: &input.recipient_user_id,
                    text: &input.text,
                },
                &request_id,
            ),
        )
        .await
    }

    #[tool(
        title = "Reply to a private message",
        description = "Send the exact requested reply under one visible parent message in the selected private conversation. The other participant can receive a notification. Requires messages:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn reply_to_my_message(
        &self,
        Parameters(input): Parameters<SafeToolParams<WithRequestId<ReplyMessageParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<MessageActionReceiptOutput>, CallToolResult> {
        const OP: &str = "reply_to_my_message";
        let WithRequestId {
            fields: input,
            request_id,
        } = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.write_message(
                &access,
                MessageAction::Reply {
                    conversation_id: input.conversation_id,
                    parent_id: input.message_id,
                },
                AddressedText {
                    recipient_username: &input.recipient_username,
                    recipient_user_id: &input.recipient_user_id,
                    text: &input.text,
                },
                &request_id,
            ),
        )
        .await
    }

    #[tool(
        title = "Edit a private message I sent",
        description = "Replace the text of one message you sent within the allowed edit window. The recipient sees the replacement. Check the audience and current revision first. Requires messages:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn edit_my_message(
        &self,
        Parameters(input): Parameters<SafeToolParams<WithRequestId<EditMessageParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<MessageActionReceiptOutput>, CallToolResult> {
        const OP: &str = "edit_my_message";
        let WithRequestId {
            fields: input,
            request_id,
        } = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.write_message(
                &access,
                MessageAction::Edit {
                    conversation_id: input.conversation_id,
                    message_id: input.message_id,
                    expected_revision: input.expected_revision,
                },
                AddressedText {
                    recipient_username: &input.recipient_username,
                    recipient_user_id: &input.recipient_user_id,
                    text: &input.text,
                },
                &request_id,
            ),
        )
        .await
    }
}
