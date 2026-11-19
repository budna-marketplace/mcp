use super::*;
use crate::output::{DraftReceiptOutput, publication::PublicationReadinessOutput};
use budna_mcp_client::LISTING_ID_PATTERN;
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReadinessParams {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    listing_id: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct PublishParams {
    #[schemars(length(min=44,max=44),regex(pattern=LISTING_ID_PATTERN))]
    listing_id: String,
    #[schemars(range(min = 1))]
    expected_revision: i64,
    #[schemars(
        length(min = 49, max = 49),
        regex(pattern = "^quote_[A-Za-z0-9_-]{43}$")
    )]
    quote_id: String,
    #[schemars(length(min = 36, max = 36))]
    request_id: String,
}
#[tool_router(router=publication_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Check readiness and fees for publishing my draft",
        description = "Check the selected draft, verification, media, eligibility, timing, upfront payment and estimated sale fees/proceeds at the stated sale price. Show any selected priority and credit. Resolve verification, media and upfront payment on Budna before requesting a new quote. Requires selling:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_publication_readiness(
        &self,
        Parameters(input): Parameters<SafeToolParams<ReadinessParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<PublicationReadinessOutput>, CallToolResult> {
        const OP: &str = "get_my_publication_readiness";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .publication_readiness(&access, &input.listing_id),
        )
        .await
    }
    #[tool(
        title = "Publish my quoted draft now",
        description = "Publish the selected eligible draft immediately on the seller’s explicit request. Review its exact revision, quoted fees, selected priority and estimated proceeds first. A changed quote requires renewed review. Reuse the request ID after an interrupted call and check the actual listing state. Requires listingpublication:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn publish_my_quoted_draft(
        &self,
        Parameters(input): Parameters<SafeToolParams<PublishParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<DraftReceiptOutput>, CallToolResult> {
        const OP: &str = "publish_my_quoted_draft";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.publish_draft(
                &access,
                &input.listing_id,
                input.expected_revision,
                &input.quote_id,
                &input.request_id,
            ),
        )
        .await
    }
}
