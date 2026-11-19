use super::*;
use crate::output::returns::{ReceiptOutput, TermsOutput};
use budna_mcp_client::ReturnChange;
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReviewParams {
    #[schemars(range(min = 1))]
    order_id: i64,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ChangeParams {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(length(min = 43, max = 43))]
    expected_revision: String,
    change: ReturnChange,
    #[schemars(length(min = 36, max = 36))]
    request_id: String,
}
#[tool_router(router=return_review_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Review my order return",
        description = "Read the participating account’s return rights, current refundable amount, who handles the refund, return postage and the order link for first-party label booking. Requires returnreview:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_return_terms(
        &self,
        Parameters(input): Parameters<SafeToolParams<ReviewParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<TermsOutput>, CallToolResult> {
        const OP: &str = "get_my_return_terms";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, async {
            let q = self.client.return_terms(&access, input.order_id).await?;
            Ok((q, self.public_urls.listing_origin()))
        })
        .await
    }
}
#[tool_router(router=return_change_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Change my order return",
        description = "Use the current revision for one eligible order return action. Withdrawing from a qualifying purchase, accepting a return and confirming receipt require approval in Budna. The buyer can supply or correct carrier tracking or cancel an eligible return directly. Label payments, booking and evidence stay in Budna. Reuse the request ID after an interrupted response. Requires returns:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn change_my_order_return(
        &self,
        Parameters(input): Parameters<SafeToolParams<ChangeParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ReceiptOutput>, CallToolResult> {
        const OP: &str = "change_my_order_return";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .change_return(
                    &access,
                    input.order_id,
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
