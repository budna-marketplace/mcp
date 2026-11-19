use super::*;
use crate::{
    output::binding_checkout::{CheckoutQuoteOutput, CheckoutReceiptOutput, CheckoutStateOutput},
    params::ListingIdParams,
};
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ReviewCheckout {
    #[schemars(length(min = 44, max = 44))]
    listing_id: String,
    #[schemars(length(min = 36, max = 36))]
    session_id: String,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct PlaceCheckout {
    #[schemars(length(min = 44, max = 44))]
    listing_id: String,
    #[schemars(length(min = 36, max = 36))]
    session_id: String,
    #[schemars(length(min = 36, max = 36))]
    quote_id: String,
    #[schemars(length(min = 43, max = 43))]
    terms_digest: String,
}
#[tool_router(router=binding_checkout_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Get my Buy Now checkout",
        description = "Read your existing checkout for one listing, including whether delivery needs selecting in Budna or an order already exists. Requires orders:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_binding_checkout(
        &self,
        Parameters(input): Parameters<SafeToolParams<ListingIdParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<CheckoutStateOutput>, CallToolResult> {
        const OP: &str = "get_my_binding_checkout";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, self.client.personal_checkout(&a, &i.listing_id))
            .await
    }
    #[tool(
        title = "Prepare my binding Buy Now order",
        description = "Review selected delivery, item quantity, full price, unpaid fee, deadline and whether the auction ends. Preparing the short-lived quote reserves no stock. Requires orders:read.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    async fn prepare_my_binding_checkout_order(
        &self,
        Parameters(input): Parameters<SafeToolParams<ReviewCheckout>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<CheckoutQuoteOutput>, CallToolResult> {
        const OP: &str = "prepare_my_binding_checkout_order";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(
            OP,
            self.client
                .quote_binding_checkout(&a, &i.listing_id, &i.session_id),
        )
        .await
    }
    #[tool(
        title = "Place my reviewed Buy Now order",
        description = "Request first-party approval to place a binding unpaid order on the selected terms. Approval holds stock, ends a Buy Now auction and can incur a nonpayment fee. Complete payment on Budna. Requires checkoutorders:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn submit_my_quoted_checkout_order(
        &self,
        Parameters(input): Parameters<SafeToolParams<PlaceCheckout>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<CheckoutReceiptOutput>, CallToolResult> {
        const OP: &str = "submit_my_quoted_checkout_order";
        let i = input.parse().map_err(|e| input_error(OP, &e))?;
        let a = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .place_binding_checkout(
                    &a,
                    &i.listing_id,
                    &i.session_id,
                    &i.quote_id,
                    &i.terms_digest,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
}
