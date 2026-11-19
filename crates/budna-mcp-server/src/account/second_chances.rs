use super::*;
use crate::{output::second_chances::*, params::second_chances::*};
use budna_mcp_client::ChanceIntent;
#[tool_router(router=second_chance_sale_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Get my second-chance sale",
        description = "Check whether an unpaid auction order you sold can start a second-chance offer or cancel an active one. Requires orders:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_second_chance_sale(
        &self,
        Parameters(input): Parameters<SafeToolParams<SaleStateParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<SellerStateOutput>, CallToolResult> {
        const OP: &str = "get_my_second_chance_sale";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client.second_chance_sale_state(&a, i.source_order_id),
        )
        .await
    }
    #[tool(
        title = "Prepare my second-chance sale",
        description = "Prepare the price and deadlines to start or cancel one cascade. Budna selects eligible bidders in order. Requires orders:read.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn prepare_my_second_chance_sale(
        &self,
        Parameters(input): Parameters<SafeToolParams<SaleQuote>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<QuoteOutput>, CallToolResult> {
        const OP: &str = "prepare_my_second_chance_sale";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .prepare_second_chance(&a, ChanceIntent::Seller(&i.change)),
        )
        .await
    }
    #[tool(
        title = "Submit my quoted second-chance sale",
        description = "Starting a cascade requires approval in Budna. Cancelling an active cascade withdraws the unaccepted invitation. Reuse the exact quote after an interrupted submission. Requires secondchances:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn submit_my_quoted_second_chance_sale(
        &self,
        Parameters(input): Parameters<SafeToolParams<SubmitSale>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ChanceReceiptOutput>, CallToolResult> {
        const OP: &str = "submit_my_quoted_second_chance_sale";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .submit_second_chance(
                    &a,
                    ChanceIntent::Seller(&i.change),
                    &i.quote_id,
                    &i.terms_digest,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
}
#[tool_router(router=second_chance_answer_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Get my second-chance invitation",
        description = "Look up your active, personal invitation from a received second-chance offer. A buyer can answer only the invitation addressed to them. Requires offers:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_second_chance_invitation(
        &self,
        Parameters(input): Parameters<SafeToolParams<InviteParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<InvitationOutput>, CallToolResult> {
        const OP: &str = "get_my_second_chance_invitation";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, self.client.second_chance_invitation(&a, i.offer_id))
            .await
    }
    #[tool(
        title = "Prepare my second-chance answer",
        description = "Review one invitation and its fixed price, fees, response window and payment deadline before accepting or passing on it. Requires offers:read.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn prepare_my_second_chance_answer(
        &self,
        Parameters(input): Parameters<SafeToolParams<AnswerQuote>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<QuoteOutput>, CallToolResult> {
        const OP: &str = "prepare_my_second_chance_answer";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .prepare_second_chance(&a, ChanceIntent::Buyer(&i.change)),
        )
        .await
    }
    #[tool(
        title = "Submit my quoted second-chance answer",
        description = "Accepting a second-chance invitation requires approval in Budna and creates a binding unpaid order. Declining passes the opportunity to the next eligible bidder. Requires secondchanceanswers:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn submit_my_quoted_second_chance_answer(
        &self,
        Parameters(input): Parameters<SafeToolParams<SubmitAnswer>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ChanceReceiptOutput>, CallToolResult> {
        const OP: &str = "submit_my_quoted_second_chance_answer";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .submit_second_chance(
                    &a,
                    ChanceIntent::Buyer(&i.change),
                    &i.quote_id,
                    &i.terms_digest,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
}
