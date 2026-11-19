use super::*;
use crate::output::DraftReceiptOutput;
use budna_mcp_client::LISTING_ID_PATTERN;
use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SimilarDraftParams {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    listing_id: String,
    /// The updated_at timestamp from the current owner listing.
    #[schemars(range(min = 1))]
    expected_revision: i64,
    /// A UUID for this requested copy. Reuse it whenever its outcome is uncertain.
    #[schemars(length(min = 36, max = 36))]
    request_id: String,
}

#[tool_router(router=draft_authoring_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Make a similar unpublished draft",
        description = "Copy supported fields from one owned listing into a new unpublished draft. Read the source and its revision, confirm the customer's chosen source and reuse one request ID if interrupted. Media and prior marketplace activity stay with the source. Requires a verified seller and draftauthoring:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn create_my_similar_draft(
        &self,
        Parameters(input): Parameters<SafeToolParams<SimilarDraftParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<DraftReceiptOutput>, CallToolResult> {
        const OP: &str = "create_my_similar_draft";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.create_similar_draft(
                &access,
                &input.listing_id,
                input.expected_revision,
                &input.request_id,
            ),
        )
        .await
    }
}
