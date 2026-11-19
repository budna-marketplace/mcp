use super::*;
use crate::output::parcel::{ReceiptOutput, TermsOutput};
use budna_mcp_client::{ParcelManagement, ParcelSelection};
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReviewParams {
    selection: ParcelSelection,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct RequestParams {
    #[schemars(length(min = 43, max = 43))]
    expected_revision: String,
    #[schemars(range(min = 1))]
    reviewed_at: i64,
    /// Order IDs from the reviewed selection, in ascending order.
    #[schemars(length(min = 2, max = 25))]
    order_ids: Vec<i64>,
    /// Reuse this UUID for the same request after an interrupted response.
    #[schemars(length(min = 36, max = 36))]
    request_id: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ManageParams {
    #[schemars(length(min = 43, max = 43))]
    expected_revision: String,
    #[schemars(length(min = 36, max = 36))]
    request_id: String,
    change: ParcelManagement,
}
#[tool_router(router=parcel_review_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Review my combined parcel",
        description = "Review 2 to 25 distinct unpaid orders from the same seller, or select an existing parcel by parcel ID or a participating order. Read every member, amount, quantity, deadline, shipping terms and available actions. Order IDs must be in ascending order. Requires parcelreview:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_combined_parcel_terms(
        &self,
        Parameters(input): Parameters<SafeToolParams<ReviewParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<TermsOutput>, CallToolResult> {
        const OP: &str = "get_my_combined_parcel_terms";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, async {
            let terms = self.client.parcel_terms(&access, &input.selection).await?;
            Ok((terms, self.public_urls.listing_origin()))
        })
        .await
    }
}
#[tool_router(router=parcel_request_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Request combined shipping",
        description = "Request Budna approval to group exactly the reviewed orders into one parcel. Approval clears their previous delivery selections and can change payment deadlines. Open the supplied Budna confirmation page. The buyer chooses delivery, protection and payment on Budna. Requires parcelrequests:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn request_my_combined_parcel(
        &self,
        Parameters(input): Parameters<SafeToolParams<RequestParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ReceiptOutput>, CallToolResult> {
        const OP: &str = "request_my_combined_parcel";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .request_combined_parcel(
                    &access,
                    &input.expected_revision,
                    input.reviewed_at,
                    &input.order_ids,
                    &input.request_id,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
}
#[tool_router(router=parcel_management_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Change my combined parcel",
        description = "With the current parcel revision, the seller can declare package size and weight with an optional exact SEK price for custom shipping, or decline. The buyer can dissolve an eligible group. Declining or dissolving returns its members to standalone orders. Updating a quote can clear the buyer’s prior shipping selection. Requires parcelmanagement:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn change_my_combined_parcel(
        &self,
        Parameters(input): Parameters<SafeToolParams<ManageParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ReceiptOutput>, CallToolResult> {
        const OP: &str = "change_my_combined_parcel";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .manage_combined_parcel(
                    &access,
                    &input.expected_revision,
                    &input.change,
                    &input.request_id,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
}
