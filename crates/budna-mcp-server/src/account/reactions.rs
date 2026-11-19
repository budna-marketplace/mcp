use super::*;
use crate::{
    output::messages::{MessageReactionReceiptOutput, MessageReactionViewOutput},
    params::messages::{ReactionTargetParams, SetReactionParams},
};
#[tool_router(router=reactions_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Read reactions on a private message",
        description = "Read your reaction and counts on one message you can see. Requires messages:read. Reading does not mark the thread as read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_message_reaction(
        &self,
        Parameters(input): Parameters<SafeToolParams<ReactionTargetParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<MessageReactionViewOutput>, CallToolResult> {
        const OP: &str = "get_my_message_reaction";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .message_reaction(&access, input.conversation_id, input.message_id),
        )
        .await
    }

    #[tool(
        title = "Set or clear my private message reaction",
        description = "On request, set your one reaction on a visible private message or explicitly pass null to clear it. The other participant can see the change. Requires messagereactions:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn set_my_message_reaction(
        &self,
        Parameters(input): Parameters<SafeToolParams<WithRequestId<SetReactionParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<MessageReactionReceiptOutput>, CallToolResult> {
        const OP: &str = "set_my_message_reaction";
        let WithRequestId {
            fields: input,
            request_id,
        } = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.set_message_reaction(
                &access,
                input.conversation_id,
                input.message_id,
                input.reaction,
                &request_id,
            ),
        )
        .await
    }
}
