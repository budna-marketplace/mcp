use super::*;
use crate::{output::LiveReminderReceiptOutput, params::LiveReminderParams};
use crate::{
    output::{ActionStatusOutput, PrivateWatchlistOutput},
    params::{
        AccountActionStatusParams, CreateWatchlistParams, DeleteWatchlistParams,
        RenameWatchlistParams,
    },
};
use budna_mcp_client::PrivateWatchlistChange;

#[tool_router(router = action_status_router, vis = "pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Check an account action receipt",
        description = "Look up one action on the original connection with its current write permission. Only executed confirms completion. If the receipt was lost, use Connected apps in Budna.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_action_status(
        &self,
        Parameters(input): Parameters<SafeToolParams<AccountActionStatusParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ActionStatusOutput>, CallToolResult> {
        let input = input
            .parse()
            .map_err(|e| input_error(ACTION_STATUS_TOOL, &e))?;
        let access = request_access(&context, ACTION_STATUS_TOOL)?;
        self.execute(
            ACTION_STATUS_TOOL,
            self.client.action_status(&access, &input.action_id),
        )
        .await
    }
}

#[tool_router(router = watchlist_management_router, vis = "pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Create a private watchlist",
        description = "Create one private list on the customer's explicit request with watchlists:write. Check the receipt before repeating a request.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn create_my_private_watchlist(
        &self,
        Parameters(input): Parameters<SafeToolParams<CreateWatchlistParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<PrivateWatchlistOutput>, CallToolResult> {
        let operation = "create_my_private_watchlist";
        let input = input.parse().map_err(|e| input_error(operation, &e))?;
        self.manage_watchlist(
            operation,
            context,
            PrivateWatchlistChange::Create { name: &input.name },
        )
        .await
    }

    #[tool(
        title = "Rename a private watchlist",
        description = "Rename one owned, private, non-default list on the customer's explicit request with watchlists:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn rename_my_private_watchlist(
        &self,
        Parameters(input): Parameters<SafeToolParams<RenameWatchlistParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<PrivateWatchlistOutput>, CallToolResult> {
        let operation = "rename_my_private_watchlist";
        let input = input.parse().map_err(|e| input_error(operation, &e))?;
        self.manage_watchlist(
            operation,
            context,
            PrivateWatchlistChange::Rename {
                watchlist_id: input.watchlist_id,
                name: &input.name,
            },
        )
        .await
    }

    #[tool(
        title = "Delete a private watchlist",
        description = "Permanently delete one owned, private, non-default list and all memberships in that list on the customer's explicit request. Requires watchlists:write. Other lists keep their memberships.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn delete_my_private_watchlist(
        &self,
        Parameters(input): Parameters<SafeToolParams<DeleteWatchlistParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<PrivateWatchlistOutput>, CallToolResult> {
        let operation = "delete_my_private_watchlist";
        let input = input.parse().map_err(|e| input_error(operation, &e))?;
        self.manage_watchlist(
            operation,
            context,
            PrivateWatchlistChange::Delete {
                watchlist_id: input.watchlist_id,
            },
        )
        .await
    }
}

impl AccountMcpServer {
    async fn manage_watchlist(
        &self,
        operation: &'static str,
        context: RequestContext<RoleServer>,
        change: PrivateWatchlistChange<'_>,
    ) -> Result<Json<PrivateWatchlistOutput>, CallToolResult> {
        let access = request_access(&context, operation)?;
        let key = uuid::Uuid::new_v4().to_string();
        self.execute(
            operation,
            self.client.change_private_watchlist(&access, change, &key),
        )
        .await
    }
}

#[tool_router(router = reminder_router, vis = "pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Set or remove a live reminder",
        description = "On the customer's explicit request, set or remove their personal reminder for one live session with livereminders:write. The readback list needs watchlist:read. Only executed confirms completion.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn set_my_live_reminder(
        &self,
        Parameters(input): Parameters<SafeToolParams<LiveReminderParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<LiveReminderReceiptOutput>, CallToolResult> {
        let input = input.parse().map_err(|e| input_error(REMINDER_TOOL, &e))?;
        let access = request_access(&context, REMINDER_TOOL)?;
        let key = uuid::Uuid::new_v4().to_string();
        self.execute(
            REMINDER_TOOL,
            self.client
                .set_live_reminder(&access, &input.session_id, input.reminded, &key),
        )
        .await
    }
}
