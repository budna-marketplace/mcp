use super::*;
use crate::{
    output::personal::{OrderReviewFactsOutput, OrderReviewReceiptOutput},
    params::personal::RateOrderParams,
};
#[tool_router(router=order_review_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Check who I can rate for an order",
        description = "Read the other party’s public handle when available, the listing, review eligibility and audience for one of your orders with orders:read. The review can become visible on the marketplace.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_order_rating_eligibility(
        &self,
        Parameters(input): Parameters<SafeToolParams<AccountOrderIdParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<OrderReviewFactsOutput>, CallToolResult> {
        const OP: &str = "get_my_order_rating_eligibility";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, self.client.order_review_facts(&access, input.order_id))
            .await
    }
    #[tool(
        title = "Rate the other party on one order",
        description = "Submit the stars and exact comment the customer chose for one eligible order. For a star-only rating, explicitly pass null. Reviews can be public and Budna notifies the other party. Confirm the order, audience and stars. Requires ratings:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn rate_my_order_counterparty(
        &self,
        Parameters(input): Parameters<SafeToolParams<RateOrderParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<OrderReviewReceiptOutput>, CallToolResult> {
        const OP: &str = "rate_my_order_counterparty";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.rate_order(
                &access,
                input.order_id,
                input.stars,
                input.comment.as_deref(),
                input.audience,
                &uuid::Uuid::new_v4().to_string(),
            ),
        )
        .await
    }
}
