use super::*;
use crate::output::dispute_cases::*;
use budna_mcp_client::CaseDiscount;
use budna_mcp_client::{CaseChange, CaseDecision, CaseTerms};
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct OfferTarget {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(range(min = 1))]
    dispute_id: i64,
    #[schemars(range(min = 1))]
    expected_case_revision: i64,
    #[schemars(range(min = 1))]
    offer_id: i64,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ProposeSettlement {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(range(min = 1))]
    dispute_id: i64,
    #[schemars(range(min = 1))]
    expected_case_revision: i64,
    #[schemars(range(min = 1))]
    expected_order_revision: i64,
    amount: CaseDiscount,
    message: Option<String>,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct AcceptSettlement {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(range(min = 1))]
    dispute_id: i64,
    #[schemars(range(min = 1))]
    expected_case_revision: i64,
    #[schemars(range(min = 1))]
    expected_order_revision: i64,
    #[schemars(range(min = 1))]
    offer_id: i64,
    amount: CaseDiscount,
    #[schemars(range(min = 1))]
    expires_at: i64,
}
#[tool_router(router=dispute_settlements_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Decline my settlement offer",
        description = "As the buyer, decline this specific offer. The dispute stays open. Requires disputesettlements:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn decline_my_dispute_settlement(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<OfferTarget>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<CaseReceiptOutput>, CallToolResult> {
        const OP: &str = "decline_my_dispute_settlement";
        let WithRequestId {
            fields: i,
            request_id,
        } = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .change_dispute(
                    &access,
                    i.order_id,
                    CaseChange::DeclineOffer {
                        dispute_id: i.dispute_id,
                        offer_id: i.offer_id,
                        expected_case_revision: i.expected_case_revision,
                    },
                    &request_id,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
    #[tool(
        title = "Withdraw my settlement proposal",
        description = "As the seller, withdraw this specific outstanding offer. The buyer loses the option to accept it. Requires disputesettlements:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn withdraw_my_settlement_offer(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<OfferTarget>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<CaseReceiptOutput>, CallToolResult> {
        const OP: &str = "withdraw_my_settlement_offer";
        let WithRequestId {
            fields: i,
            request_id,
        } = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .change_dispute(
                    &access,
                    i.order_id,
                    CaseChange::WithdrawOffer {
                        dispute_id: i.dispute_id,
                        offer_id: i.offer_id,
                        expected_case_revision: i.expected_case_revision,
                    },
                    &request_id,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
    #[tool(
        title = "Propose a settlement discount",
        description = "Request Budna confirmation of the exact discount and optional message. If approved, the buyer can accept for 24 hours. Acceptance requests a refund and can resolve the case. Requires disputesettlements:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn propose_my_dispute_settlement(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<ProposeSettlement>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<CaseReceiptOutput>, CallToolResult> {
        const OP: &str = "propose_my_dispute_settlement";
        let WithRequestId {
            fields: i,
            request_id,
        } = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .decide_dispute(
                    &access,
                    CaseTerms {
                        order_id: i.order_id,
                        dispute_id: i.dispute_id,
                        expected_case_revision: i.expected_case_revision,
                        expected_order_revision: i.expected_order_revision,
                        decision: CaseDecision::Propose {
                            amount: i.amount,
                            message: i.message,
                        },
                    },
                    &request_id,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
    #[tool(
        title = "Accept a reviewed settlement",
        description = "Request Budna confirmation of the exact offer, discount, currency and expiration. Acceptance requests the refund and resolves the case. Bank processing can take longer. Requires disputesettlements:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn accept_my_dispute_settlement(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<AcceptSettlement>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<CaseReceiptOutput>, CallToolResult> {
        const OP: &str = "accept_my_dispute_settlement";
        let WithRequestId {
            fields: i,
            request_id,
        } = params.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, async {
            let r = self
                .client
                .decide_dispute(
                    &access,
                    CaseTerms {
                        order_id: i.order_id,
                        dispute_id: i.dispute_id,
                        expected_case_revision: i.expected_case_revision,
                        expected_order_revision: i.expected_order_revision,
                        decision: CaseDecision::Accept {
                            offer_id: i.offer_id,
                            amount: i.amount,
                            expires_at: i.expires_at,
                        },
                    },
                    &request_id,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
}
