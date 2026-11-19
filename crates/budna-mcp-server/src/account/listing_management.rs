use super::*;
use crate::output::{DraftReceiptOutput, listing_management::ListingManagementOutput};
use budna_mcp_client::ListingManagementChange;
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ListingChangeParams {
    #[schemars(length(min = 44, max = 44))]
    listing_id: String,
    operation: ListingManagementChange,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct QuotedListingChange {
    #[schemars(length(min = 44, max = 44))]
    listing_id: String,
    operation: ListingManagementChange,
    #[schemars(range(min = 1))]
    expected_revision: i64,
    #[schemars(length(min = 49, max = 49))]
    quote_id: String,
}
#[tool_router(router=listing_management_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Review a change to my listing",
        description = "Preview one change to an owned listing with selling:read. Review fees, buyer commitments, times and watcher notifications before editing, extending, relisting or deleting. Relisting opens a new round when applied. This check changes no listing.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn prepare_my_listing_change(
        &self,
        Parameters(params): Parameters<SafeToolParams<ListingChangeParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ListingManagementOutput>, CallToolResult> {
        const OP: &str = "prepare_my_listing_change";
        let i = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .prepare_listing_change(&access, &i.listing_id, &i.operation),
        )
        .await
    }
    #[tool(
        title = "Apply the reviewed change to my listing",
        description = "On the seller’s explicit request, apply the exact reviewed edit, extension, relist or deletion with listingmanagement:write. Deletion removes the listing. Relisting opens a new round and can notify watchers. Supply the quoted terms and revision and preserve the request ID across retries.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn apply_my_quoted_listing_change(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<QuotedListingChange>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<DraftReceiptOutput>, CallToolResult> {
        const OP: &str = "apply_my_quoted_listing_change";
        let WithRequestId {
            fields: i,
            request_id,
        } = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.apply_listing_change(
                &access,
                &i.listing_id,
                &i.operation,
                i.expected_revision,
                &i.quote_id,
                &request_id,
            ),
        )
        .await
    }
}
