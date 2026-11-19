use super::*;
use crate::{
    output::{SavedSearchAuthoringOutput, SavedSearchReceiptOutput},
    params::{CreateSearchParams, DeleteSearchParams, UpdateSearchParams},
};
use budna_mcp_client::SavedSearchChange;

#[tool_router(router=saved_search_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Check saved-search limits",
        description = "Read how many searches the connected account has saved, its current limit and allowed alert frequencies with watchlist:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_saved_search_authoring(
        &self,
        Parameters(input): Parameters<SafeToolParams<EmptyAccountParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<SavedSearchAuthoringOutput>, CallToolResult> {
        input
            .parse()
            .map_err(|e| input_error(SEARCH_AUTHORING_TOOL, &e))?;
        let access = request_access(&context, SEARCH_AUTHORING_TOOL)?;
        self.execute(
            SEARCH_AUTHORING_TOOL,
            self.client.saved_search_authoring(&access),
        )
        .await
    }
    #[tool(
        title = "Save a marketplace search",
        description = "On the customer's request, save one named search with savedsearches:write. Use taxonomy reads for valid category filters and check the account's search limit. Supply exact decimal money in the saved search market currency.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn create_my_saved_search(
        &self,
        Parameters(input): Parameters<SafeToolParams<WithRequestId<CreateSearchParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<SavedSearchReceiptOutput>, CallToolResult> {
        const OP: &str = "create_my_saved_search";
        let intent = input.parse().map_err(|e| input_error(OP, &e))?;
        let fields = intent.fields.changes().map_err(|e| input_error(OP, &e))?;
        self.execute_saved_search(
            OP,
            context,
            &intent.request_id,
            SavedSearchChange::Create(fields),
        )
        .await
    }
    #[tool(
        title = "Update a saved search or pause its alerts",
        description = "Update only the supplied search settings under savedsearches:write. Read its current updated_at for expected_updated_at. Omitted criteria are preserved. Set active=false to pause or true to resume. This leaves match read state alone.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn update_my_saved_search(
        &self,
        Parameters(input): Parameters<SafeToolParams<WithRequestId<UpdateSearchParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<SavedSearchReceiptOutput>, CallToolResult> {
        const OP: &str = "update_my_saved_search";
        let WithRequestId {
            fields: params,
            request_id,
        } = input.parse().map_err(|e| input_error(OP, &e))?;
        let change = SavedSearchChange::Update {
            search_id: params.search_id,
            expected_updated_at: params.expected_updated_at,
            changes: params.changes().map_err(|e| input_error(OP, &e))?,
        };
        self.execute_saved_search(OP, context, &request_id, change)
            .await
    }
    #[tool(
        title = "Delete one saved search",
        description = "Permanently delete one saved search and its alert configuration on the customer's explicit request with savedsearches:write. Read its current updated_at for expected_updated_at.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn delete_my_saved_search(
        &self,
        Parameters(input): Parameters<SafeToolParams<WithRequestId<DeleteSearchParams>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<SavedSearchReceiptOutput>, CallToolResult> {
        const OP: &str = "delete_my_saved_search";
        let WithRequestId {
            fields: params,
            request_id,
        } = input.parse().map_err(|e| input_error(OP, &e))?;
        self.execute_saved_search(
            OP,
            context,
            &request_id,
            SavedSearchChange::Delete {
                search_id: params.search_id,
                expected_updated_at: params.expected_updated_at,
            },
        )
        .await
    }
}
impl AccountMcpServer {
    async fn execute_saved_search(
        &self,
        op: &'static str,
        context: RequestContext<RoleServer>,
        request_id: &str,
        change: SavedSearchChange,
    ) -> Result<Json<SavedSearchReceiptOutput>, CallToolResult> {
        let access = request_access(&context, op)?;
        self.execute(
            op,
            self.client
                .change_saved_search(&access, &change, request_id),
        )
        .await
    }
}
