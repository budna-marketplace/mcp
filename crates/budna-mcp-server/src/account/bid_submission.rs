use super::*;
use crate::{
    output::bid_submission::{BidQuoteOutput, BidSubmissionOutput},
    params::bid_submission::{PrepareBidParams, SubmitBidParams},
};

#[tool_router(router=bid_submission_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Prepare my bid",
        description = "Prepare a short-lived quote for one exact bid in SEK. Review the current round, minimum, applicable fees and payment deadline with the customer. Preparing the quote reserves no stock and places no bid. Requires bids:read.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn prepare_my_bid(
        &self,
        Parameters(input): Parameters<SafeToolParams<PrepareBidParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<BidQuoteOutput>, CallToolResult> {
        const OP: &str = "prepare_my_bid";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .prepare_bid(&access, &input.listing_id, &input.bid),
        )
        .await
    }
    #[tool(
        title = "Submit my quoted bid",
        description = "Submit the exact bid the customer requested for one listing and quoted round. A winning bid binds the buyer. Budna applies the connected app’s approval and spending rules. Open the Budna approval page if the receipt requests confirmation. After an interrupted submission, reuse the same quote and amount. Requires bids:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn submit_my_quoted_bid(
        &self,
        Parameters(input): Parameters<SafeToolParams<SubmitBidParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<BidSubmissionOutput>, CallToolResult> {
        const OP: &str = "submit_my_quoted_bid";
        let input = input.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, async {
            let receipt = self
                .client
                .submit_bid(&access, &input.listing_id, &input.quote_id, &input.bid)
                .await?;
            Ok((receipt, self.public_urls.listing_origin()))
        })
        .await
    }
}
