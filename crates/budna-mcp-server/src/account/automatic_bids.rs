use super::*;
use crate::{
    output::automatic_bids::{
        PauseMaximumOutput, SetMaximumOutput, StandingBidQuoteOutput, StandingBidStateOutput,
    },
    params::automatic_bids::{
        PauseMaximumParams, PrepareMaximumParams, SetMaximumParams, StandingParams,
    },
};
#[tool_router(router=automatic_bids_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Get my automatic bid state",
        description = "Read your standing maximum on one auction and the revision needed to pause it. Only your own maximum is visible. Requires bids:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_automatic_bid_state(
        &self,
        Parameters(input): Parameters<SafeToolParams<StandingParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<StandingBidStateOutput>, CallToolResult> {
        const OP: &str = "get_my_automatic_bid_state";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, self.client.standing_bid(&a, &i.listing_id))
            .await
    }
    #[tool(
        title = "Prepare my automatic bid maximum",
        description = "Prepare a short-lived quote to set or raise one private automatic maximum. Review the maximum, applicable fees, round, current mandate and possible immediate bid with the buyer. Requires bids:read.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn prepare_my_automatic_bid_maximum(
        &self,
        Parameters(input): Parameters<SafeToolParams<PrepareMaximumParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<StandingBidQuoteOutput>, CallToolResult> {
        const OP: &str = "prepare_my_automatic_bid_maximum";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.prepare_maximum(&a, &i.listing_id, &i.maximum),
        )
        .await
    }
    #[tool(
        title = "Set my quoted automatic bid maximum",
        description = "Authorize Budna to bid up to the requested private maximum on this quoted round. Setting or raising a maximum can place a binding bid immediately. Approval and spending limits apply to the full maximum. Requires autobids:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn set_my_quoted_automatic_bid_maximum(
        &self,
        Parameters(input): Parameters<SafeToolParams<SetMaximumParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<SetMaximumOutput>, CallToolResult> {
        const OP: &str = "set_my_quoted_automatic_bid_maximum";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, async {
            let result = self
                .client
                .set_maximum(&a, &i.listing_id, &i.quote_id, &i.maximum)
                .await?;
            Ok((result, self.public_urls.listing_origin()))
        })
        .await
    }
    #[tool(
        title = "Pause my automatic bid",
        description = "Pause future automatic bids for the reviewed standing instruction. Existing bids stay binding. Reuse the same idempotency key after an interrupted request. Requires autobids:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn pause_my_automatic_bid(
        &self,
        Parameters(input): Parameters<SafeToolParams<PauseMaximumParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<PauseMaximumOutput>, CallToolResult> {
        const OP: &str = "pause_my_automatic_bid";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .pause_maximum(&a, &i.listing_id, &i.expected_revision, &i.idempotency_key),
        )
        .await
    }
}
