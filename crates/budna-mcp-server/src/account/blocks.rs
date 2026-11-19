use super::*;
use crate::{
    output::personal::{OutgoingBlocksOutput, PersonalBlockReceiptOutput},
    params::personal::{OutgoingBlockParams, PersonalBlockParams},
};

#[tool_router(router=blocks_read_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "List accounts I have blocked",
        description = "Read a bounded page of the connected account’s own outgoing blocks. This separate personalblocks:read grant covers blocked public handles and IDs.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn list_my_personal_blocks(
        &self,
        Parameters(input): Parameters<SafeToolParams<OutgoingBlockParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<OutgoingBlocksOutput>, CallToolResult> {
        const OP: &str = "list_my_personal_blocks";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .outgoing_blocks(&access, input.limit.unwrap_or(20), input.after.as_deref()),
        )
        .await
    }
}
#[tool_router(router=blocks_write_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Block or unblock one public account",
        description = "On explicit request, block or unblock one account with personalblocks:write. Blocking removes follows and pauses standing automatic bids between both accounts. Unblocking restores neither. Explain these effects before submitting.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn set_my_personal_block(
        &self,
        Parameters(input): Parameters<SafeToolParams<PersonalBlockParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<PersonalBlockReceiptOutput>, CallToolResult> {
        const OP: &str = "set_my_personal_block";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.set_personal_block(
                &access,
                &input.user_id,
                input.blocked,
                &uuid::Uuid::new_v4().to_string(),
            ),
        )
        .await
    }
}
