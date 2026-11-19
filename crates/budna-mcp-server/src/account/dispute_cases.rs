use super::*;
use crate::output::dispute_cases::*;
use budna_mcp_client::{CaseAudience, CaseReason, CaseRemedy};
use budna_mcp_client::{CaseChange, CaseDecision, CaseTerms};
use rmcp::schemars;
use serde::Deserialize;
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct OpenCase {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(range(min = 1))]
    expected_order_revision: i64,
    expected_listing_id: String,
    reason: CaseReason,
    title: String,
    description: String,
    requested_remedy: Option<CaseRemedy>,
    audience: CaseAudience,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct CaseStatement {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(range(min = 1))]
    dispute_id: i64,
    #[schemars(range(min = 1))]
    expected_case_revision: i64,
    body: String,
    audience: CaseAudience,
}
#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct WithdrawClaim {
    #[schemars(range(min = 1))]
    order_id: i64,
    #[schemars(range(min = 1))]
    dispute_id: i64,
    #[schemars(range(min = 1))]
    expected_case_revision: i64,
    #[schemars(range(min = 1))]
    expected_order_revision: i64,
}
#[tool_router(router=case_options_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Review my dispute options",
        description = "Check the order, party, eligibility, current case, revisions, available settlement, payout hold and deadlines. Read case statements with the existing dispute-detail tool. Requires orders:read.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn get_my_order_dispute_options(
        &self,
        Parameters(p): Parameters<SafeToolParams<AccountOrderIdParams>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<DisputeOptionsOutput>, CallToolResult> {
        const OP: &str = "get_my_order_dispute_options";
        let i = p.parse().map_err(|e| input_error(OP, &e))?;
        let access = request_access(&context, OP)?;
        self.execute(OP, self.client.case_options(&access, i.order_id))
            .await
    }
}
#[tool_router(router=dispute_cases_router,vis="pub(super)")]
impl AccountMcpServer {
    #[tool(
        title = "Open a dispute on my order",
        description = "Open one eligible dispute. Confirm the reason, remedy and exact text. Both parties and Budna can see it. An open case can hold seller payout. Add sensitive evidence directly in Budna. Requires disputecases:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn open_my_order_dispute(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<OpenCase>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<CaseReceiptOutput>, CallToolResult> {
        const OP: &str = "open_my_order_dispute";
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
                    CaseChange::Open {
                        expected_listing_id: i.expected_listing_id,
                        expected_order_revision: i.expected_order_revision,
                        title: i.title,
                        description: i.description,
                        reason: i.reason,
                        requested_remedy: i.requested_remedy,
                        audience: i.audience,
                    },
                    &request_id,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
    #[tool(
        title = "Add my party statement",
        description = "Share the customer’s exact statement with both parties and Budna on one open case. Use the current case revision. Requires disputecases:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn add_my_dispute_statement(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<CaseStatement>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<CaseReceiptOutput>, CallToolResult> {
        const OP: &str = "add_my_dispute_statement";
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
                    CaseChange::Statement {
                        dispute_id: i.dispute_id,
                        expected_case_revision: i.expected_case_revision,
                        body: i.body,
                        audience: i.audience,
                    },
                    &request_id,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
    #[tool(
        title = "Request withdrawal of my dispute",
        description = "Ask Budna to confirm withdrawal of this claim. Approval closes the dispute and can release its payout hold. Review the order and case first. Requires disputecases:write.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = true,
            open_world_hint = true
        )
    )]
    async fn request_withdraw_my_dispute(
        &self,
        Parameters(params): Parameters<SafeToolParams<WithRequestId<WithdrawClaim>>>,
        context: RequestContext<RoleServer>,
    ) -> Result<Json<CaseReceiptOutput>, CallToolResult> {
        const OP: &str = "request_withdraw_my_dispute";
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
                        decision: CaseDecision::WithdrawClaim,
                    },
                    &request_id,
                )
                .await?;
            Ok((r, self.public_urls.listing_origin()))
        })
        .await
    }
}
