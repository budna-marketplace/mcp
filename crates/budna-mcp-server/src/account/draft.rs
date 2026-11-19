use super::*;
use crate::{
    output::{DraftEligibilityOutput, DraftReceiptOutput},
    params::{DraftParams, EmptyAccountParams},
};
use budna_mcp_client::{DraftInput, DraftVerification};

#[tool_router(router = draft_eligibility_router, vis = "pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Check seller verification for a listing draft",
        description = "Check the connected seller's current verification with profile:read. Open the returned Budna web page when verification is required, then check again. Checking creates no draft.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    pub async fn get_my_listing_draft_eligibility(
        &self,
        Parameters(input): Parameters<SafeToolParams<EmptyAccountParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<DraftEligibilityOutput>, CallToolResult> {
        input
            .parse()
            .map_err(|e| input_error(DRAFT_ELIGIBILITY_TOOL, &e))?;
        let access = request_access(&context, DRAFT_ELIGIBILITY_TOOL)?;
        self.execute(DRAFT_ELIGIBILITY_TOOL, async {
            let eligibility = self.client.draft_eligibility(&access).await?;
            let destination = match eligibility.seller_verification {
                DraftVerification::Satisfied => None,
                DraftVerification::VerifyIdentityOnBudna => Some("/account/verification"),
                DraftVerification::VerifyCompanyOnBudna => Some("/account/wallet?setup=payouts"),
            };
            Ok(DraftEligibilityOutput::new(
                eligibility,
                destination.map(|path| format!("{}{path}", self.public_urls.listing_origin())),
            ))
        })
        .await
    }
}

#[tool_router(router=listing_draft_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Create an unpublished listing draft",
        description = "On the verified seller's explicit request, save one unpublished draft with listingdrafts:write. Select the market and currency. Seller verification takes place in Budna web. To add media use Budna. Only executed confirms a draft was saved.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = true
        )
    )]
    pub async fn create_my_listing_draft(
        &self,
        Parameters(input): Parameters<SafeToolParams<DraftParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<DraftReceiptOutput>, CallToolResult> {
        let input = input
            .parse()
            .map_err(|e| input_error(DRAFT_CREATE_TOOL, &e))?;
        let access = request_access(&context, DRAFT_CREATE_TOOL)?;
        let draft = DraftInput {
            market: input.market.as_str(),
            currency: input.currency.as_str(),
            title: input.title.as_deref(),
            description: input.description.as_deref(),
        };
        self.execute(
            DRAFT_CREATE_TOOL,
            self.client
                .create_draft(&access, &draft, &uuid::Uuid::new_v4().to_string()),
        )
        .await
    }
}
