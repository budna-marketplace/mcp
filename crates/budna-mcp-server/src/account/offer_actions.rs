use super::*;
use crate::{
    output::offer_actions::{OfferReceiptOutput, ReviewedOfferOutput},
    params::offer_actions::{PrepareOfferParams, SubmitOfferParams},
};
#[tool_router(router=offer_actions_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Prepare my offer change",
        description = "Prepare a short-lived quote to make, raise, counter, lower, accept, reject or withdraw a proposal. Review the total for all units, who can answer, the amounts that remain open, fees and deadlines. Requires offers:read.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn prepare_my_offer_change(
        &self,
        Parameters(input): Parameters<SafeToolParams<PrepareOfferParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<ReviewedOfferOutput>, CallToolResult> {
        const OP: &str = "prepare_my_offer_change";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .prepare_offer_change(&a, &i.listing_id, &i.proposal),
        )
        .await
    }
    #[tool(
        title = "Submit my quoted offer change",
        description = "Apply the exact reviewed change to one negotiation. A proposal can bind its sender if accepted. Accepting reserves the units and creates an unpaid order. Reuse the same quote when checking an interrupted submission. Requires offers:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn submit_my_quoted_offer_change(
        &self,
        Parameters(input): Parameters<SafeToolParams<SubmitOfferParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<OfferReceiptOutput>, CallToolResult> {
        const OP: &str = "submit_my_quoted_offer_change";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .submit_offer_change(&a, &i.listing_id, &i.quote_id, &i.terms_digest, &i.proposal)
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
}
