use super::*;
use crate::output::order_closure::*;
use budna_mcp_client::{CloseOrderChoice, OrderClosureMove};
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct OrderRef {
    #[schemars(range(min = 1))]
    order_id: i64,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ClosureReview {
    #[schemars(range(min = 1))]
    order_id: i64,
    operation: CloseOrderChoice,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct SubmitClosure {
    #[schemars(range(min = 1))]
    order_id: i64,
    operation: CloseOrderChoice,
    #[schemars(length(min = 36, max = 36))]
    quote_id: String,
    #[schemars(length(min = 43, max = 43))]
    terms_digest: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct RelistReview {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(length(min = 36, max = 36))]
    quote_id: String,
    #[schemars(length(min = 43, max = 43))]
    terms_digest: String,
}
#[tool_router(router=order_closure_state_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Get my order closure",
        description = "Read your order status, payment deadline and whether an unpaid closure or its separate relist has completed. Requires orders:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_order_closure(
        &self,
        Parameters(input): Parameters<SafeToolParams<OrderRef>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ClosureStateOutput>, CallToolResult> {
        const OP: &str = "get_my_order_closure";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, self.client.order_closure_state(&a, i.order_id))
            .await
    }
}
#[tool_router(router=order_closure_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Prepare my order cancellation or overdue close",
        description = "Review who owes any fee, what will be refunded or retained, stock restoration and the payment deadline. A consumer withdrawing from a funded business order uses the return tools. Requires orders:read.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn prepare_my_order_closure(
        &self,
        Parameters(input): Parameters<SafeToolParams<ClosureReview>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ClosureQuoteOutput>, CallToolResult> {
        const OP: &str = "prepare_my_order_closure";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .quote_order_closure(&a, i.order_id, i.operation.into()),
        )
        .await
    }
    #[tool(
        title = "Submit my reviewed order closure",
        description = "Request approval in Budna for the exact cancellation or overdue seller close. The approved step can charge a fee or request a refund. Read the same order after approval or an interrupted request. Requires orderclosures:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn submit_my_quoted_order_closure(
        &self,
        Parameters(input): Parameters<SafeToolParams<SubmitClosure>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ClosureReceiptOutput>, CallToolResult> {
        const OP: &str = "submit_my_quoted_order_closure";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .submit_order_closure(
                    &a,
                    i.order_id,
                    i.operation.into(),
                    &i.quote_id,
                    &i.terms_digest,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
}
#[tool_router(router=closed_relist_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Prepare to relist my closed unpaid sale",
        description = "Review the original listing price and duration after your unpaid order has closed. Requires orders:read.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn prepare_my_closed_order_relist(
        &self,
        Parameters(input): Parameters<SafeToolParams<OrderRef>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ClosureQuoteOutput>, CallToolResult> {
        const OP: &str = "prepare_my_closed_order_relist";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .quote_order_closure(&a, i.order_id, OrderClosureMove::RelistClosedOrder),
        )
        .await
    }
    #[tool(
        title = "Relist my reviewed closed unpaid sale",
        description = "Put the eligible listing back on sale on the reviewed terms. If the order already closed and a relist failed, request a new relist quote for the closed order. Requires ordersrelist:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn relist_my_quoted_closed_order(
        &self,
        Parameters(input): Parameters<SafeToolParams<RelistReview>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ClosureReceiptOutput>, CallToolResult> {
        const OP: &str = "relist_my_quoted_closed_order";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .submit_order_closure(
                    &a,
                    i.order_id,
                    OrderClosureMove::RelistClosedOrder,
                    &i.quote_id,
                    &i.terms_digest,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
}
